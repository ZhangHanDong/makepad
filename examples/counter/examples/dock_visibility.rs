//! Owned standalone regression fixture. See tools/dock_visibility_smoke.py.
pub use makepad_widgets;
use makepad_widgets::*;

app_main!(App);

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    let Panel = View{
        width: Fill height: Fill flow: Down spacing: 8
        padding: Inset{left: 20 top: 20}
        send := Button{text: "Send" width: 100 height: 40}
        refreshed := Label{text: "Refresh: 0"}
    }
    let NestedPanel = View{
        width: Fill height: Fill
        inner := Dock{
            width: Fill height: Fill
            root := DockTabs{tabs: [@inner_a, @inner_b] selected: 0 closable: false}
            inner_a := DockTab{name: "Inner A" template: @PermanentTab kind: @Panel}
            inner_b := DockTab{name: "Inner B" template: @PermanentTab kind: @Panel}
            Panel := Panel{}
        }
    }
    startup() do #(App::script_component(vm)){
        ui: Root{
            main_window := Window{
                window.title: "Dock interaction visibility fixture"
                window.inner_size: vec2(900, 520)
                body +: {
                    flow: Down spacing: 8
                    toolbar := View{
                        width: Fill height: Fit flow: Right spacing: 8
                        mode_flat := Button{text: "Flat"}
                        mode_split := Button{text: "Split"}
                        mode_nested := Button{text: "Nested"}
                        refresh_retained := Button{text: "Refresh retained"}
                    }
                    counts := Label{text: "Counts: A=0 B=0 C=0 D=0 E=0 F=0 InnerA=0 InnerB=0"}
                    dock := Dock{
                        width: Fill height: Fill
                        root := DockTabs{tabs: [@tab_a, @tab_b, @tab_c, @tab_d, @tab_e, @tab_f] selected: 0 closable: false}
                        tab_a := DockTab{name: "A" template: @PermanentTab kind: @Panel}
                        tab_b := DockTab{name: "B" template: @PermanentTab kind: @Panel}
                        tab_c := DockTab{name: "C" template: @PermanentTab kind: @Panel}
                        tab_d := DockTab{name: "D" template: @PermanentTab kind: @Panel}
                        tab_e := DockTab{name: "E" template: @PermanentTab kind: @Panel}
                        tab_f := DockTab{name: "F" template: @PermanentTab kind: @Panel}
                        nested_outer := DockTab{name: "Nested" template: @PermanentTab kind: @NestedPanel}
                        Panel := Panel{}
                        NestedPanel := NestedPanel{}
                    }
                }
            }
        }
    }
}

#[derive(Script, ScriptHook)]
pub struct App {
    #[source]
    source: ScriptObjectRef,
    #[live]
    ui: WidgetRef,
    #[rust]
    counts: [usize; 8],
    #[rust]
    refresh: usize,
}

impl MatchEvent for App {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        let ids = [id!(tab_a), id!(tab_b), id!(tab_c), id!(tab_d), id!(tab_e), id!(tab_f)];
        for (index, id) in ids.iter().enumerate() {
            if self.ui.button(cx, &[id!(dock), *id, id!(send)]).clicked(actions) {
                self.counts[index] += 1;
            }
        }
        for (index, id) in [id!(inner_a), id!(inner_b)].iter().enumerate() {
            if self.ui.button(cx, &[id!(dock), id!(nested_outer), id!(inner), *id, id!(send)]).clicked(actions) {
                self.counts[6 + index] += 1;
            }
        }
        self.ui.label(cx, ids!(counts)).set_text(cx, &format!(
            "Counts: A={} B={} C={} D={} E={} F={} InnerA={} InnerB={}",
            self.counts[0], self.counts[1], self.counts[2], self.counts[3],
            self.counts[4], self.counts[5], self.counts[6], self.counts[7],
        ));

        if self.ui.button(cx, ids!(refresh_retained)).clicked(actions) {
            self.refresh += 1;
            for id in ids {
                self.ui.label(cx, &[id!(dock), id, id!(refreshed)]).set_text(cx, &format!("Refresh: {}", self.refresh));
            }
        }

        let flat = self.ui.button(cx, ids!(mode_flat)).clicked(actions);
        let split = self.ui.button(cx, ids!(mode_split)).clicked(actions);
        let nested = self.ui.button(cx, ids!(mode_nested)).clicked(actions);
        if flat || split || nested {
            let dock = self.ui.dock(cx, ids!(dock));
            let mut layout = dock.clone_state().expect("fixture Dock exists");
            if split {
                layout.insert(id!(root), DockItem::Splitter {
                    axis: SplitterAxis::Horizontal, align: SplitterAlign::Weighted(0.5),
                    a: id!(left), b: id!(right),
                });
                layout.insert(id!(left), DockItem::tabs(ids[..3].to_vec(), 0, false));
                layout.insert(id!(right), DockItem::tabs(ids[3..].to_vec(), 0, false));
            } else {
                let tabs = if nested { vec![id!(nested_outer), id!(tab_f)] } else { ids.to_vec() };
                layout.insert(id!(root), DockItem::tabs(tabs, 0, false));
            }
            dock.load_state_preserving_items(cx, layout);
            self.ui.redraw(cx);
        }
    }
}

impl AppMain for App {
    fn script_mod(vm: &mut ScriptVm) -> ScriptValue {
        crate::makepad_widgets::script_mod(vm);
        self::script_mod(vm)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        self.match_event(cx, event);
        self.ui.handle_event(cx, event, &mut Scope::empty());
    }
}
