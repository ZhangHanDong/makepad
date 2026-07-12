#[cfg(target_os = "macos")]
use crate::window::WindowId;
use crate::{
    cx_api::{
        NativeTextInputChanged, NativeTextInputFocusChanged, NativeTextInputSelectionChanged,
    },
    makepad_live_id::LiveId,
    makepad_math::Rect,
    os::apple::{
        apple_native_host::{clipped_native_host_layout, order_native_host_view, MacosNativeHost},
        apple_sys::*,
        apple_util::{nsstring_to_string, str_to_nsstring},
    },
};
use makepad_objc_sys::{class, msg_send, sel};
#[cfg(feature = "diag-native-leak-count")]
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::OnceLock;

#[cfg(all(target_os = "macos", feature = "diag-native-leak-count"))]
static NATIVE_TEXT_INPUT_LIVE_COUNT: AtomicI32 = AtomicI32::new(0);

#[cfg(all(target_os = "macos", feature = "diag-native-leak-count"))]
pub fn native_text_input_live_count() -> i32 {
    NATIVE_TEXT_INPUT_LIVE_COUNT.load(Ordering::SeqCst)
}

#[cfg(target_os = "macos")]
fn native_text_input_delegate_class() -> *const Class {
    unsafe fn text_input_id(this: &Object) -> LiveId {
        LiveId(*this.get_ivar::<u64>("text_input_id"))
    }

    unsafe fn post_selection_changed(this: &Object, editor: ObjcId) {
        if editor == nil {
            return;
        }
        let range: NSRange = msg_send![editor, selectedRange];
        let start = range.location;
        let end = range.location.saturating_add(range.length);
        let last_start: u64 = *this.get_ivar("selection_start");
        let last_end: u64 = *this.get_ivar("selection_end");
        if last_start == start && last_end == end {
            return;
        }
        let this_mut = this as *const Object as *mut Object;
        (*this_mut).set_ivar("selection_start", start);
        (*this_mut).set_ivar("selection_end", end);
        let _ = crate::Cx::try_post_action(NativeTextInputSelectionChanged::new(
            text_input_id(this),
            start as usize,
            end as usize,
        ));
    }

    extern "C" fn control_text_did_change(this: &Object, _: Sel, notification: ObjcId) {
        unsafe {
            let is_programmatic: BOOL = *this.get_ivar("is_programmatic_update");
            if is_programmatic != NO {
                return;
            }
            let field: ObjcId = msg_send![notification, object];
            if field == nil {
                return;
            }
            let string: ObjcId = msg_send![field, stringValue];
            let text = nsstring_to_string(string);
            let _ =
                crate::Cx::try_post_action(NativeTextInputChanged::new(text_input_id(this), text));
            let editor: ObjcId = msg_send![field, currentEditor];
            post_selection_changed(this, editor);
        }
    }

    extern "C" fn control_text_did_begin_editing(this: &Object, _: Sel, _: ObjcId) {
        unsafe {
            let _ = crate::Cx::try_post_action(NativeTextInputFocusChanged::new(
                text_input_id(this),
                true,
            ));
        }
    }

    extern "C" fn control_text_did_end_editing(this: &Object, _: Sel, _: ObjcId) {
        unsafe {
            let _ = crate::Cx::try_post_action(NativeTextInputFocusChanged::new(
                text_input_id(this),
                false,
            ));
        }
    }

    extern "C" fn text_view_did_change_selection(this: &Object, _: Sel, notification: ObjcId) {
        unsafe {
            let editor: ObjcId = msg_send![notification, object];
            post_selection_changed(this, editor);
        }
    }

    static CLASS: OnceLock<usize> = OnceLock::new();
    *CLASS.get_or_init(|| {
        let superclass = class!(NSObject);
        let mut decl = ClassDecl::new("MakepadNativeTextInputDelegate", superclass).unwrap();
        unsafe {
            decl.add_method(
                sel!(controlTextDidChange:),
                control_text_did_change as extern "C" fn(&Object, Sel, ObjcId),
            );
            decl.add_method(
                sel!(controlTextDidBeginEditing:),
                control_text_did_begin_editing as extern "C" fn(&Object, Sel, ObjcId),
            );
            decl.add_method(
                sel!(controlTextDidEndEditing:),
                control_text_did_end_editing as extern "C" fn(&Object, Sel, ObjcId),
            );
            decl.add_method(
                sel!(textViewDidChangeSelection:),
                text_view_did_change_selection as extern "C" fn(&Object, Sel, ObjcId),
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

#[cfg(target_os = "macos")]
pub(crate) struct MacosNativeTextInput {
    text_input_id: LiveId,
    attached_window: Option<WindowId>,
    host_view: ObjcId,
    field: ObjcId,
    delegate: ObjcId,
    text: String,
    placeholder: String,
    editable: bool,
    secure: bool,
    #[cfg(feature = "diag-native-leak-count")]
    counted: bool,
}

#[cfg(target_os = "macos")]
impl MacosNativeTextInput {
    pub(crate) fn new(
        text_input_id: LiveId,
        text: &str,
        placeholder: &str,
        editable: bool,
        secure: bool,
    ) -> Self {
        let mut input = Self {
            text_input_id,
            attached_window: None,
            host_view: nil,
            field: nil,
            delegate: nil,
            text: String::new(),
            placeholder: String::new(),
            editable,
            secure,
            #[cfg(feature = "diag-native-leak-count")]
            counted: true,
        };
        #[cfg(feature = "diag-native-leak-count")]
        crate::log!(
            "NativeTextInput live count: {}",
            NATIVE_TEXT_INPUT_LIVE_COUNT.fetch_add(1, Ordering::SeqCst) + 1
        );
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
            let delegate: ObjcId = msg_send![native_text_input_delegate_class(), new];
            if delegate != nil {
                (*delegate).set_ivar("text_input_id", self.text_input_id.0);
                (*delegate).set_ivar::<BOOL>("is_programmatic_update", NO);
                (*delegate).set_ivar("selection_start", 0_u64);
                (*delegate).set_ivar("selection_end", 0_u64);
                self.delegate = delegate;
            }

            // NSTextField cannot toggle secure entry at runtime without
            // swapping the cell class, so the secure/plain distinction is
            // baked in at alloc time via the class choice here. Runtime
            // toggling is handled as a logged no-op (see set_secure below).
            let field_class = if self.secure {
                class!(NSSecureTextField)
            } else {
                class!(NSTextField)
            };
            let field: ObjcId = msg_send![field_class, alloc];
            let field: ObjcId = msg_send![field, initWithFrame: NSRect {
                origin: NSPoint { x: 0.0, y: 0.0 },
                size: NSSize { width: 1.0, height: 22.0 },
            }];
            if field != nil {
                let () = msg_send![field, setHidden: YES];
                let () = msg_send![field, setBezeled: YES];
                let () = msg_send![field, setEditable: if self.editable { YES } else { NO }];
                if self.delegate != nil {
                    let () = msg_send![field, setDelegate: self.delegate];
                }
                self.field = field;
            }
        }
    }

    fn ensure_host_view(&mut self) {
        if self.host_view != nil {
            return;
        }
        unsafe {
            let host_view: ObjcId = msg_send![class!(NSView), alloc];
            let host_view: ObjcId = msg_send![host_view, initWithFrame: NSRect {
                origin: NSPoint { x: 0.0, y: 0.0 },
                size: NSSize { width: 1.0, height: 1.0 },
            }];
            if host_view != nil {
                let () = msg_send![host_view, setWantsLayer: YES];
                let layer: ObjcId = msg_send![host_view, layer];
                if layer != nil {
                    let () = msg_send![layer, setMasksToBounds: YES];
                }
                let () = msg_send![host_view, setHidden: YES];
                self.host_view = host_view;
            }
        }
    }

    fn ensure_attached(&mut self, window_id: WindowId, parent_view: ObjcId) {
        self.ensure_field();
        self.ensure_host_view();
        if self.field == nil || self.host_view == nil {
            return;
        }
        unsafe {
            let host_super_view: ObjcId = msg_send![self.host_view, superview];
            if self.attached_window != Some(window_id) || host_super_view != parent_view {
                let () = msg_send![self.host_view, removeFromSuperview];
                let () = msg_send![parent_view, addSubview: self.host_view];
                self.attached_window = Some(window_id);
            }

            let field_super_view: ObjcId = msg_send![self.field, superview];
            if field_super_view != self.host_view {
                let () = msg_send![self.field, removeFromSuperview];
                let () = msg_send![self.host_view, addSubview: self.field];
            }
        }
    }

    pub(crate) fn update(
        &mut self,
        window_id: WindowId,
        parent_view: ObjcId,
        unclipped_rect: Rect,
        clipped_rect: Rect,
        visible: bool,
    ) {
        self.ensure_attached(window_id, parent_view);
        if self.field == nil || self.host_view == nil {
            return;
        }
        unsafe {
            order_native_host_view(parent_view, self.host_view);
            let (host_rect, field_rect, is_visible) =
                clipped_native_host_layout(unclipped_rect, clipped_rect, visible);
            let host_frame = NSRect {
                origin: NSPoint {
                    x: host_rect.pos.x,
                    y: host_rect.pos.y,
                },
                size: NSSize {
                    width: host_rect.size.x.max(0.0),
                    height: host_rect.size.y.max(0.0),
                },
            };
            let field_frame = NSRect {
                origin: NSPoint {
                    x: field_rect.pos.x,
                    y: field_rect.pos.y,
                },
                size: NSSize {
                    width: field_rect.size.x.max(0.0),
                    height: field_rect.size.y.max(0.0),
                },
            };
            let () = msg_send![self.host_view, setFrame: host_frame];
            let () = msg_send![self.field, setFrame: field_frame];
            let () = msg_send![self.field, setHidden: if is_visible { NO } else { YES }];
            let () = msg_send![self.host_view, setHidden: if is_visible { NO } else { YES }];
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
                    let () = msg_send![self.field, setStringValue: text];
                }
                self.set_programmatic_update(false);
            }
        }
    }

    fn set_programmatic_update(&mut self, programmatic: bool) {
        unsafe {
            if self.delegate != nil {
                (*self.delegate).set_ivar::<BOOL>(
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
                    let () = msg_send![self.field, setPlaceholderString: placeholder];
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
                let () = msg_send![self.field, setEditable: if editable { YES } else { NO }];
            }
        }
    }

    /// NSTextField cannot switch between secure (NSSecureTextField) and
    /// plain entry at runtime without swapping the underlying cell class,
    /// which would require rebuilding the field and its delegate wiring.
    /// The secure flag is only applied at construction time (see `new`);
    /// this is a logged no-op until that refactor is worth doing.
    pub(crate) fn set_secure(&mut self, secure: bool) {
        if self.secure == secure {
            return;
        }
        crate::log!("NativeTextInput: secure toggle not implemented on macOS host yet");
    }

    pub(crate) fn focus(&mut self) {
        self.ensure_field();
        unsafe {
            if self.field != nil {
                let window: ObjcId = msg_send![self.field, window];
                if window != nil {
                    let () = msg_send![window, makeFirstResponder: self.field];
                }
            }
        }
    }

    pub(crate) fn blur(&mut self) {
        unsafe {
            if self.field != nil {
                let window: ObjcId = msg_send![self.field, window];
                if window != nil {
                    let () = msg_send![window, makeFirstResponder: nil];
                }
            }
        }
    }

    pub(crate) fn select_all(&mut self) {
        self.ensure_field();
        unsafe {
            if self.field != nil {
                let () = msg_send![self.field, selectText: nil];
            }
        }
    }

    pub(crate) fn copy(&mut self) {
        self.send_editor_command(sel!(copy:));
    }

    pub(crate) fn cut(&mut self) {
        self.send_editor_command(sel!(cut:));
    }

    pub(crate) fn paste(&mut self) {
        self.send_editor_command(sel!(paste:));
    }

    fn send_editor_command(&mut self, selector: Sel) {
        self.focus();
        unsafe {
            if self.field != nil {
                let editor: ObjcId = msg_send![self.field, currentEditor];
                if editor != nil {
                    let () = msg_send![editor, performSelector: selector withObject: nil];
                }
            }
        }
    }

    pub(crate) fn detach(&mut self) {
        unsafe {
            if self.field != nil {
                let () = msg_send![self.field, removeFromSuperview];
                let () = msg_send![self.field, setHidden: YES];
            }
            if self.host_view != nil {
                let () = msg_send![self.host_view, removeFromSuperview];
                let () = msg_send![self.host_view, setHidden: YES];
            }
        }
        self.attached_window = None;
    }

    pub(crate) fn cleanup(&mut self) {
        self.detach();
        unsafe {
            // Retain count: alloc/init -> 1, addSubview retains -> 2,
            // removeFromSuperview releases -> 1, this release drops our owner -> 0.
            if self.field != nil {
                let () = msg_send![self.field, setDelegate: nil];
                let () = msg_send![self.field, release];
                self.field = nil;
            }
            if self.host_view != nil {
                let () = msg_send![self.host_view, release];
                self.host_view = nil;
            }
            if self.delegate != nil {
                let () = msg_send![self.delegate, release];
                self.delegate = nil;
            }
        }
        #[cfg(feature = "diag-native-leak-count")]
        if self.counted {
            crate::log!(
                "NativeTextInput live count: {}",
                NATIVE_TEXT_INPUT_LIVE_COUNT.fetch_sub(1, Ordering::SeqCst) - 1
            );
            self.counted = false;
        }
    }
}

#[cfg(target_os = "macos")]
impl Drop for MacosNativeTextInput {
    fn drop(&mut self) {
        self.cleanup();
    }
}

#[cfg(target_os = "macos")]
impl MacosNativeHost for MacosNativeTextInput {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn update_layout(
        &mut self,
        window_id: WindowId,
        parent_view: ObjcId,
        unclipped_rect: Rect,
        clipped_rect: Rect,
        visible: bool,
    ) {
        self.update(
            window_id,
            parent_view,
            unclipped_rect,
            clipped_rect,
            visible,
        );
    }

    fn detach(&mut self) {
        self.detach();
    }

    fn cleanup(&mut self) {
        self.cleanup();
    }
}
