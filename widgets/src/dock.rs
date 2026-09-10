use crate::{
    makepad_derive_widget::*,
    makepad_draw::*,
    makepad_micro_serde::*,
    splitter::{Splitter, SplitterAction, SplitterAlign, SplitterAxis},
    tab::Tab,
    tab_bar::{TabBar, TabBarAction},
    widget::*,
    widget_tree::CxWidgetExt,
};
use std::collections::{HashMap, HashSet};

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*

    mod.widgets.DrawRoundCorner = set_type_default() do #(DrawRoundCorner::script_shader(vm)){
        ..mod.draw.DrawQuad
        border_radius: 20.
        flip: vec2(0.0, 0.0)
    }

    // Register DockItem enum variants for DSL parsing (prefixed to avoid conflict with widgets)
    mod.widgets.DockSplitter = #(DockItemSplitter::script_api(vm))
    mod.widgets.DockTabs = #(DockItemTabs::script_api(vm))
    mod.widgets.DockTab = #(DockItemTab::script_api(vm))

    mod.widgets.DockBase = #(Dock::register_widget(vm))

    mod.widgets.Dock = set_type_default() do mod.widgets.DockBase{
        flow: Down

        tab_bar: TabBarGradientY{}
        splitter: Splitter{}

        padding: Inset{left: theme.dock_border_size, top: 0, right: theme.dock_border_size, bottom: theme.dock_border_size}

        round_corner +: {
            border_radius: 20.
            color: instance(theme.color_bg_app)
            flip: vec2(0.0, 0.0)

            pixel: fn() {
                let pos = vec2(
                    mix(self.pos.x, 1.0 - self.pos.x, self.flip.x)
                    mix(self.pos.y, 1.0 - self.pos.y, self.flip.y)
                )

                let sdf = Sdf2d.viewport(pos * self.rect_size)
                sdf.rect(-10., -10., self.rect_size.x * 2.0, self.rect_size.y * 2.0)
                sdf.box(
                    0.25
                    0.25
                    self.rect_size.x * 2.0
                    self.rect_size.y * 2.0
                    4.0
                )

                sdf.subtract()

                sdf.fill(self.color)
                return sdf.result
            }
        }
        drag_target_preview +: {
            draw_depth: 10.0
            color: theme.color_drag_target_preview
        }
    }

    mod.widgets.DockFlat = mod.widgets.DockBase{
        flow: Down

        tab_bar: TabBarFlat{}
        splitter: Splitter{}

        padding: Inset{left: theme.dock_border_size, top: 0, right: theme.dock_border_size, bottom: theme.dock_border_size}

        round_corner +: {
            border_radius: 20.

            pixel: fn() {
                let pos = vec2(
                    mix(self.pos.x, 1.0 - self.pos.x, self.flip.x)
                    mix(self.pos.y, 1.0 - self.pos.y, self.flip.y)
                )

                let sdf = Sdf2d.viewport(pos * self.rect_size)
                sdf.rect(-10., -10., self.rect_size.x * 2.0, self.rect_size.y * 2.0)
                sdf.box(
                    0.25
                    0.25
                    self.rect_size.x * 2.0
                    self.rect_size.y * 2.0
                    4.0
                )

                sdf.subtract()
                return sdf.fill(theme.color_bg_app)
            }
        }

        drag_target_preview +: {
            draw_depth: 10.0
            color: theme.color_drag_target_preview
        }
    }
}

#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawRoundCorner {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    draw_super: DrawQuad,
    #[live]
    border_radius: f32,
    #[live]
    flip: Vec2f,
}

impl DrawRoundCorner {
    fn draw_corners(&mut self, cx: &mut Cx2d, rect: Rect) {
        self.flip = vec2(0.0, 0.0);
        let rad = dvec2(self.border_radius as f64, self.border_radius as f64);
        let pos = rect.pos;
        let size = rect.size;
        self.draw_abs(cx, Rect { pos, size: rad });
        self.flip = vec2(1.0, 0.0);
        self.draw_abs(
            cx,
            Rect {
                pos: pos + dvec2(size.x - rad.x, 0.),
                size: rad,
            },
        );
        self.flip = vec2(1.0, 1.0);
        self.draw_abs(
            cx,
            Rect {
                pos: pos + dvec2(size.x - rad.x, size.y - rad.y),
                size: rad,
            },
        );
        self.flip = vec2(0.0, 1.0);
        self.draw_abs(
            cx,
            Rect {
                pos: pos + dvec2(0., size.y - rad.y),
                size: rad,
            },
        );
    }
}

#[derive(Script, WidgetRegister, WidgetRef, WidgetSet)]
pub struct Dock {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[rust]
    draw_state: DrawStateWrap<Vec<DrawStackItem>>,
    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,
    #[live]
    drop_target_draw_list: DrawList2d,
    #[live]
    round_corner: DrawRoundCorner,
    #[live]
    drag_target_preview: DrawColor,
    #[live]
    ghost_tab_draw_list: DrawList2d,

    #[live]
    tab_bar: ScriptObjectRef,
    #[live]
    splitter: ScriptObjectRef,

    #[rust]
    needs_save: bool,
    #[rust]
    area: Area,

    #[rust]
    tab_bars: ComponentMap<LiveId, TabBarWrap>,
    #[rust]
    splitters: ComponentMap<LiveId, Splitter>,

    #[rust]
    dock_items: HashMap<LiveId, DockItem>,
    #[rust]
    templates: HashMap<LiveId, ScriptObjectRef>,
    #[rust]
    items: ComponentMap<LiveId, (LiveId, WidgetRef)>,
    #[rust]
    drop_state: Option<DropPosition>,
    /// Info about the tab currently being dragged, for ghost tab rendering.
    #[rust]
    dragging_tab: Option<DraggingTab>,
    #[rust]
    dock_item_iter_stack: Vec<(LiveId, usize)>,
    /// Monotonic counter for drag/drop-allocated container IDs. Lives in the
    /// reserved range above INTERNAL_ID_FLOOR so it can't collide with DSL hashes.
    #[rust]
    next_internal_id: u64,
}

/// Floor for internally-generated dock-item IDs. Name-hash LiveIds are 46 bits,
/// so bit 47 keeps our generated IDs safely out of their range.
const INTERNAL_ID_FLOOR: u64 = 1 << 47;

/// Holds a clone of the tab being dragged so we can render it as a ghost overlay.
struct DraggingTab {
    cursor: Vec2d,
    name: String,
    /// The size of the original tab, used for fixed-size rendering.
    size: Vec2d,
    ghost: Tab,
}

impl ScriptHook for Dock {
    fn on_before_apply(
        &mut self,
        _vm: &mut ScriptVm,
        apply: &Apply,
        _scope: &mut Scope,
        _value: ScriptValue,
    ) {
        if apply.is_reload() {
            self.templates.clear();
        }
        if apply.is_new() {
            self.dock_items.clear();
        }
    }

    fn on_after_apply(
        &mut self,
        vm: &mut ScriptVm,
        apply: &Apply,
        scope: &mut Scope,
        value: ScriptValue,
    ) {
        // Collect templates and dock items from the object's vec. Only
        // during template applies (not eval) to avoid storing temporaries.
        //
        // On `Apply::Reload` we re-collect content templates — that's
        // the point of reload: pick up freshly-evaluated `mod.widgets.*`
        // entries. But dock-item entries (Splitter / Tabs / Tab) describe
        // the *layout*, which at runtime may differ from the DSL defaults
        // (tabs opened, splitters moved, etc.). Re-inserting from the DSL
        // would clobber that runtime state and wipe the whole dock. On
        // reload we therefore only insert dock items for IDs that don't
        // already exist — allowing LiveEdit to add new default items while
        // preserving runtime state for existing ones.
        let is_reload = apply.is_reload();
        if !apply.is_eval() {
            if let Some(obj) = value.as_object() {
                vm.vec_with(obj, |vm, vec| {
                    for kv in vec {
                        if let Some(id) = kv.key.as_id() {
                            // Check type and parse accordingly
                            if let Some(val_obj) = kv.value.as_object() {
                                if vm.bx.heap.type_matches_id(
                                    val_obj,
                                    DockItemSplitter::script_type_id_static(),
                                ) {
                                    if !is_reload || !self.dock_items.contains_key(&id) {
                                        let splitter =
                                            DockItemSplitter::script_from_value(vm, kv.value);
                                        self.dock_items.insert(id, splitter.to_dock_item());
                                    }
                                } else if vm
                                    .bx
                                    .heap
                                    .type_matches_id(val_obj, DockItemTabs::script_type_id_static())
                                {
                                    if !is_reload || !self.dock_items.contains_key(&id) {
                                        let tabs = DockItemTabs::script_from_value(vm, kv.value);
                                        self.dock_items.insert(id, tabs.to_dock_item());
                                    }
                                } else if vm
                                    .bx
                                    .heap
                                    .type_matches_id(val_obj, DockItemTab::script_type_id_static())
                                {
                                    if !is_reload || !self.dock_items.contains_key(&id) {
                                        let tab = DockItemTab::script_from_value(vm, kv.value);
                                        self.dock_items.insert(id, tab.to_dock_item());
                                    }
                                } else {
                                    // Not a dock item, treat as content template - root it
                                    self.templates
                                        .insert(id, vm.bx.heap.new_object_ref(val_obj));
                                }
                            }
                            // Non-object values can't be rooted, skip them for templates
                        }
                    }
                });
            }
        }

        // Update existing items if templates changed
        if apply.is_reload() {
            for (kind, widget) in self.items.values_mut() {
                if let Some(template_ref) = self.templates.get(kind) {
                    let template_value: ScriptValue = template_ref.as_object().into();
                    widget.script_apply(vm, apply, scope, template_value);
                }
            }

            // Update tab_bars with the tab_bar template
            if !self.tab_bar.is_zero() {
                for tab_bar in self.tab_bars.values_mut() {
                    tab_bar
                        .tab_bar
                        .script_apply(vm, apply, scope, self.tab_bar.as_object().into());
                }
            }

            // Update splitters with the splitter template
            if !self.splitter.is_zero() {
                for splitter in self.splitters.values_mut() {
                    splitter.script_apply(vm, apply, scope, self.splitter.as_object().into());
                }
            }
        }

        // Create items for all tabs if this is new
        if apply.is_new() {
            self.create_all_items_with_vm(vm);
        }
        vm.cx_mut().widget_tree_mark_dirty(self.uid);
    }
}

impl WidgetNode for Dock {
    fn widget_uid(&self) -> WidgetUid {
        self.uid
    }
    fn walk(&mut self, _cx: &mut Cx) -> Walk {
        self.walk
    }
    fn area(&self) -> Area {
        self.area
    }

    fn children(&self, visit: &mut dyn FnMut(LiveId, WidgetRef)) {
        for (id, (_, widget)) in self.items.iter() {
            visit(*id, widget.clone());
        }
        // The tabs are widgets too (the design tweaker picks and styles
        // them); the bars that own them are not, so they surface here.
        for (_, tab_bar) in self.tab_bars.iter() {
            for (id, tab) in tab_bar.tab_bar.tab_refs() {
                visit(id, tab);
            }
        }
    }

    fn interaction_child_visibility(&self, visit: &mut dyn FnMut(WidgetUid, bool)) -> bool {
        let layout = self.interaction_layout();
        let mut available = true;
        let mut restrict = |widget: &WidgetRef, visible| {
            if let Some(uid) = widget.try_widget_uid() {
                visit(uid, visible);
            } else if !widget.is_empty() {
                // An actively borrowed child has no readable UID. Do not let
                // the default-allowed edge turn it into an actionable tab.
                available = false;
            }
        };
        for (id, (_, body)) in self.items.iter() {
            restrict(body, layout.content.contains(id));
        }
        for (bar_id, bar) in self.tab_bars.iter() {
            for (tab_id, header) in bar.tab_bar.raw_tab_refs() {
                let current = matches!(self.dock_items.get(bar_id),
                    Some(DockItem::Tabs { tabs, .. }) if tabs.contains(&tab_id));
                let is_tab = matches!(self.dock_items.get(&tab_id), Some(DockItem::Tab { .. }));
                restrict(&header, layout.bars.contains(bar_id) && current && is_tab);
            }
        }
        available
    }

    fn redraw(&mut self, cx: &mut Cx) {
        self.area.redraw(cx);
        // A redraw of the dock is a redraw of everything it shows. Each
        // panel's tab content lives behind a RETAINED draw list
        // (`contents_draw_list`, gated with `is_redrawing()` in draw), so
        // marking only the dock's own area leaves every tab's content cached:
        // an app-level `ui.redraw(cx)` would repaint the dock chrome while
        // the panels inside — viewports included — kept showing stale
        // pixels. Mark the retained lists and recurse into the items so the
        // redraw contract (a widget redraws its whole subtree) holds.
        for (_, tab_bar) in self.tab_bars.iter_mut() {
            tab_bar.contents_draw_list.redraw(cx);
        }
        for (_, (_, item)) in self.items.iter_mut() {
            item.redraw(cx);
        }
    }
}

pub struct DockVisibleItemIterator<'a> {
    stack: &'a mut Vec<(LiveId, usize)>,
    dock_items: &'a HashMap<LiveId, DockItem>,
    items: &'a ComponentMap<LiveId, (LiveId, WidgetRef)>,
}

