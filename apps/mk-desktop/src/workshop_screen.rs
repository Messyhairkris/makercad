//! "My workshop" (doc 04 section 4.3, decision D30): the walkthrough that keeps the **global**
//! tools and materials up to date (tools, then materials, then what changed), and the
//! end-of-project offer that hands a finished project's leftover **local** materials to it.
//!
//! Global items live in the workshop library (`mk_inventory::library`, the SQLite file); local
//! ones belong to one project and live only in its `project/inventory.json`, where each entry says
//! `"scope": "global"` or `"scope": "local"`. Nothing a person types into a project reaches the
//! workshop on its own: the only ways in are this walkthrough and the offer at the end.

use egui::{vec2, Align, Frame, Layout, Margin, RichText, ScrollArea, Stroke, Ui};
use mk_inventory::library::{Library, LibraryError, Material, Tool};
use mk_inventory::quickadd::quick_add;
use mk_inventory::taxonomy;
use mk_inventory::taxonomy_data::Access;
use serde_json::Value;

use crate::quest_screen::{
    accent_button, add_box, card, column, describe_item, muted, plain_button, scope_word,
};
use crate::theme::{Palette, ACCENT, SANCHO_INK, STAGE_DONE};

/// The walkthrough's steps.
pub const STEPS: [&str; 3] = ["Your tools", "Your materials", "All set"];

/// What the person did on the walkthrough's screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkshopAction {
    /// "Done": leave the walkthrough.
    Close,
}

/// The access words, as a person says them.
pub fn access_words(access: Access) -> &'static str {
    match access {
        Access::Owned => "I own it",
        Access::Shared => "Shared with me",
        Access::Makerspace => "At a makerspace",
        Access::Rentable => "I can rent it",
        Access::Wishlist => "Wishlist",
    }
}

const ACCESS_ALL: [Access; 5] = [
    Access::Owned,
    Access::Shared,
    Access::Makerspace,
    Access::Rentable,
    Access::Wishlist,
];

/// The walkthrough in progress: the open library, and what has been typed.
pub struct WorkshopEdit {
    library: Library,
    /// The step in front (an index into [`STEPS`]).
    pub step: usize,
    tools: Vec<Tool>,
    materials: Vec<Material>,
    pub tool_text: String,
    pub material_text: String,
    /// The last thing said about an entry.
    pub hint: Option<String>,
    started_with: (usize, usize),
}

impl WorkshopEdit {
    /// Opens the library file at `path` (made, empty, when there is none).
    pub fn open(path: &std::path::Path) -> Result<Self, LibraryError> {
        if let Some(folder) = path.parent() {
            let _ = std::fs::create_dir_all(folder);
        }
        Self::with_library(Library::open(path)?)
    }

    /// The walkthrough over an open library.
    pub fn with_library(library: Library) -> Result<Self, LibraryError> {
        let mut edit = Self {
            library,
            step: 0,
            tools: Vec::new(),
            materials: Vec::new(),
            tool_text: String::new(),
            material_text: String::new(),
            hint: None,
            started_with: (0, 0),
        };
        edit.reload()?;
        edit.started_with = (edit.tools.len(), edit.materials.len());
        Ok(edit)
    }

    fn reload(&mut self) -> Result<(), LibraryError> {
        self.tools = self.library.tools()?;
        self.materials = self.library.materials()?;
        Ok(())
    }

    fn said(&mut self, text: String) {
        self.hint = Some(text);
    }

    fn failed(&mut self, e: LibraryError) {
        self.hint = Some(format!("Not saved: {e}"));
    }

    /// The tools, oldest first.
    pub fn tools(&self) -> &[Tool] {
        &self.tools
    }

    /// The material lines, oldest first.
    pub fn materials(&self) -> &[Material] {
        &self.materials
    }

