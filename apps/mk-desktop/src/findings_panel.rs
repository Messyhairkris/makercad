//! Sancho's Findings panel (doc 07 section 7.4): everything the rule packs found, said or not,
//! sorted worst first, grouped by severity, kind or place, each with a plain reason, a "Why?"
//! that opens its derivation (`mk_assist::why`), and Dismiss.
//!
//! The model (sorting, grouping, the badge's words, the derivations) is the Crew's
//! (`mk_assist::panel`, `mk_assist::why`); this file only draws it. Until the window can turn a
//! SolidCraft design into the facts the packs read, the panel is shown with the golden dresser's
//! findings (`mk_assist::golden`), and it says so.

use egui::{vec2, Color32, CornerRadius, Frame, Margin, RichText, ScrollArea, Sense, Stroke, Ui};
use mk_assist::findings::{Finding, Kind, Location, Severity, Source};
use mk_assist::golden;
use mk_assist::panel::{self, Filter, Group, Row};
use mk_assist::why;

use crate::theme::{self, Palette};

/// How wide the panel is.
pub const WIDTH: f32 = 372.0;

const CRITICAL: Color32 = theme::hex(0xE5534B);

/// What the person did in the panel this frame.
enum Act {
    Why(Finding),
    Dismiss(Finding),
    Restore(Finding),
}

/// The Findings panel's state.
pub struct FindingsPanel {
    rows: Vec<Row>,
    group: Group,
    hide_dismissed: bool,
    /// The finding whose derivation is open.
    why_open: Option<Finding>,
    /// Where these findings come from, said at the top.
    note: String,
}

impl FindingsPanel {
    /// A panel for `findings`, all open.
    pub fn new(findings: Vec<Finding>, note: &str) -> Self {
        Self {
            rows: findings
                .into_iter()
                .map(|finding| Row {
                    finding,
                    dismissed: false,
                })
                .collect(),
            group: Group::Severity,
            hide_dismissed: false,
            why_open: None,
            note: note.to_string(),
        }
    }

    /// The golden dresser's findings: with its five seeded faults, or the sound one (nothing).
    pub fn golden(faulted: bool) -> Self {
        let (chest, note) = if faulted {
            (
                golden::faulted(),
                "Shown for the golden dresser with its five seeded faults: a demonstration, not \
                 your design (the window cannot read a design's facts yet).",
            )
        } else {
            (
                golden::sound(),
                "Shown for the golden dresser without its faults: a demonstration, not your \
                 design (the window cannot read a design's facts yet).",
            )
        };
        Self::new(golden::evaluate_all(&chest), note)
    }

    /// How many findings are open (not dismissed).
    pub fn open_count(&self) -> usize {
        panel::badge(&self.rows).open
    }

    /// The badge's words ("5 things to look at, 1 serious").
    pub fn headline(&self) -> String {
        panel::badge_text(&panel::badge(&self.rows))
    }

    /// Opens the "Why?" of the first finding that came from rule `rule` (for screenshots).
    pub fn open_why_for(&mut self, rule: &str) {
        self.why_open = self
            .rows
            .iter()
            .map(|r| &r.finding)
            .find(|f| matches!(&f.source, Source::Rule(id) if id == rule))
            .cloned();
    }

    fn set_dismissed(&mut self, finding: &Finding, dismissed: bool) {
        for row in self.rows.iter_mut().filter(|r| &r.finding == finding) {
            row.dismissed = dismissed;
        }
    }