impl<'a> Iterator for DockVisibleItemIterator<'a> {
    type Item = (LiveId, WidgetRef);
    fn next(&mut self) -> Option<Self::Item> {
        while let Some((item_id, index)) = self.stack.pop() {
            if let Some(dock_item) = self.dock_items.get(&item_id) {
                match dock_item {
                    DockItem::Splitter { a, b, .. } => {
                        if index == 0 {
                            self.stack.push((item_id, 1));
                            self.stack.push((*a, 0));
                        } else {
                            self.stack.push((*b, 0));
                        }
                    }
                    DockItem::Tabs { tabs, selected, .. } => {
                        if let Some(tab_id) = tabs.get(*selected) {
                            self.stack.push((*tab_id, 0));
                        }
                    }
                    DockItem::Tab { .. } => {
                        if let Some((_, widget)) = self.items.get(&item_id) {
                            return Some((item_id, widget.clone()));
                        }
                    }
                }
            }
        }
        None
    }
}

struct TabBarWrap {
    tab_bar: TabBar,
    contents_draw_list: DrawList2d,
    contents_rect: Rect,
}

#[derive(Copy, Debug, Clone)]
enum DrawStackItem {
    Invalid,
    SplitLeft { id: LiveId },
    SplitRight { id: LiveId },
    SplitEnd { id: LiveId },
    Tabs { id: LiveId },
    TabLabel { id: LiveId, index: usize },
    Tab { id: LiveId },
    TabContent { id: LiveId },
}

impl DrawStackItem {
    fn from_dock_item(id: LiveId, dock_item: Option<&DockItem>) -> Self {
        match dock_item {
            None => DrawStackItem::Invalid,
            Some(DockItem::Splitter { .. }) => DrawStackItem::SplitLeft { id },
            Some(DockItem::Tabs { .. }) => DrawStackItem::Tabs { id },
            Some(DockItem::Tab { .. }) => DrawStackItem::Tab { id },
        }
    }
}

#[derive(Clone, Debug, Default)]
pub enum DockAction {
    SplitPanelChanged {
        panel_id: LiveId,
        axis: SplitterAxis,
        align: SplitterAlign,
    },
    TabWasPressed(LiveId),
    TabCloseWasPressed(LiveId),
    ShouldTabStartDrag(LiveId),
    Drag(DragHitEvent),
    Drop(DropHitEvent),
    #[default]
    None,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DropPosition {
    part: DropPart,
    rect: Rect,
    id: LiveId,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DropPart {
    Left,
    Right,
    Top,
    Bottom,
    Center,
    TabBar,
    Tab,
}

/// DSL-parseable wrapper for DockItem::Splitter
#[derive(Script, ScriptHook, Default)]
pub struct DockItemSplitter {
    #[source]
    source: ScriptObjectRef,
    #[live]
    pub axis: SplitterAxis,
    #[live]
    pub align: SplitterAlign,
    #[live]
    pub a: LiveId,
    #[live]
    pub b: LiveId,
}

impl DockItemSplitter {
    pub fn to_dock_item(&self) -> DockItem {
        DockItem::Splitter {
            axis: self.axis,
            align: self.align,
            a: self.a,
            b: self.b,
        }
    }
}

/// DSL-parseable wrapper for DockItem::Tabs
#[derive(Script, ScriptHook, Default)]
pub struct DockItemTabs {
    #[source]
    source: ScriptObjectRef,
    #[live]
    pub tabs: Vec<LiveId>,
    #[live]
    pub selected: usize,
    #[live(true)]
    pub closable: bool,
    #[live]
    pub hide_tab_bar: bool,
}

impl DockItemTabs {
    pub fn to_dock_item(&self) -> DockItem {
        DockItem::Tabs {
            tabs: self.tabs.clone(),
            selected: self.selected,
            closable: self.closable,
            hide_tab_bar: self.hide_tab_bar,
        }
    }
}

/// DSL-parseable wrapper for DockItem::Tab
#[derive(Script, ScriptHook, Default)]
pub struct DockItemTab {
    #[source]
    source: ScriptObjectRef,
    #[live]
    pub name: String,
    #[live]
    pub template: LiveId,
    #[live]
    pub kind: LiveId,
}

impl DockItemTab {
    pub fn to_dock_item(&self) -> DockItem {
        DockItem::Tab {
            name: self.name.clone(),
            template: self.template,
            kind: self.kind,
        }
    }
}

#[derive(Clone, Debug, SerRon, DeRon)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DockItem {
    Splitter {
        axis: SplitterAxis,
        align: SplitterAlign,
        a: LiveId,
        b: LiveId,
    },
    Tabs {
        tabs: Vec<LiveId>,
        selected: usize,
        closable: bool,
        #[cfg_attr(feature = "serde", serde(default))]
        hide_tab_bar: bool,
    },
    Tab {
        name: String,
        template: LiveId,
        kind: LiveId,
    },
}

impl Default for DockItem {
    fn default() -> Self {
        DockItem::Tab {
            name: "Tab".to_string(),
            template: id!(PermanentTab),
            kind: LiveId(0),
        }
    }
}

impl DockItem {
    pub fn splitter(axis: SplitterAxis, align: SplitterAlign, a: LiveId, b: LiveId) -> Self {
        DockItem::Splitter { axis, align, a, b }
    }

    pub fn tabs(tabs: Vec<LiveId>, selected: usize, closable: bool) -> Self {
        DockItem::Tabs {
            tabs,
            selected,
            closable,
            hide_tab_bar: false,
        }
    }

    pub fn tab(name: String, kind: LiveId, template: LiveId) -> Self {
        DockItem::Tab {
            name,
            template,
            kind,
        }
    }
}

fn preserve_item_for_layout(
    dock_items: &HashMap<LiveId, DockItem>,
    id: LiveId,
    old_kind: LiveId,
) -> bool {
    match dock_items.get(&id) {
        Some(DockItem::Tab { kind, .. }) => *kind == old_kind,
        // Not in this layout: retain it as a dormant tab body.
        None => true,
        // A container took this ID, so it cannot also name a tab.
        Some(_) => false,
    }
}

#[derive(Clone, Debug, Default)]
pub struct DockCompactDump {
    pub tabs: Vec<DockCompactTabsInfo>,
    pub tab_headers: Vec<DockCompactTabInfo>,
}

#[derive(Default)]
struct DockInteractionLayout {
    content: HashSet<LiveId>,
    bars: HashSet<LiveId>,
}

pub(crate) struct DockInteractionDump {
    // Retained inspection metadata, clipped geometry, and layout eligibility.
    pub tabs: Vec<(DockCompactTabsInfo, Option<Rect>, bool)>,
    pub tab_headers: Vec<(DockCompactTabInfo, Option<Rect>, bool)>,
}

#[derive(Clone, Debug)]
pub struct DockCompactTabsInfo {
    pub tabs_id: LiveId,
    pub selected_tab_id: Option<LiveId>,
    pub tab_count: usize,
    pub rect: Rect,
}

#[derive(Clone, Debug)]
pub struct DockCompactTabInfo {
    pub tabs_id: LiveId,
    pub tab_id: LiveId,
    pub is_active: bool,
    pub title: String,
    pub rect: Rect,
}

impl Dock {
    pub fn unique_id(&self, base: u64) -> LiveId {
        let mut id = LiveId(base);
        let mut i = 0u32;
        while self.dock_items.get(&id).is_some() {
            id = id.bytes_append(&i.to_be_bytes());
            i += 1;
        }
        id
    }

    /// Hands out a fresh LiveId for a drag/drop-created Tabs or Splitter.
    /// First call after load_state scans the loaded state to seed past the max
    /// existing ID, so subsequent allocations can't collide.
    fn next_internal_id(&mut self) -> LiveId {
        if self.next_internal_id < INTERNAL_ID_FLOOR {
            let max = self
                .dock_items
                .keys()
                .map(|k| k.0)
                .filter(|v| *v >= INTERNAL_ID_FLOOR)
                .max();
            self.next_internal_id = max.map(|m| m + 1).unwrap_or(INTERNAL_ID_FLOOR);
        }
        let id = LiveId(self.next_internal_id);
        self.next_internal_id += 1;
        id
    }

    pub fn compact_dump(&self, cx: &Cx) -> DockCompactDump {
        let mut tabs = Vec::new();
        let mut tab_headers = Vec::new();
        let mut tabs_ids = Vec::new();
        tabs_ids.extend(self.tab_bars.keys().copied());
        tabs_ids.sort_by_key(|id| id.0);

        for tabs_id in tabs_ids {
            let Some(DockItem::Tabs {
                tabs: tab_ids,
                selected,
                ..
            }) = self.dock_items.get(&tabs_id)
            else {
                continue;
            };
            let Some(tab_bar) = self.tab_bars.get(&tabs_id) else {
                continue;
            };

            let bar_rect = tab_bar.tab_bar.bar_rect(cx);
            tabs.push(DockCompactTabsInfo {
                tabs_id,
                selected_tab_id: tab_ids.get(*selected).copied(),
                tab_count: tab_ids.len(),
                rect: bar_rect,
            });

            for (index, tab_id) in tab_ids.iter().enumerate() {
                let Some(tab_rect) = tab_bar.tab_bar.tab_rect(cx, *tab_id) else {
                    continue;
                };
                let title = match self.dock_items.get(tab_id) {
                    Some(DockItem::Tab { name, .. }) => name.clone(),
                    _ => String::new(),
                };
                tab_headers.push(DockCompactTabInfo {
                    tabs_id,
                    tab_id: *tab_id,
                    is_active: index == *selected,
                    title,
                    rect: tab_rect,
                });
            }
        }

        DockCompactDump { tabs, tab_headers }
    }

    fn interaction_layout(&self) -> DockInteractionLayout {
        let mut layout = DockInteractionLayout::default();
        let mut seen = HashSet::new();
        let mut pending = vec![id!(root)];
        while let Some(id) = pending.pop() {
            if !seen.insert(id) {
                continue;
            }
            match self.dock_items.get(&id) {
                Some(DockItem::Splitter { a, b, .. }) => {
                    pending.push(*b);
                    pending.push(*a);
                }
                Some(DockItem::Tabs { tabs, selected, hide_tab_bar, .. }) => {
                    if !hide_tab_bar {
                        layout.bars.insert(id);
                    }
                    if let Some(tab) = tabs.get(*selected) {
                        if matches!(self.dock_items.get(tab), Some(DockItem::Tab { .. })) {
                            pending.push(*tab);
                        }
                    }
                }
                Some(DockItem::Tab { .. }) => { layout.content.insert(id); }
                None => {}
            }
        }
        layout
    }

    pub(crate) fn interaction_dump(&self, cx: &Cx) -> DockInteractionDump {
        let layout = self.interaction_layout();
        let dump = self.compact_dump(cx);
        DockInteractionDump {
            tabs: dump.tabs.into_iter().map(|tabs| {
                let rect = self.tab_bars.get(&tabs.tabs_id)
                    .and_then(|bar| bar.tab_bar.interaction_bar_rect(cx));
                let eligible = layout.bars.contains(&tabs.tabs_id);
                (tabs, rect, eligible)
            }).collect(),
            tab_headers: dump.tab_headers.into_iter().map(|tab| {
                let rect = self.tab_bars.get(&tab.tabs_id)
                    .and_then(|bar| bar.tab_bar.interaction_tab_rect(cx, tab.tab_id));
                let eligible = layout.bars.contains(&tab.tabs_id)
                    && matches!(self.dock_items.get(&tab.tab_id), Some(DockItem::Tab { .. }));
                (tab, rect, eligible)
            }).collect(),
        }
    }

    fn create_all_items(&mut self, cx: &mut Cx) {
        let mut items = Vec::new();
        for (item_id, item) in self.dock_items.iter() {
            if let DockItem::Tab { kind, .. } = item {
                items.push((*item_id, *kind));
            }
        }
        for (item_id, kind) in items {
            self.item_or_create(cx, item_id, kind);
        }
    }

    fn create_all_items_with_vm(&mut self, vm: &mut ScriptVm) {
        let mut items = Vec::new();
        for (item_id, item) in self.dock_items.iter() {
            if let DockItem::Tab { kind, .. } = item {
                items.push((*item_id, *kind));
            }
        }
        for (item_id, kind) in items {
            self.item_or_create_with_vm(vm, item_id, kind);
        }
    }

    fn item_or_create_with_vm(
        &mut self,
        vm: &mut ScriptVm,
        entry_id: LiveId,
        template: LiveId,
    ) -> Option<WidgetRef> {
        // Check if item already exists
        if let Some(entry) = self.items.get(&entry_id) {
            return Some(entry.1.clone());
        }

        // Get template and create new item
        if let Some(template_ref) = self.templates.get(&template) {
            let template_value: ScriptValue = template_ref.as_object().into();
            let widget = WidgetRef::script_from_value(vm, template_value);
            let cx = vm.cx_mut();
            self.items
                .get_or_insert(cx, entry_id, |_cx| (template, widget.clone()));
            cx.widget_tree_insert_child_deep(self.uid, entry_id, widget.clone());
            Some(widget)
        } else {
            warning!("Template not found: {template}. Did you add it to the <Dock> instance?");
            None
        }
    }

    fn begin(&mut self, cx: &mut Cx2d, walk: Walk) {
        cx.begin_turtle(walk, self.layout);
    }

