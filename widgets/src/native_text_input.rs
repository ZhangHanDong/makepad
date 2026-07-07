use crate::{
    makepad_derive_widget::*,
    makepad_draw::*,
    makepad_platform::{
        NativeTextInputChanged, NativeTextInputCloseRequested, NativeTextInputFocusChanged,
        NativeTextInputId, NativeTextInputSelectionChanged,
    },
    makepad_script::ScriptFnRef,
    widget::*,
    widget_async::ScriptAsyncResult,
};

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*

    mod.widgets.NativeTextInputBase = #(NativeTextInput::register_widget(vm))

    mod.widgets.NativeTextInput = set_type_default() do mod.widgets.NativeTextInputBase{
        width: Fill
        height: 32
        editable: true
        secure: false
        draw_bg +: {
            color: #0000
        }
    }
}

#[derive(Script, ScriptHook, Widget)]
pub struct NativeTextInput {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[walk]
    walk: Walk,
    #[redraw]
    #[live]
    draw_bg: DrawColor,
    #[live]
    text: ArcStringMut,
    #[live]
    placeholder: ArcStringMut,
    #[live(true)]
    editable: bool,
    #[live(false)]
    secure: bool,
    #[visible]
    #[live(true)]
    visible: bool,
    #[live]
    on_change: Option<ScriptFnRef>,
    #[rust]
    spawned: bool,
    #[rust]
    last_text: String,
    #[rust]
    last_placeholder: String,
    #[rust]
    last_editable: bool,
    #[rust]
    last_secure: bool,
    // Layout dedup: on OHOS every host call is a blocking JS-thread round
    // trip, so update() must only fire when the resolved rect changed.
    #[rust]
    last_sync_rect: Option<Rect>,
    #[rust]
    native_detached: bool,
    #[rust]
    selection_start: usize,
    #[rust]
    selection_end: usize,
}

impl NativeTextInput {
    fn native_id(&self) -> NativeTextInputId {
        NativeTextInputId(LiveId(self.uid.0))
    }

    fn sync_native(&mut self, cx: &mut Cx) {
        let id = self.native_id();
        let text = self.text.as_ref();
        let placeholder = self.placeholder.as_ref();
        if !self.spawned {
            cx.native_text_input(id)
                .spawn(text, placeholder, self.editable, self.secure);
            self.spawned = true;
            self.last_text.clear();
            self.last_text.push_str(text);
            self.last_placeholder.clear();
            self.last_placeholder.push_str(placeholder);
            self.last_editable = self.editable;
            self.last_secure = self.secure;
            self.last_sync_rect = None;
        }
        if self.last_text != text {
            cx.native_text_input(id).set_text(text, true);
            self.last_text.clear();
            self.last_text.push_str(text);
        }
        if self.last_placeholder != placeholder {
            cx.native_text_input(id).set_placeholder(placeholder);
            self.last_placeholder.clear();
            self.last_placeholder.push_str(placeholder);
        }
        if self.last_editable != self.editable {
            cx.native_text_input(id).set_editable(self.editable);
            self.last_editable = self.editable;
        }
        if self.last_secure != self.secure {
            cx.native_text_input(id).set_secure(self.secure);
            self.last_secure = self.secure;
        }
        let rect = self.draw_bg.area().clipped_rect(cx);
        if self.last_sync_rect != Some(rect) {
            cx.native_text_input(id)
                .update(self.draw_bg.area(), self.visible);
            self.last_sync_rect = Some(rect);
        }
        self.native_detached = false;
    }

    fn set_text_internal(&mut self, cx: &mut Cx, text: &str) {
        self.text.as_mut_empty().push_str(text);
        self.last_text.clear();
        self.last_text.push_str(text);
        if self.spawned {
            cx.native_text_input(self.native_id()).set_text(text, true);
        }
        self.draw_bg.redraw(cx);
    }

    fn close_native(&mut self, cx: &mut Cx) {
        self.visible = false;
        if self.spawned {
            cx.native_text_input(self.native_id()).close();
            self.spawned = false;
            self.last_sync_rect = None;
            self.native_detached = false;
        }
    }

    fn emit_change(&mut self, cx: &mut Cx, uid: WidgetUid, text: String) {
        self.text.as_mut_empty().push_str(&text);
        self.last_text.clear();
        self.last_text.push_str(&text);
        cx.widget_action(uid, NativeTextInputAction::Changed(text.clone()));
        if let Some(handler) = self.on_change.as_object() {
            cx.with_vm(|vm| {
                let str_val = vm.bx.heap.new_string_from_str(&text);
                vm.call(ScriptValue::from(handler), &[ScriptValue::from(str_val)]);
            });
        }
        self.draw_bg.redraw(cx);
    }

