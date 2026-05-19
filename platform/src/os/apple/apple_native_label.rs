#[cfg(target_os = "macos")]
use crate::window::WindowId;
use crate::{
    makepad_math::Rect,
    os::apple::{
        apple_native_host::{clipped_native_host_layout, order_native_host_view, MacosNativeHost},
        apple_sys::*,
        apple_util::str_to_nsstring,
    },
};
use makepad_objc_sys::{class, msg_send};

#[cfg(target_os = "macos")]
pub(crate) struct MacosNativeLabel {
    attached_window: Option<WindowId>,
    host_view: ObjcId,
    field: ObjcId,
    text: String,
}

#[cfg(target_os = "macos")]
impl MacosNativeLabel {
    pub(crate) fn new(text: &str) -> Self {
        let mut label = Self {
            attached_window: None,
            host_view: nil,
            field: nil,
            text: String::new(),
        };
        label.ensure_field();
        label.set_text(text);
        label
    }

    fn ensure_field(&mut self) {
        if self.field != nil {
            return;
        }
        unsafe {
            let field: ObjcId = msg_send![class!(NSTextField), alloc];
            let field: ObjcId = msg_send![field, initWithFrame: NSRect {
                origin: NSPoint { x: 0.0, y: 0.0 },
                size: NSSize { width: 1.0, height: 22.0 },
            }];
            if field != nil {
                let () = msg_send![field, setHidden: YES];
                let () = msg_send![field, setBezeled: NO];
                let () = msg_send![field, setDrawsBackground: NO];
                let () = msg_send![field, setEditable: NO];
                let () = msg_send![field, setSelectable: NO];
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

    pub(crate) fn set_text(&mut self, text: &str) {
        self.ensure_field();
        self.text.clear();
        self.text.push_str(text);
        unsafe {
            if self.field != nil {
                let text = str_to_nsstring(text);
                if text != nil {
                    let () = msg_send![self.field, setStringValue: text];
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
            if self.field != nil {
                let () = msg_send![self.field, release];
                self.field = nil;
            }
            if self.host_view != nil {
                let () = msg_send![self.host_view, release];
                self.host_view = nil;
            }
        }
    }
}

#[cfg(target_os = "macos")]
impl Drop for MacosNativeLabel {
    fn drop(&mut self) {
        self.cleanup();
    }
}

#[cfg(target_os = "macos")]
impl MacosNativeHost for MacosNativeLabel {
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
