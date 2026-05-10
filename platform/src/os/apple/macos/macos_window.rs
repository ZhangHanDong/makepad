use {
    crate::{
        area::Area,
        event::{
            finger::MouseButton, DragItem, KeyModifiers, MouseDownEvent, MouseMoveEvent,
            MouseUpEvent, NativeGlassBackendState, NativeGlassBatch, NativeGlassBatchResult,
            NativeGlassBatchValidationError, NativeGlassContainerResult, NativeGlassControlBatch,
            NativeGlassControlBatchValidationError, NativeGlassControlDescriptor,
            NativeGlassHitTest, NativeGlassInstallState, NativeGlassPanelDescriptor,
            NativeGlassPanelResult, NativeGlassStyle, ScrollEvent, TextInputEvent,
            WindowCloseRequestedEvent, WindowClosedEvent, WindowDragQueryEvent,
            WindowDragQueryResponse, WindowGeom, WindowGeomChangeEvent,
            WindowNativeSubstrateResolvedEvent, WindowNativeSubstrateState,
            WindowNativeSubstrateStyle,
        },
        makepad_math::{Rect, Vec2d, Vec4f},
        os::{
            apple::apple_sys::*,
            apple::apple_util::{nsstring_to_string, str_to_nsstring},
            macos::{
                macos_app::{get_macos_class_global, with_macos_app, MacosApp},
                macos_event::MacosEvent,
            },
        },
        window::{
            MacosWindowChrome, MacosWindowConfig, MacosWindowKind, MacosWindowLevel,
            WindowBackdrop, WindowId, WindowVisuals,
        },
        LiveId,
    },
    std::{
        cell::Cell,
        os::raw::{c_char, c_void},
        rc::Rc,
    },
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MacosNativeGlassStyle {
    Regular,
    Clear,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NativeGlassCgEventProbeMode {
    Global,
    Process,
}

impl MacosNativeGlassStyle {
    fn default_ns_style_raw(self) -> i64 {
        match self {
            // Runtime values follow the Swift declaration order:
            // NSGlassEffectView.Style.regular, then .clear.
            Self::Regular => 0,
            Self::Clear => 1,
        }
    }

    fn style_override_env_var(self) -> &'static str {
        match self {
            Self::Regular => "AICHAT_MACOS_GLASS_STYLE_REGULAR_RAW",
            Self::Clear => "AICHAT_MACOS_GLASS_STYLE_CLEAR_RAW",
        }
    }

    fn ns_style_raw(self) -> i64 {
        let var = self.style_override_env_var();
        match std::env::var(var) {
            Ok(value) => match value.trim().parse::<i64>() {
                Ok(raw) => {
                    crate::log!("[liquid-glass] style-override var={} value={}", var, raw);
                    raw
                }
                Err(_) => {
                    crate::log!(
                        "[liquid-glass] style-override-invalid var={} value={}",
                        var,
                        value
                    );
                    self.default_ns_style_raw()
                }
            },
            Err(_) => self.default_ns_style_raw(),
        }
    }

    fn tint_alpha(self) -> f64 {
        match self {
            Self::Regular => 0.12,
            Self::Clear => 0.03,
        }
    }

    pub(crate) fn log_name(self) -> &'static str {
        match self {
            Self::Regular => "regular",
            Self::Clear => "clear",
        }
    }

    fn to_event_style(self) -> WindowNativeSubstrateStyle {
        match self {
            Self::Regular => WindowNativeSubstrateStyle::MacosGlassRegular,
            Self::Clear => WindowNativeSubstrateStyle::MacosGlassClear,
        }
    }
}

#[derive(Clone)]
pub struct MacosWindow {
    pub(crate) window_id: WindowId,
    pub(crate) container_view: ObjcId,
    pub(crate) view: ObjcId,
    pub(crate) window: ObjcId,
    pub(crate) ime_spot: Vec2d,
    // When ime_active is false, key events are not forward to NSTextInputContext so IME dose not active.
    pub(crate) ime_active: bool,
    pub(crate) is_fullscreen: bool,
    pub(crate) is_popup: bool,
    pub(crate) macos_config: MacosWindowConfig,
    pub(crate) visual_effect_view: ObjcId,
    pub(crate) native_substrate_view: ObjcId,
    pub(crate) native_glass_container_view: ObjcId,
    pub(crate) native_glass_panel_views: Vec<ObjcId>,
    pub(crate) last_native_glass_batch: Option<NativeGlassBatch>,
    pub(crate) last_native_glass_batch_result: Option<NativeGlassBatchResult>,
    pub(crate) native_glass_control_views: Vec<ObjcId>,
    pub(crate) native_glass_control_targets: Vec<ObjcId>,
    pub(crate) last_native_glass_control_batch: Option<NativeGlassControlBatch>,
    pub(crate) native_glass_perform_click_probe_fired_controls: Vec<(LiveId, Rect)>,
    pub(crate) native_glass_accessibility_press_probe_fired_controls: Vec<(LiveId, Rect)>,
    pub(crate) native_glass_mouse_event_probe_fired_controls: Vec<(LiveId, Rect)>,
    pub(crate) native_glass_cg_event_probe_fired_controls: Vec<(LiveId, Rect)>,
    pub(crate) proof_substrate_view: ObjcId,
    pub(crate) above_metal_glass_probe_view: ObjcId,
    pub(crate) last_mouse_pos: Vec2d,
    window_delegate: ObjcId,
    live_resize_timer: ObjcId,
    last_window_geom: Option<WindowGeom>,
}

impl MacosWindow {
    const NS_VIEW_WIDTH_SIZABLE: i64 = 1 << 1;
    const NS_VIEW_HEIGHT_SIZABLE: i64 = 1 << 4;

    unsafe fn init_container_view(&mut self, rect: NSRect) {
        let autoresize = Self::NS_VIEW_WIDTH_SIZABLE | Self::NS_VIEW_HEIGHT_SIZABLE;

        let () = msg_send![self.container_view, initWithFrame: rect];
        let () = msg_send![self.container_view, setAutoresizingMask: autoresize];
        let () = msg_send![self.container_view, setWantsLayer: YES];
        let layer: ObjcId = msg_send![self.container_view, layer];
        if layer != nil {
            let () = msg_send![layer, setOpaque: NO];
            let () = msg_send![
                layer,
                setBackgroundColor: CGColorCreateGenericRGB(0.0, 0.0, 0.0, 0.0)
            ];
        }

        let () = msg_send![self.view, setFrame: rect];
        let () = msg_send![self.view, setAutoresizingMask: autoresize];
    }

    pub fn install_proof_substrate(&mut self, mode: &str) {
        unsafe {
            if self.proof_substrate_view != nil {
                return;
            }

            let bounds: NSRect = msg_send![self.container_view, bounds];
            let container_layer: ObjcId = msg_send![self.container_view, layer];
            if container_layer != nil {
                let (red, green, blue) = if mode == "stripes" {
                    (0.0, 0.0, 0.0)
                } else {
                    (1.0, 0.0, 1.0)
                };
                let () = msg_send![
                    container_layer,
                    setBackgroundColor: CGColorCreateGenericRGB(red, green, blue, 1.0)
                ];
            }

            let proof_view: ObjcId = msg_send![class!(NSView), alloc];
            let proof_view: ObjcId = msg_send![proof_view, initWithFrame: bounds];
            let () = msg_send![
                proof_view,
                setAutoresizingMask: Self::NS_VIEW_WIDTH_SIZABLE | Self::NS_VIEW_HEIGHT_SIZABLE
            ];
            let () = msg_send![proof_view, setWantsLayer: YES];
            let layer: ObjcId = msg_send![proof_view, layer];
            if layer != nil {
                let () = msg_send![layer, setOpaque: YES];
                let (red, green, blue) = if mode == "stripes" {
                    (0.0, 0.0, 0.0)
                } else {
                    (1.0, 0.0, 1.0)
                };
                let () = msg_send![
                    layer,
                    setBackgroundColor: CGColorCreateGenericRGB(red, green, blue, 1.0)
                ];
            }

            if mode == "stripes" {
                let colors = [
                    (1.0, 0.08, 0.08),
                    (0.08, 0.85, 0.18),
                    (0.05, 0.32, 1.0),
                    (1.0, 0.86, 0.08),
                    (0.95, 0.08, 1.0),
                    (0.05, 0.92, 0.95),
                ];
                let stripe_count = 96usize;
                let proof_width = bounds.size.width.max(4096.0);
                let stripe_width = (proof_width / stripe_count as f64).max(1.0);
                for index in 0..stripe_count {
                    let stripe: ObjcId = msg_send![class!(NSView), alloc];
                    let stripe_frame = NSRect {
                        origin: NSPoint {
                            x: index as f64 * stripe_width,
                            y: 0.0,
                        },
                        size: NSSize {
                            width: stripe_width + 1.0,
                            height: bounds.size.height,
                        },
                    };
                    let stripe: ObjcId = msg_send![stripe, initWithFrame: stripe_frame];
                    if stripe == nil {
                        continue;
                    }
                    let () = msg_send![
                        stripe,
                        setAutoresizingMask: Self::NS_VIEW_HEIGHT_SIZABLE
                    ];
                    let () = msg_send![stripe, setWantsLayer: YES];
                    let stripe_layer: ObjcId = msg_send![stripe, layer];
                    if stripe_layer != nil {
                        let (red, green, blue) = colors[index % colors.len()];
                        let () = msg_send![stripe_layer, setOpaque: YES];
                        let () = msg_send![
                            stripe_layer,
                            setBackgroundColor: CGColorCreateGenericRGB(red, green, blue, 1.0)
                        ];
                    }
                    let () = msg_send![proof_view, addSubview: stripe];
                }
            }

            let () = msg_send![
                self.container_view,
                addSubview: proof_view
                positioned: -1i64
                relativeTo: self.view
            ];
            self.proof_substrate_view = proof_view;
            crate::log!("[liquid-glass] proof-substrate={} installed", mode);
        }
    }

    fn native_glass_effect_view_class() -> ObjcId {
        unsafe {
            makepad_objc_sys::runtime::objc_getClass(
                b"NSGlassEffectView\0".as_ptr() as *const c_char
            ) as ObjcId
        }
    }

    fn native_glass_container_view_class() -> ObjcId {
        unsafe {
            makepad_objc_sys::runtime::objc_getClass(
                b"NSGlassEffectContainerView\0".as_ptr() as *const c_char
            ) as ObjcId
        }
    }

    fn native_glass_ns_rect_from_makepad_rect(rect: Rect, container_height: f64) -> NSRect {
        NSRect {
            origin: NSPoint {
                x: rect.pos.x,
                y: (container_height - rect.pos.y - rect.size.y).max(0.0),
            },
            size: NSSize {
                width: rect.size.x.max(0.0),
                height: rect.size.y.max(0.0),
            },
        }
    }

    fn native_glass_panel_ns_rect(panel_rect: Rect, container_rect: Rect) -> NSRect {
        let relative = Rect {
            pos: Vec2d {
                x: panel_rect.pos.x - container_rect.pos.x,
                y: panel_rect.pos.y - container_rect.pos.y,
            },
            size: panel_rect.size,
        };
        Self::native_glass_ns_rect_from_makepad_rect(relative, container_rect.size.y)
    }

    fn native_glass_panel_frame_snapshot_line(
        container_id: LiveId,
        panel: &NativeGlassPanelDescriptor,
        panel_frame: NSRect,
    ) -> String {
        format!(
            "[liquid-glass] native-panel-frame container={:?} panel={:?} makepad=({:.1},{:.1},{:.1},{:.1}) appkit=({:.1},{:.1},{:.1},{:.1}) z_order={} visible={}",
            container_id,
            panel.id,
            panel.rect.pos.x,
            panel.rect.pos.y,
            panel.rect.size.x,
            panel.rect.size.y,
            panel_frame.origin.x,
            panel_frame.origin.y,
            panel_frame.size.width,
            panel_frame.size.height,
            panel.z_order,
            panel.visible
        )
    }

    pub(crate) fn native_glass_geometry_snapshot_enabled_from_value(value: Option<&str>) -> bool {
        matches!(value, Some("1" | "true" | "self-resize"))
    }

    pub(crate) fn native_glass_geometry_snapshot_enabled() -> bool {
        Self::native_glass_geometry_snapshot_enabled_from_value(
            std::env::var("MAKEPAD_NATIVE_GLASS_GEOMETRY_SNAPSHOT")
                .ok()
                .as_deref(),
        )
    }

    pub(crate) fn native_glass_window_geometry_changed(
        old_geom: &WindowGeom,
        new_geom: &WindowGeom,
    ) -> bool {
        old_geom.position != new_geom.position
            || old_geom.inner_size != new_geom.inner_size
            || (old_geom.dpi_factor - new_geom.dpi_factor).abs() > 0.001
    }

    fn above_metal_probe_frame(bounds: NSRect) -> NSRect {
        let inset_x = 76.0f64.min(bounds.size.width * 0.18);
        let inset_y = 64.0f64.min(bounds.size.height * 0.18);
        NSRect {
            origin: NSPoint {
                x: inset_x,
                y: inset_y,
            },
            size: NSSize {
                width: (bounds.size.width - inset_x * 2.0).max(1.0),
                height: (bounds.size.height - inset_y * 2.0).max(1.0),
            },
        }
    }

    fn ns_color_from_vec4f(tint: Vec4f) -> ObjcId {
        unsafe {
            msg_send![
                class!(NSColor),
                colorWithSRGBRed: tint.x as f64
                green: tint.y as f64
                blue: tint.z as f64
                alpha: tint.w as f64
            ]
        }
    }

