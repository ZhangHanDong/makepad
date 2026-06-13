pub use makepad_code_editor;
// Linking the kit activates its `script_mod!` block, which registers
// `mod.widgets.DiagramView`. Without this `pub use`, the DSL can't resolve
// the template below.
pub use makepad_diagram_kit;
pub use makepad_widgets;

use makepad_ai::*;
use makepad_widgets::makepad_draw::svg::{
    collect_edges, collect_text_cmds, parse_svg, SvgDocument, SvgEdge, SvgTextAnchor, SvgTextCmd,
};
use makepad_widgets::makepad_platform::makepad_micro_serde::*;
use makepad_widgets::*;
use streaming_markdown_kit::{
    streaming_display_with_latex_autowrap_remend, wrap_bare_latex, SanitizeOptions,
};

app_main!(App);

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*
    use mod.widgets.CodeView
    use mod.widgets.DiagramView
    use mod.text.*
    use mod.res.*
    use mod.draw.*

    // Override theme fonts. Two purposes:
    //   1. font_code — CJK-capable monospace (LXGW Mono) so `` `inline` ``
    //      and CodeView render Chinese correctly.
    //   2. font_regular — add a symbols-capable latin (NotoSans) so Unicode
    //      blocks outside IBM Plex Sans's repertoire (arrows U+2190-U+21FF,
    //      math operators, misc technical) render as glyphs instead of tofu.
    //
    // Note: Makepad's Markdown widget bakes `theme.font_*` at expansion time,
    // so these theme-level overrides are necessary but not sufficient —
    // per-instance overrides on each Markdown instance are also applied below.
    mod.themes.dark = mod.themes.dark{
        font_code: TextStyle{
            font_size: theme.font_size_code
            font_family: FontFamily{
                latin := FontMember{res: crate_resource("self:resources/LiberationMono-Regular.ttf") asc: 0.0 desc: 0.0}
                chinese := FontMember{res: crate_resource("self:resources/LXGWWenKaiMono-Regular.ttf") asc: 0.0 desc: 0.0}
                symbols := FontMember{res: crate_resource("self:resources/NotoSans-Regular.ttf") asc: 0.0 desc: 0.0}
                emoji := FontMember{res: crate_resource("self:resources/NotoColorEmoji.ttf") asc: 0.0 desc: 0.0}
            }
            line_spacing: 1.35
        }
        font_regular: mod.themes.dark.font_regular{
            font_family: FontFamily{
                latin := FontMember{res: crate_resource("self:resources/NotoSans-Regular.ttf") asc: 0.0 desc: 0.0}
                chinese := FontMember{res: crate_resource("self:resources/LXGWWenKaiMono-Regular.ttf") asc: 0.0 desc: 0.0}
                symbols := FontMember{res: crate_resource("self:resources/NotoSans-Regular.ttf") asc: 0.0 desc: 0.0}
                emoji := FontMember{res: crate_resource("self:resources/NotoColorEmoji.ttf") asc: 0.0 desc: 0.0}
            }
        }
    }

    let ai_ink = #x06130F
    let ai_panel = #x0A3A30
    let ai_panel_deep = #x06251F
    let ai_cream = #xF3E3C7
    let ai_cream_dim = #xE0D2BACC
    let ai_cyan = #x72E4FF
    let ai_cyan_soft = #x72E4FF77
    let ai_gold = #xF6BE63
    let ai_gold_soft = #xF6BE6388

    let chat_scene_bg = Gradient{x1: 0 y1: 0 x2: 1 y2: 1
        Stop{offset: 0 color: #x071018 opacity: 0.78}
        Stop{offset: 0.44 color: #x121923 opacity: 0.74}
        Stop{offset: 0.72 color: #x18202B opacity: 0.70}
        Stop{offset: 1 color: #x201722 opacity: 0.74}
    }

    let chat_scene_cyan = RadGradient{cx: 0.14 cy: 0.16 r: 0.44
        Stop{offset: 0 color: #x72E4FF opacity: 0.70}
        Stop{offset: 0.44 color: #x2E84FF opacity: 0.20}
        Stop{offset: 1 color: #x2E84FF opacity: 0.0}
    }

    let chat_scene_gold = RadGradient{cx: 0.88 cy: 0.14 r: 0.36
        Stop{offset: 0 color: #xFFD18A opacity: 0.52}
        Stop{offset: 0.50 color: #xFF8F3A opacity: 0.15}
        Stop{offset: 1 color: #xFF8F3A opacity: 0.0}
    }

    let chat_scene_violet = RadGradient{cx: 0.64 cy: 0.88 r: 0.48
        Stop{offset: 0 color: #xDCA5FF opacity: 0.48}
        Stop{offset: 0.54 color: #x806DFF opacity: 0.14}
        Stop{offset: 1 color: #x806DFF opacity: 0.0}
    }

    let chat_scene_mint = RadGradient{cx: 0.28 cy: 0.76 r: 0.38
        Stop{offset: 0 color: #x8AFFD1 opacity: 0.42}
        Stop{offset: 0.48 color: #x2BD7B7 opacity: 0.12}
        Stop{offset: 1 color: #x2BD7B7 opacity: 0.0}
    }

    let ChatSceneVector = Vector{
        width: Fill
        height: Fill
        viewbox: vec4(0 0 1200 820)

        Circle{cx: 160 cy: 112 r: 350 fill: chat_scene_cyan}
        Circle{cx: 1080 cy: 112 r: 290 fill: chat_scene_gold}
        Circle{cx: 768 cy: 790 r: 390 fill: chat_scene_violet}
        Circle{cx: 320 cy: 650 r: 300 fill: chat_scene_mint}

        Rect{x: 24 y: 28 w: 1152 h: 760 rx: 38 ry: 38 fill: #x07101822}
        Rect{x: 24 y: 28 w: 1152 h: 760 rx: 38 ry: 38 fill: false stroke: #xFFFFFF1A stroke_width: 1.2}
        Rect{x: 28 y: 32 w: 1144 h: 752 rx: 36 ry: 36 fill: false stroke: #x72E4FF20 stroke_width: 1.0}
        Rect{x: 42 y: 44 w: 1116 h: 724 rx: 32 ry: 32 fill: false stroke: #xFFD18A10 stroke_width: 0.8}

        Path{d: "M -80 190 C 170 72 330 120 520 70 S 905 20 1280 110" fill: false stroke: #x72E4FF26 stroke_width: 2.6 stroke_linecap: "round"}
        Path{d: "M -60 610 C 160 500 348 548 548 480 S 900 380 1260 475" fill: false stroke: #xDCA5FF20 stroke_width: 2.2 stroke_linecap: "round"}
        Path{d: "M 1120 -40 C 960 156 900 286 730 374 S 470 528 248 878" fill: false stroke: #xFFD18A1C stroke_width: 2.0 stroke_linecap: "round"}

        Rect{x: 92 y: 74 w: 320 h: 118 rx: 34 ry: 34 fill: #xFFFFFF05}
        Rect{x: 850 y: 84 w: 244 h: 88 rx: 30 ry: 30 fill: #xFFFFFF06}
        Rect{x: 470 y: 612 w: 330 h: 118 rx: 34 ry: 34 fill: #xFFFFFF05}
    }

    let ToolbarLabel = Label {
        draw_text.color: ai_cream_dim
        draw_text.text_style.font_size: 11
    }

    let ToolbarGlass = GlassPanel {
        height: 38
        flow: Right
        align: Align{y: 0.5}
        spacing: 8
        padding: Inset{left: 12 right: 12 top: 0 bottom: 0}
        draw_bg +: {
            tint_color: #x06231C
            tint_alpha: 0.88
            border_color: #x72E4FF
            border_alpha: 0.24
            border_width: 1.0
            corner_radius: 18.0
            halo_strength: 0.0
            halo_radius: 0.0
            highlight_strength: 0.10
            highlight_band_height: 18.0
            noise_strength: 0.003
        }
    }

    let ComposerGlass = AppleGlassRoundedView {
        width: Fill{min: 620 max: 1040}
        height: Fit
        new_batch: true
        flow: Down
        padding: Inset{left: 18 top: 14 right: 14 bottom: 12}
        spacing: 10
        draw_bg +: {
            blur_level: 5.2
            lensing_effect: 0.96
            lensing_strength: 38.0
            lensing_width: 18.0
            corner_radius: 24.0
            tint_color: #xB8B8B8
            tint_alpha: 0.08
            surface_alpha: 0.76
            border_color: #xEAD8B8
            border_alpha: 0.56
            border_width: 1.0
            specular_strength: 0.20
            fallback_color: #x123A34
            shadow_color: #x000000CC
            shadow_radius: 46.0
            shadow_offset: vec2(0.0, 20.0)
            diffraction_strength: 5.6
        }
    }

    let PillButton = ButtonFlat {
        height: 34
        padding: Inset{left: 14 right: 14 top: 0 bottom: 0}
        draw_text +: {
            color: ai_cream
            text_style +: { font_size: 11 }
        }
        draw_bg +: {
            color: #x08251EB8
            color_hover: #x123B31DD
            border_color: #xEAD8B82D
            border_size: 1.0
            border_radius: 10.0
        }
    }

    let IconButton = ButtonFlat {
        width: 36
        height: 36
        padding: 0
        draw_text +: {
            color: ai_cream
            text_style +: { font_size: 15 }
        }
        draw_bg +: {
            color: #x08251EB0
            color_hover: #x154337DD
            border_color: #xEAD8B82A
            border_size: 1.0
            border_radius: 10.0
        }
    }

    let SendButton = ButtonFlat {
        width: 44
        height: 44
        padding: 0
        draw_text +: {
            color: ai_ink
            text_style +: { font_size: 26 }
        }
        draw_bg +: {
            hover: instance(0.0)
            down: instance(0.0)
            focus: instance(0.0)
            disabled: instance(0.0)
            color: ai_gold
            color_hover: #xFFD98B
            border_color: #xFFF0D277
            border_size: 1.0
            border_radius: 10.0
            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                let r = min(self.rect_size.x, self.rect_size.y) * 0.5 - 3.0
                let center = self.rect_size * 0.5
                let p = self.pos - vec2(0.5, 0.5)
                let radial = clamp(length(p) * 2.0, 0.0, 1.0)
                let top_highlight = clamp(1.0 - self.pos.y * 2.2, 0.0, 1.0)
                let lower_shadow = smoothstep(0.35, 1.0, self.pos.y)
                let paper_noise = (
                    Math.random_2d(self.pos * self.rect_size * 0.42)
                    + Math.random_2d(self.pos * self.rect_size * 1.3) * 0.35
                    - 0.68
                ) * 0.045
                let fill = self.color
                    .mix(self.color_focus, self.focus)
                    .mix(self.color_hover, self.hover)
                    .mix(self.color_down, self.down)
                    .mix(self.color_disabled, self.disabled)
                let glass_fill = vec4(
                    fill.rgb
                        + vec3(0.20, 0.13, 0.04) * top_highlight
                        - vec3(0.18, 0.11, 0.04) * lower_shadow
                        - vec3(0.06, 0.04, 0.02) * radial
                        + paper_noise,
                    fill.a
                )

                sdf.circle(center.x + 0.8, center.y + 1.4, r + 1.5)
                sdf.fill(#x3A241370)

                sdf.circle(center.x, center.y, r + 1.8)
                sdf.fill_keep(#xA86F35)
                sdf.stroke(#xF6D99A88, 1.0)

                sdf.circle(center.x, center.y, r)
                sdf.fill_keep(glass_fill)
                sdf.stroke(#xF9D58AAA, 1.2)

                sdf.circle(center.x - r * 0.18, center.y - r * 0.24, r * 0.46)
                sdf.stroke(#xFFF3CF48, 0.8)
                return sdf.result
            }
        }
    }

    let GlassSlider = SliderMinimal {
        width: 170
        height: 28
        text: ""
        min: 0.72
        max: 0.98
        step: 0.01
        default: 0.90
        precision: 2
        label_walk: Walk{width: 0 height: 0}
        text_input: TextInput{
            width: 0
            height: 0
            is_read_only: true
        }
        draw_bg +: {
            hover: instance(0.0)
            focus: instance(0.0)
            drag: instance(0.0)
            disabled: instance(0.0)
            border_size: 0.0
            offset_y: 11.0
            handle_size: 20.0
            color: #x9CC9C24A
            color_hover: #x9CC9C266
            color_focus: #x9CC9C266
            color_drag: #x9CC9C280
            color_2: #x0A241EAA
            color_2_hover: #x0E3028CC
            color_2_focus: #x0E3028CC
            color_2_drag: #x123C32DD
            val_color: ai_gold
            val_color_hover: #xFFD98B
            val_color_focus: #xFFD98B
            val_color_drag: #xFFE2A3
            handle_color: ai_gold
            handle_color_hover: #xFFF0D2
            handle_color_focus: #xFFF0D2
            handle_color_drag: #xFFF0D2
            border_color: #x72E4FF44
            border_color_2: #x00000055
            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                let track_y = self.rect_size.y * 0.5 - 2.0
                let track_h = 4.0
                let handle_x = clamp(
                    self.slide_pos * self.rect_size.x,
                    8.0,
                    self.rect_size.x - 8.0
                )
                let handle_r = 8.0 + self.hover * 1.0

                sdf.box(0.0, track_y, self.rect_size.x, track_h, 2.0)
                sdf.fill(#x6EA99E66)

                sdf.box(0.0, track_y, handle_x, track_h, 2.0)
                sdf.fill(self.val_color.mix(self.val_color_hover, self.hover))

                sdf.circle(handle_x, self.rect_size.y * 0.5, handle_r)
                sdf.fill_keep(self.handle_color.mix(self.handle_color_hover, self.hover))
                sdf.stroke(#xFFF0D288, 1.0)

                return sdf.result
            }
        }
    }

    let MermaidSvgView = #(MermaidSvgView::register_widget(vm)) {
        width: Fill
        height: Fit
        // Animated flow dot shader: SDF circle + halo. Per-edge color
        // (incl. pulse alpha in `.w`) is written from Rust.
        draw_flow_dot +: {
            color: #xe2e8f0
            pixel: fn() {
                let r = length(self.pos - vec2(0.5, 0.5))
                let core = 1.0 - smoothstep(0.30, 0.38, r)
                let halo = (1.0 - smoothstep(0.38, 0.50, r)) * 0.55
                let a = clamp(core + halo, 0.0, 1.0) * self.color.w
                return Pal.premul(vec4(self.color.xyz, a))
            }
        }
        draw_text +: {
            color: #xe2e8f0
            text_style: theme.font_code{
                font_size: 12
                font_family: FontFamily{
                    latin := FontMember{res: crate_resource("self:resources/LiberationMono-Regular.ttf") asc: 0.0 desc: 0.0}
                    chinese := FontMember{res: crate_resource("self:resources/LXGWWenKaiMono-Regular.ttf") asc: 0.0 desc: 0.0}
                    emoji := FontMember{res: crate_resource("self:resources/NotoColorEmoji.ttf") asc: 0.0 desc: 0.0}
                }
            }
        }
    }

    let ChatList = #(ChatList::register_widget(vm)) {
        width: Fill
        height: Fill

        list := PortalList {
            width: Fill
            height: Fill
            flow: Down
            drag_scrolling: false
            auto_tail: true
            smooth_tail: true
            selectable: true

            User := RoundedView {
                width: Fill
                height: Fit
                margin: Inset{top: 4 bottom: 4 left: 50 right: 8}
                padding: Inset{left: 12 top: 8 right: 12 bottom: 8}
                flow: Overlay
                show_bg: true
                draw_bg +: {
                    color: #x0B2A22E6
                    radius: 12.0
                }

                selectable := Markdown {
                    width: Fill
                    height: Fit
                    selectable: true
                    use_code_block_widget: true
                    use_math_widget: true
                    body: ""
                    // Per-instance override for `` `inline code` ``. The
                    // Markdown widget bakes `theme.font_code` at expansion
                    // time, so a later `mod.themes.dark{...}` override
                    // doesn't reach it. Without this override, CJK inside
                    // backticks renders as tofu (no glyph) because Liberation
                    // Mono is Latin-only.
                    text_style_fixed: theme.font_code{
                        font_family: FontFamily{
                            latin := FontMember{res: crate_resource("self:resources/LiberationMono-Regular.ttf") asc: 0.0 desc: 0.0}
                            chinese := FontMember{res: crate_resource("self:resources/LXGWWenKaiMono-Regular.ttf") asc: 0.0 desc: 0.0}
                            symbols := FontMember{res: crate_resource("self:resources/NotoSans-Regular.ttf") asc: 0.0 desc: 0.0}
                            emoji := FontMember{res: crate_resource("self:resources/NotoColorEmoji.ttf") asc: 0.0 desc: 0.0}
                        }
                    }
                    // Prose font family with symbols fallback — fixes "tofu"
                    // for Unicode arrows / math / misc technical symbols
                    // (observed trigger: `1→5`, `≤`, `≥`, `α` in prose).
                    text_style_normal: theme.font_regular{
                        font_family: FontFamily{
                            latin := FontMember{res: crate_resource("self:resources/NotoSans-Regular.ttf") asc: 0.0 desc: 0.0}
                            chinese := FontMember{res: crate_resource("self:resources/LXGWWenKaiMono-Regular.ttf") asc: 0.0 desc: 0.0}
                            symbols := FontMember{res: crate_resource("self:resources/NotoSans-Regular.ttf") asc: 0.0 desc: 0.0}
                            emoji := FontMember{res: crate_resource("self:resources/NotoColorEmoji.ttf") asc: 0.0 desc: 0.0}
                        }
                    }
                    code_block := ScrollXView {
                        width: Fill
                        height: Fit
                        flow: Right
                        code_view := CodeView {
                            keep_cursor_at_end: false
                            editor +: {
                                height: Fit
                                draw_bg +: { color: #x031510EE }
                            }
                        }
                    }
                    splash_block := View {
                        width: Fill
                        height: Fit
                        splash_view := Splash {
                            width: Fill
                            height: Fit
                        }
                    }
                    // Diagram block — rendered by makepad-diagram-kit's
                    // DiagramView. The inner `diagram_view` id matches what
                    // the markdown widget's `ids!(diagram_view).set_text`
                    // dispatch expects.
                    diagram_block := ScrollXView {
                        width: Fill
                        height: Fit
                        flow: Right
                        diagram_view := DiagramView {
                            width: Fit
                            height: Fit
                        }
                    }
                    mermaid_block := ScrollXView {
                        width: Fill
                        height: Fit
                        flow: Right
                        mermaid_view := MermaidSvgView {
                            width: Fit
                            height: Fit
                        }
                    }
                    inline_math := MathView {
                        font_size: 13.0
                    }
                    display_math := MathView {
                        font_size: 15.0
                    }
                }

                View {
                    width: Fill
                    height: Fit
                    align: Align{x: 1.0}
                    delete_button := ButtonFlat {
                        width: Fit
                        height: Fit
                        padding: Inset{top: 2 bottom: 2 left: 6 right: 6}
                        margin: Inset{top: 2 right: 2}
                        text: "x"
                        draw_text +: {
                            color: #888
                            text_style +: { font_size: 9 }
                        }
                    }
                }
            }

            Assistant := RoundedView {
                width: Fill
                height: Fit
                margin: Inset{top: 4 bottom: 4 left: 8 right: 50}
                padding: Inset{left: 12 top: 8 right: 12 bottom: 8}
                flow: Overlay
                show_bg: true
                draw_bg +: {
                    color: #x0B2A22E6
                    radius: 12.0
                }

                RubberView {
                    width: Fill
                    height: Fit
                    smoothing: 0.3

                    selectable := Markdown {
                        width: Fill
                        height: Fit
                        selectable: true
                        use_code_block_widget: true
                        use_math_widget: true
                        body: ""
                        // Per-instance override — same as User's Markdown
                        // above. Fixes `` `中文` `` inline-code tofu.
                        text_style_fixed: theme.font_code{
                            font_family: FontFamily{
                                latin := FontMember{res: crate_resource("self:resources/LiberationMono-Regular.ttf") asc: 0.0 desc: 0.0}
                                chinese := FontMember{res: crate_resource("self:resources/LXGWWenKaiMono-Regular.ttf") asc: 0.0 desc: 0.0}
                                emoji := FontMember{res: crate_resource("self:resources/NotoColorEmoji.ttf") asc: 0.0 desc: 0.0}
                            }
                        }
                        draw_text +: {
                            get_color: fn() {
                                let fade_chars = 50.0
                                let dist_from_end = self.total_chars - self.char_index
                                let t = clamp(dist_from_end / fade_chars, 0.0, 1.0)
                                let alpha = pow(t, 0.5)
                                return vec4(self.color.rgb, self.color.a * alpha)
                            }
                        }
                        code_block := ScrollXView {
                            width: Fill
                            height: Fit
                            flow: Right
                            code_view := CodeView {
                                keep_cursor_at_end: true
                                editor +: {
                                    height: Fit
                                    draw_bg +: { color: #x031510EE }
                                    // Local font override: CodeView is defined in the
                                    // makepad-code-editor crate and bakes `theme.font_code`
                                    // at its own expansion time, so later `mod.themes.dark`
                                    // overrides don't reach it. Override per-instance.
                                    draw_text +: {
                                        text_style: theme.font_code{
                                            font_family: FontFamily{
                                                latin := FontMember{res: crate_resource("self:resources/LiberationMono-Regular.ttf") asc: 0.0 desc: 0.0}
                                                chinese := FontMember{res: crate_resource("self:resources/LXGWWenKaiMono-Regular.ttf") asc: 0.0 desc: 0.0}
                                                emoji := FontMember{res: crate_resource("self:resources/NotoColorEmoji.ttf") asc: 0.0 desc: 0.0}
                                            }
                                        }
                                    }
                                    draw_gutter +: {
                                        text_style: theme.font_code{
                                            font_family: FontFamily{
                                                latin := FontMember{res: crate_resource("self:resources/LiberationMono-Regular.ttf") asc: 0.0 desc: 0.0}
                                                chinese := FontMember{res: crate_resource("self:resources/LXGWWenKaiMono-Regular.ttf") asc: 0.0 desc: 0.0}
                                                emoji := FontMember{res: crate_resource("self:resources/NotoColorEmoji.ttf") asc: 0.0 desc: 0.0}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        splash_block := SolidView{
                            flow: Overlay
                            new_batch: true
                            width: Fill
                            height: Fit
                            splash_view := Splash {
                                flow: Overlay
                                width: Fill
                                height: Fit
                            }
                        }
                        // Diagram block — see User-side comment.
                        diagram_block := ScrollXView{
                            flow: Right
                            new_batch: true
                            width: Fill
                            height: Fit
                            diagram_view := DiagramView {
                                width: Fit
                                height: Fit
                            }
                        }
                        mermaid_block := ScrollXView{
                            flow: Right
                            new_batch: true
                            width: Fill
                            height: Fit
                            mermaid_view := MermaidSvgView {
                                width: Fit
                                height: Fit
                            }
                        }
                        inline_math := MathView {
                            font_size: 13.0
                        }
                        display_math := MathView {
                            font_size: 15.0
                        }
                    }
                }

                View {
                    width: Fill
                    height: Fit
                    align: Align{x: 1.0}
                    copy_button := ButtonFlat {
                        width: Fit
                        height: Fit
                        padding: Inset{top: 2 bottom: 2 left: 6 right: 6}
                        margin: Inset{top: 2 right: 2}
                        text: "copy"
                        draw_text +: {
                            color: #888
                            text_style +: { font_size: 9 }
                        }
                    }
                    delete_button := ButtonFlat {
                        width: Fit
                        height: Fit
                        padding: Inset{top: 2 bottom: 2 left: 6 right: 6}
                        margin: Inset{top: 2 right: 2}
                        text: "x"
                        draw_text +: {
                            color: #888
                            text_style +: { font_size: 9 }
                        }
                    }
                }
            }
        }
    }

    startup() do #(App::script_component(vm)){
        ui: Root{
            main_window := Window{
                show_caption_bar: false
                pass.clear_color: #00000000
                window.transparent: true
                // window.backdrop: WindowBackdrop.Blur — disabled until
                // platform bug fixed in macos_window.rs:532 (addSubview
                // positioned arg must be NSWindowBelow/-1 or NSWindowAbove/1,
                // not 0). See issues/aichat-liquid-glass-backdrop-platform-bug.md
                window.macos: MacosWindowConfig{chrome: MacosWindowChrome.Borderless resizable: true}
                window.inner_size: vec2(900, 700)
                window.title: " "
                body +: {
                    flow: Overlay
                    padding: 3
                    spacing: 0
                    draw_bg.color: #00000000

                    app_shell := GlassPanel {
                        width: Fill
                        height: Fill
                        new_batch: true
                        flow: Right
                        padding: Inset{left: 20 top: 20 right: 20 bottom: 20}
                        spacing: 0
                        draw_bg +: {
                            tint_color: #x0D4035
                            tint_alpha: 0.64
                            border_color: ai_cyan
                            border_alpha: 0.38
                            border_width: 1.0
                            corner_radius: 30.0
                            halo_color: ai_cyan
                            halo_strength: 0.0
                            halo_radius: 0.0
                            highlight_strength: 0.28
                            highlight_band_height: 58.0
                            chroma_strength: 0.0
                            noise_strength: 0.004
                        }

                    sidebar := GlassPanel {
                        width: 298
                        height: Fill
                        new_batch: true
                        flow: Down
                        padding: Inset{left: 14 top: 14 right: 14 bottom: 14}
                        spacing: 10
                        draw_bg +: {
                            tint_color: #x0A3A30
                            tint_alpha: 0.62
                            border_color: #xEAD8B8
                            border_alpha: 0.20
                            border_width: 0.0
                            corner_radius: 22.0
                            halo_strength: 0.0
                            halo_radius: 0.0
                            highlight_strength: 0.16
                            highlight_band_height: 54.0
                            chroma_strength: 0.0
                            noise_strength: 0.004
                        }

                        sidebar_header := View {
                            width: Fill
                            height: Fit
                            flow: Down
                            spacing: 8
                            margin: Inset{top: 4 bottom: 18}

                            View {
                                width: Fill
                                height: Fit
                                flow: Right
                                spacing: 10
                                align: Align{y: 0.5}

                                Label {
                                    text: "AI"
                                    draw_text.color: ai_cyan
                                    draw_text.text_style.font_size: 14
                                }

                                Label {
                                    text: "AI Chat"
                                    draw_text.color: ai_cream
                                    draw_text.text_style.font_size: 15
                                }
                            }

                            sidebar_subtitle := Label {
                                text: "Diagram workspace"
                                draw_text.color: ai_cream_dim
                                draw_text.text_style.font_size: 11
                            }
                        }

                        nav_chat := ButtonFlat {
                            width: Fill
                            height: 38
                            text: "●  会话"
                            align: Align{x: 0.0 y: 0.5}
                            padding: Inset{left: 14 right: 12}
                            draw_text +: {
                                color: ai_cream
                                text_style +: { font_size: 12 }
                            }
                            draw_bg +: {
                                color: #x0B6B67AA
                                color_hover: #x108E88CC
                                border_color: #x72E4FF66
                                border_size: 1.0
                                border_radius: 10.0
                            }
                        }

                        nav_appgen := ButtonFlat {
                            width: Fill
                            height: 38
                            text: "◇  App 生成"
                            align: Align{x: 0.0 y: 0.5}
                            padding: Inset{left: 14 right: 12}
                            draw_text +: {
                                color: ai_cream_dim
                                text_style +: { font_size: 12 }
                            }
                            draw_bg +: {
                                color: #x00000000
                                color_hover: #xEAD8B814
                                border_color: #x00000000
                                border_size: 1.0
                                border_radius: 10.0
                            }
                        }

                        nav_search := ButtonFlat {
                            width: Fill
                            height: 30
                            text: "⌕  搜索"
                            align: Align{x: 0.0 y: 0.5}
                            padding: Inset{left: 4 right: 4}
                            draw_text +: {
                                color: #xE4D4B6
                                text_style +: { font_size: 12 }
                            }
                            draw_bg +: {
                                color: #00000000
                                color_hover: #xEAD8B814
                                border_size: 0.0
                                border_radius: 8.0
                            }
                        }

                        nav_plugins := ButtonFlat {
                            width: Fill
                            height: 30
                            text: "⌘  插件"
                            align: Align{x: 0.0 y: 0.5}
                            padding: Inset{left: 4 right: 4}
                            draw_text +: {
                                color: #xE4D4B6
                                text_style +: { font_size: 12 }
                            }
                            draw_bg +: {
                                color: #00000000
                                color_hover: #xEAD8B814
                                border_size: 0.0
                                border_radius: 8.0
                            }
                        }

                        nav_automation := ButtonFlat {
                            width: Fill
                            height: 30
                            text: ">  自动化"
                            align: Align{x: 0.0 y: 0.5}
                            padding: Inset{left: 4 right: 4}
                            draw_text +: {
                                color: #xE4D4B6
                                text_style +: { font_size: 12 }
                            }
                            draw_bg +: {
                                color: #00000000
                                color_hover: #xEAD8B814
                                border_size: 0.0
                                border_radius: 8.0
                            }
                        }

                        nav_project := ButtonFlat {
                            width: Fill
                            height: 30
                            text: "#  项目"
                            align: Align{x: 0.0 y: 0.5}
                            padding: Inset{left: 4 right: 4}
                            draw_text +: {
                                color: #xE4D4B6
                                text_style +: { font_size: 12 }
                            }
                            draw_bg +: {
                                color: #00000000
                                color_hover: #xEAD8B814
                                border_size: 0.0
                                border_radius: 8.0
                            }
                        }

                        Label {
                            text: "对话"
                            margin: Inset{top: 28 bottom: 2 left: 0 right: 0}
                            draw_text.color: #xCDBF9FA0
                            draw_text.text_style.font_size: 12
                        }

                        Label {
                            text: "暂无聊天"
                            draw_text.color: #xCDBF9F55
                            draw_text.text_style.font_size: 12
                        }

                        View { width: Fill height: Fill }

                        settings_button := ButtonFlat {
                            width: Fill
                            height: 32
                            text: "*  设置"
                            align: Align{x: 0.0 y: 0.5}
                            padding: Inset{left: 4 right: 4}
                            draw_text +: {
                                color: #xF3E3C7
                                text_style +: { font_size: 12 }
                            }
                            draw_bg +: {
                                color: #00000000
                                color_hover: #xEAD8B814
                                border_size: 0.0
                                border_radius: 8.0
                            }
                        }
                    }

                    SolidView {
                        width: 1
                        height: Fill
                        draw_bg.color: #xEAD8B81E
                    }

                    main_area := GlassPanel {
                        width: Fill
                        height: Fill
                        new_batch: true
                        flow: Down
                        padding: Inset{left: 34 top: 18 right: 34 bottom: 22}
                        spacing: 12
                        draw_bg +: {
                            tint_color: #x0B3B31
                            tint_alpha: 0.58
                            border_color: #xEAD8B8
                            border_alpha: 0.16
                            border_width: 0.0
                            corner_radius: 22.0
                            halo_strength: 0.0
                            halo_radius: 0.0
                            highlight_strength: 0.16
                            highlight_band_height: 56.0
                            chroma_strength: 0.0
                            noise_strength: 0.004
                        }

                        top_bar := View {
                            width: Fill
                            height: 40
                            flow: Right
                            align: Align{y: 0.5}

                            workspace_title := Label {
                                text: "AI Chat"
                                draw_text.color: ai_cream
                                draw_text.text_style.font_size: 14
                            }

                            View { width: Fill height: 1 }
                        }

                        chat_shell := View {
                            width: Fill
                            height: Fill
                            flow: Overlay

                            empty_state := View {
                                width: Fill
                                height: Fill
                                flow: Down
                                align: Align{x: 0.5 y: 0.46}
                                spacing: 18

                                empty_title := Label {
                                    text: "我们该做什么？"
                                    draw_text.color: #xF3E3C7
                                    draw_text.text_style.font_size: 27
                                }

                                empty_subtitle := Label {
                                    text: "输入自然语言，生成可交互的 Makepad diagram。"
                                    draw_text.color: #xCDBF9FAA
                                    draw_text.text_style.font_size: 12
                                }
                            }

                            chat_list := ChatList {}
                        }

                        status_label := Label {
                            width: Fill
                            height: Fit
                            text: "Initializing..."
                            margin: Inset{left: 92 right: 92 top: 4 bottom: 0}
                            draw_text.text_style.font_size: 10
                            draw_text.color: #xE2D2B9CC
                        }

                        composer_row := View {
                            width: Fill
                            height: Fit
                            align: Align{x: 0.5 y: 0.0}

                            composer := ComposerGlass {
                                input := TextInput {
                                    width: Fill
                                    height: 56
                                    empty_text: "问任何事。输入 @ 使用插件或提及文件"
                                    draw_bg +: {
                                        color: #00000000
                                        color_hover: #00000000
                                        color_focus: #00000000
                                        border_size: 0.0
                                        border_radius: 0.0
                                    }
                                    // Per-instance font override — TextInput bakes
                                    // `theme.font_regular` at DSL-expansion time, same
                                    // issue as Markdown/CodeView. Without this the
                                    // input box shows tofu for CJK and U+2192 arrows.
                                    draw_text +: {
                                        color: ai_cream
                                        color_empty: ai_cream_dim
                                        text_style: theme.font_regular{
                                            line_spacing: theme.font_wdgt_line_spacing
                                            font_size: 13
                                            font_family: FontFamily{
                                                latin := FontMember{res: crate_resource("self:resources/NotoSans-Regular.ttf") asc: 0.0 desc: 0.0}
                                                chinese := FontMember{res: crate_resource("self:resources/LXGWWenKaiMono-Regular.ttf") asc: 0.0 desc: 0.0}
                                                symbols := FontMember{res: crate_resource("self:resources/NotoSans-Regular.ttf") asc: 0.0 desc: 0.0}
                                                emoji := FontMember{res: crate_resource("self:resources/NotoColorEmoji.ttf") asc: 0.0 desc: 0.0}
                                            }
                                        }
                                    }
                                }

                                composer_actions := View {
                                    width: Fill
                                    height: Fit
                                    flow: Right
                                    align: Align{y: 0.5}
                                    spacing: 8

                                    attach_button := IconButton { text: "+" }
                                    mention_button := IconButton { text: "@" }
                                    tools_button := IconButton { text: "⌘" }

                                    Label {
                                        text: "默认权限"
                                        draw_text.color: ai_cream_dim
                                        draw_text.text_style.font_size: 11
                                    }

                                    thinking_toggle := ToggleFlat {
                                        text: "Thinking"
                                        active: false
                                        draw_text +: {
                                            color: ai_cream_dim
                                            text_style +: { font_size: 11 }
                                        }
                                    }

                                    View { width: Fill height: 1 }

                                    cancel_button := ButtonFlat {
                                        text: "Cancel"
                                        width: 72
                                        height: 32
                                        visible: false
                                        draw_text +: {
                                            color: #xF2F4F8
                                            text_style +: { font_size: 11 }
                                        }
                                        draw_bg +: {
                                            color: #x4B332FCC
                                            color_hover: #x64413ADD
                                            border_color: #xEAD8B818
                                            border_size: 1.0
                                            border_radius: 10.0
                                        }
                                    }

                                    clear_button := PillButton {
                                        text: "Clear"
                                        width: 78
                                        height: 36
                                        draw_bg +: {
                                            color: #x08251EC8
                                            color_hover: #x123B31EE
                                            border_color: #xEAD8B83A
                                            border_size: 1.0
                                            border_radius: 10.0
                                        }
                                    }

                                    send_button := SendButton {
                                        text: "↑"
                                    }
                                }
                            }
                        }
                    }
                    }

                    toolbar_overlay := PopupNotification{
                        align: Align{x: 1.0 y: 0.0}
                        content +: {
                            width: Fit
                            height: 48
                            margin: Inset{top: 35 right: 34}
                            flow: Right
                            spacing: 12
                            clip_x: false
                            clip_y: false

                            backend_toolbar := ToolbarGlass {
                                width: 286

                                ToolbarLabel {
                                    text: "Backend"
                                    width: 76
                                }

                                backend_dropdown := DropDown {
                                    width: Fill
                                    height: 30
                                    popup_menu_position: PopupMenuPosition.BelowInput
                                    labels: ["Claude Code" "Claude Splash" "Claude (ACP)" "Claude (API)" "Gemini" "Gemini Splash" "OpenAI" "Moonshot"]
                                    popup_menu: PopupMenuFlat{
                                        width: 170
                                        padding: Inset{left: 4 right: 4 top: 4 bottom: 4}
                                        draw_bg +: {
                                            color: #x06231CF2
                                            border_color: #x72E4FF38
                                            border_size: 1.0
                                            border_radius: 12.0
                                        }
                                        menu_item: PopupMenuItem{
                                            height: 26
                                            padding: Inset{left: 18 right: 10 top: 0 bottom: 0}
                                            draw_text +: {
                                                color: ai_cream
                                                color_hover: #xFFF0D2
                                                color_active: ai_cream
                                                text_style +: { font_size: 11 }
                                            }
                                            draw_bg +: {
                                                color: #x00000000
                                                color_hover: #x123B31DD
                                                color_active: #xEAD8B82D
                                                border_color: #x00000000
                                                border_color_hover: #x72E4FF22
                                                border_color_active: #x72E4FF44
                                                border_size: 1.0
                                                border_radius: 6.0
                                                mark_color_active: ai_gold
                                            }
                                        }
                                    }
                                    draw_text +: {
                                        color: ai_cream
                                        text_style +: { font_size: 11 }
                                    }
                                    draw_bg +: {
                                        color: #x08251ED8
                                        color_hover: #x12382FEE
                                        border_color: #xEAD8B832
                                        border_size: 1.0
                                        border_radius: 15.0
                                        arrow_color: ai_cream
                                    }
                                }
                            }

                            opacity_toolbar := ToolbarGlass {
                                width: 318

                                ToolbarLabel {
                                    text: "Glass"
                                    width: 54
                                }

                                opacity_slider := GlassSlider {}

                                opacity_value := Label {
                                    width: 42
                                    text: "90%"
                                    margin: Inset{left: 4}
                                    draw_text.color: ai_cream_dim
                                    draw_text.text_style.font_size: 11
                                }
                            }
                        }
                    }

                    resize_grip := Vector{
                        width: 34
                        height: 34
                        margin: Inset{right: 18 bottom: 18}
                        align: Align{x: 1.0 y: 1.0}
                        viewbox: vec4(0 0 34 34)
                        Path{d: "M 18 28 L 28 18" fill: false stroke: #xEAD8B8AA stroke_width: 1.5 stroke_linecap: "round"}
                        Path{d: "M 12 28 L 28 12" fill: false stroke: #xF3E3C788 stroke_width: 1.2 stroke_linecap: "round"}
                        Path{d: "M 24 28 L 28 24" fill: false stroke: #x9F7E4BAA stroke_width: 1.5 stroke_linecap: "round"}
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ActiveWorkspace {
    #[default]
    Chat,
    AppGen,
}

impl ActiveWorkspace {
    fn save_path(self) -> &'static str {
        match self {
            Self::Chat => CHAT_SAVE_PATH,
            Self::AppGen => APP_GEN_SAVE_PATH,
        }
    }

    fn prompt_section(self) -> &'static str {
        match self {
            Self::Chat => "User",
            Self::AppGen => "App Generation Request",
        }
    }
}

pub static ACTIVE_WORKSPACE: std::sync::RwLock<ActiveWorkspace> =
    std::sync::RwLock::new(ActiveWorkspace::Chat);

// Extra directory searched for API key / config files (e.g. MOONSHOT_API_KEY,
// MOONSHOT_BASE_URL). Desktop reads env or CWD; OHOS/mobile sets this to the
// app sandbox data dir at startup since env/CWD are not usable there.
pub static KEY_CONFIG_DIR: std::sync::RwLock<Option<String>> = std::sync::RwLock::new(None);

// Global chat state accessible to ChatList widget
pub static CHAT_DATA: std::sync::RwLock<ChatData> = std::sync::RwLock::new(ChatData {
    messages: Vec::new(),
    streaming_text: String::new(),
    thinking_text: String::new(),
    is_streaming: false,
});

pub static APP_GEN_DATA: std::sync::RwLock<ChatData> = std::sync::RwLock::new(ChatData {
    messages: Vec::new(),
    streaming_text: String::new(),
    thinking_text: String::new(),
    is_streaming: false,
});

pub static APP_DEMO_STATE: std::sync::RwLock<AppDemoState> = std::sync::RwLock::new(AppDemoState {
    count: 0,
    timer: TimerDemoState {
        duration_seconds: 25 * 60,
        remaining_seconds: 25 * 60,
        is_running: false,
    },
});

// Slider position range (NOT alpha — alpha is derived per-layer).
const DEFAULT_GLASS_OPACITY: f64 = 0.90;
const MIN_GLASS_OPACITY: f64 = 0.10;
const MAX_GLASS_OPACITY: f64 = 1.00;

#[derive(Debug, Clone, Copy, PartialEq)]
struct GlassOpacity {
    app: f32,
    sidebar: f32,
    main: f32,
    composer: f32,
}

// Map slider [0.10..1.00] to layer-specific material strength. Broad shell
// layers stay light so the overlay AppleGlass surfaces can visibly refract the
// scene instead of being hidden behind an opaque green wash.
fn glass_opacity_values(slider: f64) -> GlassOpacity {
    let t = ((slider.clamp(MIN_GLASS_OPACITY, MAX_GLASS_OPACITY) - MIN_GLASS_OPACITY)
        / (MAX_GLASS_OPACITY - MIN_GLASS_OPACITY)) as f32;
    let shell = 0.32 + t * 0.26;
    GlassOpacity {
        app: shell,
        main: (shell + 0.04).min(0.62),
        sidebar: (shell + 0.06).min(0.66),
        composer: 0.035 + t * 0.055,
    }
}

fn should_start_window_drag(abs: DVec2, size: DVec2) -> bool {
    const RESIZE_EDGE_MARGIN: f64 = 10.0;
    const DRAG_STRIP_HEIGHT: f64 = 52.0;
    const RIGHT_TOOLBAR_WIDTH: f64 = 260.0;

    abs.y > RESIZE_EDGE_MARGIN
        && abs.y < DRAG_STRIP_HEIGHT
        && abs.x > RESIZE_EDGE_MARGIN
        && abs.x < size.x - RESIZE_EDGE_MARGIN
        && abs.x < size.x - RIGHT_TOOLBAR_WIDTH
}

/// Some LLMs, when asked "show me a markdown file demo with ... inside",
/// wrap their ENTIRE reply in a single ```markdown ... ``` fence. Because
/// CommonMark does not support fence nesting, pulldown-cmark then treats
/// the whole reply as ONE code block — collapsing markdown structure
/// (headings, lists, inner fences, math, …) into monospace text and killing
/// the streaming fade animation.
///
/// Strategy is aggressive: as soon as the text starts with ```markdown\n (or
/// ```md\n), strip that opener even if the outer fence hasn't closed yet.
/// Otherwise we'd keep the whole streaming reply stuck in code-block mode
/// until the final token arrives. The trailing outer fence is stripped too
/// when present.
fn unwrap_outer_markdown_fence(text: &str) -> &str {
    let trimmed_text = text.trim_start();
    // CommonMark allows fences of any length ≥ 3 — 3 backticks for plain
    // code, 4+ for wrappers that want to contain inner 3-backtick blocks.
    // LLMs use both; handle any length.
    let bt_count = trimmed_text.bytes().take_while(|b| *b == b'`').count();
    if bt_count < 3 {
        return text;
    }
    let after_ticks = &trimmed_text[bt_count..];
    let body_start = after_ticks
        .strip_prefix("markdown\n")
        .or_else(|| after_ticks.strip_prefix("md\n"));
    let Some(body) = body_start else {
        return text;
    };
    // Try to strip a matching closing fence at the end: same length or
    // longer, optionally followed by trailing whitespace. If streaming is
    // mid-way and there's no close yet, return the opener-stripped body.
    let close_pat = "`".repeat(bt_count);
    let end_trimmed = body.trim_end();
    if let Some(without_close) = end_trimmed.strip_suffix(&close_pat) {
        return without_close.trim_end_matches('\n').trim_end();
    }
    body
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DiagramFenceStatus {
    None,
    Valid,
    UnclosedNonDiagram,
    Invalid,
}

struct OpenReplyFence {
    count: usize,
    fence_char: char,
    info: String,
    body_start: usize,
}

fn scan_diagram_fence_status(text: &str) -> DiagramFenceStatus {
    let text = unwrap_outer_markdown_fence(text);
    let mut status = DiagramFenceStatus::None;
    let mut open: Option<OpenReplyFence> = None;
    let mut line_start = 0;
    let bytes = text.as_bytes();
    let mut i = 0;

    while i <= bytes.len() {
        let at_end = i == bytes.len();
        let is_newline = !at_end && bytes[i] == b'\n';
        if at_end || is_newline {
            let line = text.get(line_start..i).unwrap_or("");
            let next_line_start = if is_newline { i + 1 } else { i };

            match &open {
                Some(fence) => {
                    if reply_fence_closes(line, fence) {
                        if fence.info.eq_ignore_ascii_case("diagram") {
                            let body = text.get(fence.body_start..line_start).unwrap_or("");
                            if crate::makepad_diagram_kit::parse(body.trim()).is_err() {
                                return DiagramFenceStatus::Invalid;
                            }
                            status = DiagramFenceStatus::Valid;
                        }
                        open = None;
                    }
                }
                None => {
                    if let Some((count, fence_char, info)) = reply_fence_opens(line) {
                        open = Some(OpenReplyFence {
                            count,
                            fence_char,
                            info,
                            body_start: next_line_start,
                        });
                    }
                }
            }

            line_start = next_line_start;
            i += 1;
        } else {
            i += 1;
        }
    }

    match open {
        Some(fence) if fence.info.eq_ignore_ascii_case("diagram") => DiagramFenceStatus::Invalid,
        Some(_) => DiagramFenceStatus::UnclosedNonDiagram,
        None => status,
    }
}

fn reply_fence_opens(line: &str) -> Option<(usize, char, String)> {
    let trimmed = line.trim_start().trim_end_matches('\r');
    let first = trimmed.chars().next()?;
    if first != '`' && first != '~' {
        return None;
    }

    let count = trimmed.chars().take_while(|ch| *ch == first).count();
    if count < 3 {
        return None;
    }

    let info = trimmed[count..]
        .trim()
        .split_ascii_whitespace()
        .next()
        .unwrap_or("")
        .to_string();
    Some((count, first, info))
}

fn reply_fence_closes(line: &str, fence: &OpenReplyFence) -> bool {
    let trimmed = line.trim_start().trim_end_matches('\r');
    let count = trimmed
        .chars()
        .take_while(|ch| *ch == fence.fence_char)
        .count();
    count >= fence.count && trimmed[count..].trim().is_empty()
}

fn assistant_message_is_safe_to_store(text: &str) -> bool {
    scan_diagram_fence_status(text) != DiagramFenceStatus::Invalid
}

fn assistant_message_is_safe_for_history(text: &str) -> bool {
    // Stateless replay: only re-inject diagram-free messages. A rendered or
    // malformed diagram in history can confuse the next turn, so keep those
    // out of stateless replay even if they are safe to display.
    scan_diagram_fence_status(text) == DiagramFenceStatus::None
}

const CHAT_SAVE_PATH: &str = "aichat_history.json";
const APP_GEN_SAVE_PATH: &str = "aichat_appgen_history.json";
const APP_STATE_SAVE_PATH: &str = "aichat_app_state.json";
const MAX_STATELESS_HISTORY_MESSAGES: usize = 12;

#[derive(Clone, Debug, SerJson, DeJson)]
pub struct AppDemoState {
    count: i64,
    timer: TimerDemoState,
}

#[derive(Clone, Debug, SerJson, DeJson)]
pub struct TimerDemoState {
    duration_seconds: i64,
    remaining_seconds: i64,
    is_running: bool,
}

impl Default for AppDemoState {
    fn default() -> Self {
        Self {
            count: 0,
            timer: TimerDemoState::default(),
        }
    }
}

impl Default for TimerDemoState {
    fn default() -> Self {
        Self {
            duration_seconds: 25 * 60,
            remaining_seconds: 25 * 60,
            is_running: false,
        }
    }
}

impl AppDemoState {
    fn load_from_disk() -> Self {
        std::fs::read_to_string(APP_STATE_SAVE_PATH)
            .ok()
            .and_then(|s| Self::deserialize_json(&s).ok())
            .unwrap_or_default()
    }

    fn save_to_disk(&self) {
        let _ = std::fs::write(APP_STATE_SAVE_PATH, self.serialize_json());
    }

    fn prompt_json(&self) -> String {
        format!(
            "{{\n  \"count\": {},\n  \"timer\": {{\n    \"duration_seconds\": {},\n    \"remaining_seconds\": {},\n    \"display\": \"{}\",\n    \"is_running\": {},\n    \"button_label\": \"{}\"\n  }}\n}}",
            self.count,
            self.timer.duration_seconds,
            self.timer.remaining_seconds,
            self.timer.display(),
            self.timer.is_running,
            self.timer.button_label()
        )
    }
}

impl TimerDemoState {
    fn display(&self) -> String {
        let total = self.remaining_seconds.max(0);
        format!("{:02}:{:02}", total / 60, total % 60)
    }

    fn button_label(&self) -> &'static str {
        if self.is_running {
            "Pause"
        } else {
            "Start"
        }
    }

    fn reset(&mut self) {
        self.remaining_seconds = self.duration_seconds.max(60);
        self.is_running = false;
    }
}

fn state_template_value(state: &AppDemoState, path: &str) -> Option<String> {
    match path {
        "count" => Some(state.count.to_string()),
        "timer.duration_seconds" => Some(state.timer.duration_seconds.to_string()),
        "timer.remaining_seconds" => Some(state.timer.remaining_seconds.to_string()),
        "timer.display" => Some(state.timer.display()),
        "timer.is_running" => Some(state.timer.is_running.to_string()),
        "timer.button_label" => Some(state.timer.button_label().to_string()),
        _ => None,
    }
}

fn render_state_templates(raw: &str, state: &AppDemoState) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut rest = raw;

    while let Some(start) = rest.find("{{state.") {
        out.push_str(&rest[..start]);
        let after_open = &rest[start + "{{state.".len()..];

        let Some(end) = after_open.find("}}") else {
            out.push_str(&rest[start..]);
            return out;
        };

        let path = after_open[..end].trim();
        if let Some(value) = state_template_value(state, path) {
            out.push_str(&value);
        } else {
            log!("[render_state_templates] unknown path: {}", path);
            out.push_str("{{state.");
            out.push_str(path);
            out.push_str("}}");
        }

        rest = &after_open[end + "}}".len()..];
    }

    out.push_str(rest);
    out
}

fn prompt_with_state(section: &str, body: &str) -> String {
    let state = APP_DEMO_STATE.read().unwrap();
    format!(
        "[Current app state]\n{}\n\n[{}]\n{}",
        state.prompt_json(),
        section,
        body
    )
}

fn app_generation_prompt_with_state(body: &str) -> String {
    let state = APP_DEMO_STATE.read().unwrap();
    let capability = AppCapability::detect(body);
    let title = body
        .lines()
        .next()
        .unwrap_or("Generated App")
        .chars()
        .take(48)
        .collect::<String>()
        .replace('"', "'");
    format!(
        r#"[Current app state]
{}

[App Generation Request]
{}

[Host-selected capability]
{}

{}

[Plan contract]
First output exactly one ```appplan json fenced block using this plan as the source of truth:
```appplan json
{}
```

[UI output contract]
After the appplan block, output exactly one ```runsplash fenced block.
Do not return diagrams, JSX, React, JavaScript, handler source files, or companion logic.
Use only the state paths and agent.notify actions listed in the capability manifest.
Use Makepad Splash syntax, for example:
```runsplash
RoundedView{{
    width: Fill height: Fit
    flow: Down spacing: 12 padding: 16
    Label{{ text: "Count: {{{{state.count}}}}" }}
    Button{{ text: "+1" on_click: || agent.notify("inc", {{}}) }}
}}
```
Use {{{{state.count}}}} for displayed counter state when the UI has a count.
The payload for listed demo actions must be an empty object.
If a required control is listed in the manifest, it must be visible in the UI."#,
        state.prompt_json(),
        body,
        capability.app_type(),
        capability.manifest(),
        capability.app_plan_json(&title)
    )
}

fn app_generation_session_system_prompt() -> String {
    let splash_md_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../splash.md");
    let splash_md = std::fs::read_to_string(&splash_md_path)
        .unwrap_or_else(|_| include_str!("../../../splash.md").to_string());
    format!(
        r#"You are an app-generation agent for Makepad aichat.

Your job is to generate live, clickable app UI using Makepad Splash.

Hard output rules:
- For app UI requests, return one ```appplan json fenced block followed by one ```runsplash fenced block.
- Do not return ```diagram, JSX, React, JavaScript, TypeScript, handler source files, or companion logic.
- Do not explain the code before, between, or after those fenced blocks.
- Use only the capability manifest supplied in the user message. Do not invent host actions.
- `use mod.prelude.widgets.*` is automatically prepended. Do not include imports.
- Do not wrap content in Root{{}} or Window{{}}. The content is inserted into an existing container.

Interactive generated UI can notify the host from button callbacks:

Button{{ text: "+1" on_click: || agent.notify("inc", {{}}) }}

The user message supplies the current capability manifest. For example, counter apps may use:

agent.notify("inc", {{}})
agent.notify("dec", {{}})
agent.notify("reset", {{}})
agent.notify("ask_ai", {{}})

Timer apps may use:

agent.notify("timer.start", {{}})
agent.notify("timer.pause", {{}})
agent.notify("timer.toggle", {{}})
agent.notify("timer.reset", {{}})
agent.notify("timer.add_minute", {{}})
agent.notify("timer.subtract_minute", {{}})

Only use actions listed in the current capability manifest.

Display host state with markdown-layer placeholders inside string literals:

Label{{ text: "Count: {{{{state.count}}}}" }}
Label{{ text: "{{{{state.timer.display}}}}" }}

Here is the Splash scripting manual. Follow it exactly:

{splash_md}"#
    )
}

fn chat_data_for_workspace(workspace: ActiveWorkspace) -> &'static std::sync::RwLock<ChatData> {
    match workspace {
        ActiveWorkspace::Chat => &CHAT_DATA,
        ActiveWorkspace::AppGen => &APP_GEN_DATA,
    }
}

fn active_workspace() -> ActiveWorkspace {
    *ACTIVE_WORKSPACE.read().unwrap()
}

fn active_chat_data() -> &'static std::sync::RwLock<ChatData> {
    chat_data_for_workspace(active_workspace())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AppCapability {
    Counter,
    Timer,
    Todo,
    Generic,
}

impl AppCapability {
    fn detect(request: &str) -> Self {
        let text = request.to_lowercase();
        if text.contains("番茄")
            || text.contains("pomodoro")
            || text.contains("timer")
            || text.contains("倒计时")
            || text.contains("时钟")
        {
            Self::Timer
        } else if text.contains("todo")
            || text.contains("待办")
            || text.contains("任务")
            || text.contains("清单")
        {
            Self::Todo
        } else if text.contains("counter") || text.contains("计数") || text.contains("count") {
            Self::Counter
        } else {
            Self::Generic
        }
    }

    fn app_type(self) -> &'static str {
        match self {
            Self::Counter => "counter",
            Self::Timer => "timer",
            Self::Todo => "todo",
            Self::Generic => "generic",
        }
    }

    fn app_plan_json(self, title: &str) -> String {
        match self {
            Self::Counter => format!(
                r#"{{"app_type":"counter","title":"{}","state_paths":["count"],"actions":["inc","dec","reset"],"required_controls":["increment","decrement","reset"]}}"#,
                title
            ),
            Self::Timer => format!(
                r#"{{"app_type":"timer","title":"{}","state_paths":["timer.display","timer.is_running","timer.button_label"],"actions":["timer.start","timer.pause","timer.toggle","timer.reset","timer.add_minute","timer.subtract_minute"],"required_controls":["visible start or pause","reset","optional duration adjustment"]}}"#,
                title
            ),
            Self::Todo => format!(
                r#"{{"app_type":"todo","title":"{}","state_paths":[],"actions":[],"required_controls":["list","add item","toggle item","delete item"],"status":"planned but not implemented in host runtime yet"}}"#,
                title
            ),
            Self::Generic => format!(
                r#"{{"app_type":"generic","title":"{}","state_paths":["count"],"actions":["ask_ai"],"required_controls":["primary interaction"],"status":"host runtime has only generic ask_ai plus counter/timer capabilities"}}"#,
                title
            ),
        }
    }

    fn manifest(self) -> &'static str {
        match self {
            Self::Counter => {
                r#"[Available state paths]
{{state.count}}

[Available actions]
agent.notify("inc", {})
agent.notify("dec", {})
agent.notify("reset", {})
agent.notify("ask_ai", {})

[Required controls]
- A visible +1 button
- A visible -1 button
- A visible Reset button"#
            }
            Self::Timer => {
                r#"[Available state paths]
{{state.timer.display}}              // formatted MM:SS
{{state.timer.duration_seconds}}     // total configured seconds
{{state.timer.remaining_seconds}}    // remaining seconds
{{state.timer.is_running}}           // "true" or "false"
{{state.timer.button_label}}         // "Start" or "Pause"

[Available actions]
agent.notify("timer.start", {})
agent.notify("timer.pause", {})
agent.notify("timer.toggle", {})
agent.notify("timer.reset", {})
agent.notify("timer.add_minute", {})
agent.notify("timer.subtract_minute", {})
agent.notify("ask_ai", {})

[Required controls]
- A clearly visible Start/Pause control using {{state.timer.button_label}} and timer.toggle, or separate Start and Pause buttons
- A visible Reset button
- Optional +1 minute and -1 minute buttons"#
            }
            Self::Todo => {
                r#"[Available state paths]
Todo state is not implemented yet in this demo runtime.

[Available actions]
agent.notify("ask_ai", {})

[Required controls]
- Render a static todo mockup or ask for Todo runtime support.
- Do not invent unimplemented todo.add/todo.delete actions."#
            }
            Self::Generic => {
                r#"[Available state paths]
{{state.count}}
{{state.timer.display}}
{{state.timer.button_label}}

[Available actions]
agent.notify("ask_ai", {})
agent.notify("inc", {})
agent.notify("dec", {})
agent.notify("reset", {})
agent.notify("timer.toggle", {})
agent.notify("timer.reset", {})

[Required controls]
- Use only the actions listed above.
- If the app needs unavailable host behavior, show a static mockup and include an Ask AI button."#
            }
        }
    }
}

fn stateless_history_messages(messages: &[ChatMessage]) -> Vec<Message> {
    let mut history = Vec::new();
    let mut index = 0;

    while index + 1 < messages.len() {
        let user = &messages[index];
        let assistant = &messages[index + 1];
        if user.role == ChatRole::User
            && assistant.role == ChatRole::Assistant
            && assistant_message_is_safe_for_history(&assistant.text)
        {
            history.push(Message::user(&user.text));
            history.push(Message::assistant(&assistant.text));
            index += 2;
        } else {
            index += 1;
        }
    }

    let start = history.len().saturating_sub(MAX_STATELESS_HISTORY_MESSAGES);
    history.split_off(start)
}

#[derive(SerJson, DeJson)]
struct SavedMessage {
    role: String,
    content: String,
}

#[derive(SerJson, DeJson, Default)]
struct SavedHistory {
    messages: Vec<SavedMessage>,
}

#[derive(Clone)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub text: String,
}

#[derive(Script, ScriptHook, Widget)]
pub struct MermaidSvgView {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,
    #[redraw]
    #[live]
    draw_svg: DrawSvg,
    #[live]
    draw_text: DrawText,
    #[live]
    draw_flow_dot: DrawColor,
    #[rust]
    doc: SvgDocument,
    #[rust]
    content_w: f64,
    #[rust]
    content_h: f64,
    #[rust]
    last_src_hash: u64,
    #[rust]
    pending_src_hash: u64,
    #[rust]
    cached_text_cmds: Vec<SvgTextCmd>,
    #[rust]
    cached_edges: Vec<SvgEdge>,
    #[rust(1.0f64)]
    zoom: f64,
    #[rust]
    pan: DVec2,
    #[rust]
    drag_start_abs: Option<DVec2>,
    #[rust]
    drag_start_pan: DVec2,
    #[rust]
    last_rect: Rect,
    #[rust]
    anim_t: f32,
    #[rust]
    next_frame: NextFrame,
}

impl MermaidSvgView {
    pub fn set_svg_str(&mut self, cx: &mut Cx, svg: &str) {
        use std::hash::{DefaultHasher, Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        svg.hash(&mut hasher);
        let hash = hasher.finish();
        if hash == self.last_src_hash && !self.doc.root.is_empty() {
            return;
        }

        self.last_src_hash = hash;
        self.doc = parse_svg(svg);
        self.cached_text_cmds = collect_text_cmds(&self.doc);
        self.cached_edges = collect_edges(&self.doc);
        self.draw_svg.cache_valid = false;
        self.draw_svg.set_doc_bounds(&self.doc);
        if let Some(vb) = self.doc.viewbox.as_ref() {
            self.draw_svg.content_bounds = (vb.x, vb.y, vb.x + vb.width, vb.y + vb.height);
            self.content_w = vb.width as f64;
            self.content_h = vb.height as f64;
            self.draw_svg.content_size = dvec2(self.content_w, self.content_h);
        }
        self.redraw(cx);
    }

    pub fn set_mermaid_src(&mut self, cx: &mut Cx, src: &str) {
        use std::hash::{DefaultHasher, Hash, Hasher};

        let cleaned: String = src.chars().filter(|c| *c != '▋').collect();
        let trimmed = cleaned.trim();
        if trimmed.is_empty() || trimmed.len() < 8 {
            return;
        }

        let mut hasher = DefaultHasher::new();
        trimmed.hash(&mut hasher);
        let hash = hasher.finish();
        if hash == self.last_src_hash && !self.doc.root.is_empty() {
            return;
        }

        // Streaming debounce: render only when the same source arrives twice
        // in a row. During active token streaming the body changes every
        // frame; after a pause or close it stabilizes and renders once.
        if hash != self.pending_src_hash {
            self.pending_src_hash = hash;
            return;
        }

        match streaming_markdown_kit::render_mermaid_to_svg(trimmed) {
            Ok(svg) => {
                self.set_svg_str(cx, &svg);
                self.last_src_hash = hash;
            }
            Err(err) => {
                log!("mermaid render error: {:?}", err);
            }
        }
    }
}

impl Widget for MermaidSvgView {
    fn set_text(&mut self, cx: &mut Cx, v: &str) {
        self.set_mermaid_src(cx, v);
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        if self.next_frame.is_event(event).is_some() {
            self.anim_t = (self.anim_t + 0.003).rem_euclid(1.0);
            self.next_frame = cx.new_next_frame();
            self.redraw(cx);
        }

        match event.hits_with_capture_overload(cx, self.draw_svg.area(), true) {
            Hit::FingerDown(fe) if fe.is_primary_hit() => {
                if fe.tap_count >= 2 {
                    self.zoom = 1.0;
                    self.pan = DVec2::default();
                    self.drag_start_abs = None;
                    self.redraw(cx);
                } else {
                    self.drag_start_abs = Some(fe.abs);
                    self.drag_start_pan = self.pan;
                    cx.set_cursor(MouseCursor::Grabbing);
                }
            }
            Hit::FingerMove(fe) => {
                if let Some(start) = self.drag_start_abs {
                    self.pan = self.drag_start_pan + (fe.abs - start);
                    self.redraw(cx);
                }
            }
            Hit::FingerUp(_) => {
                if self.drag_start_abs.is_some() {
                    self.drag_start_abs = None;
                    cx.set_cursor(MouseCursor::Grab);
                }
            }
            Hit::FingerHoverIn(_) => cx.set_cursor(MouseCursor::Grab),
            Hit::FingerScroll(fs) => {
                if !fs.modifiers.is_primary() {
                    return;
                }
                let dy = if fs.scroll.y.abs() > f64::EPSILON {
                    fs.scroll.y
                } else {
                    fs.scroll.x
                };
                let factor = (1.0 - dy * 0.005).clamp(0.5, 2.0);
                let old_zoom = self.zoom.max(0.01);
                let new_zoom = (old_zoom * factor).clamp(0.2, 8.0);
                let local = fs.abs - self.last_rect.pos - self.pan;
                let content_local = local / old_zoom;
                self.pan = fs.abs - self.last_rect.pos - content_local * new_zoom;
                self.zoom = new_zoom;
                self.redraw(cx);
            }
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        if self.doc.root.is_empty() {
            return DrawStep::done();
        }
        let sw = self.draw_svg.content_size.x;
        let sh = self.draw_svg.content_size.y;
        if sw <= 0.0 || sh <= 0.0 {
            return DrawStep::done();
        }
        let walk = Walk {
            abs_pos: walk.abs_pos,
            margin: walk.margin,
            width: match walk.width {
                Size::Fit { .. } => Size::Fixed(sw),
                other => other,
            },
            height: match walk.height {
                Size::Fit { .. } => Size::Fixed(sh),
                other => other,
            },
            metrics: walk.metrics,
        };
        let rect = cx.walk_turtle(walk);
        self.last_rect = rect;

        let zoom = if self.zoom > 0.01 { self.zoom } else { 1.0 };
        let effective_rect = Rect {
            pos: rect.pos + self.pan,
            size: rect.size * zoom,
        };

        self.draw_svg.svg_doc = Some(std::mem::take(&mut self.doc));
        self.draw_svg.has_animations = false;
        self.draw_svg.render_to_rect(cx, &effective_rect, 0.0);
        self.doc = self.draw_svg.svg_doc.take().unwrap_or_default();

        let text_cmds = std::mem::take(&mut self.cached_text_cmds);
        self.render_text_cmds(cx, &effective_rect, &text_cmds);
        self.cached_text_cmds = text_cmds;

        let edges = std::mem::take(&mut self.cached_edges);
        self.render_flow_dots(cx, &effective_rect, &edges);
        let has_edges = !edges.is_empty();
        self.cached_edges = edges;

        if has_edges {
            self.next_frame = cx.new_next_frame();
        }
        DrawStep::done()
    }
}

impl MermaidSvgView {
    fn render_text_cmds(&mut self, cx: &mut Cx2d, rect: &Rect, cmds: &[SvgTextCmd]) {
        if cmds.is_empty() {
            return;
        }
        let (min_x, min_y, max_x, max_y) = self.draw_svg.content_bounds;
        let content_w = (max_x - min_x) as f64;
        let content_h = (max_y - min_y) as f64;
        if content_w <= 0.0 || content_h <= 0.0 {
            return;
        }
        let scale = (rect.size.x / content_w).min(rect.size.y / content_h);
        let render_w = content_w * scale;
        let render_h = content_h * scale;
        let origin_x = rect.pos.x + (rect.size.x - render_w) * 0.5;
        let origin_y = rect.pos.y + (rect.size.y - render_h) * 0.5;
        const PX_TO_PT: f64 = 0.75;

        for cmd in cmds {
            if cmd.text.trim().is_empty() {
                continue;
            }
            let world_font_size = (cmd.font_size as f64 * scale * PX_TO_PT).max(1.0);
            self.draw_text.text_style.font_size = world_font_size as f32;
            self.draw_text.color =
                vec4(cmd.color.0, cmd.color.1, cmd.color.2, cmd.color.3.max(0.0));

            let lines: Vec<&str> = cmd.text.split('\n').collect();
            let line_step_screen = world_font_size * 1.2;
            let base_cy = origin_y + (cmd.y as f64 - min_y as f64) * scale;
            let base_cx_screen = origin_x + (cmd.x as f64 - min_x as f64) * scale;

            for (line_index, line) in lines.iter().enumerate() {
                if line.is_empty() {
                    continue;
                }
                let estimated_width: f64 = line
                    .chars()
                    .map(|ch| {
                        let advance = if (ch as u32) >= 0x2E80 { 1.0 } else { 0.55 };
                        advance * world_font_size
                    })
                    .sum();
                let anchor_shift = match cmd.text_anchor {
                    SvgTextAnchor::Start => 0.0,
                    SvgTextAnchor::Middle => -0.5,
                    SvgTextAnchor::End => -1.0,
                } * estimated_width;

                let px = base_cx_screen + anchor_shift;
                let cy = base_cy + line_step_screen * line_index as f64;
                let py = cy - world_font_size * 0.7;
                self.draw_text.draw_abs(cx, dvec2(px, py), line);
            }
        }
    }

    fn render_flow_dots(&mut self, cx: &mut Cx2d, rect: &Rect, edges: &[SvgEdge]) {
        if edges.is_empty() {
            return;
        }
        let (min_x, min_y, max_x, max_y) = self.draw_svg.content_bounds;
        let content_w = (max_x - min_x) as f64;
        let content_h = (max_y - min_y) as f64;
        if content_w <= 0.0 || content_h <= 0.0 {
            return;
        }
        let scale = (rect.size.x / content_w).min(rect.size.y / content_h);
        let render_w = content_w * scale;
        let render_h = content_h * scale;
        let origin_x = rect.pos.x + (rect.size.x - render_w) * 0.5;
        let origin_y = rect.pos.y + (rect.size.y - render_h) * 0.5;
        let dot_size = 10.0_f64;
        let pulse = 0.55 + 0.45 * (self.anim_t * std::f32::consts::TAU * 1.5).sin().abs();

        for (edge_index, edge) in edges.iter().enumerate() {
            if edge.points.len() < 2 {
                continue;
            }
            let phase = (self.anim_t + edge_index as f32 * 0.17).rem_euclid(1.0);
            let max_index = edge.points.len() - 1;
            let float_index = phase * max_index as f32;
            let point_index = float_index as usize;
            let next_index = (point_index + 1).min(max_index);
            let frac = float_index - point_index as f32;
            let p0 = edge.points[point_index];
            let p1 = edge.points[next_index];
            let wx = p0.0 + (p1.0 - p0.0) * frac;
            let wy = p0.1 + (p1.1 - p0.1) * frac;

            let sx = origin_x + (wx as f64 - min_x as f64) * scale;
            let sy = origin_y + (wy as f64 - min_y as f64) * scale;

            self.draw_flow_dot.color = vec4(
                edge.color.0,
                edge.color.1,
                edge.color.2,
                edge.color.3 * pulse,
            );
            self.draw_flow_dot.draw_abs(
                cx,
                Rect {
                    pos: dvec2(sx - dot_size * 0.5, sy - dot_size * 0.5),
                    size: dvec2(dot_size, dot_size),
                },
            );
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum ChatRole {
    User,
    Assistant,
}

pub struct ChatData {
    pub messages: Vec<ChatMessage>,
    pub streaming_text: String,
    pub thinking_text: String,
    pub is_streaming: bool,
}

impl ChatData {
    pub fn save_to_disk(&self, path: &str) {
        let saved = SavedHistory {
            messages: self
                .messages
                .iter()
                .map(|m| SavedMessage {
                    role: match m.role {
                        ChatRole::User => "user".to_string(),
                        ChatRole::Assistant => "assistant".to_string(),
                    },
                    content: m.text.clone(),
                })
                .collect(),
        };
        let _ = std::fs::write(path, saved.serialize_json());
    }

    pub fn load_from_disk(path: &str) -> Vec<ChatMessage> {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|s| SavedHistory::deserialize_json(&s).ok())
            .map(|saved| {
                saved
                    .messages
                    .into_iter()
                    .map(|m| ChatMessage {
                        role: if m.role == "user" {
                            ChatRole::User
                        } else {
                            ChatRole::Assistant
                        },
                        text: m.content,
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}

// ChatList widget wrapping PortalList for chat message display.
#[derive(Script, ScriptHook, Widget)]
pub struct ChatList {
    #[deref]
    view: View,
    #[rust]
    animating_msg: Option<usize>,
}

impl Widget for ChatList {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let data = active_chat_data().read().unwrap();

        while let Some(item) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = item.as_portal_list().borrow_mut() {
                let msg_count = data.messages.len();
                let items_len = msg_count + data.is_streaming as usize;
                list.set_item_range(cx, 0, items_len);

                while let Some(item_id) = list.next_visible_item(cx) {
                    if data.is_streaming && item_id == msg_count {
                        let just_started = self.animating_msg != Some(item_id);
                        if just_started {
                            self.animating_msg = Some(item_id);
                        }

                        let (item_widget, _) = list.item_with_existed(cx, item_id, id!(Assistant));
                        let streaming_body;
                        let text: &str = if data.streaming_text.is_empty() {
                            if data.thinking_text.is_empty() {
                                "..."
                            } else {
                                "Thinking..."
                            }
                        } else {
                            let opts = SanitizeOptions {
                                trim_unclosed_fence: false,
                                ..SanitizeOptions::default()
                            };
                            // Remend keeps fenced blocks, tables and math
                            // self-consistent mid-stream so the Markdown
                            // widget doesn't re-layout a half-closed block
                            // on every token.
                            streaming_body = streaming_display_with_latex_autowrap_remend(
                                &data.streaming_text,
                                opts,
                            );
                            &streaming_body
                        };
                        let mut markdown = item_widget.markdown(cx, ids!(selectable));
                        // Unwrap outer ```markdown wrapper in streaming
                        // content: some LLMs emit the wrapper as the very
                        // first tokens, so we'd otherwise render a growing
                        // code block for the whole stream.
                        markdown.set_text(cx, unwrap_outer_markdown_fence(text));
                        if just_started {
                            markdown.reset_all_streaming_animations();
                        } else {
                            markdown.start_streaming_animation();
                        }
                        item_widget.draw_all_unscoped(cx);
                        continue;
                    }

                    if let Some(msg) = data.messages.get(item_id) {
                        let is_animating = self.animating_msg == Some(item_id);
                        let template = match msg.role {
                            ChatRole::User => id!(User),
                            ChatRole::Assistant => id!(Assistant),
                        };
                        let item_widget = list.item(cx, item_id, template);
                        let mut markdown = item_widget.markdown(cx, ids!(selectable));
                        // wrap_bare_latex wraps `\cmd{…}` with `$…$` so
                        // MathView can render them.
                        let unwrapped = unwrap_outer_markdown_fence(&msg.text);
                        let state_rendered;
                        let display_text = if msg.role == ChatRole::Assistant {
                            let state = APP_DEMO_STATE.read().unwrap();
                            state_rendered = render_state_templates(unwrapped, &state);
                            state_rendered.as_str()
                        } else {
                            unwrapped
                        };
                        let rendered = wrap_bare_latex(display_text);
                        markdown.set_text(cx, &rendered);
                        if is_animating {
                            markdown.stop_streaming_animation();
                        }
                        item_widget.draw_all_unscoped(cx);
                        if is_animating && markdown.is_streaming_animation_done() {
                            self.animating_msg = None;
                        }
                    }
                }
            }
        }
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);

        if let Event::Actions(actions) = event {
            let list = self.view.portal_list(cx, ids!(list));
            if list.any_items_with_actions(actions) {
                for (item_id, item) in list.items_with_actions(actions) {
                    let copy_btn = item.button(cx, ids!(copy_button));
                    if copy_btn.clicked(actions) {
                        let data = active_chat_data().read().unwrap();
                        if let Some(msg) = data.messages.get(item_id) {
                            cx.copy_to_clipboard(&msg.text);
                        }
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BackendType {
    ClaudeCode,
    ClaudeSplash,
    ClaudeAcp,
    ClaudeApi,
    Gemini,
    GeminiSplash,
    OpenAi,
    Moonshot,
}

const ALL_BACKENDS: [BackendType; 8] = [
    BackendType::ClaudeCode,
    BackendType::ClaudeSplash,
    BackendType::ClaudeAcp,
    BackendType::ClaudeApi,
    BackendType::Gemini,
    BackendType::GeminiSplash,
    BackendType::OpenAi,
    BackendType::Moonshot,
];

impl BackendType {
    fn to_index(self) -> usize {
        ALL_BACKENDS.iter().position(|&b| b == self).unwrap()
    }

    fn from_index(index: usize) -> Option<Self> {
        ALL_BACKENDS.get(index).copied()
    }

    fn status_label(self) -> &'static str {
        match self {
            Self::ClaudeCode => "Active: Claude Code",
            Self::ClaudeSplash => "Active: Claude Splash (UI Agent via ACP)",
            Self::ClaudeAcp => "Active: Claude (ACP via Zed)",
            Self::ClaudeApi => "Active: Claude (API)",
            Self::Gemini => "Active: Gemini",
            Self::GeminiSplash => "Active: Gemini Splash (UI Agent)",
            Self::OpenAi => "Active: OpenAI",
            Self::Moonshot => "Active: Moonshot",
        }
    }

    fn system_prompt(self) -> String {
        match self {
            Self::ClaudeSplash | Self::GeminiSplash => {
                let splash_md_path =
                    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../splash.md");
                let splash_md = std::fs::read_to_string(&splash_md_path)
                    .unwrap_or_else(|_| include_str!("../../../splash.md").to_string());
                format!(
                    r#"You are an AI agent that can create on-demand UI using Makepad's Splash scripting language.

You can answer questions normally using markdown. But when it makes sense to show something visually — a layout, a UI mockup, a styled card, a button arrangement, an animation, or anything graphical — you should embed a ```runsplash code block in your markdown response. The content inside a ```runsplash block is live Splash script that will be rendered as real interactive UI inline in the chat.

IMPORTANT: `use mod.prelude.widgets.*` is automatically prepended to every runsplash block — do NOT include it yourself. All widget names (View, Label, Button, etc.) are already in scope.

The block content is Splash script. It gets evaluated and rendered as a live widget tree. Do NOT wrap it in Root{{}} or Window{{}} — the content is placed directly inside a container.

Interactive generated UI can notify the host from button callbacks:

Button{{ text: "+1" on_click: || agent.notify("inc", {{}}) }}

Supported demo actions are `inc`, `dec`, `reset`, and `ask_ai`. For the D1 counter, always use an empty object payload:

agent.notify("inc", {{}})
agent.notify("dec", {{}})
agent.notify("reset", {{}})

To display host state in D1, use the markdown-layer placeholder `{{{{state.count}}}}` inside string literals:

Label{{ text: "Count: {{{{state.count}}}}" }}

Do not hardcode mutable state when the UI should reflect host state.

Here is the complete Splash scripting manual. Follow it exactly:

{splash_md}"#
                )
            }
            _ => r#"You are a helpful assistant. Be concise but thorough.

Formatting: respond in GitHub-flavoured Markdown. Wrap every mathematical expression in LaTeX delimiters — `$…$` for inline math (e.g. `$a^2 + b^2 = c^2$`) and `$$…$$` on their own lines for display math. NEVER write LaTeX commands (`\frac`, `\mathbb`, `\sum`, `\forall`, etc.) outside these delimiters, even when the math is short.

## Diagrams — use ```diagram fenced JSON for visual structure

When a user's question benefits from a visual diagram — hierarchy, layered stack, decision flow, state machine, data model, timeline, process lanes, set overlap, request/response interaction, 2-axis comparison, or tree of concepts — emit a fenced code block with the language tag `diagram` whose body is JSON matching the diagram-kit v1 spec. Do not narrate the JSON; just render it.

The thirteen supported types are `pyramid`, `quadrant`, `tree`, `layers`, `flowchart`, `architecture`, `sequence`, `state`, `er`, `timeline`, `swimlane`, `nested`, and `venn`. Use the specific type the user asks for; do NOT downgrade `state` to `flowchart`, `er` to `architecture`, `timeline` to `sequence`, or `swimlane` to `flowchart`.

Shared optional text fields where applicable: `tag` (short uppercase, like `ROOT` `CAT` `EXT` — appears as a small pill in the top-left) and `sublabel` (mono-font secondary line below the label). Top-level `accent_idx` (integer, 0-based) or `accent_path` (array of indices, tree only) applies to `pyramid`, `quadrant`, `tree`, `layers`, and `flowchart` only. `architecture`, `sequence`, `state`, `er`, `swimlane`, `nested`, and `venn` use role-driven emphasis instead. `timeline` uses `role:"major"` for its emphasized milestone.

### `pyramid` — ranked layers, narrow apex at top

```diagram
{"type":"pyramid","levels":[{"label":"Vision","tag":"APEX"},{"label":"Strategy"},{"label":"Tactics","sublabel":"weekly"}],"accent_idx":0}
```

### `quadrant` — 2-axis scatter with 4 labelled quadrants

```diagram
{"type":"quadrant","axes":{"x":{"min":0,"max":10,"low_label":"LOW EFFORT","high_label":"HIGH EFFORT"},"y":{"min":0,"max":10,"low_label":"LOW IMPACT","high_label":"HIGH IMPACT"}},"points":[{"x":2,"y":9,"label":"quick win"},{"x":9,"y":9,"label":"big bet"},{"x":2,"y":2,"label":"fill-in"}]}
```

### `tree` — parent → children hierarchy, root at top

```diagram
{"type":"tree","root":{"label":"Product","tag":"ROOT","children":[{"label":"Core","tag":"CAT","children":[{"label":"Parse","tag":"SUB"},{"label":"Layout","tag":"SUB"}]},{"label":"Bindings","tag":"CAT"}]},"accent_path":[0,1]}
```

### `layers` — stacked horizontal bands, top layer first in array

```diagram
{"type":"layers","layers":[{"label":"Application","tag":"L7"},{"label":"Transport","tag":"L4","sublabel":"TCP · UDP"},{"label":"Network","tag":"L3"},{"label":"Physical","tag":"L1"}],"accent_idx":1}
```

### `flowchart` — nodes + edges, vertical decision flow

Node shapes: `"rect"` (default), `"oval"` (start/end), `"diamond"` (decision — no tag renders on diamonds). Edge `role`: `"default"` (muted black), `"primary"` (accent orange — the main path), `"external"` (link blue — external/HTTP calls). Edge `label` is optional mono caption at midpoint.

```diagram
{"type":"flowchart","nodes":[{"id":"req","label":"Receive","tag":"IN","shape":"oval"},{"id":"auth","label":"Authorized?","shape":"diamond"},{"id":"serve","label":"Serve","tag":"OUT","shape":"rect"}],"edges":[{"from":"req","to":"auth"},{"from":"auth","to":"serve","label":"yes","role":"primary"}],"accent_idx":2}
```

### `architecture` — 2D layered system diagram with role-tagged nodes

For cloud / service / data-flow diagrams where each box plays a distinct architectural role. Nodes get a `role`: `"focal"` (THE highlighted component — tint fill, accent stroke), `"backend"` (compute — white fill, ink stroke), `"store"` (database / cache — light ink fill, muted stroke), `"external"` (client / 3rd-party — faded), `"input"` (user input source), `"optional"` (sidecar / observability), `"security"` (auth / encryption). Layout is left-to-right layered; `"orientation":"tb"` makes it top-down.

Edges reuse the flowchart `role` enum: `"default"`, `"primary"`, `"external"`.

```diagram
{"type":"architecture","nodes":[{"id":"client","label":"Reader","tag":"EXT","role":"external"},{"id":"cdn","label":"Cloudflare","tag":"EDGE","role":"backend","sublabel":"Pages · cache"},{"id":"app","label":"Astro Origin","tag":"ORIG","role":"focal","sublabel":"SSR + MDX"},{"id":"mdx","label":"MDX Bundle","tag":"BUN","role":"store"},{"id":"cms","label":"Content CMS","tag":"CMS","role":"store"}],"edges":[{"from":"client","to":"cdn","label":"HTTPS","role":"external"},{"from":"cdn","to":"app","label":"SSR","role":"primary"},{"from":"app","to":"mdx","label":"READ"},{"from":"app","to":"cms","label":"QUERY"}]}
```

### `sequence` — actor lifelines with top-to-bottom messages

For request / response timelines between actors. Actors render across the top with vertical lifelines; messages render in array order from top to bottom. Actor `role` is `"default"` or `"focal"` only. Message `role` reuses the flowchart/architecture edge roles: `"default"`, `"primary"`, `"external"`. Use `kind:"return"` for response / return messages; right-to-left messages are also rendered as dashed returns.

```diagram
{"type":"sequence","actors":[{"id":"user","label":"User","tag":"CLIENT"},{"id":"api","label":"API Gateway","tag":"MW","role":"focal"},{"id":"db","label":"Database","tag":"STORE"}],"messages":[{"from":"user","to":"api","label":"POST /login","role":"primary"},{"from":"api","to":"db","label":"SELECT user"},{"from":"db","to":"api","label":"row","kind":"return"},{"from":"api","to":"user","label":"200 OK","kind":"return","role":"primary"}]}
```

### `state` — finite state machine with start/end dots

For order status, auth lifecycle, job queues, and connection states. States use `kind`: `"state"` (default), `"start"` / `"initial"`, or `"end"` / `"final"`. Use state `role:"focal"` for the one state to emphasize. Transitions use `from`, `to`, optional `label`, and edge `role`.

```diagram
{"type":"state","orientation":"lr","states":[{"id":"draft","label":"Draft","kind":"start"},{"id":"pending","label":"Pending Payment"},{"id":"paid","label":"Paid"},{"id":"done","label":"Done","kind":"end","role":"focal"}],"transitions":[{"from":"draft","to":"pending","label":"submit"},{"from":"pending","to":"paid","label":"pay","role":"primary"},{"from":"paid","to":"done","label":"complete"}]}
```

### `er` — entity relationship / data model

For database schemas and domain models. Entities have `id`, `name`, optional `role:"focal"`, and `fields`. Fields use `name`, optional `type`, and `key`: `"pk"`, `"fk"`, or omitted. Relationships use `from`, `to`, `from_cardinality`, `to_cardinality`, optional `label`, and edge `role`.

```diagram
{"type":"er","entities":[{"id":"user","name":"User","role":"focal","fields":[{"name":"id","type":"uuid","key":"pk"},{"name":"email","type":"text"}]},{"id":"order","name":"Order","fields":[{"name":"id","type":"uuid","key":"pk"},{"name":"user_id","type":"uuid","key":"fk"},{"name":"total","type":"money"}]}],"relationships":[{"from":"user","to":"order","from_cardinality":"1","to_cardinality":"N","label":"places","role":"primary"}]}
```

### `timeline` — milestones on a horizontal date axis

For release history, incident timelines, project plans, and roadmaps. Events use ISO-ish `time`, `label`, optional `sublabel`, and optional `role:"major"` for the highlighted milestone. Add optional top-level `axis_label`.

```diagram
{"type":"timeline","axis_label":"2026 release","events":[{"time":"2026-01-10","label":"Kickoff"},{"time":"2026-02-20","label":"Beta","role":"major"},{"time":"2026-04-01","label":"Launch","sublabel":"public"}]}
```

### `swimlane` — cross-functional process lanes

For multi-team handoffs and ownership flows. Lanes have `id` and `label`; steps have `id`, `lane`, `label`, optional `sublabel`, and optional `role:"focal"`. Edges connect steps with optional `label` and edge `role`.

```diagram
{"type":"swimlane","lanes":[{"id":"pm","label":"Product"},{"id":"eng","label":"Engineering"},{"id":"ops","label":"Operations"}],"steps":[{"id":"brief","lane":"pm","label":"Write brief"},{"id":"build","lane":"eng","label":"Build","role":"focal"},{"id":"deploy","lane":"ops","label":"Deploy"},{"id":"announce","lane":"pm","label":"Announce"}],"edges":[{"from":"brief","to":"build","label":"handoff","role":"primary"},{"from":"build","to":"deploy"},{"from":"deploy","to":"announce"}]}
```

### `nested` — containment rings for scope hierarchy

For repo/crate/module scope, trust zones, folder nesting, and blast-radius boundaries. Levels are ordered outer-to-inner. Use `role:"focal"` on the one innermost or important level.

```diagram
{"type":"nested","levels":[{"label":"Repo","sublabel":"workspace"},{"label":"Crate","sublabel":"makepad-diagram-kit"},{"label":"Module","role":"focal"}]}
```

### `venn` — set overlap / sweet spot

Prefer 2 or 3 sets. Sets use `id`, `label`, optional `sublabel`, optional `radius`. Intersections use `sets` array, `label`, and optional `role:"focal"`.

```diagram
{"type":"venn","sets":[{"id":"desirable","label":"Desirable"},{"id":"feasible","label":"Feasible"},{"id":"viable","label":"Viable"}],"intersections":[{"sets":["desirable","feasible","viable"],"label":"Product","role":"focal"}]}
```

### Rules

- Editorial density: target 4 to 10 primary elements per diagram; if you need more, split into two diagrams.
- One accent max per diagram. For role-driven diagrams, use at most one `"focal"` node / actor / state / entity / step / level / intersection; otherwise omit emphasis.
- Labels stay short (≤ 2-3 words). Put detail in `sublabel`, not in the main label.
- Keep the JSON body under 200 KB; the parser rejects larger.
- The fence body must be strictly JSON — no comments, no trailing commas.

## Fence nesting — use 4+ backticks for OUTER wrappers

CommonMark closes a 3-backtick fence at the next 3-backtick sequence — there is no nesting of same-length fences. If you want to show MARKDOWN SOURCE that itself contains fenced blocks, wrap the outer demo in at least **four backticks**."#.to_string(),
        }
    }
}

#[derive(Script, ScriptHook)]
pub struct App {
    #[live]
    ui: WidgetRef,
    #[rust]
    agent: Option<Box<dyn Agent>>,
    #[rust]
    chat_session_id: Option<SessionId>,
    #[rust]
    appgen_session_id: Option<SessionId>,
    #[rust]
    current_prompt: Option<PromptId>,
    #[rust]
    current_prompt_workspace: Option<ActiveWorkspace>,
    #[rust]
    active_workspace: ActiveWorkspace,
    #[rust]
    available_backends: Vec<BackendType>,
    #[rust]
    active_backend: Option<BackendType>,
    #[rust]
    chat_history_injected: bool,
    #[rust]
    appgen_history_injected: bool,
    #[rust]
    moonshot_thinking_enabled: bool,
    #[rust]
    app_state_timer: Timer,
    #[rust]
    glass_ripple_next_frame: NextFrame,
    #[rust]
    glass_ripple_started_at: f64,
    #[rust]
    glass_ripple_animating: bool,
}

impl App {
    fn default_backend(available_backends: &[BackendType]) -> Option<BackendType> {
        if available_backends.contains(&BackendType::Moonshot) {
            Some(BackendType::Moonshot)
        } else if available_backends.contains(&BackendType::ClaudeSplash) {
            Some(BackendType::ClaudeSplash)
        } else if available_backends.contains(&BackendType::GeminiSplash) {
            Some(BackendType::GeminiSplash)
        } else {
            available_backends.first().copied()
        }
    }

    fn detect_available_backends() -> Vec<BackendType> {
        let mut available_backends = vec![];
        if ClaudeCodeCliAgent::is_available() {
            available_backends.push(BackendType::ClaudeCode);
        }
        if ClaudeAcpAgent::is_available() {
            available_backends.push(BackendType::ClaudeSplash);
            available_backends.push(BackendType::ClaudeAcp);
        }
        if Self::read_key_file("ANTHROPIC_API_KEY").is_some() {
            available_backends.push(BackendType::ClaudeApi);
        }
        if Self::read_key_file("GOOGLE_API_KEY").is_some() {
            available_backends.push(BackendType::Gemini);
            available_backends.push(BackendType::GeminiSplash);
        }
        if Self::read_key_file("OPENAI_API_KEY").is_some() {
            available_backends.push(BackendType::OpenAi);
        }
        if Self::read_key("MOONSHOT_API_KEY").is_some() {
            available_backends.push(BackendType::Moonshot);
        }
        available_backends
    }

    fn initial_moonshot_thinking_enabled() -> bool {
        std::env::var("MOONSHOT_THINKING").ok().as_deref() == Some("enabled")
    }

    fn read_key_file(path: &str) -> Option<String> {
        // CWD first (desktop), then the platform key/config dir (mobile/OHOS,
        // where CWD is not writable but the app sandbox data dir is). Set via
        // set_key_config_dir() at startup with cx.get_data_dir().
        std::fs::read_to_string(path)
            .ok()
            .or_else(|| {
                KEY_CONFIG_DIR
                    .read()
                    .ok()
                    .and_then(|dir| dir.clone())
                    .and_then(|dir| {
                        std::fs::read_to_string(std::path::Path::new(&dir).join(path)).ok()
                    })
            })
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    /// Read a key/config value: env var first, then a same-named file in CWD
    /// or the platform key/config dir.
    fn read_key(name: &str) -> Option<String> {
        std::env::var(name)
            .ok()
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.trim().to_string())
            .or_else(|| Self::read_key_file(name))
    }

    fn create_agent(&self, backend: BackendType) -> Option<Box<dyn Agent>> {
        match backend {
            BackendType::ClaudeCode => ClaudeCodeCliAgent::is_available()
                .then(|| Box::new(ClaudeCodeCliAgent::new()) as Box<dyn Agent>),
            BackendType::ClaudeSplash | BackendType::ClaudeAcp => ClaudeAcpAgent::is_available()
                .then(|| Box::new(ClaudeAcpAgent::new()) as Box<dyn Agent>),
            BackendType::ClaudeApi => Self::read_key_file("ANTHROPIC_API_KEY").map(|key| {
                Box::new(StatelessBackendAdapter::new(Box::new(ClaudeBackend::new(
                    BackendConfig::Claude {
                        api_key: Some(key),
                        oauth_token: None,
                        model: "claude-sonnet-4-5-20250929".to_string(),
                    },
                )))) as Box<dyn Agent>
            }),
            BackendType::Gemini | BackendType::GeminiSplash => {
                Self::read_key_file("GOOGLE_API_KEY").map(|key| {
                    Box::new(StatelessBackendAdapter::new(Box::new(GeminiBackend::new(
                        BackendConfig::Gemini {
                            api_key: key,
                            model: "gemini-3-pro-preview".to_string(),
                        },
                    )))) as Box<dyn Agent>
                })
            }
            BackendType::OpenAi => Self::read_key_file("OPENAI_API_KEY").map(|key| {
                Box::new(StatelessBackendAdapter::new(Box::new(OpenAiBackend::new(
                    BackendConfig::OpenAI {
                        api_key: key,
                        model: "gpt-4o".to_string(),
                        base_url: None,
                        reasoning_effort: None,
                        thinking: None,
                        max_tokens: None,
                        temperature: None,
                    },
                )))) as Box<dyn Agent>
            }),
            BackendType::Moonshot => Self::read_key("MOONSHOT_API_KEY").map(|key| {
                let model = Self::read_key("MOONSHOT_MODEL")
                    .unwrap_or_else(|| "kimi-k2.6".to_string());
                let base_url = Self::read_key("MOONSHOT_BASE_URL").unwrap_or_else(|| {
                    "https://api.moonshot.ai/v1/chat/completions".to_string()
                });
                let thinking_enabled = self.moonshot_thinking_enabled;
                let thinking = if thinking_enabled {
                    "enabled"
                } else {
                    "disabled"
                }
                .to_string();
                let max_tokens = std::env::var("MOONSHOT_MAX_TOKENS")
                    .ok()
                    .and_then(|v| v.parse::<u32>().ok())
                    .or_else(|| thinking_enabled.then_some(16_000));
                let temperature = std::env::var("MOONSHOT_TEMPERATURE")
                    .ok()
                    .and_then(|v| v.parse::<f32>().ok())
                    .or_else(|| thinking_enabled.then_some(1.0));
                Box::new(StatelessBackendAdapter::new(Box::new(OpenAiBackend::new(
                    BackendConfig::OpenAI {
                        api_key: key,
                        model,
                        base_url: Some(base_url),
                        reasoning_effort: None,
                        thinking: Some(thinking),
                        max_tokens,
                        temperature,
                    },
                )))) as Box<dyn Agent>
            }),
        }
    }

    fn is_history_injected(&self, workspace: ActiveWorkspace) -> bool {
        match workspace {
            ActiveWorkspace::Chat => self.chat_history_injected,
            ActiveWorkspace::AppGen => self.appgen_history_injected,
        }
    }

    fn set_history_injected(&mut self, workspace: ActiveWorkspace, injected: bool) {
        match workspace {
            ActiveWorkspace::Chat => self.chat_history_injected = injected,
            ActiveWorkspace::AppGen => self.appgen_history_injected = injected,
        }
    }

    fn session_id_for(&self, workspace: ActiveWorkspace) -> Option<SessionId> {
        match workspace {
            ActiveWorkspace::Chat => self.chat_session_id,
            ActiveWorkspace::AppGen => self.appgen_session_id,
        }
    }

    fn set_session_id_for(&mut self, workspace: ActiveWorkspace, session_id: Option<SessionId>) {
        match workspace {
            ActiveWorkspace::Chat => self.chat_session_id = session_id,
            ActiveWorkspace::AppGen => self.appgen_session_id = session_id,
        }
    }

    fn reset_all_history_injected(&mut self) {
        self.chat_history_injected = false;
        self.appgen_history_injected = false;
    }

    fn update_workspace_ui(&self, cx: &mut Cx) {
        match self.active_workspace {
            ActiveWorkspace::Chat => {
                self.ui
                    .label(cx, ids!(sidebar_subtitle))
                    .set_text(cx, "Chat workspace");
                self.ui
                    .label(cx, ids!(workspace_title))
                    .set_text(cx, "AI Chat");
                self.ui
                    .label(cx, ids!(empty_title))
                    .set_text(cx, "我们该做什么？");
                self.ui
                    .label(cx, ids!(empty_subtitle))
                    .set_text(cx, "输入自然语言，生成可交互的 Makepad diagram。");
                self.ui
                    .text_input(cx, ids!(input))
                    .set_empty_text(cx, "问任何事。输入 @ 使用插件或提及文件".to_string());
                self.ui.widget(cx, ids!(nav_chat)).set_text(cx, "●  会话");
                self.ui
                    .widget(cx, ids!(nav_appgen))
                    .set_text(cx, "◇  App 生成");
            }
            ActiveWorkspace::AppGen => {
                self.ui
                    .label(cx, ids!(sidebar_subtitle))
                    .set_text(cx, "App generation workspace");
                self.ui
                    .label(cx, ids!(workspace_title))
                    .set_text(cx, "App 生成");
                self.ui
                    .label(cx, ids!(empty_title))
                    .set_text(cx, "生成一个可点击的 App");
                self.ui
                    .label(cx, ids!(empty_subtitle))
                    .set_text(cx, "这个 tab 的 prompt、历史和普通会话隔离。");
                self.ui
                    .text_input(cx, ids!(input))
                    .set_empty_text(cx, "描述要生成的 app，例如：画一个计数器".to_string());
                self.ui.widget(cx, ids!(nav_chat)).set_text(cx, "○  会话");
                self.ui
                    .widget(cx, ids!(nav_appgen))
                    .set_text(cx, "◆  App 生成");
            }
        }
    }

    fn switch_workspace(&mut self, cx: &mut Cx, workspace: ActiveWorkspace) {
        if self.active_workspace == workspace {
            return;
        }
        self.active_workspace = workspace;
        *ACTIVE_WORKSPACE.write().unwrap() = workspace;
        self.update_workspace_ui(cx);
        self.update_empty_state_visibility(cx);
        self.ui.redraw(cx);
    }

    fn prompt_for_workspace(&self, workspace: ActiveWorkspace, text: &str) -> String {
        match workspace {
            ActiveWorkspace::Chat => prompt_with_state(workspace.prompt_section(), text),
            ActiveWorkspace::AppGen => app_generation_prompt_with_state(text),
        }
    }

    fn switch_backend(&mut self, cx: &mut Cx, backend: BackendType) {
        if self.active_backend == Some(backend) {
            return;
        }
        self.activate_backend(cx, backend);
    }

    fn restart_backend(&mut self, cx: &mut Cx, backend: BackendType) {
        self.activate_backend(cx, backend);
    }

    fn activate_backend(&mut self, cx: &mut Cx, backend: BackendType) {
        if let Some(mut agent) = self.create_agent(backend) {
            let chat_config = SessionConfig {
                system_prompt: Some(backend.system_prompt()),
                ..Default::default()
            };
            let appgen_config = SessionConfig {
                system_prompt: Some(app_generation_session_system_prompt()),
                ..Default::default()
            };
            let chat_session_id = agent.create_session(cx, chat_config);
            let appgen_session_id = agent.create_session(cx, appgen_config);

            self.agent = Some(agent);
            self.active_backend = Some(backend);
            self.chat_session_id = Some(chat_session_id);
            self.appgen_session_id = Some(appgen_session_id);
            self.current_prompt = None;
            self.current_prompt_workspace = None;
            self.reset_all_history_injected();
            self.update_status(cx);
        }
    }

    fn clear_chat(&mut self, cx: &mut Cx) {
        let workspace = self.active_workspace;
        {
            let mut data = chat_data_for_workspace(workspace).write().unwrap();
            data.messages.clear();
            data.streaming_text.clear();
            data.thinking_text.clear();
            data.is_streaming = false;
            data.save_to_disk(workspace.save_path());
        }
        self.set_history_injected(workspace, false);

        let new_session_id = if let Some(agent) = &mut self.agent {
            let backend = self.active_backend.unwrap_or(BackendType::Gemini);
            let system_prompt = match workspace {
                ActiveWorkspace::Chat => backend.system_prompt(),
                ActiveWorkspace::AppGen => app_generation_session_system_prompt(),
            };
            let config = SessionConfig {
                system_prompt: Some(system_prompt),
                ..Default::default()
            };
            Some(agent.create_session(cx, config))
        } else {
            None
        };
        if let Some(session_id) = new_session_id {
            self.set_session_id_for(workspace, Some(session_id));
        }
        self.update_empty_state_visibility(cx);
        self.ui.redraw(cx);
    }

    fn update_empty_state_visibility(&self, cx: &mut Cx) {
        let show_empty_state = {
            let data = chat_data_for_workspace(self.active_workspace)
                .read()
                .unwrap();
            data.messages.is_empty() && !data.is_streaming
        };
        self.ui
            .view(cx, ids!(empty_state))
            .set_visible(cx, show_empty_state);
    }

    fn send_prompt_to_agent(
        &mut self,
        cx: &mut Cx,
        workspace: ActiveWorkspace,
        display_text: String,
        prompt_text: String,
    ) {
        if display_text.trim().is_empty() || prompt_text.trim().is_empty() {
            return;
        }

        if self.agent.is_none() || self.session_id_for(workspace).is_none() {
            return;
        }

        let items_len = {
            let mut data = chat_data_for_workspace(workspace).write().unwrap();
            data.messages.push(ChatMessage {
                role: ChatRole::User,
                text: display_text,
            });
            data.streaming_text.clear();
            data.thinking_text.clear();
            data.is_streaming = true;
            data.messages.len() + 1
        };
        self.update_empty_state_visibility(cx);

        let session_id = self.session_id_for(workspace).unwrap();

        // Inject history on first prompt for stateless backends
        let history_to_inject = if !self.is_history_injected(workspace)
            && self.agent.as_ref().unwrap().is_stateless()
        {
            let data = chat_data_for_workspace(workspace).read().unwrap();
            let history = stateless_history_messages(&data.messages[..data.messages.len() - 1]);
            drop(data);
            self.set_history_injected(workspace, true);
            (!history.is_empty()).then_some(history)
        } else {
            None
        };

        // ACP doesn't support system prompts via the protocol, so for ClaudeSplash
        // we prepend the splash system prompt context to each user message.
        let prompt_text = if self.active_backend == Some(BackendType::ClaudeSplash) {
            let system = match workspace {
                ActiveWorkspace::Chat => BackendType::ClaudeSplash.system_prompt(),
                ActiveWorkspace::AppGen => app_generation_session_system_prompt(),
            };
            format!("<system>\n{system}\n</system>\n\n{prompt_text}")
        } else {
            prompt_text
        };
        let agent = self.agent.as_mut().unwrap();
        if let Some(history) = history_to_inject {
            agent.inject_history(session_id, history);
        }
        self.current_prompt = Some(agent.send_prompt(cx, session_id, &prompt_text));
        self.current_prompt_workspace = Some(workspace);
        self.ui.view(cx, ids!(cancel_button)).set_visible(cx, true);

        if self.active_workspace == workspace {
            let chat_list = self.ui.widget(cx, ids!(chat_list));
            let list = chat_list.portal_list(cx, ids!(list));
            list.set_tail_range(true);
            list.set_first_id_and_scroll(items_len.saturating_sub(1), 0.0);
        }
        self.ui.redraw(cx);
    }

    fn send_message(&mut self, cx: &mut Cx) {
        let input = self.ui.text_input(cx, ids!(input));
        let text = input.text();
        if text.trim().is_empty() {
            return;
        }

        input.set_text(cx, "");
        let workspace = self.active_workspace;
        let prompt_text = self.prompt_for_workspace(workspace, &text);
        self.send_prompt_to_agent(cx, workspace, text, prompt_text);
    }

    fn mutate_demo_count<F>(&self, f: F)
    where
        F: FnOnce(i64) -> i64,
    {
        let mut state = APP_DEMO_STATE.write().unwrap();
        state.count = f(state.count);
        state.save_to_disk();
    }

    fn mutate_timer<F>(&self, f: F)
    where
        F: FnOnce(&mut TimerDemoState),
    {
        let mut state = APP_DEMO_STATE.write().unwrap();
        f(&mut state.timer);
        state.save_to_disk();
    }

    fn tick_timer_state(&self) -> bool {
        let mut state = APP_DEMO_STATE.write().unwrap();
        if !state.timer.is_running {
            return false;
        }
        if state.timer.remaining_seconds > 0 {
            state.timer.remaining_seconds -= 1;
        }
        if state.timer.remaining_seconds <= 0 {
            state.timer.remaining_seconds = 0;
            state.timer.is_running = false;
        }
        state.save_to_disk();
        true
    }

    fn refresh_visible_state_templates(&self, cx: &mut Cx) {
        let messages: Vec<(usize, String)> = {
            let data = chat_data_for_workspace(self.active_workspace)
                .read()
                .unwrap();
            data.messages
                .iter()
                .enumerate()
                .filter_map(|(index, msg)| {
                    (msg.role == ChatRole::Assistant).then(|| (index, msg.text.clone()))
                })
                .collect()
        };
        let state = APP_DEMO_STATE.read().unwrap();
        let chat_list = self.ui.widget(cx, ids!(chat_list));
        let list = chat_list.portal_list(cx, ids!(list));

        for (item_id, text) in messages {
            let Some((_, item)) = list.get_item(item_id) else {
                continue;
            };
            let unwrapped = unwrap_outer_markdown_fence(&text);
            let state_rendered = render_state_templates(unwrapped, &state);
            let rendered = wrap_bare_latex(&state_rendered);
            let mut markdown = item.markdown(cx, ids!(selectable));
            markdown.set_text(cx, &rendered);
        }
        cx.redraw_all();
    }

    fn handle_splash_event(&mut self, cx: &mut Cx, event_id: &str, payload: &str) {
        match event_id {
            "inc" => {
                self.mutate_demo_count(|count| count + 1);
                self.refresh_visible_state_templates(cx);
            }
            "dec" => {
                self.mutate_demo_count(|count| count - 1);
                self.refresh_visible_state_templates(cx);
            }
            "reset" => {
                self.mutate_demo_count(|_| 0);
                self.refresh_visible_state_templates(cx);
            }
            "timer.start" => {
                self.mutate_timer(|timer| {
                    if timer.remaining_seconds <= 0 {
                        timer.remaining_seconds = timer.duration_seconds.max(60);
                    }
                    timer.is_running = true;
                });
                self.refresh_visible_state_templates(cx);
            }
            "timer.pause" => {
                self.mutate_timer(|timer| {
                    timer.is_running = false;
                });
                self.refresh_visible_state_templates(cx);
            }
            "timer.toggle" => {
                self.mutate_timer(|timer| {
                    if timer.is_running {
                        timer.is_running = false;
                    } else {
                        if timer.remaining_seconds <= 0 {
                            timer.remaining_seconds = timer.duration_seconds.max(60);
                        }
                        timer.is_running = true;
                    }
                });
                self.refresh_visible_state_templates(cx);
            }
            "timer.reset" => {
                self.mutate_timer(|timer| timer.reset());
                self.refresh_visible_state_templates(cx);
            }
            "timer.add_minute" => {
                self.mutate_timer(|timer| {
                    timer.duration_seconds += 60;
                    timer.remaining_seconds += 60;
                });
                self.refresh_visible_state_templates(cx);
            }
            "timer.subtract_minute" => {
                self.mutate_timer(|timer| {
                    timer.duration_seconds = (timer.duration_seconds - 60).max(60);
                    timer.remaining_seconds = (timer.remaining_seconds - 60).max(0);
                    if timer.remaining_seconds == 0 {
                        timer.is_running = false;
                    }
                });
                self.refresh_visible_state_templates(cx);
            }
            "ask_ai" => {
                let workspace = self.active_workspace;
                let body = format!("User clicked \"ask_ai\" with payload: {}", payload);
                let prompt_text = self.prompt_for_workspace(workspace, &body);
                self.send_prompt_to_agent(cx, workspace, body, prompt_text);
            }
            "" => {
                log!("[splash] ignored empty agent.notify event");
            }
            other => {
                log!("[splash] unknown event: {}", other);
            }
        }
    }

    fn cancel_request(&mut self, cx: &mut Cx) {
        if let (Some(agent), Some(prompt_id)) = (&mut self.agent, self.current_prompt.take()) {
            agent.cancel_prompt(cx, prompt_id);

            let workspace = self
                .current_prompt_workspace
                .unwrap_or(self.active_workspace);
            let mut data = chat_data_for_workspace(workspace).write().unwrap();
            let text = std::mem::take(&mut data.streaming_text);
            data.thinking_text.clear();
            if !text.is_empty() {
                data.messages.push(ChatMessage {
                    role: ChatRole::Assistant,
                    text,
                });
            }
            data.is_streaming = false;
            drop(data);

            self.current_prompt_workspace = None;
            self.update_empty_state_visibility(cx);
            self.ui.view(cx, ids!(cancel_button)).set_visible(cx, false);
            self.ui.redraw(cx);
        }
    }

    fn update_status(&self, cx: &mut Cx) {
        let status = match self.active_backend {
            Some(BackendType::Moonshot) if self.moonshot_thinking_enabled => {
                "Active: Moonshot · Thinking on"
            }
            Some(BackendType::Moonshot) => "Active: Moonshot · Thinking off",
            Some(b) => b.status_label(),
            None => "No backend selected",
        };
        self.ui.label(cx, ids!(status_label)).set_text(cx, status);
    }

    fn set_glass_press_response(
        &mut self,
        cx: &mut Cx,
        flatten: f32,
        ripple_start: f32,
        ripple_strength: f32,
    ) {
        if let Some(mut glass) = self
            .ui
            .widget(cx, ids!(composer))
            .borrow_mut::<GaussRoundedView>()
        {
            glass.set_press_response(cx, flatten, ripple_start, ripple_strength);
        }
        if let Some(mut glass) = self
            .ui
            .widget(cx, ids!(backend_toolbar))
            .borrow_mut::<GaussRoundedView>()
        {
            glass.set_press_response(cx, flatten, ripple_start, ripple_strength);
        }
        if let Some(mut glass) = self
            .ui
            .widget(cx, ids!(opacity_toolbar))
            .borrow_mut::<GaussRoundedView>()
        {
            glass.set_press_response(cx, flatten, ripple_start, ripple_strength);
        }
    }

    fn start_glass_ripple(&mut self, cx: &mut Cx) {
        self.glass_ripple_animating = true;
        self.glass_ripple_started_at = 0.0;
        self.set_glass_press_response(cx, 0.0, -1000.0, 0.0);
        self.glass_ripple_next_frame = cx.new_next_frame();
    }

    fn apply_glass_opacity(&self, cx: &mut Cx, opacity: f64) {
        let opacity = opacity.clamp(MIN_GLASS_OPACITY, MAX_GLASS_OPACITY);
        let glass = glass_opacity_values(opacity);

        let mut app_shell = self.ui.view(cx, ids!(app_shell));
        script_apply_eval!(cx, app_shell, {
            draw_bg +: { tint_alpha: #(glass.app) }
        });

        let mut sidebar = self.ui.view(cx, ids!(sidebar));
        script_apply_eval!(cx, sidebar, {
            draw_bg +: { tint_alpha: #(glass.sidebar) }
        });

        let mut main_area = self.ui.view(cx, ids!(main_area));
        script_apply_eval!(cx, main_area, {
            draw_bg +: { tint_alpha: #(glass.main) }
        });

        let mut composer = self.ui.widget(cx, ids!(composer));
        script_apply_eval!(cx, composer, {
            draw_bg +: { tint_alpha: #(glass.composer) }
        });

        self.ui
            .label(cx, ids!(opacity_value))
            .set_text(cx, &format!("{:.0}%", opacity * 100.0));
        self.ui.redraw(cx);
    }
}

impl MatchEvent for App {
    fn handle_next_frame(&mut self, cx: &mut Cx, e: &NextFrameEvent) {
        if !self.glass_ripple_animating || !e.set.contains(&self.glass_ripple_next_frame) {
            return;
        }

        if self.glass_ripple_started_at <= 0.0 {
            self.glass_ripple_started_at = e.time;
        }
        let age = (e.time - self.glass_ripple_started_at).max(0.0);
        let ripple_strength = ((1.0 - age / 1.05).max(0.0) as f32) * 0.82;
        self.set_glass_press_response(
            cx,
            0.0,
            self.glass_ripple_started_at as f32,
            ripple_strength,
        );

        if age < 1.08 {
            self.glass_ripple_next_frame = cx.new_next_frame();
        } else {
            self.glass_ripple_animating = false;
            self.set_glass_press_response(cx, 0.0, -1000.0, 0.0);
        }
    }

    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        for action in actions {
            if let SplashAction::Notify { event_id, payload } = action.cast() {
                self.handle_splash_event(cx, &event_id, &payload);
            }
        }

        let opacity_slider = self.ui.slider(cx, ids!(opacity_slider));
        if let Some(opacity) = opacity_slider
            .slided(actions)
            .or_else(|| opacity_slider.end_slide(actions))
        {
            self.apply_glass_opacity(cx, opacity);
            self.start_glass_ripple(cx);
        }
        if let Some(enabled) = self
            .ui
            .check_box(cx, ids!(thinking_toggle))
            .changed(actions)
        {
            self.moonshot_thinking_enabled = enabled;
            if self.active_backend == Some(BackendType::Moonshot) {
                if self.current_prompt.is_some() {
                    self.cancel_request(cx);
                }
                self.restart_backend(cx, BackendType::Moonshot);
            } else {
                self.update_status(cx);
            }
        }

        // Markdown link click — dispatch through robius-open for cross-platform
        // coverage (macOS/Linux/Windows/iOS/Android/WASM). Desktop requires a
        // modifier (Cmd on macOS, Cmd/Ctrl elsewhere) so plain clicks stay
        // available for drag-selection inside the Markdown widget; mobile &
        // web have no modifier concept, so a plain tap opens the URL.
        for action in actions {
            if let Some(widget_action) = action.as_widget_action() {
                if let makepad_widgets::markdown::MarkdownAction::LinkNavigated { url, modifiers } =
                    widget_action.cast()
                {
                    let should_open = {
                        #[cfg(any(
                            target_os = "ios",
                            target_os = "android",
                            target_arch = "wasm32"
                        ))]
                        {
                            let _ = modifiers;
                            true
                        }
                        #[cfg(not(any(
                            target_os = "ios",
                            target_os = "android",
                            target_arch = "wasm32"
                        )))]
                        {
                            modifiers.logo || modifiers.control
                        }
                    };
                    if should_open {
                        if let Err(e) = robius_open::Uri::new(&url).open() {
                            log::warn!("failed to open URL {}: {:?}", url, e);
                        }
                    }
                }
            }
        }
        if self.ui.button(cx, ids!(send_button)).clicked(actions) {
            self.start_glass_ripple(cx);
            self.send_message(cx);
        }
        if self.ui.button(cx, ids!(cancel_button)).clicked(actions) {
            self.start_glass_ripple(cx);
            self.cancel_request(cx);
        }
        if self.ui.button(cx, ids!(clear_button)).clicked(actions) {
            self.start_glass_ripple(cx);
            self.clear_chat(cx);
        }
        if self.ui.button(cx, ids!(nav_chat)).clicked(actions) {
            self.switch_workspace(cx, ActiveWorkspace::Chat);
        }
        if self.ui.button(cx, ids!(nav_appgen)).clicked(actions) {
            self.switch_workspace(cx, ActiveWorkspace::AppGen);
        }
        if self
            .ui
            .text_input(cx, ids!(input))
            .returned(actions)
            .is_some()
        {
            self.start_glass_ripple(cx);
            self.send_message(cx);
        }
        if self.ui.text_input(cx, ids!(input)).escaped(actions) {
            self.cancel_request(cx);
        }
        if let Some(index) = self
            .ui
            .drop_down(cx, ids!(backend_dropdown))
            .selected(actions)
        {
            if let Some(backend) = BackendType::from_index(index) {
                self.switch_backend(cx, backend);
                self.start_glass_ripple(cx);
            }
        }

        // Handle message deletion
        let chat_list = self.ui.widget(cx, ids!(chat_list));
        let list = chat_list.portal_list(cx, ids!(list));
        for (item_id, item) in list.items_with_actions(actions) {
            if item.button(cx, ids!(delete_button)).pressed(actions) {
                let workspace = self.active_workspace;
                let mut data = chat_data_for_workspace(workspace).write().unwrap();
                if item_id < data.messages.len() {
                    data.messages.remove(item_id);
                    data.save_to_disk(workspace.save_path());
                }
                drop(data);
                self.update_empty_state_visibility(cx);
                self.ui.redraw(cx);
            }
        }
    }

    fn handle_startup(&mut self, cx: &mut Cx) {
        self.active_workspace = ActiveWorkspace::Chat;
        *ACTIVE_WORKSPACE.write().unwrap() = self.active_workspace;
        self.app_state_timer = cx.start_interval(1.0);
        // On platforms with a sandbox data dir (OHOS/mobile), point the key
        // file search at it and re-detect backends now that the dir is known.
        // detect_available_backends() ran in after_new_from_script without cx.
        if let Some(data_dir) = cx.get_data_dir() {
            if let Ok(mut dir) = KEY_CONFIG_DIR.write() {
                *dir = Some(data_dir.clone());
            }
            log!("aichat: key/config dir set to {}", data_dir);
            self.available_backends = Self::detect_available_backends();
            log!("aichat: backends after re-detect: {:?}", self.available_backends);
        }
        let default_backend = Self::default_backend(&self.available_backends);
        self.ui
            .popup_notification(cx, ids!(toolbar_overlay))
            .open(cx);
        if let Some(backend) = default_backend {
            self.switch_backend(cx, backend);
            self.ui
                .drop_down(cx, ids!(backend_dropdown))
                .set_selected_item(cx, backend.to_index());
        }
        self.update_status(cx);
        self.update_workspace_ui(cx);
        self.update_empty_state_visibility(cx);
        self.ui
            .slider(cx, ids!(opacity_slider))
            .set_value(cx, DEFAULT_GLASS_OPACITY);
        self.ui.check_box(cx, ids!(thinking_toggle)).set_active(
            cx,
            self.moonshot_thinking_enabled,
            Animate::No,
        );
        self.apply_glass_opacity(cx, DEFAULT_GLASS_OPACITY);
        self.start_glass_ripple(cx);
    }

    fn handle_timer(&mut self, cx: &mut Cx, event: &TimerEvent) {
        if self.app_state_timer.is_timer(event).is_some() && self.tick_timer_state() {
            self.refresh_visible_state_templates(cx);
        }
    }
}

impl AppMain for App {
    fn script_mod(vm: &mut ScriptVm) -> ScriptValue {
        crate::makepad_widgets::script_mod(vm);
        crate::makepad_widgets::register_agent_module(vm);
        crate::makepad_code_editor::script_mod(vm);
        crate::makepad_diagram_kit::script_mod(vm);
        self::script_mod(vm)
    }

    fn after_new_from_script(_vm: &mut ScriptVm, app: &mut Self) {
        CHAT_DATA.write().unwrap().messages = ChatData::load_from_disk(CHAT_SAVE_PATH);
        APP_GEN_DATA.write().unwrap().messages = ChatData::load_from_disk(APP_GEN_SAVE_PATH);
        *APP_DEMO_STATE.write().unwrap() = AppDemoState::load_from_disk();
        app.available_backends = Self::detect_available_backends();
        app.moonshot_thinking_enabled = Self::initial_moonshot_thinking_enabled();
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

        self.match_event(cx, event);
        self.ui.handle_event(cx, event, &mut Scope::empty());

        if let Some(agent) = &mut self.agent {
            for event in agent.handle_event(cx, event) {
                match event {
                    AgentEvent::SessionReady { .. } => {
                        self.update_status(cx);
                    }
                    AgentEvent::SessionError { error, .. } => {
                        self.ui
                            .label(cx, ids!(status_label))
                            .set_text(cx, &format!("Error: {}", error));
                    }
                    AgentEvent::TextDelta { text, .. } => {
                        log!("aichat UI text delta chars={}", text.chars().count());
                        let workspace = self
                            .current_prompt_workspace
                            .unwrap_or(self.active_workspace);
                        let item_id = {
                            let mut data = chat_data_for_workspace(workspace).write().unwrap();
                            data.streaming_text.push_str(&text);
                            data.messages.len()
                        };
                        if self.active_workspace == workspace {
                            let chat_list = self.ui.widget(cx, ids!(chat_list));
                            let list = chat_list.portal_list(cx, ids!(list));
                            if let Some((_, item)) = list.get_item(item_id) {
                                item.widget(cx, ids!(splash_view)).redraw(cx);
                            }
                        }
                        cx.redraw_all();
                    }
                    AgentEvent::ThinkingDelta { text, .. } => {
                        log!("aichat UI thinking delta chars={}", text.chars().count());
                        let workspace = self
                            .current_prompt_workspace
                            .unwrap_or(self.active_workspace);
                        {
                            let mut data = chat_data_for_workspace(workspace).write().unwrap();
                            data.thinking_text.push_str(&text);
                        }
                        self.ui
                            .label(cx, ids!(status_label))
                            .set_text(cx, "Thinking...");
                        cx.redraw_all();
                    }
                    AgentEvent::TurnComplete { .. } => {
                        let workspace = self
                            .current_prompt_workspace
                            .unwrap_or(self.active_workspace);
                        let mut data = chat_data_for_workspace(workspace).write().unwrap();
                        let text = std::mem::take(&mut data.streaming_text);
                        log!(
                            "aichat UI turn complete content_chars={}",
                            text.chars().count()
                        );
                        data.thinking_text.clear();
                        if !text.is_empty() {
                            if assistant_message_is_safe_to_store(&text) {
                                data.messages.push(ChatMessage {
                                    role: ChatRole::Assistant,
                                    text,
                                });
                            } else {
                                self.ui.label(cx, ids!(status_label)).set_text(
                                    cx,
                                    "Error: incomplete diagram response discarded; retry",
                                );
                            }
                        }
                        data.is_streaming = false;
                        data.save_to_disk(workspace.save_path());
                        drop(data);

                        self.current_prompt = None;
                        self.current_prompt_workspace = None;
                        self.ui.view(cx, ids!(cancel_button)).set_visible(cx, false);
                        self.update_empty_state_visibility(cx);
                        cx.redraw_all();
                    }
                    AgentEvent::PromptError { error, .. } => {
                        log!("aichat UI prompt error: {}", error);
                        let workspace = self
                            .current_prompt_workspace
                            .unwrap_or(self.active_workspace);
                        {
                            let mut data = chat_data_for_workspace(workspace).write().unwrap();
                            data.messages.push(ChatMessage {
                                role: ChatRole::Assistant,
                                text: format!("Error: {error}"),
                            });
                            data.is_streaming = false;
                            data.thinking_text.clear();
                            data.save_to_disk(workspace.save_path());
                        }
                        self.current_prompt = None;
                        self.current_prompt_workspace = None;
                        self.ui.view(cx, ids!(cancel_button)).set_visible(cx, false);
                        self.update_empty_state_visibility(cx);
                        self.ui
                            .label(cx, ids!(status_label))
                            .set_text(cx, &format!("Error: {}", error));
                        cx.redraw_all();
                    }
                    AgentEvent::ToolRequest { .. } => {}
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use makepad_widgets::DVec2;

    use super::{
        assistant_message_is_safe_for_history, assistant_message_is_safe_to_store,
        glass_opacity_values, render_state_templates, should_start_window_drag, Agent, App,
        AppDemoState, BackendType, ClaudeCodeCliAgent, DEFAULT_GLASS_OPACITY, MAX_GLASS_OPACITY,
        MIN_GLASS_OPACITY,
    };

    #[test]
    fn aichat_glass_opacity_slider_contract() {
        // v3: slider is a position value; shell alpha and Apple Glass tint are
        // derived separately because they use different material models.
        assert!((DEFAULT_GLASS_OPACITY - 0.90).abs() < f64::EPSILON);
        let values = glass_opacity_values(DEFAULT_GLASS_OPACITY);
        // Shell panels retain the existing layer ordering.
        assert!(values.app < values.main);
        assert!(values.main < values.sidebar);
        assert!((0.54..0.57).contains(&values.app));
        // Composer is an AppleGlassRoundedView overlay, so tint remains low.
        assert!((0.075..0.09).contains(&values.composer));
    }

    #[test]
    fn aichat_liquid_glass_shell_contract() {
        // v3: shell layer ordering is preserved, while composer tint stays in
        // the low Apple Glass range at every legal slider value.
        let low = glass_opacity_values(0.0);
        let high = glass_opacity_values(2.0);
        // Slider is clamped: low.app uses MIN_GLASS_OPACITY, high.app uses MAX.
        assert!(low.app < high.app);
        assert!(high.app > 0.57);
        assert!(high.app <= 0.59);
        for &slider in &[
            MIN_GLASS_OPACITY,
            0.30_f64,
            0.60,
            DEFAULT_GLASS_OPACITY,
            MAX_GLASS_OPACITY,
        ] {
            let v = glass_opacity_values(slider);
            assert!(v.app < v.main, "slider={}", slider);
            assert!(v.main <= v.sidebar, "slider={}", slider);
            assert!((0.035..=0.090).contains(&v.composer), "slider={}", slider);
        }
    }

    #[test]
    fn aichat_drag_strip_preserves_resize_edges() {
        let size = DVec2 { x: 900.0, y: 700.0 };
        assert!(should_start_window_drag(DVec2 { x: 120.0, y: 24.0 }, size));
        assert!(!should_start_window_drag(DVec2 { x: 4.0, y: 24.0 }, size));
        assert!(!should_start_window_drag(DVec2 { x: 120.0, y: 4.0 }, size));
        assert!(!should_start_window_drag(DVec2 { x: 880.0, y: 24.0 }, size));
        assert!(!should_start_window_drag(DVec2 { x: 700.0, y: 24.0 }, size));
    }

    #[test]
    fn aichat_backend_type_includes_claude_code() {
        assert_eq!(BackendType::ClaudeCode.to_index(), 0);
        assert_eq!(BackendType::from_index(0), Some(BackendType::ClaudeCode));
        assert_eq!(
            BackendType::ClaudeCode.status_label(),
            "Active: Claude Code"
        );
        assert!(BackendType::ClaudeCode
            .system_prompt()
            .contains("The thirteen supported types"));
    }

    #[test]
    fn aichat_create_claude_code_agent() {
        let _agent: Box<dyn Agent> = Box::new(ClaudeCodeCliAgent::new());
    }

    #[test]
    fn state_templates_render_count() {
        let mut state = AppDemoState::default();
        state.count = 7;
        assert_eq!(
            render_state_templates("Count: {{state.count}}", &state),
            "Count: 7"
        );
    }

    #[test]
    fn state_templates_preserve_unknown_path() {
        let mut state = AppDemoState::default();
        state.count = 7;
        assert_eq!(
            render_state_templates("Count: {{state.cont}}", &state),
            "Count: {{state.cont}}"
        );
    }

    #[test]
    fn state_templates_render_multiple_count_placeholders() {
        let mut state = AppDemoState::default();
        state.count = -2;
        assert_eq!(
            render_state_templates("{{state.count}} / {{state.count}}", &state),
            "-2 / -2"
        );
    }

    #[test]
    fn state_templates_render_timer_paths() {
        let mut state = AppDemoState::default();
        state.timer.remaining_seconds = 65;
        state.timer.is_running = true;
        assert_eq!(
            render_state_templates(
                "{{state.timer.display}} {{state.timer.button_label}} {{state.timer.is_running}}",
                &state,
            ),
            "01:05 Pause true"
        );
    }

    #[test]
    fn aichat_defaults_to_moonshot_when_available() {
        let available = [
            BackendType::ClaudeCode,
            BackendType::ClaudeSplash,
            BackendType::Moonshot,
        ];
        assert_eq!(
            App::default_backend(&available),
            Some(BackendType::Moonshot)
        );
    }

    #[test]
    fn non_splash_prompt_documents_sequence_diagrams() {
        let prompt = BackendType::Moonshot.system_prompt();

        assert!(prompt.contains("### `sequence`"));
        assert!(prompt.contains(r#""type":"sequence""#));
        assert!(prompt.contains(r#"kind:"return""#));
        assert!(prompt.contains(r#""kind":"return""#));
        assert!(!prompt.contains("All five types"));
    }

    #[test]
    fn non_splash_prompt_documents_all_diagram_types() {
        let prompt = BackendType::Moonshot.system_prompt();

        assert!(prompt.contains("The thirteen supported types"));
        for ty in [
            "pyramid",
            "quadrant",
            "tree",
            "layers",
            "flowchart",
            "architecture",
            "sequence",
            "state",
            "er",
            "timeline",
            "swimlane",
            "nested",
            "venn",
        ] {
            assert!(
                prompt.contains(&format!(r#""type":"{ty}""#)),
                "prompt should document {ty}"
            );
        }
        assert!(prompt.contains(r#""kind":"end""#));
        assert!(prompt.contains(r#""key":"pk""#));
        assert!(prompt.contains(r#""role":"major""#));
        assert!(!prompt.contains("The seven supported types"));
        assert!(!prompt.contains("doesn't have a native timeline type"));
    }

    #[test]
    fn history_injection_allows_valid_diagram_assistant_messages() {
        let text = r#"```diagram
{"type":"state","orientation":"lr","states":[{"id":"draft","label":"Draft","kind":"start"},{"id":"done","label":"Done","kind":"end","role":"focal"}],"transitions":[{"from":"draft","to":"done","label":"submit"}]}
```"#;

        assert!(assistant_message_is_safe_to_store(text));
        assert!(!assistant_message_is_safe_for_history(text));
    }

    #[test]
    fn history_injection_rejects_incomplete_diagram_assistant_messages() {
        let text = r#"```diagram
{"type":"state","orientation":"lr","states":[{"id":"draft","label":"Draft","kind":"start"},{"id":"pending","label":"Pending Payment"},{"id":"paid","label":"
"#;

        assert!(!assistant_message_is_safe_to_store(text));
        assert!(!assistant_message_is_safe_for_history(text));
    }

    #[test]
    fn history_injection_rejects_invalid_closed_diagram_assistant_messages() {
        let text = r#"```diagram
{"type":"state","orientation":"lr","states":[{"id":"draft","label":"Draft"}],
```"#;

        assert!(!assistant_message_is_safe_to_store(text));
        assert!(!assistant_message_is_safe_for_history(text));
    }

    #[test]
    fn history_injection_allows_non_diagram_assistant_messages() {
        let text = "这里是普通解释，没有 diagram fence。";

        assert!(assistant_message_is_safe_to_store(text));
        assert!(assistant_message_is_safe_for_history(text));
    }

    // Regression: an unclosed *non-diagram* fence (e.g. response truncated
    // mid `rust`/`mermaid` block) was discarding the entire reply because
    // FenceScanError::Unclosed was treated the same as a malformed diagram.
    #[test]
    fn store_keeps_reply_with_unclosed_non_diagram_fence() {
        let text = "Here's a markdown demo:\n\n```rust\nfn main() {\n    println!(\"hi\";\n";
        assert!(assistant_message_is_safe_to_store(text));
        assert!(!assistant_message_is_safe_for_history(text));
    }

    #[test]
    fn store_rejects_bad_diagram_even_with_later_unclosed_non_diagram_fence() {
        let text = r#"```diagram
{"type":"state","orientation":"lr","states":[{"id":"draft","label":"Draft"}],
```

```rust
fn main() {
"#;

        assert!(!assistant_message_is_safe_to_store(text));
        assert!(!assistant_message_is_safe_for_history(text));
    }

    #[test]
    fn outer_markdown_wrapper_is_unwrapped_before_diagram_safety_scan() {
        let text = r#"```markdown
Here is a diagram:

```diagram
{"type":"state","orientation":"lr","states":[{"id":"draft","label":"Draft","kind":"start"},{"id":"done","label":"Done","kind":"end"}],"transitions":[{"from":"draft","to":"done","label":"submit"}]}
```
```"#;

        assert!(assistant_message_is_safe_to_store(text));
        assert!(!assistant_message_is_safe_for_history(text));
    }

    #[test]
    fn outer_markdown_wrapper_rejects_bad_inner_diagram() {
        let text = r#"```markdown
Here is a broken diagram:

```diagram
{"type":"state","orientation":"lr","states":[{"id":"draft","label":"Draft"}],
```
```"#;

        assert!(!assistant_message_is_safe_to_store(text));
        assert!(!assistant_message_is_safe_for_history(text));
    }
}
