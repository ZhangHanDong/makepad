use crate::makepad_draw::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.View

    mod.widgets.GlassPanel = View{
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

                let fill_rgb = self.tint_color.rgb + highlight + noise
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
}
