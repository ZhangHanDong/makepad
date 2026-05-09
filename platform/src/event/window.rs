use {
    crate::{makepad_live_id::LiveId, makepad_math::*, window::WindowId}, //makepad_microserde::*,
    std::cell::Cell,
    std::rc::Rc,
};

/// Safe area insets describing regions of the screen that should not contain
/// interactive content (e.g., notch/Dynamic Island, home indicator, rounded corners).
/// Values are in logical points (not physical pixels).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SafeAreaInsets {
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct WindowGeom {
    pub dpi_factor: f64,
    pub can_fullscreen: bool,
    pub xr_is_presenting: bool,
    pub is_fullscreen: bool,
    pub is_topmost: bool,
    pub position: Vec2d,
    pub inner_size: Vec2d,
    pub outer_size: Vec2d,
    /// Safe area insets for this window (non-zero on devices with notches,
    /// rounded corners, home indicators, etc.)
    pub safe_area_insets: SafeAreaInsets,
    /// Bounding box of the window-chrome buttons drawn by this window, in logical
    /// pixels with a top-left origin at the top-left corner of the content view
    /// (Y increases downward — Makepad's standard coordinate system).
    ///
    /// **Per-platform values:**
    /// - **macOS** — bounding box of the three traffic-light buttons (close /
    ///   miniaturize / zoom), queried live from the OS via `standardWindowButton:`.
    ///   Buttons sit on the left side of the title bar.
    /// - **Windows** — bounding box of the three Makepad-drawn caption buttons
    ///   (minimize / maximize / close), each 46 × 29 logical px, right-aligned at
    ///   the top of the caption bar.
    /// - **Linux / Wayland with `custom_window_chrome`** — same button layout as
    ///   Windows (right-aligned, 138 × 29 logical px).
    /// - **All other platforms** (X11 with WM decorations, LinuxDirect, Android,
    ///   iOS, Web, …) — zero rect, because the platform either provides its own
    ///   chrome outside the content area or has no title bar at all.
    ///
    /// **How to use this:**
    /// When drawing custom content inside a caption bar (e.g., a title label,
    /// search field, or toolbar), use this rect to determine which region is
    /// occupied by chrome buttons so you can apply the necessary margins and avoid
    /// overdrawing the buttons.  On macOS the occupied region is on the left; on
    /// Windows / Wayland it is on the right.  A zero rect means there are no
    /// chrome buttons to avoid.
    pub window_chrome_buttons: Rect,
}

#[derive(Clone, Debug)]
pub struct WindowGeomChangeEvent {
    pub window_id: WindowId,
    pub old_geom: WindowGeom,
    pub new_geom: WindowGeom,
}

#[derive(Clone, Debug)]
pub struct WindowMovedEvent {
    pub window_id: WindowId,
    pub old_pos: Vec2d,
    pub new_pos: Vec2d,
}

#[derive(Clone, Debug)]
pub struct WindowCloseRequestedEvent {
    pub window_id: WindowId,
    pub accept_close: Rc<Cell<bool>>,
}