    fn emit_selection_change(&mut self, cx: &mut Cx, uid: WidgetUid, start: usize, end: usize) {
        self.selection_start = start.min(end);
        self.selection_end = start.max(end);
        cx.widget_action(
            uid,
            NativeTextInputAction::SelectionChanged {
                start: self.selection_start,
                end: self.selection_end,
            },
        );
    }
}

impl Widget for NativeTextInput {
    fn script_call(
        &mut self,
        vm: &mut ScriptVm,
        method: LiveId,
        args: ScriptValue,
    ) -> ScriptAsyncResult {
        if method == live_id!(text) {
            let str_val = vm.bx.heap.new_string_from_str(self.text.as_ref());
            return ScriptAsyncResult::Return(str_val.into());
        }
        if method == live_id!(set_text) {
            if let Some(args_obj) = args.as_object() {
                let trap = vm.bx.threads.cur().trap.pass();
                let value = vm.bx.heap.vec_value(args_obj, 0, trap);
                if !value.is_err() {
                    let new_text = vm.bx.heap.temp_string_with(|heap, out| {
                        heap.cast_to_string(value, out);
                        out.to_string()
                    });
                    vm.with_cx_mut(|cx| {
                        self.set_text_internal(cx, &new_text);
                    });
                }
            }
            return ScriptAsyncResult::Return(NIL);
        }
        if method == live_id!(focus) {
            vm.with_cx_mut(|cx| {
                cx.native_text_input(self.native_id())
                    .command(live_id!(focus));
            });
            return ScriptAsyncResult::Return(NIL);
        }
        if method == live_id!(blur) {
            vm.with_cx_mut(|cx| {
                cx.native_text_input(self.native_id())
                    .command(live_id!(blur));
            });
            return ScriptAsyncResult::Return(NIL);
        }
        if method == live_id!(select_all) {
            vm.with_cx_mut(|cx| {
                cx.native_text_input(self.native_id())
                    .command(live_id!(select_all));
            });
            return ScriptAsyncResult::Return(NIL);
        }
        if method == live_id!(copy) {
            vm.with_cx_mut(|cx| {
                cx.native_text_input(self.native_id())
                    .command(live_id!(copy));
            });
            return ScriptAsyncResult::Return(NIL);
        }
        if method == live_id!(cut) {
            vm.with_cx_mut(|cx| {
                cx.native_text_input(self.native_id())
                    .command(live_id!(cut));
            });
            return ScriptAsyncResult::Return(NIL);
        }
        if method == live_id!(paste) {
            vm.with_cx_mut(|cx| {
                cx.native_text_input(self.native_id())
                    .command(live_id!(paste));
            });
            return ScriptAsyncResult::Return(NIL);
        }
        ScriptAsyncResult::MethodNotFound
    }

    fn text(&self) -> String {
        self.text.as_ref().to_string()
    }

