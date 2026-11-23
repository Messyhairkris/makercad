//! "Start something new": the Quest (doc 05 section 5.2; `docs/design/ui-v1/Quest.dc.html`).
//!
//! The steps are the Crew's questions-as-data (`mk_log::quest`), and every one can be skipped.
//! The rail on the left shows them, the middle asks the one in front, and the right shows what
//! the answers set up so far, read by the project interpreter (`mk_assist::interpret`):
//! environments, scale, the toolbox, and Sancho's questions, each with its why. Tools come from
//! the taxonomy's kits and words (`mk_inventory::taxonomy`), materials from quick-add.
//! Confirming the review (or "Skip to designing") gives a [`Created`]: the intent, the
//! environments and the tools-and-materials snapshot (`project/inventory.json`).

use egui::{vec2, Align, CornerRadius, Frame, Key, Layout, Margin, RichText, Stroke, Ui};
use mk_assist::interpret::{self, Interpretation, Why};
use mk_inventory::library::{Library, LibraryError};
use mk_inventory::quickadd::{quick_add, Item, Kind};
use mk_inventory::taxonomy;
use mk_inventory::taxonomy_data::Access;
use mk_log::quest::{self, Answer, AnswerKind, FitPick, Quest, StepId, Template};
use mk_project::project::{is_real_date, Budget, Date};
use serde_json::json;

use crate::theme::{Palette, ACCENT, PAPER, SANCHO_INK};

/// The entry name of the tools-and-materials snapshot (doc 03's layout).
pub const INVENTORY_ENTRY: &str = "project/inventory.json";

/// A scale class (doc 05 section 5.5), whichever crate named it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scale {
    Room,
    Furniture,
    Body,
    Bench,
    Macro,
}

impl Scale {
    fn from_quest(class: quest::ScaleClass) -> Self {
        match class {
            quest::ScaleClass::Room => Scale::Room,
            quest::ScaleClass::Furniture => Scale::Furniture,
            quest::ScaleClass::Body => Scale::Body,
            quest::ScaleClass::Bench => Scale::Bench,
            quest::ScaleClass::Macro => Scale::Macro,
        }
    }

    fn from_interpreter(class: interpret::ScaleClass) -> Self {
        match class {
            interpret::ScaleClass::Room => Scale::Room,
            interpret::ScaleClass::Furniture => Scale::Furniture,
            interpret::ScaleClass::Body => Scale::Body,
            interpret::ScaleClass::Bench => Scale::Bench,
            interpret::ScaleClass::Macro => Scale::Macro,
        }
    }

    /// The class's name, as the review card says it.
    pub fn name(self) -> &'static str {
        match self {
            Scale::Room => "Room",
            Scale::Furniture => "Furniture",
            Scale::Body => "Body",
            Scale::Bench => "Bench",
            Scale::Macro => "Macro",
        }
    }

    /// What the class sets up (doc 05 section 5.5's table, with the bench row as amended).
    pub fn describe(self) -> &'static str {
        match self {
            Scale::Room => "Sancho stands in the room at 168 cm. Grid 100 mm.",
            Scale::Furniture => "A floor patch, Sancho at true size beside the piece. Grid 10 mm.",
            Scale::Body => "Worn or carried: Sancho can try it on. Grid 5 mm.",
            Scale::Bench => {
                "A workbench with a cutting mat, Sancho on the floor beside it. Grid 1 mm."
            }
            Scale::Macro => "A cutting mat with a millimetre grid. Grid 0.1 mm.",
        }
    }
}

/// What the maker picked for the budget step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BudgetPick {
    /// A number, typed in `budget_text`.
    Cap,
    OnlyWhatIHave,
    NotSure,
}

/// One environment the answers call for, with why.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnvNote {
    /// The interpreter's key ("solid", "fold", ...).
    pub key: String,
    /// "because you said *box*".
    pub why: String,
    /// Under the interpreter's confidence line: shown as a question, not a decision.
    pub question: bool,
}

/// Where the workshop library lives: `%LOCALAPPDATA%\MakerCAD\library.db`, beside the graphics
/// memory and the hardware profile (one per person, not per project). `MAKERCAD_LIBRARY`
/// names another file, for screenshots and checks.
pub fn library_path() -> Option<std::path::PathBuf> {
    if let Some(path) = std::env::var_os("MAKERCAD_LIBRARY") {
        return Some(path.into());
    }
    crate::graphics::memory_path().map(|p| p.with_file_name("library.db"))
}

/// What the maker's workshop holds, offered at the tools and materials steps ("Workshop once,
/// projects forever": doc 05 section 5.3).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Workshop {
    /// Tool kinds the maker has (owned or shared) that the taxonomy knows, without repeats.
    pub tools: Vec<&'static str>,
    /// Tools in the library that are not offered: a wishlist, a makerspace's or a rental, or a
    /// kind the taxonomy doesn't know.
    pub left_out: usize,
    /// Material lines on hand.
    pub materials: Vec<Item>,
}

impl Workshop {
    /// Reads the workshop out of the library file at `path` (made, empty, when there is none).
    pub fn open(path: &std::path::Path) -> Result<Workshop, LibraryError> {
        if let Some(folder) = path.parent() {
            // A first run: the folder may not exist yet. A failure shows up as the open's error.
            let _ = std::fs::create_dir_all(folder);
        }
        Workshop::read(&Library::open(path)?)
    }

    /// Reads the workshop out of an open library.
    pub fn read(library: &Library) -> Result<Workshop, LibraryError> {
        let mut workshop = Workshop::default();
        for tool in library.tools()? {
            let have = matches!(tool.access, Access::Owned | Access::Shared);
            match taxonomy::tool(&tool.kind) {
                Some(known) if have => {
                    if !workshop.tools.contains(&known.id) {
                        workshop.tools.push(known.id);
                    }
                }
                _ => workshop.left_out += 1,
            }
        }
        workshop.materials = library
            .materials()?
            .into_iter()
            .filter(|m| m.on_hand)
            .map(|m| m.item)
            .collect();
        Ok(workshop)
    }
}

/// What confirming the Quest makes.
#[derive(Clone, Debug, PartialEq)]
pub struct Created {
    pub name: String,
    pub goal: String,
    pub deadline: Option<Date>,
    pub budget: Option<Budget>,
    /// The manifest's environments: "design" first, then the others the answers call for.
    pub environments: Vec<String>,
    pub scale: Option<Scale>,
    /// `project/inventory.json`'s bytes.
    pub inventory: Vec<u8>,
    /// The steps skipped or never answered, for the Intent card.
    pub outstanding: Vec<StepId>,
}

/// A Quest in progress, and what has been typed into each step.
pub struct NewProject {
    quest: Quest,
    /// The step in front, as an index into the steps.
    pub at: usize,
    pub name: String,
    pub goal: String,
    pub fit: Option<FitPick>,
    /// The longest size, when the fit pick is "I'll type sizes".
    pub size_text: String,
    /// Tool kind ids, in the order they were added.
    pub tools: Vec<&'static str>,
    pub tool_text: String,
    pub materials: Vec<Item>,
    pub material_text: String,
    pub deadline_text: String,
    pub no_deadline: bool,
    pub budget: Option<BudgetPick>,
    pub budget_text: String,
    pub currency: String,
    pub notes: Vec<String>,
    pub note_text: String,
    /// The last thing said about an entry ("I don't know that tool yet").
    pub hint: Option<String>,
    /// Bare numbers in the size box are inches, not millimetres.
    pub inches: bool,
    /// The maker's workshop, when the library could be read.
    pub workshop: Workshop,
}

