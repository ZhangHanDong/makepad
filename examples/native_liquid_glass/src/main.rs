pub use makepad_widgets;

use makepad_widgets::button::ButtonNativeGlassRole;
use makepad_widgets::*;

app_main!(App);

const DEFAULT_CLEAR_TINT_ALPHA: f32 = 0.22;
const DEFAULT_CONTAINER_SPACING: f64 = 28.0;
const DEFAULT_MAIN_RADIUS: f64 = 44.0;
const MIN_TINT_ALPHA: f32 = 0.06;
const MAX_TINT_ALPHA: f32 = 0.36;
const TINT_ALPHA_STEP: f32 = 0.04;
const MIN_CONTAINER_SPACING: f64 = 12.0;
const MAX_CONTAINER_SPACING: f64 = 52.0;
const CONTAINER_SPACING_STEP: f64 = 4.0;
const MIN_MAIN_RADIUS: f64 = 24.0;
const MAX_MAIN_RADIUS: f64 = 72.0;
const MAIN_RADIUS_STEP: f64 = 4.0;
const NATIVE_PRESS_PULSE_DURATION: f64 = 3.0;
const NATIVE_PRESS_PULSE_ATTACK: f64 = 0.55;
const NATIVE_PRESS_PULSE_HOLD: f64 = 0.35;
const NATIVE_PRESS_PULSE_SPACING_BOOST: f64 = 20.0;
const NATIVE_PRESS_PULSE_RADIUS_BOOST: f64 = 16.0;
const NATIVE_PRESS_PULSE_TINT_BOOST: f32 = 0.10;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*
    use mod.draw.*

    let CaptionText = Label{
        width: Fit
        height: Fit
        draw_text.color: #x202633C8
        draw_text.text_style.font_size: 11
    }

    let PanelLabel = Label{
        width: Fit
        height: Fit
        draw_text.color: #x111722E6
        draw_text.text_style: theme.font_bold{font_size: 13}
    }

    let TopPanelText = Label{
        width: Fit
        height: Fit
        draw_text.color: #xF7F8FFE8
        draw_text.text_style.font_size: 11
    }

    let TopPanelLabel = Label{
        width: Fit
        height: Fit
        draw_text.color: #xF7F8FFF0
        draw_text.text_style: theme.font_bold{font_size: 13}
    }

    let ControlButton = GlassButton{
        height: 30
        padding: Inset{left: 12 right: 12 top: 0 bottom: 0}
        draw_text +: {
            color: #xF7F8FF
            color_hover: #xFFFFFF
            color_down: #xDCE7F3
            color_focus: #xFFFFFF
            text_style +: {font_size: 11}
        }
        draw_bg +: {
            color: #x1B2636CC
            color_hover: #x24354DDD
            color_down: #x101722EE
            color_focus: #x24354DEE
            border_color: #xFFFFFF36
            border_color_hover: #xFFFFFF5C
            border_radius: 10.0
            border_size: 1.0
        }
    }

    let NativeRoleButton = GlassButton{
        width: 112
        height: 34
        padding: Inset{left: 12 right: 12 top: 0 bottom: 0}
        draw_text +: {
            color: #x101722E8
            color_hover: #x07101AF0
            color_down: #x07101AF0
            color_focus: #x07101AF0
            text_style +: {font_size: 11}
        }
        draw_bg +: {
            color: #xFFFFFF2A
            color_hover: #xFFFFFF42
            color_down: #xDCE7F35C
            color_focus: #xFFFFFF4C
            border_color: #xFFFFFF72
            border_color_hover: #xFFFFFFA0
            border_radius: 10.0
            border_size: 1.0
        }
    }

    let ControlValue = Label{
        width: Fit
        height: Fit
        draw_text.color: #x111722E6
        draw_text.text_style.font_size: 11
    }

    startup() do #(App::script_component(vm)){
        ui: Root{
            main_window := Window{
                show_caption_bar: false
                pass.clear_color: #00000000
                window.transparent: true
                window.macos: MacosWindowConfig{chrome: MacosWindowChrome.Borderless resizable: true}
                window.inner_size: vec2(820, 560)
                window.title: "Native Liquid Glass"

                body +: {
                    flow: Overlay
                    padding: 0
                    spacing: 0
                    draw_bg.color: #00000000

                    glass_container := GlassContainer{
                        width: Fill
                        height: Fill
                        native: true
                        spacing: 28.0
                        flow: Overlay

                        main_panel := GlassPanel{
                            width: Fill
                            height: Fill
                            margin: Inset{left: 28 top: 28 right: 28 bottom: 28}
                            native: true
                            native_radius: 44.0
                            native_z_order: 0.0
                            show_bg: false
                            draw_bg +: {
                                tint_alpha: 0.0
                                border_alpha: 0.0
                                corner_radius: 44.0
                            }
                        }

                        top_panel_host := View{
                            width: Fill
                            height: Fill
                            flow: Overlay
                            align: Align{x: 1.0 y: 0.0}

                            top_panel := GlassPanel{
                                width: 282
                                height: 88
                                margin: Inset{top: 54 right: 56}
                                native: true
                                native_radius: 30.0
                                native_z_order: 1.0
                                show_bg: false
                                draw_bg +: {
                                    tint_alpha: 0.0
                                    border_alpha: 0.0
                                    corner_radius: 30.0
                                }
                            }
                        }

                        bottom_pill_host := View{
                            width: Fill
                            height: Fill
                            flow: Overlay
                            align: Align{x: 0.5 y: 1.0}

                            bottom_pill := GlassPanel{
                                width: 360
                                height: 64
                                margin: Inset{bottom: 58}
                                native: true
                                native_radius: 32.0
                                native_z_order: 2.0
                                show_bg: false
                                draw_bg +: {
                                    tint_alpha: 0.0
                                    border_alpha: 0.0
                                    corner_radius: 32.0
                                }
                            }
                        }

                        native_control_host := View{
                            width: Fill
                            height: Fill
                            flow: Overlay
                            align: Align{x: 1.0 y: 1.0}
                            draw_bg.color: #00000000

                            native_control_matrix := View{
                                width: Fit
                                height: Fit
                                margin: Inset{right: 64 bottom: 88}
                                flow: Down
                                spacing: 8
                                align: Align{x: 1.0 y: 1.0}
                                draw_bg.color: #00000000

                                native_control_row_1 := View{
                                    width: Fit
                                    height: Fit
                                    flow: Right
                                    spacing: 8
                                    draw_bg.color: #00000000

                                    native_default_button := NativeRoleButton{text: "Default"}
                                    native_primary_button := NativeRoleButton{text: "Primary"}
                                }

                                native_control_row_2 := View{
                                    width: Fit
                                    height: Fit
                                    flow: Right
                                    spacing: 8
                                    draw_bg.color: #00000000

                                    native_utility_button := NativeRoleButton{text: "Utility"}
                                    native_icon_button := NativeRoleButton{text: "Icon"}
                                    native_nav_button := NativeRoleButton{text: "Nav"}
                                }
                            }
                        }
                    }

                    foreground := View{
                        width: Fill
                        height: Fill
                        flow: Down
                        padding: Inset{left: 60 top: 54 right: 60 bottom: 52}
                        spacing: 16
                        draw_bg.color: #00000000

                        title_group := View{
                            width: Fill
                            height: 96
                            flow: Down
                            spacing: 8
                            draw_bg.color: #00000000

                            caption_stack := View{
                                width: Fill
                                height: 22
                                flow: Overlay
                                draw_bg.color: #00000000

                                Label{
                                    width: Fit
                                    height: Fit
                                    margin: Inset{left: 1 top: 1}
                                    text: "Makepad Native"
                                    draw_text.color: #xFFFFFFB8
                                    draw_text.text_style.font_size: 11
                                }
                                CaptionText{text: "Makepad Native"}
                            }

                            title_stack := View{
                                width: Fill
                                height: 60
                                flow: Overlay
                                draw_bg.color: #00000000

                                Label{
                                    width: Fit
                                    height: Fit
                                    margin: Inset{left: 1 top: 1}
                                    text: "Liquid Glass"
                                    draw_text.color: #xFFFFFFB8
                                    draw_text.text_style: theme.font_bold{font_size: 38}
                                }
                                Label{
                                    width: Fit
                                    height: Fit
                                    text: "Liquid Glass"
                                    draw_text.color: #x0E1116F2
                                    draw_text.text_style: theme.font_bold{font_size: 38}
                                }
                            }
                        }

                        controls := View{
                            width: Fill
                            height: Fit
                            flow: Down
                            spacing: 8
                            draw_bg.color: #00000000

                            mode_row := View{
                                width: Fill
                                height: Fit
                                flow: Right
                                spacing: 8
                                align: Align{x: 0.0 y: 0.5}
                                draw_bg.color: #00000000

                                mode_panels_button := ControlButton{text: "Panels"}
                                mode_morph_button := ControlButton{text: "Morph"}
                                mode_controls_button := ControlButton{text: "Controls"}
                                mode_readability_button := ControlButton{text: "Readability"}
                            }

                            morph_row := View{
                                visible: false
                                width: Fill
                                height: Fit
                                flow: Right
                                spacing: 8
                                align: Align{x: 0.0 y: 0.5}
                                draw_bg.color: #00000000

                                morph_near_button := ControlButton{text: "Near"}
                                morph_far_button := ControlButton{text: "Far"}
                                morph_overlap_button := ControlButton{text: "Overlap"}
                            }

                            control_row_1 := View{
                                width: Fill
                                height: Fit
                                flow: Right
                                spacing: 8
                                align: Align{x: 0.0 y: 0.5}
                                draw_bg.color: #00000000

                                style_clear_button := ControlButton{text: "Clear"}
                                style_regular_button := ControlButton{text: "Regular"}
                                reset_button := ControlButton{text: "Reset"}
                                hide_controls_button := ControlButton{text: "Hide"}
                            }

                            control_row_2 := View{
                                width: Fill
                                height: Fit
                                flow: Right
                                spacing: 8
                                align: Align{x: 0.0 y: 0.5}
                                draw_bg.color: #00000000

                                tint_down_button := ControlButton{text: "Tint -"}
                                tint_up_button := ControlButton{text: "Tint +"}
                                spacing_down_button := ControlButton{text: "Spacing -"}
                                spacing_up_button := ControlButton{text: "Spacing +"}
                                radius_down_button := ControlButton{text: "Radius -"}
                                radius_up_button := ControlButton{text: "Radius +"}
                            }

                            status_readout := ControlValue{
                                text: "Panels  Clear  tint 22%  spacing 28  radius 44"
                            }
                        }

                        tune_button_host := View{
                            width: Fill
                            height: Fill
                            flow: Overlay
                            align: Align{x: 0.0 y: 0.0}
                            draw_bg.color: #00000000

                            tune_button := ControlButton{
                                visible: false
                                text: "Tune"
                                margin: Inset{left: 60 top: 166}
                            }
                        }

                        View{
                            width: Fill
                            height: Fill
                        }

                        bottom_copy := View{
                            width: Fill
                            height: Fit
                            flow: Right
                            spacing: 14
                            align: Align{x: 0.5 y: 0.5}

                            bottom_left_label := PanelLabel{text: "Clear"}
                            spacing_caption := CaptionText{text: "container spacing 28"}
                            bottom_right_label := PanelLabel{text: "Regular"}
                            native_control_status := CaptionText{text: "native button 0"}
                        }
                    }

                    top_panel_copy := View{
                        width: Fill
                        height: Fill
                        flow: Overlay
                        align: Align{x: 1.0 y: 0.0}

                        top_panel_copy_card := View{
                            width: 282
                            height: 88
                            margin: Inset{top: 54 right: 56}
                            flow: Down
                            spacing: 6
                            padding: Inset{left: 22 top: 18 right: 22 bottom: 16}
                            draw_bg.color: #00000000

                            top_panel_title := TopPanelLabel{text: "Native panels"}
                            top_panel_line_1 := TopPanelText{text: "NSGlassEffectView descriptors"}
                            top_panel_line_2 := TopPanelText{text: "Metal layer stays transparent"}
                        }
                    }

                    drag_strip := View{
                        width: Fill
                        height: 52
                        draw_bg.color: #00000000
                    }

                    resize_grip_host := View{
                        width: Fill
                        height: Fill
                        flow: Overlay
                        align: Align{x: 1.0 y: 1.0}

                        resize_grip := Vector{
                            width: 34
                            height: 34
                            margin: Inset{right: 18 bottom: 18}
                            viewbox: vec4(0 0 34 34)
                            Path{d: "M 18 28 L 28 18" fill: false stroke: #xFFFFFFA0 stroke_width: 1.5 stroke_linecap: "round"}
                            Path{d: "M 12 28 L 28 12" fill: false stroke: #xFFFFFF76 stroke_width: 1.2 stroke_linecap: "round"}
                            Path{d: "M 24 28 L 28 24" fill: false stroke: #xFFFFFFA0 stroke_width: 1.5 stroke_linecap: "round"}
                        }
                    }
                }
            }
        }
    }
}

#[derive(Script, ScriptHook)]
pub struct App {
    #[live]
    ui: WidgetRef,
    #[rust]
    configured: bool,
    #[rust]
    demo_mode: NativeDemoVisualMode,
    #[rust]
    morph_state: NativeDemoMorphState,
    #[rust]
    style_mode: NativeDemoStyleMode,
    #[rust]
    tint_alpha: f32,
    #[rust]
    container_spacing: f64,
    #[rust]
    main_radius: f64,
    #[rust]
    controls_visible: bool,
    #[rust]
    native_button_activations: u32,
    #[rust]
    native_press_pulse_next_frame: NextFrame,
    #[rust]
    native_press_pulse_start_time: f64,
    #[rust]
    native_press_pulse_amount: f64,
    #[rust]
    native_press_pulse_active: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum NativeDemoVisualMode {
    #[default]
    Panels,
    Morph,
    Controls,
    Readability,
}

impl NativeDemoVisualMode {
    fn label(self) -> &'static str {
        match self {
            Self::Panels => "Panels",
            Self::Morph => "Morph",
            Self::Controls => "Controls",
            Self::Readability => "Readability",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum NativeDemoMorphState {
    #[default]
    Near,
    Far,
    Overlap,
}

impl NativeDemoMorphState {
    fn label(self) -> &'static str {
        match self {
            Self::Near => "Near",
            Self::Far => "Far",
            Self::Overlap => "Overlap",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct NativeDemoInset {
    left: f64,
    top: f64,
    right: f64,
    bottom: f64,
}

impl NativeDemoInset {
    const fn all(value: f64) -> Self {
        Self {
            left: value,
            top: value,
            right: value,
            bottom: value,
        }
    }

    const fn top_right(top: f64, right: f64) -> Self {
        Self {
            left: 0.0,
            top,
            right,
            bottom: 0.0,
        }
    }

    const fn bottom(bottom: f64) -> Self {
        Self {
            left: 0.0,
            top: 0.0,
            right: 0.0,
            bottom,
        }
    }

    fn to_inset(self) -> Inset {
        Inset {
            left: self.left,
            top: self.top,
            right: self.right,
            bottom: self.bottom,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct NativeDemoVisualScene {
    main_panel_margin: NativeDemoInset,
    top_panel_width: f64,
    top_panel_height: f64,
    top_panel_margin: NativeDemoInset,
    bottom_panel_width: f64,
    bottom_panel_height: f64,
    bottom_panel_margin: NativeDemoInset,
    top_panel_title: &'static str,
    top_panel_line_1: &'static str,
    top_panel_line_2: &'static str,
    bottom_left_label: &'static str,
    bottom_right_label: &'static str,
    native_control_visible: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct NativeDemoControlRoleSpec {
    label: &'static str,
    role: ButtonNativeGlassRole,
}

fn native_liquid_glass_control_role_matrix() -> [NativeDemoControlRoleSpec; 5] {
    [
        NativeDemoControlRoleSpec {
            label: "Default",
            role: ButtonNativeGlassRole::Default,
        },
        NativeDemoControlRoleSpec {
            label: "Primary",
            role: ButtonNativeGlassRole::Primary,
        },
        NativeDemoControlRoleSpec {
            label: "Utility",
            role: ButtonNativeGlassRole::Utility,
        },
        NativeDemoControlRoleSpec {
            label: "Icon",
            role: ButtonNativeGlassRole::Icon,
        },
        NativeDemoControlRoleSpec {
            label: "Nav",
            role: ButtonNativeGlassRole::Nav,
        },
    ]
}

#[cfg(test)]
fn native_liquid_glass_visible_control_count(scene: NativeDemoVisualScene) -> usize {
    if scene.native_control_visible {
        native_liquid_glass_control_role_matrix().len()
    } else {
        0
    }
}

fn native_liquid_glass_visual_scene(
    mode: NativeDemoVisualMode,
    morph_state: NativeDemoMorphState,
) -> NativeDemoVisualScene {
    match mode {
        NativeDemoVisualMode::Panels => NativeDemoVisualScene {
            main_panel_margin: NativeDemoInset::all(28.0),
            top_panel_width: 282.0,
            top_panel_height: 88.0,
            top_panel_margin: NativeDemoInset::top_right(54.0, 56.0),
            bottom_panel_width: 360.0,
            bottom_panel_height: 64.0,
            bottom_panel_margin: NativeDemoInset::bottom(58.0),
            top_panel_title: "Native panels",
            top_panel_line_1: "NSGlassEffectView descriptors",
            top_panel_line_2: "Metal layer stays transparent",
            bottom_left_label: "Clear",
            bottom_right_label: "Regular",
            native_control_visible: false,
        },
        NativeDemoVisualMode::Morph => match morph_state {
            NativeDemoMorphState::Near => NativeDemoVisualScene {
                main_panel_margin: NativeDemoInset::all(64.0),
                top_panel_width: 300.0,
                top_panel_height: 92.0,
                top_panel_margin: NativeDemoInset::top_right(170.0, 260.0),
                bottom_panel_width: 300.0,
                bottom_panel_height: 84.0,
                bottom_panel_margin: NativeDemoInset::bottom(208.0),
                top_panel_title: "Morph: Near",
                top_panel_line_1: "edges sit inside container spacing",
                top_panel_line_2: "watch for Apple-provided merging",
                bottom_left_label: "Near",
                bottom_right_label: "Gap",
                native_control_visible: false,
            },
            NativeDemoMorphState::Far => NativeDemoVisualScene {
                main_panel_margin: NativeDemoInset::all(64.0),
                top_panel_width: 280.0,
                top_panel_height: 86.0,
                top_panel_margin: NativeDemoInset::top_right(62.0, 62.0),
                bottom_panel_width: 340.0,
                bottom_panel_height: 70.0,
                bottom_panel_margin: NativeDemoInset::bottom(70.0),
                top_panel_title: "Morph: Far",
                top_panel_line_1: "panels sit outside merge distance",
                top_panel_line_2: "container spacing should not fuse them",
                bottom_left_label: "Far",
                bottom_right_label: "Separate",
                native_control_visible: false,
            },
            NativeDemoMorphState::Overlap => NativeDemoVisualScene {
                main_panel_margin: NativeDemoInset::all(64.0),
                top_panel_width: 340.0,
                top_panel_height: 104.0,
                top_panel_margin: NativeDemoInset::top_right(202.0, 230.0),
                bottom_panel_width: 360.0,
                bottom_panel_height: 96.0,
                bottom_panel_margin: NativeDemoInset::bottom(204.0),
                top_panel_title: "Morph: Overlap",
                top_panel_line_1: "Regular and Clear panels cross",
                top_panel_line_2: "z-order and style mixing stay native",
                bottom_left_label: "Overlap",
                bottom_right_label: "Mixed",
                native_control_visible: false,
            },
        },
        NativeDemoVisualMode::Controls => NativeDemoVisualScene {
            main_panel_margin: NativeDemoInset::all(72.0),
            top_panel_width: 278.0,
            top_panel_height: 86.0,
            top_panel_margin: NativeDemoInset::top_right(62.0, 62.0),
            bottom_panel_width: 420.0,
            bottom_panel_height: 72.0,
            bottom_panel_margin: NativeDemoInset::bottom(80.0),
            top_panel_title: "Native controls",
            top_panel_line_1: "AppKit button installed above Metal",
            top_panel_line_2: "action routes back to Makepad",
            bottom_left_label: "Button",
            bottom_right_label: "Action",
            native_control_visible: true,
        },
        NativeDemoVisualMode::Readability => NativeDemoVisualScene {
            main_panel_margin: NativeDemoInset::all(44.0),
            top_panel_width: 360.0,
            top_panel_height: 120.0,
            top_panel_margin: NativeDemoInset::top_right(74.0, 68.0),
            bottom_panel_width: 480.0,
            bottom_panel_height: 76.0,
            bottom_panel_margin: NativeDemoInset::bottom(70.0),
            top_panel_title: "Readability",
            top_panel_line_1: "contrast tokens stay above native glass",
            top_panel_line_2: "bright and dark wallpaper check",
            bottom_left_label: "Bright",
            bottom_right_label: "Dark",
            native_control_visible: false,
        },
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum NativeDemoStyleMode {
    #[default]
    Clear,
    Regular,
}

impl NativeDemoStyleMode {
    fn label(self) -> &'static str {
        match self {
            Self::Clear => "Clear",
            Self::Regular => "Regular",
        }
    }

    fn style(self) -> makepad_widgets::glass_panel::GlassNativeStyle {
        match self {
            Self::Clear => makepad_widgets::glass_panel::GlassNativeStyle::Clear,
            Self::Regular => makepad_widgets::glass_panel::GlassNativeStyle::Regular,
        }
    }

    fn opposite_style(self) -> makepad_widgets::glass_panel::GlassNativeStyle {
        match self {
            Self::Clear => makepad_widgets::glass_panel::GlassNativeStyle::Regular,
            Self::Regular => makepad_widgets::glass_panel::GlassNativeStyle::Clear,
        }
    }
}

fn clamp_native_demo_tint_alpha(value: f32) -> f32 {
    value.clamp(MIN_TINT_ALPHA, MAX_TINT_ALPHA)
}

fn clamp_native_demo_spacing(value: f64) -> f64 {
    value.clamp(MIN_CONTAINER_SPACING, MAX_CONTAINER_SPACING)
}

fn clamp_native_demo_radius(value: f64) -> f64 {
    value.clamp(MIN_MAIN_RADIUS, MAX_MAIN_RADIUS)
}

fn native_liquid_glass_clear_tint_with_alpha(alpha: f32) -> Option<Vec4f> {
    Some(vec4(0.90, 0.96, 1.0, clamp_native_demo_tint_alpha(alpha)))
}

fn native_liquid_glass_regular_tint_with_alpha(alpha: f32) -> Option<Vec4f> {
    Some(vec4(1.0, 1.0, 1.0, clamp_native_demo_tint_alpha(alpha)))
}

fn native_liquid_glass_tint_for_mode(mode: NativeDemoStyleMode, alpha: f32) -> Option<Vec4f> {
    match mode {
        NativeDemoStyleMode::Clear => native_liquid_glass_clear_tint_with_alpha(alpha),
        NativeDemoStyleMode::Regular => native_liquid_glass_regular_tint_with_alpha(alpha),
    }
}

fn native_liquid_glass_status_text(
    visual_mode: NativeDemoVisualMode,
    morph_state: NativeDemoMorphState,
    mode: NativeDemoStyleMode,
    alpha: f32,
    spacing: f64,
    radius: f64,
) -> String {
    let visual_label = match visual_mode {
        NativeDemoVisualMode::Morph => format!("{} {}", visual_mode.label(), morph_state.label()),
        _ => visual_mode.label().to_string(),
    };
    format!(
        "{}  {}  tint {:.0}%  spacing {:.0}  radius {:.0}",
        visual_label,
        mode.label(),
        clamp_native_demo_tint_alpha(alpha) * 100.0,
        clamp_native_demo_spacing(spacing),
        clamp_native_demo_radius(radius)
    )
}

fn native_liquid_glass_press_pulse_amount(elapsed: f64) -> f64 {
    fn smootherstep(t: f64) -> f64 {
        let t = t.clamp(0.0, 1.0);
        t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
    }

    if elapsed <= 0.0 {
        return 0.0;
    }
    if elapsed < NATIVE_PRESS_PULSE_ATTACK {
        return smootherstep(elapsed / NATIVE_PRESS_PULSE_ATTACK);
    }
    if elapsed < NATIVE_PRESS_PULSE_ATTACK + NATIVE_PRESS_PULSE_HOLD {
        return 1.0;
    }
    if elapsed >= NATIVE_PRESS_PULSE_DURATION {
        0.0
    } else {
        let release_duration =
            NATIVE_PRESS_PULSE_DURATION - NATIVE_PRESS_PULSE_ATTACK - NATIVE_PRESS_PULSE_HOLD;
        let release_elapsed = elapsed - NATIVE_PRESS_PULSE_ATTACK - NATIVE_PRESS_PULSE_HOLD;
        1.0 - smootherstep(release_elapsed / release_duration)
    }
}

fn native_liquid_glass_press_pulse_should_continue(elapsed: f64) -> bool {
    elapsed < NATIVE_PRESS_PULSE_DURATION
}

fn native_liquid_glass_effective_tuning(
    alpha: f32,
    spacing: f64,
    radius: f64,
    pulse: f64,
) -> (f32, f64, f64) {
    let pulse = pulse.clamp(0.0, 1.0);
    (
        clamp_native_demo_tint_alpha(alpha + (pulse as f32 * NATIVE_PRESS_PULSE_TINT_BOOST)),
        clamp_native_demo_spacing(spacing + pulse * NATIVE_PRESS_PULSE_SPACING_BOOST),
        clamp_native_demo_radius(radius + pulse * NATIVE_PRESS_PULSE_RADIUS_BOOST),
    )
}

fn native_liquid_glass_control_status_text(activations: u32, pulse: f64) -> String {
    if pulse > 0.01 {
        format!("native button {} pulse {:.0}%", activations, pulse * 100.0)
    } else {
        format!("native button {}", activations)
    }
}

fn native_liquid_glass_visual_mode_tuning(
    visual_mode: NativeDemoVisualMode,
    alpha: f32,
    spacing: f64,
    radius: f64,
) -> (f32, f64, f64) {
    match visual_mode {
        NativeDemoVisualMode::Panels => (
            clamp_native_demo_tint_alpha(alpha),
            clamp_native_demo_spacing(spacing),
            clamp_native_demo_radius(radius),
        ),
        NativeDemoVisualMode::Morph => (
            clamp_native_demo_tint_alpha(alpha + 0.03),
            clamp_native_demo_spacing(spacing + 14.0),
            clamp_native_demo_radius(radius + 10.0),
        ),
        NativeDemoVisualMode::Controls => (
            clamp_native_demo_tint_alpha(alpha + 0.01),
            clamp_native_demo_spacing(spacing - 6.0),
            clamp_native_demo_radius(radius - 6.0),
        ),
        NativeDemoVisualMode::Readability => (
            clamp_native_demo_tint_alpha(alpha + 0.10),
            clamp_native_demo_spacing(spacing),
            clamp_native_demo_radius(radius),
        ),
    }
}

fn native_liquid_glass_window_visuals() -> WindowVisuals {
    WindowVisuals {
        transparent: true,
        backdrop: WindowBackdrop::None,
        backdrop_intensity: 1.0,
    }
    .normalized()
}

fn should_start_window_drag(abs: DVec2, size: DVec2) -> bool {
    const EDGE_MARGIN: f64 = 10.0;
    const DRAG_HEIGHT: f64 = 52.0;

    abs.y > EDGE_MARGIN
        && abs.y < DRAG_HEIGHT
        && abs.x > EDGE_MARGIN
        && abs.x < size.x - EDGE_MARGIN
}

impl App {
    fn ensure_default_tuning_state(&mut self) {
        if self.tint_alpha <= 0.0 {
            self.tint_alpha = DEFAULT_CLEAR_TINT_ALPHA;
        }
        if self.container_spacing <= 0.0 {
            self.container_spacing = DEFAULT_CONTAINER_SPACING;
        }
        if self.main_radius <= 0.0 {
            self.main_radius = DEFAULT_MAIN_RADIUS;
        }
        self.tint_alpha = clamp_native_demo_tint_alpha(self.tint_alpha);
        self.container_spacing = clamp_native_demo_spacing(self.container_spacing);
        self.main_radius = clamp_native_demo_radius(self.main_radius);
    }

    fn configure_native_liquid_glass(&mut self, cx: &mut Cx) {
        self.ensure_default_tuning_state();
        self.configure_window_visuals(cx);
        self.configure_native_panels(cx);
        self.configure_native_controls(cx);
        self.configure_morph_controls(cx);
        self.update_tuning_labels(cx);
        self.ui.redraw(cx);
    }

    fn configure_window_visuals(&mut self, cx: &mut Cx) {
        if !matches!(cx.os_type(), OsType::Macos) {
            return;
        }

        let window_id = CxWindowPool::id_zero();
        let visuals = native_liquid_glass_window_visuals();
        if cx.windows[window_id].window_visuals() == visuals {
            return;
        }

        cx.windows[window_id].transparent = visuals.transparent;
        cx.windows[window_id].backdrop = visuals.backdrop;
        cx.windows[window_id].backdrop_intensity = visuals.backdrop_intensity;

        if cx.windows[window_id].is_created {
            cx.push_unique_platform_op(CxOsOp::SetWindowVisuals(window_id, visuals));
        }
    }

    fn configure_native_panels(&mut self, cx: &mut Cx) {
        let rounded_rect = makepad_widgets::glass_panel::GlassNativeShape::RoundedRect;
        let capsule = makepad_widgets::glass_panel::GlassNativeShape::Capsule;
        let scene = native_liquid_glass_visual_scene(self.demo_mode, self.morph_state);
        let main_style = self.style_mode.style();
        let top_style = self.style_mode.opposite_style();
        let (mode_tint_alpha, mode_spacing, mode_radius) = native_liquid_glass_visual_mode_tuning(
            self.demo_mode,
            self.tint_alpha,
            self.container_spacing,
            self.main_radius,
        );
        let (effective_tint_alpha, effective_spacing, effective_radius) =
            native_liquid_glass_effective_tuning(
                mode_tint_alpha,
                mode_spacing,
                mode_radius,
                self.native_press_pulse_amount,
            );
        let main_tint = native_liquid_glass_tint_for_mode(self.style_mode, effective_tint_alpha);
        let top_tint = native_liquid_glass_tint_for_mode(
            match self.style_mode {
                NativeDemoStyleMode::Clear => NativeDemoStyleMode::Regular,
                NativeDemoStyleMode::Regular => NativeDemoStyleMode::Clear,
            },
            (effective_tint_alpha * 0.82).max(MIN_TINT_ALPHA),
        );
        let container_spacing = effective_spacing;
        let main_radius = effective_radius;
        let top_radius = clamp_native_demo_radius(main_radius * 0.68).min(34.0);
        let main_panel_margin = scene.main_panel_margin.to_inset();
        let top_panel_margin = scene.top_panel_margin.to_inset();
        let bottom_panel_margin = scene.bottom_panel_margin.to_inset();

        let mut glass_container = self.ui.widget(cx, ids!(glass_container));
        script_apply_eval!(cx, glass_container, {
            native: true
            spacing: #(container_spacing)
        });

        let mut main_panel = self.ui.view(cx, ids!(main_panel));
        script_apply_eval!(cx, main_panel, {
            native: true
            native_style: #(main_style)
            native_shape: #(rounded_rect)
            native_radius: #(main_radius)
            native_tint: #(main_tint)
            margin: #(main_panel_margin)
            draw_bg +: {
                corner_radius: #(main_radius)
            }
        });

        let mut top_panel = self.ui.view(cx, ids!(top_panel));
        script_apply_eval!(cx, top_panel, {
            width: #(scene.top_panel_width)
            height: #(scene.top_panel_height)
            margin: #(top_panel_margin)
            native: true
            native_style: #(top_style)
            native_shape: #(rounded_rect)
            native_radius: #(top_radius)
            native_tint: #(top_tint)
            draw_bg +: {
                corner_radius: #(top_radius)
            }
        });

        let mut bottom_pill = self.ui.view(cx, ids!(bottom_pill));
        script_apply_eval!(cx, bottom_pill, {
            width: #(scene.bottom_panel_width)
            height: #(scene.bottom_panel_height)
            margin: #(bottom_panel_margin)
            native: true
            native_style: #(main_style)
            native_shape: #(capsule)
            native_tint: #(main_tint)
        });

        let mut top_panel_copy_card = self.ui.view(cx, ids!(top_panel_copy_card));
        script_apply_eval!(cx, top_panel_copy_card, {
            width: #(scene.top_panel_width)
            height: #(scene.top_panel_height)
            margin: #(top_panel_margin)
        });
    }

    fn configure_native_controls(&mut self, cx: &mut Cx) {
        let visible = native_liquid_glass_visual_scene(self.demo_mode, self.morph_state)
            .native_control_visible;
        self.ui
            .view(cx, ids!(native_control_host))
            .set_visible(cx, visible);
        let matrix = native_liquid_glass_control_role_matrix();

        let mut native_default_button = self.ui.widget(cx, ids!(native_default_button));
        script_apply_eval!(cx, native_default_button, {
            visible: #(visible)
            native_control: #(visible)
            native_control_role: #(matrix[0].role)
        });
        let mut native_primary_button = self.ui.widget(cx, ids!(native_primary_button));
        script_apply_eval!(cx, native_primary_button, {
            visible: #(visible)
            native_control: #(visible)
            native_control_role: #(matrix[1].role)
        });
        let mut native_utility_button = self.ui.widget(cx, ids!(native_utility_button));
        script_apply_eval!(cx, native_utility_button, {
            visible: #(visible)
            native_control: #(visible)
            native_control_role: #(matrix[2].role)
        });
        let mut native_icon_button = self.ui.widget(cx, ids!(native_icon_button));
        script_apply_eval!(cx, native_icon_button, {
            visible: #(visible)
            native_control: #(visible)
            native_control_role: #(matrix[3].role)
        });
        let mut native_nav_button = self.ui.widget(cx, ids!(native_nav_button));
        script_apply_eval!(cx, native_nav_button, {
            visible: #(visible)
            native_control: #(visible)
            native_control_role: #(matrix[4].role)
        });
    }

    fn configure_morph_controls(&mut self, cx: &mut Cx) {
        self.ui
            .view(cx, ids!(morph_row))
            .set_visible(cx, self.demo_mode == NativeDemoVisualMode::Morph);
    }

    fn start_native_press_pulse(&mut self, cx: &mut Cx) {
        self.native_press_pulse_active = true;
        self.native_press_pulse_start_time = -1.0;
        self.native_press_pulse_amount = 0.0;
        self.native_press_pulse_next_frame = cx.new_next_frame();
        log!(
            "[liquid-glass] standalone-native-example native-control pulse=start count={}",
            self.native_button_activations
        );
        self.configure_native_liquid_glass(cx);
    }

    fn register_native_control_activation(&mut self, cx: &mut Cx, label: &'static str) {
        self.native_button_activations = self.native_button_activations.saturating_add(1);
        log!(
            "[liquid-glass] standalone-native-example native-control activated label={} count={}",
            label,
            self.native_button_activations
        );
        self.start_native_press_pulse(cx);
    }

    fn update_native_press_pulse(&mut self, cx: &mut Cx, event: &NextFrameEvent) {
        if !self.native_press_pulse_active
            || !event.set.contains(&self.native_press_pulse_next_frame)
        {
            return;
        }

        if self.native_press_pulse_start_time < 0.0 {
            self.native_press_pulse_start_time = event.time;
        }

        let elapsed = event.time - self.native_press_pulse_start_time;
        self.native_press_pulse_amount = native_liquid_glass_press_pulse_amount(elapsed);
        self.configure_native_liquid_glass(cx);

        if native_liquid_glass_press_pulse_should_continue(elapsed) {
            self.native_press_pulse_next_frame = cx.new_next_frame();
        } else {
            self.native_press_pulse_active = false;
            log!(
                "[liquid-glass] standalone-native-example native-control pulse=stop count={}",
                self.native_button_activations
            );
        }
    }

    fn update_tuning_labels(&mut self, cx: &mut Cx) {
        let scene = native_liquid_glass_visual_scene(self.demo_mode, self.morph_state);
        let (mode_tint_alpha, mode_spacing, mode_radius) = native_liquid_glass_visual_mode_tuning(
            self.demo_mode,
            self.tint_alpha,
            self.container_spacing,
            self.main_radius,
        );
        let status = native_liquid_glass_status_text(
            self.demo_mode,
            self.morph_state,
            self.style_mode,
            mode_tint_alpha,
            mode_spacing,
            mode_radius,
        );
        self.ui
            .label(cx, ids!(status_readout))
            .set_text(cx, &status);
        self.ui
            .label(cx, ids!(spacing_caption))
            .set_text(cx, &format!("container spacing {:.0}", mode_spacing));
        self.ui
            .label(cx, ids!(top_panel_title))
            .set_text(cx, scene.top_panel_title);
        self.ui
            .label(cx, ids!(top_panel_line_1))
            .set_text(cx, scene.top_panel_line_1);
        self.ui
            .label(cx, ids!(top_panel_line_2))
            .set_text(cx, scene.top_panel_line_2);
        self.ui
            .label(cx, ids!(bottom_left_label))
            .set_text(cx, scene.bottom_left_label);
        self.ui
            .label(cx, ids!(bottom_right_label))
            .set_text(cx, scene.bottom_right_label);
        self.ui.label(cx, ids!(native_control_status)).set_text(
            cx,
            &native_liquid_glass_control_status_text(
                self.native_button_activations,
                self.native_press_pulse_amount,
            ),
        );
        self.ui
            .view(cx, ids!(controls))
            .set_visible(cx, self.controls_visible);
        self.ui
            .button(cx, ids!(tune_button))
            .set_visible(cx, !self.controls_visible);
    }

    fn reset_tuning(&mut self) {
        self.demo_mode = NativeDemoVisualMode::Panels;
        self.morph_state = NativeDemoMorphState::Near;
        self.style_mode = NativeDemoStyleMode::Clear;
        self.tint_alpha = DEFAULT_CLEAR_TINT_ALPHA;
        self.container_spacing = DEFAULT_CONTAINER_SPACING;
        self.main_radius = DEFAULT_MAIN_RADIUS;
    }

    fn step_tint(&mut self, delta: f32) {
        self.tint_alpha = clamp_native_demo_tint_alpha(self.tint_alpha + delta);
    }

    fn step_spacing(&mut self, delta: f64) {
        self.container_spacing = clamp_native_demo_spacing(self.container_spacing + delta);
    }

    fn step_radius(&mut self, delta: f64) {
        self.main_radius = clamp_native_demo_radius(self.main_radius + delta);
    }
}

impl MatchEvent for App {
    fn handle_startup(&mut self, cx: &mut Cx) {
        if !self.configured {
            self.configured = true;
            self.controls_visible = true;
            self.configure_native_liquid_glass(cx);
            log!("[liquid-glass] standalone-native-example configured=true panels=3");
        }
    }

    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        let mut changed = false;

        if self
            .ui
            .button(cx, ids!(native_default_button))
            .clicked(actions)
        {
            self.register_native_control_activation(cx, "Default");
        }
        if self
            .ui
            .button(cx, ids!(native_primary_button))
            .clicked(actions)
        {
            self.register_native_control_activation(cx, "Primary");
        }
        if self
            .ui
            .button(cx, ids!(native_utility_button))
            .clicked(actions)
        {
            self.register_native_control_activation(cx, "Utility");
        }
        if self.ui.button(cx, ids!(native_icon_button)).clicked(actions) {
            self.register_native_control_activation(cx, "Icon");
        }
        if self.ui.button(cx, ids!(native_nav_button)).clicked(actions) {
            self.register_native_control_activation(cx, "Nav");
        }

        if self
            .ui
            .button(cx, ids!(mode_panels_button))
            .clicked(actions)
        {
            self.demo_mode = NativeDemoVisualMode::Panels;
            changed = true;
        }
        if self.ui.button(cx, ids!(mode_morph_button)).clicked(actions) {
            self.demo_mode = NativeDemoVisualMode::Morph;
            changed = true;
        }
        if self
            .ui
            .button(cx, ids!(mode_controls_button))
            .clicked(actions)
        {
            self.demo_mode = NativeDemoVisualMode::Controls;
            changed = true;
        }
        if self
            .ui
            .button(cx, ids!(mode_readability_button))
            .clicked(actions)
        {
            self.demo_mode = NativeDemoVisualMode::Readability;
            changed = true;
        }

        if self.ui.button(cx, ids!(morph_near_button)).clicked(actions) {
            self.demo_mode = NativeDemoVisualMode::Morph;
            self.morph_state = NativeDemoMorphState::Near;
            changed = true;
        }
        if self.ui.button(cx, ids!(morph_far_button)).clicked(actions) {
            self.demo_mode = NativeDemoVisualMode::Morph;
            self.morph_state = NativeDemoMorphState::Far;
            changed = true;
        }
        if self
            .ui
            .button(cx, ids!(morph_overlap_button))
            .clicked(actions)
        {
            self.demo_mode = NativeDemoVisualMode::Morph;
            self.morph_state = NativeDemoMorphState::Overlap;
            changed = true;
        }

        if self
            .ui
            .button(cx, ids!(style_clear_button))
            .clicked(actions)
        {
            self.style_mode = NativeDemoStyleMode::Clear;
            changed = true;
        }
        if self
            .ui
            .button(cx, ids!(style_regular_button))
            .clicked(actions)
        {
            self.style_mode = NativeDemoStyleMode::Regular;
            changed = true;
        }
        if self.ui.button(cx, ids!(reset_button)).clicked(actions) {
            self.reset_tuning();
            changed = true;
        }
        if self
            .ui
            .button(cx, ids!(hide_controls_button))
            .clicked(actions)
        {
            self.controls_visible = false;
            changed = true;
        }
        if self.ui.button(cx, ids!(tune_button)).clicked(actions) {
            self.controls_visible = true;
            changed = true;
        }
        if self.ui.button(cx, ids!(tint_down_button)).clicked(actions) {
            self.step_tint(-TINT_ALPHA_STEP);
            changed = true;
        }
        if self.ui.button(cx, ids!(tint_up_button)).clicked(actions) {
            self.step_tint(TINT_ALPHA_STEP);
            changed = true;
        }
        if self
            .ui
            .button(cx, ids!(spacing_down_button))
            .clicked(actions)
        {
            self.step_spacing(-CONTAINER_SPACING_STEP);
            changed = true;
        }
        if self.ui.button(cx, ids!(spacing_up_button)).clicked(actions) {
            self.step_spacing(CONTAINER_SPACING_STEP);
            changed = true;
        }
        if self
            .ui
            .button(cx, ids!(radius_down_button))
            .clicked(actions)
        {
            self.step_radius(-MAIN_RADIUS_STEP);
            changed = true;
        }
        if self.ui.button(cx, ids!(radius_up_button)).clicked(actions) {
            self.step_radius(MAIN_RADIUS_STEP);
            changed = true;
        }

        if changed {
            let (mode_tint_alpha, mode_spacing, mode_radius) =
                native_liquid_glass_visual_mode_tuning(
                    self.demo_mode,
                    self.tint_alpha,
                    self.container_spacing,
                    self.main_radius,
                );
            log!(
                "[liquid-glass] standalone-native-example tuning {}",
                native_liquid_glass_status_text(
                    self.demo_mode,
                    self.morph_state,
                    self.style_mode,
                    mode_tint_alpha,
                    mode_spacing,
                    mode_radius
                )
            );
            self.configure_native_liquid_glass(cx);
        }
    }

    fn handle_next_frame(&mut self, cx: &mut Cx, event: &NextFrameEvent) {
        self.update_native_press_pulse(cx, event);
    }
}

impl AppMain for App {
    fn script_mod(vm: &mut ScriptVm) -> ScriptValue {
        crate::makepad_widgets::script_mod(vm);
        self::script_mod(vm)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        if let Event::WindowDragQuery(dq) = event {
            if Some(dq.window_id) == self.ui.window(cx, ids!(main_window)).window_id() {
                let size = self.ui.window(cx, ids!(main_window)).get_inner_size(cx);
                if should_start_window_drag(dq.abs, size) {
                    dq.response.set(WindowDragQueryResponse::Caption);
                    cx.set_cursor(MouseCursor::Default);
                }
            }
        }

        if let Event::WindowNativeSubstrateResolved(event) = event {
            log!(
                "[liquid-glass] standalone-native-example state={:?} style={:?} reason={}",
                event.state,
                event.style,
                event.reason
            );
        }

        self.match_event(cx, event);
        self.ui.handle_event(cx, event, &mut Scope::empty());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standalone_native_glass_uses_transparent_window_visuals() {
        let visuals = native_liquid_glass_window_visuals();

        assert!(visuals.transparent);
        assert_eq!(visuals.backdrop, WindowBackdrop::None);
        assert_eq!(visuals.backdrop_intensity, 1.0);
    }

    #[test]
    fn standalone_native_glass_clear_tint_remains_translucent() {
        let tint = native_liquid_glass_clear_tint_with_alpha(DEFAULT_CLEAR_TINT_ALPHA)
            .expect("clear tint should be present");

        assert!(tint.w > 0.0);
        assert!(tint.w < 0.35);
    }

    #[test]
    fn standalone_native_glass_tuning_values_are_clamped() {
        assert_eq!(clamp_native_demo_tint_alpha(-1.0), MIN_TINT_ALPHA);
        assert_eq!(clamp_native_demo_tint_alpha(1.0), MAX_TINT_ALPHA);
        assert_eq!(clamp_native_demo_spacing(4.0), MIN_CONTAINER_SPACING);
        assert_eq!(clamp_native_demo_spacing(80.0), MAX_CONTAINER_SPACING);
        assert_eq!(clamp_native_demo_radius(10.0), MIN_MAIN_RADIUS);
        assert_eq!(clamp_native_demo_radius(90.0), MAX_MAIN_RADIUS);
    }

    #[test]
    fn standalone_native_glass_status_text_reports_effective_values() {
        assert_eq!(
            native_liquid_glass_status_text(
                NativeDemoVisualMode::Panels,
                NativeDemoMorphState::Near,
                NativeDemoStyleMode::Clear,
                0.22,
                28.0,
                44.0
            ),
            "Panels  Clear  tint 22%  spacing 28  radius 44"
        );
    }

    #[test]
    fn standalone_native_glass_visual_modes_have_stable_labels() {
        assert_eq!(NativeDemoVisualMode::Panels.label(), "Panels");
        assert_eq!(NativeDemoVisualMode::Morph.label(), "Morph");
        assert_eq!(NativeDemoVisualMode::Controls.label(), "Controls");
        assert_eq!(NativeDemoVisualMode::Readability.label(), "Readability");
    }

    #[test]
    fn standalone_native_glass_visual_modes_adjust_effective_tuning() {
        let panels =
            native_liquid_glass_visual_mode_tuning(NativeDemoVisualMode::Panels, 0.22, 28.0, 44.0);
        let morph =
            native_liquid_glass_visual_mode_tuning(NativeDemoVisualMode::Morph, 0.22, 28.0, 44.0);
        let readability = native_liquid_glass_visual_mode_tuning(
            NativeDemoVisualMode::Readability,
            0.22,
            28.0,
            44.0,
        );

        assert_eq!(panels, (0.22, 28.0, 44.0));
        assert!(morph.1 > panels.1);
        assert!(morph.2 > panels.2);
        assert!(readability.0 > panels.0);
    }

    #[test]
    fn standalone_native_glass_visual_modes_have_distinct_scene_layouts() {
        let panels = native_liquid_glass_visual_scene(
            NativeDemoVisualMode::Panels,
            NativeDemoMorphState::Near,
        );
        let morph = native_liquid_glass_visual_scene(
            NativeDemoVisualMode::Morph,
            NativeDemoMorphState::Near,
        );
        let controls = native_liquid_glass_visual_scene(
            NativeDemoVisualMode::Controls,
            NativeDemoMorphState::Near,
        );

        assert_ne!(panels.top_panel_width, morph.top_panel_width);
        assert!(morph.bottom_panel_margin.bottom > panels.bottom_panel_margin.bottom);
        assert!(controls.main_panel_margin.left > panels.main_panel_margin.left);
        assert_eq!(morph.top_panel_title, "Morph: Near");
    }

    #[test]
    fn standalone_native_glass_controls_and_readability_modes_set_scene_purpose() {
        let panels = native_liquid_glass_visual_scene(
            NativeDemoVisualMode::Panels,
            NativeDemoMorphState::Near,
        );
        let controls = native_liquid_glass_visual_scene(
            NativeDemoVisualMode::Controls,
            NativeDemoMorphState::Near,
        );
        let readability = native_liquid_glass_visual_scene(
            NativeDemoVisualMode::Readability,
            NativeDemoMorphState::Near,
        );

        assert!(!panels.native_control_visible);
        assert!(controls.native_control_visible);
        assert!(!readability.native_control_visible);
        assert!(readability.top_panel_line_1.contains("contrast"));
    }

    #[test]
    fn standalone_native_glass_control_role_matrix_has_all_public_roles() {
        let matrix = native_liquid_glass_control_role_matrix();

        assert_eq!(matrix.len(), 5);
        assert_eq!(matrix[0].label, "Default");
        assert_eq!(matrix[0].role, ButtonNativeGlassRole::Default);
        assert_eq!(matrix[1].label, "Primary");
        assert_eq!(matrix[1].role, ButtonNativeGlassRole::Primary);
        assert_eq!(matrix[2].label, "Utility");
        assert_eq!(matrix[2].role, ButtonNativeGlassRole::Utility);
        assert_eq!(matrix[3].label, "Icon");
        assert_eq!(matrix[3].role, ButtonNativeGlassRole::Icon);
        assert_eq!(matrix[4].label, "Nav");
        assert_eq!(matrix[4].role, ButtonNativeGlassRole::Nav);
    }

    #[test]
    fn standalone_native_glass_controls_scene_exposes_role_matrix() {
        let scene = native_liquid_glass_visual_scene(
            NativeDemoVisualMode::Controls,
            NativeDemoMorphState::Near,
        );

        assert!(scene.native_control_visible);
        assert_eq!(native_liquid_glass_visible_control_count(scene), 5);
    }

    #[test]
    fn standalone_native_glass_morph_states_have_distinct_scene_layouts() {
        let near = native_liquid_glass_visual_scene(
            NativeDemoVisualMode::Morph,
            NativeDemoMorphState::Near,
        );
        let far = native_liquid_glass_visual_scene(
            NativeDemoVisualMode::Morph,
            NativeDemoMorphState::Far,
        );
        let overlap = native_liquid_glass_visual_scene(
            NativeDemoVisualMode::Morph,
            NativeDemoMorphState::Overlap,
        );

        assert_ne!(near.top_panel_margin.top, far.top_panel_margin.top);
        assert_ne!(near.bottom_panel_margin.bottom, overlap.bottom_panel_margin.bottom);
        assert_eq!(far.bottom_left_label, "Far");
        assert_eq!(overlap.bottom_left_label, "Overlap");
    }

    #[test]
    fn standalone_native_glass_morph_overlap_state_places_panels_in_overlap_zone() {
        let overlap = native_liquid_glass_visual_scene(
            NativeDemoVisualMode::Morph,
            NativeDemoMorphState::Overlap,
        );
        let top_bottom = overlap.top_panel_margin.top + overlap.top_panel_height;
        let bottom_top = 560.0 - overlap.bottom_panel_margin.bottom - overlap.bottom_panel_height;

        assert!(bottom_top < top_bottom);
    }

    #[test]
    fn standalone_native_glass_control_status_reports_activation_count() {
        assert_eq!(
            native_liquid_glass_control_status_text(0, 0.0),
            "native button 0"
        );
        assert_eq!(
            native_liquid_glass_control_status_text(3, 0.0),
            "native button 3"
        );
        assert_eq!(
            native_liquid_glass_control_status_text(3, 0.5),
            "native button 3 pulse 50%"
        );
    }

    #[test]
    fn standalone_native_glass_press_pulse_decays_to_zero() {
        assert_eq!(native_liquid_glass_press_pulse_amount(0.0), 0.0);
        assert!(native_liquid_glass_press_pulse_amount(NATIVE_PRESS_PULSE_ATTACK * 0.5) > 0.45);
        assert_eq!(
            native_liquid_glass_press_pulse_amount(NATIVE_PRESS_PULSE_ATTACK),
            1.0
        );
        assert_eq!(
            native_liquid_glass_press_pulse_amount(
                NATIVE_PRESS_PULSE_ATTACK + NATIVE_PRESS_PULSE_HOLD * 0.5
            ),
            1.0
        );
        assert!(native_liquid_glass_press_pulse_amount(NATIVE_PRESS_PULSE_DURATION * 0.5) > 0.5);
        assert_eq!(
            native_liquid_glass_press_pulse_amount(NATIVE_PRESS_PULSE_DURATION),
            0.0
        );
    }

    #[test]
    fn standalone_native_glass_press_pulse_continues_during_attack_zero_point() {
        assert!(native_liquid_glass_press_pulse_should_continue(0.0));
        assert!(native_liquid_glass_press_pulse_should_continue(
            NATIVE_PRESS_PULSE_DURATION - 0.01
        ));
        assert!(!native_liquid_glass_press_pulse_should_continue(
            NATIVE_PRESS_PULSE_DURATION
        ));
    }

    #[test]
    fn standalone_native_glass_effective_tuning_applies_pulse_boosts() {
        let (alpha, spacing, radius) = native_liquid_glass_effective_tuning(0.20, 28.0, 44.0, 1.0);

        assert_eq!(alpha, 0.30);
        assert_eq!(spacing, 48.0);
        assert_eq!(radius, 60.0);
    }

    #[test]
    fn standalone_native_glass_drag_strip_avoids_resize_edges() {
        let size = dvec2(820.0, 560.0);

        assert!(should_start_window_drag(dvec2(120.0, 24.0), size));
        assert!(!should_start_window_drag(dvec2(4.0, 24.0), size));
        assert!(!should_start_window_drag(dvec2(120.0, 4.0), size));
        assert!(!should_start_window_drag(dvec2(816.0, 24.0), size));
    }
}