    fn end(&mut self, cx: &mut Cx2d) {
        if self
            .drop_target_draw_list
            .begin(cx, Walk::default())
            .is_redrawing()
        {
            if let Some(pos) = &self.drop_state {
                self.drag_target_preview.draw_abs(cx, pos.rect);
            }
            self.drop_target_draw_list.end(cx);
        }

        // Draw the ghost tab overlay BEFORE retaining/ending, so it's
        // registered as the last overlay and renders on top of everything.
        if self.dragging_tab.is_some() {
            self.ghost_tab_draw_list.begin_overlay_last(cx);
            let size = cx.current_pass_size();
            cx.begin_root_turtle(size, Layout::default());
            let dt = self.dragging_tab.as_mut().unwrap();
            let ghost_pos = dvec2(dt.cursor.x + 8.0, dt.cursor.y - 30.0);
            dt.ghost.walk = Walk {
                abs_pos: Some(ghost_pos),
                width: Size::Fixed(dt.size.x),
                height: Size::Fixed(dt.size.y),
                ..Walk::default()
            };
            // Ensure the ghost tab renders above the drop target preview (draw_depth 10.0).
            dt.ghost.set_draw_depth(20.0);
            dt.ghost.draw(cx, &dt.name);
            cx.end_pass_sized_turtle();
            self.ghost_tab_draw_list.end(cx);
        } else {
            // Clear the ghost tab overlay when not dragging.
            self.ghost_tab_draw_list.begin_always(cx);
            self.ghost_tab_draw_list.end(cx);
        }

        self.tab_bars.retain_visible();
        self.splitters.retain_visible();

        for splitter in self.splitters.values() {
            self.round_corner
                .draw_corners(cx, splitter.area_a().rect(cx));
            self.round_corner
                .draw_corners(cx, splitter.area_b().rect(cx));
        }
        self.round_corner.draw_corners(cx, cx.turtle().rect());

        cx.end_turtle_with_area(&mut self.area);
    }

    fn find_drop_position(&self, cx: &Cx, abs: Vec2d) -> Option<DropPosition> {
        for (tab_bar_id, tab_bar) in self.tab_bars.iter() {
            // Skip panels with hidden tab bars — they should not be drop targets.
            if let Some(DockItem::Tabs {
                hide_tab_bar: true, ..
            }) = self.dock_items.get(tab_bar_id)
            {
                continue;
            }
            let rect = tab_bar.contents_rect;
            if let Some((tab_id, rect)) = tab_bar.tab_bar.is_over_tab(cx, abs) {
                return Some(DropPosition {
                    part: DropPart::Tab,
                    id: tab_id,
                    rect,
                });
            } else if let Some(rect) = tab_bar.tab_bar.is_over_tab_bar(cx, abs) {
                return Some(DropPosition {
                    part: DropPart::TabBar,
                    id: *tab_bar_id,
                    rect,
                });
            } else if rect.contains(abs) {
                let top_left = rect.pos;
                let bottom_right = rect.pos + rect.size;
                if (abs.x - top_left.x) / rect.size.x < 0.1 {
                    return Some(DropPosition {
                        part: DropPart::Left,
                        id: *tab_bar_id,
                        rect: Rect {
                            pos: rect.pos,
                            size: Vec2d {
                                x: rect.size.x / 2.0,
                                y: rect.size.y,
                            },
                        },
                    });
                } else if (bottom_right.x - abs.x) / rect.size.x < 0.1 {
                    return Some(DropPosition {
                        part: DropPart::Right,
                        id: *tab_bar_id,
                        rect: Rect {
                            pos: Vec2d {
                                x: rect.pos.x + rect.size.x / 2.0,
                                y: rect.pos.y,
                            },
                            size: Vec2d {
                                x: rect.size.x / 2.0,
                                y: rect.size.y,
                            },
                        },
                    });
                } else if (abs.y - top_left.y) / rect.size.y < 0.1 {
                    return Some(DropPosition {
                        part: DropPart::Top,
                        id: *tab_bar_id,
                        rect: Rect {
                            pos: rect.pos,
                            size: Vec2d {
                                x: rect.size.x,
                                y: rect.size.y / 2.0,
                            },
                        },
                    });
                } else if (bottom_right.y - abs.y) / rect.size.y < 0.1 {
                    return Some(DropPosition {
                        part: DropPart::Bottom,
                        id: *tab_bar_id,
                        rect: Rect {
                            pos: Vec2d {
                                x: rect.pos.x,
                                y: rect.pos.y + rect.size.y / 2.0,
                            },
                            size: Vec2d {
                                x: rect.size.x,
                                y: rect.size.y / 2.0,
                            },
                        },
                    });
                } else {
                    return Some(DropPosition {
                        part: DropPart::Center,
                        id: *tab_bar_id,
                        rect,
                    });
                }
            }
        }
        None
    }

    pub fn item(&self, entry_id: LiveId) -> Option<WidgetRef> {
        // `load_state_preserving_items` may keep a tab body resident while it
        // is absent from the current layout. Resident is not the same as
        // visible: callers asking for a current Dock item must not see that
        // cache entry until its Tab node is loaded again.
        if !matches!(self.dock_items.get(&entry_id), Some(DockItem::Tab { .. })) {
            return None;
        }
        if let Some(entry) = self.items.get(&entry_id) {
            return Some(entry.1.clone());
        }
        None
    }

    fn drop_target_tab_id(&self, cx: &Cx, abs: Vec2d) -> Option<LiveId> {
        let pos = self.find_drop_position(cx, abs)?;
        match pos.part {
            DropPart::Tab => Some(pos.id),
            DropPart::TabBar
            | DropPart::Left
            | DropPart::Right
            | DropPart::Top
            | DropPart::Bottom
            | DropPart::Center => {
                let DockItem::Tabs { tabs, selected, .. } = self.dock_items.get(&pos.id)? else {
                    return None;
                };
                tabs.get(*selected).copied()
            }
        }
    }

    pub fn item_or_create(
        &mut self,
        cx: &mut Cx,
        entry_id: LiveId,
        template: LiveId,
    ) -> Option<WidgetRef> {
        if let Some(template_ref) = self.templates.get(&template) {
            let template_value: ScriptValue = template_ref.as_object().into();
            let existed = self.items.contains_key(&entry_id);
            let entry = self.items.get_or_insert(cx, entry_id, |cx| {
                cx.with_vm(|vm| (template, WidgetRef::script_from_value(vm, template_value)))
            });
            if !existed {
                cx.widget_tree_insert_child_deep(self.uid, entry_id, entry.1.clone());
            }
            Some(entry.1.clone())
        } else {
            warning!("Template not found: {template}. Did you add it to the <Dock> instance?");
            None
        }
    }

    pub fn items(&mut self) -> &ComponentMap<LiveId, (LiveId, WidgetRef)> {
        &self.items
    }

    pub fn visible_items(&mut self) -> DockVisibleItemIterator<'_> {
        self.dock_item_iter_stack.clear();
        self.dock_item_iter_stack.push((id!(root), 0));
        DockVisibleItemIterator {
            stack: &mut self.dock_item_iter_stack,
            dock_items: &self.dock_items,
            items: &self.items,
        }
    }

    fn set_parent_split_in_items(
        dock_items: &mut HashMap<LiveId, DockItem>,
        what_item: LiveId,
        replace_item: LiveId,
    ) -> bool {
        for item in dock_items.values_mut() {
            match item {
                DockItem::Splitter { a, b, .. } => {
                    if what_item == *a {
                        *a = replace_item;
                        return true;
                    } else if what_item == *b {
                        *b = replace_item;
                        return true;
                    }
                }
                _ => (),
            }
        }
        false
    }

    fn remove_tabs_container_from_tree(
        dock_items: &mut HashMap<LiveId, DockItem>,
        tabs_id: LiveId,
    ) -> Option<LiveId> {
        let mut found: Option<(LiveId, LiveId)> = None;
        for (splitter_id, item) in dock_items.iter() {
            if let DockItem::Splitter { a, b, .. } = item {
                if tabs_id == *a {
                    found = Some((*splitter_id, *b));
                    break;
                } else if tabs_id == *b {
                    found = Some((*splitter_id, *a));
                    break;
                }
            }
        }
        let (splitter_id, sibling_id) = found?;
        if !dock_items.contains_key(&sibling_id) {
            return None;
        }

        if splitter_id == id!(root) {
            // Can't just remove root, the walking code starts there. Copy the
            // sibling's content into the root slot instead, and drop the sibling.
            if let Some(sibling_item) = dock_items.remove(&sibling_id) {
                dock_items.insert(splitter_id, sibling_item);
                dock_items.remove(&tabs_id);
                return Some(splitter_id);
            }
            return None;
        }

        if !Self::set_parent_split_in_items(dock_items, splitter_id, sibling_id) {
            return None;
        }
        dock_items.remove(&splitter_id);
        dock_items.remove(&tabs_id);
        Some(sibling_id)
    }

    fn split_tabs_container_in_items(
        dock_items: &mut HashMap<LiveId, DockItem>,
        target_tabs_id: LiveId,
        new_tabs_id: LiveId,
        new_split_id: LiveId,
        old_root_id: Option<LiveId>,
        part: DropPart,
    ) -> bool {
        if !matches!(
            part,
            DropPart::Left | DropPart::Right | DropPart::Top | DropPart::Bottom
        ) {
            return false;
        }
        if !matches!(dock_items.get(&target_tabs_id), Some(DockItem::Tabs { .. })) {
            return false;
        }
        if new_tabs_id == target_tabs_id
            || !matches!(dock_items.get(&new_tabs_id), Some(DockItem::Tabs { .. }))
        {
            return false;
        }

        let root = id!(root);
        if target_tabs_id == root {
            let Some(old_root_id) = old_root_id else {
                return false;
            };
            if dock_items.contains_key(&old_root_id) {
                return false;
            }
        } else if dock_items.contains_key(&new_split_id) {
            return false;
        }

        let target_child_id = if target_tabs_id == root {
            let Some(old_root_id) = old_root_id else {
                return false;
            };
            let Some(old_root_item) = dock_items.remove(&root) else {
                return false;
            };
            dock_items.insert(old_root_id, old_root_item);
            old_root_id
        } else {
            if !Self::set_parent_split_in_items(dock_items, target_tabs_id, new_split_id) {
                return false;
            }
            target_tabs_id
        };

        let split_id = if target_tabs_id == root {
            root
        } else {
            new_split_id
        };
        let split = match part {
            DropPart::Left => DockItem::Splitter {
                axis: SplitterAxis::Horizontal,
                align: SplitterAlign::Weighted(0.5),
                a: new_tabs_id,
                b: target_child_id,
            },
            DropPart::Right => DockItem::Splitter {
                axis: SplitterAxis::Horizontal,
                align: SplitterAlign::Weighted(0.5),
                a: target_child_id,
                b: new_tabs_id,
            },
            DropPart::Top => DockItem::Splitter {
                axis: SplitterAxis::Vertical,
                align: SplitterAlign::Weighted(0.5),
                a: new_tabs_id,
                b: target_child_id,
            },
            DropPart::Bottom => DockItem::Splitter {
                axis: SplitterAxis::Vertical,
                align: SplitterAlign::Weighted(0.5),
                a: target_child_id,
                b: new_tabs_id,
            },
            _ => unreachable!(),
        };
        dock_items.insert(split_id, split);
        true
    }

    fn split_tabs_container(
        &mut self,
        cx: &mut Cx,
        target_tabs_id: LiveId,
        new_tabs_id: LiveId,
        part: DropPart,
    ) -> bool {
        let root = id!(root);
        let old_root_id = if target_tabs_id == root {
            Some(self.next_internal_id())
        } else {
            None
        };
        let new_split_id = if target_tabs_id == root {
            root
        } else {
            self.next_internal_id()
        };
        let did_split = Self::split_tabs_container_in_items(
            &mut self.dock_items,
            target_tabs_id,
            new_tabs_id,
            new_split_id,
            old_root_id,
            part,
        );
        if did_split {
            self.redraw_item(
                cx,
                if target_tabs_id == root {
                    root
                } else {
                    new_split_id
                },
            );
            self.area.redraw(cx);
        }
        did_split
    }

    fn remap_drop_tabs_after_move_in_items(
        dock_items: &HashMap<LiveId, DockItem>,
        tabs_id: LiveId,
    ) -> Option<LiveId> {
        if matches!(dock_items.get(&tabs_id), Some(DockItem::Tabs { .. })) {
            Some(tabs_id)
        } else if tabs_id != id!(root)
            && matches!(dock_items.get(&id!(root)), Some(DockItem::Tabs { .. }))
        {
            Some(id!(root))
        } else {
            None
        }
    }

    fn remap_drop_tabs_after_move(&self, tabs_id: LiveId) -> Option<LiveId> {
        Self::remap_drop_tabs_after_move_in_items(&self.dock_items, tabs_id)
    }

    fn push_tab_into_tabs(&mut self, cx: &mut Cx, tabs_id: LiveId, tab_id: LiveId) -> bool {
        if let Some(DockItem::Tabs { tabs, selected, .. }) = self.dock_items.get_mut(&tabs_id) {
            if let Some(pos) = tabs.iter().position(|id| *id == tab_id) {
                *selected = pos;
            } else {
                tabs.push(tab_id);
                *selected = tabs.len() - 1;
            }
            if let Some(tab_bar) = self.tab_bars.get(&tabs_id) {
                tab_bar.contents_draw_list.redraw(cx);
            }
            true
        } else {
            false
        }
    }