    /// Adds the tool the typed words name (unless the workshop has that kind already).
    pub fn add_tool_words(&mut self) {
        let text = self.tool_text.trim().to_string();
        if text.is_empty() {
            return;
        }
        let Some(tool) = taxonomy::tool_by_words(&text) else {
            self.said(format!(
                "I don't know \u{201c}{text}\u{201d} yet. Try another word for it, or a kit."
            ));
            return;
        };
        if self.tools.iter().any(|t| t.kind == tool.id) {
            self.said(format!("You already have {}.", tool.name));
            self.tool_text.clear();
            return;
        }
        match self.library.add_tool(tool.id, "", Access::Owned, None) {
            Ok(_) => {
                self.said(format!("Added {}.", tool.name));
                self.tool_text.clear();
                if let Err(e) = self.reload() {
                    self.failed(e);
                }
            }
            Err(e) => self.failed(e),
        }
    }

    /// Adds every tool of a kit the workshop doesn't have yet.
    pub fn add_kit(&mut self, kit_id: &str) {
        let name = taxonomy::kit(kit_id).map_or(kit_id, |k| k.name);
        match self.library.add_kit(kit_id) {
            Ok(0) => self.said(format!("{name}: you already have all of these.")),
            Ok(n) => self.said(format!(
                "{name}: {n} tool{} added. Change any you don't really have.",
                if n == 1 { "" } else { "s" }
            )),
            Err(e) => self.failed(e),
        }
        if let Err(e) = self.reload() {
            self.failed(e);
        }
    }

    /// Changes how the person has a tool.
    pub fn set_access(&mut self, id: i64, access: Access) {
        if let Err(e) = self
            .library
            .set_tool_access(id, access)
            .and_then(|_| self.reload())
        {
            self.failed(e);
        }
    }

    /// Takes a tool out of the workshop.
    pub fn remove_tool(&mut self, id: i64) {
        if let Err(e) = self.library.remove_tool(id).and_then(|_| self.reload()) {
            self.failed(e);
        }
    }

    /// Reads the typed line with quick-add and keeps it.
    pub fn add_material(&mut self) {
        let text = self.material_text.trim().to_string();
        if text.is_empty() {
            return;
        }
        match self.library.add_material(&text) {
            Ok(m) => {
                self.said(format!(
                    "\u{201c}{text}\u{201d}: {}.",
                    describe_item(&m.item)
                ));
                self.material_text.clear();
                if let Err(e) = self.reload() {
                    self.failed(e);
                }
            }
            Err(e) => self.failed(e),
        }
    }

    /// Changes how many of a material line there are (never below zero).
    pub fn change_qty(&mut self, id: i64, delta: i64) {
        let Some(now) = self
            .materials
            .iter()
            .find(|m| m.id == id)
            .map(|m| m.item.qty)
        else {
            return;
        };
        let next = (i64::from(now) + delta).clamp(0, i64::from(u32::MAX)) as u32;
        if let Err(e) = self.library.set_qty(id, next).and_then(|_| self.reload()) {
            self.failed(e);
        }
    }

    /// Marks a material line as on hand or used up.
    pub fn set_on_hand(&mut self, id: i64, on_hand: bool) {
        if let Err(e) = self
            .library
            .set_on_hand(id, on_hand)
            .and_then(|_| self.reload())
        {
            self.failed(e);
        }
    }

    /// Takes a material line out of the workshop.
    pub fn remove_material(&mut self, id: i64) {
        if let Err(e) = self.library.remove_material(id).and_then(|_| self.reload()) {
            self.failed(e);
        }
    }

    /// The last step's words: what the workshop holds, and what changed on this visit.
    pub fn summary(&self) -> String {
        let (tools, materials) = (self.tools.len(), self.materials.len());
        let on_hand = self
            .materials
            .iter()
            .filter(|m| m.item.qty > 0 && m.on_hand)
            .count();
        let owned = self
            .tools
            .iter()
            .filter(|t| matches!(t.access, Access::Owned | Access::Shared))
            .count();
        let change = |now: usize, then: usize, what: &str| {
            let words = |n: usize| {
                if n == 1 {
                    what.to_string()
                } else {
                    format!("{what}s")
                }
            };
            match now.cmp(&then) {
                std::cmp::Ordering::Greater => format!(
                    ", {} more {} than when you opened this",
                    now - then,
                    words(now - then)
                ),
                std::cmp::Ordering::Less => format!(
                    ", {} fewer {} than when you opened this",
                    then - now,
                    words(then - now)
                ),
                std::cmp::Ordering::Equal => String::new(),
            }
        };
        format!(
            "Your workshop has {tools} tools ({owned} you have at hand){} and {materials} material \
             lines ({on_hand} on hand){}.",
            change(tools, self.started_with.0, "tool"),
            change(materials, self.started_with.1, "material line"),
        )
    }

