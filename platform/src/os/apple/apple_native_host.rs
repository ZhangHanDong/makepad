#[cfg(target_os = "ios")]
use crate::{makepad_math::Rect, os::apple::apple_sys::ObjcId};
#[cfg(target_os = "macos")]
use crate::{
    makepad_math::Rect,
    os::apple::apple_sys::{nil, ObjcId},
    window::WindowId,
};
#[cfg(target_os = "macos")]
use makepad_objc_sys::{msg_send, sel, sel_impl};
#[cfg(target_os = "macos")]
use std::any::Any;
#[cfg(target_os = "ios")]
use std::any::Any;

#[cfg(target_os = "macos")]
pub(crate) trait MacosNativeHost {
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn update_layout(
        &mut self,
        window_id: WindowId,
        parent_view: ObjcId,
        unclipped_rect: Rect,
        clipped_rect: Rect,
        visible: bool,
    );
    fn detach(&mut self);
    fn cleanup(&mut self);
}

#[cfg(target_os = "macos")]
pub(crate) fn clipped_native_host_layout(
    unclipped_rect: Rect,
    clipped_rect: Rect,
    visible: bool,
) -> (Rect, Rect, bool) {
    let content_rect = Rect {
        pos: unclipped_rect.pos - clipped_rect.pos,
        size: unclipped_rect.size,
    };
    let is_visible = visible && clipped_rect.size.x > 0.0 && clipped_rect.size.y > 0.0;
    (clipped_rect, content_rect, is_visible)
}

#[cfg(target_os = "macos")]
pub(crate) fn order_native_host_view(parent_view: ObjcId, host_view: ObjcId) {
    const NS_WINDOW_ABOVE: i64 = 1;
    if parent_view == nil || host_view == nil {
        return;
    }
    unsafe {
        let () = msg_send![parent_view, addSubview: host_view positioned: NS_WINDOW_ABOVE relativeTo: nil];
    }
}

#[cfg(target_os = "ios")]
pub(crate) trait IosNativeHost {
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn update_layout(&mut self, parent_view: ObjcId, rect: Rect, visible: bool);
    fn detach(&mut self);
    fn cleanup(&mut self);
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::clipped_native_host_layout;
    use crate::makepad_math::{dvec2, Rect};

    #[test]
    fn offsets_content_inside_clipped_host() {
        let unclipped = Rect {
            pos: dvec2(20.0, 40.0),
            size: dvec2(300.0, 32.0),
        };
        let clipped = Rect {
            pos: dvec2(50.0, 40.0),
            size: dvec2(270.0, 32.0),
        };

        let (host, content, visible) = clipped_native_host_layout(unclipped, clipped, true);

        assert_eq!(host, clipped);
        assert_eq!(content.pos, dvec2(-30.0, 0.0));
        assert_eq!(content.size, unclipped.size);
        assert!(visible);
    }

    #[test]
    fn hides_empty_clip() {
        let rect = Rect {
            pos: dvec2(20.0, 40.0),
            size: dvec2(300.0, 32.0),
        };
        let clipped = Rect {
            pos: rect.pos,
            size: dvec2(0.0, 32.0),
        };

        let (_, _, visible) = clipped_native_host_layout(rect, clipped, true);

        assert!(!visible);
    }
}
