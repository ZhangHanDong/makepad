use crate::{
    makepad_derive_widget::*,
    makepad_draw::*,
    makepad_platform::{NativeLabelCloseRequested, NativeLabelId},
    widget::*,
};

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*

    mod.widgets.NativeLabelBase = #(NativeLabel::register_widget(vm))

    mod.widgets.NativeLabel = set_type_default() do mod.widgets.NativeLabelBase{
        width: Fill
        height: 24
        draw_bg +: {
            color: #0000
        }
    }
}

#[derive(Script, ScriptHook, Widget)]
pub struct NativeLabel {
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
    #[visible]
    #[live(true)]
    visible: bool,
    #[rust]
    spawned: bool,
    #[rust]
    last_text: String,
}

impl NativeLabel {
    fn native_id(&self) -> NativeLabelId {
        NativeLabelId(LiveId(self.uid.0))
    }

    fn sync_native(&mut self, cx: &mut Cx) {
        let id = self.native_id();
        let text = self.text.as_ref();
        if !self.spawned {
            cx.native_label(id).spawn(text);
            self.spawned = true;
            self.last_text.clear();
            self.last_text.push_str(text);
        }
        if self.last_text != text {
            cx.native_label(id).set_text(text);
            self.last_text.clear();
            self.last_text.push_str(text);
        }
        cx.native_label(id)
            .update(self.draw_bg.area(), self.visible);
    }

    fn set_text_internal(&mut self, cx: &mut Cx, text: &str) {
        self.text.as_mut_empty().push_str(text);
        self.last_text.clear();
        self.last_text.push_str(text);
        if self.spawned {
            cx.native_label(self.native_id()).set_text(text);
        }
        self.draw_bg.redraw(cx);
    }
}

impl Widget for NativeLabel {
    fn text(&self) -> String {
        self.text.as_ref().to_string()
    }

    fn set_text(&mut self, cx: &mut Cx, text: &str) {
        self.set_text_internal(cx, text);
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        if matches!(event, Event::Shutdown) && self.spawned {
            cx.native_label(self.native_id()).close();
            self.spawned = false;
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        self.draw_bg.draw_walk(cx, walk);
        if self.visible {
            self.sync_native(cx);
        } else if self.spawned {
            cx.native_label(self.native_id()).detach();
        }
        DrawStep::done()
    }
}

impl Drop for NativeLabel {
    fn drop(&mut self) {
        if self.spawned {
            let _ = Cx::try_post_action(NativeLabelCloseRequested::new(self.native_id()));
        }
    }
}

impl NativeLabelRef {
    pub fn set_text(&self, cx: &mut Cx, text: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_text_internal(cx, text);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_cx() -> Cx {
        Cx::new(Box::new(|_cx: &mut Cx, _event: &Event| {}))
    }

    fn test_label(cx: &mut Cx, uid: WidgetUid) -> NativeLabel {
        let draw_bg = cx.with_vm(DrawColor::script_new);
        NativeLabel {
            uid,
            source: Default::default(),
            walk: Default::default(),
            draw_bg,
            text: Default::default(),
            visible: true,
            spawned: false,
            last_text: String::new(),
        }
    }

    #[test]
    fn set_text_updates_widget_state() {
        let mut cx = test_cx();
        let mut label = test_label(&mut cx, WidgetUid(401));

        label.set_text(&mut cx, "before spawn");
        assert_eq!(label.text(), "before spawn");
        assert_eq!(label.last_text, "before spawn");

        label.spawned = true;
        label.set_text(&mut cx, "after spawn");
        assert_eq!(label.text(), "after spawn");
        assert_eq!(label.last_text, "after spawn");
    }

    #[test]
    fn shutdown_marks_spawned_native_label_closed() {
        let mut cx = test_cx();
        let mut label = test_label(&mut cx, WidgetUid(402));
        label.spawned = true;

        label.handle_event(&mut cx, &Event::Shutdown, &mut Scope::empty());

        assert!(!label.spawned);
    }
}
