//! The Intent card (doc 05 section 5.2, doc 04 section 4.2): items, what is open, progress.
//!
//! Written by Hoss (the Crew, campaign mk-core-31, 2026-11-22) from the contract
//! crew/contracts/mk_intent_card.md (ATLAS-api-harness), which Gordon wrote himself as a PA (the contract, the reference and the hidden tests).

/// How one card item stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemState {
    Done,
    Outstanding,
}

/// One line of the Intent card.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub id: String,
    pub label: String,
    pub state: ItemState,
    pub detail: Option<String>,
}

/// The card: whether the intent is missing, and its items in card order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Card {
    pub intent_missing: bool,
    pub items: Vec<Item>,
}

/// What the project holds. Blank or whitespace-only text counts as none.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Facts {
    pub name: Option<String>,
    pub goal: Option<String>,
    pub fit_known: bool,
    pub tools: usize,
    pub materials: usize,
    pub deadline: Option<String>,
    pub no_deadline: bool,
    pub budget: Option<String>,
    pub inspiration: usize,
}

/// The card item ids, in the order `build` writes them.
pub const ID_NAME: &str = "name";
pub const ID_GOAL: &str = "goal";
pub const ID_FIT: &str = "fit";
pub const ID_TOOLS: &str = "tools";
pub const ID_MATERIALS: &str = "materials";
pub const ID_DEADLINE: &str = "deadline";
pub const ID_BUDGET: &str = "budget";
pub const ID_INSPIRATION: &str = "inspiration";

/// The card item ids, in card order.
pub const IDS: [&str; 8] = [
    ID_NAME,
    ID_GOAL,
    ID_FIT,
    ID_TOOLS,
    ID_MATERIALS,
    ID_DEADLINE,
    ID_BUDGET,
    ID_INSPIRATION,
];

/// The fit item's detail when the space is known (v1 starting value, to tune).
pub const DETAIL_FIT: &str = "Size known";
/// The deadline item's detail when no deadline was chosen (v1 starting value, to tune).
pub const DETAIL_NO_DEADLINE: &str = "No deadline";
/// The noun after a count of 1 (v1 starting value, to tune).
pub const NOUN_TOOL: &str = "tool";
/// The noun after any other count (v1 starting value, to tune).
pub const NOUN_TOOLS: &str = "tools";
/// The noun after a count of 1 (v1 starting value, to tune).
pub const NOUN_MATERIAL: &str = "material";
/// The noun after any other count (v1 starting value, to tune).
pub const NOUN_MATERIALS: &str = "materials";
/// The noun after a count of 1 (v1 starting value, to tune).
pub const NOUN_INSPIRATION: &str = "inspiration";
/// The noun after any other count (v1 starting value, to tune).
pub const NOUN_INSPIRATIONS: &str = "inspirations";
/// The headline when the goal is blank.
pub const HEADLINE_INTENT_MISSING: &str = "Intent missing: tell Sancho what you want to make";
/// The headline when nothing is outstanding.
pub const HEADLINE_ALL_SET: &str = "All set";
/// The headline when exactly one item is outstanding.
pub const HEADLINE_ONE_OPEN: &str = "1 question still open";
/// The headline when two or more items are outstanding; `{n}` is the count in `{}`.
pub const HEADLINE_MANY_OPEN: &str = "{n} questions still open";

/// Returns true if `text` is empty or consists only of whitespace.
pub fn is_blank(text: &str) -> bool {
    text.is_empty() || text.chars().all(|c| c.is_whitespace())
}