impl NewProject {
    /// A new Quest (the default template).
    pub fn new(inches: bool) -> Self {
        Self {
            quest: Quest::new(Template::Default),
            at: 0,
            name: String::new(),
            goal: String::new(),
            fit: None,
            size_text: String::new(),
            tools: Vec::new(),
            tool_text: String::new(),
            materials: Vec::new(),
            material_text: String::new(),
            deadline_text: String::new(),
            no_deadline: false,
            budget: None,
            budget_text: String::new(),
            currency: "USD".into(),
            notes: Vec::new(),
            note_text: String::new(),
            hint: None,
            inches,
            workshop: Workshop::default(),
        }
    }

    /// The workshop tool kinds this project doesn't list yet.
    pub fn workshop_tools_missing(&self) -> Vec<&'static str> {
        self.workshop
            .tools
            .iter()
            .copied()
            .filter(|k| !self.tools.contains(k))
            .collect()
    }

    /// The workshop's material lines this project doesn't list yet.
    pub fn workshop_materials_missing(&self) -> Vec<Item> {
        self.workshop
            .materials
            .iter()
            .filter(|m| !self.materials.iter().any(|mine| mine.text == m.text))
            .cloned()
            .collect()
    }

    /// Lists every tool of the workshop (the maker unticks any not needed here).
    pub fn use_workshop_tools(&mut self) {
        let missing = self.workshop_tools_missing();
        self.hint = Some(format!(
            "Your workshop: {} added. Untick any you won't use here.",
            match missing.len() {
                1 => "1 tool".to_string(),
                n => format!("{n} tools"),
            }
        ));
        self.tools.extend(missing);
    }

    /// Lists every material line the workshop has on hand.
    pub fn use_workshop_materials(&mut self) {
        let missing = self.workshop_materials_missing();
        self.hint = Some(format!(
            "Your workshop: {} added. Remove any you won't use here.",
            match missing.len() {
                1 => "1 material line".to_string(),
                n => format!("{n} material lines"),
            }
        ));
        self.materials.extend(missing);
    }

    /// Whether a tool of this project is the workshop's (global) rather than the project's own
    /// (local, D30): the workshop has the kind.
    pub fn tool_is_global(&self, kind: &str) -> bool {
        self.workshop.tools.contains(&kind)
    }

    /// Whether a material line of this project is the workshop's: the workshop has that line.
    pub fn material_is_global(&self, item: &Item) -> bool {
        self.workshop.materials.iter().any(|w| w.text == item.text)
    }

    /// A sample dresser's answers, for `--quest` screenshots: every step before the review is
    /// answered, so the rail, the fields and the review card all show something.
    pub fn fill_sample(&mut self) {
        self.name = "Alcove dresser".into();
        self.goal =
            "A dresser for the bedroom alcove, four drawers, that comes apart for moving".into();
        self.fit = Some(FitPick::TypeSizes);
        self.size_text = if self.inches { "5 ft" } else { "1500 mm" }.into();
        self.add_kit("starter_woodshop");
        for line in [
            "15 pine 2x4s, 3 foot",
            "half sheet 3/4 ply",
            "4 pairs drawer slides",
        ] {
            self.material_text = line.into();
            self.add_material();
        }
        self.deadline_text = "2027-01-15".into();
        self.budget = Some(BudgetPick::Cap);
        self.budget_text = "150".into();
        self.note_text = "warm oak, rounded corners".into();
        self.add_note();
        self.hint = None;
        // Record the answers so the rail's marks show them (a failed step just stays open).
        for _ in 0..self.quest.steps().len() - 1 {
            if self.advance().is_err() {
                break;
            }
        }
        self.at = 0;
    }

    /// The steps, in order.
    pub fn steps(&self) -> &[quest::Step] {
        self.quest.steps()
    }

    /// The step in front.
    pub fn step(&self) -> &quest::Step {
        let steps = self.quest.steps();
        &steps[self.at.min(steps.len() - 1)]
    }

    /// The answer recorded for a step, if any.
    pub fn answer_for(&self, id: &StepId) -> Option<&Answer> {
        self.quest.answer_for(id)
    }

    /// Shows step `index`.
    pub fn go_to(&mut self, index: usize) {
        self.at = index.min(self.quest.steps().len() - 1);
        self.hint = None;
    }

    /// Records the step in front as skipped and moves on.
    pub fn skip(&mut self) {
        let id = self.step().id.clone();
        if id != StepId::Review {
            let _ = self.quest.answer(id, Answer::Skipped);
            self.go_to(self.at + 1);
        }
    }

    /// Records what was typed into the step in front and moves on; says why when it can't.
    pub fn advance(&mut self) -> Result<(), String> {
        let step = self.step().clone();
        let answer = self.answer_of(&step)?;
        self.quest
            .answer(step.id, answer)
            .map_err(|e| format!("{e:?}"))?;
        self.go_to(self.at + 1);
        Ok(())
    }

    /// The answer the fields give for `step` (Skipped when nothing was given).
    fn answer_of(&self, step: &quest::Step) -> Result<Answer, String> {
        let text = |s: &str| {
            let s = s.trim();
            if s.is_empty() {
                Answer::Skipped
            } else {
                Answer::Text(s.to_string())
            }
        };
        let count = |n: usize| Answer::Items(u32::try_from(n).unwrap_or(u32::MAX));
        Ok(match step.id {
            StepId::Name => text(&self.name),
            StepId::Goal => text(&self.goal),
            StepId::Fit => self.fit.map_or(Answer::Skipped, Answer::Fit),
            StepId::Tools => count(self.tools.len()),
            StepId::Materials => count(self.materials.len()),
            StepId::Inspiration => count(self.notes.len()),
            StepId::Deadline if self.no_deadline => Answer::Deadline(None),
            StepId::Deadline => match self.deadline()? {
                Some(d) => Answer::Deadline(Some((d.year, d.month, d.day))),
                None => Answer::Skipped,
            },
            StepId::Budget => match self.budget {
                None => Answer::Skipped,
                Some(BudgetPick::OnlyWhatIHave) => Answer::BudgetOnlyWhatIHave,
                Some(BudgetPick::NotSure) => Answer::BudgetNotSure,
                Some(BudgetPick::Cap) => match self.budget_cap()? {
                    Some(cap) => Answer::BudgetCap(cap, self.currency.trim().to_uppercase()),
                    None => Answer::Skipped,
                },
            },
            StepId::BoxContents | StepId::HowMany => Answer::Skipped,
            StepId::Review => Answer::Confirmed,
        })
        .map(|answer| match (step.kind, answer) {
            // The review takes only a confirmation; everything else may be skipped.
            (AnswerKind::Review, _) => Answer::Confirmed,
            (_, answer) => answer,
        })
    }

    /// The typed deadline: none when the box is empty; an error when it is not a real date.
    pub fn deadline(&self) -> Result<Option<Date>, String> {
        let text = self.deadline_text.trim();
        if self.no_deadline || text.is_empty() {
            return Ok(None);
        }
        parse_date(text).map(Some).ok_or_else(|| {
            format!("\u{201c}{text}\u{201d} isn't a date I can read. Type it as 2026-12-24.")
        })
    }

    /// The typed budget in the currency's smallest unit: none when the box is empty.
    pub fn budget_cap(&self) -> Result<Option<u64>, String> {
        let text = self.budget_text.trim();
        if text.is_empty() {
            return Ok(None);
        }
        parse_money(text).map(Some).ok_or_else(|| {
            format!("\u{201c}{text}\u{201d} isn't an amount I can read. Type it as 150 or 149.99.")
        })
    }

    /// Adds the tool the typed words name (the longest matching phrase wins).
    pub fn add_tool_words(&mut self) {
        let text = self.tool_text.trim().to_string();
        if text.is_empty() {
            return;
        }
        match taxonomy::tool_by_words(&text) {
            Some(tool) => {
                if !self.tools.contains(&tool.id) {
                    self.tools.push(tool.id);
                }
                self.hint = Some(format!("\u{201c}{text}\u{201d}: {}.", tool.name));
                self.tool_text.clear();
            }
            None => {
                self.hint = Some(format!(
                    "I don't know \u{201c}{text}\u{201d} yet. Try another word for it, or a kit."
                ))
            }
        }
    }

    /// Adds every tool of a kit the maker doesn't already have.
    pub fn add_kit(&mut self, kit_id: &str) {
        if let Some(kit) = taxonomy::kit(kit_id) {
            let before = self.tools.len();
            for id in kit.tools {
                if !self.tools.contains(id) {
                    self.tools.push(*id);
                }
            }
            self.hint = Some(format!(
                "{}: {} tools added. Untick any you don't have.",
                kit.name,
                self.tools.len() - before
            ));
        }
    }

    /// Reads the typed line with quick-add and lists it.
    pub fn add_material(&mut self) {
        let text = self.material_text.trim().to_string();
        if text.is_empty() {
            return;
        }
        let item = quick_add(&text);
        self.hint = Some(format!("\u{201c}{text}\u{201d}: {}.", describe_item(&item)));
        self.materials.push(item);
        self.material_text.clear();
    }

    /// Adds the typed inspiration note.
    pub fn add_note(&mut self) {
        let text = self.note_text.trim().to_string();
        if !text.is_empty() {
            self.notes.push(text);
            self.note_text.clear();
        }
    }

    /// What the interpreter reads from the answers so far.
    pub fn interpretation(&self) -> Interpretation {
        interpret::interpret(&interpret::Input {
            goal: self.goal.clone(),
            style_words: Vec::new(),
            notes: self.notes.clone(),
            materials: self.materials.iter().map(|m| m.text.clone()).collect(),
            attachments: Vec::new(),
        })
    }

    /// The scale class: a typed longest size wins over the interpreter's guess.
    pub fn scale(&self, interpretation: &Interpretation) -> Option<(Scale, String)> {
        if self.fit == Some(FitPick::TypeSizes) {
            if let Some(mm) = self.longest_mm() {
                if let Some(class) = quest::class_for_size(mm) {
                    return Some((
                        Scale::from_quest(class),
                        format!("its longest size is {}", self.size_text.trim()),
                    ));
                }
            }
        }
        if self.fit == Some(FitPick::SmallNoFixedSpace) && interpretation.scale.is_none() {
            return Some((Scale::Bench, "you said it's small".into()));
        }
        interpretation
            .scale
            .as_ref()
            .map(|(class, why)| (Scale::from_interpreter(*class), because(why)))
    }

    /// The typed longest size in millimetres, when it reads as a length.
    pub fn longest_mm(&self) -> Option<f64> {
        let unit = if self.inches {
            mk_units::Unit::In
        } else {
            mk_units::Unit::Mm
        };
        mk_units::parse_length(&self.size_text, unit).ok()
    }

    /// The environments the answers call for, each once, in the order they were found.
    pub fn environments(&self, interpretation: &Interpretation) -> Vec<EnvNote> {
        let mut out: Vec<EnvNote> = Vec::new();
        let mut add = |key: &str, why: String, question: bool| {
            if !out.iter().any(|e| e.key == key) {
                out.push(EnvNote {
                    key: key.into(),
                    why,
                    question,
                });
            }
        };
        for item in &interpretation.envs {
            add(&item.value, because(&item.why), item.why.is_question());
        }
        if !self.materials.is_empty() {
            add("stock", "you listed materials".into(), false);
        }
        if matches!(self.deadline(), Ok(Some(_))) {
            add("plan", "you have a deadline".into(), false);
        }
        add("solid", "every project can be modelled".into(), false);
        out
    }

    /// What confirming makes; the Quest is confirmed first (with "Skip to designing", every
    /// step not yet answered is skipped).
    pub fn finish(&mut self, skip_rest: bool) -> Result<Created, String> {
        if skip_rest {
            self.quest.skip_to_designing();
        } else if !self.quest.is_done() {
            let _ = self.quest.answer(StepId::Review, Answer::Confirmed);
        }
        let interpretation = self.interpretation();
        let name = match self.name.trim() {
            "" => "My project".to_string(),
            name => name.to_string(),
        };
        let deadline = self.deadline().unwrap_or(None);
        let budget = match self.budget {
            Some(BudgetPick::Cap) => self.budget_cap().unwrap_or(None).map(|cap| Budget {
                cap_minor: cap,
                currency: self.currency.trim().to_uppercase(),
                spend_only_if_needed: false,
            }),
            Some(BudgetPick::OnlyWhatIHave) => Some(Budget {
                cap_minor: 0,
                currency: self.currency.trim().to_uppercase(),
                spend_only_if_needed: true,
            }),
            _ => None,
        };
        let mut environments = vec!["design".to_string()];
        for env in self.environments(&interpretation) {
            if env.question || env.key == "solid" || env.key == "sketch" {
                continue;
            }
            if !environments.contains(&env.key) {
                environments.push(env.key);
            }
        }
        Ok(Created {
            name,
            goal: self.goal.trim().to_string(),
            deadline,
            budget,
            environments,
            scale: self.scale(&interpretation).map(|(s, _)| s),
            inventory: self.inventory_json(),
            outstanding: self.quest.outstanding(),
        })
    }

    /// `project/inventory.json`: the tools and materials this project starts with.
    pub fn inventory_json(&self) -> Vec<u8> {
        let tools: Vec<_> = self
            .tools
            .iter()
            .map(|id| {
                let name = taxonomy::tool(id).map_or("", |t| t.name);
                json!({ "kind": id, "name": name, "scope": scope_word(self.tool_is_global(id)) })
            })
            .collect();
        let materials: Vec<_> = self
            .materials
            .iter()
            .map(|m| {
                json!({
                    "scope": scope_word(self.material_is_global(m)),
                    "said": m.text,
                    "qty": m.qty,
                    "kind": format!("{:?}", m.kind).to_lowercase(),
                    "material": m.material,
                    "section_mm": m.section_mm.map(|(a, b)| [a, b]),
                    "length_mm": m.length_mm,
                    "thickness_mm": m.thickness_mm,
                    "sheet_mm": m.sheet_mm.map(|(a, b)| [a, b]),
                    "condition": m.condition,
                    "notes": m.notes,
                })
            })
            .collect();
        let value = json!({ "version": 2, "tools": tools, "materials": materials });
        serde_json::to_vec_pretty(&value).unwrap_or_default()
    }
}

