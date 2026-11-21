//! The opening screen: `docs/design/ui-v1/Main.dc.html`, drawn with egui.

use egui::{
    pos2, vec2, Align, Align2, Color32, CornerRadius, CursorIcon, FontId, Frame, Layout, Margin,
    Mesh, Rect, Response, RichText, Sense, Shadow, Shape, Stroke, StrokeKind, TextureHandle, Ui,
};

use crate::sancho;
use crate::theme::{
    Palette, ACCENT, CHIP_DONE, CHIP_WAITING, PAPER, SANCHO_INK, STAGE_DONE, STAGE_NOW,
};

/// The five stages every project moves through, in order.
pub const STAGES: [&str; 5] = ["Define", "Design", "Plan", "Build", "Publish"];

/// How a Recent row's chip is coloured.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChipKind {
    /// Something is waiting for the maker.
    Waiting,
    /// Finished.
    Done,
    /// Anything else.
    Neutral,
}

/// One project in the Recent list.
#[derive(Clone, Debug)]
pub struct Recent {
    /// The project's name.
    pub name: String,
    /// How many of the five [`STAGES`] are done.
    pub done: u8,
    /// Whether the next stage is under way.
    pub under_way: bool,
    /// The deadline, or "no deadline".
    pub when: String,
    /// The chip's text.
    pub chip: String,
    /// The chip's colour.
    pub chip_kind: ChipKind,
}

/// The Crew icons the opening screen uses; `None` draws the painted stand-in.
#[derive(Clone, Default)]
pub struct HomeIcons {
    /// `app.settings`, header.
    pub settings: Option<TextureHandle>,
    /// `app.help`, header.
    pub help: Option<TextureHandle>,
    /// `app.new`, the Start card.
    pub new: Option<TextureHandle>,
    /// `app.open`, the Keep going card.
    pub open: Option<TextureHandle>,
    /// `app.workshop`, the workshop row.
    pub workshop: Option<TextureHandle>,
}

impl HomeIcons {
    /// The ids, and the size in points each is drawn at.
    pub const WANTED: [(&'static str, f32); 5] = [
        ("app.settings", 28.0),
        ("app.help", 28.0),
        ("app.new", 48.0),
        ("app.open", 48.0),
        ("app.workshop", 44.0),
    ];
}

/// Everything the opening screen shows.
pub struct HomeModel<'a> {
    /// The look's colours.
    pub palette: Palette,
    /// Crew icons, where they have landed.
    pub icons: HomeIcons,
    /// Recent projects, newest first.
    pub recents: &'a [Recent],
    /// The project to continue and the note about where it stopped.
    pub keep_going: Option<(&'a str, &'a str)>,
    /// The line under "My workshop".
    pub workshop_line: String,
    /// What Sancho says.
    pub sancho_says: &'a str,
    /// Sancho's height, in millimetres.
    pub sancho_height_mm: f64,
}

/// What the person asked for on the opening screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HomeAction {
    /// "Start something new": the Quest.
    StartNew,
    /// "Keep going": the project in progress.
    KeepGoing,
    /// A row of the Recent list.
    OpenRecent(usize),
    /// "Open workshop".
    OpenWorkshop,
    /// "Import project".
    ImportProject,
    /// "Examples".
    Examples,
    /// The gear.
    Settings,
    /// The question mark.
    Help,
}

/// Width at which Sancho moves from beside the cards to below them.
const SIDE_BY_SIDE: f32 = 980.0;
/// Sancho's column.
const ASIDE_WIDTH: f32 = 290.0;

/// Draws the opening screen into `ui` and returns what the person clicked, if anything.
pub fn show(ui: &mut Ui, m: &HomeModel<'_>) -> Option<HomeAction> {
    let p = m.palette;
    paint_background(ui, &p);
    let mut action = None;

    Frame::new()
        .inner_margin(Margin::symmetric(32, 18))
        .show(ui, |ui| {
            header(ui, &p, &m.icons, &mut action);
        });

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            Frame::new()
                .inner_margin(Margin {
                    left: 32,
                    right: 32,
                    top: 8,
                    bottom: 32,
                })
                .show(ui, |ui| {
                    let width = ui.available_width().min(1240.0);
                    let side = width >= SIDE_BY_SIDE;
                    let main_width = if side {
                        width - ASIDE_WIDTH - 32.0
                    } else {
                        width
                    };
                    ui.horizontal_top(|ui| {
                        ui.allocate_ui_with_layout(
                            vec2(main_width, 0.0),
                            Layout::top_down(Align::Min),
                            |ui| {
                                ui.set_width(main_width);
                                main_column(ui, m, &mut action);
                                if !side {
                                    ui.add_space(24.0);
                                    aside(ui, m);
                                }
                            },
                        );
                        if side {
                            ui.add_space(32.0);
                            ui.allocate_ui_with_layout(
                                vec2(ASIDE_WIDTH, 0.0),
                                Layout::top_down(Align::Center),
                                |ui| {
                                    ui.set_width(ASIDE_WIDTH);
                                    aside(ui, m);
                                },
                            );
                        }
                    });
                });
        });
    action
}