    /// Draws the panel's contents.
    pub fn show(&mut self, ui: &mut Ui, p: &Palette) {
        let mut act = None;
        ui.add_space(12.0);
        ui.label(
            RichText::new("Sancho's findings")
                .size(18.0)
                .strong()
                .color(p.ink),
        );
        ui.label(RichText::new(self.headline()).size(14.0).color(p.ink));
        ui.add_space(2.0);
        ui.label(RichText::new(&self.note).size(12.0).color(p.muted));
        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("Group by").size(12.0).color(p.muted));
            for (group, name) in [
                (Group::Severity, "How serious"),
                (Group::Kind, "Kind"),
                (Group::Location, "Place"),
            ] {
                ui.selectable_value(&mut self.group, group, name);
            }
        });
        ui.checkbox(&mut self.hide_dismissed, "Hide the ones I dismissed");
        ui.add_space(6.0);
        let shown = panel::filter(
            &self.rows,
            &Filter {
                hide_dismissed: self.hide_dismissed,
                ..Filter::default()
            },
        );
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                if shown.is_empty() {
                    ui.add_space(16.0);
                    ui.label(
                        RichText::new(
                            "Nothing to look at. Sancho has no findings for this design.",
                        )
                        .size(14.0)
                        .color(p.muted),
                    );
                }
                for (label, rows) in panel::group(&shown, self.group) {
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(format!("{label}  ({})", rows.len()))
                            .size(13.0)
                            .strong()
                            .color(p.muted),
                    );
                    for row in &rows {
                        row_ui(ui, p, row, self.why_open.as_ref(), &mut act);
                    }
                }
                ui.add_space(12.0);
            });
        match act {
            Some(Act::Why(f)) => {
                self.why_open = if self.why_open.as_ref() == Some(&f) {
                    None
                } else {
                    Some(f)
                };
            }
            Some(Act::Dismiss(f)) => self.set_dismissed(&f, true),
            Some(Act::Restore(f)) => self.set_dismissed(&f, false),
            None => {}
        }
    }
}

/// The colour that goes with a severity.
fn severity_colour(severity: Severity) -> Color32 {
    match severity {
        Severity::Critical => CRITICAL,
        Severity::Warning => theme::STAGE_NOW,
        Severity::Info => theme::LINK,
    }
}

/// What a kind of finding is called.
fn kind_words(kind: Kind) -> &'static str {
    match kind {
        Kind::Forbidden => "Can't be built",
        Kind::Unsafe => "Unsafe",
        Kind::Denied => "Not allowed",
        Kind::NotSuitable => "Not suitable",
        Kind::Inventory => "Your tools",
        Kind::Fit => "Fit",
        Kind::Schedule => "Schedule",
        Kind::Budget => "Budget",
        Kind::Tip => "Tip",
        Kind::Praise => "Nicely done",
    }
}

/// Where a finding is, in words.
fn place_words(location: &Location) -> String {
    match location {
        Location::Project => "the project".to_string(),
        Location::Entity(id) => id.replace('_', " "),
        Location::Step(n) => format!("step {n}"),
        Location::Field(f) => f.clone(),
    }
}

/// What a fix id means, for the rule packs' fixes.
fn fix_words(id: &str) -> Option<&'static str> {
    Some(match id {
        "part.thickness" => "Make the part thicker, or give it less to carry.",
        "fastener.length" => "Use a different length of screw.",
        "fastener.move" => "Move the fastener further from the edge.",
        "part.attach" => "Fasten the part to something.",
        "slot.shorten" => "Make the slot shorter.",
        "inventory.add_tool" => "Add a tool that can do it to your workshop.",
        "plan.move_step" => "Move the step out of your quiet hours.",
        "step.change_material" => "Pick another material for the step.",
        "space.ventilate" => "Ventilate the space, or work outside.",
        "plan.add_electrician" => "Plan for an electrician to do or check the wiring.",
        "part.add_anchor" => "Anchor it to a wall.",
        "part.change_anchor" => "Use a stronger anchor.",
        "part.confirm_wall" => "Find out what the wall is made of.",
        _ => return None,
    })
}