    /// Goes to a step (an out-of-range one is the last).
    pub fn go_to(&mut self, step: usize) {
        self.step = step.min(STEPS.len() - 1);
    }
}

/// How many tools and material lines the library file at `path` holds (None when it cannot be
/// read).
pub fn counts(path: &std::path::Path) -> Option<(usize, usize)> {
    let library = Library::open(path).ok()?;
    Some((library.tools().ok()?.len(), library.materials().ok()?.len()))
}

// ---- the end-of-project offer ----

/// One local material a finished project can hand to the workshop.
#[derive(Clone, Debug, PartialEq)]
pub struct LeftOver {
    /// The words it was typed in ("half sheet 3/4 ply").
    pub said: String,
    /// How many the project listed.
    pub qty: u32,
    /// How many are left (the person says).
    pub left: u32,
    /// Whether to add it to the workshop.
    pub keep: bool,
}

/// The local materials in a project's `inventory.json`. An entry with no scope (a project made
/// before D30) counts as local: it is only offered, never added without a yes.
pub fn local_materials(inventory: &[u8]) -> Vec<LeftOver> {
    let Ok(value) = serde_json::from_slice::<Value>(inventory) else {
        return Vec::new();
    };
    value["materials"]
        .as_array()
        .map(|list| {
            list.iter()
                .filter(|m| m["scope"].as_str().unwrap_or("local") == "local")
                .filter_map(|m| {
                    let said = m["said"].as_str()?.trim().to_string();
                    if said.is_empty() {
                        return None;
                    }
                    let qty = m["qty"].as_u64().unwrap_or(1).min(u64::from(u32::MAX)) as u32;
                    Some(LeftOver {
                        said,
                        qty,
                        left: qty,
                        keep: false,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Adds the chosen leftovers to the workshop, each with the number left. Returns the words of
/// the ones added.
pub fn add_leftovers(library: &Library, items: &[LeftOver]) -> Result<Vec<String>, LibraryError> {
    let mut added = Vec::new();
    for item in items.iter().filter(|i| i.keep && i.left > 0) {
        let material = library.add_material(&item.said)?;
        library.set_qty(material.id, item.left)?;
        added.push(item.said.clone());
    }
    Ok(added)
}

/// `inventory.json` with the material lines in `said` now global (they went to the workshop).
/// Bytes that are not an inventory come back unchanged.
pub fn mark_global(inventory: &[u8], said: &[String]) -> Vec<u8> {
    let Ok(mut value) = serde_json::from_slice::<Value>(inventory) else {
        return inventory.to_vec();
    };
    if let Some(list) = value["materials"].as_array_mut() {
        for m in list.iter_mut() {
            let words = m["said"].as_str().unwrap_or("").trim().to_string();
            if said.contains(&words) {
                m["scope"] = Value::String(scope_word(true).to_string());
            }
        }
    }
    serde_json::to_vec_pretty(&value).unwrap_or_else(|_| inventory.to_vec())
}

/// What the person chose in the end-of-project window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FinishAction {
    /// Add the ticked ones to the workshop.
    Add,
    /// Not now: leave the workshop as it is.
    NotNow,
}

/// The end-of-project window's contents.
pub fn finish_ui(ui: &mut Ui, items: &mut [LeftOver], p: &Palette) -> Option<FinishAction> {
    let mut action = None;
    ui.label(
        RichText::new("The project is finished. Anything left over?")
            .size(18.0)
            .strong()
            .color(p.ink),
    );
    muted(
        ui,
        p,
        "These materials were only for this project. Tick the ones with some left, say how many, \
         and they join your workshop for every project.",
    );
    ui.add_space(8.0);
    ScrollArea::vertical().max_height(340.0).show(ui, |ui| {
        for item in items.iter_mut() {
            ui.horizontal(|ui| {
                ui.checkbox(&mut item.keep, "");
                ui.vertical(|ui| {
                    ui.set_width(420.0);
                    ui.label(
                        RichText::new(describe_item(&quick_add(&item.said)))
                            .size(15.0)
                            .strong()
                            .color(p.ink),
                    );
                    ui.label(
                        RichText::new(format!("from \u{201c}{}\u{201d}", item.said))
                            .size(13.0)
                            .color(p.muted),
                    );
                });
                ui.label(RichText::new("left:").size(13.0).color(p.muted));
                ui.add_enabled(
                    item.keep,
                    egui::DragValue::new(&mut item.left).range(0..=9999),
                );
            });
        }
    });
    ui.add_space(10.0);
    let chosen = items.iter().filter(|i| i.keep && i.left > 0).count();
    ui.horizontal(|ui| {
        let label = match chosen {
            0 => "Add to my workshop".to_string(),
            n => format!("Add {n} to my workshop"),
        };
        if ui.add_enabled(chosen > 0, accent_button(&label)).clicked() {
            action = Some(FinishAction::Add);
        }
        if ui.add(plain_button(p, "Not now")).clicked() {
            action = Some(FinishAction::NotNow);
        }
    });
    action
}

// ---- the walkthrough's screen ----

/// Draws the walkthrough into `ui`.
pub fn show(ui: &mut Ui, edit: &mut WorkshopEdit, p: &Palette) -> Option<WorkshopAction> {
    crate::home::paint_background(ui, p);
    let mut action = None;
    Frame::new()
        .inner_margin(Margin::symmetric(32, 18))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("My workshop")
                        .size(24.0)
                        .strong()
                        .color(p.ink),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.add(accent_button("Done")).clicked() {
                        action = Some(WorkshopAction::Close);
                    }
                });
            });
        });
    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            Frame::new()
                .inner_margin(Margin {
                    left: 32,
                    right: 32,
                    top: 4,
                    bottom: 32,
                })
                .show(ui, |ui| {
                    let width = ui.available_width().min(1100.0);
                    ui.horizontal_top(|ui| {
                        column(ui, 200.0, |ui| rail(ui, edit, p));
                        ui.add_space(24.0);
                        column(ui, width - 224.0, |ui| {
                            card(ui, p, |ui| match edit.step {
                                0 => tools_step(ui, edit, p),
                                1 => materials_step(ui, edit, p),
                                _ => {
                                    ui.label(
                                        RichText::new("All set").size(24.0).strong().color(p.ink),
                                    );
                                    ui.add_space(6.0);
                                    ui.label(RichText::new(edit.summary()).size(16.0).color(p.ink));
                                    muted(
                                        ui,
                                        p,
                                        "Every project can use these. A project's own materials \
                                         stay with it; when it is finished you can add what is \
                                         left to this workshop.",
                                    );
                                    ui.add_space(10.0);
                                    if ui.add(accent_button("Done")).clicked() {
                                        action = Some(WorkshopAction::Close);
                                    }
                                }
                            });
                            nav(ui, edit, p);
                        });
                    });
                });
        });
    action
}