    fn default_native_glass_tint(style: NativeGlassStyle) -> Vec4f {
        match style {
            NativeGlassStyle::Regular => Vec4f {
                x: 0.06,
                y: 0.08,
                z: 0.12,
                w: 0.12,
            },
            NativeGlassStyle::Clear => Vec4f {
                x: 0.06,
                y: 0.08,
                z: 0.12,
                w: 0.03,
            },
        }
    }

    fn event_style_from_native_glass(style: NativeGlassStyle) -> WindowNativeSubstrateStyle {
        match style {
            NativeGlassStyle::Regular => WindowNativeSubstrateStyle::MacosGlassRegular,
            NativeGlassStyle::Clear => WindowNativeSubstrateStyle::MacosGlassClear,
        }
    }

    pub(crate) fn install_above_metal_glass_probe(&mut self, style: MacosNativeGlassStyle) {
        unsafe {
            if self.above_metal_glass_probe_view != nil {
                return;
            }

            let glass_class = Self::native_glass_effect_view_class();
            if glass_class.is_null() {
                crate::log!(
                    "[liquid-glass] above-metal-probe state=unsupported reason=class-missing"
                );
                return;
            }

            let can_init: BOOL =
                msg_send![glass_class, instancesRespondToSelector: sel!(initWithFrame:)];
            if can_init != YES {
                crate::log!(
                    "[liquid-glass] above-metal-probe state=failed reason=missing-initWithFrame"
                );
                return;
            }

            let bounds: NSRect = msg_send![self.container_view, bounds];
            let frame = Self::above_metal_probe_frame(bounds);
            let glass_view: ObjcId = msg_send![glass_class, alloc];
            let glass_view: ObjcId = msg_send![glass_view, initWithFrame: frame];
            if glass_view == nil {
                crate::log!(
                    "[liquid-glass] above-metal-probe state=failed reason=alloc-init-failed"
                );
                return;
            }

            let () = msg_send![
                glass_view,
                setAutoresizingMask: Self::NS_VIEW_WIDTH_SIZABLE | Self::NS_VIEW_HEIGHT_SIZABLE
            ];
            let () = msg_send![glass_view, setWantsLayer: YES];
            let layer: ObjcId = msg_send![glass_view, layer];
            if layer != nil {
                let () = msg_send![layer, setMasksToBounds: YES];
                let () = msg_send![layer, setCornerRadius: 30.0f64];
            }

            let set_style_sel = sel!(setStyle:);
            let can_set_style: BOOL = msg_send![glass_view, respondsToSelector: set_style_sel];
            let style_raw = style.ns_style_raw();
            if can_set_style == YES {
                let () = msg_send![glass_view, setStyle: style_raw];
            }

            let set_tint_sel = sel!(setTintColor:);
            let can_set_tint: BOOL = msg_send![glass_view, respondsToSelector: set_tint_sel];
            if can_set_tint == YES {
                let tint: ObjcId = msg_send![
                    class!(NSColor),
                    colorWithSRGBRed: 0.94f64
                    green: 0.96f64
                    blue: 1.0f64
                    alpha: 0.10f64
                ];
                let () = msg_send![glass_view, setTintColor: tint];
            }

            let set_corner_radius_sel = sel!(setCornerRadius:);
            let can_set_corner_radius: BOOL =
                msg_send![glass_view, respondsToSelector: set_corner_radius_sel];
            if can_set_corner_radius == YES {
                let () = msg_send![glass_view, setCornerRadius: 30.0f64];
            }

            let () = msg_send![
                self.container_view,
                addSubview: glass_view
                positioned: 1i64
                relativeTo: self.view
            ];
            let superview: ObjcId = msg_send![glass_view, superview];
            if superview != self.container_view {
                let () = msg_send![glass_view, removeFromSuperview];
                crate::log!(
                    "[liquid-glass] above-metal-probe state=failed reason=attach-unverified"
                );
                return;
            }

            self.above_metal_glass_probe_view = glass_view;
            crate::log!(
                "[liquid-glass] above-metal-probe state=installed style={} style_raw={} input=diagnostic-overlay",
                style.log_name(),
                style_raw
            );
        }
    }

    fn window_native_substrate_state_from_native_glass(
        state: NativeGlassInstallState,
    ) -> WindowNativeSubstrateState {
        match state {
            NativeGlassInstallState::Installed => WindowNativeSubstrateState::Installed,
            NativeGlassInstallState::Failed | NativeGlassInstallState::Rejected => {
                WindowNativeSubstrateState::PreflightFailed
            }
            NativeGlassInstallState::Skipped | NativeGlassInstallState::Partial => {
                WindowNativeSubstrateState::VisibilityUnverified
            }
        }
    }

    fn native_substrate_resolved_event(
        &self,
        state: WindowNativeSubstrateState,
        style: Option<MacosNativeGlassStyle>,
        reason: &'static str,
    ) -> WindowNativeSubstrateResolvedEvent {
        WindowNativeSubstrateResolvedEvent {
            window_id: self.window_id,
            state,
            style: style.map(MacosNativeGlassStyle::to_event_style),
            reason,
        }
    }

    pub(crate) fn install_native_glass_substrate(
        &mut self,
        style: MacosNativeGlassStyle,
    ) -> WindowNativeSubstrateResolvedEvent {
        unsafe {
            if self.native_substrate_view != nil {
                let style_raw = style.ns_style_raw();
                crate::log!(
                    "[liquid-glass] state=4 substrate=macos-native style={} style_raw={}",
                    style.log_name(),
                    style_raw
                );
                return self.native_substrate_resolved_event(
                    WindowNativeSubstrateState::Installed,
                    Some(style),
                    "already-installed",
                );
            }

            let glass_class = Self::native_glass_effect_view_class();
            if glass_class.is_null() {
                crate::log!("[liquid-glass] state=1 reason=class-missing detail=NSGlassEffectView");
                return self.native_substrate_resolved_event(
                    WindowNativeSubstrateState::ClassMissing,
                    None,
                    "class-missing",
                );
            }

            let bounds: NSRect = msg_send![self.container_view, bounds];
            let shell_inset = 3.0f64;
            let glass_frame = NSRect {
                origin: NSPoint {
                    x: shell_inset,
                    y: shell_inset,
                },
                size: NSSize {
                    width: (bounds.size.width - shell_inset * 2.0).max(0.0),
                    height: (bounds.size.height - shell_inset * 2.0).max(0.0),
                },
            };
            let container_layer: ObjcId = msg_send![self.container_view, layer];
            if container_layer != nil {
                let () = msg_send![container_layer, setOpaque: NO];
                let () = msg_send![
                    container_layer,
                    setBackgroundColor: CGColorCreateGenericRGB(0.0, 0.0, 0.0, 0.0)
                ];
            }

            let can_init: BOOL =
                msg_send![glass_class, instancesRespondToSelector: sel!(initWithFrame:)];
            if can_init != YES {
                crate::log!("[liquid-glass] state=2 reason=missing-initWithFrame");
                return self.native_substrate_resolved_event(
                    WindowNativeSubstrateState::PreflightFailed,
                    Some(style),
                    "missing-initWithFrame",
                );
            }

            let glass_view: ObjcId = msg_send![glass_class, alloc];
            let glass_view: ObjcId = msg_send![glass_view, initWithFrame: glass_frame];
            if glass_view == nil {
                crate::log!("[liquid-glass] state=2 reason=alloc-init-failed");
                return self.native_substrate_resolved_event(
                    WindowNativeSubstrateState::PreflightFailed,
                    Some(style),
                    "alloc-init-failed",
                );
            }

            let () = msg_send![
                glass_view,
                setAutoresizingMask: Self::NS_VIEW_WIDTH_SIZABLE | Self::NS_VIEW_HEIGHT_SIZABLE
            ];
            let () = msg_send![glass_view, setWantsLayer: YES];
            let glass_layer: ObjcId = msg_send![glass_view, layer];
            if glass_layer != nil {
                let () = msg_send![glass_layer, setMasksToBounds: YES];
                let () = msg_send![glass_layer, setCornerRadius: 30.0f64];
            }

            let set_style_sel = sel!(setStyle:);
            let can_set_style: BOOL = msg_send![glass_view, respondsToSelector: set_style_sel];
            let style_raw = style.ns_style_raw();
            if can_set_style == YES {
                let () = msg_send![glass_view, setStyle: style_raw];
            } else {
                crate::log!("[liquid-glass] state=2 reason=missing-setStyle");
                return self.native_substrate_resolved_event(
                    WindowNativeSubstrateState::PreflightFailed,
                    Some(style),
                    "missing-setStyle",
                );
            }

            let set_tint_sel = sel!(setTintColor:);
            let can_set_tint: BOOL = msg_send![glass_view, respondsToSelector: set_tint_sel];
            if can_set_tint == YES {
                let tint: ObjcId = msg_send![
                    class!(NSColor),
                    colorWithSRGBRed: 0.06f64
                    green: 0.08f64
                    blue: 0.12f64
                    alpha: style.tint_alpha()
                ];
                let () = msg_send![glass_view, setTintColor: tint];
            }

            let set_corner_radius_sel = sel!(setCornerRadius:);
            let can_set_corner_radius: BOOL =
                msg_send![glass_view, respondsToSelector: set_corner_radius_sel];
            if can_set_corner_radius == YES {
                let () = msg_send![glass_view, setCornerRadius: 30.0f64];
            }

            let () = msg_send![
                self.container_view,
                addSubview: glass_view
                positioned: -1i64
                relativeTo: self.view
            ];
            let superview: ObjcId = msg_send![glass_view, superview];
            if superview != self.container_view {
                let () = msg_send![glass_view, removeFromSuperview];
                crate::log!("[liquid-glass] state=3 reason=attach-unverified");
                return self.native_substrate_resolved_event(
                    WindowNativeSubstrateState::VisibilityUnverified,
                    Some(style),
                    "attach-unverified",
                );
            }
            self.native_substrate_view = glass_view;
            crate::log!(
                "[liquid-glass] state=4 substrate=macos-native style={} style_raw={}",
                style.log_name(),
                style_raw
            );
            self.native_substrate_resolved_event(
                WindowNativeSubstrateState::Installed,
                Some(style),
                "installed-on-proofed-hierarchy",
            )
        }
    }

    fn clear_native_glass_batch_views(&mut self) {
        unsafe {
            for panel in self.native_glass_panel_views.drain(..) {
                let () = msg_send![panel, removeFromSuperview];
            }
            if self.native_glass_container_view != nil {
                let () = msg_send![self.native_glass_container_view, removeFromSuperview];
                self.native_glass_container_view = nil;
            }
        }
    }

    fn clear_native_glass_control_views(&mut self) {
        unsafe {
            for control in self.native_glass_control_views.drain(..) {
                let () = msg_send![control, removeFromSuperview];
            }
            self.native_glass_control_targets.clear();
        }
    }

    fn native_glass_result_for_validation_error(
        &self,
        batch: &NativeGlassBatch,
        error: NativeGlassBatchValidationError,
    ) -> NativeGlassBatchResult {
        let reason = match error {
            NativeGlassBatchValidationError::TooManyContainers { .. } => "too-many-containers",
            NativeGlassBatchValidationError::TooManyVisiblePanels { .. } => {
                "too-many-visible-panels"
            }
            NativeGlassBatchValidationError::InteractiveHitTestUnsupported { .. } => {
                "interactive-hit-test-unsupported"
            }
        };
        NativeGlassBatchResult {
            window_id: batch.window_id,
            backend_state: NativeGlassBackendState::Rejected,
            containers: batch
                .containers
                .iter()
                .map(|container| {
                    NativeGlassContainerResult::from_panel_results(
                        container.id,
                        NativeGlassInstallState::Rejected,
                        reason,
                        container
                            .panels
                            .iter()
                            .map(|panel| NativeGlassPanelResult {
                                id: panel.id,
                                state: NativeGlassInstallState::Rejected,
                                reason,
                            })
                            .collect(),
                    )
                })
                .collect(),
        }
    }

    fn native_glass_class_missing_result(
        &self,
        batch: &NativeGlassBatch,
        reason: &'static str,
    ) -> NativeGlassBatchResult {
        NativeGlassBatchResult {
            window_id: batch.window_id,
            backend_state: NativeGlassBackendState::Unsupported,
            containers: batch
                .containers
                .iter()
                .map(|container| {
                    NativeGlassContainerResult::from_panel_results(
                        container.id,
                        NativeGlassInstallState::Failed,
                        reason,
                        container
                            .panels
                            .iter()
                            .map(|panel| NativeGlassPanelResult {
                                id: panel.id,
                                state: NativeGlassInstallState::Failed,
                                reason,
                            })
                            .collect(),
                    )
                })
                .collect(),
        }
    }

    fn log_native_glass_batch_result(result: &NativeGlassBatchResult) {
        let (installed, failed) = result
            .containers
            .iter()
            .fold((0usize, 0usize), |acc, item| {
                (acc.0 + item.installed_panels, acc.1 + item.failed_panels)
            });
        crate::log!(
            "[liquid-glass] backend=apple-native-underlay state={:?} containers={} panels_installed={} panels_failed={}",
            result.backend_state,
            result.containers.len(),
            installed,
            failed
        );
        for container in &result.containers {
            crate::log!(
                "[liquid-glass] container={:?} state={:?} reason={} panels_installed={} panels_failed={}",
                container.id,
                container.state,
                container.reason,
                container.installed_panels,
                container.failed_panels
            );
        }
    }

