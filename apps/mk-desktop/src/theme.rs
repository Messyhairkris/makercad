//! The three looks from the design canvas (`docs/design/ui-v1/Main.dc.html`) and the shared
//! accent colours of doc 18 ("Storybook Workshop").

use egui::{Color32, CornerRadius, FontId, Stroke, TextStyle, Theme, Visuals};

/// A look the person can pick in Settings.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Look {
    /// Warm paper fading to sky: the default.
    Daydream,
    /// Soft lilac, white panels.
    Pastel,
    /// Dark, for evenings in the shop.
    NightWorkshop,
}

impl Look {
    /// Every look, in the order Settings lists them.
    pub const ALL: [Look; 3] = [Look::Daydream, Look::Pastel, Look::NightWorkshop];

    /// The name a person sees.
    pub fn name(self) -> &'static str {
        match self {
            Look::Daydream => "Daydream",
            Look::Pastel => "Pastel",
            Look::NightWorkshop => "Night Workshop",
        }
    }

    /// The key stored in the settings file. Never change one: old settings files keep it (C3).
    pub fn key(self) -> &'static str {
        match self {
            Look::Daydream => "daydream",
            Look::Pastel => "pastel",
            Look::NightWorkshop => "night-workshop",
        }
    }

    /// The look stored under `key`, if it is one.
    pub fn from_key(key: &str) -> Option<Look> {
        Look::ALL.into_iter().find(|l| l.key() == key)
    }

    /// The colours of this look.
    pub fn palette(self) -> Palette {
        match self {
            Look::Daydream => Palette {
                bg_top: hex(0xFFF8EC),
                bg_bottom: hex(0xE6F2FF),
                bg2: hex(0xFFF8EC),
                panel: hex(0xFFFDF7),
                ink: hex(0x2B2A4C),
                muted: hex(0x5B5775),
                line: hex(0xE4DCCB),
                shadow: hex(0xEADFCB),
                chip: hex(0xE6F2FF),
                dark: false,
            },
            Look::Pastel => Palette {
                bg_top: hex(0xF7F4FB),
                bg_bottom: hex(0xF7F4FB),
                bg2: hex(0xF7F4FB),
                panel: hex(0xFFFFFF),
                ink: hex(0x3A3550),
                muted: hex(0x625C78),
                line: hex(0xE7E0F5),
                shadow: hex(0xECE6F7),
                chip: hex(0xEFE9FB),
                dark: false,
            },
            Look::NightWorkshop => Palette {
                bg_top: hex(0x1F1D2B),
                bg_bottom: hex(0x1F1D2B),
                bg2: hex(0x2A2838),
                panel: hex(0x2A2838),
                ink: hex(0xF2EEE6),
                muted: hex(0xC9C3D6),
                line: hex(0x3B384D),
                shadow: hex(0x16151F),
                chip: hex(0x3B384D),
                dark: true,
            },
        }
    }
}

/// The colours one look paints with.
#[derive(Clone, Copy, Debug)]
pub struct Palette {
    /// Background at the top of the window.
    pub bg_top: Color32,
    /// Background at the bottom of the window.
    pub bg_bottom: Color32,
    /// Plain background behind buttons.
    pub bg2: Color32,
    /// Cards and panels.
    pub panel: Color32,
    /// Text and outlines.
    pub ink: Color32,
    /// Secondary text.
    pub muted: Color32,
    /// Panel borders.
    pub line: Color32,
    /// The flat shadow under cards.
    pub shadow: Color32,
    /// Neutral chips.
    pub chip: Color32,
    /// Whether this is a dark look.
    pub dark: bool,
}