    fn insert_tab_before_tab_in_items(
        dock_items: &mut HashMap<LiveId, DockItem>,
        target_tab_id: LiveId,
        tab_id: LiveId,
    ) -> Option<(LiveId, usize)> {
        for (tabs_id, item) in dock_items.iter_mut() {
            if let DockItem::Tabs { tabs, selected, .. } = item {
                if let Some(pos) = tabs.iter().position(|id| *id == target_tab_id) {
                    tabs.insert(pos, tab_id);
                    *selected = pos;
                    return Some((*tabs_id, pos));
                }
            }
        }
        None
    }

    fn insert_tab_before_tab(
        &mut self,
        cx: &mut Cx,
        target_tab_id: LiveId,
        tab_id: LiveId,
    ) -> bool {
        let Some((tab_bar_id, _pos)) =
            Self::insert_tab_before_tab_in_items(&mut self.dock_items, target_tab_id, tab_id)
        else {
            return false;
        };
        if let Some(tab_bar) = self.tab_bars.get(&tab_bar_id) {
            tab_bar.contents_draw_list.redraw(cx);
        }
        true
    }

    fn redraw_item(&mut self, cx: &mut Cx, what_item_id: LiveId) {
        if let Some(tab_bar) = self.tab_bars.get_mut(&what_item_id) {
            tab_bar.contents_draw_list.redraw(cx);
        }
        if let Some((_kind, item)) = self.items.get_mut(&what_item_id) {
            item.redraw(cx);
        }
    }

    fn splitter_position(&self, splitter_id: LiveId) -> Option<f64> {
        self.splitters
            .get(&splitter_id)
            .map(|splitter| splitter.position())
    }

    fn set_splitter_align(
        &mut self,
        cx: &mut Cx,
        splitter_id: LiveId,
        align: SplitterAlign,
        mark_dirty: bool,
    ) -> bool {
        let Some(DockItem::Splitter {
            a,
            b,
            align: current_align,
            ..
        }) = self.dock_items.get_mut(&splitter_id)
        else {
            return false;
        };

        let a = *a;
        let b = *b;
        *current_align = align;
        if let Some(splitter) = self.splitters.get_mut(&splitter_id) {
            splitter.set_align(align);
        }
        self.redraw_item(cx, a);
        self.redraw_item(cx, b);
        self.area.redraw(cx);
        if mark_dirty {
            self.needs_save = true;
        }
        true
    }

    fn unsplit_tabs(&mut self, cx: &mut Cx, tabs_id: LiveId) {
        self.needs_save = true;
        if let Some(replacement_id) =
            Self::remove_tabs_container_from_tree(&mut self.dock_items, tabs_id)
        {
            self.redraw_item(cx, replacement_id);
            self.area.redraw(cx);
        }
    }

    fn select_tab(&mut self, cx: &mut Cx, tab_id: LiveId) {
        for (tabs_id, item) in self.dock_items.iter_mut() {
            match item {
                DockItem::Tabs { tabs, selected, .. } => {
                    if let Some(pos) = tabs.iter().position(|v| *v == tab_id) {
                        if *selected == pos {
                            return;
                        }
                        self.needs_save = true;
                        *selected = pos;
                        if let Some(tab_bar) = self.tab_bars.get(&tabs_id) {
                            tab_bar.contents_draw_list.redraw(cx);
                        }
                        return;
                    }
                }
                _ => (),
            }
        }
    }

    fn set_tab_title(&mut self, cx: &mut Cx, tab_id: LiveId, new_name: String) {
        if let Some(DockItem::Tab { name, .. }) = self.dock_items.get_mut(&tab_id) {
            if *name == new_name {
                return;
            }
            self.needs_save = true;
            *name = new_name;
            self.redraw_tab(cx, tab_id);
        }
    }

    fn redraw_tab(&mut self, cx: &mut Cx, tab_id: LiveId) {
        for (tabs_id, item) in self.dock_items.iter_mut() {
            match item {
                DockItem::Tabs { tabs, .. } => {
                    if tabs.iter().any(|v| *v == tab_id) {
                        if let Some(tab_bar) = self.tab_bars.get(&tabs_id) {
                            tab_bar.contents_draw_list.redraw(cx);
                        }
                    }
                }
                _ => (),
            }
        }
    }

    fn find_tab_bar_of_tab(&self, tab_id: LiveId) -> Option<(LiveId, usize)> {
        for (tabs_id, item) in self.dock_items.iter() {
            match item {
                DockItem::Tabs { tabs, .. } => {
                    if let Some(pos) = tabs.iter().position(|v| *v == tab_id) {
                        return Some((*tabs_id, pos));
                    }
                }
                _ => (),
            }
        }
        None
    }

    fn close_tab(&mut self, cx: &mut Cx, tab_id: LiveId, keep_item: bool) -> Option<LiveId> {
        self.needs_save = true;
        for (tabs_id, item) in self.dock_items.iter_mut() {
            match item {
                DockItem::Tabs {
                    tabs,
                    selected,
                    closable,
                    ..
                } => {
                    if let Some(pos) = tabs.iter().position(|v| *v == tab_id) {
                        let tabs_id = *tabs_id;
                        tabs.remove(pos);
                        if tabs.is_empty() {
                            if *closable {
                                self.unsplit_tabs(cx, tabs_id);
                            }
                            if !keep_item {
                                self.dock_items.remove(&tab_id);
                                self.items.remove(&tab_id);
                            }
                            self.area.redraw(cx);
                            return None;
                        } else {
                            let next_tab = if *selected >= tabs.len() {
                                tabs[*selected - 1]
                            } else {
                                tabs[*selected]
                            };
                            self.select_tab(cx, next_tab);
                            // When the closed tab was the active one, the next tab usually
                            // lands at the same selected index, so select_tab's
                            // index-unchanged shortcut skips the redraw. The displayed
                            // contents still changed to a different item, so redraw them
                            // explicitly or the closed tab's pixels stay on screen.
                            if let Some(tab_bar) = self.tab_bars.get(&tabs_id) {
                                tab_bar.contents_draw_list.redraw(cx);
                            }
                            if !keep_item {
                                self.dock_items.remove(&tab_id);
                                self.items.remove(&tab_id);
                            }
                            self.area.redraw(cx);
                            return Some(tabs_id);
                        }
                    }
                }
                _ => (),
            }
        }
        None
    }

    fn check_drop_is_noop(&self, tab_id: LiveId, item_id: LiveId) -> bool {
        for (tabs_id, item) in self.dock_items.iter() {
            match item {
                DockItem::Tabs { tabs, .. } => {
                    if tabs.iter().any(|v| *v == tab_id) {
                        if *tabs_id == item_id && tabs.len() == 1 {
                            return true;
                        }
                    }
                }
                _ => (),
            }
        }
        false
    }

    fn handle_drop(&mut self, cx: &mut Cx, abs: Vec2d, item: LiveId, is_move: bool) -> bool {
        if is_move && self.find_tab_bar_of_tab(item).is_none() {
            return false;
        }
        if let Some(mut pos) = self.find_drop_position(cx, abs) {
            self.needs_save = true;
            match pos.part {
                DropPart::Left | DropPart::Right | DropPart::Top | DropPart::Bottom => {
                    if is_move {
                        if self.check_drop_is_noop(item, pos.id) {
                            return false;
                        }
                        self.close_tab(cx, item, true);
                        let Some(remapped_id) = self.remap_drop_tabs_after_move(pos.id) else {
                            return false;
                        };
                        pos.id = remapped_id;
                    } else if !matches!(self.dock_items.get(&pos.id), Some(DockItem::Tabs { .. })) {
                        return false;
                    }
                    let new_tabs = self.next_internal_id();
                    self.dock_items.insert(
                        new_tabs,
                        DockItem::Tabs {
                            tabs: vec![item],
                            closable: true,
                            hide_tab_bar: false,
                            selected: 0,
                        },
                    );
                    if !self.split_tabs_container(cx, pos.id, new_tabs, pos.part) {
                        self.dock_items.remove(&new_tabs);
                        return false;
                    }

                    return true;
                }
                DropPart::Center => {
                    if is_move {
                        if self.check_drop_is_noop(item, pos.id) {
                            return false;
                        }
                        self.close_tab(cx, item, true);
                        let Some(remapped_id) = self.remap_drop_tabs_after_move(pos.id) else {
                            return false;
                        };
                        pos.id = remapped_id;
                    }
                    return self.push_tab_into_tabs(cx, pos.id, item);
                }
                DropPart::TabBar => {
                    if is_move {
                        if self.check_drop_is_noop(item, pos.id) {
                            return false;
                        }
                        self.close_tab(cx, item, true);
                        let Some(remapped_id) = self.remap_drop_tabs_after_move(pos.id) else {
                            return false;
                        };
                        pos.id = remapped_id;
                    }
                    return self.push_tab_into_tabs(cx, pos.id, item);
                }
                DropPart::Tab => {
                    if is_move {
                        if pos.id == item {
                            return false;
                        }
                        self.close_tab(cx, item, true);
                    }
                    return self.insert_tab_before_tab(cx, pos.id, item);
                }
            }
        }
        false
    }

    fn drop_create(
        &mut self,
        cx: &mut Cx,
        abs: Vec2d,
        item: LiveId,
        kind: LiveId,
        name: String,
        template: LiveId,
    ) {
        if self.handle_drop(cx, abs, item, false) {
            self.needs_save = true;
            self.dock_items.insert(
                item,
                DockItem::Tab {
                    name,
                    template,
                    kind,
                },
            );
            self.item_or_create(cx, item, kind);
            self.select_tab(cx, item);
            self.area.redraw(cx);
        }
    }

    fn drop_clone(
        &mut self,
        cx: &mut Cx,
        abs: Vec2d,
        item: LiveId,
        new_item: LiveId,
        template: LiveId,
    ) {
        if let Some(DockItem::Tab { name, kind, .. }) = self.dock_items.get(&item) {
            let name = name.clone();
            let kind = *kind;
            if self.handle_drop(cx, abs, new_item, false) {
                self.needs_save = true;
                self.dock_items.insert(
                    new_item,
                    DockItem::Tab {
                        name,
                        template,
                        kind,
                    },
                );
                self.item_or_create(cx, new_item, kind);
                self.select_tab(cx, new_item);
            }
        }
    }

    fn create_and_select_tab(
        &mut self,
        cx: &mut Cx,
        parent: LiveId,
        item: LiveId,
        kind: LiveId,
        name: String,
        template: LiveId,
        insert_after: Option<usize>,
    ) -> Option<WidgetRef> {
        if let Some(widgetref) = self.items.get(&item).map(|(_, w)| w.clone()) {
            self.select_tab(cx, item);
            Some(widgetref)
        } else {
            let ret = self.create_tab(cx, parent, item, kind, name, template, insert_after);
            self.select_tab(cx, item);
            ret
        }
    }

    fn create_tab(
        &mut self,
        cx: &mut Cx,
        parent: LiveId,
        item: LiveId,
        kind: LiveId,
        name: String,
        template: LiveId,
        insert_after: Option<usize>,
    ) -> Option<WidgetRef> {
        if let Some(DockItem::Tabs { tabs, .. }) = self.dock_items.get_mut(&parent) {
            if let Some(after) = insert_after {
                tabs.insert(after + 1, item);
            } else {
                tabs.push(item);
            }
            self.needs_save = true;
            self.dock_items.insert(
                item,
                DockItem::Tab {
                    name,
                    template,
                    kind,
                },
            );
            self.item_or_create(cx, item, kind)
        } else {
            None
        }
    }

    fn replace_tab(
        &mut self,
        cx: &mut Cx,
        tab_item_id: LiveId,
        new_kind: LiveId,
        new_name: Option<String>,
        select: bool,
    ) -> Option<(WidgetRef, bool)> {
        let Some(DockItem::Tab { name, kind, .. }) = self.dock_items.get_mut(&tab_item_id) else {
            return None;
        };
        if let Some(template_ref) = self.templates.get(&new_kind) {
            let template_value: ScriptValue = template_ref.as_object().into();
            let Some((existing_kind, existing_widgetref)) = self.items.get_mut(&tab_item_id) else {
                return None;
            };
            let (new_widgetref, was_replaced) = if *existing_kind == new_kind {
                (existing_widgetref.clone(), false)
            } else {
                *existing_kind = new_kind;
                *existing_widgetref =
                    cx.with_vm(|vm| WidgetRef::script_from_value(vm, template_value));
                *kind = new_kind;
                (existing_widgetref.clone(), true)
            };

            if let Some(new_name) = new_name {
                *name = new_name;
            }
            if select {
                self.select_tab(cx, tab_item_id);
            }
            self.needs_save = true;
            self.redraw_tab(cx, tab_item_id);
            Some((new_widgetref, was_replaced))
        } else {
            warning!("Template not found: {new_kind}. Did you add it to the <Dock> instance?");
            None
        }
    }

    pub fn drawing_item_id(&self) -> Option<LiveId> {
        if let Some(stack) = self.draw_state.as_ref() {
            match stack.last() {
                Some(DrawStackItem::Tab { id }) => return Some(*id),
                _ => (),
            }
        }
        None
    }

    pub fn load_state(&mut self, cx: &mut Cx, dock_items: HashMap<LiveId, DockItem>) {
        self.dock_items = dock_items;
        self.items.clear();
        self.tab_bars.clear();
        self.splitters.clear();
        // Reset so the next next_internal_id call re-seeds from loaded state.
        self.next_internal_id = 0;
        self.area.redraw(cx);
        self.create_all_items(cx);
    }

