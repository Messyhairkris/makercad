//! Icons drawn by the Crew from doc 18, rendered at runtime from their SVG.
//!
//! Each icon uses only doc 18's palette roles, so a look recolours it by exact match: on the
//! Night Workshop look the ink turns light and the paper turns dark. Icons are rasterised once
//! per (id, look, size) and kept as textures. An id with no SVG (or one that fails to render)
//! returns `None`, and the caller draws its painted stand-in, so the app never depends on an
//! icon being present.

use std::collections::HashMap;

use egui::{ColorImage, Context, TextureHandle, TextureOptions};

use crate::icon_data::SVGS;
use crate::theme::Look;

/// The palette roles a dark look swaps: (light, dark).
const NIGHT: &[(&str, &str)] = &[
    ("#2B2A4C", "#F2EEE6"), // ink
    ("#FFFDF7", "#2A2838"), // paper
    ("#CFE3FA", "#3E4A6B"), // tint
];

/// Rendered icons.
#[derive(Default)]
pub struct Icons {
    cache: HashMap<(String, Look, u32), Option<TextureHandle>>,
}

impl Icons {
    /// Whether an SVG exists for `id`.
    pub fn has(id: &str) -> bool {
        SVGS.iter().any(|(i, _)| *i == id)
    }

    /// The icon `id` for `look` at `px` physical pixels, rendering it the first time.
    pub fn get(&mut self, ctx: &Context, id: &str, look: Look, px: u32) -> Option<TextureHandle> {
        let key = (id.to_string(), look, px);
        if let Some(t) = self.cache.get(&key) {
            return t.clone();
        }
        let texture = SVGS
            .iter()
            .find(|(i, _)| *i == id)
            .and_then(|(_, svg)| render(&recolour(svg, look), px))
            .map(|image| {
                ctx.load_texture(
                    format!("icon:{id}:{}:{px}", look.key()),
                    image,
                    TextureOptions::LINEAR,
                )
            });
        self.cache.insert(key, texture.clone());
        texture
    }
}

/// `svg` with `look`'s colours for the palette roles.
pub fn recolour(svg: &str, look: Look) -> String {
    if !look.palette().dark {
        return svg.to_string();
    }
    let mut out = svg.to_string();
    // Swap through placeholders so a colour already swapped is not swapped back.
    for (i, (light, _)) in NIGHT.iter().enumerate() {
        out = out
            .replace(light, &format!("@{i}@"))
            .replace(&light.to_ascii_lowercase(), &format!("@{i}@"));
    }
    for (i, (_, dark)) in NIGHT.iter().enumerate() {
        out = out.replace(&format!("@{i}@"), dark);
    }
    out
}

/// Rasterises a 64 x 64 viewBox SVG to `px` x `px`.
pub fn render(svg: &str, px: u32) -> Option<ColorImage> {
    let options = resvg::usvg::Options::default();
    let tree = resvg::usvg::Tree::from_data(svg.as_bytes(), &options).ok()?;
    let size = tree.size();
    let mut pixmap = resvg::tiny_skia::Pixmap::new(px.max(1), px.max(1))?;
    let transform = resvg::tiny_skia::Transform::from_scale(
        px as f32 / size.width().max(1.0),
        px as f32 / size.height().max(1.0),
    );
    resvg::render(&tree, transform, &mut pixmap.as_mut());
    let side = usize::try_from(px.max(1)).ok()?;
    Some(ColorImage::from_rgba_premultiplied(
        [side, side],
        pixmap.data(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64"><rect x="8" y="8" width="48" height="48" fill="#FFFDF7" stroke="#2B2A4C" stroke-width="4"/></svg>"##;

    #[test]
    fn a_light_look_keeps_the_colours() {
        assert_eq!(recolour(SVG, Look::Daydream), SVG);
    }

    #[test]
    fn the_night_look_swaps_ink_and_paper_without_swapping_back() {
        let night = recolour(SVG, Look::NightWorkshop);
        assert!(night.contains(r##"fill="#2A2838""##), "{night}");
        assert!(night.contains(r##"stroke="#F2EEE6""##), "{night}");
    }

    #[test]
    fn an_svg_renders_to_the_asked_size() {
        let image = render(SVG, 32).expect("renders");
        assert_eq!(image.size, [32, 32]);
        // The middle is paper, the edge of the square is ink.
        let at = |x: usize, y: usize| image.pixels[y * 32 + x];
        assert!(at(16, 16).r() > 200, "{:?}", at(16, 16));
        assert!(at(4, 16).r() < 100, "{:?}", at(4, 16));
    }

    #[test]
    fn junk_does_not_render_and_does_not_crash() {
        assert!(render("<svg", 16).is_none());
        assert!(render("", 16).is_none());
    }
}