    fn native_glass_control_validation_reason(
        error: NativeGlassControlBatchValidationError,
    ) -> &'static str {
        match error {
            NativeGlassControlBatchValidationError::TooManyVisibleControls { .. } => {
                "too-many-visible-controls"
            }
            NativeGlassControlBatchValidationError::EmptyVisibleControlRect { .. } => {
                "empty-visible-control-rect"
            }
        }
    }

    fn log_native_glass_control_batch(
        batch: &NativeGlassControlBatch,
        state: NativeGlassBackendState,
        reason: &'static str,
    ) {
        crate::log!(
            "[liquid-glass] backend=apple-native-controls state={:?} reason={} controls_total={} controls_visible={}",
            state,
            reason,
            batch.controls.len(),
            batch.visible_control_count()
        );
    }

    unsafe fn native_glass_objc_class_name(object: ObjcId) -> String {
        if object == nil {
            return "nil".to_string();
        }
        let class: ObjcId = msg_send![object, class];
        nsstring_to_string(NSStringFromClass(class as _))
    }

    unsafe fn log_native_glass_control_hierarchy(&self) {
        let subviews: ObjcId = msg_send![self.container_view, subviews];
        let count: usize = msg_send![subviews, count];
        crate::log!(
            "[liquid-glass] backend=apple-native-controls hierarchy subviews={}",
            count
        );
        for index in 0..count {
            let view: ObjcId = msg_send![subviews, objectAtIndex: index];
            let frame: NSRect = msg_send![view, frame];
            let hidden: BOOL = msg_send![view, isHidden];
            let role = if view == self.view {
                "metal-view"
            } else if self
                .native_glass_control_views
                .iter()
                .any(|control| *control == view)
            {
                "native-control"
            } else {
                "other"
            };
            crate::log!(
                "[liquid-glass] backend=apple-native-controls hierarchy index={} role={} class={} frame=({:.1},{:.1},{:.1},{:.1}) hidden={}",
                index,
                role,
                Self::native_glass_objc_class_name(view),
                frame.origin.x,
                frame.origin.y,
                frame.size.width,
                frame.size.height,
                hidden
            );
        }
    }

    fn native_glass_button_class() -> ObjcId {
        get_macos_class_global().native_glass_button as ObjcId
    }

    fn native_glass_control_frame(
        control: &NativeGlassControlDescriptor,
        bounds: NSRect,
    ) -> NSRect {
        Self::native_glass_ns_rect_from_makepad_rect(control.rect, bounds.size.height)
    }

    fn native_glass_control_frame_snapshot_line(
        control: &NativeGlassControlDescriptor,
        control_frame: NSRect,
    ) -> String {
        format!(
            "[liquid-glass] native-control-frame control={:?} kind={:?} label={:?} makepad=({:.1},{:.1},{:.1},{:.1}) appkit=({:.1},{:.1},{:.1},{:.1}) z_order={} visible={}",
            control.id,
            control.kind,
            control.label,
            control.rect.pos.x,
            control.rect.pos.y,
            control.rect.size.x,
            control.rect.size.y,
            control_frame.origin.x,
            control_frame.origin.y,
            control_frame.size.width,
            control_frame.size.height,
            control.z_order,
            control.visible
        )
    }

    fn native_glass_control_hit_test_probe_line(
        control: &NativeGlassControlDescriptor,
        point: NSPoint,
        result_class: &str,
        matches_control: bool,
    ) -> String {
        format!(
            "[liquid-glass] backend=apple-native-controls event=appkit-hit-test-probe control={:?} label={:?} point=({:.1},{:.1}) result_class={} matches_control={}",
            control.id,
            control.label,
            point.x,
            point.y,
            result_class,
            matches_control
        )
    }

    fn native_glass_control_perform_click_probe_matches_value(
        value: Option<&str>,
        control: &NativeGlassControlDescriptor,
    ) -> bool {
        let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
            return false;
        };
        matches!(value, "1" | "true" | "on" | "all")
            || control.label.as_str().eq_ignore_ascii_case(value)
    }

    fn native_glass_control_perform_click_probe_matches(
        control: &NativeGlassControlDescriptor,
    ) -> bool {
        Self::native_glass_control_perform_click_probe_matches_value(
            std::env::var("MAKEPAD_NATIVE_GLASS_CONTROL_PERFORM_CLICK_PROBE")
                .ok()
                .as_deref(),
            control,
        )
    }

    fn native_glass_control_accessibility_press_probe_matches(
        control: &NativeGlassControlDescriptor,
    ) -> bool {
        Self::native_glass_control_perform_click_probe_matches_value(
            std::env::var("MAKEPAD_NATIVE_GLASS_CONTROL_ACCESSIBILITY_PRESS_PROBE")
                .ok()
                .as_deref(),
            control,
        )
    }

    fn native_glass_button_bezel_style_raw_from_value(value: Option<&str>) -> i64 {
        value
            .and_then(|value| value.trim().parse::<i64>().ok())
            .unwrap_or(16)
    }

    fn native_glass_button_bezel_style_raw() -> i64 {
        let raw = Self::native_glass_button_bezel_style_raw_from_value(
            std::env::var("MAKEPAD_NATIVE_GLASS_BUTTON_BEZEL_RAW")
                .ok()
                .as_deref(),
        );
        if std::env::var("MAKEPAD_NATIVE_GLASS_BUTTON_BEZEL_RAW").is_ok() {
            crate::log!(
                "[liquid-glass] backend=apple-native-controls bezel-override raw={}",
                raw
            );
        }
        raw
    }

    fn native_glass_button_style_line(
        control: &NativeGlassControlDescriptor,
        bezel_raw: i64,
    ) -> String {
        format!(
            "[liquid-glass] backend=apple-native-controls button-style control={:?} label={:?} bezel=glass raw={}",
            control.id,
            control.label,
            bezel_raw
        )
    }

    fn native_glass_control_accessibility_line(control: &NativeGlassControlDescriptor) -> String {
        format!(
            "[liquid-glass] backend=apple-native-controls accessibility-label control={:?} label={:?}",
            control.id, control.label
        )
    }

    fn native_glass_control_mouse_event_probe_matches(
        control: &NativeGlassControlDescriptor,
    ) -> bool {
        Self::native_glass_control_perform_click_probe_matches_value(
            std::env::var("MAKEPAD_NATIVE_GLASS_CONTROL_MOUSE_EVENT_PROBE")
                .ok()
                .as_deref(),
            control,
        )
    }

    fn native_glass_control_probe_key(control: &NativeGlassControlDescriptor) -> (LiveId, Rect) {
        (control.id, control.rect)
    }

    fn native_glass_control_probe_already_fired(
        fired_controls: &[(LiveId, Rect)],
        control: &NativeGlassControlDescriptor,
    ) -> bool {
        fired_controls.contains(&Self::native_glass_control_probe_key(control))
    }

    fn native_glass_control_cg_event_probe_mode_value(
        value: Option<&str>,
        control: &NativeGlassControlDescriptor,
    ) -> Option<NativeGlassCgEventProbeMode> {
        let value = value?.trim();
        let (mode, target) = if let Some(target) = value.strip_prefix("pid:") {
            (NativeGlassCgEventProbeMode::Process, target)
        } else if let Some(target) = value.strip_prefix("process:") {
            (NativeGlassCgEventProbeMode::Process, target)
        } else if let Some(target) = value.strip_prefix("global:") {
            (NativeGlassCgEventProbeMode::Global, target)
        } else {
            (NativeGlassCgEventProbeMode::Global, value)
        };

        Self::native_glass_control_perform_click_probe_matches_value(Some(target), control)
            .then_some(mode)
    }

    fn native_glass_control_cg_event_probe_mode(
        control: &NativeGlassControlDescriptor,
    ) -> Option<NativeGlassCgEventProbeMode> {
        Self::native_glass_control_cg_event_probe_mode_value(
            std::env::var("MAKEPAD_NATIVE_GLASS_CONTROL_CGEVENT_PROBE")
                .ok()
                .as_deref(),
            control,
        )
    }

    fn native_glass_cg_event_point_for_display_height(
        appkit_screen_point: NSPoint,
        display_height: f64,
    ) -> NSPoint {
        NSPoint {
            x: appkit_screen_point.x,
            y: display_height - appkit_screen_point.y,
        }
    }

    unsafe fn native_glass_cg_event_point_from_appkit_screen_point(
        appkit_screen_point: NSPoint,
    ) -> NSPoint {
        Self::native_glass_cg_event_point_for_display_height(
            appkit_screen_point,
            CGDisplayPixelsHigh(CGMainDisplayID()) as f64,
        )
    }

    unsafe fn log_native_glass_control_hit_test_probe(
        &self,
        control: &NativeGlassControlDescriptor,
        control_view: ObjcId,
        control_frame: NSRect,
    ) {
        let point = NSPoint {
            x: control_frame.origin.x + control_frame.size.width * 0.5,
            y: control_frame.origin.y + control_frame.size.height * 0.5,
        };
        let hit_view: ObjcId = msg_send![self.container_view, hitTest: point];
        crate::log!(
            "{}",
            Self::native_glass_control_hit_test_probe_line(
                control,
                point,
                &Self::native_glass_objc_class_name(hit_view),
                hit_view == control_view,
            )
        );
    }

    unsafe fn run_native_glass_control_mouse_event_probe(
        &mut self,
        control: &NativeGlassControlDescriptor,
        control_frame: NSRect,
    ) {
        if !Self::native_glass_control_mouse_event_probe_matches(control)
            || Self::native_glass_control_probe_already_fired(
                &self.native_glass_mouse_event_probe_fired_controls,
                control,
            )
        {
            return;
        }
        self.native_glass_mouse_event_probe_fired_controls
            .push(Self::native_glass_control_probe_key(control));
        let point = NSPoint {
            x: control_frame.origin.x + control_frame.size.width * 0.5,
            y: control_frame.origin.y + control_frame.size.height * 0.5,
        };
        crate::log!(
            "[liquid-glass] backend=apple-native-controls event=synthetic-mouse-probe control={:?} label={:?} point=({:.1},{:.1})",
            control.id,
            control.label,
            point.x,
            point.y
        );
        let window_number: i64 = msg_send![self.window, windowNumber];
        let mouse_down: ObjcId = msg_send![
            class!(NSEvent),
            mouseEventWithType: NSEventType::NSLeftMouseDown
            location: point
            modifierFlags: 0u64
            timestamp: 0f64
            windowNumber: window_number
            context: nil
            eventNumber: 1i64
            clickCount: 1i64
            pressure: 1.0f64
        ];
        let mouse_up: ObjcId = msg_send![
            class!(NSEvent),
            mouseEventWithType: NSEventType::NSLeftMouseUp
            location: point
            modifierFlags: 0u64
            timestamp: 0f64
            windowNumber: window_number
            context: nil
            eventNumber: 2i64
            clickCount: 1i64
            pressure: 0.0f64
        ];
        let ns_app: ObjcId = msg_send![class!(NSApplication), sharedApplication];
        let () = msg_send![ns_app, postEvent: mouse_down atStart: NO];
        let () = msg_send![ns_app, postEvent: mouse_up atStart: NO];
    }

    unsafe fn run_native_glass_control_cg_event_probe(
        &mut self,
        control: &NativeGlassControlDescriptor,
        control_frame: NSRect,
    ) {
        let Some(mode) = Self::native_glass_control_cg_event_probe_mode(control) else {
            return;
        };
        if Self::native_glass_control_probe_already_fired(
            &self.native_glass_cg_event_probe_fired_controls,
            control,
        ) {
            return;
        }
        self.native_glass_cg_event_probe_fired_controls
            .push(Self::native_glass_control_probe_key(control));
        let local_point = NSPoint {
            x: control_frame.origin.x + control_frame.size.width * 0.5,
            y: control_frame.origin.y + control_frame.size.height * 0.5,
        };
        let screen_rect: NSRect = msg_send![
            self.window,
            convertRectToScreen: NSRect {
                origin: local_point,
                size: NSSize {
                    width: 0.0,
                    height: 0.0,
                },
            }
        ];
        let cg_point =
            Self::native_glass_cg_event_point_from_appkit_screen_point(screen_rect.origin);
        crate::log!(
            "[liquid-glass] backend=apple-native-controls event=cg-event-probe mode={:?} control={:?} label={:?} local=({:.1},{:.1}) screen=({:.1},{:.1}) cg=({:.1},{:.1})",
            mode,
            control.id,
            control.label,
            local_point.x,
            local_point.y,
            screen_rect.origin.x,
            screen_rect.origin.y,
            cg_point.x,
            cg_point.y
        );
        let source = CGEventSourceCreate(1);
        let event = CGEventCreateMouseEvent(source, kCGEventLeftMouseDown, cg_point, 0);
        CGEventSetIntegerValueField(event, kCGMouseEventClickState, 1);
        match mode {
            NativeGlassCgEventProbeMode::Global => CGEventPost(0, event),
            NativeGlassCgEventProbeMode::Process => CGEventPostToPid(std::process::id(), event),
        }
        let event = CGEventCreateMouseEvent(source, kCGEventLeftMouseUp, cg_point, 0);
        CGEventSetIntegerValueField(event, kCGMouseEventClickState, 1);
        match mode {
            NativeGlassCgEventProbeMode::Global => CGEventPost(0, event),
            NativeGlassCgEventProbeMode::Process => CGEventPostToPid(std::process::id(), event),
        }
    }

    unsafe fn install_native_glass_button_control(
        &mut self,
        control: &NativeGlassControlDescriptor,
        button_class: ObjcId,
        bounds: NSRect,
    ) -> bool {
        let button_frame = Self::native_glass_control_frame(control, bounds);
        crate::log!(
            "{}",
            Self::native_glass_control_frame_snapshot_line(control, button_frame)
        );
        let button: ObjcId = msg_send![button_class, alloc];
        let button: ObjcId = msg_send![button, initWithFrame: button_frame];
        if button == nil {
            return false;
        }

        let title = str_to_nsstring(&control.label);
        let () = msg_send![button, setTitle: title];
        let () = msg_send![button, setEnabled: if control.enabled { YES } else { NO }];
        let () = msg_send![button, setHidden: if control.visible { NO } else { YES }];
        let () = msg_send![button, setWantsLayer: YES];
        let set_bezel_style_sel = sel!(setBezelStyle:);
        let can_set_bezel_style: BOOL = msg_send![button, respondsToSelector: set_bezel_style_sel];
        if can_set_bezel_style == YES {
            let bezel_raw = Self::native_glass_button_bezel_style_raw();
            let () = msg_send![button, setBezelStyle: bezel_raw];
            crate::log!(
                "{}",
                Self::native_glass_button_style_line(control, bezel_raw)
            );
        } else {
            crate::log!(
                "[liquid-glass] backend=apple-native-controls button-style control={:?} label={:?} state=Unsupported reason=setBezelStyle-missing",
                control.id,
                control.label
            );
        }
        let set_accessibility_label_sel = sel!(setAccessibilityLabel:);
        let can_set_accessibility_label: BOOL =
            msg_send![button, respondsToSelector: set_accessibility_label_sel];
        if can_set_accessibility_label == YES {
            let () = msg_send![button, setAccessibilityLabel: title];
            crate::log!("{}", Self::native_glass_control_accessibility_line(control));
        } else {
            crate::log!(
                "[liquid-glass] backend=apple-native-controls accessibility-label control={:?} label={:?} state=Unsupported reason=setAccessibilityLabel-missing",
                control.id,
                control.label
            );
        }

        let target: ObjcId = msg_send![get_macos_class_global().native_glass_control_target, new];
        if target == nil {
            let () = msg_send![button, removeFromSuperview];
            return false;
        }
        (*target).set_ivar("window_index", self.window_id.0);
        (*target).set_ivar("window_generation", self.window_id.1);
        (*target).set_ivar("control_id_u64", control.id.0);
        let () = msg_send![button, setTarget: target];
        let () = msg_send![button, setAction: sel!(nativeGlassControlAction:)];

        let () = msg_send![
            self.container_view,
            addSubview: button
            positioned: 1i64
            relativeTo: nil
        ];
        self.log_native_glass_control_hit_test_probe(control, button, button_frame);
        self.run_native_glass_control_cg_event_probe(control, button_frame);
        self.run_native_glass_control_mouse_event_probe(control, button_frame);
        if Self::native_glass_control_accessibility_press_probe_matches(control)
            && !Self::native_glass_control_probe_already_fired(
                &self.native_glass_accessibility_press_probe_fired_controls,
                control,
            )
        {
            self.native_glass_accessibility_press_probe_fired_controls
                .push(Self::native_glass_control_probe_key(control));
            let accessibility_press_sel = sel!(accessibilityPerformPress);
            let can_accessibility_press: BOOL =
                msg_send![button, respondsToSelector: accessibility_press_sel];
            if can_accessibility_press == YES {
                crate::log!(
                    "[liquid-glass] backend=apple-native-controls event=accessibility-press-probe control={:?} label={:?}",
                    control.id,
                    control.label
                );
                let pressed: BOOL = msg_send![button, accessibilityPerformPress];
                crate::log!(
                    "[liquid-glass] backend=apple-native-controls event=accessibility-press-result control={:?} label={:?} result={}",
                    control.id,
                    control.label,
                    pressed == YES
                );
            } else {
                crate::log!(
                    "[liquid-glass] backend=apple-native-controls event=accessibility-press-probe control={:?} label={:?} state=Unsupported reason=accessibilityPerformPress-missing",
                    control.id,
                    control.label
                );
            }
        }
        if Self::native_glass_control_perform_click_probe_matches(control)
            && !Self::native_glass_control_probe_already_fired(
                &self.native_glass_perform_click_probe_fired_controls,
                control,
            )
        {
            self.native_glass_perform_click_probe_fired_controls
                .push(Self::native_glass_control_probe_key(control));
            crate::log!(
                "[liquid-glass] backend=apple-native-controls event=perform-click-probe control={:?} label={:?}",
                control.id,
                control.label
            );
            let () = msg_send![button, performClick: nil];
        }
        self.native_glass_control_views.push(button);
        self.native_glass_control_targets.push(target);
        true
    }

    pub(crate) fn update_native_glass_control_batch(&mut self, batch: NativeGlassControlBatch) {
        self.update_native_glass_control_batch_inner(batch, false, "descriptor-update");
    }

    pub(crate) fn refresh_native_glass_control_frames_for_geometry_change(&mut self) {
        let Some(batch) = self.last_native_glass_control_batch.clone() else {
            return;
        };
        self.update_native_glass_control_batch_inner(batch, true, "geometry-change");
    }

    fn update_native_glass_control_batch_inner(
        &mut self,
        batch: NativeGlassControlBatch,
        force_reinstall: bool,
        reason: &'static str,
    ) {
        if self
            .last_native_glass_control_batch
            .as_ref()
            .map(|last| last.equivalent_for_native_update(&batch))
            .unwrap_or(false)
            && !force_reinstall
        {
            return;
        }

        if let Err(error) = batch.validate_v4_10() {
            Self::log_native_glass_control_batch(
                &batch,
                NativeGlassBackendState::Rejected,
                Self::native_glass_control_validation_reason(error),
            );
            self.last_native_glass_control_batch = Some(batch);
            return;
        }

        unsafe {
            if force_reinstall {
                crate::log!(
                    "[liquid-glass] backend=apple-native-controls event=frame-refresh reason={} controls_total={} controls_visible={}",
                    reason,
                    batch.controls.len(),
                    batch.visible_control_count()
                );
            }
            self.clear_native_glass_control_views();
            let button_class = Self::native_glass_button_class();
            if button_class == nil {
                Self::log_native_glass_control_batch(
                    &batch,
                    NativeGlassBackendState::Unsupported,
                    "nsbutton-class-missing",
                );
                self.last_native_glass_control_batch = Some(batch);
                return;
            }

            let bounds: NSRect = msg_send![self.container_view, bounds];
            let mut visible_controls: Vec<&NativeGlassControlDescriptor> = batch
                .controls
                .iter()
                .filter(|control| control.visible)
                .collect();
            visible_controls.sort_by_key(|control| control.z_order);

            let mut installed = 0usize;
            for control in visible_controls {
                if self.install_native_glass_button_control(control, button_class, bounds) {
                    installed += 1;
                }
            }

            let visible_count = batch.visible_control_count();
            let (state, reason) = if visible_count == installed {
                (
                    NativeGlassBackendState::Installed,
                    "installed-appkit-buttons",
                )
            } else if installed == 0 {
                (NativeGlassBackendState::Rejected, "button-install-failed")
            } else {
                (NativeGlassBackendState::Partial, "partial-button-install")
            };
            Self::log_native_glass_control_batch(&batch, state, reason);
            self.log_native_glass_control_hierarchy();
            self.last_native_glass_control_batch = Some(batch);
        }
    }

    pub(crate) fn log_native_glass_frame_snapshot(
        &self,
        reason: &'static str,
        old_geom: &WindowGeom,
        new_geom: &WindowGeom,
    ) {
        let Some(batch) = &self.last_native_glass_batch else {
            return;
        };
        crate::log!(
            "[liquid-glass] native-display-frame-snapshot reason={} old_dpi={:.3} new_dpi={:.3} old_pos=({:.1},{:.1}) new_pos=({:.1},{:.1}) containers={}",
            reason,
            old_geom.dpi_factor,
            new_geom.dpi_factor,
            old_geom.position.x,
            old_geom.position.y,
            new_geom.position.x,
            new_geom.position.y,
            batch.containers.len()
        );
        for container in &batch.containers {
            let mut panels: Vec<&NativeGlassPanelDescriptor> = container.panels.iter().collect();
            panels.sort_by_key(|panel| panel.z_order);
            for panel in panels {
                let panel_frame = Self::native_glass_panel_ns_rect(panel.rect, container.rect);
                crate::log!(
                    "{}",
                    Self::native_glass_panel_frame_snapshot_line(container.id, panel, panel_frame)
                );
            }
        }
    }

    fn cache_native_glass_batch_result(
        &mut self,
        batch: &NativeGlassBatch,
        result: &NativeGlassBatchResult,
    ) {
        self.last_native_glass_batch = Some(batch.clone());
        self.last_native_glass_batch_result = Some(result.clone());
    }

    pub(crate) fn install_native_glass_batch(
        &mut self,
        batch: NativeGlassBatch,
    ) -> (
        NativeGlassBatchResult,
        Option<WindowNativeSubstrateResolvedEvent>,
    ) {
        if self
            .last_native_glass_batch
            .as_ref()
            .map(|last| last.equivalent_for_native_update(&batch))
            .unwrap_or(false)
        {
            if let Some(result) = self.last_native_glass_batch_result.clone() {
                return (result, None);
            }
        }

        if let Err(error) = batch.validate_v4_1() {
            let result = self.native_glass_result_for_validation_error(&batch, error);
            Self::log_native_glass_batch_result(&result);
            self.cache_native_glass_batch_result(&batch, &result);
            return (result, None);
        }

        let Some(container) = batch.containers.first() else {
            self.clear_native_glass_batch_views();
            let result = NativeGlassBatchResult {
                window_id: batch.window_id,
                backend_state: NativeGlassBackendState::Installed,
                containers: Vec::new(),
            };
            Self::log_native_glass_batch_result(&result);
            self.cache_native_glass_batch_result(&batch, &result);
            return (result, None);
        };

        unsafe {
            let container_class = Self::native_glass_container_view_class();
            if container_class.is_null() {
                crate::log!(
                    "[liquid-glass] state=1 reason=class-missing detail=NSGlassEffectContainerView"
                );
                let result =
                    self.native_glass_class_missing_result(&batch, "container-class-missing");
                Self::log_native_glass_batch_result(&result);
                self.cache_native_glass_batch_result(&batch, &result);
                return (result, None);
            }

            let glass_class = Self::native_glass_effect_view_class();
            if glass_class.is_null() {
                crate::log!("[liquid-glass] state=1 reason=class-missing detail=NSGlassEffectView");
                let result = self.native_glass_class_missing_result(&batch, "panel-class-missing");
                Self::log_native_glass_batch_result(&result);
                self.cache_native_glass_batch_result(&batch, &result);
                return (result, None);
            }

            let can_init_container: BOOL =
                msg_send![container_class, instancesRespondToSelector: sel!(initWithFrame:)];
            let can_init_panel: BOOL =
                msg_send![glass_class, instancesRespondToSelector: sel!(initWithFrame:)];
            if can_init_container != YES || can_init_panel != YES {
                let result =
                    self.native_glass_class_missing_result(&batch, "missing-initWithFrame");
                Self::log_native_glass_batch_result(&result);
                self.cache_native_glass_batch_result(&batch, &result);
                return (result, None);
            }

            self.clear_native_glass_batch_views();
            if self.native_substrate_view != nil {
                let () = msg_send![self.native_substrate_view, removeFromSuperview];
                self.native_substrate_view = nil;
            }

            let bounds: NSRect = msg_send![self.container_view, bounds];
            let native_container_frame =
                Self::native_glass_ns_rect_from_makepad_rect(container.rect, bounds.size.height);
            let native_container: ObjcId = msg_send![container_class, alloc];
            let native_container: ObjcId =
                msg_send![native_container, initWithFrame: native_container_frame];
            if native_container == nil {
                let result =
                    self.native_glass_class_missing_result(&batch, "container-alloc-init-failed");
                Self::log_native_glass_batch_result(&result);
                self.cache_native_glass_batch_result(&batch, &result);
                return (result, None);
            }

            let () = msg_send![
                native_container,
                setAutoresizingMask: Self::NS_VIEW_WIDTH_SIZABLE | Self::NS_VIEW_HEIGHT_SIZABLE
            ];
            let () = msg_send![native_container, setWantsLayer: YES];
            let set_spacing_sel = sel!(setSpacing:);
            let can_set_spacing: BOOL =
                msg_send![native_container, respondsToSelector: set_spacing_sel];
            if can_set_spacing == YES {
                let () = msg_send![native_container, setSpacing: container.spacing];
                crate::log!(
                    "[liquid-glass] native-container-spacing container={:?} spacing={:.3}",
                    container.id,
                    container.spacing
                );
            }

            let mut panels: Vec<&NativeGlassPanelDescriptor> = container
                .panels
                .iter()
                .filter(|panel| panel.visible)
                .collect();
            panels.sort_by_key(|panel| panel.z_order);

            let mut panel_results = Vec::new();
            for panel in panels {
                let panel_frame = Self::native_glass_panel_ns_rect(panel.rect, container.rect);
                let panel_view: ObjcId = msg_send![glass_class, alloc];
                let panel_view: ObjcId = msg_send![panel_view, initWithFrame: panel_frame];
                if panel_view == nil {
                    panel_results.push(NativeGlassPanelResult {
                        id: panel.id,
                        state: NativeGlassInstallState::Failed,
                        reason: "panel-alloc-init-failed",
                    });
                    continue;
                }

                let () = msg_send![panel_view, setWantsLayer: YES];
                let panel_layer: ObjcId = msg_send![panel_view, layer];
                let corner_radius = panel.shape.corner_radius_for_rect(panel.rect);
                if panel_layer != nil {
                    let () = msg_send![panel_layer, setMasksToBounds: YES];
                    let () = msg_send![panel_layer, setCornerRadius: corner_radius];
                }

                let set_style_sel = sel!(setStyle:);
                let can_set_style: BOOL = msg_send![panel_view, respondsToSelector: set_style_sel];
                if can_set_style == YES {
                    let () = msg_send![panel_view, setStyle: panel.style.macos_raw_value()];
                }

                let set_tint_sel = sel!(setTintColor:);
                let can_set_tint: BOOL = msg_send![panel_view, respondsToSelector: set_tint_sel];
                if can_set_tint == YES {
                    let tint = panel
                        .tint
                        .unwrap_or_else(|| Self::default_native_glass_tint(panel.style));
                    let ns_tint = Self::ns_color_from_vec4f(tint);
                    let () = msg_send![panel_view, setTintColor: ns_tint];
                }

                let set_corner_radius_sel = sel!(setCornerRadius:);
                let can_set_corner_radius: BOOL =
                    msg_send![panel_view, respondsToSelector: set_corner_radius_sel];
                if can_set_corner_radius == YES {
                    let () = msg_send![panel_view, setCornerRadius: corner_radius];
                }

                let () = msg_send![native_container, addSubview: panel_view];
                self.native_glass_panel_views.push(panel_view);
                panel_results.push(NativeGlassPanelResult {
                    id: panel.id,
                    state: NativeGlassInstallState::Installed,
                    reason: "installed",
                });
            }

            let () = msg_send![
                self.container_view,
                addSubview: native_container
                positioned: -1i64
                relativeTo: self.view
            ];
            self.native_glass_container_view = native_container;

            let installed_panels = panel_results
                .iter()
                .filter(|panel| panel.state == NativeGlassInstallState::Installed)
                .count();
            let failed_panels = panel_results.len().saturating_sub(installed_panels);
            let container_state = if failed_panels == 0 {
                NativeGlassInstallState::Installed
            } else if installed_panels == 0 {
                NativeGlassInstallState::Failed
            } else {
                NativeGlassInstallState::Partial
            };
            let backend_state = if failed_panels == 0 {
                NativeGlassBackendState::Installed
            } else if installed_panels == 0 {
                NativeGlassBackendState::Rejected
            } else {
                NativeGlassBackendState::Partial
            };
            let result_container = NativeGlassContainerResult::from_panel_results(
                container.id,
                container_state,
                if failed_panels == 0 {
                    "installed"
                } else {
                    "partial"
                },
                panel_results,
            );
            let result = NativeGlassBatchResult {
                window_id: batch.window_id,
                backend_state,
                containers: vec![result_container],
            };
            Self::log_native_glass_batch_result(&result);
            self.cache_native_glass_batch_result(&batch, &result);

            let first_style = container
                .panels
                .iter()
                .find(|panel| panel.visible && panel.hit_test == NativeGlassHitTest::Passthrough)
                .map(|panel| panel.style);
            if let Some(style) = first_style {
                crate::log!(
                    "[liquid-glass] state=4 substrate=macos-native style={} style_raw={}",
                    match style {
                        NativeGlassStyle::Regular => "regular",
                        NativeGlassStyle::Clear => "clear",
                    },
                    style.macos_raw_value()
                );
            }
            let compat_event = first_style.map(|style| WindowNativeSubstrateResolvedEvent {
                window_id: batch.window_id,
                state: Self::window_native_substrate_state_from_native_glass(container_state),
                style: Some(Self::event_style_from_native_glass(style)),
                reason: if failed_panels == 0 {
                    "installed-native-glass-batch"
                } else {
                    "partial-native-glass-batch"
                },
            });

            (result, compat_event)
        }
    }

    fn alloc_window(window_class: *const Class, window_id: WindowId) -> MacosWindow {
        unsafe {
            let pool: ObjcId = msg_send![class!(NSAutoreleasePool), new];

            let window: ObjcId = msg_send![window_class, alloc];
            let window_delegate: ObjcId = msg_send![get_macos_class_global().window_delegate, new];
            let container_view: ObjcId = msg_send![class!(NSView), alloc];
            let view: ObjcId = msg_send![get_macos_class_global().view, alloc];

            let () = msg_send![pool, drain];
            with_macos_app(|app| app.cocoa_windows.push((window, view)));
            MacosWindow {
                is_fullscreen: false,
                is_popup: false,
                macos_config: MacosWindowConfig::default(),
                visual_effect_view: nil,
                native_substrate_view: nil,
                native_glass_container_view: nil,
                native_glass_panel_views: Vec::new(),
                last_native_glass_batch: None,
                last_native_glass_batch_result: None,
                native_glass_control_views: Vec::new(),
                native_glass_control_targets: Vec::new(),
                last_native_glass_control_batch: None,
                native_glass_perform_click_probe_fired_controls: Vec::new(),
                native_glass_accessibility_press_probe_fired_controls: Vec::new(),
                native_glass_mouse_event_probe_fired_controls: Vec::new(),
                native_glass_cg_event_probe_fired_controls: Vec::new(),
                proof_substrate_view: nil,
                above_metal_glass_probe_view: nil,
                container_view,
                live_resize_timer: nil,
                window_delegate: window_delegate,
                window: window,
                window_id: window_id,
                view: view,
                last_window_geom: None,
                ime_spot: Vec2d::default(),
                last_mouse_pos: Vec2d::default(),
                ime_active: false,
            }
        }
    }

    pub fn new(window_id: WindowId, macos_config: MacosWindowConfig) -> MacosWindow {
        let window_class = match macos_config.kind {
            MacosWindowKind::Standard => get_macos_class_global().window,
            MacosWindowKind::FloatingPanel => get_macos_class_global().panel,
        };
        let mut window = Self::alloc_window(window_class, window_id);
        window.macos_config = macos_config.normalized();
        window
    }

    pub fn new_popup(window_id: WindowId) -> MacosWindow {
        Self::alloc_window(get_macos_class_global().window, window_id)
    }

    fn style_mask_for_config(config: MacosWindowConfig) -> u64 {
        let mut style_mask = NSWindowStyleMask::NSFullSizeContentViewWindowMask as u64;

        match config.chrome {
            MacosWindowChrome::Borderless => {
                style_mask |= NSWindowStyleMask::NSBorderlessWindowMask as u64;
                if config.resizable {
                    style_mask |= NSWindowStyleMask::NSResizableWindowMask as u64;
                }
            }
            MacosWindowChrome::Titled => {
                style_mask |= NSWindowStyleMask::NSTitledWindowMask as u64;
                if config.closable {
                    style_mask |= NSWindowStyleMask::NSClosableWindowMask as u64;
                }
                if config.miniaturizable {
                    style_mask |= NSWindowStyleMask::NSMiniaturizableWindowMask as u64;
                }
                if config.resizable {
                    style_mask |= NSWindowStyleMask::NSResizableWindowMask as u64;
                }
            }
        }

        if config.kind == MacosWindowKind::FloatingPanel && config.non_activating {
            style_mask |= NSWindowStyleMask::NSNonactivatingPanelWindowMask as u64;
        }

        style_mask
    }

    fn collection_behavior_for_config(config: MacosWindowConfig) -> u64 {
        let mut collection_behavior = 0;
        if config.join_all_spaces {
            collection_behavior |= NSWindowCollectionBehaviorCanJoinAllSpaces;
        }
        if config.full_screen_auxiliary {
            collection_behavior |= NSWindowCollectionBehaviorFullScreenAuxiliary;
        } else {
            collection_behavior |= NSWindowCollectionBehaviorFullScreenPrimary;
        }
        collection_behavior
    }

    fn level_to_native(level: MacosWindowLevel) -> i64 {
        match level {
            MacosWindowLevel::Normal => NSNormalWindowLevel,
            MacosWindowLevel::Floating => NSFloatingWindowLevel,
            MacosWindowLevel::StatusBar => NSStatusWindowLevel,
        }
    }

    pub fn set_window_level(&mut self, level: MacosWindowLevel) {
        unsafe {
            let () = msg_send![self.window, setLevel: Self::level_to_native(level)];
        }
    }

    pub fn set_topmost(&mut self, topmost: bool) {
        let level = if topmost {
            MacosWindowLevel::Floating
        } else {
            MacosWindowLevel::Normal
        };
        self.set_window_level(level);
        self.send_change_event();
    }

    fn is_topmost(&self) -> bool {
        let level: i64 = unsafe { msg_send![self.window, level] };
        level > NSNormalWindowLevel
    }

    pub fn is_nonactivating_panel(&self) -> bool {
        self.macos_config.kind == MacosWindowKind::FloatingPanel && self.macos_config.non_activating
    }

    pub fn needs_panel_to_become_key(&self) -> bool {
        self.macos_config.kind == MacosWindowKind::FloatingPanel
            && self.macos_config.becomes_key_only_if_needed
    }

    // complete window initialization with pointers to self
    pub fn init(
        &mut self,
        title: &str,
        size: Vec2d,
        position: Option<Vec2d>,
        is_fullscreen: bool,
        macos_config: MacosWindowConfig,
    ) {
        self.macos_config = macos_config.normalized();
        unsafe {
            let pool: ObjcId = msg_send![class!(NSAutoreleasePool), new];

            // set the backpointeers
            (*self.window_delegate).set_ivar("macos_window_ptr", self as *mut _ as *mut c_void);
            let () = msg_send![self.view, initWithPtr: self as *mut _ as *mut c_void];

            let left_top = if let Some(position) = position {
                NSPoint {
                    x: position.x as f64,
                    y: position.y as f64,
                }
            } else {
                NSPoint { x: 0., y: 0. }
            };
            let ns_size = NSSize {
                width: size.x as f64,
                height: size.y as f64,
            };
            let window_frame = NSRect {
                origin: left_top,
                size: ns_size,
            };
            let window_masks = Self::style_mask_for_config(self.macos_config);

            let () = msg_send![
                self.window,
                initWithContentRect: window_frame
                styleMask: window_masks as u64
                backing: NSBackingStoreType::NSBackingStoreBuffered as u64
                defer: NO
            ];

            let () = msg_send![self.window, setDelegate: self.window_delegate];

            let title = str_to_nsstring(title);
            let () = msg_send![self.window, setReleasedWhenClosed: NO];
            let () = msg_send![self.window, setTitle: title];
            let () = msg_send![self.window, setTitleVisibility: NSWindowTitleVisibility::NSWindowTitleHidden];
            let () = msg_send![self.window, setTitlebarAppearsTransparent: YES];
            let () = msg_send![
                self.window,
                setCollectionBehavior: Self::collection_behavior_for_config(self.macos_config)
            ];
            self.set_window_level(self.macos_config.level);

            if self.macos_config.kind == MacosWindowKind::FloatingPanel {
                let becomes_key_only_if_needed = if self.macos_config.becomes_key_only_if_needed {
                    YES
                } else {
                    NO
                };
                let () = msg_send![self.window, setHidesOnDeactivate: NO];
                let () = msg_send![
                    self.window,
                    setBecomesKeyOnlyIfNeeded: becomes_key_only_if_needed
                ];
            }

            let () = msg_send![self.window, setAcceptsMouseMovedEvents: YES];

            let () = msg_send![self.view, setLayerContentsRedrawPolicy: 2];

            let rect = NSRect {
                origin: NSPoint { x: 0., y: 0. },
                size: ns_size,
            };
            self.init_container_view(rect);
            let () = msg_send![self.window, setContentView: self.container_view];
            let () = msg_send![self.container_view, addSubview: self.view];
            let () = msg_send![self.window, makeFirstResponder: self.view];
            if self.is_nonactivating_panel() {
                let () = msg_send![self.window, orderFront: nil];
            } else {
                let () = msg_send![self.window, makeKeyAndOrderFront: nil];
            }

            let track: ObjcId = msg_send![class!(NSTrackingArea), alloc];
            let track: ObjcId = msg_send![
                track,
                initWithRect: rect
                options: NSTrackignActiveAlways
                    | NSTrackingInVisibleRect
                    | NSTrackingMouseEnteredAndExited
                    | NSTrackingMouseMoved
                    | NSTrackingCursorUpdate
                owner: self.view
                userInfo: nil
            ];
            let () = msg_send![self.view, addTrackingArea: track];

            if position.is_none() {
                let () = msg_send![self.window, center];
            }

            self.last_window_geom = Some(self.get_window_geom());

            let input_context: ObjcId = msg_send![self.view, inputContext];
            let () = msg_send![input_context, invalidateCharacterCoordinates];
            if is_fullscreen {
                self.maximize();
            }

            Self::set_application_icon();

            let () = msg_send![pool, drain];
        }
    }

    // complete window initialization with pointers to self
    /// Initialize as a popup window (borderless NSPanel at popup menu level).
    /// `position` is in screen coordinates. `parent_window` is the parent NSWindow for coordinate conversion.
    pub fn init_popup(&mut self, size: Vec2d, position: Vec2d, parent_window: ObjcId) {
        self.is_popup = true;
        unsafe {
            let pool: ObjcId = msg_send![class!(NSAutoreleasePool), new];

            // set the backpointers
            (*self.window_delegate).set_ivar("macos_window_ptr", self as *mut _ as *mut c_void);
            let () = msg_send![self.view, initWithPtr: self as *mut _ as *mut c_void];

            // Convert position from parent-client coordinates to screen coordinates.
            // The position is relative to the parent window's content view origin (top-left).
            let parent_frame: NSRect = msg_send![parent_window, frame];
            let parent_content: NSRect = msg_send![parent_window, contentLayoutRect];
            // macOS screen coordinates: origin at bottom-left.
            // Parent content top-left in screen coords:
            let screen_x = parent_frame.origin.x + parent_content.origin.x + position.x;
            // Flip Y: parent content top is at frame.origin.y + frame.size.height - titlebar
            let parent_content_top =
                parent_frame.origin.y + parent_frame.size.height - parent_content.origin.y;
            let screen_y = parent_content_top - position.y - size.y;

            let ns_size = NSSize {
                width: size.x as f64,
                height: size.y as f64,
            };
            let window_frame = NSRect {
                origin: NSPoint {
                    x: screen_x,
                    y: screen_y,
                },
                size: ns_size,
            };

            // NSPanel with borderless style
            let window_masks = NSWindowStyleMask::NSBorderlessWindowMask as u64
                | NSWindowStyleMask::NSFullSizeContentViewWindowMask as u64;

            let () = msg_send![
                self.window,
                initWithContentRect: window_frame
                styleMask: window_masks as u64
                backing: NSBackingStoreType::NSBackingStoreBuffered as u64
                defer: NO
            ];

            let () = msg_send![self.window, setDelegate: self.window_delegate];
            let () = msg_send![self.window, setReleasedWhenClosed: NO];

            let () = msg_send![self.window, setLevel: NSPopUpMenuWindowLevel];
            let () = msg_send![self.window, setHasShadow: YES];

            let () = msg_send![self.window, setAcceptsMouseMovedEvents: YES];

            let () = msg_send![self.view, setLayerContentsRedrawPolicy: 2]; //duringViewResize

            let rect = NSRect {
                origin: NSPoint { x: 0., y: 0. },
                size: ns_size,
            };
            self.init_container_view(rect);
            let () = msg_send![self.window, setContentView: self.container_view];
            let () = msg_send![self.container_view, addSubview: self.view];
            let () = msg_send![self.window, makeFirstResponder: self.view];

            // orderFront instead of makeKeyAndOrderFront to avoid stealing key focus initially
            // Then makeKey so we get resignKey on focus loss
            let () = msg_send![self.window, orderFront: nil];
            let () = msg_send![self.window, makeKeyWindow];

            let track: ObjcId = msg_send![class!(NSTrackingArea), alloc];
            let track: ObjcId = msg_send![
                track,
                initWithRect: rect
                options: NSTrackignActiveAlways
                    | NSTrackingInVisibleRect
                    | NSTrackingMouseEnteredAndExited
                    | NSTrackingMouseMoved
                    | NSTrackingCursorUpdate
                owner: self.view
                userInfo: nil
            ];
            let () = msg_send![self.view, addTrackingArea: track];

            let input_context: ObjcId = msg_send![self.view, inputContext];
            let () = msg_send![input_context, invalidateCharacterCoordinates];

            let () = msg_send![pool, drain];
        }
    }

    /// Set the application dock icon from the default Makepad icon (RGBA8 bitmap).
    unsafe fn set_application_icon() {
        let icon = crate::app_icon::window_icon();
        let buf = match icon.buffers.first() {
            Some(b) => b,
            None => return,
        };
        let width = buf.width as usize;
        let height = buf.height as usize;

        let bitmap_rep: ObjcId = msg_send![class!(NSBitmapImageRep), alloc];
        let bitmap_rep: ObjcId = msg_send![bitmap_rep,
            initWithBitmapDataPlanes: std::ptr::null_mut::<*mut u8>()
            pixelsWide: width as i64
            pixelsHigh: height as i64
            bitsPerSample: 8i64
            samplesPerPixel: 4i64
            hasAlpha: YES
            isPlanar: NO
            colorSpaceName: str_to_nsstring("NSDeviceRGBColorSpace")
            bytesPerRow: (width * 4) as i64
            bitsPerPixel: 32i64
        ];
        if bitmap_rep == nil {
            return;
        }

        let bitmap_data: *mut u8 = msg_send![bitmap_rep, bitmapData];
        if !bitmap_data.is_null() {
            std::ptr::copy_nonoverlapping(buf.data.as_ptr(), bitmap_data, width * height * 4);
        }

        let size = NSSize {
            width: width as f64,
            height: height as f64,
        };
        let ns_image: ObjcId = msg_send![class!(NSImage), alloc];
        let ns_image: ObjcId = msg_send![ns_image, initWithSize: size];
        let () = msg_send![ns_image, addRepresentation: bitmap_rep];

        let ns_app: ObjcId = msg_send![class!(NSApplication), sharedApplication];
        let () = msg_send![ns_app, setApplicationIconImage: ns_image];
    }

    pub fn set_ime_spot(&mut self, spot: Vec2d) {
        self.ime_spot = spot;
    }

    pub fn start_live_resize(&mut self) {
        if self.live_resize_timer != nil {
            return;
        }
        unsafe {
            let pool: ObjcId = msg_send![class!(NSAutoreleasePool), new];
            let timer_delegate_instance = with_macos_app(|app| app.timer_delegate_instance);
            self.live_resize_timer = msg_send![
                class!(NSTimer),
                timerWithTimeInterval: 0.01666666
                target: timer_delegate_instance
                selector: sel!(receivedLiveResize:)
                userInfo: nil
                repeats: YES
            ];
            let nsrunloop: ObjcId = msg_send![class!(NSRunLoop), mainRunLoop];
            let () = msg_send![nsrunloop, addTimer: self.live_resize_timer forMode: NSRunLoopCommonModes];

            let () = msg_send![pool, release];
        }

        self.do_callback(MacosEvent::WindowResizeLoopStart(self.window_id));
    }

    pub fn end_live_resize(&mut self) {
        unsafe {
            if self.live_resize_timer != nil {
                let () = msg_send![self.live_resize_timer, invalidate];
                self.live_resize_timer = nil;
            }
        }
        self.do_callback(MacosEvent::WindowResizeLoopStop(self.window_id));
    }

    pub fn close_window(&mut self) {
        unsafe {
            //get_macos_app_global();
            let () = msg_send![self.window, close];
        }
    }

    pub fn restore(&mut self) {
        unsafe {
            let () = msg_send![self.window, toggleFullScreen: nil];
        }
    }
    pub fn hide(&mut self) {
        unsafe {
            let () = msg_send![self.window, orderOut: nil];
        }
    }
    pub fn deminiaturize(&mut self) {
        unsafe {
            let () = msg_send![self.window, deminiaturize: nil];
        }
    }
    pub fn maximize(&mut self) {
        unsafe {
            let () = msg_send![self.window, toggleFullScreen: nil];
        }
    }

    pub fn minimize(&mut self) {
        unsafe {
            let () = msg_send![self.window, miniaturize: nil];
        }
    }

    pub fn set_window_buttons_visible(&mut self, visible: bool) {
        unsafe {
            let close: ObjcId = msg_send![self.window, standardWindowButton: 0u64];
            let miniaturize: ObjcId = msg_send![self.window, standardWindowButton: 1u64];
            let zoom: ObjcId = msg_send![self.window, standardWindowButton: 2u64];
            let hidden = if visible { NO } else { YES };
            let () = msg_send![close, setHidden: hidden];
            let () = msg_send![miniaturize, setHidden: hidden];
            let () = msg_send![zoom, setHidden: hidden];
        }
    }

    pub fn set_window_visuals(&mut self, visuals: WindowVisuals) {
        const NS_VIEW_WIDTH_SIZABLE: i64 = 1 << 1;
        const NS_VIEW_HEIGHT_SIZABLE: i64 = 1 << 4;
        const NS_VISUAL_EFFECT_MATERIAL_HUD_WINDOW: i64 = 1;
        const NS_VISUAL_EFFECT_MATERIAL_UNDER_WINDOW_BACKGROUND: i64 = 12;
        const NS_VISUAL_EFFECT_BLENDING_MODE_BEHIND_WINDOW: i64 = 0;
        const NS_VISUAL_EFFECT_STATE_ACTIVE: i64 = 1;

        unsafe {
            let opaque = if visuals.transparent { NO } else { YES };
            let () = msg_send![self.window, setOpaque: opaque];
            let bg_color = if visuals.transparent {
                let clear: ObjcId = msg_send![class!(NSColor), clearColor];
                clear
            } else {
                let color: ObjcId = msg_send![class!(NSColor), windowBackgroundColor];
                color
            };
            let () = msg_send![self.window, setBackgroundColor: bg_color];

            let use_effect = visuals.backdrop != WindowBackdrop::None;
            if use_effect {
                let effect_view = if self.visual_effect_view == nil {
                    let effect_view: ObjcId = msg_send![class!(NSVisualEffectView), alloc];
                    let bounds: NSRect = msg_send![self.view, bounds];
                    let effect_view: ObjcId = msg_send![effect_view, initWithFrame: bounds];
                    let () = msg_send![
                        effect_view,
                        setAutoresizingMask: NS_VIEW_WIDTH_SIZABLE | NS_VIEW_HEIGHT_SIZABLE
                    ];
                    let () = msg_send![self.view, addSubview: effect_view positioned: 0i64 relativeTo: nil];
                    self.visual_effect_view = effect_view;
                    effect_view
                } else {
                    self.visual_effect_view
                };
                let material = match visuals.backdrop {
                    WindowBackdrop::Blur => NS_VISUAL_EFFECT_MATERIAL_HUD_WINDOW,
                    WindowBackdrop::Auto | WindowBackdrop::Vibrancy => {
                        NS_VISUAL_EFFECT_MATERIAL_UNDER_WINDOW_BACKGROUND
                    }
                    WindowBackdrop::Mica | WindowBackdrop::Acrylic => {
                        NS_VISUAL_EFFECT_MATERIAL_UNDER_WINDOW_BACKGROUND
                    }
                    WindowBackdrop::None => NS_VISUAL_EFFECT_MATERIAL_UNDER_WINDOW_BACKGROUND,
                };
                let () = msg_send![effect_view, setMaterial: material];
                let () = msg_send![
                    effect_view,
                    setBlendingMode: NS_VISUAL_EFFECT_BLENDING_MODE_BEHIND_WINDOW
                ];
                let () = msg_send![effect_view, setState: NS_VISUAL_EFFECT_STATE_ACTIVE];
                let alpha = visuals.backdrop_intensity.clamp(0.0, 1.0) as f64;
                let () = msg_send![effect_view, setAlphaValue: alpha];
            } else if self.visual_effect_view != nil {
                let () = msg_send![self.visual_effect_view, removeFromSuperview];
                self.visual_effect_view = nil;
            }
        }
    }

    pub fn time_now(&self) -> f64 {
        with_macos_app(|app| app.time_now())
    }

    /// Returns the bounding box of all three macOS traffic-light buttons
    /// (close / miniaturize / zoom) in Makepad's coordinate system
    /// (top-left origin, Y increases downward).
    ///
    /// Returns `None` if any button is missing (e.g., borderless popup windows).
    pub fn traffic_lights_geom(&self) -> Option<Rect> {
        unsafe {
            let close: ObjcId = msg_send![self.window, standardWindowButton: 0u64];
            let miniaturize: ObjcId = msg_send![self.window, standardWindowButton: 1u64];
            let zoom: ObjcId = msg_send![self.window, standardWindowButton: 2u64];
            if close.is_null() || miniaturize.is_null() || zoom.is_null() {
                return None;
            }
            let content_view: ObjcId = msg_send![self.window, contentView];
            let content_frame: NSRect = msg_send![content_view, frame];
            let h = content_frame.size.height;

            // Convert a button's NSRect from its superview's coord space to the
            // content view's coord space, then flip to Makepad's top-left origin.
            let ns_to_rect = |btn: ObjcId| -> (f64, f64, f64, f64) {
                let superview: ObjcId = msg_send![btn, superview];
                let ns_frame: NSRect = msg_send![btn, frame];
                let ns_frame: NSRect = msg_send![
                    content_view,
                    convertRect: ns_frame
                    fromView: superview
                ];
                let top = h - (ns_frame.origin.y + ns_frame.size.height);
                let left = ns_frame.origin.x;
                let right = ns_frame.origin.x + ns_frame.size.width;
                let bottom = h - ns_frame.origin.y;
                (top, left, right, bottom)
            };

            let (t0, l0, r0, b0) = ns_to_rect(close);
            let (t1, l1, r1, b1) = ns_to_rect(miniaturize);
            let (t2, l2, r2, b2) = ns_to_rect(zoom);

            let top = t0.min(t1).min(t2);
            let left = l0.min(l1).min(l2);
            let right = r0.max(r1).max(r2);
            let bottom = b0.max(b1).max(b2);

            // During a fullscreen transition the content view resizes before
            // the buttons' superview repositions, so the Y-flip can place the
            // buttons near the bottom of the view.  Traffic-light buttons are
            // always near the top of the window, so discard bogus geometry
            // where they appear in the lower half of the content area.
            if top > h * 0.5 {
                return None;
            }

            let rect = Rect {
                pos: Vec2d { x: left, y: top },
                size: Vec2d {
                    x: right - left,
                    y: bottom - top,
                },
            };

            Some(rect)
        }
    }

    pub fn get_window_geom(&self) -> WindowGeom {
        WindowGeom {
            xr_is_presenting: false,
            is_topmost: self.is_topmost(),
            is_fullscreen: self.is_fullscreen,
            can_fullscreen: false,
            inner_size: self.get_inner_size(),
            outer_size: self.get_outer_size(),
            dpi_factor: self.get_dpi_factor(),
            position: self.get_position(),
            window_chrome_buttons: self.traffic_lights_geom().unwrap_or_default(),
            ..Default::default()
        }
    }

    pub fn do_callback(&mut self, event: MacosEvent) {
        MacosApp::do_callback(event);
    }

    pub fn set_position(&mut self, pos: Vec2d) {
        let mut window_frame: NSRect = unsafe { msg_send![self.window, frame] };
        window_frame.origin.x = pos.x as f64;
        window_frame.origin.y = pos.y as f64;
        //not very nice: CGDisplay::main().pixels_high() as f64
        unsafe {
            let () = msg_send![self.window, setFrame: window_frame display: YES];
        };
    }

    pub fn get_position(&self) -> Vec2d {
        let window_frame: NSRect = unsafe { msg_send![self.window, frame] };
        Vec2d {
            x: window_frame.origin.x,
            y: window_frame.origin.y,
        }
    }

    pub fn get_ime_origin(&self) -> Vec2d {
        let shift_x = 5.0; // unknown why
        let shift_y = -10.0;
        let rect = NSRect {
            origin: NSPoint { x: 0.0, y: 0.0 },
            //view_frame.size.height),
            size: NSSize {
                width: 0.0,
                height: 0.0,
            },
        };
        let out: NSRect = unsafe { msg_send![self.window, convertRectToScreen: rect] };
        Vec2d {
            x: out.origin.x + shift_x,
            y: out.origin.y + shift_y,
        }
    }

    pub fn get_inner_size(&self) -> Vec2d {
        let view_frame: NSRect = unsafe { msg_send![self.view, frame] };
        Vec2d {
            x: view_frame.size.width,
            y: view_frame.size.height,
        }
    }

    pub fn get_outer_size(&self) -> Vec2d {
        let window_frame: NSRect = unsafe { msg_send![self.window, frame] };
        Vec2d {
            x: window_frame.size.width,
            y: window_frame.size.height,
        }
    }

    pub fn set_outer_size(&self, size: Vec2d) {
        let mut window_frame: NSRect = unsafe { msg_send![self.window, frame] };
        window_frame.size.width = size.x;
        window_frame.size.height = size.y;
        unsafe {
            let () = msg_send![self.window, setFrame: window_frame display: YES];
        };
    }

    pub fn get_dpi_factor(&self) -> f64 {
        let scale: f64 = unsafe { msg_send![self.window, backingScaleFactor] };
        scale
    }

    pub fn send_change_event(&mut self) {
        //return;
        let new_geom = self.get_window_geom();
        let old_geom = if let Some(old_geom) = &self.last_window_geom {
            old_geom.clone()
        } else {
            new_geom.clone()
        };
        self.last_window_geom = Some(new_geom.clone());
        self.do_callback(MacosEvent::WindowGeomChange(WindowGeomChangeEvent {
            window_id: self.window_id,
            old_geom: old_geom,
            new_geom: new_geom,
        }));
        self.do_callback(MacosEvent::Paint);
        // we should schedule a timer for +16ms another Paint
    }

    pub(crate) fn window_geom_change_from_old_geom(
        &mut self,
        old_geom: WindowGeom,
    ) -> WindowGeomChangeEvent {
        let new_geom = self.get_window_geom();
        self.last_window_geom = Some(new_geom.clone());
        WindowGeomChangeEvent {
            window_id: self.window_id,
            old_geom,
            new_geom,
        }
    }

    pub fn send_got_focus_event(&mut self) {
        self.do_callback(MacosEvent::WindowGotFocus(self.window_id));
    }

    pub fn send_lost_focus_event(&mut self) {
        if self.is_popup {
            self.do_callback(MacosEvent::PopupDismissed(
                crate::event::window::PopupDismissedEvent {
                    window_id: self.window_id,
                    reason: crate::event::window::PopupDismissReason::FocusLost,
                },
            ));
            return;
        }
        self.do_callback(MacosEvent::WindowLostFocus(self.window_id));
    }

    pub fn send_popup_escape_dismiss_event(&mut self) -> bool {
        if !self.is_popup {
            return false;
        }
        crate::log!(
            "[liquid-glass] transient-window=popup-dismiss event=escape popup={:?}",
            self.window_id
        );
        self.do_callback(MacosEvent::PopupDismissed(
            crate::event::window::PopupDismissedEvent {
                window_id: self.window_id,
                reason: crate::event::window::PopupDismissReason::Escape,
            },
        ));
        true
    }

    pub fn mouse_down_can_drag_window(&mut self) -> bool {
        let response = Rc::new(Cell::new(WindowDragQueryResponse::NoAnswer));
        self.do_callback(MacosEvent::WindowDragQuery(WindowDragQueryEvent {
            window_id: self.window_id,
            abs: self.last_mouse_pos,
            response: response.clone(),
        }));
        match response.get() {
            WindowDragQueryResponse::Caption | WindowDragQueryResponse::SysMenu => true,
            WindowDragQueryResponse::Client | WindowDragQueryResponse::NoAnswer => false,
        }
    }

    pub fn send_mouse_down(&mut self, button: MouseButton, modifiers: KeyModifiers) {
        let () = unsafe { msg_send![self.window, makeFirstResponder: self.view] };
        self.do_callback(MacosEvent::MouseDown(MouseDownEvent {
            button,
            modifiers,
            window_id: self.window_id,
            abs: self.last_mouse_pos,
            time: self.time_now(),
            handled: Cell::new(Area::Empty),
        }));
    }

    pub fn send_mouse_up(&mut self, button: MouseButton, modifiers: KeyModifiers) {
        self.do_callback(MacosEvent::MouseUp(MouseUpEvent {
            button,
            modifiers,
            window_id: self.window_id,
            abs: self.last_mouse_pos,
            time: self.time_now(),
        }));
    }

    pub fn send_mouse_move(&mut self, _event: ObjcId, pos: Vec2d, modifiers: KeyModifiers) {
        self.last_mouse_pos = pos;

        if !self.is_nonactivating_panel() {
            with_macos_app(|app| app.startup_focus_hack());
        }

        self.do_callback(MacosEvent::MouseMove(MouseMoveEvent {
            window_id: self.window_id,
            abs: pos,
            modifiers: modifiers,
            time: self.time_now(),
            handled: Cell::new(Area::Empty),
        }));

        //get_macos_app_global().ns_event = ptr::null_mut();
    }

    pub fn send_scroll(&mut self, scroll: Vec2d, modifiers: KeyModifiers, is_mouse: bool) {
        self.do_callback(MacosEvent::Scroll(ScrollEvent {
            window_id: self.window_id,
            scroll,
            abs: self.last_mouse_pos,
            modifiers,
            time: self.time_now(),
            is_mouse,
            handled_x: Cell::new(false),
            handled_y: Cell::new(false),
        }));
    }

    pub fn send_window_close_requested_event(&mut self) -> bool {
        let accept_close = Rc::new(Cell::new(true));
        self.do_callback(MacosEvent::WindowCloseRequested(
            WindowCloseRequestedEvent {
                window_id: self.window_id,
                accept_close: accept_close.clone(),
            },
        ));
        if !accept_close.get() {
            return false;
        }
        true
    }

    pub fn send_window_closed_event(&mut self) {
        self.do_callback(MacosEvent::WindowClosed(WindowClosedEvent {
            window_id: self.window_id,
        }))
    }

    pub fn send_text_input(&mut self, input: String, replace_last: bool) {
        self.do_callback(MacosEvent::TextInput(TextInputEvent {
            input: input,
            was_paste: false,
            replace_last: replace_last,
            ..Default::default()
        }))
    }

    pub fn set_ime_active(&mut self, active: bool) {
        self.ime_active = active;
    }

    #[cfg(target_os = "macos")]
    pub fn start_dragging(&mut self, items: Vec<DragItem>) {
        let ns_event: ObjcId = unsafe {
            let ns_app: ObjcId = msg_send![class!(NSApplication), sharedApplication];
            msg_send![ns_app, currentEvent]
        };
        let mut dragged_files = Vec::new();
        for item in items {
            match item {
                DragItem::FilePath { path, internal_id } => {
                    let pasteboard_item: ObjcId =
                        unsafe { msg_send![class!(NSPasteboardItem), new] };
                    let _: () = unsafe {
                        msg_send![
                            pasteboard_item,
                            setString: str_to_nsstring(
                                &if let Some(id) = internal_id{
                                    format!("file://{}#makepad_internal_id={}", if path.len()==0{"makepad_internal_empty"}else {&path}, id.0)
                                }
                                else{
                                    format!("file://{}",if path.len()==0{"makepad_internal_empty"}else {&path})
                                }
                            )
                            forType: NSPasteboardTypeFileURL
                        ]
                    };
                    let dragging_item: ObjcId = unsafe { msg_send![class!(NSDraggingItem), alloc] };
                    let _: () = unsafe {
                        msg_send![dragging_item, initWithPasteboardWriter: pasteboard_item]
                    };
                    let bounds: NSRect = unsafe { msg_send![self.view, bounds] };
                    let _: () = unsafe {
                        msg_send![dragging_item, setDraggingFrame: bounds contents: self.view]
                    };
                    dragged_files.push(dragging_item)
                }
                _ => {
                    crate::error!("Dragging string not implemented on macos yet");
                }
            }
        }

        let dragging_items: ObjcId = unsafe {
            msg_send![
                class!(NSArray),
                arrayWithObjects: dragged_files.as_ptr()
                count: dragged_files.len()
            ]
        };

        unsafe {
            let _: ObjcId = msg_send![
                self.view,
                beginDraggingSessionWithItems: dragging_items
                event: ns_event
                source: self.view
            ];
        }

        /*
         self.delegate?.cellClick(self ,index:self.index)
        //
        let pasteboardItem = NSPasteboardItem()
        pasteboardItem.setString(zText!.stringValue, forType:.string)
        let draggingItem = NSDraggingItem(pasteboardWriter: pasteboardItem)
        draggingItem.setDraggingFrame(self.bounds, contents:self)
        beginDraggingSession(with: [draggingItem], event: event, source: self.zIcon.image)
        */

        // TODO
    }
}