    /// Load a new node map without destroying stable tab bodies.
    ///
    /// This is for alternate layouts of the same logical editors. A tab
    /// absent from `dock_items` stays resident in `items`, ready to be drawn
    /// again when a later layout names it. If a present tab reuses an ID with
    /// a different kind, the old body is dropped and recreated normally.
    /// Splitters and tab bars are layout objects and are always rebuilt.
    pub fn load_state_preserving_items(
        &mut self,
        cx: &mut Cx,
        dock_items: HashMap<LiveId, DockItem>,
    ) {
        self.items
            .retain(|id, (old_kind, _)| preserve_item_for_layout(&dock_items, *id, *old_kind));
        self.dock_items = dock_items;
        self.tab_bars.clear();
        self.splitters.clear();
        // Reset so the next next_internal_id call re-seeds from loaded state.
        self.next_internal_id = 0;
        self.area.redraw(cx);
        self.create_all_items(cx);
    }
}

impl Widget for Dock {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        let uid = self.widget_uid();
        let dock_items = &mut self.dock_items;
        for (panel_id, splitter) in self.splitters.iter_mut() {
            for action in cx.capture_actions(|cx| splitter.handle_event(cx, event, scope)) {
                match action.as_widget_action().cast() {
                    SplitterAction::Changed { axis, align } => {
                        if let Some(DockItem::Splitter {
                            axis: _axis,
                            align: _align,
                            ..
                        }) = dock_items.get_mut(&panel_id)
                        {
                            *_axis = axis;
                            *_align = align;
                        }
                        self.needs_save = true;
                        cx.widget_action(
                            uid,
                            DockAction::SplitPanelChanged {
                                panel_id: *panel_id,
                                axis,
                                align,
                            },
                        );
                    }
                    _ => (),
                }
            }
        }
        for (panel_id, tab_bar) in self.tab_bars.iter_mut() {
            let contents_view = &mut tab_bar.contents_draw_list;
            for action in cx.capture_actions(|cx| tab_bar.tab_bar.handle_event(cx, event, scope)) {
                match action.as_widget_action().cast() {
                    TabBarAction::ShouldTabStartDrag(item) => {
                        let name = if let Some(DockItem::Tab { name, .. }) = dock_items.get(&item) {
                            name.clone()
                        } else {
                            String::new()
                        };
                        let tab_size = tab_bar
                            .tab_bar
                            .tab_rect(cx, item)
                            .map(|r| r.size)
                            .unwrap_or(dvec2(100.0, 30.0));
                        if let Some(ghost) = tab_bar.tab_bar.create_ghost_tab(cx, item) {
                            self.dragging_tab = Some(DraggingTab {
                                cursor: Vec2d::default(),
                                name,
                                size: tab_size,
                                ghost,
                            });
                        } else {
                            warning!("Dock: could not create ghost tab for {:?}", item);
                        }
                        cx.widget_action(uid, DockAction::ShouldTabStartDrag(item))
                    }
                    TabBarAction::TabWasPressed(tab_id) => {
                        self.needs_save = true;
                        if let Some(DockItem::Tabs { tabs, selected, .. }) =
                            dock_items.get_mut(&panel_id)
                        {
                            if let Some(sel) = tabs.iter().position(|v| *v == tab_id) {
                                *selected = sel;
                                contents_view.redraw(cx);
                                cx.widget_action(uid, DockAction::TabWasPressed(tab_id))
                            } else {
                                log!("Cannot find tab {}", tab_id.0);
                            }
                        }
                    }
                    TabBarAction::TabCloseWasPressed(tab_id) => {
                        cx.widget_action(uid, DockAction::TabCloseWasPressed(tab_id));
                        self.needs_save = true;
                    }
                    TabBarAction::None => (),
                }
            }
        }
        // Drag/drop hit-testing must stay scoped to the visible tab content.
        // Otherwise hidden cached tab items can claim the drop before the
        // selected tab sees it.
        let visible_items_only = event.requires_visibility()
            || matches!(event, Event::Drag(_) | Event::Drop(_) | Event::DragEnd);

        if visible_items_only {
            for (_id, item) in self.visible_items() {
                item.handle_event(cx, event, scope);
            }
        } else {
            for (_id, (_templ_id, item)) in self.items.iter_mut() {
                item.handle_event(cx, event, scope);
            }
        }

        if let Event::DragEnd = event {
            self.drop_state = None;
            self.dragging_tab = None;
            let redraw_id = cx.redraw_id;
            let ghost_dl_id = self.ghost_tab_draw_list.draw_list_id();
            cx.draw_lists[ghost_dl_id].clear_draw_items(redraw_id);
            if let Some(pass_id) = cx.draw_lists[ghost_dl_id].draw_pass_id {
                cx.repaint_pass_and_child_passes(pass_id);
            }
            let drop_dl_id = self.drop_target_draw_list.draw_list_id();
            cx.draw_lists[drop_dl_id].clear_draw_items(redraw_id);
            self.area.redraw(cx);
        }

        match event.drag_hits(cx, self.area) {
            DragHit::Drag(f) => {
                self.drop_state = None;
                // Update ghost tab cursor position.
                if let Some(ref mut dt) = self.dragging_tab {
                    dt.cursor = f.abs;
                }
                self.area.redraw(cx);
                self.drop_target_draw_list.redraw(cx);
                match f.state {
                    DragState::In | DragState::Over => {
                        cx.widget_action(uid, DockAction::Drag(f.clone()))
                    }
                    DragState::Out => {}
                }
            }
            DragHit::Drop(f) => {
                self.needs_save = true;
                self.drop_state = None;
                self.dragging_tab = None;
                let redraw_id = cx.redraw_id;
                cx.draw_lists[self.ghost_tab_draw_list.draw_list_id()].clear_draw_items(redraw_id);
                cx.draw_lists[self.drop_target_draw_list.draw_list_id()]
                    .clear_draw_items(redraw_id);
                self.area.redraw(cx);
                cx.widget_action(uid, DockAction::Drop(f.clone()))
            }
            DragHit::DragEnd => {
                self.drop_state = None;
                self.dragging_tab = None;
                let redraw_id = cx.redraw_id;
                cx.draw_lists[self.ghost_tab_draw_list.draw_list_id()].clear_draw_items(redraw_id);
                cx.draw_lists[self.drop_target_draw_list.draw_list_id()]
                    .clear_draw_items(redraw_id);
                self.area.redraw(cx);
            }
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let dock_uid = self.widget_uid();
        if self
            .draw_state
            .begin_with(cx, &self.dock_items, |_, dock_items| {
                let id = id!(root);
                let root_item = dock_items.get(&id);
                vec![DrawStackItem::from_dock_item(id, root_item)]
            })
        {
            self.begin(cx, walk);
        }

        while let Some(stack) = self.draw_state.as_mut() {
            let item = stack.pop();
            match item {
                Some(DrawStackItem::SplitLeft { id }) => {
                    stack.push(DrawStackItem::SplitRight { id });
                    let splitter_template = self.splitter.clone();
                    let splitter = self.splitters.get_or_insert(cx, id, |cx| {
                        cx.with_vm(|vm| {
                            Splitter::script_from_value(vm, splitter_template.as_object().into())
                        })
                    });
                    if let Some(DockItem::Splitter { axis, align, a, .. }) =
                        self.dock_items.get(&id)
                    {
                        splitter.set_axis(*axis);
                        splitter.set_align(*align);
                        splitter.begin(cx, Walk::fill());
                        stack.push(DrawStackItem::from_dock_item(*a, self.dock_items.get(a)));
                        continue;
                    } else {
                        panic!()
                    }
                }
                Some(DrawStackItem::SplitRight { id }) => {
                    stack.push(DrawStackItem::SplitEnd { id });
                    let splitter = self.splitters.get_mut(&id).unwrap();
                    splitter.middle(cx);
                    if let Some(DockItem::Splitter { b, .. }) = self.dock_items.get(&id) {
                        stack.push(DrawStackItem::from_dock_item(*b, self.dock_items.get(b)));
                        continue;
                    } else {
                        panic!()
                    }
                }
                Some(DrawStackItem::SplitEnd { id }) => {
                    let splitter = self.splitters.get_mut(&id).unwrap();
                    splitter.end(cx);
                }
                Some(DrawStackItem::Tabs { id }) => {
                    if let Some(DockItem::Tabs {
                        selected,
                        hide_tab_bar,
                        ..
                    }) = self.dock_items.get(&id)
                    {
                        let tab_bar_template = self.tab_bar.clone();
                        let tab_bar = self.tab_bars.get_or_insert(cx, id, |cx| {
                            cx.with_vm(|vm| TabBarWrap {
                                tab_bar: TabBar::script_from_value(
                                    vm,
                                    tab_bar_template.as_object().into(),
                                ),
                                contents_draw_list: DrawList2d::script_new(vm),
                                contents_rect: Rect::default(),
                            })
                        });
                        if !*hide_tab_bar {
                            let walk = tab_bar.tab_bar.walk(cx);
                            tab_bar.tab_bar.tree_parent = dock_uid;
                            tab_bar.tab_bar.begin(cx, Some(*selected), walk);
                            stack.push(DrawStackItem::TabLabel { id, index: 0 });
                        } else {
                            // Skip the tab bar entirely, go straight to content.
                            stack.push(DrawStackItem::TabLabel {
                                id,
                                index: usize::MAX,
                            });
                        }
                    } else {
                        panic!()
                    }
                }
                Some(DrawStackItem::TabLabel { id, index }) => {
                    if let Some(DockItem::Tabs { tabs, selected, .. }) = self.dock_items.get(&id) {
                        let tab_bar = self.tab_bars.get_mut(&id).unwrap();
                        if index < tabs.len() {
                            if let Some(DockItem::Tab { name, template, .. }) =
                                self.dock_items.get(&tabs[index])
                            {
                                tab_bar
                                    .tab_bar
                                    .draw_tab(cx, tabs[index].into(), name, *template);
                            }
                            stack.push(DrawStackItem::TabLabel {
                                id,
                                index: index + 1,
                            });
                        } else {
                            if index != usize::MAX {
                                tab_bar.tab_bar.end(cx);
                            }
                            tab_bar.contents_rect = cx.turtle().rect();
                            if !tabs.is_empty()
                                && tab_bar
                                    .contents_draw_list
                                    .begin(cx, Walk::default())
                                    .is_redrawing()
                            {
                                stack.push(DrawStackItem::TabContent { id });
                                if *selected < tabs.len() {
                                    stack.push(DrawStackItem::Tab {
                                        id: tabs[*selected],
                                    });
                                }
                            }
                        }
                    } else {
                        panic!()
                    }
                }
                Some(DrawStackItem::Tab { id }) => {
                    stack.push(DrawStackItem::Tab { id });
                    if let Some(DockItem::Tab { kind, .. }) = self.dock_items.get(&id) {
                        if let Some(template_ref) = self.templates.get(kind) {
                            let template_value: ScriptValue = template_ref.as_object().into();
                            let kind_copy = *kind;
                            let existed = self.items.contains_key(&id);
                            let (_, entry) = self.items.get_or_insert(cx, id, |cx| {
                                cx.with_vm(|vm| {
                                    (kind_copy, WidgetRef::script_from_value(vm, template_value))
                                })
                            });
                            if !existed {
                                cx.widget_tree_insert_child_deep(self.uid, id, entry.clone());
                            }
                            entry.draw(cx, scope)?;
                        }
                    }
                    stack.pop();
                }
                Some(DrawStackItem::TabContent { id }) => {
                    if let Some(DockItem::Tabs { .. }) = self.dock_items.get(&id) {
                        let tab_bar = self.tab_bars.get_mut(&id).unwrap();
                        tab_bar.contents_draw_list.end(cx);
                    } else {
                        panic!()
                    }
                }
                Some(DrawStackItem::Invalid) => {}
                None => break,
            }
        }

        self.end(cx);
        self.draw_state.end();

        DrawStep::done()
    }
}

impl DockRef {
    pub fn item(&self, entry_id: LiveId) -> WidgetRef {
        if let Some(dock) = self.borrow() {
            if let Some(item) = dock.item(entry_id) {
                return item;
            }
        }
        WidgetRef::empty()
    }

    pub fn item_or_create(
        &self,
        cx: &mut Cx,
        entry_id: LiveId,
        template: LiveId,
    ) -> Option<WidgetRef> {
        if let Some(mut dock) = self.borrow_mut() {
            return dock.item_or_create(cx, entry_id, template);
        }
        None
    }

    pub fn close_tab(&self, cx: &mut Cx, tab_id: LiveId) {
        if let Some(mut dock) = self.borrow_mut() {
            dock.close_tab(cx, tab_id, false);
        }
    }

    pub fn accept_drag(&self, cx: &mut Cx, dh: DragHitEvent, dr: DragResponse) {
        if let Some(mut dock) = self.borrow_mut() {
            if let Some(pos) = dock.find_drop_position(cx, dh.abs) {
                *dh.response.lock().unwrap() = dr;
                dock.drop_state = Some(pos);
            } else {
                dock.drop_state = None;
            }
        }
    }

    pub fn drawing_item_id(&self) -> Option<LiveId> {
        if let Some(dock) = self.borrow() {
            return dock.drawing_item_id();
        }
        None
    }

    pub fn drop_clone(
        &self,
        cx: &mut Cx,
        abs: Vec2d,
        old_item: LiveId,
        new_item: LiveId,
        template: LiveId,
    ) {
        if let Some(mut dock) = self.borrow_mut() {
            dock.drop_clone(cx, abs, old_item, new_item, template);
        }
    }

    pub fn drop_move(&self, cx: &mut Cx, abs: Vec2d, item: LiveId) {
        if let Some(mut dock) = self.borrow_mut() {
            dock.handle_drop(cx, abs, item, true);
        }
    }