/// How `project/inventory.json` says whether an entry is the workshop's or the project's own
/// (D30).
pub fn scope_word(global: bool) -> &'static str {
    if global {
        "global"
    } else {
        "local"
    }
}

/// "because you said *box*", or the attachment, or the note.
fn because(why: &Why) -> String {
    if why.matched.is_empty() {
        "from your answers".into()
    } else {
        format!("because you said \u{201c}{}\u{201d}", why.matched)
    }
}

/// `YYYY-MM-DD` (or with `/`), a real date.
pub fn parse_date(text: &str) -> Option<Date> {
    let parts: Vec<&str> = text.trim().split(['-', '/']).collect();
    let [y, m, d] = parts.as_slice() else {
        return None;
    };
    let date = Date {
        year: y.trim().parse().ok()?,
        month: m.trim().parse().ok()?,
        day: d.trim().parse().ok()?,
    };
    (date.year >= 1970 && is_real_date(date)).then_some(date)
}

/// "150", "149.99", "$1,200": the amount in the currency's smallest unit.
pub fn parse_money(text: &str) -> Option<u64> {
    let cleaned: String = text
        .trim()
        .trim_start_matches(['$', '\u{a3}', '\u{20ac}'])
        .chars()
        .filter(|c| *c != ',')
        .collect();
    let (whole, cents) = match cleaned.split_once('.') {
        Some((w, c)) if (1..=2).contains(&c.len()) => (w, c),
        Some(_) => return None,
        None => (cleaned.as_str(), "0"),
    };
    if whole.is_empty() || !whole.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    if !cents.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let cents: u64 = if cents.len() == 1 {
        cents.parse::<u64>().ok()? * 10
    } else {
        cents.parse().ok()?
    };
    whole
        .parse::<u64>()
        .ok()?
        .checked_mul(100)?
        .checked_add(cents)
}