    fn set_text(&mut self, cx: &mut Cx, text: &str) {
        self.set_text_internal(cx, text);
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        match event {
            Event::Shutdown => {
                if self.spawned {
                    cx.native_text_input(self.native_id()).close();
                    self.spawned = false;
                    self.last_sync_rect = None;
                    self.native_detached = false;
                }
            }
            Event::Actions(actions) => {
                for action in actions {
                    if let Some(change) = action.downcast_ref::<NativeTextInputChanged>() {
                        if change.text_input_id == self.native_id().0 {
                            self.emit_change(cx, self.uid, change.text.clone());
                        }
                    }
                    if let Some(focus) = action.downcast_ref::<NativeTextInputFocusChanged>() {
                        if focus.text_input_id == self.native_id().0 {
                            if focus.has_focus {
                                cx.widget_action(self.uid, NativeTextInputAction::KeyFocus);
                            } else {
                                cx.widget_action(self.uid, NativeTextInputAction::KeyFocusLost);
                            }
                        }
                    }
                    if let Some(selection) =
                        action.downcast_ref::<NativeTextInputSelectionChanged>()
                    {
                        if selection.text_input_id == self.native_id().0 {
                            self.emit_selection_change(
                                cx,
                                self.uid,
                                selection.start,
                                selection.end,
                            );
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        self.draw_bg.draw_walk(cx, walk);
        if self.visible {
            self.sync_native(cx);
        } else if self.spawned && !self.native_detached {
            cx.native_text_input(self.native_id()).detach();
            self.native_detached = true;
            // Force a layout re-send when the input becomes visible again.
            self.last_sync_rect = None;
        }
        DrawStep::done()
    }
}

impl Drop for NativeTextInput {
    fn drop(&mut self) {
        if self.spawned {
            let _ = Cx::try_post_action(NativeTextInputCloseRequested::new(self.native_id()));
        }
    }
}

impl NativeTextInputRef {
    pub fn text(&self) -> String {
        self.borrow()
            .map(|inner| inner.text.as_ref().to_string())
            .unwrap_or_default()
    }

    pub fn selection(&self) -> (usize, usize) {
        self.borrow()
            .map(|inner| (inner.selection_start, inner.selection_end))
            .unwrap_or_default()
    }

    pub fn changed(&self, actions: &Actions) -> Option<String> {
        for action in actions.filter_widget_actions_cast::<NativeTextInputAction>(self.widget_uid())
        {
            if let NativeTextInputAction::Changed(text) = action {
                return Some(text);
            }
        }
        None
    }

    pub fn key_focus(&self, actions: &Actions) -> bool {
        for action in actions.filter_widget_actions_cast::<NativeTextInputAction>(self.widget_uid())
        {
            if let NativeTextInputAction::KeyFocus = action {
                return true;
            }
        }
        false
    }

    pub fn key_focus_lost(&self, actions: &Actions) -> bool {
        for action in actions.filter_widget_actions_cast::<NativeTextInputAction>(self.widget_uid())
        {
            if let NativeTextInputAction::KeyFocusLost = action {
                return true;
            }
        }
        false
    }

    pub fn selection_changed(&self, actions: &Actions) -> Option<(usize, usize)> {
        for action in actions.filter_widget_actions_cast::<NativeTextInputAction>(self.widget_uid())
        {
            if let NativeTextInputAction::SelectionChanged { start, end } = action {
                return Some((start, end));
            }
        }
        None
    }

    pub fn focus(&self, cx: &mut Cx) {
        if let Some(inner) = self.borrow() {
            cx.native_text_input(inner.native_id())
                .command(live_id!(focus));
        }
    }

    pub fn blur(&self, cx: &mut Cx) {
        if let Some(inner) = self.borrow() {
            cx.native_text_input(inner.native_id())
                .command(live_id!(blur));
        }
    }

    pub fn select_all(&self, cx: &mut Cx) {
        if let Some(inner) = self.borrow() {
            cx.native_text_input(inner.native_id())
                .command(live_id!(select_all));
        }
    }

    pub fn copy(&self, cx: &mut Cx) {
        if let Some(inner) = self.borrow() {
            cx.native_text_input(inner.native_id())
                .command(live_id!(copy));
        }
    }

    pub fn cut(&self, cx: &mut Cx) {
        if let Some(inner) = self.borrow() {
            cx.native_text_input(inner.native_id())
                .command(live_id!(cut));
        }
    }

    pub fn paste(&self, cx: &mut Cx) {
        if let Some(inner) = self.borrow() {
            cx.native_text_input(inner.native_id())
                .command(live_id!(paste));
        }
    }

    pub fn set_text(&self, cx: &mut Cx, text: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_text_internal(cx, text);
        }
    }

    pub fn close(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.close_native(cx);
        }
    }
}

#[derive(Clone, Debug, Default)]
pub enum NativeTextInputAction {
    #[default]
    None,
    KeyFocus,
    KeyFocusLost,
    Changed(String),
    SelectionChanged {
        start: usize,
        end: usize,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_cx() -> Cx {
        Cx::new(Box::new(|_cx: &mut Cx, _event: &Event| {}))
    }

    fn test_input(cx: &mut Cx, uid: WidgetUid) -> NativeTextInput {
        let draw_bg = cx.with_vm(DrawColor::script_new);
        NativeTextInput {
            uid,
            source: Default::default(),
            walk: Default::default(),
            draw_bg,
            text: Default::default(),
            placeholder: Default::default(),
            editable: true,
            secure: false,
            visible: true,
            on_change: None,
            spawned: false,
            last_text: String::new(),
            last_placeholder: String::new(),
            last_editable: true,
            last_secure: false,
            last_sync_rect: None,
            native_detached: false,
            selection_start: 0,
            selection_end: 0,
        }
    }

    #[test]
    fn native_changed_event_updates_text_and_emits_widget_action() {
        let mut cx = test_cx();
        let uid = WidgetUid(301);
        let mut input = test_input(&mut cx, uid);
        let event = Event::Actions(vec![Box::new(NativeTextInputChanged::new(
            input.native_id(),
            "typed from native",
        ))]);

        let actions = cx.capture_actions(|cx| {
            input.handle_event(cx, &event, &mut Scope::empty());
        });

        assert_eq!(input.text(), "typed from native");
        assert!(matches!(
            actions.find_widget_action_cast::<NativeTextInputAction>(uid),
            NativeTextInputAction::Changed(text) if text == "typed from native"
        ));
    }

    #[test]
    fn native_focus_events_emit_widget_actions() {
        let mut cx = test_cx();
        let uid = WidgetUid(302);
        let mut input = test_input(&mut cx, uid);
        let focus_event = Event::Actions(vec![Box::new(NativeTextInputFocusChanged::new(
            input.native_id(),
            true,
        ))]);
        let blur_event = Event::Actions(vec![Box::new(NativeTextInputFocusChanged::new(
            input.native_id(),
            false,
        ))]);

        let focus_actions = cx.capture_actions(|cx| {
            input.handle_event(cx, &focus_event, &mut Scope::empty());
        });
        let blur_actions = cx.capture_actions(|cx| {
            input.handle_event(cx, &blur_event, &mut Scope::empty());
        });

        assert!(matches!(
            focus_actions.find_widget_action_cast::<NativeTextInputAction>(uid),
            NativeTextInputAction::KeyFocus
        ));
        assert!(matches!(
            blur_actions.find_widget_action_cast::<NativeTextInputAction>(uid),
            NativeTextInputAction::KeyFocusLost
        ));
    }

    #[test]
    fn native_selection_event_updates_selection_and_emits_widget_action() {
        let mut cx = test_cx();
        let uid = WidgetUid(303);
        let mut input = test_input(&mut cx, uid);
        let event = Event::Actions(vec![Box::new(NativeTextInputSelectionChanged::new(
            input.native_id(),
            7,
            3,
        ))]);

        let actions = cx.capture_actions(|cx| {
            input.handle_event(cx, &event, &mut Scope::empty());
        });

        assert_eq!((input.selection_start, input.selection_end), (3, 7));
        assert!(matches!(
            actions.find_widget_action_cast::<NativeTextInputAction>(uid),
            NativeTextInputAction::SelectionChanged { start: 3, end: 7 }
        ));
    }

    #[test]
    fn native_events_for_other_ids_are_ignored() {
        let mut cx = test_cx();
        let uid = WidgetUid(304);
        let mut input = test_input(&mut cx, uid);
        let event = Event::Actions(vec![
            Box::new(NativeTextInputChanged::new(LiveId(999), "wrong input")),
            Box::new(NativeTextInputFocusChanged::new(LiveId(999), true)),
            Box::new(NativeTextInputSelectionChanged::new(LiveId(999), 1, 2)),
        ]);

        let actions = cx.capture_actions(|cx| {
            input.handle_event(cx, &event, &mut Scope::empty());
        });

        assert_eq!(input.text(), "");
        assert_eq!((input.selection_start, input.selection_end), (0, 0));
        assert!(matches!(
            actions.find_widget_action_cast::<NativeTextInputAction>(uid),
            NativeTextInputAction::None
        ));
    }

    #[test]
    fn set_text_updates_widget_state() {
        let mut cx = test_cx();
        let mut input = test_input(&mut cx, WidgetUid(305));

        input.set_text(&mut cx, "before spawn");
        assert_eq!(input.text(), "before spawn");
        assert_eq!(input.last_text, "before spawn");

        input.spawned = true;
        input.set_text(&mut cx, "after spawn");
        assert_eq!(input.text(), "after spawn");
        assert_eq!(input.last_text, "after spawn");
    }

    #[test]
    fn shutdown_marks_spawned_native_text_input_closed() {
        let mut cx = test_cx();
        let mut input = test_input(&mut cx, WidgetUid(306));
        input.spawned = true;

        input.handle_event(&mut cx, &Event::Shutdown, &mut Scope::empty());

        assert!(!input.spawned);
    }

    #[test]
    fn close_native_hides_and_marks_native_text_input_closed() {
        let mut cx = test_cx();
        let mut input = test_input(&mut cx, WidgetUid(307));
        input.spawned = true;

        input.close_native(&mut cx);

        assert!(!input.visible);
        assert!(!input.spawned);
    }

    #[test]
    fn widget_ref_reads_native_text_input_text() {
        let mut cx = test_cx();
        let mut input = test_input(&mut cx, WidgetUid(308));
        input.set_text(&mut cx, "readable");
        let input_ref = WidgetRef::new_with_inner(Box::new(input));

        assert_eq!(input_ref.text(), "readable");
    }
}
