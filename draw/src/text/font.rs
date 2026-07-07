use {
    super::{
        font_face::FontFace,
        geom::{Point, Rect},
        glyph_outline,
        glyph_outline::GlyphOutline,
        glyph_raster_image::GlyphRasterImage,
        intern::Intern,
        loader::FontData,
        rasterizer::{RasterizedGlyph, Rasterizer},
    },
    fxhash::FxHashMap,
    rustybuzz,
    rustybuzz::ttf_parser,
    std::{
        cell::RefCell,
        hash::{Hash, Hasher},
        rc::Rc,
    },
};

/// Cap on the per-font glyph-outline cache (distinct glyphs). Large enough that typical Latin
/// usage never reaches it, but bounds CJK/emoji-heavy sessions from growing without limit.
const MAX_CACHED_GLYPH_OUTLINES: usize = 8192;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct FontId(u64);

impl From<u64> for FontId {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<&str> for FontId {
    fn from(value: &str) -> Self {
        Self(value.intern().as_ptr() as u64)
    }
}

#[derive(Debug)]
pub struct Font {
    id: FontId,
    rasterizer: Rc<RefCell<Rasterizer>>,
    face: FontFace,
    units_per_em: f32,
    ascender_in_ems: f32,
    descender_in_ems: f32,
    line_gap_in_ems: f32,
    cached_glyph_outlines: RefCell<FxHashMap<GlyphId, Option<GlyphOutline>>>,
    /// Apple `hvgl` renderer for iOS/macOS system fonts whose composite-part
    /// outlines `ttf_parser` can't decode. Lazily initialized on first outline:
    /// outer `None` = not yet attempted; inner `None` = font has no `hvgl` table
    /// or `libhvf` is unavailable (so fall back to `ttf_parser`).
    #[cfg(any(target_os = "ios", target_os = "tvos", target_os = "macos"))]
    hvgl_renderer: RefCell<Option<Option<super::hvgl_render::HvglRenderer>>>,
    /// CoreText color-emoji rasterizer for iOS/tvOS `sbix` fonts whose color
    /// bitmaps use Apple's private `emjc` format (which `ttf_parser` can't
    /// decode). Same lazy-init convention as `hvgl_renderer`: outer `None` = not
    /// yet attempted; inner `None` = font has no `sbix` or CoreText is
    /// unavailable (fall back to the outline path).
    #[cfg(any(target_os = "ios", target_os = "tvos"))]
    color_emoji_renderer:
        RefCell<Option<Option<super::color_emoji_render::ColorEmojiRenderer>>>,
}

impl Font {
    pub fn new(
        id: FontId,
        rasterizer: Rc<RefCell<Rasterizer>>,
        face: FontFace,
        ascender_fudge_in_ems: f32,
        descender_fudge_in_ems: f32,
    ) -> Self {
        let (units_per_em, ascender_in_ems, descender_in_ems, line_gap_in_ems) = face
            .with_ttf_parser_face(|face| {
                let units_per_em = face.units_per_em() as f32;
                (
                    units_per_em,
                    face.ascender() as f32 / units_per_em + ascender_fudge_in_ems,
                    face.descender() as f32 / units_per_em + descender_fudge_in_ems,
                    face.line_gap() as f32 / units_per_em,
                )
            });
        Self {
            id,
            rasterizer,
            face,
            units_per_em,
            ascender_in_ems,
            descender_in_ems,
            line_gap_in_ems,
            cached_glyph_outlines: RefCell::new(FxHashMap::default()),
            #[cfg(any(target_os = "ios", target_os = "tvos", target_os = "macos"))]
            hvgl_renderer: RefCell::new(None),
            #[cfg(any(target_os = "ios", target_os = "tvos"))]
            color_emoji_renderer: RefCell::new(None),
        }
    }

    pub fn id(&self) -> FontId {
        self.id
    }

    pub fn data(&self) -> &FontData {
        self.face.data()
    }