/// Build a `Card` from the supplied `Facts`.
pub fn build(facts: &Facts) -> Card {
    // intent_missing depends only on the goal field.
    let intent_missing = match &facts.goal {
        Some(s) => is_blank(s),
        None => true,
    };

    // Helper to get trimmed text if known.
    fn known_text(opt: &Option<String>) -> Option<String> {
        opt.as_ref()
            .filter(|s| !is_blank(s))
            .map(|s| s.trim().to_string())
    }

    // Helper for count detail.
    fn count_detail(count: usize, singular: &str, plural: &str) -> String {
        let noun = if count == 1 { singular } else { plural };
        format!("{} {}", count, noun)
    }

    // Mapping from id to label.
    fn label_for_id(id: &str) -> &'static str {
        match id {
            ID_NAME => "Name",
            ID_GOAL => "What to make",
            ID_FIT => "Where it fits",
            ID_TOOLS => "Your tools",
            ID_MATERIALS => "Your materials",
            ID_DEADLINE => "When you need it",
            ID_BUDGET => "Budget",
            ID_INSPIRATION => "What inspires you",
            _ => "",
        }
    }

    let mut items = Vec::with_capacity(IDS.len());

    for &id in IDS.iter() {
        let (state, detail) = match id {
            ID_NAME => {
                if let Some(txt) = known_text(&facts.name) {
                    (ItemState::Done, Some(txt))
                } else {
                    (ItemState::Outstanding, None)
                }
            }
            ID_GOAL => {
                if let Some(txt) = known_text(&facts.goal) {
                    (ItemState::Done, Some(txt))
                } else {
                    (ItemState::Outstanding, None)
                }
            }
            ID_FIT => {
                if facts.fit_known {
                    (ItemState::Done, Some(DETAIL_FIT.to_string()))
                } else {
                    (ItemState::Outstanding, None)
                }
            }
            ID_TOOLS => {
                if facts.tools > 0 {
                    let d = count_detail(facts.tools, NOUN_TOOL, NOUN_TOOLS);
                    (ItemState::Done, Some(d))
                } else {
                    (ItemState::Outstanding, None)
                }
            }
            ID_MATERIALS => {
                if facts.materials > 0 {
                    let d = count_detail(facts.materials, NOUN_MATERIAL, NOUN_MATERIALS);
                    (ItemState::Done, Some(d))
                } else {
                    (ItemState::Outstanding, None)
                }
            }
            ID_DEADLINE => {
                if let Some(txt) = known_text(&facts.deadline) {
                    (ItemState::Done, Some(txt))
                } else if facts.no_deadline {
                    (ItemState::Done, Some(DETAIL_NO_DEADLINE.to_string()))
                } else {
                    (ItemState::Outstanding, None)
                }
            }
            ID_BUDGET => {
                if let Some(txt) = known_text(&facts.budget) {
                    (ItemState::Done, Some(txt))
                } else {
                    (ItemState::Outstanding, None)
                }
            }
            ID_INSPIRATION => {
                if facts.inspiration > 0 {
                    let d = count_detail(facts.inspiration, NOUN_INSPIRATION, NOUN_INSPIRATIONS);
                    (ItemState::Done, Some(d))
                } else {
                    (ItemState::Outstanding, None)
                }
            }
            _ => (ItemState::Outstanding, None),
        };

        items.push(Item {
            id: id.to_string(),
            label: label_for_id(id).to_string(),
            state,
            detail,
        });
    }

    Card {
        intent_missing,
        items,
    }
}

/// Return a vector of references to the outstanding items, in card order.
pub fn outstanding(card: &Card) -> Vec<&Item> {
    card.items
        .iter()
        .filter(|it| it.state == ItemState::Outstanding)
        .collect()
}

/// Return the first outstanding item, or `None` if none.
pub fn next_outstanding(card: &Card) -> Option<&Item> {
    outstanding(card).into_iter().next()
}

/// Return a tuple `(done_count, total_items)`.
pub fn progress(card: &Card) -> (usize, usize) {
    let done = card
        .items
        .iter()
        .filter(|it| it.state == ItemState::Done)
        .count();
    (done, card.items.len())
}

/// Produce the appropriate headline for the card.
pub fn headline(card: &Card) -> String {
    if card.intent_missing {
        return HEADLINE_INTENT_MISSING.to_string();
    }

    let open_count = card
        .items
        .iter()
        .filter(|it| it.state == ItemState::Outstanding)
        .count();

    match open_count {
        0 => HEADLINE_ALL_SET.to_string(),
        1 => HEADLINE_ONE_OPEN.to_string(),
        n => HEADLINE_MANY_OPEN.replace("{n}", &n.to_string()),
    }
}