/// What quick-add understood, in a few words.
pub fn describe_item(item: &Item) -> String {
    let mut parts = vec![format!("{} \u{d7}", item.qty)];
    if !item.material.is_empty() {
        parts.push(item.material.clone());
    }
    parts.push(
        match item.kind {
            Kind::Board => "board",
            Kind::Sheet => "sheet",
            Kind::Other => "item",
        }
        .into(),
    );
    if let Some((a, b)) = item.section_mm {
        parts.push(format!("{a:.0} \u{d7} {b:.0} mm"));
    }
    if let Some(t) = item.thickness_mm {
        parts.push(format!("{t:.1} mm thick"));
    }
    if let Some((a, b)) = item.sheet_mm {
        parts.push(format!("{a:.0} \u{d7} {b:.0} mm"));
    }
    if let Some(l) = item.length_mm {
        parts.push(format!("{l:.0} mm long"));
    }
    if let Some(c) = &item.condition {
        parts.push(c.clone());
    }
    parts.join(" ")
}

/// An environment key's name on the review card.
pub fn env_name(key: &str) -> &str {
    match key {
        "solid" => "Solid",
        "sketch" => "Sketch",
        "stock" => "Stock & Cut",
        "fold" => "Fold & Package",
        "drawings" => "Drawings",
        "softgoods" => "Soft Goods",
        "motion" => "Motion",
        "plan" => "Plan",
        "space" => "Space & Scan",
        other => other,
    }
}

/// A toolbox command's name ("drawer_wizard" -> "Drawer wizard").
pub fn tool_ui_name(key: &str) -> String {
    let spaced = key.replace('_', " ");
    let mut chars = spaced.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

/// The rail's short name for a step.
fn short(id: &StepId) -> &'static str {
    match id {
        StepId::Name => "Name it",
        StepId::Goal => "What to make",
        StepId::Fit => "Where it fits",
        StepId::Tools => "Your tools",
        StepId::Materials => "Your materials",
        StepId::Deadline => "When you need it",
        StepId::Budget => "Budget",
        StepId::Inspiration => "What inspires you",
        StepId::BoxContents => "What goes in the box",
        StepId::HowMany => "How many",
        StepId::Review => "Your workspace",
    }
}

/// What the person asked for on the Quest screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuestAction {
    /// Back to the opening screen; nothing is made.
    Cancel,
    /// The review was confirmed: make the project.
    Create,
    /// "Skip to designing": make the project from what there is.
    SkipToDesigning,
}

/// Width at which the "what your answers set up" column moves below the question.
const SIDE_BY_SIDE: f32 = 1100.0;
const RAIL_WIDTH: f32 = 210.0;
const ASIDE_WIDTH: f32 = 330.0;

/// Draws the Quest into `ui` and returns what the person asked for, if anything.
pub fn show(ui: &mut Ui, np: &mut NewProject, p: &Palette) -> Option<QuestAction> {
    crate::home::paint_background(ui, p);
    let mut action = None;
    let interpretation = np.interpretation();

    Frame::new()
        .inner_margin(Margin::symmetric(32, 18))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("New project")
                        .size(24.0)
                        .strong()
                        .color(p.ink),
                );
                if !np.name.trim().is_empty() {
                    ui.label(
                        RichText::new(format!("\u{b7} {}", np.name.trim()))
                            .size(20.0)
                            .color(p.muted),
                    );
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.add(plain_button(p, "Cancel")).clicked() {
                        action = Some(QuestAction::Cancel);
                    }
                    if ui
                        .add(accent_button("Skip to designing"))
                        .on_hover_text("Every question stays on the Intent card for later.")
                        .clicked()
                    {
                        action = Some(QuestAction::SkipToDesigning);
                    }
                });
            });
        });

    egui::ScrollArea::vertical()
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
                    let width = ui.available_width().min(1320.0);
                    let side = width >= SIDE_BY_SIDE;
                    let middle = if side {
                        width - RAIL_WIDTH - ASIDE_WIDTH - 48.0
                    } else {
                        width - RAIL_WIDTH - 24.0
                    };
                    ui.horizontal_top(|ui| {
                        column(ui, RAIL_WIDTH, |ui| rail(ui, np, p));
                        ui.add_space(24.0);
                        column(ui, middle, |ui| {
                            if let Some(a) = question(ui, np, &interpretation, p) {
                                action = Some(a);
                            }
                            if !side && np.step().id != StepId::Review {
                                ui.add_space(20.0);
                                set_up(ui, np, &interpretation, p);
                            }
                        });
                        if side && np.step().id != StepId::Review {
                            ui.add_space(24.0);
                            column(ui, ASIDE_WIDTH, |ui| set_up(ui, np, &interpretation, p));
                        }
                    });
                });
        });
    action
}

pub(crate) fn column(ui: &mut Ui, width: f32, add: impl FnOnce(&mut Ui)) {
    ui.allocate_ui_with_layout(vec2(width, 0.0), Layout::top_down(Align::Min), |ui| {
        ui.set_width(width);
        add(ui);
    });
}

