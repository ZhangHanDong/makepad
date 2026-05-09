use crate::{makepad_derive_widget::*, makepad_draw::*, view::View, widget::*};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Script, ScriptHook)]
pub enum GlassNativeStyle {
    #[default]
    Regular,
    Clear,
}

impl GlassNativeStyle {
    fn to_native(self) -> NativeGlassStyle {
        match self {
            Self::Regular => NativeGlassStyle::Regular,
            Self::Clear => NativeGlassStyle::Clear,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Script, ScriptHook)]
pub enum GlassNativeHitTest {
    #[default]
    Passthrough,
    Interactive,
}

impl GlassNativeHitTest {
    fn to_native(self) -> NativeGlassHitTest {
        match self {
            Self::Passthrough => NativeGlassHitTest::Passthrough,
            Self::Interactive => NativeGlassHitTest::Interactive,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Script, ScriptHook)]
pub enum GlassNativeShape {
    #[default]
    RoundedRect,
    Capsule,
}

impl GlassNativeShape {
    fn to_native(self, radius: f64) -> NativeGlassShape {
        match self {
            Self::RoundedRect => NativeGlassShape::RoundedRect { radius },
            Self::Capsule => NativeGlassShape::Capsule,
        }
    }
}

#[derive(Clone, Debug)]
struct NativeGlassCollection {
    id: LiveId,
    rect: Rect,
    spacing: f64,
    panels: Vec<NativeGlassPanelDescriptor>,
    controls: Vec<NativeGlassControlDescriptor>,
}

#[derive(Default)]
struct NativeGlassCollector {
    stack: Vec<NativeGlassCollection>,
}

impl NativeGlassCollector {
    fn begin(&mut self, id: LiveId, spacing: f64) {
        self.stack.push(NativeGlassCollection {
            id,
            rect: Rect::default(),
            spacing,
            panels: Vec::new(),
            controls: Vec::new(),
        });
    }

    fn push_panel(&mut self, panel: NativeGlassPanelDescriptor) {
        if let Some(collection) = self.stack.last_mut() {
            collection.panels.push(panel);
        }
    }

    #[allow(dead_code)]
    fn push_control(&mut self, control: NativeGlassControlDescriptor) {
        if let Some(collection) = self.stack.last_mut() {
            collection.controls.push(control);
        }
    }

    fn finish(&mut self, rect: Rect) -> Option<NativeGlassCollection> {
        let mut collection = self.stack.pop()?;
        collection.rect = rect;
        Some(collection)
    }
}

pub(crate) fn push_native_glass_control_descriptor(
    cx: &mut Cx2d,
    descriptor: NativeGlassControlDescriptor,
) {
    cx.global::<NativeGlassCollector>().push_control(descriptor);
}

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.View

    mod.widgets.GlassContainerBase = #(GlassContainer::register_widget(vm))
    mod.widgets.GlassPanelBase = #(GlassPanel::register_widget(vm))

    mod.widgets.GlassContainer = set_type_default() do mod.widgets.GlassContainerBase{
        width: Fill
        height: Fit
        spacing: 20.0
    }

