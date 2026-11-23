//! The workspace's shell around a design (doc 11 §11.4, `docs/design/ui-v2`): the project bar
//! along the top, the environment tabs under it, the toolbox down the left, the status bar along
//! the bottom, and command search over all of it (Ctrl+K).
//!
//! Each piece draws itself and returns the [`Act`] the person asked for; `lib.rs` carries it
//! out, so a click, a search result and a shortcut all go the same way. Icons come from the
//! Crew's set (doc 18); one that has not landed yet is drawn as a painted stand-in.

use egui::{
    pos2, vec2, Align, Align2, Color32, Context, CornerRadius, FontId, Frame, Id, Key, Layout,
    Margin, Modifiers, Rect, Response, RichText, Sense, Shape, Stroke, StrokeKind, TextureHandle,
    Ui,
};
use solidcraft_viewport::StandardView;

use crate::commands::{self, Act, Command, Later, Units};
use crate::environments::{Env, MORE};
use crate::icons::Icons;
use crate::theme::{Look, Palette, ACCENT, CHIP_WAITING, LINK, STAGE_NOW};

/// The project bar's height.
pub const BAR_H: f32 = 56.0;
/// The environment tabs' height.
pub const TABS_H: f32 = 44.0;
/// The toolbox's width.
pub const TOOL_W: f32 = 76.0;

/// What the shell's pieces draw with: the look's colours and the icons.
pub struct Kit<'a> {
    /// The look's colours.
    pub p: Palette,
    look: Look,
    icons: &'a mut Icons,
}

impl<'a> Kit<'a> {
    /// A kit for `look`.
    pub fn new(look: Look, icons: &'a mut Icons) -> Self {
        Self {
            p: look.palette(),
            look,
            icons,
        }
    }

    fn texture(&mut self, ctx: &Context, id: &str, points: f32) -> Option<TextureHandle> {
        let px = (points * ctx.pixels_per_point()).round().clamp(1.0, 512.0) as u32;
        self.icons.get(ctx, id, self.look, px)
    }

    /// Paints icon `id` to fill `rect`; faded when `dim`. An icon that has not landed is drawn as
    /// a rounded square with the first letter of `label`.
    pub fn paint(&mut self, ui: &Ui, id: &str, rect: Rect, label: &str, dim: bool) {
        let tint = if dim {
            Color32::from_white_alpha(90)
        } else {
            Color32::WHITE
        };
        if let Some(texture) = self.texture(ui.ctx(), id, rect.height()) {
            let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
            ui.painter().image(texture.id(), rect, uv, tint);
            return;
        }
        let ink = if dim {
            self.p.muted.gamma_multiply(0.6)
        } else {
            self.p.ink
        };
        let r = rect.shrink(rect.width() * 0.1);
        ui.painter().rect(
            r,
            CornerRadius::same((r.width() * 0.25) as u8),
            self.p.panel,
            Stroke::new((r.width() / 14.0).max(1.0), ink),
            StrokeKind::Inside,
        );
        let letter: String = label.chars().take(1).collect();
        ui.painter().text(
            r.center(),
            Align2::CENTER_CENTER,
            letter,
            FontId::proportional(r.height() * 0.6),
            ink,
        );
    }
}

/// A square icon button with a hint; `selected` shows it as on.
#[allow(clippy::too_many_arguments)]
fn icon_button(
    ui: &mut Ui,
    kit: &mut Kit<'_>,
    id: &str,
    label: &str,
    hint: &str,
    enabled: bool,
    selected: bool,
    size: f32,
) -> Response {
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(vec2(size, size), sense);
    let hovered = enabled && response.hovered();
    if selected || hovered {
        ui.painter().rect(
            rect,
            CornerRadius::same(10),
            if selected { kit.p.chip } else { kit.p.bg2 },
            Stroke::new(1.5, if hovered { ACCENT } else { kit.p.line }),
            StrokeKind::Inside,
        );
    }
    kit.paint(ui, id, rect.shrink(size * 0.15), label, !enabled);
    response.on_hover_text(hint)
}

/// A small filled triangle pointing down (the default font has no ▾).
fn paint_caret(ui: &Ui, at: egui::Pos2, colour: Color32) {
    ui.painter().add(Shape::convex_polygon(
        vec![
            at + vec2(-5.0, -3.0),
            at + vec2(5.0, -3.0),
            at + vec2(0.0, 3.0),
        ],
        colour,
        Stroke::NONE,
    ));
}

