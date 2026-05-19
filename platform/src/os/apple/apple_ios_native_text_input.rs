#[cfg(target_os = "ios")]
use crate::{
    cx_api::{
        NativeTextInputChanged, NativeTextInputFocusChanged, NativeTextInputSelectionChanged,
    },
    makepad_live_id::LiveId,
    makepad_math::Rect,
    os::apple::{
        apple_native_host::IosNativeHost,
        apple_sys::*,
        apple_util::{nsstring_to_string, str_to_nsstring},
    },
};
#[cfg(target_os = "ios")]
use makepad_objc_sys::{class, msg_send, sel};
#[cfg(target_os = "ios")]
use std::sync::OnceLock;

#[cfg(target_os = "ios")]
const UI_CONTROL_EVENT_EDITING_DID_BEGIN: usize = 1 << 16;
#[cfg(target_os = "ios")]
const UI_CONTROL_EVENT_EDITING_CHANGED: usize = 1 << 17;
#[cfg(target_os = "ios")]
const UI_CONTROL_EVENT_EDITING_DID_END: usize = 1 << 18;

#[cfg(target_os = "ios")]
fn native_text_input_target_class() -> *const Class {
    unsafe fn text_input_id(this: &Object) -> LiveId {
        LiveId(*this.get_ivar::<u64>("text_input_id"))
    }

    unsafe fn post_selection_changed(this: &Object, sender: ObjcId) {
        let selected_range: ObjcId = msg_send![sender, selectedTextRange];
        if selected_range == nil {
            return;
        }
        let beginning: ObjcId = msg_send![sender, beginningOfDocument];
        let start_position: ObjcId = msg_send![selected_range, start];
        let end_position: ObjcId = msg_send![selected_range, end];
        if beginning == nil || start_position == nil || end_position == nil {
            return;
        }
        let start: isize =
            msg_send![sender, offsetFromPosition: beginning toPosition: start_position];
        let end: isize = msg_send![sender, offsetFromPosition: beginning toPosition: end_position];
        let (Ok(start), Ok(end)) = (usize::try_from(start), usize::try_from(end)) else {
            return;
        };
        let (start, end) = (start.min(end), start.max(end));
        let last_start: u64 = *this.get_ivar("selection_start");
        let last_end: u64 = *this.get_ivar("selection_end");
        let (Ok(start_u64), Ok(end_u64)) = (u64::try_from(start), u64::try_from(end)) else {
            return;
        };
        if last_start == start_u64 && last_end == end_u64 {
            return;
        }
        let this_mut = this as *const Object as *mut Object;
        (*this_mut).set_ivar("selection_start", start_u64);
        (*this_mut).set_ivar("selection_end", end_u64);
        let _ = crate::Cx::try_post_action(NativeTextInputSelectionChanged::new(
            text_input_id(this),
            start,
            end,
        ));
    }

    extern "C" fn editing_changed(this: &Object, _: Sel, sender: ObjcId) {
        unsafe {
            let is_programmatic: BOOL = *this.get_ivar("is_programmatic_update");
            if is_programmatic != NO {
                return;
            }
            let text_obj: ObjcId = msg_send![sender, text];
            let text = nsstring_to_string(text_obj);
            let _ =
                crate::Cx::try_post_action(NativeTextInputChanged::new(text_input_id(this), text));
            post_selection_changed(this, sender);
        }
    }

    extern "C" fn editing_did_begin(this: &Object, _: Sel, _: ObjcId) {
        unsafe {
            let _ = crate::Cx::try_post_action(NativeTextInputFocusChanged::new(
                text_input_id(this),
                true,
            ));
        }
    }

    extern "C" fn editing_did_end(this: &Object, _: Sel, _: ObjcId) {
        unsafe {
            let _ = crate::Cx::try_post_action(NativeTextInputFocusChanged::new(
                text_input_id(this),
                false,
            ));
        }
    }

    extern "C" fn text_field_did_change_selection(this: &Object, _: Sel, sender: ObjcId) {
        unsafe {
            post_selection_changed(this, sender);
        }
    }

    static CLASS: OnceLock<usize> = OnceLock::new();
    *CLASS.get_or_init(|| {
        let superclass = class!(NSObject);
        let mut decl = ClassDecl::new("MakepadIosNativeTextInputTarget", superclass).unwrap();
        unsafe {
            decl.add_method(
                sel!(editingChanged:),
                editing_changed as extern "C" fn(&Object, Sel, ObjcId),
            );
            decl.add_method(
                sel!(editingDidBegin:),
                editing_did_begin as extern "C" fn(&Object, Sel, ObjcId),
            );
            decl.add_method(
                sel!(editingDidEnd:),
                editing_did_end as extern "C" fn(&Object, Sel, ObjcId),
            );
            decl.add_method(
                sel!(textFieldDidChangeSelection:),
                text_field_did_change_selection as extern "C" fn(&Object, Sel, ObjcId),
            );
        }
        decl.add_ivar::<u64>("text_input_id");
        decl.add_ivar::<BOOL>("is_programmatic_update");
        decl.add_ivar::<u64>("selection_start");
        decl.add_ivar::<u64>("selection_end");
        let class: *const Class = decl.register();
        class as usize
    }) as *const Class
}