pub(crate) fn accent_button(text: &str) -> egui::Button<'_> {
    egui::Button::new(RichText::new(text).size(15.0).strong().color(SANCHO_INK))
        .min_size(vec2(0.0, 40.0))
        .fill(ACCENT)
        .stroke(Stroke::new(2.0, SANCHO_INK))
        .corner_radius(CornerRadius::same(12))
}

pub(crate) fn plain_button<'a>(p: &Palette, text: &'a str) -> egui::Button<'a> {
    egui::Button::new(RichText::new(text).size(15.0).strong().color(p.ink))
        .min_size(vec2(0.0, 40.0))
        .fill(p.bg2)
        .stroke(Stroke::new(2.0, p.line))
        .corner_radius(CornerRadius::same(12))
}

pub(crate) fn card(ui: &mut Ui, p: &Palette, add: impl FnOnce(&mut Ui)) {
    Frame::new()
        .fill(p.panel)
        .stroke(Stroke::new(2.0, p.line))
        .corner_radius(CornerRadius::same(18))
        .inner_margin(Margin::symmetric(22, 18))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            add(ui);
        });
}

fn rail(ui: &mut Ui, np: &mut NewProject, p: &Palette) {
    let mut go = None;
    let steps: Vec<quest::Step> = np.steps().to_vec();
    for (i, step) in steps.iter().enumerate() {
        let here = i == np.at;
        let (mark, done) = match np.answer_for(&step.id) {
            Some(Answer::Skipped) => ("\u{2013}".to_string(), false),
            Some(_) => (String::new(), true),
            None => ((i + 1).to_string(), false),
        };
        let fill = if here {
            ACCENT
        } else if done {
            crate::theme::STAGE_DONE
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
            let ink = if here || done { SANCHO_INK } else { p.muted };
            if done {
                // The font has no check mark glyph, so it is drawn.
                let c = dot.center();
                let stroke = Stroke::new(2.2, ink);
                ui.painter()
                    .line_segment([c + vec2(-5.0, 0.5), c + vec2(-1.5, 4.0)], stroke);
                ui.painter()
                    .line_segment([c + vec2(-1.5, 4.0), c + vec2(5.5, -4.0)], stroke);
            } else {
                ui.painter().text(
                    dot.center(),
                    egui::Align2::CENTER_CENTER,
                    mark,
                    egui::FontId::proportional(14.0),
                    ink,
                );
            }
            let text =
                RichText::new(short(&step.id))
                    .size(15.0)
                    .color(if here { p.ink } else { p.muted });
            ui.add(egui::Label::new(if here { text.strong() } else { text }).selectable(false));
        });
        let click = ui.interact(
            row.response.rect,
            ui.scope_id().with(("quest-step", i)),
            egui::Sense::click(),
        );
        if click.clicked() {
            go = Some(i);
        }
        ui.add_space(6.0);
    }
    if let Some(i) = go {
        np.go_to(i);
    }
    ui.add_space(10.0);
    ui.add(
        egui::Label::new(
            RichText::new(
                "Every question can be skipped. Skipped ones stay on the Intent card for later.",
            )
            .size(13.0)
            .color(p.muted),
        )
        .wrap(),
    );
}

/// The step in front: its question, its fields, and Next / Skip.
fn question(
    ui: &mut Ui,
    np: &mut NewProject,
    interpretation: &Interpretation,
    p: &Palette,
) -> Option<QuestAction> {
    let mut action = None;
    let step = np.step().clone();
    card(ui, p, |ui| {
        ui.label(
            RichText::new(step.question)
                .size(26.0)
                .strong()
                .color(p.ink),
        );
        ui.add_space(4.0);
        match step.id {
            StepId::Name => {
                muted(ui, p, "A short name. It becomes the file's name too.");
                let r = field(ui, &mut np.name, "Alcove dresser");
                enter_moves_on(ui, &r, np);
            }
            StepId::Goal => {
                muted(
                    ui,
                    p,
                    "Say it the way you'd tell a friend: what, for where, and anything it must do.",
                );
                ui.add(
                    egui::TextEdit::multiline(&mut np.goal)
                        .hint_text("A dresser for the bedroom alcove, four drawers, that comes apart for moving")
                        .desired_rows(4)
                        .desired_width(f32::INFINITY)
                        .font(egui::FontId::proportional(17.0)),
                );
            }
            StepId::Fit => fit_step(ui, np, p),
            StepId::Tools => tools_step(ui, np, p),
            StepId::Materials => materials_step(ui, np, p),
            StepId::Deadline => {
                muted(
                    ui,
                    p,
                    "A date helps Plan fit the work into the evenings you have.",
                );
                ui.horizontal(|ui| {
                    ui.add_enabled_ui(!np.no_deadline, |ui| {
                        let r = field(ui, &mut np.deadline_text, "2026-12-24");
                        enter_moves_on(ui, &r, np);
                    });
                    ui.checkbox(&mut np.no_deadline, RichText::new("No deadline").size(15.0));
                });
            }
            StepId::Budget => budget_step(ui, np, p),
            StepId::Inspiration => {
                muted(ui, p, "Notes, words for the style, links to pictures. Images come with the mood board.");
                if add_box(
                    ui,
                    p,
                    &mut np.note_text,
                    "warm oak, rounded corners, like a ship's chest",
                ) {
                    np.add_note();
                }
                let mut remove = None;
                for (i, note) in np.notes.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!("\u{2022} {note}"))
                                .size(15.0)
                                .color(p.ink),
                        );
                        if ui.small_button("\u{d7}").on_hover_text("Remove").clicked() {
                            remove = Some(i);
                        }
                    });
                }
                if let Some(i) = remove {
                    np.notes.remove(i);
                }
            }
            StepId::BoxContents | StepId::HowMany => {
                muted(
                    ui,
                    p,
                    "The packaging questions come with Fold & Package (P7).",
                );
            }
            StepId::Review => {
                muted(
                    ui,
                    p,
                    "This is what your answers set up. You can change any of it later.",
                );
                ui.add_space(6.0);
                set_up_body(ui, np, interpretation, p);
                let outstanding = np.quest.outstanding();
                if !outstanding.is_empty() {
                    ui.add_space(8.0);
                    let names: Vec<&str> = outstanding.iter().map(short).collect();
                    muted(
                        ui,
                        p,
                        &format!("Still open, on the Intent card: {}.", names.join(", ")),
                    );
                }
            }
        }
        if let Some(hint) = &np.hint {
            ui.add_space(6.0);
            ui.label(RichText::new(hint).size(14.0).color(p.muted).italics());
        }
        ui.add_space(14.0);
        ui.horizontal(|ui| {
            if step.id == StepId::Review {
                if ui.add(accent_button("Open the workspace")).clicked() {
                    action = Some(QuestAction::Create);
                }
            } else {
                let label = match np.steps().get(np.at + 1) {
                    Some(next) => format!("Next: {}", short(&next.id).to_lowercase()),
                    None => "Next".into(),
                };
                if ui.add(accent_button(&label)).clicked() {
                    if let Err(e) = np.advance() {
                        np.hint = Some(e);
                    }
                }
                if ui.add(plain_button(p, "Skip for now")).clicked() {
                    np.skip();
                }
            }
            if np.at > 0 && ui.add(plain_button(p, "Back")).clicked() {
                np.go_to(np.at - 1);
            }
        });
    });
    action
}

