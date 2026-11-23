//! Sancho's early observations (doc 05 sections 5.2 and 5.4): rules over the goal, tools, materials and scale.
//!
//! Written by Deepak (the Crew, campaign mk-core-32, 2026-11-22) from the contract
//! crew/contracts/mk_observations.md (ATLAS-api-harness), which Gordon wrote himself as a PA (the contract, the reference and the hidden tests).

/// The rule id of O1, the joinery advice.
pub const RULE_JOINERY: &str = "obs.joinery";
/// The rule id of O2, the knock-down question.
pub const RULE_KNOCK_DOWN: &str = "obs.knock_down";
/// The rule id of O3, the household-kit tip.
pub const RULE_HOUSEHOLD_KIT: &str = "obs.household_kit";
/// The rule id of O4, the materials question.
pub const RULE_MATERIALS: &str = "obs.materials";

/// The rule ids, in the order their observations are emitted.
pub const RULES: [&str; 4] = [
    RULE_JOINERY,
    RULE_KNOCK_DOWN,
    RULE_HOUSEHOLD_KIT,
    RULE_MATERIALS,
];

/// The cut capabilities a furniture goal needs, in the order O1 tries them.
pub const NEEDED_CUTS: [&str; 2] = ["cut.rip", "cut.crosscut"];

/// The furniture words O1 looks for, in tie-break order.
const FURNITURE_WORDS: [&str; 4] = ["dresser", "shelf", "desk", "cabinet"];

/// The words and phrases O2 looks for, as the contract lists them.
const KNOCK_DOWN_WORDS: [&str; 18] = [
    "come apart",
    "comes apart",
    "take apart",
    "takes apart",
    "disassemble",
    "dismantle",
    "knock down",
    "knockdown",
    "flat pack",
    "flatpack",
    "flat packed",
    "portable",
    "move",
    "moved",
    "moving",
    "movable",
    "moveable",
    "transport",
];

/// How the maker has a tool (doc 04 section 4.3.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Access {
    Owned,
    Shared,
    Makerspace,
    Rentable,
    Wishlist,
}

/// The scale class of the project (doc 05 section 5.5): the classes the docs name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Scale {
    Bench,
    Body,
    Room,
}

/// What an observation is: advice, a question or a tip.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    Advice,
    Question,
    Tip,
}

/// One capability the maker has, the tool kind that supplies it, and its access.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tool {
    /// The tool kind from the taxonomy: "jigsaw", "table saw".
    pub kind: String,
    /// The capability as the tool record writes it: "cut.rip", "cut.rip:rough".
    pub capability: String,
    pub access: Access,
}

/// What the observations read.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ObservationFacts {
    pub goal: String,
    pub scale: Option<Scale>,
    pub tools: Vec<Tool>,
    pub materials: Vec<String>,
}

/// One early observation shown on the review card.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Observation {
    /// The rule that produced it: one of `RULES`.
    pub rule: &'static str,
    pub kind: Kind,
    /// One line of plain words from a template.
    pub text: String,
    /// Why Sancho says it.
    pub why: String,
}

/// The capability family: the text before the first ':', or all of it.
pub fn family(supply: &str) -> &str {
    match supply.split_once(':') {
        Some((head, _)) => head,
        None => supply,
    }
}

/// The capability qualifier: the text after the first ':', or "".
pub fn qualifier(supply: &str) -> &str {
    match supply.split_once(':') {
        Some((_, tail)) => tail,
        None => "",
    }
}

/// Whether the maker can use a tool now: true for Owned, Shared, Makerspace and Rentable.
pub fn usable(access: Access) -> bool {
    match access {
        Access::Owned | Access::Shared | Access::Makerspace | Access::Rentable => true,
        Access::Wishlist => false,
    }
}

/// Whether `needle` appears in `text` as whole words, case-insensitively, simple plurals allowed.
pub fn mentions(text: &str, needle: &str) -> bool {
    let text_runs = runs(text);
    let needle_runs = runs(needle);
    first_match_index(&text_runs, &needle_runs).is_some()
}

