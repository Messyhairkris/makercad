//! Sancho's core rule pack (doc 07 section 7.5): always loaded.
//!
//! Written by Deepak (the Crew, campaign mk-core-17, 2026-11-22) from the contract
//! crew/contracts/mk_rules_core.md (ATLAS-api-harness), which Claude finished from Deepak's brief.

use std::collections::HashSet;

use crate::findings::{Finding, Kind, Location, Severity, Source};

/// Rule id: the project has no goal to check the design against.
pub const RULE_INTENT_MISSING: &str = "core.intent_missing";
/// Rule id: a skipped prompt matters to what the maker is doing now.
pub const RULE_PROMPT_SKIPPED: &str = "core.prompt_skipped";
/// Rule id: a size looks like the wrong unit.
pub const RULE_UNIT_SANITY: &str = "core.unit_sanity";

/// The rule ids in the order `evaluate` runs them.
pub const RULES: [&str; 3] = [RULE_INTENT_MISSING, RULE_PROMPT_SKIPPED, RULE_UNIT_SANITY];

/// Doc 05 section 5.5's scale classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScaleClass {
    Room,
    Furniture,
    Body,
    Bench,
    Macro,
}

/// A prompt moment the maker skipped, and whether it matters now (the app decides that).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prompt {
    pub id: String,
    pub skipped: bool,
    pub relevant_now: bool,
}

/// One part of the design: its name and its box in mm (any order of sides).
#[derive(Debug, Clone, PartialEq)]
pub struct Part {
    pub name: String,
    pub size_mm: [f64; 3],
}

/// What the core pack reads.
#[derive(Debug, Clone, PartialEq)]
pub struct ProjectFacts {
    /// The intent's goal text; None when the project has no intent at all.
    pub goal: Option<String>,
    pub scale: ScaleClass,
    pub prompts: Vec<Prompt>,
    pub parts: Vec<Part>,
}

impl ScaleClass {
    /// The class's key, as findings' facts write it.
    pub fn key(self) -> &'static str {
        match self {
            ScaleClass::Room => "room",
            ScaleClass::Furniture => "furniture",
            ScaleClass::Body => "body",
            ScaleClass::Bench => "bench",
            ScaleClass::Macro => "macro",
        }
    }

    /// The class's typical size range in mm, inclusive (doc 05 section 5.5).
    pub fn range_mm(self) -> (f64, f64) {
        match self {
            ScaleClass::Room => (500.0, 10000.0),
            ScaleClass::Furniture => (300.0, 2500.0),
            ScaleClass::Body => (100.0, 1000.0),
            ScaleClass::Bench => (10.0, 500.0),
            ScaleClass::Macro => (0.0, 10.0),
        }
    }
}

/// The largest of a part's three sides in mm, or None when any side is not finite or not
/// positive (such a part is ignored by every rule).
pub fn largest_mm(part: &Part) -> Option<f64> {
    let mut largest = f64::NEG_INFINITY;
    for &side in &part.size_mm {
        if !side.is_finite() || side <= 0.0 {
            return None;
        }
        if side > largest {
            largest = side;
        }
    }
    Some(largest)
}

/// Suggests a scale class from one size in mm; Body is about wearing, so it is never suggested.
pub fn class_for_mm(mm: f64) -> Option<ScaleClass> {
    if !mm.is_finite() || mm <= 0.0 {
        return None;
    }
    if mm < 10.0 {
        Some(ScaleClass::Macro)
    } else if mm <= 500.0 {
        Some(ScaleClass::Bench)
    } else if mm <= 2500.0 {
        Some(ScaleClass::Furniture)
    } else {
        Some(ScaleClass::Room)
    }
}

/// The key of the class `class_for_mm` suggests for `mm`, or "" when it suggests none.
fn suggested_key(mm: f64) -> &'static str {
    match class_for_mm(mm) {
        Some(class) => class.key(),
        None => "",
    }
}