fn paint_background(ui: &Ui, p: &Palette) {
    let r = ui.max_rect();
    let mut mesh = Mesh::default();
    mesh.colored_vertex(r.left_top(), p.bg_top);
    mesh.colored_vertex(r.right_top(), p.bg_top);
    mesh.colored_vertex(r.left_bottom(), p.bg_bottom);
    mesh.colored_vertex(r.right_bottom(), p.bg_bottom);
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(1, 3, 2);
    ui.painter().add(Shape::mesh(mesh));
}

fn header(ui: &mut Ui, p: &Palette, icons: &HomeIcons, action: &mut Option<HomeAction>) {
    ui.horizontal(|ui| {
        let (logo, _) = ui.allocate_exact_size(vec2(40.0, 40.0), Sense::hover());
        paint_logo(ui, logo);
        ui.label(RichText::new("MakerCAD").size(24.0).strong().color(p.ink));
        ui.label(
            RichText::new("Design with what you have")
                .size(14.0)
                .color(p.muted),
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if square_button(ui, p, icons.help.as_ref(), "?", "Help").clicked() {
                *action = Some(HomeAction::Help);
            }
            if square_button(ui, p, icons.settings.as_ref(), "\u{2699}", "Settings").clicked() {
                *action = Some(HomeAction::Settings);
            }
        });
    });
}

/// Draws `texture` to fill `rect`.
fn paint_texture(ui: &Ui, texture: &TextureHandle, rect: Rect) {
    let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
    ui.painter().image(texture.id(), rect, uv, Color32::WHITE);
}

/// MakerCAD's mark: a coral tile with a pencil crossing a ruler.
fn paint_logo(ui: &Ui, rect: Rect) {
    let painter = ui.painter();
    painter.rect_filled(rect, CornerRadius::same(12), ACCENT);
    let c = rect.center();
    let ruler = Rect::from_center_size(c + vec2(0.0, 6.0), vec2(26.0, 8.0));
    painter.rect_filled(ruler, CornerRadius::same(2), PAPER);
    for i in 0..5 {
        let x = ruler.left() + 4.0 + i as f32 * 4.5;
        let h = if i % 2 == 0 { 4.0 } else { 2.5 };
        painter.line_segment(
            [pos2(x, ruler.top()), pos2(x, ruler.top() + h)],
            Stroke::new(1.2, SANCHO_INK),
        );
    }
    let a = c + vec2(-9.0, 4.0);
    let b = c + vec2(8.0, -13.0);
    painter.line_segment([a, b], Stroke::new(5.0, SANCHO_INK));
    painter.line_segment([a, a + (b - a) * 0.18], Stroke::new(5.0, PAPER));
}

fn square_button(
    ui: &mut Ui,
    p: &Palette,
    icon: Option<&TextureHandle>,
    glyph: &str,
    hint: &str,
) -> Response {
    let label = if icon.is_some() {
        RichText::new("")
    } else {
        RichText::new(glyph).size(20.0).strong().color(p.ink)
    };
    let response = ui
        .add(
            egui::Button::new(label)
                .min_size(vec2(44.0, 44.0))
                .fill(p.panel)
                .stroke(Stroke::new(2.0, p.line))
                .corner_radius(CornerRadius::same(14)),
        )
        .on_hover_text(hint);
    if let Some(texture) = icon {
        paint_texture(
            ui,
            texture,
            Rect::from_center_size(response.rect.center(), vec2(28.0, 28.0)),
        );
    }
    response
}