/// What the project bar shows.
pub struct BarModel<'a> {
    /// The project's name.
    pub name: &'a str,
    /// What it is (for an example), shown in the intent pill.
    pub about: Option<&'a str>,
    /// What Undo would take back, if anything.
    pub undo: Option<&'a str>,
    /// What Redo would put back, if anything.
    pub redo: Option<&'a str>,
    /// Sancho: shown or not, and his label ("Sancho 1.68 m"); `None` when there is no figure.
    pub sancho: Option<(bool, String)>,
    /// SolidCraft's examples (name, one line), for the name's menu.
    pub examples: &'a [(&'static str, &'static str)],
}

/// The project bar: home, the project's name and its menu, what it is, command search, undo and
/// redo, Sancho, help and settings.
pub fn project_bar(ui: &mut Ui, kit: &mut Kit<'_>, m: &BarModel<'_>) -> Option<Act> {
    let mut act = None;
    ui.horizontal_centered(|ui| {
        if icon_button(
            ui,
            kit,
            "app.home",
            "Home",
            "Home: the opening screen",
            true,
            false,
            40.0,
        )
        .clicked()
        {
            act = Some(Act::Home);
        }
        ui.add_space(4.0);
        let menu = ui.menu_button(
            RichText::new(format!("{}     ", m.name))
                .size(18.0)
                .strong()
                .color(kit.p.ink),
            |ui| {
                ui.label(
                    RichText::new("Open an example")
                        .size(13.0)
                        .color(kit.p.muted),
                );
                for (i, (name, about)) in m.examples.iter().enumerate() {
                    if ui.button(*name).on_hover_text(*about).clicked() {
                        act = Some(Act::OpenExample(i));
                        ui.close();
                    }
                }
                ui.separator();
                if ui.button("Go to the opening screen").clicked() {
                    act = Some(Act::Home);
                    ui.close();
                }
            },
        );
        let r = menu.response.rect;
        paint_caret(ui, pos2(r.right() - 16.0, r.center().y), kit.p.muted);
        if let Some(about) = m.about {
            intent_pill(ui, kit, about);
        }

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if icon_button(
                ui,
                kit,
                "app.settings",
                "Settings",
                "Settings",
                true,
                false,
                40.0,
            )
            .clicked()
            {
                act = Some(Act::Settings);
            }
            if icon_button(
                ui,
                kit,
                "app.help",
                "Help",
                "About MakerCAD",
                true,
                false,
                40.0,
            )
            .clicked()
            {
                act = Some(Act::About);
            }
            if let Some((shown, label)) = &m.sancho {
                let hint = if *shown {
                    format!("{label}, standing beside the design. Click to hide him")
                } else {
                    format!("Show {label} beside the design, at true size")
                };
                if icon_button(ui, kit, "app.sancho", "Sancho", &hint, true, *shown, 40.0).clicked()
                {
                    act = Some(Act::Sancho);
                }
            }
            ui.add_space(6.0);
            let redo_hint = m.redo.map_or("Nothing to redo".to_string(), |l| {
                format!("Redo: {l} (Ctrl+Y)")
            });
            if icon_button(
                ui,
                kit,
                "app.redo",
                "Redo",
                &redo_hint,
                m.redo.is_some(),
                false,
                36.0,
            )
            .clicked()
            {
                act = Some(Act::Redo);
            }
            let undo_hint = m.undo.map_or("Nothing to undo".to_string(), |l| {
                format!("Undo: {l} (Ctrl+Z)")
            });
            if icon_button(
                ui,
                kit,
                "app.undo",
                "Undo",
                &undo_hint,
                m.undo.is_some(),
                false,
                36.0,
            )
            .clicked()
            {
                act = Some(Act::Undo);
            }
            ui.add_space(8.0);
            if search_box(ui, kit).clicked() {
                act = Some(Act::Search);
            }
        });
    });
    act
}