    pub(super) fn with_ttf_parser_face<R>(&self, f: impl FnOnce(&ttf_parser::Face<'_>) -> R) -> R {
        self.face.with_ttf_parser_face(f)
    }

    pub(super) fn with_rustybuzz_face<R>(&self, f: impl FnOnce(&rustybuzz::Face<'_>) -> R) -> R {
        self.face.with_rustybuzz_face(f)
    }

    pub fn units_per_em(&self) -> f32 {
        self.units_per_em
    }

    pub fn ascender_in_ems(&self) -> f32 {
        self.ascender_in_ems
    }

    pub fn descender_in_ems(&self) -> f32 {
        self.descender_in_ems
    }

    pub fn line_gap_in_ems(&self) -> f32 {
        self.line_gap_in_ems
    }

    pub fn glyph_outline(&self, glyph_id: GlyphId) -> Option<GlyphOutline> {
        if let Some(outline) = self.cached_glyph_outlines.borrow().get(&glyph_id) {
            return outline.clone();
        }

        let units_per_em = self.units_per_em;

        // Apple `hvgl` system fonts (iOS/macOS): composite-part outlines that
        // `ttf_parser` can't decode — render them via Apple's `libhvf`.
        #[cfg(any(target_os = "ios", target_os = "tvos", target_os = "macos"))]
        if let Some(outline) = self.hvgl_glyph_outline(glyph_id, units_per_em) {
            self.cache_glyph_outline(glyph_id, Some(outline.clone()));
            return Some(outline);
        }

        let outline = self.with_ttf_parser_face(|face| {
            let glyph_id = ttf_parser::GlyphId(glyph_id);
            let mut builder = glyph_outline::Builder::new();
            let bounds = face.outline_glyph(glyph_id, &mut builder)?;
            let min = Point::new(bounds.x_min as f32, bounds.y_min as f32);
            let max = Point::new(bounds.x_max as f32, bounds.y_max as f32);
            Some(builder.finish(Rect::new(min, max - min), units_per_em))
        });

        self.cache_glyph_outline(glyph_id, outline.clone());
        outline
    }

    fn cache_glyph_outline(&self, glyph_id: GlyphId, outline: Option<GlyphOutline>) {
        let mut cache = self.cached_glyph_outlines.borrow_mut();
        // Bound the per-font outline cache. The cap is generous, so this only triggers
        // for scripts with thousands of distinct glyphs (e.g. CJK); clearing simply forces
        // the currently-visible glyphs to be re-extracted from the font face on next use.
        if cache.len() >= MAX_CACHED_GLYPH_OUTLINES {
            cache.clear();
        }
        cache.insert(glyph_id, outline);
    }

    /// Render a glyph via Apple's `libhvf`, if this is an `hvgl` system font and
    /// `libhvf` is available. `None` otherwise (caller falls back to `ttf_parser`).
    #[cfg(any(target_os = "ios", target_os = "tvos", target_os = "macos"))]
    fn hvgl_glyph_outline(&self, glyph_id: GlyphId, units_per_em: f32) -> Option<GlyphOutline> {
        use super::hvgl_render::HvglRenderer;

        // Lazily open the renderer (copies the raw `hvgl` bytes into an aligned
        // buffer, so nothing borrows the face afterwards).
        {
            let mut slot = self.hvgl_renderer.borrow_mut();
            if slot.is_none() {
                let opened = self.with_ttf_parser_face(|face| {
                    face.tables()
                        .hvgl
                        .and_then(|t| HvglRenderer::open(t.data()))
                });
                *slot = Some(opened);
            }
        }

        let slot = self.hvgl_renderer.borrow();
        let renderer = slot.as_ref().unwrap().as_ref()?;

        let mut builder = glyph_outline::Builder::new();
        let bounds = renderer.outline(ttf_parser::GlyphId(glyph_id), &mut builder)?;
        let outline = builder.finish(bounds, units_per_em);
        Some(outline)
    }

    pub fn glyph_outline_bounds_in_ems(
        &self,
        glyph_id: GlyphId,
        out_outline: &mut Option<GlyphOutline>,
    ) -> Option<Rect<f32>> {
        // Check the outline cache first — it stores the full outline,
        // from which we can derive bounds.
        if let Some(cached) = self.cached_glyph_outlines.borrow().get(&glyph_id) {
            *out_outline = cached.clone();
            return cached.as_ref().map(|o| o.bounds_in_ems());
        }

        // Not cached yet — compute via glyph_outline() which will populate the cache.
        if let Some(outline) = self.glyph_outline(glyph_id) {
            let bounds_in_ems = outline.bounds_in_ems();
            *out_outline = Some(outline);
            Some(bounds_in_ems)
        } else {
            None
        }
    }

    pub fn with_glyph_raster_image<R>(
        &self,
        glyph_id: GlyphId,
        dpxs_per_em: f32,
        f: impl FnOnce(GlyphRasterImage<'_>) -> R,
    ) -> Option<R> {
        self.with_ttf_parser_face(|face| {
            let glyph_id = ttf_parser::GlyphId(glyph_id);
            let image = face.glyph_raster_image(glyph_id, dpxs_per_em as u16)?;
            let raster = GlyphRasterImage::from_raster_glyph_image(image)?;
            Some(f(raster))
        })
    }

    pub fn has_glyph_raster_image(&self, glyph_id: GlyphId, dpxs_per_em: f32) -> bool {
        self.with_ttf_parser_face(|face| {
            let glyph_id = ttf_parser::GlyphId(glyph_id);
            face.glyph_raster_image(glyph_id, dpxs_per_em as u16)
                .is_some()
        })
    }

    /// Whether this font carries an `sbix` color-bitmap table. On iOS/tvOS this
    /// gates the CoreText color-emoji path: `sbix` glyphs (Apple's private `emjc`
    /// bitmaps) must stay on the raster path rather than the slug/outline path,
    /// and only `sbix` fonts open a `ColorEmojiRenderer`.
    #[cfg(any(target_os = "ios", target_os = "tvos"))]
    pub fn has_sbix(&self) -> bool {
        self.with_ttf_parser_face(|face| face.tables().sbix.is_some())
    }

    /// Rasterize a color-emoji glyph via CoreText (`CTFontDrawGlyphs`) if this is
    /// an `sbix` font and CoreText is available; run `f` on the resulting raster.
    /// `None` otherwise (caller falls back to the outline path). Lazily opens the
    /// renderer from the reassembled sfnt bytes — see `color_emoji_render`.
    #[cfg(any(target_os = "ios", target_os = "tvos"))]
    pub fn with_color_emoji_raster<R>(
        &self,
        glyph_id: GlyphId,
        dpxs_per_em: f32,
        f: impl FnOnce(&super::color_emoji_render::ColorRaster) -> R,
    ) -> Option<R> {
        use super::color_emoji_render::ColorEmojiRenderer;

        // Lazily open the renderer. Only `sbix` fonts carry color bitmaps, so
        // skip (and remember) anything else — this keeps the CGFont rebuild off
        // the common path for the Latin/CJK/etc. text fonts.
        {
            let mut slot = self.color_emoji_renderer.borrow_mut();
            if slot.is_none() {
                let opened = if self.has_sbix() {
                    ColorEmojiRenderer::open(self.data().as_slice())
                } else {
                    None
                };
                *slot = Some(opened);
            }
        }

        let slot = self.color_emoji_renderer.borrow();
        let renderer = slot.as_ref().unwrap().as_ref()?;
        let raster = renderer.rasterize(glyph_id, dpxs_per_em)?;
        Some(f(&raster))
    }

    pub fn rasterize_glyph(&self, glyph_id: GlyphId, dpxs_per_em: f32) -> Option<RasterizedGlyph> {
        self.rasterizer
            .borrow_mut()
            .rasterize_glyph(self, glyph_id, dpxs_per_em)
    }

    pub fn rasterize_glyph_stable_fallback(
        &self,
        glyph_id: GlyphId,
        dpxs_per_em: f32,
    ) -> Option<RasterizedGlyph> {
        self.rasterizer
            .borrow_mut()
            .rasterize_glyph_stable_fallback(self, glyph_id, dpxs_per_em)
    }
}

impl Eq for Font {}

impl Hash for Font {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for Font {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

pub type GlyphId = u16;