pub(crate) fn muted(ui: &mut Ui, p: &Palette, text: &str) {
    ui.add(egui::Label::new(RichText::new(text).size(15.0).color(p.muted)).wrap());
    ui.add_space(4.0);
}

fn field(ui: &mut Ui, text: &mut String, hint: &str) -> egui::Response {
    ui.add(
        egui::TextEdit::singleline(text)
            .hint_text(hint)
            .desired_width(360.0)
            .font(egui::FontId::proportional(17.0)),
    )
}

/// Enter in a one-line field means Next.
fn enter_moves_on(ui: &Ui, response: &egui::Response, np: &mut NewProject) {
    if response.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
        if let Err(e) = np.advance() {
            np.hint = Some(e);
        }
    }
}

/// A one-line box with an Add button; true when Add was pressed (or Enter in the box).
pub(crate) fn add_box(ui: &mut Ui, p: &Palette, text: &mut String, hint: &str) -> bool {
    let mut wanted = false;
    ui.horizontal(|ui| {
        let r = field(ui, text, hint);
        if r.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
            wanted = true;
            r.request_focus();
        }
        if ui.add(plain_button(p, "Add")).clicked() {
            wanted = true;
        }
    });
    wanted
}

fn fit_step(ui: &mut Ui, np: &mut NewProject, p: &Palette) {
    for (pick, label) in [
        (FitPick::TypeSizes, "I'll type sizes"),
        (FitPick::HaveScan, "I have a scan"),
        (
            FitPick::PhotographWithReference,
            "I'll photograph it with a reference",
        ),
        (FitPick::SmallNoFixedSpace, "It's small, no fixed space"),
        (FitPick::DecideLater, "I'll decide later"),
    ] {
        if ui
            .radio(np.fit == Some(pick), RichText::new(label).size(16.0))
            .clicked()
        {
            np.fit = Some(pick);
        }
    }
    match np.fit {
        Some(FitPick::TypeSizes) => {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("Its longest size").size(15.0).color(p.ink));
                field(
                    ui,
                    &mut np.size_text,
                    if np.inches { "48 in" } else { "1200 mm" },
                );
            });
            if !np.size_text.trim().is_empty() {
                let said = match np.longest_mm().and_then(quest::class_for_size) {
                    Some(class) => format!("That's {} scale.", Scale::from_quest(class).name()),
                    None => {
                        "I can't read that as a length yet (try 1200 mm, 4 ft or 48 in).".into()
                    }
                };
                muted(ui, p, &said);
            }
        }
        Some(FitPick::HaveScan) | Some(FitPick::PhotographWithReference) => {
            muted(
                ui,
                p,
                "Space & Scan comes in P6; the project remembers you have one.",
            );
        }
        _ => {}
    }
}

fn tools_step(ui: &mut Ui, np: &mut NewProject, p: &Palette) {
    if !np.workshop_tools_missing().is_empty() {
        ui.horizontal_wrapped(|ui| {
            let more = match np.workshop.left_out {
                0 => String::new(),
                n => format!(" ({n} more are on a wishlist, borrowed, or not known to me yet)"),
            };
            ui.label(
                RichText::new(format!(
                    "Your workshop has {} tools I know{more}.",
                    np.workshop.tools.len()
                ))
                .size(15.0)
                .color(p.ink),
            );
            if ui.add(plain_button(p, "Use my workshop")).clicked() {
                np.use_workshop_tools();
            }
        });
        ui.add_space(6.0);
    }
    muted(
        ui,
        p,
        "Start from a kit, then untick what you don't have, or type a tool's name.",
    );
    ui.horizontal_wrapped(|ui| {
        for kit in taxonomy::kits() {
            if ui.add(plain_button(p, kit.name)).clicked() {
                np.add_kit(kit.id);
            }
        }
    });
    ui.add_space(6.0);
    if add_box(
        ui,
        p,
        &mut np.tool_text,
        "jigsaw, cordless drill, sewing machine",
    ) {
        np.add_tool_words();
    }
    ui.add_space(6.0);
    let mut remove = None;
    ui.horizontal_wrapped(|ui| {
        for (i, id) in np.tools.iter().enumerate() {
            let name = taxonomy::tool(id).map_or(*id, |t| t.name);
            // From the workshop (global) is a filled chip; typed here (local, D30) is outlined.
            let global = np.tool_is_global(id);
            let chip = egui::Button::new(RichText::new(name).size(14.0).color(p.ink))
                .fill(if global { p.chip } else { p.panel })
                .stroke(if global {
                    Stroke::new(1.0, p.line)
                } else {
                    Stroke::new(2.0, ACCENT)
                })
                .corner_radius(CornerRadius::same(10));
            let hint = if global {
                "From your workshop. Click to leave it out of this project."
            } else {
                "Only for this project. Click to remove it."
            };
            if ui.add(chip).on_hover_text(hint).clicked() {
                remove = Some(i);
            }
        }
    });
    if let Some(i) = remove {
        np.tools.remove(i);
    }
    if !np.tools.is_empty() {
        let own = np.tools.iter().filter(|k| !np.tool_is_global(k)).count();
        muted(
            ui,
            p,
            &if own == 0 {
                format!("{} tools.", np.tools.len())
            } else {
                format!(
                    "{} tools; the {own} with an outline are only for this project. Keep your \
                     workshop up to date in My workshop.",
                    np.tools.len()
                )
            },
        );
    }
}

fn materials_step(ui: &mut Ui, np: &mut NewProject, p: &Palette) {
    if !np.workshop_materials_missing().is_empty() {
        ui.horizontal_wrapped(|ui| {
            ui.label(
                RichText::new(format!(
                    "Your workshop has {} material lines on hand.",
                    np.workshop.materials.len()
                ))
                .size(15.0)
                .color(p.ink),
            );
            if ui.add(plain_button(p, "Use my workshop")).clicked() {
                np.use_workshop_materials();
            }
        });
        ui.add_space(6.0);
    }
    muted(ui, p, "Type them the way you'd say them.");
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new("Try:").size(14.0).color(p.muted));
        for example in [
            "15 pine 2x4s, 3 foot",
            "half sheet 3/4 ply",
            "2 yards 500D Cordura",
            "an old bookshelf",
        ] {
            if ui
                .add(egui::Button::new(RichText::new(example).size(14.0).color(p.ink)).fill(p.chip))
                .clicked()
            {
                np.material_text = example.into();
            }
        }
    });
    ui.add_space(4.0);
    if add_box(ui, p, &mut np.material_text, "15 pine 2x4s, 3 foot") {
        np.add_material();
    }
    ui.add_space(6.0);
    let mut remove = None;
    for (i, item) in np.materials.iter().enumerate() {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(describe_item(item))
                    .size(15.0)
                    .strong()
                    .color(p.ink),
            );
            ui.label(
                RichText::new(format!("from \u{201c}{}\u{201d}", item.text))
                    .size(13.0)
                    .color(p.muted),
            );
            if np.material_is_global(item) {
                ui.label(RichText::new("your workshop").size(12.0).color(p.muted));
            } else {
                ui.label(
                    RichText::new("only for this project")
                        .size(12.0)
                        .strong()
                        .color(ACCENT),
                );
            }
            if ui.small_button("\u{d7}").on_hover_text("Remove").clicked() {
                remove = Some(i);
            }
        });
    }
    if let Some(i) = remove {
        np.materials.remove(i);
    }
}