/// The pill saying what the project is: the person's intent, or an example's one line.
fn intent_pill(ui: &mut Ui, kit: &mut Kit<'_>, about: &str) {
    let text = format!("Example: {about}");
    let font = FontId::proportional(13.0);
    let galley = ui
        .painter()
        .layout_no_wrap(text.clone(), font.clone(), kit.p.muted);
    let room = (ui.available_width() - 470.0).max(120.0);
    let w = (galley.size().x + 44.0).min(room);
    let (rect, response) = ui.allocate_exact_size(vec2(w, 32.0), Sense::hover());
    ui.painter().rect(
        rect,
        CornerRadius::same(16),
        kit.p.bg2,
        Stroke::new(1.5, kit.p.line),
        StrokeKind::Inside,
    );
    kit.paint(
        ui,
        "app.intent",
        Rect::from_center_size(pos2(rect.left() + 18.0, rect.center().y), vec2(20.0, 20.0)),
        "Intent",
        false,
    );
    let clip = Rect::from_min_max(
        pos2(rect.left() + 32.0, rect.top()),
        pos2(rect.right() - 12.0, rect.bottom()),
    );
    ui.painter_at(clip).text(
        pos2(clip.left(), rect.center().y),
        Align2::LEFT_CENTER,
        text,
        font,
        kit.p.muted,
    );
    response.on_hover_text(format!(
        "What this project is. For your own projects the Quest asks what you want to make (P2).\n\n{about}"
    ));
}

/// The search box in the project bar; a click opens command search.
fn search_box(ui: &mut Ui, kit: &mut Kit<'_>) -> Response {
    let w = 300.0_f32.min((ui.available_width() - 40.0).max(160.0));
    let (rect, response) = ui.allocate_exact_size(vec2(w, 36.0), Sense::click());
    let stroke = if response.hovered() {
        ACCENT
    } else {
        kit.p.line
    };
    ui.painter().rect(
        rect,
        CornerRadius::same(18),
        kit.p.panel,
        Stroke::new(1.5, stroke),
        StrokeKind::Inside,
    );
    kit.paint(
        ui,
        "app.search",
        Rect::from_center_size(pos2(rect.left() + 20.0, rect.center().y), vec2(22.0, 22.0)),
        "Search",
        false,
    );
    ui.painter().text(
        pos2(rect.left() + 38.0, rect.center().y),
        Align2::LEFT_CENTER,
        "Search or say what to do\u{2026}",
        FontId::proportional(14.0),
        kit.p.muted,
    );
    if rect.width() > 230.0 {
        let chip =
            Rect::from_center_size(pos2(rect.right() - 32.0, rect.center().y), vec2(46.0, 22.0));
        ui.painter()
            .rect_filled(chip, CornerRadius::same(6), kit.p.chip);
        ui.painter().text(
            chip.center(),
            Align2::CENTER_CENTER,
            "Ctrl K",
            FontId::proportional(11.0),
            kit.p.muted,
        );
    }
    response.on_hover_text("Command search: type what you want to do, in your own words (Ctrl+K)")
}