fn main_column(ui: &mut Ui, m: &HomeModel<'_>, action: &mut Option<HomeAction>) {
    let p = m.palette;
    ui.columns(2, |cols| {
        let new_icon = m
            .icons
            .new
            .as_ref()
            .map_or(Icon::Plus, |t| Icon::Texture(t.clone()));
        let open_icon = m
            .icons
            .open
            .as_ref()
            .map_or(Icon::Arrow, |t| Icon::Texture(t.clone()));
        if big_card(&mut cols[0], &p, new_icon, "Start something new", |ui| {
            ui.label(
                RichText::new(
                    "Tell Sancho what you want to make, what you have to make it with, and \
                     where it has to fit. Every question can be skipped.",
                )
                .size(16.0)
                .color(p.muted),
            );
        })
        .clicked()
        {
            *action = Some(HomeAction::StartNew);
        }
        if big_card(&mut cols[1], &p, open_icon, "Keep going", |ui| {
            match m.keep_going {
                Some((project, note)) => {
                    // One wrapped paragraph in two colours (a row of labels would wrap at the
                    // 44-point touch height, with gaps between the lines).
                    let mut job = egui::text::LayoutJob::default();
                    let format = |color: Color32| egui::TextFormat {
                        font_id: FontId::proportional(16.0),
                        color,
                        ..Default::default()
                    };
                    job.append(&format!("{project}. "), 0.0, format(p.ink));
                    job.append(note, 0.0, format(p.muted));
                    job.wrap.max_width = ui.available_width();
                    ui.label(job);
                }
                None => {
                    ui.label(
                        RichText::new(
                            "Nothing under way yet. The project you are building shows up here, \
                         with where you stopped.",
                        )
                        .size(16.0)
                        .color(p.muted),
                    );
                }
            }
        })
        .clicked()
        {
            *action = Some(HomeAction::KeepGoing);
        }
    });
    ui.add_space(28.0);
    recent_projects(ui, m, action);
    ui.add_space(28.0);
    workshop(ui, m, action);
}

#[derive(Clone)]
enum Icon {
    Plus,
    Arrow,
    Texture(TextureHandle),
}

fn big_card(
    ui: &mut Ui,
    p: &Palette,
    icon: Icon,
    title: &str,
    body: impl FnOnce(&mut Ui),
) -> Response {
    let frame = card_frame(p, 28, 28);
    let shown = frame.show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.set_min_height(150.0);
        let (r, _) = ui.allocate_exact_size(vec2(48.0, 48.0), Sense::hover());
        paint_icon(ui, r, icon, p);
        ui.add_space(6.0);
        ui.label(RichText::new(title).size(26.0).strong().color(p.ink));
        body(ui);
    });
    clickable(ui, shown.response, 28)
}

fn card_frame(p: &Palette, radius: u8, margin: i8) -> Frame {
    Frame::new()
        .fill(p.panel)
        .stroke(Stroke::new(2.0, p.line))
        .corner_radius(CornerRadius::same(radius))
        .inner_margin(Margin::same(margin))
        .shadow(Shadow {
            offset: [0, 6],
            blur: 0,
            spread: 0,
            color: p.shadow,
        })
}

/// Makes a drawn card clickable, with a coral outline and a hand cursor on hover.
fn clickable(ui: &Ui, shown: Response, radius: u8) -> Response {
    let response = ui.interact(shown.rect, shown.id.with("click"), Sense::click());
    if response.hovered() {
        ui.painter().rect_stroke(
            shown.rect,
            CornerRadius::same(radius),
            Stroke::new(2.5, ACCENT),
            StrokeKind::Inside,
        );
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    }
    response
}

fn paint_icon(ui: &Ui, r: Rect, icon: Icon, p: &Palette) {
    let painter = ui.painter();
    let c = r.center();
    let ink = Stroke::new(4.0, SANCHO_INK);
    match icon {
        Icon::Texture(texture) => paint_texture(ui, &texture, r),
        Icon::Plus => {
            painter.circle_filled(c, 22.0, ACCENT);
            painter.line_segment([c - vec2(10.0, 0.0), c + vec2(10.0, 0.0)], ink);
            painter.line_segment([c - vec2(0.0, 10.0), c + vec2(0.0, 10.0)], ink);
        }
        Icon::Arrow => {
            let fill = if p.dark {
                Color32::from_rgb(0x3E, 0x4A, 0x6B)
            } else {
                p.chip
            };
            painter.circle_filled(c, 22.0, fill);
            painter.circle_stroke(c, 22.0, Stroke::new(2.0, p.line));
            let ink = Stroke::new(4.0, p.ink);
            painter.line_segment([c - vec2(10.0, 0.0), c + vec2(9.0, 0.0)], ink);
            painter.line_segment([c + vec2(10.0, 0.0), c + vec2(2.0, -8.0)], ink);
            painter.line_segment([c + vec2(10.0, 0.0), c + vec2(2.0, 8.0)], ink);
        }
    }
}