fn rail(ui: &mut Ui, edit: &mut WorkshopEdit, p: &Palette) {
    let mut go = None;
    for (i, name) in STEPS.iter().enumerate() {
        let here = i == edit.step;
        let fill = if here {
            ACCENT
        } else if i < edit.step {
            STAGE_DONE
        } else {
            p.panel
        };
        let row = ui.horizontal(|ui| {
            let (dot, _) = ui.allocate_exact_size(vec2(28.0, 28.0), egui::Sense::hover());
            ui.painter().circle(
                dot.center(),
                13.0,
                fill,
                Stroke::new(2.0, if here { SANCHO_INK } else { p.line }),
            );
            ui.painter().text(
                dot.center(),
                egui::Align2::CENTER_CENTER,
                (i + 1).to_string(),
                egui::FontId::proportional(14.0),
                if here || i < edit.step {
                    SANCHO_INK
                } else {
                    p.muted
                },
            );
            let text = RichText::new(*name)
                .size(15.0)
                .color(if here { p.ink } else { p.muted });
            ui.add(egui::Label::new(if here { text.strong() } else { text }).selectable(false));
        });
        if ui
            .interact(
                row.response.rect,
                ui.scope_id().with(("workshop-step", i)),
                egui::Sense::click(),
            )
            .clicked()
        {
            go = Some(i);
        }
        ui.add_space(6.0);
    }
    if let Some(i) = go {
        edit.go_to(i);
    }
}