/// The environment tabs, and the "+ environment" menu of the others.
pub fn env_tabs(ui: &mut Ui, kit: &mut Kit<'_>, active: Env) -> Option<Act> {
    let mut act = None;
    ui.horizontal(|ui| {
        ui.add_space(4.0);
        for env in Env::TABS {
            let on = env == active;
            let font = FontId::proportional(15.0);
            let colour = if on { kit.p.ink } else { kit.p.muted };
            let galley = ui
                .painter()
                .layout_no_wrap(env.name().to_string(), font.clone(), colour);
            let size = vec2(galley.size().x + 54.0, TABS_H - 6.0);
            let (rect, response) = ui.allocate_exact_size(size, Sense::click());
            let rect = rect.translate(vec2(0.0, 3.0));
            if on {
                ui.painter().rect(
                    rect,
                    CornerRadius {
                        nw: 10,
                        ne: 10,
                        sw: 0,
                        se: 0,
                    },
                    kit.p.panel,
                    Stroke::new(1.5, kit.p.line),
                    StrokeKind::Inside,
                );
                let bar = Rect::from_min_max(
                    pos2(rect.left() + 12.0, rect.bottom() - 4.0),
                    pos2(rect.right() - 12.0, rect.bottom()),
                );
                ui.painter().rect_filled(bar, CornerRadius::same(2), ACCENT);
            } else if response.hovered() {
                ui.painter()
                    .rect_filled(rect, CornerRadius::same(10), kit.p.chip);
            }
            kit.paint(
                ui,
                env.icon(),
                Rect::from_center_size(pos2(rect.left() + 21.0, rect.center().y), vec2(22.0, 22.0)),
                env.name(),
                false,
            );
            ui.painter().galley(
                pos2(rect.left() + 38.0, rect.center().y - galley.size().y / 2.0),
                galley,
                colour,
            );
            let hint = match env.plan() {
                None => format!("{}: works now", env.name()),
                Some(p) => format!("{} (comes in {}): {}", env.name(), p.phase, p.does),
            };
            if response.on_hover_text(hint).clicked() {
                act = Some(Act::Tab(env));
            }
        }
        ui.add_space(8.0);
        // As tall as a tab, not a full-size button.
        ui.spacing_mut().interact_size.y = 30.0;
        ui.spacing_mut().button_padding = vec2(10.0, 4.0);
        ui.menu_button(
            RichText::new("+ environment").size(14.0).color(kit.p.muted),
            |ui| {
                ui.label(
                    RichText::new("More environments, each loaded when a project needs it")
                        .size(13.0)
                        .color(kit.p.muted),
                );
                for more in MORE {
                    let (rect, response) =
                        ui.allocate_exact_size(vec2(330.0, 34.0), Sense::click());
                    if response.hovered() {
                        ui.painter()
                            .rect_filled(rect, CornerRadius::same(8), kit.p.chip);
                    }
                    kit.paint(
                        ui,
                        more.icon,
                        Rect::from_center_size(
                            pos2(rect.left() + 18.0, rect.center().y),
                            vec2(22.0, 22.0),
                        ),
                        more.name,
                        false,
                    );
                    ui.painter().text(
                        pos2(rect.left() + 38.0, rect.center().y),
                        Align2::LEFT_CENTER,
                        more.name,
                        FontId::proportional(15.0),
                        kit.p.ink,
                    );
                    ui.painter().text(
                        pos2(rect.right() - 8.0, rect.center().y),
                        Align2::RIGHT_CENTER,
                        format!("comes in {}", more.plan.phase),
                        FontId::proportional(12.0),
                        kit.p.muted,
                    );
                    if response.on_hover_text(more.plan.does).clicked() {
                        act = Some(Act::Later(Later {
                            what: more.name,
                            phase: more.plan.phase,
                            note: more.plan.does,
                        }));
                        ui.close();
                    }
                }
            },
        );
    });
    act
}

/// The toolbox's view group: the commands that work on the view today. The sketch and make tools
/// join it with the modelling tools (next in P1).
const VIEW_TOOLS: [(&str, &str, &str, Act); 6] = [
    (
        "view.fit",
        "Fit",
        "Fit the view to the design (double-click the view)",
        Act::FitView,
    ),
    (
        "view.zoom",
        "All",
        "Show everything: the design, its bench or floor, and Sancho",
        Act::WholeView,
    ),
    (
        "view.iso",
        "Corner",
        "Look from the corner",
        Act::LookFrom(StandardView::Iso),
    ),
    (
        "view.front",
        "Front",
        "Look from the front",
        Act::LookFrom(StandardView::Front),
    ),
    (
        "view.top",
        "Top",
        "Look from the top",
        Act::LookFrom(StandardView::Top),
    ),
    (
        "view.right",
        "Right",
        "Look from the right",
        Act::LookFrom(StandardView::Right),
    ),
];

/// The toolbox down the left of the Design workspace.
pub fn toolbox(ui: &mut Ui, kit: &mut Kit<'_>) -> Option<Act> {
    let mut act = None;
    ui.vertical_centered(|ui| {
        ui.add_space(8.0);
        ui.label(
            RichText::new("Toolbox")
                .size(11.0)
                .strong()
                .color(kit.p.muted),
        );
        ui.add_space(4.0);
        ui.label(RichText::new("VIEW").size(10.0).strong().color(kit.p.muted));
        for (icon, label, hint, tool) in VIEW_TOOLS {
            let (rect, response) =
                ui.allocate_exact_size(vec2(TOOL_W - 12.0, 54.0), Sense::click());
            if response.hovered() {
                ui.painter().rect(
                    rect,
                    CornerRadius::same(10),
                    kit.p.chip,
                    Stroke::new(1.5, ACCENT),
                    StrokeKind::Inside,
                );
            }
            kit.paint(
                ui,
                icon,
                Rect::from_center_size(pos2(rect.center().x, rect.top() + 19.0), vec2(28.0, 28.0)),
                label,
                false,
            );
            ui.painter().text(
                pos2(rect.center().x, rect.bottom() - 8.0),
                Align2::CENTER_CENTER,
                label,
                FontId::proportional(11.0),
                kit.p.ink,
            );
            if response.on_hover_text(hint).clicked() {
                act = Some(tool);
            }
        }
        ui.add_space(12.0);
        ui.label(
            RichText::new("Sketch and make tools come next")
                .size(10.0)
                .color(kit.p.muted),
        );
    });
    act
}