#[derive(Clone, Debug)]
pub struct WindowClosedEvent {
    pub window_id: WindowId,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowNativeSubstrateStyle {
    MacosGlassRegular,
    MacosGlassClear,
    IosGlassRegular,
    IosGlassClear,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowNativeSubstrateState {
    ClassMissing,
    PreflightFailed,
    VisibilityUnverified,
    Installed,
}

#[derive(Clone, Debug)]
pub struct WindowNativeSubstrateResolvedEvent {
    pub window_id: WindowId,
    pub state: WindowNativeSubstrateState,
    pub style: Option<WindowNativeSubstrateStyle>,
    pub reason: &'static str,
}

pub const NATIVE_GLASS_MAX_PANELS_PER_WINDOW_V4_1: usize = 12;
pub const NATIVE_GLASS_MAX_CONTROLS_PER_WINDOW_V4_10: usize = 24;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NativeGlassShape {
    RoundedRect { radius: f64 },
    Capsule,
}

impl NativeGlassShape {
    pub fn corner_radius_for_rect(self, rect: Rect) -> f64 {
        match self {
            Self::RoundedRect { radius } => radius,
            Self::Capsule => rect.size.x.min(rect.size.y) * 0.5,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeGlassStyle {
    Regular,
    Clear,
}

impl NativeGlassStyle {
    pub fn macos_raw_value(self) -> i64 {
        match self {
            Self::Regular => 0,
            Self::Clear => 1,
        }
    }

    fn default_ios_raw_value(self) -> i64 {
        match self {
            Self::Regular => 0,
            Self::Clear => 1,
        }
    }

    fn ios_style_override_env_var(self) -> &'static str {
        match self {
            Self::Regular => "AICHAT_IOS_GLASS_STYLE_REGULAR_RAW",
            Self::Clear => "AICHAT_IOS_GLASS_STYLE_CLEAR_RAW",
        }
    }

    pub fn ios_raw_value_from_override(self, override_value: Option<&str>) -> i64 {
        override_value
            .and_then(|value| value.trim().parse::<i64>().ok())
            .unwrap_or_else(|| self.default_ios_raw_value())
    }

    pub fn ios_raw_value(self) -> i64 {
        match std::env::var(self.ios_style_override_env_var()) {
            Ok(value) => self.ios_raw_value_from_override(Some(&value)),
            Err(_) => self.default_ios_raw_value(),
        }
    }

    pub fn ios_event_style(self) -> WindowNativeSubstrateStyle {
        match self {
            Self::Regular => WindowNativeSubstrateStyle::IosGlassRegular,
            Self::Clear => WindowNativeSubstrateStyle::IosGlassClear,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeGlassHitTest {
    Passthrough,
    Interactive,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeGlassButtonRole {
    Default,
    Primary,
    Icon,
    Nav,
    Utility,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeGlassControlKind {
    Button { role: NativeGlassButtonRole },
}

#[derive(Clone, Debug, PartialEq)]
pub struct NativeGlassControlDescriptor {
    pub id: LiveId,
    pub rect: Rect,
    pub kind: NativeGlassControlKind,
    pub style: NativeGlassStyle,
    pub tint: Option<Vec4f>,
    pub z_order: i32,
    pub enabled: bool,
    pub visible: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NativeGlassControlBatch {
    pub window_id: WindowId,
    pub controls: Vec<NativeGlassControlDescriptor>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NativeGlassPanelDescriptor {
    pub id: LiveId,
    pub rect: Rect,
    pub shape: NativeGlassShape,
    pub style: NativeGlassStyle,
    pub tint: Option<Vec4f>,
    pub hit_test: NativeGlassHitTest,
    pub z_order: i32,
    pub visible: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NativeGlassContainerDescriptor {
    pub id: LiveId,
    pub rect: Rect,
    pub spacing: f64,
    pub panels: Vec<NativeGlassPanelDescriptor>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NativeGlassBatch {
    pub window_id: WindowId,
    pub containers: Vec<NativeGlassContainerDescriptor>,
}

const NATIVE_GLASS_UPDATE_RECT_EPSILON: f64 = 0.5;
const NATIVE_GLASS_UPDATE_TINT_EPSILON: f64 = 0.001;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeGlassBatchValidationError {
    TooManyContainers {
        count: usize,
    },
    TooManyVisiblePanels {
        count: usize,
        max: usize,
    },
    InteractiveHitTestUnsupported {
        container_id: LiveId,
        panel_id: LiveId,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeGlassControlBatchValidationError {
    TooManyVisibleControls { count: usize, max: usize },
    EmptyVisibleControlRect { control_id: LiveId },
}

impl NativeGlassBatch {
    pub fn visible_panel_count(&self) -> usize {
        self.containers
            .iter()
            .flat_map(|container| container.panels.iter())
            .filter(|panel| panel.visible)
            .count()
    }

    pub fn validate_v4_1(&self) -> Result<(), NativeGlassBatchValidationError> {
        if self.containers.len() > 1 {
            return Err(NativeGlassBatchValidationError::TooManyContainers {
                count: self.containers.len(),
            });
        }

        let visible_panel_count = self.visible_panel_count();
        if visible_panel_count > NATIVE_GLASS_MAX_PANELS_PER_WINDOW_V4_1 {
            return Err(NativeGlassBatchValidationError::TooManyVisiblePanels {
                count: visible_panel_count,
                max: NATIVE_GLASS_MAX_PANELS_PER_WINDOW_V4_1,
            });
        }

        for container in &self.containers {
            for panel in &container.panels {
                if panel.hit_test == NativeGlassHitTest::Interactive {
                    return Err(
                        NativeGlassBatchValidationError::InteractiveHitTestUnsupported {
                            container_id: container.id,
                            panel_id: panel.id,
                        },
                    );
                }
            }
        }

        Ok(())
    }

    fn native_float_equivalent(a: f64, b: f64) -> bool {
        (a - b).abs() <= NATIVE_GLASS_UPDATE_RECT_EPSILON
    }

    fn native_rect_equivalent(a: Rect, b: Rect) -> bool {
        Self::native_float_equivalent(a.pos.x, b.pos.x)
            && Self::native_float_equivalent(a.pos.y, b.pos.y)
            && Self::native_float_equivalent(a.size.x, b.size.x)
            && Self::native_float_equivalent(a.size.y, b.size.y)
    }

    fn native_shape_equivalent(a: NativeGlassShape, b: NativeGlassShape) -> bool {
        match (a, b) {
            (
                NativeGlassShape::RoundedRect { radius: a },
                NativeGlassShape::RoundedRect { radius: b },
            ) => Self::native_float_equivalent(a, b),
            (NativeGlassShape::Capsule, NativeGlassShape::Capsule) => true,
            _ => false,
        }
    }

    fn native_tint_equivalent(a: Option<Vec4f>, b: Option<Vec4f>) -> bool {
        match (a, b) {
            (Some(a), Some(b)) => {
                (a.x - b.x).abs() <= NATIVE_GLASS_UPDATE_TINT_EPSILON as f32
                    && (a.y - b.y).abs() <= NATIVE_GLASS_UPDATE_TINT_EPSILON as f32
                    && (a.z - b.z).abs() <= NATIVE_GLASS_UPDATE_TINT_EPSILON as f32
                    && (a.w - b.w).abs() <= NATIVE_GLASS_UPDATE_TINT_EPSILON as f32
            }
            (None, None) => true,
            _ => false,
        }
    }

    fn native_panel_equivalent(
        a: &NativeGlassPanelDescriptor,
        b: &NativeGlassPanelDescriptor,
    ) -> bool {
        a.id == b.id
            && Self::native_rect_equivalent(a.rect, b.rect)
            && Self::native_shape_equivalent(a.shape, b.shape)
            && a.style == b.style
            && Self::native_tint_equivalent(a.tint, b.tint)
            && a.hit_test == b.hit_test
            && a.z_order == b.z_order
            && a.visible == b.visible
    }

    pub fn equivalent_for_native_update(&self, other: &Self) -> bool {
        self.window_id == other.window_id
            && self.containers.len() == other.containers.len()
            && self.containers.iter().zip(&other.containers).all(|(a, b)| {
                a.id == b.id
                    && Self::native_rect_equivalent(a.rect, b.rect)
                    && Self::native_float_equivalent(a.spacing, b.spacing)
                    && a.panels.len() == b.panels.len()
                    && a.panels
                        .iter()
                        .zip(&b.panels)
                        .all(|(a, b)| Self::native_panel_equivalent(a, b))
            })
    }
}

impl NativeGlassControlBatch {
    pub fn visible_control_count(&self) -> usize {
        self.controls
            .iter()
            .filter(|control| control.visible)
            .count()
    }

    pub fn validate_v4_10(&self) -> Result<(), NativeGlassControlBatchValidationError> {
        let visible_control_count = self.visible_control_count();
        if visible_control_count > NATIVE_GLASS_MAX_CONTROLS_PER_WINDOW_V4_10 {
            return Err(
                NativeGlassControlBatchValidationError::TooManyVisibleControls {
                    count: visible_control_count,
                    max: NATIVE_GLASS_MAX_CONTROLS_PER_WINDOW_V4_10,
                },
            );
        }

        for control in &self.controls {
            if control.visible && (control.rect.size.x <= 0.0 || control.rect.size.y <= 0.0) {
                return Err(
                    NativeGlassControlBatchValidationError::EmptyVisibleControlRect {
                        control_id: control.id,
                    },
                );
            }
        }

        Ok(())
    }

    pub fn equivalent_for_native_update(&self, other: &Self) -> bool {
        self.window_id == other.window_id
            && self.controls.len() == other.controls.len()
            && self.controls.iter().zip(&other.controls).all(|(a, b)| {
                a.id == b.id
                    && NativeGlassBatch::native_rect_equivalent(a.rect, b.rect)
                    && a.kind == b.kind
                    && a.style == b.style
                    && NativeGlassBatch::native_tint_equivalent(a.tint, b.tint)
                    && a.z_order == b.z_order
                    && a.enabled == b.enabled
                    && a.visible == b.visible
            })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeGlassBackendState {
    Unsupported,
    Rejected,
    Installed,
    Partial,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeGlassInstallState {
    Skipped,
    Rejected,
    Failed,
    Partial,
    Installed,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NativeGlassBatchResult {
    pub window_id: WindowId,
    pub backend_state: NativeGlassBackendState,
    pub containers: Vec<NativeGlassContainerResult>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NativeGlassContainerResult {
    pub id: LiveId,
    pub state: NativeGlassInstallState,
    pub reason: &'static str,
    pub installed_panels: usize,
    pub failed_panels: usize,
    pub panels: Vec<NativeGlassPanelResult>,
}

impl NativeGlassContainerResult {
    pub fn from_panel_results(
        id: LiveId,
        state: NativeGlassInstallState,
        reason: &'static str,
        panels: Vec<NativeGlassPanelResult>,
    ) -> Self {
        let installed_panels = panels
            .iter()
            .filter(|panel| panel.state == NativeGlassInstallState::Installed)
            .count();
        let failed_panels = panels
            .iter()
            .filter(|panel| {
                matches!(
                    panel.state,
                    NativeGlassInstallState::Failed | NativeGlassInstallState::Rejected
                )
            })
            .count();

        Self {
            id,
            state,
            reason,
            installed_panels,
            failed_panels,
            panels,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NativeGlassPanelResult {
    pub id: LiveId,
    pub state: NativeGlassInstallState,
    pub reason: &'static str,
}

#[derive(Clone, Debug)]
pub enum PopupDismissReason {
    FocusLost,
    OutsideClick,
    Escape,
    Compositor,
    ParentClosed,
}

/// Notification that a popup window should be closed.
///
/// The app **must** call `WindowHandle::close()` to actually close the popup.
/// The framework does not auto-close popup windows on dismissal.
///
/// On Wayland the compositor may force-close the surface (`PopupDone`); in
/// that case `PopupDismissed` fires after the surface is already gone.
///
/// Common reasons: `OutsideClick`, `FocusLost`, `Escape`, `Compositor`,
/// `ParentClosed`.
#[derive(Clone, Debug)]
pub struct PopupDismissedEvent {
    pub window_id: WindowId,
    pub reason: PopupDismissReason,
}
/*
#[derive(Clone, Debug)]
pub struct WindowResizeLoopEvent {
    pub was_started: bool,
    pub window_id: WindowId
}*/

#[derive(Clone, Debug, Copy)]
pub enum WindowDragQueryResponse {
    NoAnswer,
    Client,
    Caption,
    SysMenu, // windows only
}

#[derive(Clone, Debug)]
pub struct WindowDragQueryEvent {
    pub window_id: WindowId,
    pub abs: Vec2d,
    pub response: Rc<Cell<WindowDragQueryResponse>>,
}

#[cfg(test)]
mod native_glass_tests {
    use super::*;
    use crate::makepad_live_id::LiveId;

    fn panel(id: u64) -> NativeGlassPanelDescriptor {
        NativeGlassPanelDescriptor {
            id: LiveId(id),
            rect: Rect {
                pos: Vec2d { x: 0.0, y: 0.0 },
                size: Vec2d { x: 100.0, y: 48.0 },
            },
            shape: NativeGlassShape::RoundedRect { radius: 12.0 },
            style: NativeGlassStyle::Regular,
            tint: None,
            hit_test: NativeGlassHitTest::Passthrough,
            z_order: id as i32,
            visible: true,
        }
    }

    fn batch_with_panels(panels: Vec<NativeGlassPanelDescriptor>) -> NativeGlassBatch {
        NativeGlassBatch {
            window_id: WindowId(1, 1),
            containers: vec![NativeGlassContainerDescriptor {
                id: LiveId(10),
                rect: Rect {
                    pos: Vec2d { x: 0.0, y: 0.0 },
                    size: Vec2d { x: 800.0, y: 600.0 },
                },
                spacing: 20.0,
                panels,
            }],
        }
    }

    fn control(id: u64) -> NativeGlassControlDescriptor {
        NativeGlassControlDescriptor {
            id: LiveId(id),
            rect: Rect {
                pos: Vec2d { x: 8.0, y: 8.0 },
                size: Vec2d { x: 96.0, y: 36.0 },
            },
            kind: NativeGlassControlKind::Button {
                role: NativeGlassButtonRole::Default,
            },
            style: NativeGlassStyle::Clear,
            tint: None,
            z_order: id as i32,
            enabled: true,
            visible: true,
        }
    }

    fn control_batch(controls: Vec<NativeGlassControlDescriptor>) -> NativeGlassControlBatch {
        NativeGlassControlBatch {
            window_id: WindowId(1, 1),
            controls,
        }
    }

    #[test]
    fn native_glass_batch_accepts_single_container_with_twelve_visible_panels() {
        let panels = (0..NATIVE_GLASS_MAX_PANELS_PER_WINDOW_V4_1 as u64)
            .map(panel)
            .collect();
        let batch = batch_with_panels(panels);

        assert_eq!(batch.visible_panel_count(), 12);
        assert_eq!(batch.validate_v4_1(), Ok(()));
    }

    #[test]
    fn native_glass_batch_rejects_multiple_containers_in_v4_1() {
        let mut batch = batch_with_panels(vec![panel(1)]);
        batch.containers.push(NativeGlassContainerDescriptor {
            id: LiveId(11),
            rect: Rect::default(),
            spacing: 0.0,
            panels: vec![panel(2)],
        });

        assert_eq!(
            batch.validate_v4_1(),
            Err(NativeGlassBatchValidationError::TooManyContainers { count: 2 })
        );
    }

    #[test]
    fn native_glass_batch_rejects_more_than_twelve_visible_panels() {
        let panels = (0..=NATIVE_GLASS_MAX_PANELS_PER_WINDOW_V4_1 as u64)
            .map(panel)
            .collect();
        let batch = batch_with_panels(panels);

        assert_eq!(
            batch.validate_v4_1(),
            Err(NativeGlassBatchValidationError::TooManyVisiblePanels {
                count: 13,
                max: NATIVE_GLASS_MAX_PANELS_PER_WINDOW_V4_1,
            })
        );
    }

    #[test]
    fn native_glass_batch_rejects_interactive_hit_test_in_v4_1() {
        let mut interactive_panel = panel(99);
        interactive_panel.hit_test = NativeGlassHitTest::Interactive;
        let batch = batch_with_panels(vec![interactive_panel]);

        assert_eq!(
            batch.validate_v4_1(),
            Err(
                NativeGlassBatchValidationError::InteractiveHitTestUnsupported {
                    container_id: LiveId(10),
                    panel_id: LiveId(99),
                }
            )
        );
    }

    #[test]
    fn native_glass_container_result_counts_installed_and_failed_panels() {
        let result = NativeGlassContainerResult::from_panel_results(
            LiveId(10),
            NativeGlassInstallState::Partial,
            "partial",
            vec![
                NativeGlassPanelResult {
                    id: LiveId(1),
                    state: NativeGlassInstallState::Installed,
                    reason: "installed",
                },
                NativeGlassPanelResult {
                    id: LiveId(2),
                    state: NativeGlassInstallState::Failed,
                    reason: "selector missing",
                },
                NativeGlassPanelResult {
                    id: LiveId(3),
                    state: NativeGlassInstallState::Rejected,
                    reason: "interactive unsupported",
                },
            ],
        );

        assert_eq!(result.installed_panels, 1);
        assert_eq!(result.failed_panels, 2);
    }

    #[test]
    fn native_glass_style_maps_to_macos_raw_values() {
        assert_eq!(NativeGlassStyle::Regular.macos_raw_value(), 0);
        assert_eq!(NativeGlassStyle::Clear.macos_raw_value(), 1);
    }

    #[test]
    fn native_glass_style_maps_to_ios_event_styles() {
        assert_eq!(
            NativeGlassStyle::Regular.ios_event_style(),
            WindowNativeSubstrateStyle::IosGlassRegular
        );
        assert_eq!(
            NativeGlassStyle::Clear.ios_event_style(),
            WindowNativeSubstrateStyle::IosGlassClear
        );
    }

    #[test]
    fn native_glass_style_maps_to_ios_default_raw_values() {
        assert_eq!(
            NativeGlassStyle::Regular.ios_raw_value_from_override(None),
            0
        );
        assert_eq!(NativeGlassStyle::Clear.ios_raw_value_from_override(None), 1);
    }

    #[test]
    fn native_glass_style_ios_raw_value_accepts_override() {
        assert_eq!(
            NativeGlassStyle::Regular.ios_raw_value_from_override(Some("7")),
            7
        );
        assert_eq!(
            NativeGlassStyle::Clear.ios_raw_value_from_override(Some("8")),
            8
        );
    }

    #[test]
    fn native_glass_style_ios_raw_value_rejects_invalid_override() {
        assert_eq!(
            NativeGlassStyle::Regular.ios_raw_value_from_override(Some("nope")),
            0
        );
        assert_eq!(
            NativeGlassStyle::Clear.ios_raw_value_from_override(Some("")),
            1
        );
    }

    #[test]
    fn native_glass_capsule_corner_radius_uses_half_shortest_side() {
        let rect = Rect {
            pos: Vec2d { x: 0.0, y: 0.0 },
            size: Vec2d { x: 100.0, y: 48.0 },
        };

        assert_eq!(NativeGlassShape::Capsule.corner_radius_for_rect(rect), 24.0);
    }

    #[test]
    fn native_glass_batch_equivalent_for_native_update_tolerates_subpixel_jitter() {
        let mut a = batch_with_panels(vec![panel(1)]);
        let mut b = a.clone();
        b.containers[0].rect.pos.x += 0.25;
        b.containers[0].panels[0].rect.size.y += 0.25;

        assert!(a.equivalent_for_native_update(&b));

        a.containers[0].panels[0].rect.size.y += 1.0;
        assert!(!a.equivalent_for_native_update(&b));
    }

    #[test]
    fn native_glass_batch_equivalent_for_native_update_detects_semantic_changes() {
        let a = batch_with_panels(vec![panel(1)]);

        let mut spacing = a.clone();
        spacing.containers[0].spacing += 1.0;
        assert!(!a.equivalent_for_native_update(&spacing));

        let mut style = a.clone();
        style.containers[0].panels[0].style = NativeGlassStyle::Clear;
        assert!(!a.equivalent_for_native_update(&style));

        let mut tint = a.clone();
        tint.containers[0].panels[0].tint = Some(Vec4f {
            x: 1.0,
            y: 0.0,
            z: 0.0,
            w: 0.5,
        });
        assert!(!a.equivalent_for_native_update(&tint));

        let mut tint_delta = a.clone();
        tint_delta.containers[0].panels[0].tint = Some(Vec4f {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 0.01,
        });
        let mut tinted = a.clone();
        tinted.containers[0].panels[0].tint = Some(Vec4f {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 0.02,
        });
        assert!(!tint_delta.equivalent_for_native_update(&tinted));

        let mut hit_test = a.clone();
        hit_test.containers[0].panels[0].hit_test = NativeGlassHitTest::Interactive;
        assert!(!a.equivalent_for_native_update(&hit_test));

        let mut z_order = a.clone();
        z_order.containers[0].panels[0].z_order += 1;
        assert!(!a.equivalent_for_native_update(&z_order));

        let mut visible = a.clone();
        visible.containers[0].panels[0].visible = false;
        assert!(!a.equivalent_for_native_update(&visible));
    }

    #[test]
    fn native_glass_control_batch_accepts_max_visible_controls() {
        let controls = (0..NATIVE_GLASS_MAX_CONTROLS_PER_WINDOW_V4_10 as u64)
            .map(control)
            .collect();
        let batch = control_batch(controls);

        assert_eq!(batch.visible_control_count(), 24);
        assert_eq!(batch.validate_v4_10(), Ok(()));
    }

    #[test]
    fn native_glass_control_batch_rejects_too_many_visible_controls() {
        let controls = (0..=NATIVE_GLASS_MAX_CONTROLS_PER_WINDOW_V4_10 as u64)
            .map(control)
            .collect();
        let batch = control_batch(controls);

        assert_eq!(
            batch.validate_v4_10(),
            Err(
                NativeGlassControlBatchValidationError::TooManyVisibleControls {
                    count: 25,
                    max: NATIVE_GLASS_MAX_CONTROLS_PER_WINDOW_V4_10,
                }
            )
        );
    }

    #[test]
    fn native_glass_control_batch_rejects_empty_visible_control_rect() {
        let mut empty = control(7);
        empty.rect.size.x = 0.0;
        let batch = control_batch(vec![empty]);

        assert_eq!(
            batch.validate_v4_10(),
            Err(
                NativeGlassControlBatchValidationError::EmptyVisibleControlRect {
                    control_id: LiveId(7),
                }
            )
        );
    }

    #[test]
    fn native_glass_control_batch_allows_empty_hidden_control_rect() {
        let mut hidden = control(8);
        hidden.visible = false;
        hidden.rect.size.x = 0.0;
        let batch = control_batch(vec![hidden]);

        assert_eq!(batch.visible_control_count(), 0);
        assert_eq!(batch.validate_v4_10(), Ok(()));
    }

    #[test]
    fn native_glass_control_batch_equivalent_for_native_update_detects_changes() {
        let a = control_batch(vec![control(1)]);

        let mut jitter = a.clone();
        jitter.controls[0].rect.pos.x += 0.25;
        assert!(a.equivalent_for_native_update(&jitter));

        let mut role = a.clone();
        role.controls[0].kind = NativeGlassControlKind::Button {
            role: NativeGlassButtonRole::Primary,
        };
        assert!(!a.equivalent_for_native_update(&role));

        let mut enabled = a.clone();
        enabled.controls[0].enabled = false;
        assert!(!a.equivalent_for_native_update(&enabled));

        let mut visible = a.clone();
        visible.controls[0].visible = false;
        assert!(!a.equivalent_for_native_update(&visible));
    }
}
