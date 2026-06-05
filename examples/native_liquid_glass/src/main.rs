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
const GOLD_GLINT_EDGE_WIDTH: f32 = 2.2;
const GOLD_GLINT_BASE_STRENGTH: f32 = 0.28;
const GOLD_GLINT_PULSE_BOOST: f32 = 0.68;
const NATIVE_DEMO_PANEL_COUNT: usize = 3;

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

    let DebugValue = Label{
        width: Fit
        height: Fit
        draw_text.color: #xF7F8FFE8
        draw_text.text_style.font_size: 11
    }

    let GoldGlintEdge = View{
        show_bg: true
        draw_bg +: {
            edge_radius: instance(44.0)
            edge_width: instance(2.2)
            glint_strength: instance(0.28)
            pulse: instance(0.0)
            color: instance(#xD79A28E0)
            hot_color: instance(#xFFF2B8FF)

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                let inset = self.edge_width * 0.5
                let w = max(1.0, self.rect_size.x - inset * 2.0)
                let h = max(1.0, self.rect_size.y - inset * 2.0)
                let radius = min(max(0.0, self.edge_radius - inset), max(0.0, min(w, h) * 0.5 - 0.5))

                sdf.box(inset, inset, w, h, radius)

                let phase = self.pos.x * 1.35 + self.pos.y * 0.55 + self.draw_pass.time * 0.20
                let runner = 1.0 - abs(fract(phase) - 0.5) * 2.0
                let runner_peak = runner * runner * runner * runner * runner
                let phase2 = self.pos.x * -0.75 + self.pos.y * 1.15 + self.draw_pass.time * 0.13 + 0.37
                let runner2 = 1.0 - abs(fract(phase2) - 0.5) * 2.0
                let runner2_peak = runner2 * runner2 * runner2 * runner2 * runner2
                let sparkle = Math.random_2d(
                    self.pos * self.rect_size
                    + vec2(self.draw_pass.time * 31.0, self.draw_pass.time * 17.0)
                )
                let sparkle_hot = max(0.0, sparkle - 0.74) * 3.8
                let glint_peak = clamp(
                    runner_peak * 0.78 + runner2_peak * 0.58 + sparkle_hot * 0.18,
                    0.0,
                    1.0
                )
                let hot = clamp(glint_peak * self.glint_strength * 1.85 + self.pulse * 0.42, 0.0, 1.0)
                let alpha = clamp(
                    0.015 + self.glint_strength * (0.04 + glint_peak * 0.96) + self.pulse * 0.16,
                    0.0,
                    0.92
                )
                let rgb = mix(self.color.rgb, self.hot_color.rgb, hot)

                sdf.stroke(vec4(rgb, alpha), self.edge_width)
                sdf.glow(vec4(self.hot_color.rgb, alpha * 0.42), self.edge_width * 3.2)
                return sdf.result
            }
        }
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

                    gold_edge_layer := View{
                        width: Fill
                        height: Fill
                        flow: Overlay
                        draw_bg.color: #00000000

                        main_gold_edge := GoldGlintEdge{
                            width: Fill
                            height: Fill
                            margin: Inset{left: 28 top: 28 right: 28 bottom: 28}
                        }

                        top_gold_edge_host := View{
                            width: Fill
                            height: Fill
                            flow: Overlay
                            align: Align{x: 1.0 y: 0.0}
                            draw_bg.color: #00000000

                            top_gold_edge := GoldGlintEdge{
                                width: 282
                                height: 88
                                margin: Inset{top: 54 right: 56}
                            }
                        }

                        bottom_gold_edge_host := View{
                            width: Fill
                            height: Fill
                            flow: Overlay
                            align: Align{x: 0.5 y: 1.0}
                            draw_bg.color: #00000000

                            bottom_gold_edge := GoldGlintEdge{
                                width: 360
                                height: 64
                                margin: Inset{bottom: 58}
                                draw_bg +: {
                                    edge_radius: 32.0
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
                                mode_geometry_button := ControlButton{text: "Geometry"}
                                mode_morph_button := ControlButton{text: "Morph"}
                                mode_container_button := ControlButton{text: "Container"}
                                mode_debug_button := ControlButton{text: "Debug"}
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

                            container_row := View{
                                visible: false
                                width: Fill
                                height: Fit
                                flow: Right
                                spacing: 8
                                align: Align{x: 0.0 y: 0.5}
                                draw_bg.color: #00000000

                                container_near_button := ControlButton{text: "Near"}
                                container_threshold_button := ControlButton{text: "Threshold"}
                                container_far_button := ControlButton{text: "Far"}
                            }

                            readability_row := View{
                                visible: false
                                width: Fill
                                height: Fit
                                flow: Right
                                spacing: 8
                                align: Align{x: 0.0 y: 0.5}
                                draw_bg.color: #00000000

                                readability_mixed_button := ControlButton{text: "Mixed"}
                                readability_bright_button := ControlButton{text: "Bright"}
                                readability_dark_button := ControlButton{text: "Dark"}
                            }

                            native_control_state_row := View{
                                visible: false
                                width: Fill
                                height: Fit
                                flow: Right
                                spacing: 8
                                align: Align{x: 0.0 y: 0.5}
                                draw_bg.color: #00000000

                                primary_enabled_toggle_button := ControlButton{text: "Disable Primary"}
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
                                tone_button := ControlButton{text: "Tone: Arctic"}
                                reset_button := ControlButton{text: "Reset"}
                                hide_controls_button := ControlButton{text: "Hide"}
                                glint_toggle_button := ControlButton{text: "Glint Off"}
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
                                text: "Panels  Clear  tint 22%  spacing 28  radius 44  makepad-glint:on"
                            }
                            debug_readout_1 := DebugValue{
                                visible: false
                                text: "mode=Panels style=Clear tone=Arctic panels=3 z=0/1/2"
                            }
                            debug_readout_2 := DebugValue{
                                visible: false
                                text: "tint=22% spacing=28 radius=44 hit=passthrough makepad-glint:on"
                            }
                            readability_probe_box := View{
                                visible: false
                                width: Fit
                                height: Fit
                                flow: Down
                                spacing: 4
                                padding: Inset{left: 10 top: 7 right: 10 bottom: 7}
                                show_bg: true
                                draw_bg +: {
                                    color: #xFFFFFF40
                                }

                                readability_probe_title := DebugValue{
                                    text: "Mixed wallpaper probe"
                                }
                                readability_probe_detail := DebugValue{
                                    text: "foreground and separator tokens"
                                }
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
    container_preset: NativeDemoContainerPreset,
    #[rust]
    readability_probe: NativeDemoReadabilityProbe,
    #[rust]
    style_mode: NativeDemoStyleMode,
    #[rust]
    tone_mode: NativeDemoColorTone,
    #[rust]
    tint_alpha: f32,
    #[rust]
    container_spacing: f64,
    #[rust]
    main_radius: f64,
    #[rust]
    controls_visible: bool,
    #[rust]
    makepad_demo_glint_visible: bool,
    #[rust]
    native_primary_control_enabled: bool,
    #[rust]
    native_button_activations: u32,
    #[rust]
    native_press_pulse_next_frame: NextFrame,
    #[rust]
    gold_glint_next_frame: NextFrame,
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
    Geometry,
    Morph,
    Container,
    Debug,
    Controls,
    Readability,
}

impl NativeDemoVisualMode {
    fn label(self) -> &'static str {
        match self {
            Self::Panels => "Panels",
            Self::Geometry => "Geometry",
            Self::Morph => "Morph",
            Self::Container => "Container",
            Self::Debug => "Debug",
            Self::Controls => "Controls",
            Self::Readability => "Readability",
        }
    }
}

fn native_liquid_glass_start_mode_from_env_value(value: Option<&str>) -> NativeDemoVisualMode {
    match value
        .map(str::trim)
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "debug" => NativeDemoVisualMode::Debug,
        "container" => NativeDemoVisualMode::Container,
        "geometry" => NativeDemoVisualMode::Geometry,
        "morph" => NativeDemoVisualMode::Morph,
        "control" | "controls" => NativeDemoVisualMode::Controls,
        "readability" => NativeDemoVisualMode::Readability,
        _ => NativeDemoVisualMode::Panels,
    }
}

fn native_liquid_glass_start_mode_from_env() -> NativeDemoVisualMode {
    native_liquid_glass_start_mode_from_env_value(
        std::env::var("MAKEPAD_NATIVE_LIQUID_GLASS_START_MODE")
            .ok()
            .as_deref(),
    )
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum NativeDemoContainerPreset {
    Near,
    #[default]
    Threshold,
    Far,
}

impl NativeDemoContainerPreset {
    fn label(self) -> &'static str {
        match self {
            Self::Near => "Near",
            Self::Threshold => "Threshold",
            Self::Far => "Far",
        }
    }

    fn spacing(self) -> f64 {
        match self {
            Self::Near => MIN_CONTAINER_SPACING + 2.0,
            Self::Threshold => DEFAULT_CONTAINER_SPACING,
            Self::Far => MAX_CONTAINER_SPACING,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum NativeDemoReadabilityProbe {
    #[default]
    Mixed,
    Bright,
    Dark,
}

impl NativeDemoReadabilityProbe {
    fn label(self) -> &'static str {
        match self {
            Self::Mixed => "Mixed",
            Self::Bright => "Bright",
            Self::Dark => "Dark",
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
    primary_control_enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct NativeDemoDebugSummary {
    line_1: String,
    line_2: String,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct NativeDemoReadabilityTokens {
    foreground: Vec4f,
    secondary: Vec4f,
    separator: Vec4f,
    probe_background: Vec4f,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct NativeDemoV41DescriptorCoverage {
    style: bool,
    tint: bool,
    shape: bool,
    radius: bool,
    z_order: bool,
    hit_test_passthrough: bool,
    container_spacing: bool,
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

fn native_liquid_glass_primary_enabled_toggle_text(enabled: bool) -> &'static str {
    if enabled {
        "Disable Primary"
    } else {
        "Enable Primary"
    }
}

#[cfg(test)]
fn native_liquid_glass_v4_1_descriptor_coverage() -> NativeDemoV41DescriptorCoverage {
    NativeDemoV41DescriptorCoverage {
        style: true,
        tint: true,
        shape: true,
        radius: true,
        z_order: true,
        hit_test_passthrough: true,
        container_spacing: true,
    }
}

#[cfg(test)]
fn native_liquid_glass_visible_control_count(scene: NativeDemoVisualScene) -> usize {
    if scene.native_control_visible {
        native_liquid_glass_control_role_matrix().len()
    } else {
        0
    }
}

fn native_liquid_glass_control_resize_validation_text(
    visual_mode: NativeDemoVisualMode,
) -> &'static str {
    match visual_mode {
        NativeDemoVisualMode::Controls => "resize keeps 5 native controls",
        _ => "resize keeps native controls hidden outside Controls",
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
            primary_control_enabled: false,
        },
        NativeDemoVisualMode::Geometry => NativeDemoVisualScene {
            main_panel_margin: NativeDemoInset::all(42.0),
            top_panel_width: 220.0,
            top_panel_height: 112.0,
            top_panel_margin: NativeDemoInset::top_right(44.0, 64.0),
            bottom_panel_width: 240.0,
            bottom_panel_height: 48.0,
            bottom_panel_margin: NativeDemoInset::bottom(78.0),
            top_panel_title: "Geometry",
            top_panel_line_1: "rounded rect",
            top_panel_line_2: "capsule edge tracking",
            bottom_left_label: "Capsule",
            bottom_right_label: "Radius",
            native_control_visible: false,
            primary_control_enabled: false,
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
                primary_control_enabled: false,
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
                primary_control_enabled: false,
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
                primary_control_enabled: false,
            },
        },
        NativeDemoVisualMode::Container => NativeDemoVisualScene {
            main_panel_margin: NativeDemoInset::all(58.0),
            top_panel_width: 200.0,
            top_panel_height: 120.0,
            top_panel_margin: NativeDemoInset::top_right(54.0, 40.0),
            bottom_panel_width: 260.0,
            bottom_panel_height: 64.0,
            bottom_panel_margin: NativeDemoInset::bottom(312.0),
            top_panel_title: "Container",
            top_panel_line_1: "spacing threshold",
            top_panel_line_2: "native setSpacing probe",
            bottom_left_label: "Preset",
            bottom_right_label: "Spacing",
            native_control_visible: false,
            primary_control_enabled: false,
        },
        NativeDemoVisualMode::Debug => NativeDemoVisualScene {
            main_panel_margin: NativeDemoInset::all(50.0),
            top_panel_width: 180.0,
            top_panel_height: 112.0,
            top_panel_margin: NativeDemoInset::top_right(54.0, 40.0),
            bottom_panel_width: 420.0,
            bottom_panel_height: 70.0,
            bottom_panel_margin: NativeDemoInset::bottom(74.0),
            top_panel_title: "Debug batch",
            top_panel_line_1: "panel ids 0 / 1 / 2",
            top_panel_line_2: "hit-test passthrough",
            bottom_left_label: "Batch",
            bottom_right_label: "State",
            native_control_visible: false,
            primary_control_enabled: false,
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
            primary_control_enabled: true,
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
            primary_control_enabled: false,
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum NativeDemoColorTone {
    #[default]
    Arctic,
    Emerald,
    Rose,
    Violet,
    Aijiro,
    Usuao,
    Sorairo,
    Mizuhanada,
    Usuhanada,
    Asahanada,
    Kokihanada,
    Shinbashi,
    Shirahana,
    Hanaasagi,
    Mizuasagi,
    Sabiasagi,
    Minatonezumi,
    Kamenozoki,
    Seiran,
    Hisokuiro,
    Usuhanairo,
    Usuhanazakura,
}

impl NativeDemoColorTone {
    fn label(self) -> &'static str {
        match self {
            Self::Arctic => "Arctic",
            Self::Emerald => "Emerald",
            Self::Rose => "Rose",
            Self::Violet => "Violet",
            Self::Aijiro => "Aijiro",
            Self::Usuao => "Usuao",
            Self::Sorairo => "Sorairo",
            Self::Mizuhanada => "Mizuhanada",
            Self::Usuhanada => "Usuhanada",
            Self::Asahanada => "Asahanada",
            Self::Kokihanada => "Kokihanada",
            Self::Shinbashi => "Shinbashi",
            Self::Shirahana => "Shirahana",
            Self::Hanaasagi => "Hanaasagi",
            Self::Mizuasagi => "Mizuasagi",
            Self::Sabiasagi => "Sabiasagi",
            Self::Minatonezumi => "Minatonezumi",
            Self::Kamenozoki => "Kamenozoki",
            Self::Seiran => "Seiran",
            Self::Hisokuiro => "Hisokuiro",
            Self::Usuhanairo => "Usuhanairo",
            Self::Usuhanazakura => "Usuhanazakura",
        }
    }

    fn next(self) -> Self {
        match self {
            Self::Arctic => Self::Emerald,
            Self::Emerald => Self::Rose,
            Self::Rose => Self::Violet,
            Self::Violet => Self::Aijiro,
            Self::Aijiro => Self::Usuao,
            Self::Usuao => Self::Sorairo,
            Self::Sorairo => Self::Mizuhanada,
            Self::Mizuhanada => Self::Usuhanada,
            Self::Usuhanada => Self::Asahanada,
            Self::Asahanada => Self::Kokihanada,
            Self::Kokihanada => Self::Shinbashi,
            Self::Shinbashi => Self::Shirahana,
            Self::Shirahana => Self::Hanaasagi,
            Self::Hanaasagi => Self::Mizuasagi,
            Self::Mizuasagi => Self::Sabiasagi,
            Self::Sabiasagi => Self::Minatonezumi,
            Self::Minatonezumi => Self::Kamenozoki,
            Self::Kamenozoki => Self::Seiran,
            Self::Seiran => Self::Hisokuiro,
            Self::Hisokuiro => Self::Usuhanairo,
            Self::Usuhanairo => Self::Usuhanazakura,
            Self::Usuhanazakura => Self::Arctic,
        }
    }

    #[cfg(test)]
    fn japanese_blue_palette() -> [Self; 18] {
        [
            Self::Aijiro,
            Self::Usuao,
            Self::Sorairo,
            Self::Mizuhanada,
            Self::Usuhanada,
            Self::Asahanada,
            Self::Kokihanada,
            Self::Shinbashi,
            Self::Shirahana,
            Self::Hanaasagi,
            Self::Mizuasagi,
            Self::Sabiasagi,
            Self::Minatonezumi,
            Self::Kamenozoki,
            Self::Seiran,
            Self::Hisokuiro,
            Self::Usuhanairo,
            Self::Usuhanazakura,
        ]
    }

    fn clear_tint_rgb(self) -> (f32, f32, f32) {
        match self {
            Self::Arctic => (0.90, 0.96, 1.0),
            Self::Emerald => (0.74, 1.0, 0.88),
            Self::Rose => (1.0, 0.84, 0.92),
            Self::Violet => (0.88, 0.84, 1.0),
            Self::Aijiro => (0.85, 0.91, 0.92),
            Self::Usuao => (0.23, 0.55, 0.69),
            Self::Sorairo => (0.55, 0.76, 0.82),
            Self::Mizuhanada => (0.61, 0.79, 0.82),
            Self::Usuhanada => (0.31, 0.47, 0.60),
            Self::Asahanada => (0.47, 0.67, 0.71),
            Self::Kokihanada => (0.20, 0.29, 0.48),
            Self::Shinbashi => (0.34, 0.66, 0.68),
            Self::Shirahana => (0.42, 0.52, 0.56),
            Self::Hanaasagi => (0.20, 0.53, 0.61),
            Self::Mizuasagi => (0.48, 0.64, 0.64),
            Self::Sabiasagi => (0.36, 0.55, 0.54),
            Self::Minatonezumi => (0.50, 0.59, 0.62),
            Self::Kamenozoki => (0.53, 0.77, 0.77),
            Self::Seiran => (0.68, 0.83, 0.84),
            Self::Hisokuiro => (0.61, 0.75, 0.78),
            Self::Usuhanairo => (0.37, 0.51, 0.64),
            Self::Usuhanazakura => (0.33, 0.46, 0.66),
        }
    }

    fn regular_tint_rgb(self) -> (f32, f32, f32) {
        match self {
            Self::Arctic => (1.0, 1.0, 1.0),
            Self::Emerald => (0.88, 1.0, 0.94),
            Self::Rose => (1.0, 0.93, 0.96),
            Self::Violet => (0.95, 0.92, 1.0),
            _ => {
                let (r, g, b) = self.clear_tint_rgb();
                (0.52 + r * 0.48, 0.52 + g * 0.48, 0.52 + b * 0.48)
            }
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

fn native_liquid_glass_clear_tint_for_tone(tone: NativeDemoColorTone, alpha: f32) -> Option<Vec4f> {
    let (r, g, b) = tone.clear_tint_rgb();
    Some(vec4(r, g, b, clamp_native_demo_tint_alpha(alpha)))
}

fn native_liquid_glass_regular_tint_for_tone(
    tone: NativeDemoColorTone,
    alpha: f32,
) -> Option<Vec4f> {
    let (r, g, b) = tone.regular_tint_rgb();
    Some(vec4(r, g, b, clamp_native_demo_tint_alpha(alpha)))
}

fn native_liquid_glass_tint_for_mode(
    mode: NativeDemoStyleMode,
    tone: NativeDemoColorTone,
    alpha: f32,
) -> Option<Vec4f> {
    match mode {
        NativeDemoStyleMode::Clear => native_liquid_glass_clear_tint_for_tone(tone, alpha),
        NativeDemoStyleMode::Regular => native_liquid_glass_regular_tint_for_tone(tone, alpha),
    }
}

fn native_liquid_glass_readability_tokens(
    probe: NativeDemoReadabilityProbe,
) -> NativeDemoReadabilityTokens {
    match probe {
        NativeDemoReadabilityProbe::Bright => NativeDemoReadabilityTokens {
            foreground: vec4(0.03, 0.05, 0.08, 0.94),
            secondary: vec4(0.08, 0.12, 0.18, 0.72),
            separator: vec4(0.06, 0.09, 0.13, 0.48),
            probe_background: vec4(0.96, 0.98, 1.0, 0.46),
        },
        NativeDemoReadabilityProbe::Dark => NativeDemoReadabilityTokens {
            foreground: vec4(0.97, 0.98, 1.0, 0.94),
            secondary: vec4(0.82, 0.88, 0.96, 0.72),
            separator: vec4(0.94, 0.97, 1.0, 0.48),
            probe_background: vec4(0.02, 0.03, 0.05, 0.42),
        },
        NativeDemoReadabilityProbe::Mixed => NativeDemoReadabilityTokens {
            foreground: vec4(0.97, 0.98, 1.0, 0.92),
            secondary: vec4(0.72, 0.80, 0.92, 0.70),
            separator: vec4(0.86, 0.92, 1.0, 0.44),
            probe_background: vec4(0.14, 0.18, 0.24, 0.38),
        },
    }
}

fn native_liquid_glass_visual_status_label(
    visual_mode: NativeDemoVisualMode,
    morph_state: NativeDemoMorphState,
    container_preset: NativeDemoContainerPreset,
    readability_probe: NativeDemoReadabilityProbe,
) -> String {
    match visual_mode {
        NativeDemoVisualMode::Morph => format!("{} {}", visual_mode.label(), morph_state.label()),
        NativeDemoVisualMode::Container => {
            format!("{}: {}", visual_mode.label(), container_preset.label())
        }
        NativeDemoVisualMode::Readability => {
            format!("{}: {}", visual_mode.label(), readability_probe.label())
        }
        _ => visual_mode.label().to_string(),
    }
}

fn native_liquid_glass_status_text(
    visual_mode: NativeDemoVisualMode,
    morph_state: NativeDemoMorphState,
    container_preset: NativeDemoContainerPreset,
    readability_probe: NativeDemoReadabilityProbe,
    mode: NativeDemoStyleMode,
    alpha: f32,
    spacing: f64,
    radius: f64,
    makepad_demo_glint_visible: bool,
) -> String {
    let visual_label = native_liquid_glass_visual_status_label(
        visual_mode,
        morph_state,
        container_preset,
        readability_probe,
    );
    format!(
        "{}  {}  tint {:.0}%  spacing {:.0}  radius {:.0}  {}",
        visual_label,
        mode.label(),
        clamp_native_demo_tint_alpha(alpha) * 100.0,
        clamp_native_demo_spacing(spacing),
        clamp_native_demo_radius(radius),
        makepad_demo_glint_status_label(makepad_demo_glint_visible)
    )
}

fn native_liquid_glass_debug_summary(
    visual_mode: NativeDemoVisualMode,
    morph_state: NativeDemoMorphState,
    container_preset: NativeDemoContainerPreset,
    readability_probe: NativeDemoReadabilityProbe,
    style_mode: NativeDemoStyleMode,
    tone_mode: NativeDemoColorTone,
    alpha: f32,
    spacing: f64,
    radius: f64,
    panel_count: usize,
    makepad_demo_glint_visible: bool,
) -> NativeDemoDebugSummary {
    NativeDemoDebugSummary {
        line_1: format!(
            "mode={} style={} tone={} panels={} z=0/1/2",
            native_liquid_glass_visual_status_label(
                visual_mode,
                morph_state,
                container_preset,
                readability_probe,
            ),
            style_mode.label(),
            tone_mode.label(),
            panel_count
        ),
        line_2: format!(
            "tint={:.0}% spacing={:.0} radius={:.0} hit=passthrough {}",
            clamp_native_demo_tint_alpha(alpha) * 100.0,
            clamp_native_demo_spacing(spacing),
            clamp_native_demo_radius(radius),
            makepad_demo_glint_status_label(makepad_demo_glint_visible)
        ),
    }
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

fn native_liquid_glass_gold_glint_edge_width() -> f32 {
    GOLD_GLINT_EDGE_WIDTH
}

#[cfg(test)]
fn native_liquid_glass_gold_glint_center_inset(edge_width: f32) -> f32 {
    edge_width.max(0.0) * 0.5
}

#[cfg(test)]
fn native_liquid_glass_gold_glint_center_radius(edge_radius: f32, edge_width: f32) -> f32 {
    (edge_radius - native_liquid_glass_gold_glint_center_inset(edge_width)).max(0.0)
}

fn native_liquid_glass_gold_glint_strength(pulse: f64) -> f32 {
    let pulse = pulse.clamp(0.0, 1.0) as f32;
    (GOLD_GLINT_BASE_STRENGTH + pulse * GOLD_GLINT_PULSE_BOOST).clamp(0.0, 1.0)
}

fn makepad_demo_glint_toggle_text(visible: bool) -> &'static str {
    if visible {
        "Glint Off"
    } else {
        "Glint On"
    }
}

fn makepad_demo_glint_status_label(visible: bool) -> &'static str {
    if visible {
        "makepad-glint:on"
    } else {
        "makepad-glint:off"
    }
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
    container_preset: NativeDemoContainerPreset,
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
        NativeDemoVisualMode::Geometry => (
            clamp_native_demo_tint_alpha(alpha + 0.02),
            clamp_native_demo_spacing(spacing),
            clamp_native_demo_radius(radius + 18.0),
        ),
        NativeDemoVisualMode::Morph => (
            clamp_native_demo_tint_alpha(alpha + 0.03),
            clamp_native_demo_spacing(spacing + 14.0),
            clamp_native_demo_radius(radius + 10.0),
        ),
        NativeDemoVisualMode::Container => (
            clamp_native_demo_tint_alpha(alpha + 0.02),
            clamp_native_demo_spacing(container_preset.spacing()),
            clamp_native_demo_radius(radius + 4.0),
        ),
        NativeDemoVisualMode::Debug => (
            clamp_native_demo_tint_alpha(alpha + 0.04),
            clamp_native_demo_spacing(spacing),
            clamp_native_demo_radius(radius),
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
        let passthrough = makepad_widgets::glass_panel::GlassNativeHitTest::Passthrough;
        let scene = native_liquid_glass_visual_scene(self.demo_mode, self.morph_state);
        let main_style = self.style_mode.style();
        let top_style = self.style_mode.opposite_style();
        let (mode_tint_alpha, mode_spacing, mode_radius) = native_liquid_glass_visual_mode_tuning(
            self.demo_mode,
            self.container_preset,
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
        let main_tint = native_liquid_glass_tint_for_mode(
            self.style_mode,
            self.tone_mode,
            effective_tint_alpha,
        );
        let top_tint = native_liquid_glass_tint_for_mode(
            match self.style_mode {
                NativeDemoStyleMode::Clear => NativeDemoStyleMode::Regular,
                NativeDemoStyleMode::Regular => NativeDemoStyleMode::Clear,
            },
            self.tone_mode,
            (effective_tint_alpha * 0.82).max(MIN_TINT_ALPHA),
        );
        let container_spacing = effective_spacing;
        let main_radius = effective_radius;
        let top_radius = clamp_native_demo_radius(main_radius * 0.68).min(34.0);
        let main_panel_margin = scene.main_panel_margin.to_inset();
        let top_panel_margin = scene.top_panel_margin.to_inset();
        let bottom_panel_margin = scene.bottom_panel_margin.to_inset();
        let gold_edge_width = native_liquid_glass_gold_glint_edge_width();
        let gold_glint_strength =
            native_liquid_glass_gold_glint_strength(self.native_press_pulse_amount);
        let gold_pulse = self.native_press_pulse_amount.clamp(0.0, 1.0) as f32;

        self.ui
            .view(cx, ids!(gold_edge_layer))
            .set_visible(cx, self.makepad_demo_glint_visible);

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
            native_hit_test: #(passthrough)
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
            native_hit_test: #(passthrough)
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
            native_hit_test: #(passthrough)
        });

        let mut top_panel_copy_card = self.ui.view(cx, ids!(top_panel_copy_card));
        script_apply_eval!(cx, top_panel_copy_card, {
            width: #(scene.top_panel_width)
            height: #(scene.top_panel_height)
            margin: #(top_panel_margin)
        });

        let mut main_gold_edge = self.ui.view(cx, ids!(main_gold_edge));
        script_apply_eval!(cx, main_gold_edge, {
            margin: #(main_panel_margin)
            draw_bg +: {
                edge_radius: #(main_radius as f32)
                edge_width: #(gold_edge_width)
                glint_strength: #(gold_glint_strength)
                pulse: #(gold_pulse)
            }
        });

        let mut top_gold_edge = self.ui.view(cx, ids!(top_gold_edge));
        script_apply_eval!(cx, top_gold_edge, {
            width: #(scene.top_panel_width)
            height: #(scene.top_panel_height)
            margin: #(top_panel_margin)
            draw_bg +: {
                edge_radius: #(top_radius as f32)
                edge_width: #(gold_edge_width)
                glint_strength: #(gold_glint_strength)
                pulse: #(gold_pulse)
            }
        });

        let mut bottom_gold_edge = self.ui.view(cx, ids!(bottom_gold_edge));
        script_apply_eval!(cx, bottom_gold_edge, {
            width: #(scene.bottom_panel_width)
            height: #(scene.bottom_panel_height)
            margin: #(bottom_panel_margin)
            draw_bg +: {
                edge_radius: #((scene.bottom_panel_height * 0.5) as f32)
                edge_width: #(gold_edge_width)
                glint_strength: #(gold_glint_strength)
                pulse: #(gold_pulse)
            }
        });
    }

    fn configure_native_controls(&mut self, cx: &mut Cx) {
        let visible = native_liquid_glass_visual_scene(self.demo_mode, self.morph_state)
            .native_control_visible;
        let primary_enabled = visible && self.native_primary_control_enabled;
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
            enabled: #(primary_enabled)
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
        self.ui
            .view(cx, ids!(container_row))
            .set_visible(cx, self.demo_mode == NativeDemoVisualMode::Container);
        self.ui
            .view(cx, ids!(readability_row))
            .set_visible(cx, self.demo_mode == NativeDemoVisualMode::Readability);
        self.ui
            .view(cx, ids!(native_control_state_row))
            .set_visible(cx, self.demo_mode == NativeDemoVisualMode::Controls);
        self.ui
            .widget(cx, ids!(primary_enabled_toggle_button))
            .set_text(
                cx,
                native_liquid_glass_primary_enabled_toggle_text(
                    self.native_primary_control_enabled,
                ),
            );
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

    fn start_gold_glint_animation(&mut self, cx: &mut Cx) {
        self.gold_glint_next_frame = cx.new_next_frame();
    }

    fn update_gold_glint_animation(&mut self, cx: &mut Cx, event: &NextFrameEvent) {
        if !self.makepad_demo_glint_visible {
            return;
        }
        if !event.set.contains(&self.gold_glint_next_frame) {
            return;
        }
        self.ui.view(cx, ids!(gold_edge_layer)).redraw(cx);
        self.gold_glint_next_frame = cx.new_next_frame();
    }

    fn update_tuning_labels(&mut self, cx: &mut Cx) {
        let scene = native_liquid_glass_visual_scene(self.demo_mode, self.morph_state);
        let (mode_tint_alpha, mode_spacing, mode_radius) = native_liquid_glass_visual_mode_tuning(
            self.demo_mode,
            self.container_preset,
            self.tint_alpha,
            self.container_spacing,
            self.main_radius,
        );
        let status = native_liquid_glass_status_text(
            self.demo_mode,
            self.morph_state,
            self.container_preset,
            self.readability_probe,
            self.style_mode,
            mode_tint_alpha,
            mode_spacing,
            mode_radius,
            self.makepad_demo_glint_visible,
        );
        self.ui
            .label(cx, ids!(status_readout))
            .set_text(cx, &status);
        let debug_summary = native_liquid_glass_debug_summary(
            self.demo_mode,
            self.morph_state,
            self.container_preset,
            self.readability_probe,
            self.style_mode,
            self.tone_mode,
            mode_tint_alpha,
            mode_spacing,
            mode_radius,
            NATIVE_DEMO_PANEL_COUNT,
            self.makepad_demo_glint_visible,
        );
        let debug_visible = self.demo_mode == NativeDemoVisualMode::Debug;
        self.ui
            .label(cx, ids!(debug_readout_1))
            .set_text(cx, &debug_summary.line_1);
        self.ui
            .label(cx, ids!(debug_readout_2))
            .set_text(cx, &debug_summary.line_2);
        self.ui
            .widget(cx, ids!(debug_readout_1))
            .set_visible(cx, debug_visible);
        self.ui
            .widget(cx, ids!(debug_readout_2))
            .set_visible(cx, debug_visible);
        let readability_visible = self.demo_mode == NativeDemoVisualMode::Readability;
        let readability_tokens = native_liquid_glass_readability_tokens(self.readability_probe);
        self.ui.label(cx, ids!(readability_probe_title)).set_text(
            cx,
            &format!("{} wallpaper probe", self.readability_probe.label()),
        );
        self.ui
            .label(cx, ids!(readability_probe_detail))
            .set_text(cx, "foreground / separator tokens stay readable");
        let mut readability_probe_box = self.ui.view(cx, ids!(readability_probe_box));
        script_apply_eval!(cx, readability_probe_box, {
            visible: #(readability_visible)
            draw_bg +: {
                color: #(readability_tokens.probe_background)
            }
        });
        let mut readability_probe_title = self.ui.label(cx, ids!(readability_probe_title));
        script_apply_eval!(cx, readability_probe_title, {
            draw_text +: {
                color: #(readability_tokens.foreground)
            }
        });
        let mut readability_probe_detail = self.ui.label(cx, ids!(readability_probe_detail));
        script_apply_eval!(cx, readability_probe_detail, {
            draw_text +: {
                color: #(readability_tokens.secondary)
            }
        });
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
            .widget(cx, ids!(tone_button))
            .set_text(cx, &format!("Tone: {}", self.tone_mode.label()));
        self.ui.widget(cx, ids!(glint_toggle_button)).set_text(
            cx,
            makepad_demo_glint_toggle_text(self.makepad_demo_glint_visible),
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
        self.container_preset = NativeDemoContainerPreset::Threshold;
        self.readability_probe = NativeDemoReadabilityProbe::Mixed;
        self.style_mode = NativeDemoStyleMode::Clear;
        self.tone_mode = NativeDemoColorTone::Arctic;
        self.tint_alpha = DEFAULT_CLEAR_TINT_ALPHA;
        self.container_spacing = DEFAULT_CONTAINER_SPACING;
        self.main_radius = DEFAULT_MAIN_RADIUS;
        self.makepad_demo_glint_visible = true;
        self.native_primary_control_enabled = true;
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
            self.demo_mode = native_liquid_glass_start_mode_from_env();
            self.container_preset = NativeDemoContainerPreset::Threshold;
            self.readability_probe = NativeDemoReadabilityProbe::Mixed;
            self.makepad_demo_glint_visible = true;
            self.native_primary_control_enabled = true;
            self.tone_mode = NativeDemoColorTone::Arctic;
            self.configure_native_liquid_glass(cx);
            self.start_gold_glint_animation(cx);
            log!(
                "[liquid-glass] standalone-native-example configured=true panels={} mode={}",
                NATIVE_DEMO_PANEL_COUNT,
                self.demo_mode.label()
            );
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
        if self
            .ui
            .button(cx, ids!(native_icon_button))
            .clicked(actions)
        {
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
        if self
            .ui
            .button(cx, ids!(mode_geometry_button))
            .clicked(actions)
        {
            self.demo_mode = NativeDemoVisualMode::Geometry;
            changed = true;
        }
        if self.ui.button(cx, ids!(mode_morph_button)).clicked(actions) {
            self.demo_mode = NativeDemoVisualMode::Morph;
            changed = true;
        }
        if self
            .ui
            .button(cx, ids!(mode_container_button))
            .clicked(actions)
        {
            self.demo_mode = NativeDemoVisualMode::Container;
            self.container_spacing = self.container_preset.spacing();
            changed = true;
        }
        if self.ui.button(cx, ids!(mode_debug_button)).clicked(actions) {
            self.demo_mode = NativeDemoVisualMode::Debug;
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
        if self
            .ui
            .button(cx, ids!(primary_enabled_toggle_button))
            .clicked(actions)
        {
            self.demo_mode = NativeDemoVisualMode::Controls;
            self.native_primary_control_enabled = !self.native_primary_control_enabled;
            log!(
                "[liquid-glass] standalone-native-example native-control-state label=Primary enabled={}",
                self.native_primary_control_enabled
            );
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
            .button(cx, ids!(container_near_button))
            .clicked(actions)
        {
            self.demo_mode = NativeDemoVisualMode::Container;
            self.container_preset = NativeDemoContainerPreset::Near;
            self.container_spacing = self.container_preset.spacing();
            changed = true;
        }
        if self
            .ui
            .button(cx, ids!(container_threshold_button))
            .clicked(actions)
        {
            self.demo_mode = NativeDemoVisualMode::Container;
            self.container_preset = NativeDemoContainerPreset::Threshold;
            self.container_spacing = self.container_preset.spacing();
            changed = true;
        }
        if self
            .ui
            .button(cx, ids!(container_far_button))
            .clicked(actions)
        {
            self.demo_mode = NativeDemoVisualMode::Container;
            self.container_preset = NativeDemoContainerPreset::Far;
            self.container_spacing = self.container_preset.spacing();
            changed = true;
        }

        if self
            .ui
            .button(cx, ids!(readability_mixed_button))
            .clicked(actions)
        {
            self.demo_mode = NativeDemoVisualMode::Readability;
            self.readability_probe = NativeDemoReadabilityProbe::Mixed;
            changed = true;
        }
        if self
            .ui
            .button(cx, ids!(readability_bright_button))
            .clicked(actions)
        {
            self.demo_mode = NativeDemoVisualMode::Readability;
            self.readability_probe = NativeDemoReadabilityProbe::Bright;
            changed = true;
        }
        if self
            .ui
            .button(cx, ids!(readability_dark_button))
            .clicked(actions)
        {
            self.demo_mode = NativeDemoVisualMode::Readability;
            self.readability_probe = NativeDemoReadabilityProbe::Dark;
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
        if self.ui.button(cx, ids!(tone_button)).clicked(actions) {
            self.tone_mode = self.tone_mode.next();
            changed = true;
        }
        if self.ui.button(cx, ids!(reset_button)).clicked(actions) {
            self.reset_tuning();
            self.start_gold_glint_animation(cx);
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
        if self
            .ui
            .button(cx, ids!(glint_toggle_button))
            .clicked(actions)
        {
            self.makepad_demo_glint_visible = !self.makepad_demo_glint_visible;
            if self.makepad_demo_glint_visible {
                self.start_gold_glint_animation(cx);
            }
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
                    self.container_preset,
                    self.tint_alpha,
                    self.container_spacing,
                    self.main_radius,
                );
            log!(
                "[liquid-glass] standalone-native-example tuning {}",
                native_liquid_glass_status_text(
                    self.demo_mode,
                    self.morph_state,
                    self.container_preset,
                    self.readability_probe,
                    self.style_mode,
                    mode_tint_alpha,
                    mode_spacing,
                    mode_radius,
                    self.makepad_demo_glint_visible
                )
            );
            log!(
                "[liquid-glass] standalone-native-example control-resize-validation {}",
                native_liquid_glass_control_resize_validation_text(self.demo_mode)
            );
            self.configure_native_liquid_glass(cx);
        }
    }

    fn handle_next_frame(&mut self, cx: &mut Cx, event: &NextFrameEvent) {
        self.update_gold_glint_animation(cx, event);
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
        let tint = native_liquid_glass_clear_tint_for_tone(
            NativeDemoColorTone::Arctic,
            DEFAULT_CLEAR_TINT_ALPHA,
        )
        .expect("clear tint should be present");

        assert!(tint.w > 0.0);
        assert!(tint.w < 0.35);
    }

    #[test]
    fn standalone_native_glass_color_tones_change_tint_rgb_without_alpha_shift() {
        let arctic = native_liquid_glass_tint_for_mode(
            NativeDemoStyleMode::Clear,
            NativeDemoColorTone::Arctic,
            DEFAULT_CLEAR_TINT_ALPHA,
        )
        .expect("default tone tint should be present");
        let emerald = native_liquid_glass_tint_for_mode(
            NativeDemoStyleMode::Clear,
            NativeDemoColorTone::Emerald,
            DEFAULT_CLEAR_TINT_ALPHA,
        )
        .expect("emerald tone tint should be present");

        assert_ne!(arctic.x, emerald.x);
        assert_eq!(arctic.w, emerald.w);
        assert_eq!(
            NativeDemoColorTone::Arctic.next(),
            NativeDemoColorTone::Emerald
        );
        assert_eq!(
            NativeDemoColorTone::Violet.next(),
            NativeDemoColorTone::Aijiro
        );
        assert_eq!(
            NativeDemoColorTone::Usuhanazakura.next(),
            NativeDemoColorTone::Arctic
        );
    }

    #[test]
    fn standalone_native_glass_includes_japanese_blue_palette_tones() {
        let blues = NativeDemoColorTone::japanese_blue_palette();

        assert_eq!(blues.len(), 18);
        assert_eq!(blues[0], NativeDemoColorTone::Aijiro);
        assert_eq!(blues[1], NativeDemoColorTone::Usuao);
        assert_eq!(blues[2], NativeDemoColorTone::Sorairo);
        assert_eq!(blues[17], NativeDemoColorTone::Usuhanazakura);

        for tone in blues {
            let tint = native_liquid_glass_tint_for_mode(
                NativeDemoStyleMode::Clear,
                tone,
                DEFAULT_CLEAR_TINT_ALPHA,
            )
            .expect("japanese blue tone tint should be present");
            assert_eq!(tint.w, DEFAULT_CLEAR_TINT_ALPHA);
            assert!(tone.label().len() <= 14);
        }
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
                NativeDemoContainerPreset::Threshold,
                NativeDemoReadabilityProbe::Mixed,
                NativeDemoStyleMode::Clear,
                0.22,
                28.0,
                44.0,
                true,
            ),
            "Panels  Clear  tint 22%  spacing 28  radius 44  makepad-glint:on"
        );
    }

    #[test]
    fn standalone_native_glass_visual_modes_have_stable_labels() {
        assert_eq!(NativeDemoVisualMode::Panels.label(), "Panels");
        assert_eq!(NativeDemoVisualMode::Geometry.label(), "Geometry");
        assert_eq!(NativeDemoVisualMode::Morph.label(), "Morph");
        assert_eq!(NativeDemoVisualMode::Container.label(), "Container");
        assert_eq!(NativeDemoVisualMode::Debug.label(), "Debug");
        assert_eq!(NativeDemoVisualMode::Controls.label(), "Controls");
        assert_eq!(NativeDemoVisualMode::Readability.label(), "Readability");
    }

    #[test]
    fn standalone_native_glass_start_mode_env_supports_controls_probe() {
        assert_eq!(
            native_liquid_glass_start_mode_from_env_value(Some("debug")),
            NativeDemoVisualMode::Debug
        );
        assert_eq!(
            native_liquid_glass_start_mode_from_env_value(Some("container")),
            NativeDemoVisualMode::Container
        );
        assert_eq!(
            native_liquid_glass_start_mode_from_env_value(Some("geometry")),
            NativeDemoVisualMode::Geometry
        );
        assert_eq!(
            native_liquid_glass_start_mode_from_env_value(Some("controls")),
            NativeDemoVisualMode::Controls
        );
        assert_eq!(
            native_liquid_glass_start_mode_from_env_value(Some("readability")),
            NativeDemoVisualMode::Readability
        );
        assert_eq!(
            native_liquid_glass_start_mode_from_env_value(Some("unknown")),
            NativeDemoVisualMode::Panels
        );
        assert_eq!(
            native_liquid_glass_start_mode_from_env_value(None),
            NativeDemoVisualMode::Panels
        );
    }

    #[test]
    fn standalone_native_glass_container_presets_have_stable_spacing() {
        assert_eq!(NativeDemoContainerPreset::Near.label(), "Near");
        assert_eq!(NativeDemoContainerPreset::Threshold.label(), "Threshold");
        assert_eq!(NativeDemoContainerPreset::Far.label(), "Far");
        assert!(
            NativeDemoContainerPreset::Near.spacing()
                < NativeDemoContainerPreset::Threshold.spacing()
        );
        assert!(
            NativeDemoContainerPreset::Threshold.spacing()
                < NativeDemoContainerPreset::Far.spacing()
        );
    }

    #[test]
    fn standalone_native_glass_readability_probes_have_contrast_tokens() {
        assert_eq!(NativeDemoReadabilityProbe::Mixed.label(), "Mixed");
        assert_eq!(NativeDemoReadabilityProbe::Bright.label(), "Bright");
        assert_eq!(NativeDemoReadabilityProbe::Dark.label(), "Dark");

        let bright = native_liquid_glass_readability_tokens(NativeDemoReadabilityProbe::Bright);
        let dark = native_liquid_glass_readability_tokens(NativeDemoReadabilityProbe::Dark);
        let mixed = native_liquid_glass_readability_tokens(NativeDemoReadabilityProbe::Mixed);

        assert_ne!(bright.foreground, dark.foreground);
        assert_ne!(bright.separator, dark.separator);
        assert!(bright.foreground.w >= 0.88);
        assert!(dark.foreground.w >= 0.88);
        assert!(mixed.separator.w >= 0.42);
        assert!(mixed.probe_background.w < 1.0);
    }

    #[test]
    fn standalone_native_glass_visual_modes_adjust_effective_tuning() {
        let panels = native_liquid_glass_visual_mode_tuning(
            NativeDemoVisualMode::Panels,
            NativeDemoContainerPreset::Threshold,
            0.22,
            28.0,
            44.0,
        );
        let morph = native_liquid_glass_visual_mode_tuning(
            NativeDemoVisualMode::Morph,
            NativeDemoContainerPreset::Threshold,
            0.22,
            28.0,
            44.0,
        );
        let geometry = native_liquid_glass_visual_mode_tuning(
            NativeDemoVisualMode::Geometry,
            NativeDemoContainerPreset::Threshold,
            0.22,
            28.0,
            44.0,
        );
        let container = native_liquid_glass_visual_mode_tuning(
            NativeDemoVisualMode::Container,
            NativeDemoContainerPreset::Far,
            0.22,
            28.0,
            44.0,
        );
        let readability = native_liquid_glass_visual_mode_tuning(
            NativeDemoVisualMode::Readability,
            NativeDemoContainerPreset::Threshold,
            0.22,
            28.0,
            44.0,
        );

        assert_eq!(panels, (0.22, 28.0, 44.0));
        assert!(morph.1 > panels.1);
        assert!(morph.2 > panels.2);
        assert_eq!(geometry.1, panels.1);
        assert!(geometry.2 > panels.2);
        assert_eq!(container.1, NativeDemoContainerPreset::Far.spacing());
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
        let geometry = native_liquid_glass_visual_scene(
            NativeDemoVisualMode::Geometry,
            NativeDemoMorphState::Near,
        );
        let container = native_liquid_glass_visual_scene(
            NativeDemoVisualMode::Container,
            NativeDemoMorphState::Near,
        );
        let debug = native_liquid_glass_visual_scene(
            NativeDemoVisualMode::Debug,
            NativeDemoMorphState::Near,
        );
        let controls = native_liquid_glass_visual_scene(
            NativeDemoVisualMode::Controls,
            NativeDemoMorphState::Near,
        );

        assert_ne!(panels.top_panel_width, morph.top_panel_width);
        assert!(morph.bottom_panel_margin.bottom > panels.bottom_panel_margin.bottom);
        assert!(geometry.top_panel_width < panels.top_panel_width);
        assert!(geometry.top_panel_height > panels.top_panel_height);
        assert!(geometry.bottom_panel_width < panels.bottom_panel_width);
        assert_eq!(geometry.top_panel_title, "Geometry");
        assert_eq!(container.top_panel_title, "Container");
        assert!(container.top_panel_height > panels.top_panel_height);
        assert!(container.bottom_panel_margin.bottom > panels.bottom_panel_margin.bottom);
        assert_eq!(debug.top_panel_title, "Debug batch");
        assert!(controls.main_panel_margin.left > panels.main_panel_margin.left);
        assert_eq!(morph.top_panel_title, "Morph: Near");
    }

    #[test]
    fn standalone_native_glass_debug_summary_lists_descriptor_fields() {
        let summary = native_liquid_glass_debug_summary(
            NativeDemoVisualMode::Container,
            NativeDemoMorphState::Near,
            NativeDemoContainerPreset::Far,
            NativeDemoReadabilityProbe::Mixed,
            NativeDemoStyleMode::Clear,
            NativeDemoColorTone::Arctic,
            0.24,
            52.0,
            48.0,
            NATIVE_DEMO_PANEL_COUNT,
            true,
        );

        assert!(summary.line_1.contains("mode=Container: Far"));
        assert!(summary.line_1.contains("style=Clear"));
        assert!(summary.line_1.contains("tone=Arctic"));
        assert!(summary.line_1.contains("panels=3"));
        assert!(summary.line_1.contains("z=0/1/2"));
        assert!(summary.line_2.contains("tint=24%"));
        assert!(summary.line_2.contains("spacing=52"));
        assert!(summary.line_2.contains("radius=48"));
        assert!(summary.line_2.contains("makepad-glint:on"));
    }

    #[test]
    fn standalone_native_glass_v4_1_descriptor_coverage_is_explicit() {
        let coverage = native_liquid_glass_v4_1_descriptor_coverage();

        assert!(coverage.style);
        assert!(coverage.tint);
        assert!(coverage.shape);
        assert!(coverage.radius);
        assert!(coverage.z_order);
        assert!(coverage.hit_test_passthrough);
        assert!(coverage.container_spacing);
    }

    #[test]
    fn standalone_native_glass_container_mode_reports_selected_preset() {
        assert_eq!(
            native_liquid_glass_status_text(
                NativeDemoVisualMode::Container,
                NativeDemoMorphState::Near,
                NativeDemoContainerPreset::Threshold,
                NativeDemoReadabilityProbe::Mixed,
                NativeDemoStyleMode::Clear,
                0.22,
                NativeDemoContainerPreset::Threshold.spacing(),
                44.0,
                true,
            ),
            "Container: Threshold  Clear  tint 22%  spacing 28  radius 44  makepad-glint:on"
        );
    }

    #[test]
    fn standalone_native_glass_readability_status_reports_selected_probe() {
        assert_eq!(
            native_liquid_glass_status_text(
                NativeDemoVisualMode::Readability,
                NativeDemoMorphState::Near,
                NativeDemoContainerPreset::Threshold,
                NativeDemoReadabilityProbe::Bright,
                NativeDemoStyleMode::Clear,
                0.32,
                28.0,
                44.0,
                false,
            ),
            "Readability: Bright  Clear  tint 32%  spacing 28  radius 44  makepad-glint:off"
        );
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
        assert!(scene.primary_control_enabled);
    }

    #[test]
    fn standalone_native_glass_primary_enabled_toggle_text_tracks_state() {
        assert_eq!(
            native_liquid_glass_primary_enabled_toggle_text(true),
            "Disable Primary"
        );
        assert_eq!(
            native_liquid_glass_primary_enabled_toggle_text(false),
            "Enable Primary"
        );
    }

    #[test]
    fn standalone_native_glass_resize_validation_text_distinguishes_controls_mode() {
        assert_eq!(
            native_liquid_glass_control_resize_validation_text(NativeDemoVisualMode::Controls),
            "resize keeps 5 native controls"
        );
        assert_eq!(
            native_liquid_glass_control_resize_validation_text(NativeDemoVisualMode::Panels),
            "resize keeps native controls hidden outside Controls"
        );
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
        assert_ne!(
            near.bottom_panel_margin.bottom,
            overlap.bottom_panel_margin.bottom
        );
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

    #[test]
    fn standalone_native_glass_gold_glint_is_panel_edge_not_window_frame() {
        assert!(native_liquid_glass_gold_glint_edge_width() < 4.0);
        assert!(
            native_liquid_glass_gold_glint_strength(1.0)
                > native_liquid_glass_gold_glint_strength(0.0)
        );
        assert_eq!(
            native_liquid_glass_gold_glint_strength(9.0),
            native_liquid_glass_gold_glint_strength(1.0)
        );
    }

    #[test]
    fn standalone_native_glass_gold_glint_path_tracks_panel_edge() {
        let edge_width = native_liquid_glass_gold_glint_edge_width();
        let center_inset = native_liquid_glass_gold_glint_center_inset(edge_width);

        assert_eq!(center_inset, edge_width * 0.5);
        assert_eq!(
            native_liquid_glass_gold_glint_center_radius(44.0, edge_width),
            44.0 - center_inset
        );
    }

    #[test]
    fn standalone_native_glass_demo_glint_toggle_text_tracks_visibility() {
        assert_eq!(makepad_demo_glint_toggle_text(true), "Glint Off");
        assert_eq!(makepad_demo_glint_toggle_text(false), "Glint On");
    }

    #[test]
    fn standalone_native_glass_status_marks_makepad_demo_glint_overlay() {
        assert_eq!(makepad_demo_glint_status_label(true), "makepad-glint:on");
        assert_eq!(makepad_demo_glint_status_label(false), "makepad-glint:off");
    }
}