/// What the status bar shows.
pub struct StatusModel<'a> {
    /// Lengths in mm or inches: the open project's own system, else the default for new ones.
    pub units: Units,
    /// Whether the units belong to an open project (so the toggle changes that project).
    pub document: bool,
    /// Sancho's height in mm, when a design with him is open.
    pub sancho_mm: Option<f64>,
    /// The graphics backend in short ("dx12").
    pub backend: &'a str,
    /// The details for the backend's hover: version, timings, adapter.
    pub details: &'a str,
    /// Whether a design is open (the material meter belongs to a design).
    pub design: bool,
}

/// The status bar: the material meter (P3), then units, Sancho's height and the graphics backend.
pub fn status_bar(ui: &mut Ui, kit: &mut Kit<'_>, m: &StatusModel<'_>) -> Option<Act> {
    let mut act = None;
    // A slim bar: small buttons, not the 44 px ones the rest of the shell uses.
    ui.spacing_mut().interact_size.y = 20.0;
    ui.spacing_mut().button_padding = vec2(8.0, 1.0);
    ui.horizontal_centered(|ui| {
        if m.design {
            let (rect, _) = ui.allocate_exact_size(vec2(20.0, 20.0), Sense::hover());
            kit.paint(ui, "stock.meter", rect, "Material", false);
            ui.label(
                RichText::new("Material")
                    .size(12.0)
                    .strong()
                    .color(kit.p.ink),
            );
            ui.label(
                RichText::new(
                    "not chosen yet: the meter fills when parts get their boards (Stock & Cut, P3)",
                )
                .size(12.0)
                .color(kit.p.muted),
            );
        } else {
            ui.label(
                RichText::new(format!(
                    "MakerCAD {}  \u{b7}  foundation build",
                    env!("CARGO_PKG_VERSION")
                ))
                .size(12.0)
                .color(kit.p.muted),
            );
        }
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(RichText::new(m.backend).size(12.0).color(kit.p.muted))
                .on_hover_text(m.details);
            ui.label(RichText::new("|").size(12.0).color(kit.p.line));
            if let Some(mm) = m.sancho_mm {
                ui.label(
                    RichText::new(format!("Sancho {}", m.units.height(mm)))
                        .size(12.0)
                        .color(kit.p.ink),
                );
                ui.label(RichText::new("|").size(12.0).color(kit.p.line));
            }
            for units in [Units::In, Units::Mm] {
                let on = m.units == units;
                let text = RichText::new(units.key()).size(12.0).strong().color(if on {
                    kit.p.ink
                } else {
                    kit.p.muted
                });
                let hint = match (units, m.document) {
                    (Units::Mm, false) => "New projects measure in millimetres",
                    (Units::In, false) => "New projects measure in inches",
                    (Units::Mm, true) => "This project measures in millimetres",
                    (Units::In, true) => "This project measures in inches",
                };
                if ui
                    .add(
                        egui::Button::new(text)
                            .selected(on)
                            .corner_radius(CornerRadius::same(8))
                            .min_size(vec2(30.0, 20.0)),
                    )
                    .on_hover_text(hint)
                    .clicked()
                {
                    act = Some(Act::SetUnits(units));
                }
            }
        });
    });
    act
}

/// A tab whose environment comes later: what it will do, and when.
pub fn env_page(ui: &mut Ui, kit: &mut Kit<'_>, env: Env) -> Option<Act> {
    let mut act = None;
    let plan = env.plan()?;
    ui.vertical_centered(|ui| {
        ui.add_space((ui.available_height() * 0.18).max(24.0));
        let (rect, _) = ui.allocate_exact_size(vec2(88.0, 88.0), Sense::hover());
        kit.paint(ui, env.icon(), rect, env.name(), false);
        ui.add_space(8.0);
        ui.label(RichText::new(env.name()).size(28.0).strong().color(kit.p.ink));
        ui.add_space(4.0);
        // The chip is painted: a Frame in a centred layout stretches across the window.
        let galley = ui.painter().layout_no_wrap(
            format!("comes in {}", plan.phase),
            FontId::proportional(13.0),
            crate::theme::SANCHO_INK,
        );
        let (chip, _) = ui.allocate_exact_size(galley.size() + vec2(24.0, 8.0), Sense::hover());
        ui.painter()
            .rect_filled(chip, CornerRadius::same(12), STAGE_NOW);
        ui.painter().galley(
            chip.center() - galley.size() / 2.0,
            galley,
            crate::theme::SANCHO_INK,
        );
        ui.add_space(12.0);
        ui.allocate_ui(vec2(520.0, 120.0), |ui| {
            ui.label(RichText::new(plan.does).size(17.0).color(kit.p.ink));
        });
        ui.add_space(16.0);
        ui.label(
            RichText::new(
                "The tab is here so you can see where it fits. Your design is still open in Design.",
            )
            .size(13.0)
            .color(kit.p.muted),
        );
        ui.add_space(12.0);
        if ui.button("Back to Design").clicked() {
            act = Some(Act::Tab(Env::Design));
        }
    });
    act
}