#[cfg(target_os = "ios")]
pub(crate) struct IosNativeTextInput {
    text_input_id: LiveId,
    field: ObjcId,
    target: ObjcId,
    text: String,
    placeholder: String,
    editable: bool,
}

#[cfg(target_os = "ios")]
impl IosNativeTextInput {
    pub(crate) fn new(
        text_input_id: LiveId,
        text: &str,
        placeholder: &str,
        editable: bool,
    ) -> Self {
        let mut input = Self {
            text_input_id,
            field: nil,
            target: nil,
            text: String::new(),
            placeholder: String::new(),
            editable,
        };
        input.ensure_field();
        input.set_text(text, true);
        input.set_placeholder(placeholder);
        input.set_editable(editable);
        input
    }

    fn ensure_field(&mut self) {
        if self.field != nil {
            return;
        }
        unsafe {
            let target: ObjcId = msg_send![native_text_input_target_class(), new];
            if target != nil {
                (*target).set_ivar("text_input_id", self.text_input_id.0);
                (*target).set_ivar::<BOOL>("is_programmatic_update", NO);
                (*target).set_ivar("selection_start", 0_u64);
                (*target).set_ivar("selection_end", 0_u64);
                self.target = target;
            }

            let field: ObjcId = msg_send![class!(UITextField), alloc];
            let field: ObjcId = msg_send![field, initWithFrame: NSRect {
                origin: NSPoint { x: 0.0, y: 0.0 },
                size: NSSize { width: 1.0, height: 34.0 },
            }];
            if field != nil {
                let () = msg_send![field, setHidden: YES];
                let () = msg_send![field, setBorderStyle: 3isize];
                let () = msg_send![field, setAutocapitalizationType: 0isize];
                let () = msg_send![field, setAutocorrectionType: 1isize];
                let () = msg_send![field, setSpellCheckingType: 1isize];
                let () = msg_send![field, setEnabled: if self.editable { YES } else { NO }];
                if self.target != nil {
                    let () = msg_send![field, setDelegate: self.target];
                    let () = msg_send![field, addTarget: self.target action: sel!(editingChanged:) forControlEvents: UI_CONTROL_EVENT_EDITING_CHANGED];
                    let () = msg_send![field, addTarget: self.target action: sel!(editingDidBegin:) forControlEvents: UI_CONTROL_EVENT_EDITING_DID_BEGIN];
                    let () = msg_send![field, addTarget: self.target action: sel!(editingDidEnd:) forControlEvents: UI_CONTROL_EVENT_EDITING_DID_END];
                }
                self.field = field;
            }
        }
    }

    fn ensure_attached(&mut self, parent_view: ObjcId) {
        self.ensure_field();
        if self.field == nil {
            return;
        }
        unsafe {
            let super_view: ObjcId = msg_send![self.field, superview];
            if super_view != parent_view {
                let () = msg_send![self.field, removeFromSuperview];
                let () = msg_send![parent_view, addSubview: self.field];
            }
        }
    }

    pub(crate) fn update(&mut self, parent_view: ObjcId, rect: Rect, visible: bool) {
        self.ensure_attached(parent_view);
        if self.field == nil {
            return;
        }
        unsafe {
            let frame = NSRect {
                origin: NSPoint {
                    x: rect.pos.x,
                    y: rect.pos.y,
                },
                size: NSSize {
                    width: rect.size.x.max(0.0),
                    height: rect.size.y.max(0.0),
                },
            };
            let () = msg_send![self.field, setFrame: frame];
            let () = msg_send![self.field, setHidden: if visible { NO } else { YES }];
        }
    }