fn recent_projects(ui: &mut Ui, m: &HomeModel<'_>, action: &mut Option<HomeAction>) {
    let p = m.palette;
    card_frame(&p, 24, 22).shadow(Shadow::NONE).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(
            RichText::new("Recent projects")
                .size(18.0)
                .strong()
                .color(p.ink),
        );
        ui.add_space(6.0);
        if m.recents.is_empty() {
            ui.label(
                RichText::new(
                    "No projects yet. Start something new, or open an example to see how one \
                     goes from idea to cut list.",
                )
                .size(15.0)
                .color(p.muted),
            );
        }
        for (i, recent) in m.recents.iter().enumerate() {
            if recent_row(ui, &p, recent).clicked() {
                *action = Some(HomeAction::OpenRecent(i));
            }
        }
        ui.add_space(4.0);
        ui.label(
            RichText::new(format!("Dots: {}", STAGES.join(" \u{b7} ")))
                .size(13.0)
                .color(p.muted),
        );
    });
}

fn recent_row(ui: &mut Ui, p: &Palette, r: &Recent) -> Response {
    let row = ui.horizontal(|ui| {
        ui.set_min_height(40.0);
        let (icon, _) = ui.allocate_exact_size(vec2(28.0, 28.0), Sense::hover());
        ui.painter()
            .rect_filled(icon, CornerRadius::same(8), p.chip);
        let initial: String = r.name.chars().take(1).collect();
        ui.painter().text(
            icon.center(),
            Align2::CENTER_CENTER,
            initial,
            FontId::proportional(15.0),
            p.ink,
        );
        let fixed = 5.0 * 17.0 + 120.0 + 130.0 + 40.0;
        let name_width = (ui.available_width() - fixed).max(120.0);
        ui.allocate_ui_with_layout(
            vec2(name_width, 28.0),
            Layout::left_to_right(Align::Center),
            |ui| {
                ui.set_width(name_width);
                ui.add(egui::Label::new(RichText::new(&r.name).size(16.0).strong()).truncate());
            },
        );
        stage_dots(ui, p, r.done, r.under_way);
        ui.allocate_ui_with_layout(
            vec2(120.0, 28.0),
            Layout::left_to_right(Align::Center),
            |ui| {
                ui.set_width(120.0);
                ui.label(RichText::new(&r.when).size(14.0).color(p.muted));
            },
        );
        let (fill, text) = match r.chip_kind {
            ChipKind::Waiting => (CHIP_WAITING, SANCHO_INK),
            ChipKind::Done => (CHIP_DONE, SANCHO_INK),
            ChipKind::Neutral => (p.chip, p.ink),
        };
        Frame::new()
            .fill(fill)
            .corner_radius(CornerRadius::same(10))
            .inner_margin(Margin::symmetric(10, 3))
            .show(ui, |ui| {
                ui.label(RichText::new(&r.chip).size(13.0).strong().color(text));
            });
    });
    clickable(ui, row.response, 14)
}

fn stage_dots(ui: &mut Ui, p: &Palette, done: u8, under_way: bool) {
    let (rect, response) = ui.allocate_exact_size(vec2(5.0 * 17.0, 14.0), Sense::hover());
    let painter = ui.painter();
    for i in 0..5u8 {
        let c = pos2(rect.left() + 7.0 + f32::from(i) * 17.0, rect.center().y);
        if i < done {
            painter.circle_filled(c, 6.0, STAGE_DONE);
        } else if i == done && under_way {
            painter.circle_filled(c, 6.0, STAGE_NOW);
        } else {
            painter.circle_stroke(c, 5.0, Stroke::new(2.0, p.line));
        }
    }
    let now = STAGES.get(usize::from(done)).copied().unwrap_or("finished");
    response.on_hover_text(format!(
        "{} of 5 stages done{}",
        done,
        if under_way {
            format!("; {now} under way")
        } else {
            String::new()
        }
    ));
}