/// Command search's state: open or not, what is typed, which result is chosen.
#[derive(Default)]
pub struct Search {
    /// Shown now.
    pub open: bool,
    query: String,
    chosen: usize,
    just_opened: bool,
}

impl Search {
    /// Opens it, empty.
    pub fn open(&mut self) {
        self.open = true;
        self.query.clear();
        self.chosen = 0;
        self.just_opened = true;
    }

    /// Opens it with `text` already typed (`--search`).
    pub fn open_with(&mut self, text: &str) {
        self.open();
        self.query = text.to_string();
    }
}

/// How many results command search shows.
const RESULTS: usize = 7;

/// Command search (Ctrl+K): type in plain words, choose with the arrows or the mouse, Enter runs.
/// `design_open` says whether commands that work on a design can run. The last row asks Sancho.
pub fn search_window(
    ctx: &Context,
    kit: &mut Kit<'_>,
    search: &mut Search,
    all: &[Command],
    design_open: bool,
) -> Option<Act> {
    if !search.open {
        return None;
    }
    let hits = commands::search(&search.query, all, RESULTS);
    let asks_sancho = !commands::query_words(&search.query).is_empty();
    let rows = hits.len() + usize::from(asks_sancho);
    // The arrows and Enter belong to the list, not the text field.
    let (down, up, enter) = ctx.input_mut(|i| {
        (
            i.consume_key(Modifiers::NONE, Key::ArrowDown),
            i.consume_key(Modifiers::NONE, Key::ArrowUp),
            i.consume_key(Modifiers::NONE, Key::Enter),
        )
    });
    if rows > 0 {
        if down {
            search.chosen = (search.chosen + 1) % rows;
        }
        if up {
            search.chosen = (search.chosen + rows - 1) % rows;
        }
        search.chosen = search.chosen.min(rows - 1);
    }
    let mut run: Option<usize> = enter.then_some(search.chosen);

    let area = egui::Modal::default_area(Id::unique("command-search"))
        .anchor(Align2::CENTER_TOP, vec2(0.0, 84.0));
    let frame = Frame::new()
        .fill(kit.p.panel)
        .stroke(Stroke::new(2.0, kit.p.line))
        .corner_radius(CornerRadius::same(20))
        .inner_margin(Margin::same(14));
    let modal = egui::Modal::new(Id::unique("command-search"))
        .area(area)
        .frame(frame)
        .backdrop_color(Color32::from_black_alpha(60))
        .show(ctx, |ui| {
            ui.set_width(640.0);
            Frame::new()
                .fill(kit.p.bg2)
                .stroke(Stroke::new(2.0, ACCENT))
                .corner_radius(CornerRadius::same(14))
                .inner_margin(Margin::symmetric(12, 8))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let (rect, _) = ui.allocate_exact_size(vec2(24.0, 24.0), Sense::hover());
                        kit.paint(ui, "app.search", rect, "Search", false);
                        let edit = ui.add(
                            egui::TextEdit::singleline(&mut search.query)
                                .hint_text("Search or say what to do, like \"put a hole for a screw\"")
                                .font(FontId::proportional(18.0))
                                .align(Align2::LEFT_CENTER)
                                .frame(Frame::NONE)
                                .desired_width(ui.available_width() - 40.0),
                        );
                        if search.just_opened {
                            edit.request_focus();
                            search.just_opened = false;
                        }
                        if edit.changed() {
                            search.chosen = 0;
                        }
                        ui.label(RichText::new("Esc").size(12.0).color(kit.p.muted));
                    });
                });
            ui.add_space(6.0);
            let title = if commands::query_words(&search.query).is_empty() {
                "Things you can do"
            } else if hits.is_empty() {
                "No command matches; Sancho may know"
            } else {
                "Best matches"
            };
            ui.label(RichText::new(title).size(12.0).strong().color(kit.p.muted));
            for (row, &i) in hits.iter().enumerate() {
                let c = &all[i];
                let blocked = c.needs_design && !design_open;
                let second = if let Act::Later(l) = c.act {
                    format!("{} \u{b7} comes in {}", c.place, l.phase)
                } else if blocked {
                    format!("{} \u{b7} open a design first", c.place)
                } else if c.cad.is_empty() || c.cad == c.name {
                    c.place.to_string()
                } else {
                    format!("{} \u{b7} {}", c.place, c.cad)
                };
                let dim = !c.ready() || blocked;
                if result_row(ui, kit, c.icon, c.name, &second, c.key, row == search.chosen, dim)
                    .clicked()
                {
                    run = Some(row);
                }
            }
            if asks_sancho {
                let row = hits.len();
                let line = format!("Ask Sancho: \"{}\"", search.query.trim());
                if result_row(
                    ui,
                    kit,
                    "app.sancho",
                    &line,
                    "Sancho explains, using your part \u{b7} comes in P2",
                    "",
                    row == search.chosen,
                    true,
                )
                .clicked()
                {
                    run = Some(row);
                }
            }
            ui.add_space(6.0);
            ui.label(
                RichText::new("Up and Down choose \u{b7} Enter runs \u{b7} Esc closes \u{b7} CAD words work too (fillet, extrude, chamfer)")
                    .size(12.0)
                    .color(kit.p.muted),
            );
        });
    let mut act = None;
    if let Some(row) = run {
        if let Some(&i) = hits.get(row) {
            let c = &all[i];
            act = Some(if c.needs_design && !design_open && c.ready() {
                Act::Later(Later {
                    what: c.name,
                    phase: "now",
                    note: "It works on a design: open one first (Examples on the opening screen, or the project name's menu).",
                })
            } else {
                c.act
            });
        } else if asks_sancho {
            act = Some(Act::Later(Later {
                what: "Asking Sancho",
                phase: "P2",
                note: "Sancho's answers about your own part come with his rules (P2).",
            }));
        }
        search.open = false;
    }
    if modal.should_close() {
        search.open = false;
    }
    act
}