pub fn get_cocoa_window(this: &Object) -> &mut MacosWindow {
    unsafe {
        let ptr: *mut c_void = *this.get_ivar("macos_window_ptr");
        &mut *(ptr as *mut MacosWindow)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        LiveId, NativeGlassButtonRole, NativeGlassContainerDescriptor,
        NativeGlassControlBatchValidationError, NativeGlassControlKind, NativeGlassShape,
    };

    #[test]
    fn borderless_standard_windows_can_still_resize() {
        let config = MacosWindowConfig {
            chrome: MacosWindowChrome::Borderless,
            resizable: true,
            ..MacosWindowConfig::default()
        };

        let style_mask = MacosWindow::style_mask_for_config(config);

        assert_ne!(
            style_mask & NSWindowStyleMask::NSResizableWindowMask as u64,
            0
        );
    }

    #[test]
    fn standard_windows_use_fullscreen_primary_collection_behavior() {
        let behavior = MacosWindow::collection_behavior_for_config(MacosWindowConfig::default());

        assert_ne!(behavior & NSWindowCollectionBehaviorFullScreenPrimary, 0);
        assert_eq!(behavior & NSWindowCollectionBehaviorFullScreenAuxiliary, 0);
    }

    #[test]
    fn auxiliary_windows_do_not_use_fullscreen_primary_collection_behavior() {
        let behavior =
            MacosWindow::collection_behavior_for_config(MacosWindowConfig::floating_panel());

        assert_ne!(behavior & NSWindowCollectionBehaviorFullScreenAuxiliary, 0);
        assert_eq!(behavior & NSWindowCollectionBehaviorFullScreenPrimary, 0);
    }

    #[test]
    fn native_glass_ns_rect_flips_makepad_logical_y_axis() {
        let rect = Rect {
            pos: Vec2d { x: 10.0, y: 20.0 },
            size: Vec2d { x: 100.0, y: 40.0 },
        };

        let ns_rect = MacosWindow::native_glass_ns_rect_from_makepad_rect(rect, 300.0);

        assert_eq!(ns_rect.origin.x, 10.0);
        assert_eq!(ns_rect.origin.y, 240.0);
        assert_eq!(ns_rect.size.width, 100.0);
        assert_eq!(ns_rect.size.height, 40.0);
    }

    #[test]
    fn native_glass_panel_ns_rect_is_relative_to_container_rect() {
        let container = Rect {
            pos: Vec2d { x: 100.0, y: 50.0 },
            size: Vec2d { x: 400.0, y: 300.0 },
        };
        let panel = Rect {
            pos: Vec2d { x: 120.0, y: 70.0 },
            size: Vec2d { x: 100.0, y: 40.0 },
        };

        let ns_rect = MacosWindow::native_glass_panel_ns_rect(panel, container);

        assert_eq!(ns_rect.origin.x, 20.0);
        assert_eq!(ns_rect.origin.y, 240.0);
        assert_eq!(ns_rect.size.width, 100.0);
        assert_eq!(ns_rect.size.height, 40.0);
    }

    #[test]
    fn native_glass_panel_frame_snapshot_line_contains_logical_and_appkit_frames() {
        let panel = NativeGlassPanelDescriptor {
            id: LiveId(2),
            rect: Rect {
                pos: Vec2d { x: 120.0, y: 70.0 },
                size: Vec2d { x: 100.0, y: 40.0 },
            },
            shape: NativeGlassShape::RoundedRect { radius: 12.0 },
            style: NativeGlassStyle::Clear,
            tint: None,
            hit_test: NativeGlassHitTest::Passthrough,
            z_order: 7,
            visible: true,
        };
        let panel_frame = NSRect {
            origin: NSPoint { x: 20.0, y: 240.0 },
            size: NSSize {
                width: 100.0,
                height: 40.0,
            },
        };

        let line =
            MacosWindow::native_glass_panel_frame_snapshot_line(LiveId(1), &panel, panel_frame);

        assert!(line.contains("native-panel-frame"));
        assert!(line.contains("container=0000000000000001"));
        assert!(line.contains("panel=0000000000000002"));
        assert!(line.contains("makepad=(120.0,70.0,100.0,40.0)"));
        assert!(line.contains("appkit=(20.0,240.0,100.0,40.0)"));
        assert!(line.contains("z_order=7"));
        assert!(line.contains("visible=true"));
    }

    #[test]
    fn native_glass_control_frame_snapshot_line_contains_logical_and_appkit_frames() {
        let control = NativeGlassControlDescriptor {
            id: LiveId(3),
            rect: Rect {
                pos: Vec2d { x: 120.0, y: 70.0 },
                size: Vec2d { x: 100.0, y: 40.0 },
            },
            kind: NativeGlassControlKind::Button {
                role: NativeGlassButtonRole::Default,
            },
            label: "Clear".to_string(),
            style: NativeGlassStyle::Clear,
            tint: None,
            z_order: 4,
            enabled: true,
            visible: true,
        };
        let control_frame = NSRect {
            origin: NSPoint { x: 120.0, y: 190.0 },
            size: NSSize {
                width: 100.0,
                height: 40.0,
            },
        };

        let line = MacosWindow::native_glass_control_frame_snapshot_line(&control, control_frame);

        assert!(line.contains("native-control-frame"));
        assert!(line.contains("control=0000000000000003"));
        assert!(line.contains("label=\"Clear\""));
        assert!(line.contains("makepad=(120.0,70.0,100.0,40.0)"));
        assert!(line.contains("appkit=(120.0,190.0,100.0,40.0)"));
        assert!(line.contains("z_order=4"));
        assert!(line.contains("visible=true"));
    }

    #[test]
    fn native_glass_control_hit_test_probe_line_contains_result() {
        let control = NativeGlassControlDescriptor {
            id: LiveId(9),
            rect: Rect {
                pos: Vec2d { x: 10.0, y: 20.0 },
                size: Vec2d { x: 40.0, y: 24.0 },
            },
            kind: NativeGlassControlKind::Button {
                role: NativeGlassButtonRole::Default,
            },
            label: "Send".to_string(),
            style: NativeGlassStyle::Clear,
            tint: None,
            z_order: 0,
            enabled: true,
            visible: true,
        };

        let line = MacosWindow::native_glass_control_hit_test_probe_line(
            &control,
            NSPoint { x: 30.0, y: 32.0 },
            "NativeGlassButton",
            true,
        );

        assert!(line.contains("event=appkit-hit-test-probe"));
        assert!(line.contains("control=0000000000000009"));
        assert!(line.contains("label=\"Send\""));
        assert!(line.contains("point=(30.0,32.0)"));
        assert!(line.contains("result_class=NativeGlassButton"));
        assert!(line.contains("matches_control=true"));
    }

    #[test]
    fn native_glass_control_perform_click_probe_matches_explicit_label() {
        let control = NativeGlassControlDescriptor {
            id: LiveId(10),
            rect: Rect {
                pos: Vec2d { x: 10.0, y: 20.0 },
                size: Vec2d { x: 40.0, y: 24.0 },
            },
            kind: NativeGlassControlKind::Button {
                role: NativeGlassButtonRole::Default,
            },
            label: "Clear".to_string(),
            style: NativeGlassStyle::Clear,
            tint: None,
            z_order: 0,
            enabled: true,
            visible: true,
        };

        assert!(
            MacosWindow::native_glass_control_perform_click_probe_matches_value(
                Some("Clear"),
                &control
            )
        );
        assert!(
            MacosWindow::native_glass_control_perform_click_probe_matches_value(
                Some("clear"),
                &control
            )
        );
        assert!(
            MacosWindow::native_glass_control_perform_click_probe_matches_value(
                Some("all"),
                &control
            )
        );
        assert!(
            !MacosWindow::native_glass_control_perform_click_probe_matches_value(
                Some("Send"),
                &control
            )
        );
        assert!(
            !MacosWindow::native_glass_control_perform_click_probe_matches_value(None, &control)
        );
    }

    #[test]
    fn native_glass_control_probe_key_includes_rect() {
        let control = NativeGlassControlDescriptor {
            id: LiveId(10),
            rect: Rect {
                pos: Vec2d { x: 10.0, y: 20.0 },
                size: Vec2d { x: 40.0, y: 24.0 },
            },
            kind: NativeGlassControlKind::Button {
                role: NativeGlassButtonRole::Default,
            },
            label: "Clear".to_string(),
            style: NativeGlassStyle::Clear,
            tint: None,
            z_order: 0,
            enabled: true,
            visible: true,
        };
        let mut moved_control = control.clone();
        moved_control.rect.pos.x += 12.0;
        let fired = vec![MacosWindow::native_glass_control_probe_key(&control)];

        assert!(MacosWindow::native_glass_control_probe_already_fired(
            &fired, &control
        ));
        assert!(!MacosWindow::native_glass_control_probe_already_fired(
            &fired,
            &moved_control
        ));
    }

    #[test]
    fn native_glass_button_bezel_style_defaults_to_macos26_glass_raw_value() {
        assert_eq!(
            MacosWindow::native_glass_button_bezel_style_raw_from_value(None),
            16
        );
        assert_eq!(
            MacosWindow::native_glass_button_bezel_style_raw_from_value(Some("18")),
            18
        );
        assert_eq!(
            MacosWindow::native_glass_button_bezel_style_raw_from_value(Some("not-a-number")),
            16
        );
    }

    #[test]
    fn native_glass_button_style_line_records_glass_bezel_raw_value() {
        let control = NativeGlassControlDescriptor {
            id: LiveId(12),
            rect: Rect {
                pos: Vec2d { x: 10.0, y: 20.0 },
                size: Vec2d { x: 40.0, y: 24.0 },
            },
            kind: NativeGlassControlKind::Button {
                role: NativeGlassButtonRole::Default,
            },
            label: "Clear".to_string(),
            style: NativeGlassStyle::Clear,
            tint: None,
            z_order: 0,
            enabled: true,
            visible: true,
        };

        let line = MacosWindow::native_glass_button_style_line(&control, 16);

        assert!(line.contains("button-style"));
        assert!(line.contains("control=000000000000000c"));
        assert!(line.contains("label=\"Clear\""));
        assert!(line.contains("bezel=glass"));
        assert!(line.contains("raw=16"));
    }

    #[test]
    fn native_glass_control_accessibility_line_records_label() {
        let control = NativeGlassControlDescriptor {
            id: LiveId(13),
            rect: Rect {
                pos: Vec2d { x: 10.0, y: 20.0 },
                size: Vec2d { x: 40.0, y: 24.0 },
            },
            kind: NativeGlassControlKind::Button {
                role: NativeGlassButtonRole::Default,
            },
            label: "Clear".to_string(),
            style: NativeGlassStyle::Clear,
            tint: None,
            z_order: 0,
            enabled: true,
            visible: true,
        };

        let line = MacosWindow::native_glass_control_accessibility_line(&control);

        assert!(line.contains("accessibility-label"));
        assert!(line.contains("control=000000000000000d"));
        assert!(line.contains("label=\"Clear\""));
    }

    #[test]
    fn native_glass_control_cg_event_probe_mode_parses_process_prefix() {
        let control = NativeGlassControlDescriptor {
            id: LiveId(11),
            rect: Rect {
                pos: Vec2d { x: 10.0, y: 20.0 },
                size: Vec2d { x: 40.0, y: 24.0 },
            },
            kind: NativeGlassControlKind::Button {
                role: NativeGlassButtonRole::Default,
            },
            label: "Clear".to_string(),
            style: NativeGlassStyle::Clear,
            tint: None,
            z_order: 0,
            enabled: true,
            visible: true,
        };

        assert_eq!(
            MacosWindow::native_glass_control_cg_event_probe_mode_value(Some("Clear"), &control),
            Some(NativeGlassCgEventProbeMode::Global)
        );
        assert_eq!(
            MacosWindow::native_glass_control_cg_event_probe_mode_value(
                Some("global:Clear"),
                &control
            ),
            Some(NativeGlassCgEventProbeMode::Global)
        );
        assert_eq!(
            MacosWindow::native_glass_control_cg_event_probe_mode_value(
                Some("pid:Clear"),
                &control
            ),
            Some(NativeGlassCgEventProbeMode::Process)
        );
        assert_eq!(
            MacosWindow::native_glass_control_cg_event_probe_mode_value(
                Some("process:clear"),
                &control
            ),
            Some(NativeGlassCgEventProbeMode::Process)
        );
        assert_eq!(
            MacosWindow::native_glass_control_cg_event_probe_mode_value(Some("pid:Send"), &control),
            None
        );
    }

    #[test]
    fn native_glass_cg_event_point_flips_appkit_screen_y() {
        let point = MacosWindow::native_glass_cg_event_point_for_display_height(
            NSPoint { x: 320.0, y: 240.0 },
            1080.0,
        );

        assert_eq!(point.x, 320.0);
        assert_eq!(point.y, 840.0);
    }

    #[test]
    fn native_glass_geometry_snapshot_env_accepts_truthy_values() {
        assert!(MacosWindow::native_glass_geometry_snapshot_enabled_from_value(Some("1")));
        assert!(MacosWindow::native_glass_geometry_snapshot_enabled_from_value(Some("true")));
        assert!(
            MacosWindow::native_glass_geometry_snapshot_enabled_from_value(Some("self-resize"))
        );
        assert!(!MacosWindow::native_glass_geometry_snapshot_enabled_from_value(None));
        assert!(!MacosWindow::native_glass_geometry_snapshot_enabled_from_value(Some("off")));
    }

    #[test]
    fn native_glass_control_validation_reason_maps_errors() {
        assert_eq!(
            MacosWindow::native_glass_control_validation_reason(
                NativeGlassControlBatchValidationError::TooManyVisibleControls {
                    count: 25,
                    max: 24,
                }
            ),
            "too-many-visible-controls"
        );
        assert_eq!(
            MacosWindow::native_glass_control_validation_reason(
                NativeGlassControlBatchValidationError::EmptyVisibleControlRect {
                    control_id: LiveId(7),
                }
            ),
            "empty-visible-control-rect"
        );
    }

    #[test]
    fn native_glass_window_geometry_changed_detects_position_size_and_dpi() {
        let base = WindowGeom {
            dpi_factor: 2.0,
            position: Vec2d { x: 10.0, y: 20.0 },
            inner_size: Vec2d { x: 900.0, y: 700.0 },
            ..WindowGeom::default()
        };
        let mut moved = base.clone();
        moved.position.x += 10.0;
        let mut resized = base.clone();
        resized.inner_size.y += 24.0;
        let mut dpi_changed = base.clone();
        dpi_changed.dpi_factor = 1.0;
        let mut dpi_jitter = base.clone();
        dpi_jitter.dpi_factor = 2.0005;

        assert!(!MacosWindow::native_glass_window_geometry_changed(
            &base, &base
        ));
        assert!(MacosWindow::native_glass_window_geometry_changed(
            &base, &moved
        ));
        assert!(MacosWindow::native_glass_window_geometry_changed(
            &base, &resized
        ));
        assert!(MacosWindow::native_glass_window_geometry_changed(
            &base,
            &dpi_changed
        ));
        assert!(!MacosWindow::native_glass_window_geometry_changed(
            &base,
            &dpi_jitter
        ));
    }

    #[test]
    fn native_glass_batch_equivalent_tolerates_subpixel_jitter() {
        let mut a = NativeGlassBatch {
            window_id: WindowId(0, 0),
            containers: vec![NativeGlassContainerDescriptor {
                id: LiveId(1),
                rect: Rect {
                    pos: Vec2d { x: 0.0, y: 0.0 },
                    size: Vec2d { x: 900.0, y: 700.0 },
                },
                spacing: 20.0,
                panels: vec![NativeGlassPanelDescriptor {
                    id: LiveId(2),
                    rect: Rect {
                        pos: Vec2d { x: 10.0, y: 20.0 },
                        size: Vec2d { x: 300.0, y: 200.0 },
                    },
                    shape: crate::event::NativeGlassShape::RoundedRect { radius: 24.0 },
                    style: NativeGlassStyle::Clear,
                    tint: Some(Vec4f {
                        x: 0.1,
                        y: 0.2,
                        z: 0.3,
                        w: 0.4,
                    }),
                    hit_test: NativeGlassHitTest::Passthrough,
                    z_order: 1,
                    visible: true,
                }],
            }],
        };
        let mut b = a.clone();
        b.containers[0].rect.size.x += 0.25;
        b.containers[0].panels[0].rect.pos.x += 0.25;
        b.containers[0].panels[0].rect.size.y -= 0.25;

        assert!(a.equivalent_for_native_update(&b));

        a.containers[0].panels[0].rect.size.x += 2.0;
        assert!(!a.equivalent_for_native_update(&b));
    }

    #[test]
    fn native_glass_batch_equivalent_detects_spacing_change() {
        let a = NativeGlassBatch {
            window_id: WindowId(0, 0),
            containers: vec![NativeGlassContainerDescriptor {
                id: LiveId(1),
                rect: Rect {
                    pos: Vec2d { x: 0.0, y: 0.0 },
                    size: Vec2d { x: 900.0, y: 700.0 },
                },
                spacing: 20.0,
                panels: Vec::new(),
            }],
        };
        let mut b = a.clone();
        b.containers[0].spacing = 28.0;

        assert!(!a.equivalent_for_native_update(&b));
    }

    #[test]
    fn above_metal_probe_frame_insets_inside_window_bounds() {
        let bounds = NSRect {
            origin: NSPoint { x: 0.0, y: 0.0 },
            size: NSSize {
                width: 900.0,
                height: 700.0,
            },
        };

        let frame = MacosWindow::above_metal_probe_frame(bounds);

        assert!(frame.origin.x > 0.0);
        assert!(frame.origin.y > 0.0);
        assert!(frame.size.width < bounds.size.width);
        assert!(frame.size.height < bounds.size.height);
        assert!(frame.size.width > bounds.size.width * 0.75);
        assert!(frame.size.height > bounds.size.height * 0.75);
    }
}