    mod.widgets.GlassSeparator = mod.widgets.View{
        width: Fill
        height: 1
        show_bg: true
        draw_bg +: {
            color: instance(#xEAD8B838)
        }
    }

    mod.widgets.GlassVSeparator = mod.widgets.View{
        width: 1
        height: Fill
        show_bg: true
        draw_bg +: {
            color: instance(#xEAD8B838)
        }
    }

    mod.widgets.GlassScrollEdge = mod.widgets.View{
        width: Fill
        height: 28
        show_bg: true
        draw_bg +: {
            color: instance(#xEAD8B83A)
            pixel: fn() {
                let fade = pow(clamp(1.0 - self.pos.y, 0.0, 1.0), 1.7)
                return Pal.premul(vec4(self.color.rgb, self.color.a * fade))
            }
        }
    }

    mod.widgets.GlassScrollEdgeBottom = mod.widgets.GlassScrollEdge{
        draw_bg +: {
            pixel: fn() {
                let fade = pow(clamp(self.pos.y, 0.0, 1.0), 1.7)
                return Pal.premul(vec4(self.color.rgb, self.color.a * fade))
            }
        }
    }

    mod.widgets.GlassPanel = set_type_default() do mod.widgets.GlassPanelBase{
        show_bg: true
        draw_bg +: {
            tint_color: instance(#fff)
            tint_alpha: instance(0.2)

            border_color: instance(#fff)
            border_alpha: instance(0.35)
            border_width: instance(1.0)
            corner_radius: instance(12.0)

            // v2 liquid-glass parameters.
            // halo_strength=0 disables the external cyan rim halo.
            // halo_radius is also used to inset the SDF box, so the halo has
            // room to render outside the glass without being clipped.
            halo_color: instance(#x72E4FF)
            halo_strength: instance(0.0)
            halo_radius: instance(0.0)
            // Top highlight band — thin bright row near the glass top edge.
            highlight_strength: instance(0.0)
            highlight_band_height: instance(3.0)
            // Reserved for v2 follow-up (chromatic edge dispersion).
            chroma_strength: instance(0.0)
            noise_strength: instance(0.035)

            // Deprecated v1 instance params — kept as no-ops for one
            // transition commit so existing call sites don't fail to parse.
            // Removed after all aichat sites migrate to v2 names.
            specular_strength: instance(0.0)
            use_scene_blur: instance(0.0)
            blur_amount: instance(0.0)

            // v5.1 shader-backdrop proof parameters. These are disabled by
            // default and only enabled by the dedicated aichat proof target.
            backdrop_sample_strength: instance(0.0)
            backdrop_mix: instance(0.0)
            backdrop_grid_strength: instance(0.0)
            backdrop_blur_radius: instance(0.0)
            backdrop_blur_mix: instance(0.0)
            backdrop_texture_strength: instance(0.0)
            backdrop_texture_screen_space: uniform(0.0)
            backdrop_texture_size: uniform(vec2(900.0, 700.0))
            backdrop_texture_uv_offset: uniform(vec2(0.0, 0.0))
            backdrop_texture_uv_scale: uniform(vec2(1.0, 1.0))
            backdrop_refraction_strength: instance(0.0)
            backdrop_rim_strength: instance(0.0)
            backdrop_liquid_warp_strength: instance(0.0)
            backdrop_texture: texture_2d(float)

            backdrop_signal_rgb: fn(uv: vec2) -> vec3 {
                let t = self.draw_pass.time
                let r = 0.5 + 0.5 * sin(uv.x * 22.0 + t * 1.7)
                let g = 0.5 + 0.5 * sin(uv.y * 18.0 - t * 1.3 + 2.1)
                let b = 0.5 + 0.5 * sin((uv.x + uv.y) * 15.0 + t * 1.1 + 4.2)
                let grid_x = 0.5 + 0.5 * sin(uv.x * 72.0)
                let grid_y = 0.5 + 0.5 * sin(uv.y * 72.0)
                let grid = max(pow(grid_x, 20.0), pow(grid_y, 20.0)) * self.backdrop_grid_strength
                return vec3(r, g, b).mix(vec3(1.0, 0.96, 0.78), clamp(grid, 0.0, 1.0))
            }

            backdrop_signal_blurred_rgb: fn(uv: vec2) -> vec3 {
                let px = self.backdrop_blur_radius / max(min(self.rect_size.x, self.rect_size.y), 1.0)
                let a = vec2(px, 0.0)
                let b = vec2(0.0, px)
                let c = vec2(px * 0.707, px * 0.707)
                let center = self.backdrop_signal_rgb(uv) * 0.22
                let axial = (
                    self.backdrop_signal_rgb(uv + a)
                    + self.backdrop_signal_rgb(uv - a)
                    + self.backdrop_signal_rgb(uv + b)
                    + self.backdrop_signal_rgb(uv - b)
                ) * 0.13
                let diagonal = (
                    self.backdrop_signal_rgb(uv + c)
                    + self.backdrop_signal_rgb(uv - c)
                    + self.backdrop_signal_rgb(uv + vec2(c.x, -c.y))
                    + self.backdrop_signal_rgb(uv + vec2(-c.x, c.y))
                ) * 0.065
                return center + axial + diagonal
            }

            backdrop_edge_weight: fn(uv: vec2) -> float {
                let px = uv * self.rect_size
                let edge = min(min(px.x, self.rect_size.x - px.x), min(px.y, self.rect_size.y - px.y))
                return clamp(1.0 - edge / 34.0, 0.0, 1.0)
            }

            backdrop_refracted_uv: fn(uv: vec2) -> vec2 {
                let centered = uv - vec2(0.5, 0.5)
                let edge = self.backdrop_edge_weight(uv)
                let pull = centered * edge * self.backdrop_refraction_strength * 0.090
                let t = self.draw_pass.time
                let wave_a = sin((uv.y + centered.x * 0.42) * 18.0 + t * 0.95)
                let wave_b = cos((uv.x - centered.y * 0.36) * 15.0 - t * 0.72)
                let swirl = vec2(
                    centered.y * wave_a - centered.x * wave_b,
                    centered.x * wave_b + centered.y * wave_a
                )
                let liquid_weight = clamp(0.28 + edge * 0.72, 0.0, 1.0)
                let liquid = swirl * liquid_weight * self.backdrop_liquid_warp_strength * 0.030
                return clamp(uv + pull + liquid, vec2(0.0, 0.0), vec2(1.0, 1.0))
            }

            backdrop_texture_uv: fn(uv: vec2) -> vec2 {
                let local_uv = self.backdrop_refracted_uv(uv)
                let screen_uv = self.backdrop_texture_uv_offset + local_uv * self.backdrop_texture_uv_scale
                return local_uv.mix(screen_uv, clamp(self.backdrop_texture_screen_space, 0.0, 1.0))
            }

            backdrop_texture_rgb: fn(uv: vec2) -> vec3 {
                let chroma = clamp(
                    self.chroma_strength * self.backdrop_edge_weight(uv),
                    0.0,
                    1.0
                )
                let centered = uv - vec2(0.5, 0.5)
                let offset = centered * chroma * 0.040
                let r = self.backdrop_texture.sample(
                    self.backdrop_texture_uv(clamp(uv + offset, vec2(0.0, 0.0), vec2(1.0, 1.0)))
                ).r
                let g = self.backdrop_texture.sample(self.backdrop_texture_uv(uv)).g
                let b = self.backdrop_texture.sample(
                    self.backdrop_texture_uv(clamp(uv - offset, vec2(0.0, 0.0), vec2(1.0, 1.0)))
                ).b
                return vec3(r, g, b)
            }

            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                let inset = self.border_width * 0.5 + self.halo_radius
                let w = self.rect_size.x - inset * 2.0
                let h = self.rect_size.y - inset * 2.0
                let safe_radius = min(self.corner_radius, max(0.0, min(w, h) * 0.5 - 1.0))

                if safe_radius >= min(w, h) * 0.5 - 1.5 && w > h {
                    let cy = inset + h * 0.5
                    let r = h * 0.5 - 1.0
                    sdf.circle(inset + r + 1.0, cy, r)
                    sdf.circle(inset + w - r - 1.0, cy, r)
                    sdf.rect(inset + r + 1.0, inset + 1.0, w - (r + 1.0) * 2.0, h - 2.0)
                    sdf.union()
                    sdf.union()
                } else {
                    sdf.box(inset, inset, w, h, safe_radius)
                }

                let y_px_in_glass = self.pos.y * self.rect_size.y - inset
                let band = clamp(
                    1.0 - y_px_in_glass / self.highlight_band_height,
                    0.0
                    1.0
                )
                let highlight = vec3(1.0, 0.97, 0.86) * band * self.highlight_strength

                let noise = (
                    Math.random_2d(
                        self.pos * self.rect_size
                        + vec2(self.draw_pass.time * 37.0, self.draw_pass.time * 13.0)
                    ) - 0.5
                ) * self.noise_strength

                let raw_backdrop_rgb = self.backdrop_signal_rgb(self.pos)
                let blurred_backdrop_rgb = self.backdrop_signal_blurred_rgb(self.pos)
                let blur_mix = clamp(self.backdrop_blur_mix, 0.0, 1.0)
                let procedural_backdrop_rgb = raw_backdrop_rgb.mix(blurred_backdrop_rgb, blur_mix)
                let texture_backdrop_rgb = self.backdrop_texture_rgb(self.pos)
                let texture_mix = clamp(self.backdrop_texture_strength, 0.0, 1.0)
                let backdrop_rgb = procedural_backdrop_rgb.mix(texture_backdrop_rgb, texture_mix)
                let shader_rgb = self.tint_color.rgb + highlight + noise
                let sample_mix = clamp(self.backdrop_sample_strength * self.backdrop_mix, 0.0, 1.0)
                let rim = self.backdrop_edge_weight(self.pos) * self.backdrop_rim_strength
                let fill_rgb = shader_rgb.mix(backdrop_rgb + highlight + noise, sample_mix)
                    + vec3(1.0, 0.94, 0.78) * rim
                let fill = vec4(fill_rgb, self.tint_alpha)
                sdf.fill_keep(fill)

                if self.border_width > 0.0 {
                    sdf.stroke_keep(
                        vec4(self.border_color.rgb, self.border_alpha)
                        self.border_width
                    )
                }

                if self.halo_strength > 0.0 {
                    sdf.glow(
                        vec4(self.halo_color.rgb, self.halo_strength)
                        self.halo_radius
                    )
                }

                return sdf.result
            }
        }
    }

    mod.widgets.GlassToolbar = mod.widgets.GlassPanel{
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

    mod.widgets.GlassButton = mod.widgets.ButtonFlat{
        height: 34
        padding: Inset{left: 14 right: 14 top: 0 bottom: 0}
        draw_text +: {
            color: #xF3E3C7
            color_hover: #xFFF0D2
            color_down: #xEAD8B8
            color_focus: #xFFF0D2
            text_style +: { font_size: 11 }
        }
        draw_bg +: {
            color: #x08251EB8
            color_hover: #x123B31DD
            color_down: #x061D18E8
            color_focus: #x123B31DD
            border_color: #xEAD8B82D
            border_color_hover: #x72E4FF44
            border_color_down: #xEAD8B850
            border_color_focus: #x72E4FF66
            border_size: 1.0
            border_radius: 10.0
        }
    }

    mod.widgets.GlassIconButton = mod.widgets.GlassButton{
        width: 36
        height: 36
        padding: 0
        draw_text +: {
            text_style +: { font_size: 15 }
        }
        draw_bg +: {
            color: #x08251EB0
            color_hover: #x154337DD
            border_color: #xEAD8B82A
        }
    }

    mod.widgets.GlassPrimaryButton = mod.widgets.GlassButton{
        width: 44
        height: 44
        padding: 0
        draw_text +: {
            color: #x14110B
            color_hover: #x14110B
            color_down: #x14110B
            color_focus: #x14110B
            text_style +: { font_size: 26 }
        }
        draw_bg +: {
            color: #xE6BB68
            color_hover: #xFFD98B
            color_down: #xC99748
            color_focus: #xFFD98B
            border_color: #xFFF0D277
            border_color_hover: #xFFF0D2AA
            border_color_down: #xA86F35
            border_color_focus: #xFFF0D2CC
        }
    }

    mod.widgets.GlassNavButton = mod.widgets.GlassButton{
        width: Fill
        height: 30
        align: Align{x: 0.0 y: 0.5}
        padding: Inset{left: 4 right: 4}
        draw_text +: {
            color: #xE4D4B6
            color_hover: #xF3E3C7
            text_style +: { font_size: 12 }
        }
        draw_bg +: {
            color: #x00000000
            color_hover: #xEAD8B814
            color_down: #xEAD8B81E
            color_focus: #xEAD8B814
            border_color: #x00000000
            border_color_hover: #xEAD8B82A
            border_size: 0.0
            border_radius: 8.0
        }
    }

    mod.widgets.GlassUtilityButton = mod.widgets.GlassButton{
        width: Fit
        height: Fit
        padding: Inset{top: 2 bottom: 2 left: 6 right: 6}
        draw_text +: {
            color: #x888888
            color_hover: #xEAD8B8
            text_style +: { font_size: 9 }
        }
        draw_bg +: {
            color: #x00000000
            color_hover: #xEAD8B812
            color_down: #xEAD8B81E
            color_focus: #xEAD8B812
            border_color: #x00000000
            border_color_hover: #xEAD8B82A
            border_size: 0.0
            border_radius: 8.0
        }
    }

    mod.widgets.GlassShell = mod.widgets.GlassPanel{
        width: Fill
        height: Fill
        flow: Right
        padding: Inset{left: 16 top: 16 right: 16 bottom: 16}
        spacing: 0
        draw_bg +: {
            tint_color: #x0D4035
            tint_alpha: 0.66
            border_color: #x77DDF2
            border_alpha: 0.38
            border_width: 1.0
            corner_radius: 30.0
            halo_color: #x77DDF2
            halo_strength: 0.0
            halo_radius: 0.0
            highlight_strength: 0.28
            highlight_band_height: 58.0
            chroma_strength: 0.0
            noise_strength: 0.004
        }
    }

    mod.widgets.GlassMainSurface = mod.widgets.GlassPanel{
        width: Fill
        height: Fill
        flow: Down
        padding: Inset{left: 34 top: 18 right: 34 bottom: 22}
        spacing: 12
        draw_bg +: {
            tint_color: #x0B3B31
            tint_alpha: 0.70
            border_color: #xEAD8B8
            border_alpha: 0.16
            border_width: 0.0
            corner_radius: 0.0
            halo_strength: 0.0
            halo_radius: 0.0
            highlight_strength: 0.16
            highlight_band_height: 56.0
            chroma_strength: 0.0
            noise_strength: 0.004
        }
    }

    mod.widgets.GlassComposer = mod.widgets.GlassPanel{
        flow: Down
        padding: Inset{left: 18 top: 14 right: 14 bottom: 12}
        spacing: 10
        draw_bg +: {
            tint_color: #x082E27
            tint_alpha: 0.82
            border_color: #xEAD8B8
            border_alpha: 0.24
            border_width: 0.8
            corner_radius: 24.0
            halo_color: #xA8F0FF
            halo_strength: 0.045
            halo_radius: 5.0
            highlight_strength: 0.24
            highlight_band_height: 34.0
            chroma_strength: 0.0
            noise_strength: 0.003
        }
    }

    mod.widgets.GlassSidebar = mod.widgets.GlassPanel{
        width: 298
        height: Fill
        flow: Down
        padding: Inset{left: 14 top: 14 right: 14 bottom: 14}
        spacing: 10
        draw_bg +: {
            tint_color: #x0A3A30
            tint_alpha: 0.78
            border_color: #xEAD8B8
            border_alpha: 0.20
            border_width: 0.0
            corner_radius: 0.0
            halo_strength: 0.0
            halo_radius: 0.0
            highlight_strength: 0.16
            highlight_band_height: 54.0
            chroma_strength: 0.0
            noise_strength: 0.004
        }
    }

    mod.widgets.GlassCard = mod.widgets.RoundedView{
        show_bg: true
        draw_bg +: {
            color: #x0B2A22E6
            radius: 12.0
        }
    }
}

#[derive(Clone)]
enum GlassContainerDrawState {
    Begin,
    Drawing,
}

#[derive(Script, ScriptHook, Widget)]
pub struct GlassContainer {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,

    #[live(false)]
    pub native: bool,
    #[live(20.0)]
    pub spacing: f64,

    #[rust]
    last_native_batch: Option<NativeGlassBatch>,
    #[rust]
    last_native_control_batch: Option<NativeGlassControlBatch>,

    #[rust]
    draw_state: DrawStateWrap<GlassContainerDrawState>,
}

impl Widget for GlassContainer {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        if self.draw_state.begin(cx, GlassContainerDrawState::Begin) {
            if self.native {
                cx.global::<NativeGlassCollector>()
                    .begin(LiveId(self.widget_uid().0), self.spacing);
            }
            self.draw_state.set(GlassContainerDrawState::Drawing);
        }

        if let Some(GlassContainerDrawState::Drawing) = self.draw_state.get() {
            self.view.draw_walk(cx, scope, walk)?;

            if self.native {
                let rect = self.view.area().rect(cx);
                if let Some(window_id) = cx.get_current_window_id() {
                    let collection = cx.global::<NativeGlassCollector>().finish(rect);
                    if let Some(collection) = collection {
                        let controls = collection.controls;
                        let batch = NativeGlassBatch {
                            window_id,
                            containers: vec![NativeGlassContainerDescriptor {
                                id: collection.id,
                                rect: collection.rect,
                                spacing: collection.spacing,
                                panels: collection.panels,
                            }],
                        };
                        if self.last_native_batch.as_ref() != Some(&batch) {
                            cx.push_unique_platform_op(CxOsOp::SetNativeGlassBatch(batch.clone()));
                            self.last_native_batch = Some(batch);
                        }
                        let control_batch = NativeGlassControlBatch {
                            window_id,
                            controls,
                        };
                        if !control_batch.controls.is_empty()
                            || self.last_native_control_batch.is_some()
                        {
                            if self.last_native_control_batch.as_ref() != Some(&control_batch) {
                                cx.push_unique_platform_op(CxOsOp::SetNativeGlassControlBatch(
                                    control_batch.clone(),
                                ));
                                self.last_native_control_batch = Some(control_batch);
                            }
                        }
                    }
                } else {
                    cx.global::<NativeGlassCollector>().finish(rect);
                }
            } else {
                self.last_native_batch = None;
                self.last_native_control_batch = None;
            }

            self.draw_state.end();
        }

        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }
}

#[derive(Clone)]
enum GlassPanelDrawState {
    Begin,
    Drawing,
}

#[derive(Script, ScriptHook, Widget)]
pub struct GlassPanel {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,

    #[live(false)]
    pub native: bool,
    #[live(GlassNativeStyle::Regular)]
    pub native_style: GlassNativeStyle,
    #[live(GlassNativeHitTest::Passthrough)]
    pub native_hit_test: GlassNativeHitTest,
    #[live(GlassNativeShape::RoundedRect)]
    pub native_shape: GlassNativeShape,
    #[live(12.0)]
    pub native_radius: f64,
    #[live]
    pub native_tint: Option<Vec4f>,
    #[live(0.0)]
    pub native_z_order: f64,

    #[rust]
    backdrop_texture: Option<Texture>,
    #[rust]
    backdrop_texture_screen_space: f32,
    #[rust]
    backdrop_texture_size: Vec2f,
    #[rust]
    backdrop_texture_uv_offset: Vec2f,
    #[rust]
    backdrop_texture_uv_scale: Vec2f,

    #[rust]
    draw_state: DrawStateWrap<GlassPanelDrawState>,
}

impl GlassPanel {
    pub fn set_backdrop_texture(&mut self, texture: Option<Texture>) {
        self.backdrop_texture = texture;
    }

    pub fn set_backdrop_texture_mapping(&mut self, screen_space: f32, texture_size: Vec2f) {
        self.backdrop_texture_screen_space = screen_space;
        self.backdrop_texture_size = texture_size;
    }

    fn update_backdrop_texture_uv_mapping(&mut self, cx: &Cx) -> bool {
        if self.backdrop_texture_screen_space <= 0.5 {
            self.backdrop_texture_uv_offset = vec2(0.0, 0.0);
            self.backdrop_texture_uv_scale = vec2(1.0, 1.0);
            return false;
        }

        let rect = self.view.area().rect(cx);
        if rect.size.x < 1.0 || rect.size.y < 1.0 {
            return false;
        }
        let safe_size = vec2(
            self.backdrop_texture_size.x.max(1.0),
            self.backdrop_texture_size.y.max(1.0),
        );
        let offset = vec2(
            rect.pos.x as f32 / safe_size.x,
            rect.pos.y as f32 / safe_size.y,
        );
        let scale = vec2(
            rect.size.x as f32 / safe_size.x,
            rect.size.y as f32 / safe_size.y,
        );
        let changed = (offset - self.backdrop_texture_uv_offset).length() > 0.001
            || (scale - self.backdrop_texture_uv_scale).length() > 0.001;
        self.backdrop_texture_uv_offset = offset;
        self.backdrop_texture_uv_scale = scale;
        changed
    }

    fn apply_backdrop_texture_uniforms(&mut self, cx: &Cx) {
        self.view.draw_bg.draw_vars.set_uniform(
            cx,
            live_id!(backdrop_texture_screen_space),
            &[self.backdrop_texture_screen_space],
        );
        self.view.draw_bg.draw_vars.set_uniform(
            cx,
            live_id!(backdrop_texture_size),
            &[self.backdrop_texture_size.x, self.backdrop_texture_size.y],
        );
        self.view.draw_bg.draw_vars.set_uniform(
            cx,
            live_id!(backdrop_texture_uv_offset),
            &[
                self.backdrop_texture_uv_offset.x,
                self.backdrop_texture_uv_offset.y,
            ],
        );
        self.view.draw_bg.draw_vars.set_uniform(
            cx,
            live_id!(backdrop_texture_uv_scale),
            &[
                self.backdrop_texture_uv_scale.x,
                self.backdrop_texture_uv_scale.y,
            ],
        );
    }

    fn native_descriptor(&self, cx: &Cx) -> NativeGlassPanelDescriptor {
        NativeGlassPanelDescriptor {
            id: LiveId(self.widget_uid().0),
            rect: self.view.area().rect(cx),
            shape: self.native_shape.to_native(self.native_radius),
            style: self.native_style.to_native(),
            tint: self.native_tint,
            hit_test: self.native_hit_test.to_native(),
            z_order: self.native_z_order as i32,
            visible: self.view.visible(),
        }
    }
}

impl Widget for GlassPanel {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        if self.draw_state.begin(cx, GlassPanelDrawState::Begin) {
            self.draw_state.set(GlassPanelDrawState::Drawing);
        }

        if let Some(GlassPanelDrawState::Drawing) = self.draw_state.get() {
            if let Some(texture) = &self.backdrop_texture {
                self.view.draw_bg.draw_vars.set_texture(0, texture);
            }
            self.update_backdrop_texture_uv_mapping(cx);
            self.apply_backdrop_texture_uniforms(cx);
            self.view.draw_walk(cx, scope, walk)?;
            if self.update_backdrop_texture_uv_mapping(cx) {
                self.apply_backdrop_texture_uniforms(cx);
                self.view.redraw(cx);
            }

            if self.native {
                let descriptor = self.native_descriptor(cx);
                cx.global::<NativeGlassCollector>().push_panel(descriptor);
            }

            self.draw_state.end();
        }

        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }
}

#[cfg(test)]
mod native_glass_tests {
    use super::*;

    #[test]
    fn native_glass_collector_returns_container_with_pushed_panels() {
        let mut collector = NativeGlassCollector::default();
        collector.begin(LiveId(1), 20.0);
        collector.push_panel(NativeGlassPanelDescriptor {
            id: LiveId(2),
            rect: Rect::default(),
            shape: NativeGlassShape::RoundedRect { radius: 12.0 },
            style: NativeGlassStyle::Regular,
            tint: None,
            hit_test: NativeGlassHitTest::Passthrough,
            z_order: 0,
            visible: true,
        });

        let collection = collector.finish(Rect {
            pos: dvec2(10.0, 20.0),
            size: dvec2(300.0, 200.0),
        });

        let collection = collection.expect("collection should finish");
        assert_eq!(collection.id, LiveId(1));
        assert_eq!(collection.spacing, 20.0);
        assert_eq!(collection.panels.len(), 1);
        assert_eq!(collector.stack.len(), 0);
    }

    #[test]
    fn native_glass_collector_returns_container_with_pushed_controls() {
        let mut collector = NativeGlassCollector::default();
        collector.begin(LiveId(1), 20.0);
        collector.push_control(NativeGlassControlDescriptor {
            id: LiveId(3),
            rect: Rect {
                pos: dvec2(12.0, 16.0),
                size: dvec2(80.0, 32.0),
            },
            kind: NativeGlassControlKind::Button {
                role: NativeGlassButtonRole::Primary,
            },
            label: "Send".to_string(),
            style: NativeGlassStyle::Clear,
            tint: None,
            z_order: 1,
            enabled: true,
            visible: true,
        });

        let collection = collector.finish(Rect {
            pos: dvec2(10.0, 20.0),
            size: dvec2(300.0, 200.0),
        });

        let collection = collection.expect("collection should finish");
        assert_eq!(collection.controls.len(), 1);
        assert_eq!(collection.controls[0].id, LiveId(3));
        assert_eq!(
            collection.controls[0].kind,
            NativeGlassControlKind::Button {
                role: NativeGlassButtonRole::Primary,
            }
        );
        assert_eq!(collection.controls[0].label, "Send");
        assert_eq!(collector.stack.len(), 0);
    }

    #[test]
    fn native_glass_collector_finish_without_begin_returns_none() {
        let mut collector = NativeGlassCollector::default();

        let collection = collector.finish(Rect::default());

        assert!(collection.is_none());
        assert_eq!(collector.stack.len(), 0);
    }

    #[test]
    fn glass_native_shape_maps_capsule() {
        assert_eq!(
            GlassNativeShape::Capsule.to_native(12.0),
            NativeGlassShape::Capsule
        );
    }
}