/// The observations, in rule order O1..O4.
pub fn evaluate(facts: &ObservationFacts) -> Vec<Observation> {
    let mut out: Vec<Observation> = Vec::new();
    let goal_runs = runs(&facts.goal);

    push_joinery(facts, &goal_runs, &mut out);

    // O2 reads only the goal: tools, materials and scale never change it.
    if KNOCK_DOWN_WORDS
        .iter()
        .any(|needle| first_match_index(&goal_runs, &runs(needle)).is_some())
    {
        out.push(Observation {
            rule: RULE_KNOCK_DOWN,
            kind: Kind::Question,
            text: "Should I prefer knock-down joints (cam locks) so it comes apart for moving?"
                .to_string(),
            why: "The goal mentions moving or coming apart, so knock-down joints are worth \
                  asking about."
                .to_string(),
        });
    }

    // O3: a list holding only wishlist tools is not empty, so it stays quiet.
    if facts.tools.is_empty() {
        out.push(Observation {
            rule: RULE_HOUSEHOLD_KIT,
            kind: Kind::Tip,
            text: "No tools listed: the basic household kit is the starting point \
                   (tape, utility knife, screwdrivers, hammer, scissors)."
                .to_string(),
            why: "You have not listed any tools yet, so start from the basic household kit."
                .to_string(),
        });
    }

    // O4: blank entries do not count as listed materials.
    if !facts.materials.iter().any(|m| !m.trim().is_empty()) {
        out.push(Observation {
            rule: RULE_MATERIALS,
            kind: Kind::Question,
            text: "I can't check material use yet - tell me what you have?".to_string(),
            why: "No materials are listed, so material use is unknown.".to_string(),
        });
    }

    out
}

/// O1: the joinery advice, one observation per needed cut that only a rough tool covers.
fn push_joinery(facts: &ObservationFacts, goal_runs: &[String], out: &mut Vec<Observation>) {
    // A4: a bench- or body-scale goal never gets O1.
    if !matches!(facts.scale, None | Some(Scale::Room)) {
        return;
    }
    let w = match first_furniture_word(goal_runs) {
        Some(word) => word,
        None => return,
    };
    for c in NEEDED_CUTS {
        let mut exact = false;
        let mut rough_kind: Option<&str> = None;
        for tool in &facts.tools {
            if !usable(tool.access) || family(&tool.capability) != c {
                continue;
            }
            if qualifier(&tool.capability) == "rough" {
                if rough_kind.is_none() {
                    rough_kind = Some(tool.kind.as_str());
                }
            } else {
                exact = true;
            }
        }
        if exact {
            continue;
        }
        let k = match rough_kind {
            Some(kind) => kind,
            None => continue,
        };
        out.push(Observation {
            rule: RULE_JOINERY,
            kind: Kind::Advice,
            text: format!(
                "You listed a {k} but no tool for {c}; I'll favour joinery a {k} can do."
            ),
            why: format!(
                "The goal names a {w}, which needs {c}, and the {k} you listed only does it rough."
            ),
        });
    }
}

/// The furniture word that appears first in the goal; a tie goes to the earlier list word.
fn first_furniture_word(goal_runs: &[String]) -> Option<&'static str> {
    let mut best: Option<(usize, &'static str)> = None;
    for word in FURNITURE_WORDS {
        if let Some(idx) = first_match_index(goal_runs, &runs(word)) {
            let better = match best {
                Some((best_idx, _)) => idx < best_idx,
                None => true,
            };
            if better {
                best = Some((idx, word));
            }
        }
    }
    best.map(|(_, word)| word)
}

/// The run index where the needle's runs first appear consecutively in the text's runs.
fn first_match_index(text_runs: &[String], needle_runs: &[String]) -> Option<usize> {
    if needle_runs.is_empty() || text_runs.len() < needle_runs.len() {
        return None;
    }
    let last = needle_runs.len() - 1;
    for start in 0..=(text_runs.len() - needle_runs.len()) {
        if text_runs[start..start + last] != needle_runs[..last] {
            continue;
        }
        if last_run_matches(&text_runs[start + last], &needle_runs[last]) {
            return Some(start);
        }
    }
    None
}

/// A3: the last run may equal the needle's last run or one of its simple plurals.
fn last_run_matches(text_run: &str, needle_last: &str) -> bool {
    if text_run == needle_last {
        return true;
    }
    let mut candidates: Vec<String> = Vec::new();
    candidates.push(format!("{needle_last}s"));
    candidates.push(format!("{needle_last}es"));
    if let Some(stem) = needle_last.strip_suffix('y') {
        candidates.push(format!("{stem}ies"));
    }
    if let Some(stem) = needle_last.strip_suffix("fe") {
        candidates.push(format!("{stem}ves"));
    } else if let Some(stem) = needle_last.strip_suffix('f') {
        candidates.push(format!("{stem}ves"));
    }
    candidates
        .iter()
        .any(|candidate| candidate.as_str() == text_run)
}

/// A3: lowercase, then split into runs of alphanumeric characters.
fn runs(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut cur = String::new();
    for ch in text.to_lowercase().chars() {
        if ch.is_alphanumeric() {
            cur.push(ch);
        } else if !cur.is_empty() {
            out.push(cur.clone());
            cur.clear();
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}