fn budget_step(ui: &mut Ui, np: &mut NewProject, p: &Palette) {
    muted(ui, p, "Sancho warns at 80% and again at the cap.");
    ui.horizontal(|ui| {
        if ui
            .radio(
                np.budget == Some(BudgetPick::Cap),
                RichText::new("Up to").size(16.0),
            )
            .clicked()
        {
            np.budget = Some(BudgetPick::Cap);
        }
        let r = ui.add(
            egui::TextEdit::singleline(&mut np.budget_text)
                .hint_text("150")
                .desired_width(110.0)
                .font(egui::FontId::proportional(17.0)),
        );
        if r.changed() {
            np.budget = Some(BudgetPick::Cap);
        }
        ui.add(
            egui::TextEdit::singleline(&mut np.currency)
                .desired_width(54.0)
                .font(egui::FontId::proportional(17.0)),
        );
    });
    if ui
        .radio(
            np.budget == Some(BudgetPick::OnlyWhatIHave),
            RichText::new("Use only what I have").size(16.0),
        )
        .clicked()
    {
        np.budget = Some(BudgetPick::OnlyWhatIHave);
    }
    if ui
        .radio(
            np.budget == Some(BudgetPick::NotSure),
            RichText::new("Not sure").size(16.0),
        )
        .clicked()
    {
        np.budget = Some(BudgetPick::NotSure);
    }
}

/// "What your answers set up": the right-hand column.
fn set_up(ui: &mut Ui, np: &NewProject, interpretation: &Interpretation, p: &Palette) {
    card(ui, p, |ui| {
        ui.label(
            RichText::new("What your answers set up")
                .size(18.0)
                .strong()
                .color(p.ink),
        );
        ui.add_space(6.0);
        set_up_body(ui, np, interpretation, p);
    });
}

fn heading(ui: &mut Ui, p: &Palette, text: &str) {
    ui.add_space(8.0);
    ui.label(RichText::new(text).size(13.0).strong().color(p.muted));
}