/// One result row: icon, name, a second line, a shortcut chip.
#[allow(clippy::too_many_arguments)]
fn result_row(
    ui: &mut Ui,
    kit: &mut Kit<'_>,
    icon: &str,
    name: &str,
    second: &str,
    key: &str,
    chosen: bool,
    dim: bool,
) -> Response {
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), 52.0), Sense::click());
    if chosen {
        ui.painter()
            .rect_filled(rect, CornerRadius::same(12), CHIP_WAITING);
    } else if response.hovered() {
        ui.painter()
            .rect_filled(rect, CornerRadius::same(12), kit.p.chip);
    }
    // The chosen row sits on a warm chip in every look: its text stays dark.
    let (ink, muted) = if chosen {
        (crate::theme::SANCHO_INK, crate::theme::SANCHO_INK)
    } else if dim {
        (kit.p.muted, kit.p.muted)
    } else {
        (kit.p.ink, kit.p.muted)
    };
    kit.paint(
        ui,
        icon,
        Rect::from_center_size(pos2(rect.left() + 28.0, rect.center().y), vec2(30.0, 30.0)),
        name,
        dim && !chosen,
    );
    ui.painter().text(
        pos2(rect.left() + 58.0, rect.top() + 17.0),
        Align2::LEFT_CENTER,
        name,
        FontId::proportional(16.0),
        ink,
    );
    ui.painter().text(
        pos2(rect.left() + 58.0, rect.top() + 36.0),
        Align2::LEFT_CENTER,
        second,
        FontId::proportional(12.5),
        muted,
    );
    if !key.is_empty() {
        let chip =
            Rect::from_center_size(pos2(rect.right() - 40.0, rect.center().y), vec2(58.0, 24.0));
        ui.painter()
            .rect_filled(chip, CornerRadius::same(6), kit.p.panel);
        ui.painter().text(
            chip.center(),
            Align2::CENTER_CENTER,
            key,
            FontId::proportional(12.0),
            LINK,
        );
    }
    response
}
