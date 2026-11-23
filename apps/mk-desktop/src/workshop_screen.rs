//! "My workshop" (doc 04 section 4.3, decision D30): the walkthrough that keeps the **global**
//! tools and materials up to date (tools, then materials, then what changed), and the
//! end-of-project offer that hands a finished project's leftover **local** materials to it.
//!
//! Global items live in the workshop library (`mk_inventory::library`, the SQLite file); local
//! ones belong to one project and live only in its `project/inventory.json`, where each entry says
//! `"scope": "global"` or `"scope": "local"`. Nothing a person types into a project reaches the
//! workshop on its own: the only ways in are this walkthrough and the offer at the end, which
//! covers both materials and tools.
//!
//! A tool can also be there only some days (D31: "a multimeter on weekends"): each tool has a
//! "when" that this walkthrough, the Quest and the end-of-project offer all show and change
//! through the same small form ([`WhenEditor`]).

use egui::{vec2, Align, Align2, Frame, Layout, Margin, RichText, ScrollArea, Stroke, Ui};
use mk_inventory::availability::{Availability, Day};
use mk_inventory::library::{Library, LibraryError, Material, Tool};
use mk_inventory::quickadd::quick_add;
use mk_inventory::taxonomy;
use mk_inventory::taxonomy_data::Access;
use serde_json::{json, Value};

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

// ---- when a tool can be used (D31) ----

/// The "when can you use it?" form while it is open: the days, and the note as it is being
/// typed (kept as typed, so a space between words is not lost on the way).
#[derive(Clone, Debug, PartialEq)]
pub struct WhenEditor {
    days: Availability,
    pub note: String,
    /// What to tell the person about the last click.
    pub hint: Option<String>,
}

impl WhenEditor {
    /// The form, opened on how things are now.
    pub fn new(now: &Availability) -> Self {
        Self {
            days: now.clone(),
            note: now.note().to_string(),
            hint: None,
        }
    }

    /// What the form says: the days, with the note as typed.
    pub fn result(&self) -> Availability {
        self.days.clone().with_note(&self.note)
    }

    /// Whether a day is ticked.
    pub fn covers(&self, day: Day) -> bool {
        self.days.covers(day)
    }

    /// Ticks or unticks a day. The last day cannot be unticked: a tool with no day is one the
    /// person does not have, which is what the wishlist is for.
    pub fn toggle(&mut self, day: Day) {
        self.hint = if self.days.toggle(day) {
            None
        } else {
            Some(
                "A tool needs at least one day. If you don't have it, put it on the wishlist \
                 instead."
                    .into(),
            )
        };
    }

    /// Replaces the days with a ready-made set ("Weekends"); the note stays.
    pub fn preset(&mut self, days: Availability) {
        self.days = days;
        self.hint = None;
    }
}

/// What the person did in the form.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WhenAction {
    Save,
    Cancel,
    /// "Take it out of this project" (only when the form offers it).
    Remove,
}

/// The "when can you use it?" window: ready-made sets, a button for each day, a note, and what
/// it adds up to. `can_remove` adds a button that takes the tool out of the project the form
/// was opened from.
pub fn when_window(
    ctx: &egui::Context,
    title: &str,
    editor: &mut WhenEditor,
    can_remove: bool,
    p: &Palette,
) -> Option<WhenAction> {
    let mut action = None;
    egui::Window::new(title)
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.set_width(440.0);
            muted(
                ui,
                p,
                "Some tools you only have on certain days: a friend's, a makerspace's open days. \
                 Pick the days, and add anything else that matters.",
            );
            ui.horizontal_wrapped(|ui| {
                for (label, set) in [
                    ("Any day", Availability::always()),
                    ("Weekends", Availability::weekends()),
                    ("Weekdays", Availability::weekdays()),
                ] {
                    if ui.add(plain_button(p, label)).clicked() {
                        editor.preset(set);
                    }
                }
            });
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                for day in Day::ALL {
                    let on = editor.covers(day);
                    let button =
                        egui::Button::new(RichText::new(day.short()).size(14.0).color(if on {
                            SANCHO_INK
                        } else {
                            p.ink
                        }))
                        .fill(if on { ACCENT } else { p.panel })
                        .stroke(Stroke::new(1.0, p.line))
                        .min_size(vec2(52.0, 34.0));
                    if ui.add(button).on_hover_text(day.name()).clicked() {
                        editor.toggle(day);
                    }
                }
            });
            ui.add_space(4.0);
            ui.add(
                egui::TextEdit::singleline(&mut editor.note)
                    .hint_text("anything else, like: when my brother visits")
                    .desired_width(f32::INFINITY),
            );
            if let Some(hint) = &editor.hint {
                muted(ui, p, hint);
            }
            ui.add_space(6.0);
            ui.label(
                RichText::new(format!("Available: {}", editor.result().words()))
                    .size(15.0)
                    .strong()
                    .color(p.ink),
            );
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.add(accent_button("Save")).clicked() {
                    action = Some(WhenAction::Save);
                }
                if ui.add(plain_button(p, "Cancel")).clicked() {
                    action = Some(WhenAction::Cancel);
                }
                if can_remove
                    && ui
                        .add(plain_button(p, "Take it out of this project"))
                        .clicked()
                {
                    action = Some(WhenAction::Remove);
                }
            });
        });
    action
}