fn set_up_body(ui: &mut Ui, np: &NewProject, interpretation: &Interpretation, p: &Palette) {
    heading(ui, p, "ENVIRONMENTS");
    for env in np.environments(interpretation) {
        ui.horizontal_wrapped(|ui| {
            let name = if env.question {
                format!("{}?", env_name(&env.key))
            } else {
                env_name(&env.key).to_string()
            };
            ui.label(RichText::new(name).size(15.0).strong().color(p.ink));
            ui.label(
                RichText::new(format!("\u{b7} {}", env.why))
                    .size(14.0)
                    .color(p.muted),
            );
        });
    }
    heading(ui, p, "SCALE");
    match np.scale(interpretation) {
        Some((scale, why)) => {
            ui.label(
                RichText::new(format!("{}, {why}.", scale.name()))
                    .size(15.0)
                    .strong()
                    .color(p.ink),
            );
            ui.add(
                egui::Label::new(RichText::new(scale.describe()).size(14.0).color(p.muted)).wrap(),
            );
        }
        None => {
            ui.add(
                egui::Label::new(
                    RichText::new("Not known yet: the stage follows the first part's size.")
                        .size(14.0)
                        .color(p.muted),
                )
                .wrap(),
            );
        }
    }
    heading(ui, p, "YOUR TOOLBOX WILL START WITH");
    let mut toolbox: Vec<String> = interpretation
        .tools
        .iter()
        .map(|t| tool_ui_name(&t.value))
        .collect();
    toolbox.dedup();
    if toolbox.is_empty() {
        toolbox.push("Sketch, Extrude, Fillet and the rest of Design's commands".into());
    }
    ui.add(
        egui::Label::new(
            RichText::new(toolbox.join(" \u{b7} "))
                .size(14.0)
                .color(p.ink),
        )
        .wrap(),
    );
    if !interpretation.templates.is_empty() {
        heading(ui, p, "STARTERS");
        let names: Vec<&str> = interpretation
            .templates
            .iter()
            .map(|t| t.value.as_str())
            .collect();
        ui.add(
            egui::Label::new(
                RichText::new(names.join(" \u{b7} "))
                    .size(14.0)
                    .color(p.ink),
            )
            .wrap(),
        );
    }
    let mut noticed: Vec<String> = interpretation
        .questions
        .iter()
        .map(|q| match q.value.as_str() {
            "space" => "Where will it go? If you measure the space I can check it fits.".into(),
            other => other.to_string(),
        })
        .collect();
    if np.tools.is_empty() && np.answer_for(&StepId::Tools).is_some() {
        noticed.push(
            "No tools yet: I'll suggest joinery for hand tools until you tell me more.".into(),
        );
    }
    if !noticed.is_empty() {
        heading(ui, p, "SANCHO NOTICED");
        for line in noticed {
            Frame::new()
                .fill(PAPER)
                .stroke(Stroke::new(1.5, SANCHO_INK))
                .corner_radius(CornerRadius::same(12))
                .inner_margin(Margin::symmetric(10, 6))
                .show(ui, |ui| {
                    ui.add(
                        egui::Label::new(RichText::new(line).size(14.0).color(SANCHO_INK)).wrap(),
                    );
                });
            ui.add_space(4.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates_and_money_read_the_way_people_type_them() {
        assert_eq!(
            parse_date("2026-12-24"),
            Some(Date {
                year: 2026,
                month: 12,
                day: 24
            })
        );
        assert_eq!(parse_date("2026/2/30"), None);
        assert_eq!(parse_date("next week"), None);
        assert_eq!(parse_money("150"), Some(15000));
        assert_eq!(parse_money("$1,200.5"), Some(120050));
        assert_eq!(parse_money("149.99"), Some(14999));
        assert_eq!(parse_money("12.345"), None);
        assert_eq!(parse_money("lots"), None);
    }

    #[test]
    fn a_dresser_quest_sets_up_room_scale_stock_and_plan() {
        let mut np = NewProject::new(false);
        np.name = "Alcove dresser".into();
        np.advance().unwrap();
        np.goal = "A dresser for the bedroom alcove".into();
        np.advance().unwrap();
        np.fit = Some(FitPick::DecideLater);
        np.advance().unwrap();
        np.add_kit("starter_woodshop");
        assert!(!np.tools.is_empty());
        np.advance().unwrap();
        np.material_text = "15 pine 2x4s, 3 foot".into();
        np.add_material();
        assert_eq!(np.materials.len(), 1);
        np.advance().unwrap();
        np.deadline_text = "2026-12-24".into();
        np.advance().unwrap();
        np.budget = Some(BudgetPick::Cap);
        np.budget_text = "150".into();
        np.advance().unwrap();
        np.skip();
        assert_eq!(np.step().id, StepId::Review);

        let interpretation = np.interpretation();
        let envs: Vec<String> = np
            .environments(&interpretation)
            .into_iter()
            .map(|e| e.key)
            .collect();
        assert!(envs.contains(&"stock".to_string()), "{envs:?}");
        assert!(envs.contains(&"plan".to_string()), "{envs:?}");
        assert_eq!(np.scale(&interpretation).map(|s| s.0), Some(Scale::Room));

        let created = np.finish(false).unwrap();
        assert_eq!(created.name, "Alcove dresser");
        assert_eq!(created.environments[0], "design");
        assert!(created.environments.contains(&"stock".to_string()));
        assert!(!created.environments.contains(&"solid".to_string()));
        assert_eq!(
            created.deadline,
            Some(Date {
                year: 2026,
                month: 12,
                day: 24
            })
        );
        assert_eq!(created.budget.as_ref().map(|b| b.cap_minor), Some(15000));
        assert_eq!(created.outstanding, vec![StepId::Inspiration]);
        let inventory: serde_json::Value = serde_json::from_slice(&created.inventory).unwrap();
        assert_eq!(inventory["materials"][0]["said"], "15 pine 2x4s, 3 foot");
        assert!(inventory["tools"].as_array().is_some_and(|t| !t.is_empty()));
    }

    #[test]
    fn skip_to_designing_makes_a_project_from_nothing() {
        let mut np = NewProject::new(true);
        let created = np.finish(true).unwrap();
        assert_eq!(created.name, "My project");
        assert_eq!(created.environments, vec!["design".to_string()]);
        assert!(created.outstanding.contains(&StepId::Goal));
        assert_eq!(created.scale, None);
    }

    #[test]
    fn a_typed_size_sets_the_scale_and_a_bad_date_is_refused() {
        let mut np = NewProject::new(true);
        np.fit = Some(FitPick::TypeSizes);
        np.size_text = "6 in".into();
        let interpretation = np.interpretation();
        assert_eq!(np.scale(&interpretation).map(|s| s.0), Some(Scale::Bench));
        np.go_to(5);
        assert_eq!(np.step().id, StepId::Deadline);
        np.deadline_text = "soon".into();
        assert!(np.advance().is_err());
        assert_eq!(np.step().id, StepId::Deadline);
        np.deadline_text.clear();
        np.no_deadline = true;
        np.advance().unwrap();
        assert_eq!(
            np.answer_for(&StepId::Deadline),
            Some(&Answer::Deadline(None))
        );
    }

    #[test]
    fn tools_come_from_words_and_kits_once_each() {
        let mut np = NewProject::new(false);
        np.tool_text = "a jigsaw".into();
        np.add_tool_words();
        assert_eq!(np.tools.len(), 1, "{:?}", np.hint);
        np.tool_text = "jigsaws".into();
        np.add_tool_words();
        assert_eq!(np.tools.len(), 1);
        np.tool_text = "a flux capacitor".into();
        np.add_tool_words();
        assert_eq!(np.tools.len(), 1);
        assert!(np.hint.as_deref().is_some_and(|h| h.contains("don't know")));
        np.add_kit("basic_household");
        let n = np.tools.len();
        np.add_kit("basic_household");
        assert_eq!(np.tools.len(), n);
    }

    /// A library with a jigsaw (owned), a drill (shared), a drill press (wishlist), a tool the
    /// taxonomy doesn't know, and two material lines of which one is used up.
    fn sample_library() -> Library {
        let library = Library::in_memory().unwrap();
        library.add_tool("jigsaw", "", Access::Owned, None).unwrap();
        library
            .add_tool("drill_driver", "", Access::Shared, None)
            .unwrap();
        library
            .add_tool("drill_press", "", Access::Wishlist, None)
            .unwrap();
        library
            .add_tool("flux_capacitor", "", Access::Owned, None)
            .unwrap();
        library.add_material("15 pine 2x4s, 3 foot").unwrap();
        let used_up = library.add_material("half sheet 3/4 ply").unwrap();
        library.set_on_hand(used_up.id, false).unwrap();
        library
    }

    #[test]
    fn the_workshop_offers_what_the_maker_has_and_counts_what_it_leaves_out() {
        let workshop = Workshop::read(&sample_library()).unwrap();
        assert_eq!(workshop.tools, vec!["jigsaw", "drill_driver"]);
        assert_eq!(
            workshop.left_out, 2,
            "the wishlist tool and the unknown one"
        );
        assert_eq!(workshop.materials.len(), 1);
        assert_eq!(workshop.materials[0].text, "15 pine 2x4s, 3 foot");
    }

    #[test]
    fn using_the_workshop_adds_only_what_the_project_lacks_and_only_once() {
        let mut np = NewProject::new(false);
        np.workshop = Workshop::read(&sample_library()).unwrap();
        np.tool_text = "a jigsaw".into();
        np.add_tool_words();
        assert_eq!(np.workshop_tools_missing(), vec!["drill_driver"]);
        np.use_workshop_tools();
        assert_eq!(np.tools.len(), 2);
        assert!(np.workshop_tools_missing().is_empty());
        np.use_workshop_tools();
        assert_eq!(np.tools.len(), 2, "a second press adds nothing");
        np.use_workshop_materials();
        np.use_workshop_materials();
        assert_eq!(np.materials.len(), 1);
        assert!(np.workshop_materials_missing().is_empty());
    }

    #[test]
    fn the_snapshot_says_which_items_are_the_workshops_and_which_the_projects_own() {
        let mut np = NewProject::new(false);
        np.workshop = Workshop::read(&sample_library()).unwrap();
        np.use_workshop_tools();
        np.use_workshop_materials();
        np.tool_text = "a band saw".into();
        np.add_tool_words();
        np.material_text = "2 yards 500D Cordura".into();
        np.add_material();
        let snapshot: serde_json::Value = serde_json::from_slice(&np.inventory_json()).unwrap();
        assert_eq!(snapshot["version"], 2);
        let scope_of = |list: &str, key: &str, name: &str| -> String {
            snapshot[list]
                .as_array()
                .unwrap()
                .iter()
                .find(|e| e[key] == name)
                .unwrap_or_else(|| panic!("{name} is not in {list}"))["scope"]
                .as_str()
                .unwrap()
                .to_string()
        };
        assert_eq!(scope_of("tools", "kind", "jigsaw"), "global");
        assert_eq!(scope_of("tools", "kind", "band_saw"), "local");
        assert_eq!(
            scope_of("materials", "said", "15 pine 2x4s, 3 foot"),
            "global"
        );
        assert_eq!(
            scope_of("materials", "said", "2 yards 500D Cordura"),
            "local"
        );
    }

    #[test]
    fn nothing_typed_into_the_quest_reaches_the_workshop_on_its_own() {
        let library = sample_library();
        let before = (
            library.tools().unwrap().len(),
            library.materials().unwrap().len(),
        );
        let mut np = NewProject::new(false);
        np.workshop = Workshop::read(&library).unwrap();
        np.tool_text = "a band saw".into();
        np.add_tool_words();
        np.material_text = "2 yards 500D Cordura".into();
        np.add_material();
        np.finish(true).unwrap();
        let after = (
            library.tools().unwrap().len(),
            library.materials().unwrap().len(),
        );
        assert_eq!(before, after);
    }

    #[test]
    fn a_first_run_with_no_folder_opens_an_empty_workshop() {
        let dir = std::env::temp_dir().join(format!("makercad-workshop-{}", std::process::id()));
        let path = dir.join("library.db");
        assert_eq!(Workshop::open(&path).unwrap(), Workshop::default());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