fn nav(ui: &mut Ui, edit: &mut WorkshopEdit, p: &Palette) {
    ui.add_space(12.0);
    ui.horizontal(|ui| {
        if edit.step > 0 && ui.add(plain_button(p, "Back")).clicked() {
            edit.go_to(edit.step - 1);
        }
        if edit.step + 1 < STEPS.len() {
            let label = format!("Next: {}", STEPS[edit.step + 1].to_lowercase());
            if ui.add(accent_button(&label)).clicked() {
                edit.go_to(edit.step + 1);
            }
        }
    });
}

fn tools_step(ui: &mut Ui, edit: &mut WorkshopEdit, p: &Palette) {
    ui.label(
        RichText::new("What tools do you have?")
            .size(24.0)
            .strong()
            .color(p.ink),
    );
    muted(
        ui,
        p,
        "These are the tools every project can use. Start from a kit, or type a tool's name; \
         change how you have each one, or take it out.",
    );
    ui.horizontal_wrapped(|ui| {
        for kit in taxonomy::kits() {
            if ui.add(plain_button(p, kit.name)).clicked() {
                edit.add_kit(kit.id);
            }
        }
    });
    ui.add_space(6.0);
    if add_box(
        ui,
        p,
        &mut edit.tool_text,
        "jigsaw, cordless drill, sewing machine",
    ) {
        edit.add_tool_words();
    }
    if let Some(hint) = edit.hint.clone() {
        muted(ui, p, &hint);
    }
    ui.add_space(6.0);
    let mut change = None;
    let mut remove = None;
    // The page itself scrolls: an inner scroll area here would be given almost no height.
    for tool in edit.tools() {
        ui.horizontal(|ui| {
            let name = taxonomy::tool(&tool.kind).map_or(tool.label.as_str(), |t| t.name);
            ui.add_sized(
                vec2(210.0, 24.0),
                egui::Label::new(RichText::new(name).size(15.0).color(p.ink)),
            );
            let mut access = tool.access;
            egui::ComboBox::from_id_salt(("access", tool.id))
                .selected_text(access_words(access))
                .show_ui(ui, |ui| {
                    for a in ACCESS_ALL {
                        ui.selectable_value(&mut access, a, access_words(a));
                    }
                });
            if access != tool.access {
                change = Some((tool.id, access));
            }
            if ui
                .small_button("\u{d7}")
                .on_hover_text("Take it out")
                .clicked()
            {
                remove = Some(tool.id);
            }
        });
    }
    if let Some((id, access)) = change {
        edit.set_access(id, access);
    }
    if let Some(id) = remove {
        edit.remove_tool(id);
    }
    if edit.tools().is_empty() {
        muted(ui, p, "No tools yet.");
    } else {
        muted(ui, p, &format!("{} tools.", edit.tools().len()));
    }
}