/// How a tool is named on screen: the taxonomy's name for its kind, else what it was called.
fn tool_name(tool: &Tool) -> &str {
    taxonomy::tool(&tool.kind).map_or(tool.label.as_str(), |t| t.name)
}

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
    /// The tool (its id) whose days are being changed, and the form.
    pub when_editing: Option<(i64, WhenEditor)>,
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
            when_editing: None,
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
        if self
            .when_editing
            .as_ref()
            .is_some_and(|(open, _)| *open == id)
        {
            self.when_editing = None;
        }
        if let Err(e) = self.library.remove_tool(id).and_then(|_| self.reload()) {
            self.failed(e);
        }
    }

    /// Opens the days form on a tool.
    pub fn edit_when(&mut self, id: i64) {
        if let Some(tool) = self.tools.iter().find(|t| t.id == id) {
            self.when_editing = Some((id, WhenEditor::new(&tool.when)));
        }
    }

    /// Keeps what the days form says, and closes it.
    pub fn save_when(&mut self) {
        let Some((id, editor)) = self.when_editing.take() else {
            return;
        };
        self.set_when(id, &editor.result());
    }

    /// Changes when a tool can be used, and says what it now is.
    pub fn set_when(&mut self, id: i64, when: &Availability) {
        let name = self
            .tools
            .iter()
            .find(|t| t.id == id)
            .map(|t| tool_name(t).to_string());
        match self
            .library
            .set_tool_when(id, when)
            .and_then(|_| self.reload())
        {
            Ok(()) => {
                if let Some(name) = name {
                    self.said(format!("{name}: {}.", when.words()));
                }
            }
            Err(e) => self.failed(e),
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
        let some_days = match self.tools.iter().filter(|t| !t.when.is_every_day()).count() {
            0 => String::new(),
            1 => " One tool is only there on some days.".to_string(),
            n => format!(" {n} tools are only there on some days."),
        };
        format!(
            "Your workshop has {tools} tools ({owned} you have at hand){} and {materials} material \
             lines ({on_hand} on hand){}.{some_days}",
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

/// One local tool a finished project can hand to the workshop (D31: a tool bought, or got, for
/// the project).
#[derive(Clone, Debug, PartialEq)]
pub struct LeftTool {
    /// The taxonomy's kind id.
    pub kind: String,
    /// How it is named on screen.
    pub name: String,
    /// When it can be used, as the project had it; it goes to the workshop the same way.
    pub when: Availability,
    /// How the person has it now (they say; "I own it" to begin with).
    pub access: Access,
    /// Whether to add it to the workshop.
    pub keep: bool,
}

/// The "when" an inventory entry carries: `{"days": ["sat", "sun"], "note": "at Sam's"}`. An
/// entry with none (every project made before D31), or one that cannot be read, is every day;
/// a note with no days is kept.
pub fn when_from_json(value: &Value) -> Availability {
    let note = value["note"].as_str().unwrap_or("");
    match value["days"].as_array() {
        Some(days) if !days.is_empty() => {
            Availability::from_keys(days.iter().filter_map(Value::as_str), note)
                .unwrap_or_else(|| Availability::always().with_note(note))
        }
        _ => Availability::always().with_note(note),
    }
}

/// How an inventory entry says when: `None` for every day with nothing to say (the entry then
/// has no "when" at all).
pub fn when_to_json(when: &Availability) -> Option<Value> {
    (!when.is_plain()).then(|| json!({ "days": when.keys(), "note": when.note() }))
}

/// The local tools in a project's `inventory.json`. As with materials, an entry with no scope
/// counts as local: it is only offered, never added without a yes.
pub fn local_tools(inventory: &[u8]) -> Vec<LeftTool> {
    let Ok(value) = serde_json::from_slice::<Value>(inventory) else {
        return Vec::new();
    };
    value["tools"]
        .as_array()
        .map(|list| {
            list.iter()
                .filter(|t| t["scope"].as_str().unwrap_or("local") == "local")
                .filter_map(|t| {
                    let kind = t["kind"].as_str()?.trim().to_string();
                    if kind.is_empty() {
                        return None;
                    }
                    let name = t["name"]
                        .as_str()
                        .map(str::trim)
                        .filter(|n| !n.is_empty())
                        .map(String::from)
                        .or_else(|| taxonomy::tool(&kind).map(|k| k.name.to_string()))
                        .unwrap_or_else(|| kind.clone());
                    Some(LeftTool {
                        kind,
                        name,
                        when: when_from_json(&t["when"]),
                        access: Access::Owned,
                        keep: false,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Everything a finished project can offer the workshop: its own materials and its own tools.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Leftovers {
    pub materials: Vec<LeftOver>,
    pub tools: Vec<LeftTool>,
}

impl Leftovers {
    /// What `inventory.json` has that is only for this project.
    pub fn from_inventory(inventory: &[u8]) -> Self {
        Self {
            materials: local_materials(inventory),
            tools: local_tools(inventory),
        }
    }

    /// Nothing of the project's own to offer.
    pub fn is_empty(&self) -> bool {
        self.materials.is_empty() && self.tools.is_empty()
    }

    /// How many the person has ticked (a material with none left does not count).
    pub fn chosen(&self) -> usize {
        self.materials
            .iter()
            .filter(|m| m.keep && m.left > 0)
            .count()
            + self.tools.iter().filter(|t| t.keep).count()
    }
}

/// What reached the workshop.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Added {
    /// The words of the material lines added.
    pub materials: Vec<String>,
    /// The kinds of the tools that are now the workshop's: the ones added, and ones it already had.
    pub tools: Vec<String>,
    /// How many of those tools were new to the workshop.
    pub tools_new: usize,
}

impl Added {
    /// What to tell the person, e.g. "2 material lines and 1 tool added to your workshop".
    pub fn words(&self) -> String {
        let materials = self.materials.len();
        let mut parts = Vec::new();
        if materials > 0 {
            parts.push(format!(
                "{materials} material line{}",
                if materials == 1 { "" } else { "s" }
            ));
        }
        if self.tools_new > 0 {
            parts.push(format!(
                "{} tool{}",
                self.tools_new,
                if self.tools_new == 1 { "" } else { "s" }
            ));
        }
        let had = self.tools.len() - self.tools_new;
        let mut text = match parts.len() {
            0 => "Nothing new was added to your workshop".to_string(),
            1 => format!("{} added to your workshop", parts[0]),
            _ => format!("{} and {} added to your workshop", parts[0], parts[1]),
        };
        if had > 0 {
            text.push_str(&format!(
                " ({had} tool{} already there)",
                if had == 1 { " was" } else { "s were" }
            ));
        }
        text
    }
}

/// Adds the chosen leftovers to the workshop: each material with the number left, each tool with
/// the access and days the person gave it (a tool of a kind the workshop already has is not
/// added a second time).
pub fn add_leftovers(library: &Library, left: &Leftovers) -> Result<Added, LibraryError> {
    let mut added = Added::default();
    for item in left.materials.iter().filter(|i| i.keep && i.left > 0) {
        let material = library.add_material(&item.said)?;
        library.set_qty(material.id, item.left)?;
        added.materials.push(item.said.clone());
    }
    let have = library.tools()?;
    for tool in left.tools.iter().filter(|t| t.keep) {
        if !have.iter().any(|h| h.kind == tool.kind) {
            let id = library.add_tool(&tool.kind, "", tool.access, None)?;
            if !tool.when.is_plain() {
                library.set_tool_when(id, &tool.when)?;
            }
            added.tools_new += 1;
        }
        added.tools.push(tool.kind.clone());
    }
    Ok(added)
}

/// `inventory.json` with the material lines and tools that went to the workshop now global.
/// Bytes that are not an inventory come back unchanged.
pub fn mark_global(inventory: &[u8], added: &Added) -> Vec<u8> {
    let Ok(mut value) = serde_json::from_slice::<Value>(inventory) else {
        return inventory.to_vec();
    };
    let global = || Value::String(scope_word(true).to_string());
    if let Some(list) = value["materials"].as_array_mut() {
        for m in list.iter_mut() {
            let words = m["said"].as_str().unwrap_or("").trim().to_string();
            if added.materials.contains(&words) {
                m["scope"] = global();
            }
        }
    }
    if let Some(list) = value["tools"].as_array_mut() {
        for t in list.iter_mut() {
            let kind = t["kind"].as_str().unwrap_or("").trim().to_string();
            if added.tools.contains(&kind) {
                t["scope"] = global();
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
pub fn finish_ui(ui: &mut Ui, left: &mut Leftovers, p: &Palette) -> Option<FinishAction> {
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
        "These were only for this project. Tick the ones to keep and they join your workshop for \
         every project.",
    );
    ui.add_space(8.0);
    ScrollArea::vertical().max_height(360.0).show(ui, |ui| {
        if !left.tools.is_empty() {
            ui.label(
                RichText::new("Tools you got for this project")
                    .size(15.0)
                    .strong()
                    .color(p.ink),
            );
            for tool in left.tools.iter_mut() {
                ui.horizontal(|ui| {
                    ui.checkbox(&mut tool.keep, "");
                    ui.vertical(|ui| {
                        ui.set_width(300.0);
                        ui.label(RichText::new(&tool.name).size(15.0).strong().color(p.ink));
                        if !tool.when.is_plain() {
                            ui.label(
                                RichText::new(format!("available: {}", tool.when.words()))
                                    .size(13.0)
                                    .color(p.muted),
                            )
                            .on_hover_text("You can change this later in My workshop.");
                        }
                    });
                    ui.add_enabled_ui(tool.keep, |ui| {
                        egui::ComboBox::from_id_salt(("finish-access", &tool.kind))
                            .selected_text(access_words(tool.access))
                            .show_ui(ui, |ui| {
                                for a in &ACCESS_ALL[..4] {
                                    ui.selectable_value(&mut tool.access, *a, access_words(*a));
                                }
                            });
                    });
                });
            }
            ui.add_space(8.0);
        }
        if !left.materials.is_empty() {
            ui.label(
                RichText::new("Materials with some left")
                    .size(15.0)
                    .strong()
                    .color(p.ink),
            );
            muted(ui, p, "Tick the ones with some left, and say how many.");
        }
        for item in left.materials.iter_mut() {
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
    let chosen = left.chosen();
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
    // The days form, over everything, while a tool's days are being changed.
    let title = edit.when_editing.as_ref().and_then(|(id, _)| {
        edit.tools()
            .iter()
            .find(|t| t.id == *id)
            .map(|t| format!("When can you use {}?", tool_name(t)))
    });
    let asked = match (title, edit.when_editing.as_mut()) {
        (Some(title), Some((_, editor))) => when_window(ui.ctx(), &title, editor, false, p),
        _ => None,
    };
    match asked {
        Some(WhenAction::Save) => edit.save_when(),
        Some(WhenAction::Cancel | WhenAction::Remove) => edit.when_editing = None,
        None => {}
    }
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
         change how you have each one, say which days you can use it (a friend's tool, a \
         makerspace's open days), or take it out.",
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
    let mut when = None;
    // The page itself scrolls: an inner scroll area here would be given almost no height.
    for tool in edit.tools() {
        ui.horizontal(|ui| {
            ui.add_sized(
                vec2(210.0, 24.0),
                egui::Label::new(RichText::new(tool_name(tool)).size(15.0).color(p.ink)),
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
            // When it can be used: a quiet "any day" when there is nothing to say, the days
            // when there is.
            let words = tool.when.words();
            let plain = tool.when.is_plain();
            let button = egui::Button::new(RichText::new(&words).size(14.0).color(if plain {
                p.muted
            } else {
                p.ink
            }))
            .fill(if plain { p.panel } else { p.chip })
            .stroke(Stroke::new(1.0, p.line))
            .corner_radius(egui::CornerRadius::same(10));
            if ui
                .add(button)
                .on_hover_text("When can you use it? Click to change the days.")
                .clicked()
            {
                when = Some(tool.id);
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
    if let Some(id) = when {
        edit.edit_when(id);
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
        let left = Leftovers {
            materials: items,
            tools: Vec::new(),
        };
        assert_eq!(
            left.chosen(),
            1,
            "ticked, and some left: the third has none"
        );
        let added = add_leftovers(&library, &left).unwrap();
        assert_eq!(added.materials, vec!["2 yards 500D Cordura".to_string()]);
        assert!(added.tools.is_empty() && added.tools_new == 0);
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
        let went = Added {
            materials: vec!["2 yards 500D Cordura".to_string()],
            ..Added::default()
        };
        let after = mark_global(&bytes, &went);
        let offered = local_materials(&after);
        assert_eq!(offered.len(), 1);
        assert_eq!(offered[0].said, "an old bookshelf");
        assert_eq!(mark_global(b"not json", &went), b"not json".to_vec());
    }

    fn with_tools(tools: Value) -> Vec<u8> {
        serde_json::to_vec(&json!({ "version": 2, "tools": tools, "materials": [] })).unwrap()
    }

    #[test]
    fn only_the_projects_own_tools_are_offered_with_the_days_they_came_with() {
        let bytes = with_tools(json!([
            { "kind": "jigsaw", "name": "Jigsaw", "scope": "global" },
            { "kind": "soldering_iron", "name": "Soldering iron", "scope": "local",
              "when": { "days": ["sat", "sun"], "note": "at Sam's" } },
            { "kind": "band_saw", "scope": "local" },
            { "kind": "drill_press" },
            { "kind": "", "scope": "local" },
        ]));
        let offered = local_tools(&bytes);
        let kinds: Vec<&str> = offered.iter().map(|t| t.kind.as_str()).collect();
        assert_eq!(kinds, vec!["soldering_iron", "band_saw", "drill_press"]);
        assert_eq!(
            offered[0].when,
            Availability::weekends().with_note("at Sam's")
        );
        assert!(offered[1].when.is_plain(), "no days said: every day");
        assert_eq!(
            offered[1].name, "Band saw",
            "the taxonomy's name when none is given"
        );
        assert!(offered.iter().all(|t| !t.keep && t.access == Access::Owned));
        assert!(local_tools(b"not json").is_empty());
        assert!(local_tools(b"{}").is_empty());
        assert!(Leftovers::from_inventory(b"{}").is_empty());
        assert!(!Leftovers::from_inventory(&bytes).is_empty());
    }

    #[test]
    fn days_are_written_and_read_the_same_way_and_a_bad_entry_is_every_day() {
        assert_eq!(when_to_json(&Availability::always()), None);
        let w = Availability::on(&[Day::Tue, Day::Sat])
            .unwrap()
            .with_note("makerspace");
        let json = when_to_json(&w).unwrap();
        assert_eq!(json["days"], json!(["tue", "sat"]));
        assert_eq!(when_from_json(&json), w);
        assert_eq!(when_from_json(&Value::Null), Availability::always());
        assert_eq!(
            when_from_json(&json!({ "days": ["someday"], "note": "x" })),
            Availability::always().with_note("x"),
            "a day it cannot read is every day, and the note stays"
        );
        assert_eq!(
            when_from_json(&json!({ "note": "when my brother visits" })).note(),
            "when my brother visits"
        );
        let any_with_note = Availability::always().with_note("at the makerspace");
        assert_eq!(
            when_from_json(&when_to_json(&any_with_note).unwrap()),
            any_with_note
        );
    }

    #[test]
    fn the_tools_that_are_ticked_join_the_workshop_with_their_access_and_days() {
        let library = Library::in_memory().unwrap();
        library.add_tool("jigsaw", "", Access::Owned, None).unwrap();
        let tool = |kind: &str, keep: bool, access: Access, when: Availability| LeftTool {
            kind: kind.into(),
            name: kind.into(),
            when,
            access,
            keep,
        };
        let left = Leftovers {
            materials: Vec::new(),
            tools: vec![
                tool(
                    "soldering_iron",
                    true,
                    Access::Shared,
                    Availability::weekends(),
                ),
                tool("band_saw", false, Access::Owned, Availability::always()),
                tool("jigsaw", true, Access::Owned, Availability::always()),
                tool("drill_press", true, Access::Owned, Availability::always()),
            ],
        };
        assert_eq!(left.chosen(), 3);
        let added = add_leftovers(&library, &left).unwrap();
        assert_eq!(added.tools_new, 2);
        assert_eq!(added.tools, vec!["soldering_iron", "jigsaw", "drill_press"]);
        let now = library.tools().unwrap();
        let kinds: Vec<&str> = now.iter().map(|t| t.kind.as_str()).collect();
        assert_eq!(
            kinds,
            vec!["jigsaw", "soldering_iron", "drill_press"],
            "the unticked one is not added, and the jigsaw not twice"
        );
        assert_eq!(now[1].access, Access::Shared);
        assert_eq!(now[1].when, Availability::weekends());
        assert_eq!(now[2].access, Access::Owned);
        assert!(now[2].when.is_plain());
    }

    #[test]
    fn what_went_to_the_workshop_is_marked_global_for_tools_too() {
        let bytes = with_tools(json!([
            { "kind": "soldering_iron", "scope": "local" },
            { "kind": "band_saw", "scope": "local" },
        ]));
        let went = Added {
            tools: vec!["soldering_iron".into()],
            tools_new: 1,
            ..Added::default()
        };
        let after = mark_global(&bytes, &went);
        let still: Vec<String> = local_tools(&after).into_iter().map(|t| t.kind).collect();
        assert_eq!(still, vec!["band_saw".to_string()]);
        let value: Value = serde_json::from_slice(&after).unwrap();
        assert_eq!(value["tools"][0]["scope"], "global");
    }

    #[test]
    fn what_was_added_is_told_in_plain_words() {
        let m = |n: usize| (0..n).map(|i| format!("line {i}")).collect::<Vec<_>>();
        let said = |materials: usize, tools: usize, new: usize| {
            Added {
                materials: m(materials),
                tools: (0..tools).map(|i| format!("t{i}")).collect(),
                tools_new: new,
            }
            .words()
        };
        assert_eq!(said(1, 0, 0), "1 material line added to your workshop");
        assert_eq!(said(2, 0, 0), "2 material lines added to your workshop");
        assert_eq!(said(0, 1, 1), "1 tool added to your workshop");
        assert_eq!(
            said(2, 3, 3),
            "2 material lines and 3 tools added to your workshop"
        );
        assert_eq!(
            said(0, 2, 1),
            "1 tool added to your workshop (1 tool was already there)"
        );
        assert_eq!(
            said(0, 2, 0),
            "Nothing new was added to your workshop (2 tools were already there)"
        );
    }

    #[test]
    fn a_tools_days_are_changed_in_the_form_and_kept() {
        let mut e = edit();
        e.tool_text = "soldering iron".into();
        e.add_tool_words();
        let id = e.tools()[0].id;
        assert!(e.tools()[0].when.is_plain());
        e.edit_when(id);
        let (open, form) = e.when_editing.as_mut().unwrap();
        assert_eq!(*open, id);
        form.preset(Availability::weekends());
        form.note = "when my brother ".into();
        form.note.push_str("visits");
        e.save_when();
        assert!(e.when_editing.is_none());
        assert_eq!(
            e.tools()[0].when,
            Availability::weekends().with_note("when my brother visits")
        );
        assert!(e.hint.as_deref().is_some_and(|h| h.contains("weekends")));
        assert!(e.summary().contains("One tool is only there on some days"));
        // Cancelled changes are not kept.
        e.edit_when(id);
        e.when_editing
            .as_mut()
            .unwrap()
            .1
            .preset(Availability::weekdays());
        e.when_editing = None;
        assert_eq!(e.tools()[0].when.days(), Availability::weekends().days());
        // An unknown tool opens nothing.
        e.edit_when(id + 99);
        assert!(e.when_editing.is_none());
    }

    #[test]
    fn the_last_day_cannot_be_unticked_and_the_note_keeps_its_spaces() {
        let mut form = WhenEditor::new(&Availability::on(&[Day::Sat]).unwrap());
        form.toggle(Day::Sat);
        assert!(form.covers(Day::Sat));
        assert!(form.hint.as_deref().is_some_and(|h| h.contains("wishlist")));
        form.toggle(Day::Sun);
        assert!(form.covers(Day::Sun) && form.hint.is_none());
        form.note = "  at   Sam's ".into();
        assert_eq!(form.note, "  at   Sam's ", "kept as typed while typing");
        assert_eq!(form.result().note(), "at Sam's");
        assert_eq!(form.result().days(), vec![Day::Sat, Day::Sun]);
    }

    #[test]
    fn taking_a_tool_out_closes_its_days_form() {
        let mut e = edit();
        e.tool_text = "jigsaw".into();
        e.add_tool_words();
        let id = e.tools()[0].id;
        e.edit_when(id);
        assert!(e.when_editing.is_some());
        e.remove_tool(id);
        assert!(e.when_editing.is_none());
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
