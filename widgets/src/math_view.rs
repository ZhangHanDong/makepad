use crate::*;
use makepad_draw::{
    shader::draw_glyph::GlyphShapeId,
    text::{font::Font, font_family::FontFamilyId, glyph_outline::Command as OutlineCommand},
};
use makepad_latex_math::{self as latex_math, LayoutGlyph, LayoutItem, LayoutRule, MathStyle};

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.draw
    use mod.text.*

    mod.widgets.MathViewBase = #(MathView::register_widget(vm))

    mod.widgets.MathView = set_type_default() do mod.widgets.MathViewBase{
        width: Fit
        height: Fit
        color: #fff
        font_size: 11.0
        baseline_offset: -2.0
        draw_glyph +: {
            aa_pad_px: 1.0
        }
        draw_text +: {
            text_style: TextStyle{
                font_family: FontFamily{
                    latin := FontMember{res: crate_resource("self:resources/NewCMMath-Regular.otf") asc: 0.0 desc: 0.0}
                }
                font_size: 11.0
                line_spacing: 1.2
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MathComponent {
    pub shape_id: GlyphShapeId,
    pub origin: Vec2f,
    pub size: Vec2f,
}

#[derive(Clone, Debug)]
pub struct CompiledMath {
    pub components: Vec<MathComponent>,
    pub width: f32,
    pub height: f32,
    pub ascent: f32,
    pub descent: f32,
}

#[derive(Script, ScriptHook, Widget)]
pub struct MathView {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[walk]
    walk: Walk,
    #[redraw]
    #[live]
    draw_glyph: DrawGlyph,
    #[live]
    draw_text: DrawText,
    #[live]
    text: String,
    #[live]
    color: Vec4,
    #[live(11.0)]
    font_size: f64,
    #[live(-2.0)]
    baseline_offset: f64,
    #[rust]
    old_text: String,
    #[rust]
    old_font_size: f64,
    #[rust]
    old_font_family_id: Option<FontFamilyId>,
    #[rust]
    layout_cache: Option<latex_math::LayoutOutput>,
    #[rust]
    components: Vec<MathComponent>,
    #[rust]
    debug_reason: Option<String>,
}

impl Widget for MathView {
    fn handle_event(&mut self, _cx: &mut Cx, _event: &Event, _scope: &mut Scope) {}

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, mut walk: Walk) -> DrawStep {
        self.compile_math(cx);

        let Some(layout) = self.layout_cache.as_ref() else {
            if let Some(reason) = &self.debug_reason {
                let debug_text = format!("[{}]", reason);
                self.draw_text
                    .draw_walk(cx, walk, Align::default(), debug_text.as_str());
            }
            return DrawStep::done();
        };

        let w = layout.width;
        let h = layout.height;
        walk.width = Size::Fixed(w as f64);
        walk.height = Size::Fixed(h as f64);
        walk.margin.top += self.baseline_offset;

        let bounds = cx.walk_turtle(walk);
        let color = self.color;

        for component in &self.components {
            let mut layers = {
                let Some(shape) = self.draw_glyph.shape(component.shape_id) else {
                    continue;
                };
                shape.layers.clone()
            };
            for layer in &mut layers {
                layer.color = vec4(color.x, color.y, color.z, color.w * layer.color.w);
            }
            self.draw_glyph.draw_layers_abs(
                cx,
                rect(
                    bounds.pos.x + component.origin.x as f64,
                    bounds.pos.y + component.origin.y as f64,
                    component.size.x as f64,
                    component.size.y as f64,
                ),
                &layers,
            );
        }

        DrawStep::done()
    }

    fn set_text(&mut self, cx: &mut Cx, text: &str) {
        self.text = text.to_string();
        self.redraw(cx);
    }
}

impl MathView {
    fn compile_math(&mut self, cx: &mut Cx2d) {
        let font_family_id = self.draw_text.text_style.font_family_id();
        if self.text == self.old_text
            && self.font_size == self.old_font_size
            && self.old_font_family_id == Some(font_family_id)
            && (self.layout_cache.is_some() || self.text.is_empty())
        {
            return;
        }

        self.old_text = self.text.clone();
        self.old_font_size = self.font_size;
        self.old_font_family_id = Some(font_family_id);

        if self.text.is_empty() {
            self.layout_cache = None;
            self.components.clear();
            self.draw_glyph.clear_shapes();
            self.debug_reason = None;
            return;
        }

        self.draw_glyph.clear_shapes();
        match compile_math_into(
            cx,
            &mut self.draw_glyph,
            &mut self.draw_text,
            self.font_size,
            &self.text,
        ) {
            Some(compiled) => {
                self.layout_cache = Some(latex_math::LayoutOutput {
                    items: Vec::new(),
                    width: compiled.width,
                    height: compiled.height,
                    ascent: compiled.ascent,
                    descent: compiled.descent,
                });
                self.components = compiled.components;
                self.debug_reason = None;
            }
            None => {
                self.layout_cache = None;
                self.components.clear();
                self.draw_glyph.clear_shapes();
                self.debug_reason = Some("math-layout-failed".to_string());
            }
        }
    }
}

pub fn compile_math_into(
    cx: &mut Cx2d,
    dg: &mut DrawGlyph,
    draw_text: &mut DrawText,
    font_size: f64,
    text: &str,
) -> Option<CompiledMath> {
    draw_text.text_style.ensure_fonts_loaded(cx.cx.cx);
    let font_family_id = draw_text.text_style.font_family_id();
    let layout_font = {
        let mut fonts = cx.fonts.borrow_mut();
        let family = fonts.get_or_load_font_family(font_family_id);
        family.fonts().first().cloned()
    }?;

    let nodes = latex_math::parse(text);
    let layout_size = font_size as f32 * 1.75;
    let layout = latex_math::layout(
        &nodes,
        layout_font.data().as_slice(),
        layout_size,
        MathStyle::Display,
    )?;

    let mut components = Vec::new();
    for item in &layout.items {
        match item {
            LayoutItem::Glyph(glyph) => {
                if let Some(shape_id) =
                    build_glyph_shape(dg, layout_font.as_ref(), glyph, layout.ascent)
                {
                    push_component(dg, shape_id, &mut components);
                }
            }
            LayoutItem::Rule(rule) => {
                if let Some(shape_id) = build_rule_shape(dg, rule, layout.ascent) {
                    push_component(dg, shape_id, &mut components);
                }
            }
            LayoutItem::Rect(rect) => {
                if let Some(shape_id) =
                    build_rect_shape(dg, rect.x, layout.ascent + rect.y, rect.width, rect.height)
                {
                    push_component(dg, shape_id, &mut components);
                }
            }
        }
    }

    Some(CompiledMath {
        components,
        width: layout.width,
        height: layout.height,
        ascent: layout.ascent,
        descent: layout.descent,
    })
}

fn build_glyph_shape(
    dg: &mut DrawGlyph,
    font: &Font,
    glyph: &LayoutGlyph,
    layout_ascent: f32,
) -> Option<GlyphShapeId> {
    let glyph_x = glyph.x;
    let glyph_y = layout_ascent + glyph.y;
    let font_scale = glyph.size / font.units_per_em();

    let Some(outline) = font.glyph_outline(glyph.glyph_id) else {
        return None;
    };

    dg.begin_shape();
    dg.set_color(1.0, 1.0, 1.0, 1.0);
    for command in outline.commands() {
        match command {
            OutlineCommand::MoveTo(p) => {
                dg.move_to(glyph_x + p.x * font_scale, glyph_y - p.y * font_scale);
            }
            OutlineCommand::LineTo(p) => {
                dg.line_to(glyph_x + p.x * font_scale, glyph_y - p.y * font_scale);
            }
            OutlineCommand::QuadTo(c, p) => {
                dg.quad_to(
                    glyph_x + c.x * font_scale,
                    glyph_y - c.y * font_scale,
                    glyph_x + p.x * font_scale,
                    glyph_y - p.y * font_scale,
                );
            }
            OutlineCommand::CurveTo(c1, c2, p) => {
                dg.bezier_to(
                    glyph_x + c1.x * font_scale,
                    glyph_y - c1.y * font_scale,
                    glyph_x + c2.x * font_scale,
                    glyph_y - c2.y * font_scale,
                    glyph_x + p.x * font_scale,
                    glyph_y - p.y * font_scale,
                );
            }
            OutlineCommand::Close => {
                dg.close();
            }
        }
    }
    dg.fill_layer();
    dg.commit_shape(None)
}

fn build_rule_shape(
    dg: &mut DrawGlyph,
    rule: &LayoutRule,
    layout_ascent: f32,
) -> Option<GlyphShapeId> {
    build_rect_shape(dg, rule.x, layout_ascent + rule.y, rule.width, rule.height)
}

fn build_rect_shape(dg: &mut DrawGlyph, x: f32, y: f32, w: f32, h: f32) -> Option<GlyphShapeId> {
    dg.begin_shape();
    dg.set_color(1.0, 1.0, 1.0, 1.0);
    dg.rect(x, y, w, h);
    dg.fill_layer();
    dg.commit_shape(None)
}

fn push_component(dg: &DrawGlyph, shape_id: GlyphShapeId, components: &mut Vec<MathComponent>) {
    let Some(shape) = dg.shape(shape_id) else {
        return;
    };
    components.push(MathComponent {
        shape_id,
        origin: shape.origin,
        size: shape.size,
    });
}