fn materials_step(ui: &mut Ui, edit: &mut WorkshopEdit, p: &Palette) {
    ui.label(
        RichText::new("What materials do you have?")
            .size(24.0)
            .strong()
            .color(p.ink),
    );
    muted(
        ui,
        p,
        "Type them the way you'd say them. Change how many you have, mark one used up, or take it \
         out. A material bought for one project goes in that project, not here.",
    );
    ui.add_space(4.0);
    if add_box(ui, p, &mut edit.material_text, "15 pine 2x4s, 3 foot") {
        edit.add_material();
    }
    if let Some(hint) = edit.hint.clone() {
        muted(ui, p, &hint);
    }
    ui.add_space(6.0);
    let mut qty = None;
    let mut hand = None;
    let mut remove = None;
    for m in edit.materials() {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_width(430.0);
                ui.label(
                    RichText::new(describe_item(&m.item))
                        .size(15.0)
                        .strong()
                        .color(p.ink),
                );
                ui.label(
                    RichText::new(format!("from \u{201c}{}\u{201d}", m.item.text))
                        .size(13.0)
                        .color(p.muted),
                );
            });
            if ui
                .small_button("\u{2212}")
                .on_hover_text("One fewer")
                .clicked()
            {
                qty = Some((m.id, -1));
            }
            ui.label(
                RichText::new(m.item.qty.to_string())
                    .size(14.0)
                    .color(p.ink),
            );
            if ui.small_button("+").on_hover_text("One more").clicked() {
                qty = Some((m.id, 1));
            }
            let mut on_hand = m.on_hand;
            if ui.checkbox(&mut on_hand, "on hand").changed() {
                hand = Some((m.id, on_hand));
            }
            if ui
                .small_button("\u{d7}")
                .on_hover_text("Take it out")
                .clicked()
            {
                remove = Some(m.id);
            }
        });
    }
    if let Some((id, delta)) = qty {
        edit.change_qty(id, delta);
    }
    if let Some((id, on_hand)) = hand {
        edit.set_on_hand(id, on_hand);
    }
    if let Some(id) = remove {
        edit.remove_material(id);
    }
    if edit.materials().is_empty() {
        muted(ui, p, "No materials yet.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mk_inventory::library::Library;

    fn edit() -> WorkshopEdit {
        WorkshopEdit::with_library(Library::in_memory().unwrap()).unwrap()
    }

    #[test]
    fn tools_come_from_words_and_kits_once_each() {
        let mut e = edit();
        e.tool_text = "a jigsaw".into();
        e.add_tool_words();
        assert_eq!(e.tools().len(), 1, "{:?}", e.hint);
        e.tool_text = "jigsaws".into();
        e.add_tool_words();
        assert_eq!(e.tools().len(), 1, "the same kind is not added twice");
        assert!(e.hint.as_deref().is_some_and(|h| h.contains("already")));
        e.tool_text = "a flux capacitor".into();
        e.add_tool_words();
        assert_eq!(e.tools().len(), 1);
        assert!(e.hint.as_deref().is_some_and(|h| h.contains("don't know")));
        e.add_kit("basic_household");
        let n = e.tools().len();
        assert!(n > 1);
        e.add_kit("basic_household");
        assert_eq!(e.tools().len(), n);
        assert!(e
            .hint
            .as_deref()
            .is_some_and(|h| h.contains("already have all")));
    }

    #[test]
    fn a_tool_can_be_changed_to_a_wishlist_item_and_taken_out() {
        let mut e = edit();
        e.tool_text = "jigsaw".into();
        e.add_tool_words();
        let id = e.tools()[0].id;
        assert_eq!(e.tools()[0].access, Access::Owned);
        e.set_access(id, Access::Wishlist);
        assert_eq!(e.tools()[0].access, Access::Wishlist);
        e.remove_tool(id);
        assert!(e.tools().is_empty());
    }

    #[test]
    fn materials_are_read_counted_marked_used_up_and_taken_out() {
        let mut e = edit();
        e.material_text = "15 pine 2x4s, 3 foot".into();
        e.add_material();
        assert_eq!(e.materials().len(), 1);
        let id = e.materials()[0].id;
        assert_eq!(e.materials()[0].item.qty, 15);
        e.change_qty(id, -1);
        assert_eq!(e.materials()[0].item.qty, 14);
        e.change_qty(id, -100);
        assert_eq!(e.materials()[0].item.qty, 0, "never below zero");
        e.change_qty(id, 3);
        assert_eq!(e.materials()[0].item.qty, 3);
        e.set_on_hand(id, false);
        assert!(!e.materials()[0].on_hand);
        e.remove_material(id);
        assert!(e.materials().is_empty());
        e.material_text = "   ".into();
        e.add_material();
        assert!(e.materials().is_empty(), "a blank line adds nothing");
    }

    #[test]
    fn the_last_step_says_what_changed_on_this_visit() {
        let mut e = edit();
        assert!(e.summary().contains("0 tools"));
        e.add_kit("basic_household");
        e.material_text = "half sheet 3/4 ply".into();
        e.add_material();
        let said = e.summary();
        assert!(
            said.contains("more tools than when you opened this"),
            "{said}"
        );
        assert!(
            said.contains("1 more material line than when you opened this"),
            "{said}"
        );
    }

    fn inventory(materials: &[(&str, u32, Option<&str>)]) -> Vec<u8> {
        let list: Vec<Value> = materials
            .iter()
            .map(|(said, qty, scope)| {
                let mut m = serde_json::json!({ "said": said, "qty": qty });
                if let Some(scope) = scope {
                    m["scope"] = Value::String(scope.to_string());
                }
                m
            })
            .collect();
        serde_json::to_vec(&serde_json::json!({ "version": 2, "tools": [], "materials": list }))
            .unwrap()
    }

    #[test]
    fn only_the_projects_own_materials_are_offered_when_it_is_finished() {
        let bytes = inventory(&[
            ("15 pine 2x4s, 3 foot", 15, Some("global")),
            ("2 yards 500D Cordura", 2, Some("local")),
            ("an old bookshelf", 1, None),
        ]);
        let offered = local_materials(&bytes);
        let said: Vec<&str> = offered.iter().map(|o| o.said.as_str()).collect();
        assert_eq!(said, vec!["2 yards 500D Cordura", "an old bookshelf"]);
        assert!(offered.iter().all(|o| !o.keep && o.left == o.qty));
        assert!(local_materials(b"not json").is_empty());
        assert!(local_materials(b"{}").is_empty());
    }

    #[test]
    fn the_leftovers_that_are_ticked_join_the_workshop_with_the_number_left() {
        let library = Library::in_memory().unwrap();
        let items = vec![
            LeftOver {
                said: "2 yards 500D Cordura".into(),
                qty: 2,
                left: 1,
                keep: true,
            },
            LeftOver {
                said: "half sheet 3/4 ply".into(),
                qty: 1,
                left: 1,
                keep: false,
            },
            LeftOver {
                said: "6 pine 1x4s, 4 foot".into(),
                qty: 6,
                left: 0,
                keep: true,
            },
        ];
        let added = add_leftovers(&library, &items).unwrap();
        assert_eq!(added, vec!["2 yards 500D Cordura".to_string()]);
        let kept = library.materials().unwrap();
        assert_eq!(kept.len(), 1, "unticked and none-left ones are not added");
        assert_eq!(
            kept[0].item.qty, 1,
            "the number left, not the number bought"
        );
    }

    #[test]
    fn what_went_to_the_workshop_is_marked_global_in_the_snapshot_and_not_offered_again() {
        let bytes = inventory(&[
            ("2 yards 500D Cordura", 2, Some("local")),
            ("an old bookshelf", 1, Some("local")),
        ]);
        let after = mark_global(&bytes, &["2 yards 500D Cordura".to_string()]);
        let offered = local_materials(&after);
        assert_eq!(offered.len(), 1);
        assert_eq!(offered[0].said, "an old bookshelf");
        assert_eq!(
            mark_global(b"not json", &["x".to_string()]),
            b"not json".to_vec()
        );
    }

    #[test]
    fn a_workshop_can_be_opened_from_a_file_and_kept() {
        let dir =
            std::env::temp_dir().join(format!("makercad-workshop-edit-{}", std::process::id()));
        let path = dir.join("library.db");
        {
            let mut e = WorkshopEdit::open(&path).unwrap();
            e.tool_text = "jigsaw".into();
            e.add_tool_words();
        }
        let again = WorkshopEdit::open(&path).unwrap();
        assert_eq!(again.tools().len(), 1);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