    pub fn drop_create(
        &self,
        cx: &mut Cx,
        abs: Vec2d,
        item: LiveId,
        kind: LiveId,
        name: String,
        template: LiveId,
    ) {
        if let Some(mut dock) = self.borrow_mut() {
            dock.drop_create(cx, abs, item, kind, name, template);
        }
    }

    pub fn create_and_select_tab(
        &self,
        cx: &mut Cx,
        parent: LiveId,
        item: LiveId,
        kind: LiveId,
        name: String,
        template: LiveId,
        insert_after: Option<usize>,
    ) -> Option<WidgetRef> {
        if let Some(mut dock) = self.borrow_mut() {
            dock.create_and_select_tab(cx, parent, item, kind, name, template, insert_after)
        } else {
            None
        }
    }

    pub fn create_tab(
        &self,
        cx: &mut Cx,
        parent: LiveId,
        item: LiveId,
        kind: LiveId,
        name: String,
        template: LiveId,
        insert_after: Option<usize>,
    ) -> Option<WidgetRef> {
        if let Some(mut dock) = self.borrow_mut() {
            dock.create_tab(cx, parent, item, kind, name, template, insert_after)
        } else {
            None
        }
    }

    pub fn replace_tab(
        &self,
        cx: &mut Cx,
        tab_item_id: LiveId,
        new_kind: LiveId,
        new_name: Option<String>,
        select: bool,
    ) -> Option<(WidgetRef, bool)> {
        let Some(mut dock) = self.borrow_mut() else {
            return None;
        };
        dock.replace_tab(cx, tab_item_id, new_kind, new_name, select)
    }

    pub fn set_tab_title(&self, cx: &mut Cx, tab: LiveId, title: String) {
        if let Some(mut dock) = self.borrow_mut() {
            dock.set_tab_title(cx, tab, title);
        }
    }

    pub fn find_tab_bar_of_tab(&self, tab_id: LiveId) -> Option<(LiveId, usize)> {
        if let Some(dock) = self.borrow() {
            return dock.find_tab_bar_of_tab(tab_id);
        }
        None
    }

    pub fn drop_target_tab_id(&self, cx: &Cx, abs: Vec2d) -> Option<LiveId> {
        if let Some(dock) = self.borrow() {
            return dock.drop_target_tab_id(cx, abs);
        }
        None
    }

    pub fn select_tab(&self, cx: &mut Cx, item: LiveId) {
        if let Some(mut dock) = self.borrow_mut() {
            dock.select_tab(cx, item);
        }
    }

    pub fn redraw_tab(&self, cx: &mut Cx, tab_id: LiveId) {
        if let Some(mut dock) = self.borrow_mut() {
            dock.redraw_tab(cx, tab_id);
        }
    }

    pub fn splitter_position(&self, splitter_id: LiveId) -> Option<f64> {
        self.borrow()
            .and_then(|dock| dock.splitter_position(splitter_id))
    }

    pub fn set_splitter_align(
        &self,
        cx: &mut Cx,
        splitter_id: LiveId,
        align: SplitterAlign,
        mark_dirty: bool,
    ) -> bool {
        self.borrow_mut()
            .is_some_and(|mut dock| dock.set_splitter_align(cx, splitter_id, align, mark_dirty))
    }

    pub fn unique_id(&self, base: u64) -> LiveId {
        if let Some(dock) = self.borrow() {
            return dock.unique_id(base);
        }
        LiveId(0)
    }

    pub fn check_and_clear_need_save(&self) -> bool {
        if let Some(mut dock) = self.borrow_mut() {
            if dock.needs_save {
                dock.needs_save = false;
                return true;
            }
        }
        false
    }

    pub fn clone_state(&self) -> Option<HashMap<LiveId, DockItem>> {
        if let Some(dock) = self.borrow() {
            return Some(dock.dock_items.clone());
        }
        None
    }

    pub fn load_state(&self, cx: &mut Cx, dock_items: HashMap<LiveId, DockItem>) {
        if let Some(mut dock) = self.borrow_mut() {
            dock.load_state(cx, dock_items);
        }
    }

    pub fn load_state_preserving_items(
        &self,
        cx: &mut Cx,
        dock_items: HashMap<LiveId, DockItem>,
    ) {
        if let Some(mut dock) = self.borrow_mut() {
            dock.load_state_preserving_items(cx, dock_items);
        }
    }