    pub(crate) fn set_text(&mut self, text: &str, programmatic: bool) {
        self.ensure_field();
        self.text.clear();
        self.text.push_str(text);
        unsafe {
            if self.field != nil {
                self.set_programmatic_update(programmatic);
                let text = str_to_nsstring(text);
                if text != nil {
                    let () = msg_send![self.field, setText: text];
                }
                self.set_programmatic_update(false);
            }
        }
    }

    fn set_programmatic_update(&mut self, programmatic: bool) {
        unsafe {
            if self.target != nil {
                (*self.target).set_ivar::<BOOL>(
                    "is_programmatic_update",
                    if programmatic { YES } else { NO },
                );
            }
        }
    }

    pub(crate) fn set_placeholder(&mut self, placeholder: &str) {
        if self.placeholder == placeholder {
            return;
        }
        self.ensure_field();
        self.placeholder.clear();
        self.placeholder.push_str(placeholder);
        unsafe {
            if self.field != nil {
                let placeholder = str_to_nsstring(placeholder);
                if placeholder != nil {
                    let () = msg_send![self.field, setPlaceholder: placeholder];
                }
            }
        }
    }

    pub(crate) fn set_editable(&mut self, editable: bool) {
        if self.editable == editable {
            return;
        }
        self.ensure_field();
        self.editable = editable;
        unsafe {
            if self.field != nil {
                let () = msg_send![self.field, setEnabled: if editable { YES } else { NO }];
            }
        }
    }

    pub(crate) fn focus(&mut self) {
        self.ensure_field();
        unsafe {
            if self.field != nil {
                let () = msg_send![self.field, becomeFirstResponder];
            }
        }
    }

    pub(crate) fn blur(&mut self) {
        unsafe {
            if self.field != nil {
                let () = msg_send![self.field, resignFirstResponder];
            }
        }
    }

    pub(crate) fn select_all(&mut self) {
        self.ensure_field();
        unsafe {
            if self.field != nil {
                let () = msg_send![self.field, becomeFirstResponder];
                let () = msg_send![self.field, selectAll: nil];
            }
        }
    }

    pub(crate) fn copy(&mut self) {
        self.send_edit_command(sel!(copy:));
    }

    pub(crate) fn cut(&mut self) {
        self.send_edit_command(sel!(cut:));
    }

    pub(crate) fn paste(&mut self) {
        self.send_edit_command(sel!(paste:));
    }

    fn send_edit_command(&mut self, selector: Sel) {
        self.ensure_field();
        unsafe {
            if self.field != nil {
                let () = msg_send![self.field, becomeFirstResponder];
                let () = msg_send![self.field, performSelector: selector withObject: nil];
            }
        }
    }

    pub(crate) fn detach(&mut self) {
        unsafe {
            if self.field != nil {
                let () = msg_send![self.field, removeFromSuperview];
                let () = msg_send![self.field, setHidden: YES];
            }
        }
    }

    pub(crate) fn cleanup(&mut self) {
        self.detach();
        unsafe {
            if self.field != nil {
                if self.target != nil {
                    let () = msg_send![self.field, setDelegate: nil];
                    let () = msg_send![self.field, removeTarget: self.target action: nil forControlEvents: UI_CONTROL_EVENT_EDITING_CHANGED | UI_CONTROL_EVENT_EDITING_DID_BEGIN | UI_CONTROL_EVENT_EDITING_DID_END];
                }
                let () = msg_send![self.field, release];
                self.field = nil;
            }
            if self.target != nil {
                let () = msg_send![self.target, release];
                self.target = nil;
            }
        }
    }
}

#[cfg(target_os = "ios")]
impl Drop for IosNativeTextInput {
    fn drop(&mut self) {
        self.cleanup();
    }
}

#[cfg(target_os = "ios")]
impl IosNativeHost for IosNativeTextInput {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn update_layout(&mut self, parent_view: ObjcId, rect: Rect, visible: bool) {
        self.update(parent_view, rect, visible);
    }

    fn detach(&mut self) {
        self.detach();
    }

    fn cleanup(&mut self) {
        self.cleanup();
    }
}