fn row_ui(ui: &mut Ui, p: &Palette, row: &Row, open: Option<&Finding>, act: &mut Option<Act>) {
    let f = &row.finding;
    let text = if row.dismissed { p.muted } else { p.ink };
    Frame::new()
        .fill(p.panel)
        .stroke(Stroke::new(1.5, p.line))
        .corner_radius(CornerRadius::same(12))
        .inner_margin(Margin::symmetric(12, 10))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                let (dot, _) = ui.allocate_exact_size(vec2(12.0, 12.0), Sense::hover());
                ui.painter()
                    .circle_filled(dot.center(), 5.5, severity_colour(f.severity));
                ui.label(RichText::new(&f.what).size(14.0).strong().color(text));
            });
            ui.label(
                RichText::new(format!(
                    "{} \u{b7} {}{}",
                    kind_words(f.kind),
                    place_words(&f.location),
                    if row.dismissed {
                        " \u{b7} dismissed"
                    } else {
                        ""
                    }
                ))
                .size(11.5)
                .color(p.muted),
            );
            ui.add_space(2.0);
            ui.label(RichText::new(&f.why).size(13.0).color(text));
            if let Some(words) = f.fix.as_deref().and_then(fix_words) {
                ui.label(RichText::new(words).size(13.0).italics().color(p.muted));
            }
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                let is_open = open == Some(f);
                if ui
                    .selectable_label(is_open, if is_open { "Hide why" } else { "Why?" })
                    .clicked()
                {
                    *act = Some(Act::Why(f.clone()));
                }
                let (label, make): (&str, fn(Finding) -> Act) = if row.dismissed {
                    ("Bring it back", Act::Restore)
                } else {
                    ("Dismiss", Act::Dismiss)
                };
                if ui.button(label).clicked() {
                    *act = Some(make(f.clone()));
                }
            });
            if open == Some(f) {
                derivation_ui(ui, p, f);
            }
        });
    ui.add_space(6.0);
}

/// The open "Why?": what Sancho looked at, the rule, and where its numbers come from.
fn derivation_ui(ui: &mut Ui, p: &Palette, finding: &Finding) {
    let d = why::derive(finding);
    Frame::new()
        .fill(p.bg2)
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::symmetric(10, 8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new(&d.title).size(13.0).strong().color(p.ink));
            ui.add_space(2.0);
            egui::Grid::new(("why", finding.what.as_str()))
                .num_columns(2)
                .spacing(vec2(10.0, 2.0))
                .show(ui, |ui| {
                    for (label, value) in &d.inputs {
                        ui.label(RichText::new(label).size(12.0).color(p.muted));
                        ui.label(RichText::new(value).size(12.0).color(p.ink));
                        ui.end_row();
                    }
                });
            ui.add_space(4.0);
            ui.label(RichText::new(&d.rule).size(12.0).color(p.ink));
            if let Some(source) = &d.source {
                ui.add_space(2.0);
                ui.label(
                    RichText::new(format!("Numbers from: {source}"))
                        .size(11.5)
                        .italics()
                        .color(p.muted),
                );
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_faulted_dresser_has_five_findings_and_the_sound_one_none() {
        let faulted = FindingsPanel::golden(true);
        assert_eq!(faulted.open_count(), 5);
        assert!(faulted.headline().starts_with("5 things to look at"));
        let sound = FindingsPanel::golden(false);
        assert_eq!(sound.open_count(), 0);
        assert_eq!(sound.headline(), "Nothing to look at");
    }

    #[test]
    fn dismissing_closes_a_finding_and_bringing_it_back_opens_it_again() {
        let mut panel = FindingsPanel::golden(true);
        let first = panel.rows[0].finding.clone();
        panel.set_dismissed(&first, true);
        assert_eq!(panel.open_count(), 4);
        panel.set_dismissed(&first, false);
        assert_eq!(panel.open_count(), 5);
    }

    #[test]
    fn every_fix_the_packs_give_has_words() {
        for row in &FindingsPanel::golden(true).rows {
            if let Some(fix) = row.finding.fix.as_deref() {
                assert!(fix_words(fix).is_some(), "no words for the fix {fix}");
            }
        }
    }

    #[test]
    fn every_finding_can_say_why() {
        for row in &FindingsPanel::golden(true).rows {
            let d = why::derive(&row.finding);
            assert!(!d.title.is_empty() && !d.rule.is_empty());
            assert!(!d.inputs.is_empty());
        }
    }

    #[test]
    fn a_why_opens_for_the_rule_asked_for_and_only_when_it_found_something() {
        let mut panel = FindingsPanel::golden(true);
        panel.open_why_for("structure.sag");
        let open = panel.why_open.clone().expect("the sag finding is there");
        assert_eq!(open.source, Source::Rule("structure.sag".to_string()));
        panel.open_why_for("safety.laser_never");
        assert!(panel.why_open.is_none());
    }

    #[test]
    fn places_read_as_words() {
        assert_eq!(
            place_words(&Location::Entity("carcase_top".into())),
            "carcase top"
        );
        assert_eq!(place_words(&Location::Step(4)), "step 4");
        assert_eq!(place_words(&Location::Project), "the project");
    }
}