/// Coral accent (doc 18): one per icon, the primary action.
pub const ACCENT: Color32 = hex(0xFF8A7A);
/// Wood, Sancho's colour.
pub const WOOD: Color32 = hex(0xE8C9A8);
/// Sancho's outline, the same in every look.
pub const SANCHO_INK: Color32 = hex(0x2B2A4C);
/// Paper, Sancho's speech bubble.
pub const PAPER: Color32 = hex(0xFFFDF7);
/// A finished stage dot.
pub const STAGE_DONE: Color32 = hex(0x7BC47F);
/// The stage in progress.
pub const STAGE_NOW: Color32 = hex(0xF7C548);
/// Chip: work waiting.
pub const CHIP_WAITING: Color32 = hex(0xFAD3B4);
/// Chip: finished.
pub const CHIP_DONE: Color32 = hex(0xC3E6C4);
/// Link colour on light looks.
pub const LINK: Color32 = hex(0x2B5FA8);

/// A colour from `0xRRGGBB`.
pub const fn hex(rgb: u32) -> Color32 {
    Color32::from_rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
}

/// Sets egui's colours, corners and text sizes for `look`.
pub fn apply(ctx: &egui::Context, look: Look) {
    let p = look.palette();
    let mut v = if p.dark {
        Visuals::dark()
    } else {
        Visuals::light()
    };
    v.override_text_color = Some(p.ink);
    v.panel_fill = p.bg2;
    v.window_fill = p.panel;
    v.window_stroke = Stroke::new(2.0, p.line);
    v.window_corner_radius = CornerRadius::same(20);
    v.extreme_bg_color = p.panel;
    v.hyperlink_color = if p.dark { hex(0x9CC3FF) } else { LINK };
    v.selection.bg_fill = if p.dark { hex(0x3E4A6B) } else { hex(0xCFE3FA) };
    v.selection.stroke = Stroke::new(1.0, p.ink);
    for w in [
        &mut v.widgets.noninteractive,
        &mut v.widgets.inactive,
        &mut v.widgets.hovered,
        &mut v.widgets.active,
        &mut v.widgets.open,
    ] {
        w.corner_radius = CornerRadius::same(14);
        w.fg_stroke = Stroke::new(1.5, p.ink);
    }
    v.widgets.noninteractive.bg_stroke = Stroke::new(2.0, p.line);
    v.widgets.noninteractive.bg_fill = p.panel;
    v.widgets.inactive.bg_fill = p.bg2;
    v.widgets.inactive.weak_bg_fill = p.bg2;
    v.widgets.inactive.bg_stroke = Stroke::new(2.0, p.line);
    v.widgets.hovered.bg_fill = p.chip;
    v.widgets.hovered.weak_bg_fill = p.chip;
    v.widgets.hovered.bg_stroke = Stroke::new(2.0, ACCENT);
    v.widgets.active.bg_fill = p.chip;
    v.widgets.active.weak_bg_fill = p.chip;
    v.widgets.active.bg_stroke = Stroke::new(2.0, ACCENT);
    ctx.set_theme(if p.dark { Theme::Dark } else { Theme::Light });
    ctx.set_visuals(v);
    ctx.all_styles_mut(|s| {
        s.text_styles
            .insert(TextStyle::Heading, FontId::proportional(26.0));
        s.text_styles
            .insert(TextStyle::Body, FontId::proportional(16.0));
        s.text_styles
            .insert(TextStyle::Button, FontId::proportional(15.0));
        s.text_styles
            .insert(TextStyle::Small, FontId::proportional(13.0));
        s.spacing.button_padding = egui::vec2(16.0, 9.0);
        s.spacing.item_spacing = egui::vec2(10.0, 8.0);
        s.spacing.interact_size.y = 44.0;
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_look_round_trips_through_its_key() {
        for look in Look::ALL {
            assert_eq!(Look::from_key(look.key()), Some(look));
        }
        assert_eq!(Look::from_key("sepia"), None);
    }

    #[test]
    fn hex_reads_rgb() {
        assert_eq!(hex(0xFF8A7A), Color32::from_rgb(0xFF, 0x8A, 0x7A));
    }
}