    pub fn tab_start_drag(&self, cx: &mut Cx, _tab_id: LiveId, item: DragItem) {
        cx.start_dragging(vec![item]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widget_tree::{interaction_visibility_test_support::*, WidgetTree};
    use std::{cell::Cell, rc::Rc};

    fn retained_tabs(cx: &mut Cx, owner: &DrawList, ids: &[LiveId]) -> (Dock, Vec<WidgetRef>) {
        let mut dock = cx.with_vm(Dock::script_new);
        let mut sends = Vec::new();
        let rect = Rect { pos: dvec2(20.0, 30.0), size: dvec2(100.0, 40.0) };
        for id in ids {
            let area = retained_rect(cx, owner, rect);
            assert!(area.is_valid(cx));
            assert_eq!(area.clipped_rect(cx), rect);
            let send = area_widget(area, Rc::new(Cell::new(true)), vec![]);
            let body = area_widget(Area::Empty, Rc::new(Cell::new(true)), vec![(id!(send), send.clone())]);
            dock.dock_items.insert(*id, DockItem::tab(id.0.to_string(), id!(Panel), id!(PermanentTab)));
            dock.items.insert(*id, (id!(Panel), body));
            sends.push(send);
        }
        dock.dock_items.insert(id!(root), DockItem::tabs(ids.to_vec(), 0, false));
        (dock, sends)
    }

    fn indexed_dock(dock: Dock) -> (WidgetRef, WidgetTree) {
        let root = WidgetRef::new_with_inner(Box::new(dock));
        let tree = WidgetTree::default();
        tree.observe_node(root.widget_uid(), id!(dock), root.clone(), None);
        (root, tree)
    }

    fn visible_send_uids(tree: &WidgetTree, cx: &Cx) -> Vec<String> {
        let mut uids: Vec<_> = tree.snapshot(cx).into_iter()
            .filter(|row| row.id == "send" && row.visible)
            .map(|row| row.text.unwrap()).collect();
        uids.sort();
        uids
    }

    #[test]
    fn interaction_visibility_retained_two_tabs() {
        let mut cx = init_cx();
        let owner = DrawList::new(&mut cx);
        let (dock, sends) = retained_tabs(&mut cx, &owner, &[id!(tab_a), id!(tab_b)]);
        let (root, tree) = indexed_dock(dock);
        assert_eq!(tree.snapshot(&cx).iter().filter(|row| row.id == "send").count(), 2);
        assert_eq!(visible_send_uids(&tree, &cx), vec![sends[0].widget_uid().0.to_string()]);
        root.borrow_mut::<Dock>().unwrap().select_tab(&mut cx, id!(tab_b));
        assert_eq!(visible_send_uids(&tree, &cx), vec![sends[1].widget_uid().0.to_string()]);
        assert_eq!(tree.find_within(root.widget_uid(), &[id!(tab_a), id!(send)]).widget_uid(), sends[0].widget_uid());
        assert_eq!(tree.find_within(root.widget_uid(), &[id!(tab_b), id!(send)]).widget_uid(), sends[1].widget_uid());
        assert_eq!(tree.flat_tree(&cx).iter().filter(|row| row.name == "send").count(), 2);
        assert_eq!(tree.compact_dump(&cx).matches("send").count(), 2);
    }

    #[test]
    fn interaction_visibility_retained_query_rects() {
        let mut cx = init_cx();
        let owner = DrawList::new(&mut cx);
        let (dock, _) = retained_tabs(&mut cx, &owner, &[id!(tab_a), id!(tab_b)]);
        let (root, tree) = indexed_dock(dock);
        assert_eq!(tree.query_rects(&cx, "id:send").len(), 1);
        assert_eq!(tree.query_rects(&cx, "path:tab_a/send").len(), 1);
        assert!(tree.query_rects(&cx, "path:tab_b/send").is_empty());
        root.borrow_mut::<Dock>().unwrap().select_tab(&mut cx, id!(tab_b));
        assert_eq!(tree.query_rects(&cx, "id:send").len(), 1);
        assert!(tree.query_rects(&cx, "path:tab_a/send").is_empty());
        assert_eq!(tree.query_rects(&cx, "path:tab_b/send").len(), 1);
    }

    #[derive(Script, ScriptHook, Widget)]
    struct WrappedDock {
        #[wrap]
        #[live]
        dock: Dock,
    }
    impl Widget for WrappedDock {}

    #[derive(Script, ScriptHook, Widget)]
    struct DerefDock {
        #[deref]
        dock: Dock,
    }
    impl Widget for DerefDock {}

    #[derive(Script, ScriptHook, Widget)]
    struct FindDocks {
        #[uid]
        uid: WidgetUid,
        #[find]
        #[redraw]
        #[live]
        left: Dock,
        #[find]
        #[live]
        right: Dock,
    }
    impl Widget for FindDocks {}

    fn assert_wrapped_selection(cx: &Cx, root: WidgetRef, mut expected: Vec<String>) {
        let tree = WidgetTree::default();
        tree.observe_node(root.widget_uid(), id!(wrapped), root.clone(), None);
        expected.sort();
        assert_eq!(visible_send_uids(&tree, cx), expected);
        assert_eq!(tree.query_rects(cx, "id:send").len(), expected.len());
    }

    #[test]
    fn interaction_visibility_derive_wrap() {
        let mut cx = init_cx();
        let owner = DrawList::new(&mut cx);
        let (dock, sends) = retained_tabs(&mut cx, &owner, &[id!(tab_a), id!(tab_b)]);
        let root = WidgetRef::new_with_inner(Box::new(WrappedDock { dock }));
        assert_wrapped_selection(&cx, root, vec![sends[0].widget_uid().0.to_string()]);
    }

    #[test]
    fn interaction_visibility_derive_deref() {
        let mut cx = init_cx();
        let owner = DrawList::new(&mut cx);
        let (dock, sends) = retained_tabs(&mut cx, &owner, &[id!(tab_a), id!(tab_b)]);
        let root = WidgetRef::new_with_inner(Box::new(DerefDock { dock }));
        assert_wrapped_selection(&cx, root, vec![sends[0].widget_uid().0.to_string()]);
    }

    #[test]
    fn interaction_visibility_derive_all_find_fields() {
        let mut cx = init_cx();
        let owner = DrawList::new(&mut cx);
        let (left, a) = retained_tabs(&mut cx, &owner, &[id!(tab_a), id!(tab_b)]);
        let (right, b) = retained_tabs(&mut cx, &owner, &[id!(tab_c), id!(tab_d)]);
        let root = WidgetRef::new_with_inner(Box::new(FindDocks { uid: WidgetUid::new(), left, right }));
        assert_wrapped_selection(&cx, root, vec![a[0].widget_uid().0.to_string(), b[0].widget_uid().0.to_string()]);
    }

    #[test]
    fn interaction_visibility_six_retained_controls() {
        let mut cx = init_cx();
        let owner = DrawList::new(&mut cx);
        let ids = [id!(tab_a), id!(tab_b), id!(tab_c), id!(tab_d), id!(tab_e), id!(tab_f)];
        let (dock, sends) = retained_tabs(&mut cx, &owner, &ids);
        let (root, tree) = indexed_dock(dock);
        for (index, id) in ids.iter().enumerate() {
            root.borrow_mut::<Dock>().unwrap().select_tab(&mut cx, *id);
            assert_eq!(tree.snapshot(&cx).iter().filter(|row| row.id == "send").count(), 6);
            assert_eq!(visible_send_uids(&tree, &cx), vec![sends[index].widget_uid().0.to_string()]);
            assert_eq!(tree.query_rects(&cx, "id:send").len(), 1);
            for (index, id) in ids.iter().enumerate() {
                assert_eq!(tree.find_within(root.widget_uid(), &[*id, id!(send)]).widget_uid(), sends[index].widget_uid());
            }
        }
    }

    #[test]
    fn interaction_visibility_split_and_nested_splitters() {
        let mut cx = init_cx();
        let owner = DrawList::new(&mut cx);
        let (mut dock, sends) = retained_tabs(&mut cx, &owner, &[id!(a), id!(b), id!(c), id!(d), id!(e), id!(f)]);
        dock.dock_items.insert(id!(root), DockItem::Splitter { axis: SplitterAxis::Horizontal, align: SplitterAlign::Weighted(0.5), a: id!(left), b: id!(right) });
        dock.dock_items.insert(id!(left), DockItem::tabs(vec![id!(a), id!(b)], 1, false));
        dock.dock_items.insert(id!(right), DockItem::tabs(vec![id!(c), id!(d)], 0, false));
        let (root, tree) = indexed_dock(dock);
        let mut expected = vec![sends[1].widget_uid().0.to_string(), sends[2].widget_uid().0.to_string()];
        expected.sort();
        assert_eq!(visible_send_uids(&tree, &cx), expected);
        assert_eq!(tree.query_rects(&cx, "id:send").len(), 2);
        {
            let mut dock = root.borrow_mut::<Dock>().unwrap();
            dock.dock_items.insert(id!(right), DockItem::Splitter { axis: SplitterAxis::Vertical, align: SplitterAlign::Weighted(0.5), a: id!(top), b: id!(bottom) });
            dock.dock_items.insert(id!(top), DockItem::tabs(vec![id!(c), id!(d)], 0, false));
            dock.dock_items.insert(id!(bottom), DockItem::tabs(vec![id!(e), id!(f)], 1, false));
        }
        expected.push(sends[5].widget_uid().0.to_string());
        expected.sort();
        assert_eq!(visible_send_uids(&tree, &cx), expected);
        assert_eq!(tree.query_rects(&cx, "id:send").len(), 3);
    }

    #[test]
    fn interaction_visibility_nested_dock_parent_selection() {
        let mut cx = init_cx();
        let owner = DrawList::new(&mut cx);
        let (mut outer, sends) = retained_tabs(&mut cx, &owner, &[id!(tab_a), id!(tab_b)]);
        let (inner, inner_sends) = retained_tabs(&mut cx, &owner, &[id!(inner_a), id!(inner_b)]);
        let inner = WidgetRef::new_with_inner(Box::new(inner));
        outer.items.insert(id!(tab_a), (id!(Panel), inner.clone()));
        let (root, tree) = indexed_dock(outer);
        assert_eq!(visible_send_uids(&tree, &cx), vec![inner_sends[0].widget_uid().0.to_string()]);
        root.borrow_mut::<Dock>().unwrap().select_tab(&mut cx, id!(tab_b));
        assert_eq!(visible_send_uids(&tree, &cx), vec![sends[1].widget_uid().0.to_string()]);
        inner.borrow_mut::<Dock>().unwrap().select_tab(&mut cx, id!(inner_b));
        assert_eq!(visible_send_uids(&tree, &cx), vec![sends[1].widget_uid().0.to_string()]);
        root.borrow_mut::<Dock>().unwrap().select_tab(&mut cx, id!(tab_a));
        assert_eq!(visible_send_uids(&tree, &cx), vec![inner_sends[1].widget_uid().0.to_string()]);
        assert_eq!(tree.query_rects(&cx, "id:send").len(), 1);
    }

    fn retained_headers(dock: &mut Dock, cx: &mut Cx, owner: &DrawList, bar_id: LiveId, ids: &[LiveId]) -> Vec<WidgetRef> {
        let rect = Rect { pos: dvec2(20.0, 30.0), size: dvec2(100.0, 40.0) };
        let bar_area = retained_rect(cx, owner, rect);
        let headers: Vec<_> = ids.iter().map(|id| {
            let area = retained_rect(cx, owner, rect);
            (*id, area_widget(area, Rc::new(Cell::new(true)), vec![]))
        }).collect();
        let widgets = headers.iter().map(|(_, header)| header.clone()).collect();
        let tab_bar = TabBar::interaction_visibility_fixture(cx, bar_area, headers);
        dock.tab_bars.insert(bar_id, TabBarWrap {
            tab_bar,
            contents_draw_list: cx.with_vm(DrawList2d::script_new),
            contents_rect: rect,
        });
        widgets
    }

    #[test]
    fn interaction_visibility_current_unselected_headers() {
        let mut cx = init_cx();
        let owner = DrawList::new(&mut cx);
        let (mut dock, sends) = retained_tabs(&mut cx, &owner, &[id!(tab_a), id!(tab_b)]);
        retained_headers(&mut dock, &mut cx, &owner, id!(root), &[id!(tab_a), id!(tab_b)]);
        let (_root, tree) = indexed_dock(dock);
        assert_eq!(visible_send_uids(&tree, &cx), vec![sends[0].widget_uid().0.to_string()]);
        let rows = tree.snapshot(&cx);
        for id in ["tab_a_tab", "tab_b_tab"] {
            assert!(rows.iter().find(|row| row.id == id).unwrap().visible);
            assert_eq!(tree.query_rects(&cx, &format!("id:{id}")).len(), 1);
        }
        assert_eq!(rows.iter().filter(|row| row.widget_type == "DockTab" && row.visible).count(), 2);
        assert_eq!(tree.query_rects(&cx, "type:DockTabs").len(), 1);
    }

    #[test]
    fn interaction_visibility_retained_removed_and_hidden_headers() {
        let mut cx = init_cx();
        let owner = DrawList::new(&mut cx);
        let (mut dock, sends) = retained_tabs(&mut cx, &owner, &[id!(tab_a), id!(tab_b), id!(tab_c)]);
        retained_headers(&mut dock, &mut cx, &owner, id!(root), &[id!(tab_a), id!(tab_b)]);
        dock.dock_items.insert(id!(unused), DockItem::tabs(vec![id!(tab_c)], 0, false));
        retained_headers(&mut dock, &mut cx, &owner, id!(unused), &[id!(tab_c)]);
        dock.dock_items.insert(id!(root), DockItem::tabs(vec![id!(tab_a)], 0, false));
        let (root, tree) = indexed_dock(dock);
        let rows = tree.snapshot(&cx);
        assert!(!rows.iter().find(|row| row.id == "tab_b_tab").unwrap().visible);
        assert!(!rows.iter().find(|row| row.id == "tab_c_tab").unwrap().visible);
        assert!(!rows.iter().find(|row| row.id == "unused" && row.widget_type == "DockTabs").unwrap().visible);
        assert_eq!(tree.query_rects(&cx, "type:DockTab").len(), 1);
        root.borrow_mut::<Dock>().unwrap().dock_items.insert(id!(root), DockItem::Tabs {
            tabs: vec![id!(tab_a)], selected: 0, closable: false, hide_tab_bar: true,
        });
        assert_eq!(visible_send_uids(&tree, &cx), vec![sends[0].widget_uid().0.to_string()]);
        assert!(tree.query_rects(&cx, "type:DockTabs").is_empty());
        assert!(tree.query_rects(&cx, "type:DockTab").is_empty());
        assert!(!tree.snapshot(&cx).iter().find(|row| row.id == "tab_a_tab").unwrap().visible);
        assert!(tree.compact_dump(&cx).contains("D3 "));
    }

    #[test]
    fn interaction_visibility_nested_synthetic_headers() {
        let mut cx = init_cx();
        let owner = DrawList::new(&mut cx);
        let (mut outer, _) = retained_tabs(&mut cx, &owner, &[id!(tab_a), id!(tab_b)]);
        let (mut inner, _) = retained_tabs(&mut cx, &owner, &[id!(inner_a), id!(inner_b)]);
        retained_headers(&mut inner, &mut cx, &owner, id!(root), &[id!(inner_a), id!(inner_b)]);
        outer.items.insert(id!(tab_a), (id!(Panel), WidgetRef::new_with_inner(Box::new(inner))));
        let (root, tree) = indexed_dock(outer);
        assert_eq!(tree.query_rects(&cx, "type:DockTab").len(), 2);
        root.borrow_mut::<Dock>().unwrap().select_tab(&mut cx, id!(tab_b));
        let rows = tree.snapshot(&cx);
        let synthetic: Vec<_> = rows.iter().filter(|row| matches!(row.widget_type.as_str(), "DockTab" | "DockTabs")).collect();
        assert_eq!(synthetic.len(), 3);
        assert!(synthetic.iter().all(|row| !row.visible));
        assert!(!rows.iter().find(|row| row.id == "inner_a_tab").unwrap().visible);
        assert!(!rows.iter().find(|row| row.id == "inner_b_tab").unwrap().visible);
        assert!(tree.query_rects(&cx, "type:DockTab").is_empty());
        root.borrow_mut::<Dock>().unwrap().select_tab(&mut cx, id!(tab_a));
        assert_eq!(tree.query_rects(&cx, "type:DockTab").len(), 2);
    }

    #[test]
    fn interaction_visibility_clipped_synthetic_headers() {
        let mut cx = init_cx();
        let owner = DrawList::new(&mut cx);
        let (mut dock, _) = retained_tabs(&mut cx, &owner, &[id!(tab_a), id!(tab_b)]);
        let headers = retained_headers(&mut dock, &mut cx, &owner, id!(root), &[id!(tab_a), id!(tab_b)]);
        let Area::Rect(a) = headers[0].area() else { panic!("expected retained rect") };
        cx.draw_lists[a.draw_list_id].rect_areas[a.rect_id].draw_clip = (dvec2(0.0, 0.0), dvec2(1.0, 1.0));
        let (root, tree) = indexed_dock(dock);
        let rows = tree.snapshot(&cx);
        assert!(!rows.iter().find(|row| row.id == "tab_a" && row.widget_type == "DockTab").unwrap().visible);
        assert_eq!(tree.query_rects(&cx, "type:DockTab").len(), 1);
        assert_eq!(root.borrow::<Dock>().unwrap().compact_dump(&cx).tab_headers.len(), 2);
    }

    #[test]
    fn interaction_visibility_header_own_hidden() {
        let mut cx = init_cx();
        let owner = DrawList::new(&mut cx);
        let (mut dock, _) = retained_tabs(&mut cx, &owner, &[id!(tab_a)]);
        let rect = Rect { pos: dvec2(20.0, 30.0), size: dvec2(100.0, 40.0) };
        let bar_area = retained_rect(&mut cx, &owner, rect);
        let area = retained_rect(&mut cx, &owner, rect);
        let visible = Rc::new(Cell::new(true));
        let header = area_widget(area, visible.clone(), vec![]);
        let bar = TabBar::interaction_visibility_fixture(&mut cx, bar_area, vec![(id!(tab_a), header)]);
        dock.tab_bars.insert(id!(root), TabBarWrap { tab_bar: bar, contents_draw_list: cx.with_vm(DrawList2d::script_new), contents_rect: rect });
        let (_root, tree) = indexed_dock(dock);
        assert_eq!(tree.query_rects(&cx, "type:DockTab").len(), 1);
        visible.set(false);
        assert!(!tree.snapshot(&cx).iter().find(|row| row.id == "tab_a_tab").unwrap().visible);
        assert!(tree.query_rects(&cx, "type:DockTab").is_empty());
        assert!(!tree.snapshot(&cx).iter().find(|row| row.id == "tab_a" && row.widget_type == "DockTab").unwrap().visible);
    }

    #[test]
    fn interaction_visibility_clipped_bar_and_stale_header() {
        let mut cx = init_cx();
        let owner = DrawList::new(&mut cx);
        let (mut dock, _) = retained_tabs(&mut cx, &owner, &[id!(tab_a), id!(tab_b)]);
        let rect = Rect { pos: dvec2(20.0, 30.0), size: dvec2(100.0, 40.0) };
        let bar_area = retained_rect(&mut cx, &owner, rect);
        let Area::Rect(bar_slot) = bar_area else { panic!("expected rect area") };
        cx.draw_lists[bar_slot.draw_list_id].rect_areas[bar_slot.rect_id].draw_clip = (dvec2(0.0, 0.0), dvec2(1.0, 1.0));
        let mut stale = retained_rect(&mut cx, &owner, rect);
        if let Area::Rect(area) = &mut stale { area.redraw_id = 0; }
        assert!(!stale.is_valid(&cx));
        let current = retained_rect(&mut cx, &owner, rect);
        let bar = TabBar::interaction_visibility_fixture(&mut cx, bar_area, vec![
            (id!(tab_a), area_widget(stale, Rc::new(Cell::new(true)), vec![])),
            (id!(tab_b), area_widget(current, Rc::new(Cell::new(true)), vec![])),
        ]);
        dock.tab_bars.insert(id!(root), TabBarWrap { tab_bar: bar, contents_draw_list: cx.with_vm(DrawList2d::script_new), contents_rect: rect });
        let (root, tree) = indexed_dock(dock);
        assert!(tree.query_rects(&cx, "type:DockTabs").is_empty());
        let rows = tree.snapshot(&cx);
        assert!(!rows.iter().find(|row| row.widget_type == "DockTabs").unwrap().visible);
        assert!(!rows.iter().any(|row| row.id == "tab_a" && row.widget_type == "DockTab" && row.visible));
        assert_eq!(tree.query_rects(&cx, "type:DockTab").len(), 1);
        assert_eq!(root.borrow::<Dock>().unwrap().compact_dump(&cx).tabs[0].rect, rect);
    }

    #[test]
    fn interaction_visibility_layout_missing_invalid_and_cycles() {
        let mut cx = init_cx();
        let owner = DrawList::new(&mut cx);
        let (mut dock, _) = retained_tabs(&mut cx, &owner, &[id!(tab_a), id!(tab_b)]);
        let expected: HashSet<_> = dock.visible_items().map(|(id, _)| id).collect();
        assert_eq!(dock.interaction_layout().content, expected);
        for tabs in [vec![], vec![id!(missing)], vec![id!(tab_a)]] {
            dock.dock_items.insert(id!(root), DockItem::tabs(tabs, 9, false));
            assert!(dock.interaction_layout().content.is_empty());
        }
        dock.dock_items.insert(id!(root), DockItem::tabs(vec![id!(missing)], 0, false));
        assert!(dock.interaction_layout().content.is_empty());
        dock.dock_items.remove(&id!(root));
        assert!(dock.interaction_layout().bars.is_empty());
        assert!(dock.interaction_layout().content.is_empty());
        dock.dock_items.insert(id!(root), DockItem::Splitter { axis: SplitterAxis::Horizontal, align: SplitterAlign::Weighted(0.5), a: id!(root), b: id!(missing) });
        assert!(dock.interaction_layout().content.is_empty());
        assert!(dock.interaction_layout().bars.is_empty());
    }

    #[derive(Script, ScriptHook, Widget)]
    struct FindWithDeref {
        #[deref]
        base: Dock,
        #[find]
        #[live]
        contents: Dock,
    }
    impl Widget for FindWithDeref {}

    #[test]
    fn interaction_visibility_find_precedes_deref() {
        let mut cx = init_cx();
        let owner = DrawList::new(&mut cx);
        let (base, _) = retained_tabs(&mut cx, &owner, &[id!(tab_a), id!(tab_b)]);
        let (contents, sends) = retained_tabs(&mut cx, &owner, &[id!(tab_c), id!(tab_d)]);
        let root = WidgetRef::new_with_inner(Box::new(FindWithDeref { base, contents }));
        assert_wrapped_selection(&cx, root, vec![sends[0].widget_uid().0.to_string()]);
    }

    #[derive(Script, ScriptHook, Widget)]
    struct FindDockRefs {
        #[uid]
        uid: WidgetUid,
        #[find]
        #[redraw]
        #[live]
        left: WidgetRef,
        #[find]
        #[live]
        right: WidgetRef,
    }
    impl Widget for FindDockRefs {}

    #[derive(Script, ScriptHook, Widget)]
    struct BorrowedDockWrapper {
        #[wrap]
        #[live]
        dock: WidgetRef,
    }
    impl Widget for BorrowedDockWrapper {}

    fn assert_borrowed_projection(tree: &WidgetTree, cx: &Cx, snapshot: bool, sends: usize) {
        if snapshot {
            let rows = tree.snapshot(cx);
            assert_eq!(rows.iter().filter(|row| row.id == "send").count(), sends,
                "the indexed descendants must remain inspectable during the borrow");
            assert!(rows.iter().all(|row| !row.visible), "unavailable policy must fail closed");
            assert!(rows.iter().any(|row| !row.enabled && row.width == 0 && row.height == 0
                && row.text.is_none()), "unavailable metadata must retain a disabled inspection row");
        } else {
            assert!(tree.query_rects(cx, "").is_empty(), "unavailable policy must expose no actionable rectangles");
        }
    }

    fn borrowed_body_projection(snapshot: bool) {
        let mut cx = init_cx();
        let owner = DrawList::new(&mut cx);
        let (dock, sends) = retained_tabs(&mut cx, &owner, &[id!(tab_a), id!(tab_b)]);
        let body = dock.items[id!(tab_a)].1.clone();
        let (_root, tree) = indexed_dock(dock);
        let before = tree.snapshot(&cx);
        assert_eq!(tree.query_rects(&cx, "id:send").len(), 1);
        let borrowed = body.borrow_mut::<AreaWidget>().unwrap();
        assert_borrowed_projection(&tree, &cx, snapshot, 2);
        drop(borrowed);
        assert_eq!(tree.snapshot(&cx).len(), before.len());
        assert_eq!(visible_send_uids(&tree, &cx), vec![sends[0].widget_uid().0.to_string()]);
        assert_eq!(tree.query_rects(&cx, "id:send").len(), 1);
    }

    #[test]
    fn interaction_visibility_borrowed_projection_body_snapshot() { borrowed_body_projection(true); }
    #[test]
    fn interaction_visibility_borrowed_projection_body_query_rects() { borrowed_body_projection(false); }

    fn borrowed_wrapper_projection(snapshot: bool, find: bool) {
        let mut cx = init_cx();
        let owner = DrawList::new(&mut cx);
        let (left, _) = retained_tabs(&mut cx, &owner, &[id!(tab_a), id!(tab_b)]);
        let left = WidgetRef::new_with_inner(Box::new(left));
        let root = if find {
            let (right, _) = retained_tabs(&mut cx, &owner, &[id!(tab_c), id!(tab_d)]);
            WidgetRef::new_with_inner(Box::new(FindDockRefs {
                uid: WidgetUid::new(), left: left.clone(),
                right: WidgetRef::new_with_inner(Box::new(right)),
            }))
        } else {
            WidgetRef::new_with_inner(Box::new(BorrowedDockWrapper { dock: left.clone() }))
        };
        let tree = WidgetTree::default();
        tree.observe_node(root.widget_uid(), id!(wrapped), root.clone(), None);
        let before = tree.snapshot(&cx);
        let visible_before = visible_send_uids(&tree, &cx);
        assert_eq!(visible_before.len(), if find { 2 } else { 1 });
        let borrowed = left.borrow_mut::<Dock>().unwrap();
        assert_borrowed_projection(&tree, &cx, snapshot, if find { 4 } else { 2 });
        drop(borrowed);
        assert_eq!(tree.snapshot(&cx).len(), before.len());
        assert_eq!(visible_send_uids(&tree, &cx), visible_before);
        assert_eq!(tree.query_rects(&cx, "id:send").len(), visible_before.len());
    }

    #[test]
    fn interaction_visibility_borrowed_projection_wrap_snapshot() { borrowed_wrapper_projection(true, false); }
    #[test]
    fn interaction_visibility_borrowed_projection_wrap_query_rects() { borrowed_wrapper_projection(false, false); }
    #[test]
    fn interaction_visibility_borrowed_projection_find_snapshot() { borrowed_wrapper_projection(true, true); }
    #[test]
    fn interaction_visibility_borrowed_projection_find_query_rects() { borrowed_wrapper_projection(false, true); }

    fn borrowed_header_projection(snapshot: bool) {
        let mut cx = init_cx();
        let owner = DrawList::new(&mut cx);
        let (mut dock, _) = retained_tabs(&mut cx, &owner, &[id!(tab_a), id!(tab_b)]);
        let headers = retained_headers(&mut dock, &mut cx, &owner, id!(root), &[id!(tab_a), id!(tab_b)]);
        let (_root, tree) = indexed_dock(dock);
        let before = tree.snapshot(&cx);
        assert_eq!(tree.query_rects(&cx, "type:DockTab").len(), 2);
        let borrowed = headers[1].borrow_mut::<AreaWidget>().unwrap();
        assert_borrowed_projection(&tree, &cx, snapshot, 2);
        drop(borrowed);
        assert_eq!(tree.snapshot(&cx).len(), before.len());
        assert_eq!(tree.query_rects(&cx, "type:DockTab").len(), 2);
        assert_eq!(tree.query_rects(&cx, "id:send").len(), 1);
    }

    #[test]
    fn interaction_visibility_borrowed_projection_header_snapshot() { borrowed_header_projection(true); }
    #[test]
    fn interaction_visibility_borrowed_projection_header_query_rects() { borrowed_header_projection(false); }

    #[test]
    fn interaction_visibility_borrowed_projection_header_metadata() {
        let mut cx = init_cx();
        let owner = DrawList::new(&mut cx);
        let (mut dock, _) = retained_tabs(&mut cx, &owner, &[id!(tab_a), id!(tab_b)]);
        let headers = retained_headers(&mut dock, &mut cx, &owner, id!(root), &[id!(tab_a), id!(tab_b)]);
        let borrowed = headers[1].borrow_mut::<AreaWidget>().unwrap();
        let bar = &dock.tab_bars[id!(root)].tab_bar;
        assert!(bar.interaction_tab_rect(&cx, id!(tab_b)).is_none());
        assert!(bar.tab_rect(&cx, id!(tab_b)).is_none());
        assert_eq!(dock.compact_dump(&cx).tab_headers.len(), 1);
        assert_eq!(dock.interaction_dump(&cx).tab_headers.len(), 1);
        drop(borrowed);
        assert_eq!(dock.compact_dump(&cx).tab_headers.len(), 2);
        assert_eq!(dock.interaction_dump(&cx).tab_headers.len(), 2);
    }

    #[test]
    fn interaction_visibility_borrowed_projection_shared_metadata() {
        let mut cx = init_cx();
        let owner = DrawList::new(&mut cx);
        let (dock, sends) = retained_tabs(&mut cx, &owner, &[id!(tab_a), id!(tab_b)]);
        let (_root, tree) = indexed_dock(dock);
        let before = visible_send_uids(&tree, &cx);
        let _borrowed = sends[0].borrow::<AreaWidget>().unwrap();
        assert_eq!(visible_send_uids(&tree, &cx), before, "metadata reads need no exclusive borrow");
        assert_eq!(tree.query_rects(&cx, "id:send").len(), 1);
    }

    #[test]
    fn interaction_visibility_find_refs_propagate_unavailable() {
        let mut cx = init_cx();
        let owner = DrawList::new(&mut cx);
        let (left, _) = retained_tabs(&mut cx, &owner, &[id!(tab_a), id!(tab_b)]);
        let (right, _) = retained_tabs(&mut cx, &owner, &[id!(tab_c), id!(tab_d)]);
        let left = WidgetRef::new_with_inner(Box::new(left));
        let right = WidgetRef::new_with_inner(Box::new(right));
        let wrapper = FindDockRefs { uid: WidgetUid::new(), left: left.clone(), right };
        let borrowed = left.borrow_mut::<Dock>().unwrap();
        let mut reports = Vec::new();
        assert!(!wrapper.interaction_child_visibility(&mut |uid, visible| reports.push((uid, visible))));
        assert_eq!(reports.len(), 2, "later find fields must still report after an unavailable field");
        drop(borrowed);
        assert!(wrapper.interaction_child_visibility(&mut |_, _| {}));
    }

    #[test]
    fn interaction_visibility_selected_child_own_hidden() {
        let mut cx = init_cx();
        let owner = DrawList::new(&mut cx);
        let (mut dock, sends) = retained_tabs(&mut cx, &owner, &[id!(tab_a), id!(tab_b)]);
        let visible = Rc::new(Cell::new(false));
        let body = area_widget(Area::Empty, visible.clone(), vec![(id!(send), sends[0].clone())]);
        dock.items.insert(id!(tab_a), (id!(Panel), body));
        let (_root, tree) = indexed_dock(dock);
        assert!(visible_send_uids(&tree, &cx).is_empty());
        assert!(tree.query_rects(&cx, "id:send").is_empty());
        visible.set(true);
        assert_eq!(visible_send_uids(&tree, &cx), vec![sends[0].widget_uid().0.to_string()]);
    }

    #[test]
    fn interaction_visibility_unavailable_retained_body() {
        let mut cx = init_cx();
        let owner = DrawList::new(&mut cx);
        let (mut outer, _) = retained_tabs(&mut cx, &owner, &[id!(tab_a), id!(tab_b)]);
        let (inner, _) = retained_tabs(&mut cx, &owner, &[id!(inner_a), id!(inner_b)]);
        let inner = WidgetRef::new_with_inner(Box::new(inner));
        outer.items.insert(id!(tab_a), (id!(Panel), inner.clone()));
        outer.items.insert(id!(empty), (id!(Panel), WidgetRef::empty()));
        assert!(outer.interaction_child_visibility(&mut |_, _| {}));
        let borrowed = inner.borrow_mut::<Dock>().unwrap();
        assert!(!outer.interaction_child_visibility(&mut |_, _| {}));
        drop(borrowed);
        assert!(outer.interaction_child_visibility(&mut |_, _| {}));
    }

    struct StartupCounter { uid: WidgetUid, count: Rc<Cell<usize>> }
    impl ScriptApply for StartupCounter {}
    impl WidgetNode for StartupCounter {
        fn widget_uid(&self) -> WidgetUid { self.uid }
        fn area(&self) -> Area { Area::Empty }
        fn walk(&mut self, _cx: &mut Cx) -> Walk { Walk::default() }
        fn redraw(&mut self, _cx: &mut Cx) {}
    }
    impl Widget for StartupCounter {
        fn handle_event(&mut self, _cx: &mut Cx, event: &Event, _scope: &mut Scope) {
            if matches!(event, Event::Startup) { self.count.set(self.count.get() + 1); }
        }
    }

    #[test]
    fn interaction_visibility_retains_nonvisible_events() {
        let mut cx = init_cx();
        let owner = DrawList::new(&mut cx);
        let (mut dock, _) = retained_tabs(&mut cx, &owner, &[id!(tab_a), id!(tab_b)]);
        let a = Rc::new(Cell::new(0));
        let b = Rc::new(Cell::new(0));
        for (id, count) in [(id!(tab_a), a.clone()), (id!(tab_b), b.clone())] {
            dock.items.insert(id, (id!(Panel), WidgetRef::new_with_inner(Box::new(StartupCounter { uid: WidgetUid::new(), count }))));
        }
        assert!(!Event::Startup.requires_visibility());
        dock.handle_event(&mut cx, &Event::Startup, &mut Scope::empty());
        assert_eq!((a.get(), b.get()), (1, 1));
        dock.select_tab(&mut cx, id!(tab_b));
        dock.handle_event(&mut cx, &Event::Startup, &mut Scope::empty());
        assert_eq!((a.get(), b.get()), (2, 2));
    }

    #[test]
    fn preserving_layout_keeps_absent_and_matching_tab_bodies() {
        let present = LiveId(1);
        let absent = LiveId(2);
        let kind = LiveId(3);
        let mut layout = HashMap::new();
        layout.insert(present, DockItem::tab("present".into(), kind, LiveId(4)));

        assert!(preserve_item_for_layout(&layout, present, kind));
        assert!(preserve_item_for_layout(&layout, absent, kind));
        assert!(!preserve_item_for_layout(&layout, present, LiveId(5)));
    }

    #[test]
    fn preserving_layout_drops_a_tab_when_its_id_becomes_a_container() {
        let reused = LiveId(1);
        let mut layout = HashMap::new();
        layout.insert(reused, DockItem::tabs(Vec::new(), 0, false));

        assert!(!preserve_item_for_layout(&layout, reused, LiveId(2)));
    }
}