/// R1: one Tip when the project has no goal, or a goal that is empty after trim.
pub fn intent_missing(facts: &ProjectFacts) -> Vec<Finding> {
    let missing = match facts.goal.as_deref() {
        None => true,
        Some(goal) => goal.trim().is_empty(),
    };
    if !missing {
        return Vec::new();
    }
    vec![Finding {
        kind: Kind::Tip,
        severity: Severity::Warning,
        source: Source::Rule(RULE_INTENT_MISSING.to_string()),
        location: Location::Field("intent.goal".to_string()),
        what: "No goal yet".to_string(),
        why: "Sancho checks the design against its goal, and there is none to check against."
            .to_string(),
        fix: Some("intent.edit".to_string()),
        facts: vec![("goal".to_string(), "missing".to_string())],
    }]
}

/// R2: one Tip per skipped prompt that matters now, in input order, first id wins.
pub fn prompt_skipped(facts: &ProjectFacts) -> Vec<Finding> {
    let mut findings = Vec::new();
    let mut seen: HashSet<&str> = HashSet::new();
    for prompt in &facts.prompts {
        if !seen.insert(prompt.id.as_str()) {
            continue;
        }
        if prompt.skipped && prompt.relevant_now {
            findings.push(Finding {
                kind: Kind::Tip,
                severity: Severity::Info,
                source: Source::Rule(RULE_PROMPT_SKIPPED.to_string()),
                location: Location::Field(format!("prompt.{}", prompt.id)),
                what: "A skipped question matters now".to_string(),
                why: "You skipped this question earlier, and what you are doing now depends on it."
                    .to_string(),
                fix: Some("prompt.answer".to_string()),
                facts: vec![("prompt".to_string(), prompt.id.clone())],
            });
        }
    }
    findings
}

/// R3: unit sanity, R3a (a part too big for the project) then R3b (the whole design far too
/// small). Small parts on their own are never flagged.
pub fn unit_sanity(facts: &ProjectFacts) -> Vec<Finding> {
    let mut findings = Vec::new();
    let (lower, upper) = facts.scale.range_mm();
    let class_key = facts.scale.key();

    // R3a: parts too big for the project, in input order, first name wins.
    let mut seen: HashSet<&str> = HashSet::new();
    let mut design_size: Option<f64> = None;
    for part in &facts.parts {
        if !seen.insert(part.name.as_str()) {
            continue;
        }
        let size = match largest_mm(part) {
            Some(size) => size,
            None => continue,
        };
        design_size = Some(match design_size {
            Some(current) => current.max(size),
            None => size,
        });
        if size > upper {
            findings.push(Finding {
                kind: Kind::Tip,
                severity: Severity::Warning,
                source: Source::Rule(RULE_UNIT_SANITY.to_string()),
                location: Location::Entity(part.name.clone()),
                what: format!("{} is {} mm", part.name, size),
                why: format!(
                    "That is large for a {} project (up to {} mm). Check the units.",
                    class_key, upper
                ),
                fix: Some("project.scale".to_string()),
                facts: vec![
                    ("part".to_string(), part.name.clone()),
                    ("largest_mm".to_string(), size.to_string()),
                    ("class".to_string(), class_key.to_string()),
                    ("suggested".to_string(), suggested_key(size).to_string()),
                ],
            });
        }
    }

    // R3b: the whole design far too small; never for Macro.
    if facts.scale != ScaleClass::Macro {
        if let Some(size) = design_size {
            if size < lower / 10.0 {
                findings.push(Finding {
                    kind: Kind::Tip,
                    severity: Severity::Warning,
                    source: Source::Rule(RULE_UNIT_SANITY.to_string()),
                    location: Location::Project,
                    what: format!("The whole design is {} mm", size),
                    why: format!(
                        "That is small for a {} project (from {} mm). Check the units.",
                        class_key, lower
                    ),
                    fix: Some("project.scale".to_string()),
                    facts: vec![
                        ("largest_mm".to_string(), size.to_string()),
                        ("class".to_string(), class_key.to_string()),
                        ("suggested".to_string(), suggested_key(size).to_string()),
                    ],
                });
            }
        }
    }

    findings
}

/// E1: R1's findings, then R2's, then R3a's, then R3b's.
pub fn evaluate(facts: &ProjectFacts) -> Vec<Finding> {
    let mut findings = intent_missing(facts);
    findings.extend(prompt_skipped(facts));
    findings.extend(unit_sanity(facts));
    findings
}