fn workshop(ui: &mut Ui, m: &HomeModel<'_>, action: &mut Option<HomeAction>) {
    let p = m.palette;
    card_frame(&p, 24, 20).shadow(Shadow::NONE).show(ui, |ui| {
        ui.set_width(ui.available_width());
        // The text gets the whole row; the buttons sit on their own row under it, so a long
        // adapter name never runs under a button.
        ui.horizontal(|ui| {
            let (r, _) = ui.allocate_exact_size(vec2(44.0, 44.0), Sense::hover());
            match &m.icons.workshop {
                Some(texture) => paint_texture(ui, texture, r),
                None => paint_toolbox(ui, r, &p),
            }
            ui.vertical(|ui| {
                ui.spacing_mut().interact_size.y = 0.0;
                ui.label(
                    RichText::new("My workshop")
                        .size(16.0)
                        .strong()
                        .color(p.ink),
                );
                ui.add(
                    egui::Label::new(RichText::new(&m.workshop_line).size(15.0).color(p.muted))
                        .wrap(),
                );
            });
        });
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            for (label, what) in [
                ("Open workshop", HomeAction::OpenWorkshop),
                ("Import project", HomeAction::ImportProject),
                ("Examples", HomeAction::Examples),
            ] {
                let button = egui::Button::new(RichText::new(label).size(15.0).strong())
                    .min_size(vec2(0.0, 44.0))
                    .fill(p.bg2)
                    .stroke(Stroke::new(2.0, p.line));
                if ui.add(button).clicked() {
                    *action = Some(what);
                }
            }
        });
    });
}

fn paint_toolbox(ui: &Ui, r: Rect, p: &Palette) {
    let painter = ui.painter();
    let body = Rect::from_min_max(
        pos2(r.left() + 3.0, r.top() + 16.0),
        pos2(r.right() - 3.0, r.bottom() - 4.0),
    );
    painter.rect_filled(body, CornerRadius::same(6), crate::theme::WOOD);
    painter.rect_stroke(
        body,
        CornerRadius::same(6),
        Stroke::new(2.5, SANCHO_INK),
        StrokeKind::Inside,
    );
    let handle = Rect::from_min_max(
        pos2(r.center().x - 9.0, r.top() + 6.0),
        pos2(r.center().x + 9.0, r.top() + 17.0),
    );
    painter.rect_stroke(
        handle,
        CornerRadius::same(4),
        Stroke::new(2.5, p.ink),
        StrokeKind::Inside,
    );
    painter.line_segment(
        [
            pos2(body.left(), body.top() + 9.0),
            pos2(body.right(), body.top() + 9.0),
        ],
        Stroke::new(2.0, SANCHO_INK),
    );
}

fn aside(ui: &mut Ui, m: &HomeModel<'_>) {
    let p = m.palette;
    ui.add_space(12.0);
    Frame::new()
        .fill(PAPER)
        .stroke(Stroke::new(2.0, SANCHO_INK))
        .corner_radius(CornerRadius::same(22))
        .inner_margin(Margin::symmetric(18, 16))
        .show(ui, |ui| {
            ui.set_max_width(250.0);
            ui.label(RichText::new(m.sancho_says).size(19.0).color(SANCHO_INK));
        });
    let (figure, _) = ui.allocate_exact_size(vec2(200.0, 330.0), Sense::hover());
    let tail = figure.center_top() + vec2(-8.0, 0.0);
    ui.painter().add(Shape::convex_polygon(
        vec![
            tail + vec2(-10.0, -2.0),
            tail + vec2(10.0, -2.0),
            tail + vec2(4.0, 14.0),
        ],
        PAPER,
        Stroke::new(2.0, SANCHO_INK),
    ));
    sancho::paint(
        ui.painter(),
        figure.shrink2(vec2(0.0, 12.0)).translate(vec2(0.0, 10.0)),
    );
    ui.label(
        RichText::new(height_caption(m.sancho_height_mm))
            .size(13.0)
            .color(p.muted),
    );
}

/// "Sancho is 168 cm (5' 6")": metric first, then feet and inches through mk-units. Plain
/// ASCII marks: the default fonts have no prime glyphs (they drew as boxes).
pub fn height_caption(mm: f64) -> String {
    let inches = (mm / mk_units::mm_per(mk_units::Unit::In)).round() as i64;
    format!(
        "Sancho is {} cm ({}' {}\")",
        (mm / 10.0).round() as i64,
        inches / 12,
        inches % 12
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_caption_gives_metric_then_feet_and_inches() {
        assert_eq!(height_caption(1680.0), "Sancho is 168 cm (5' 6\")");
        assert_eq!(height_caption(1829.0), "Sancho is 183 cm (6' 0\")");
    }
}
