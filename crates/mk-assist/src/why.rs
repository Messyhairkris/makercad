//! Sancho's "Why?" derivations (doc 07 section 7.6): a finding's inputs, rule and source.
//!
//! Written by Deepak (the Crew, campaign mk-core-26, 2026-11-22) from the contract
//! crew/contracts/mk_why.md (ATLAS-api-harness), which Gordon wrote himself as a PA (the contract, the reference and the hidden tests).

use crate::findings::{Finding, Source};

/// The rule id of the intent-missing rule.
pub const RULE_INTENT_MISSING: &str = "core.intent_missing";
/// The rule id of the prompt-skipped rule.
pub const RULE_PROMPT_SKIPPED: &str = "core.prompt_skipped";
/// The rule id of the unit-sanity rule.
pub const RULE_UNIT_SANITY: &str = "core.unit_sanity";
/// The rule id of the missing-tool rule.
pub const RULE_TOOLS_MISSING: &str = "tools.missing";
/// The rule id of the tool-away rule.
pub const RULE_TOOLS_AWAY: &str = "tools.away";
/// The rule id of the tool-accuracy rule.
pub const RULE_TOOLS_ACCURACY: &str = "tools.accuracy";
/// The rule id of the loud-tool rule.
pub const RULE_TOOLS_NOISE: &str = "tools.noise";
/// The rule id of the shelf-sag rule.
pub const RULE_SAG: &str = "structure.sag";
/// The rule id of the screw-through rule.
pub const RULE_SCREW_THROUGH: &str = "structure.screw_through";
/// The rule id of the screw-too-short rule.
pub const RULE_SCREW_SHORT: &str = "structure.screw_short";
/// The rule id of the edge-distance rule.
pub const RULE_EDGE_DISTANCE: &str = "structure.edge_distance";
/// The rule id of the unsupported-part rule.
pub const RULE_UNSUPPORTED: &str = "structure.unsupported";
/// The rule id of the thin-wall rule.
pub const RULE_THIN_WALL: &str = "structure.thin_wall";
/// The rule id of the slot-severs rule.
pub const RULE_SLOT_SEVERS: &str = "structure.slot_severs";

/// The rule ids that have an entry in the table, in the table's order.
pub const RULES: [&str; 14] = [
    RULE_INTENT_MISSING,
    RULE_PROMPT_SKIPPED,
    RULE_UNIT_SANITY,
    RULE_TOOLS_MISSING,
    RULE_TOOLS_AWAY,
    RULE_TOOLS_ACCURACY,
    RULE_TOOLS_NOISE,
    RULE_SAG,
    RULE_SCREW_THROUGH,
    RULE_SCREW_SHORT,
    RULE_EDGE_DISTANCE,
    RULE_UNSUPPORTED,
    RULE_THIN_WALL,
    RULE_SLOT_SEVERS,
];

/// The doc 07 section 7.6 source line for the sag table (structure.sag).
pub const SAG_SOURCE: &str = "Sag table: American Wood Council span data for pine, 30 kg/m load";

/// The title of the derivation for a Mind finding or an unknown rule id.
pub const GENERIC_TITLE: &str = "Not a built-in rule";

/// The rule words of the derivation for a Mind finding or an unknown rule id.
pub const GENERIC_RULE: &str = "This finding does not come from a built-in rule, so Sancho has no formula or rule table to show; the inputs below are the facts the finding carried.";

/// What the app draws for "Why?": the inputs, the rule in words and where it comes from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Derivation {
    pub title: String,
    /// The finding's facts, in the finding's order, each labelled by the table.
    pub inputs: Vec<(String, String)>,
    pub rule: String,
    pub source: Option<String>,
}

/// What the table knows about one rule id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuleInfo {
    pub id: &'static str,
    pub title: &'static str,
    pub rule: &'static str,
}

/// The table of W0, in the order of `RULES`.
static TABLE: [RuleInfo; 14] = [
    RuleInfo {
        id: RULE_INTENT_MISSING,
        title: "Intent missing",
        rule: "Sancho starts from your intent: what the piece is for and what done looks like. Until that is written down, no rule pack can judge the plan, so this finding asks for it.",
    },
    RuleInfo {
        id: RULE_PROMPT_SKIPPED,
        title: "Prompt skipped",
        rule: "A question that decides a later step was skipped. Sancho asks it again instead of guessing, because a guessed answer would spread through every rule that depends on it.",
    },
    RuleInfo {
        id: RULE_UNIT_SANITY,
        title: "Unit sanity",
        rule: "Every length is checked against a plausible size for a home workshop. A value far outside it is more likely a unit slip (millimetres written as metres, or inches as millimetres) than a real size.",
    },
    RuleInfo {
        id: RULE_TOOLS_MISSING,
        title: "No tool for the job",
        rule: "A step needs a capability and none of your tools supplies it. A wishlist tool does not count: it is not in the shop.",
    },
    RuleInfo {
        id: RULE_TOOLS_AWAY,
        title: "Tool away from home",
        rule: "A step needs a capability you have only at a makerspace or as a rental, so doing it means a trip.",
    },
    RuleInfo {
        id: RULE_TOOLS_ACCURACY,
        title: "Tool accuracy",
        rule: "A step asks for a tolerance finer than the tool holds. The tool's error, not your care, would decide the result.",
    },
    RuleInfo {
        id: RULE_TOOLS_NOISE,
        title: "Loud tool in quiet hours",
        rule: "A step is planned inside a window you asked to keep quiet, and the only tools that can do it are loud.",
    },
    RuleInfo {
        id: RULE_SAG,
        title: "Shelf sag",
        rule: "A shelf bends under its load. The sag at mid-span is sag = 5 w L^4 / (384 E I), with w = load x 9.81 / 1000 N/mm and I = depth x thickness^3 / 12 (mm^4), so w is in N/mm, L in mm, E in MPa and the answer in mm; it is compared with the sag you allow.",
    },
    RuleInfo {
        id: RULE_SCREW_THROUGH,
        title: "Screw through the stock",
        rule: "A screw longer than the stock it enters passes through and out the far face: it holds nothing and leaves a point showing. The screw must stop inside the stock.",
    },
    RuleInfo {
        id: RULE_SCREW_SHORT,
        title: "Screw too short",
        rule: "A screw that does not reach far enough into the second piece grips too little to hold. The screw must enter the second piece far enough to bite.",
    },
    RuleInfo {
        id: RULE_EDGE_DISTANCE,
        title: "Fastener too close to the edge",
        rule: "A screw or dowel close to an edge splits the wood. The hole's centre must stay at least the distance the material asks for, which grows with the fastener's diameter.",
    },
    RuleInfo {
        id: RULE_UNSUPPORTED,
        title: "Part held by nothing",
        rule: "A part is not joined to anything that carries it, so it would fall or flap. Every part needs a path of support back to the frame.",
    },
    RuleInfo {
        id: RULE_THIN_WALL,
        title: "Wall too thin",
        rule: "A wall thinner than the material can hold breaks out under load or when fastened. Thicken it, or change the joint.",
    },
    RuleInfo {
        id: RULE_SLOT_SEVERS,
        title: "Slot severs the part",
        rule: "A slot that spans the full width removes the material that held the two halves together, so the part comes apart. Leave a bridge of material.",
    },
];

/// The table entry whose id equals `rule_id` exactly, or `None` (rule W1).
pub fn info(rule_id: &str) -> Option<RuleInfo> {
    TABLE.iter().find(|entry| entry.id == rule_id).copied()
}

/// All fourteen entries in the order of `RULES` (rule W2).
pub fn rules() -> &'static [RuleInfo] {
    &TABLE
}

/// The table's label for `fact_key` under `rule_id`, or `None` (rule W3).
pub fn label(rule_id: &str, fact_key: &str) -> Option<&'static str> {
    let text = match (rule_id, fact_key) {
        (RULE_INTENT_MISSING, "field") => "Field",
        (RULE_PROMPT_SKIPPED, "prompt") => "Question",
        (RULE_PROMPT_SKIPPED, "step") => "Step",
        (RULE_UNIT_SANITY, "value") => "Value",
        (RULE_UNIT_SANITY, "unit") => "Unit",
        (RULE_TOOLS_MISSING, "step") => "Step",
        (RULE_TOOLS_MISSING, "capability") => "Capability",
        (RULE_TOOLS_AWAY, "step") => "Step",
        (RULE_TOOLS_AWAY, "capability") => "Capability",
        (RULE_TOOLS_AWAY, "access") => "Access",
        (RULE_TOOLS_ACCURACY, "step") => "Step",
        (RULE_TOOLS_ACCURACY, "capability") => "Capability",
        (RULE_TOOLS_ACCURACY, "tool") => "Tool",
        (RULE_TOOLS_ACCURACY, "accuracy_mm") => "Tool accuracy (mm)",
        (RULE_TOOLS_ACCURACY, "tolerance_mm") => "Tolerance (mm)",
        (RULE_TOOLS_NOISE, "step") => "Step",
        (RULE_TOOLS_NOISE, "capability") => "Capability",
        (RULE_TOOLS_NOISE, "tool") => "Tool",
        (RULE_TOOLS_NOISE, "day") => "Day",
        (RULE_SAG, "part") => "Part",
        (RULE_SAG, "span_mm") => "Span (mm)",
        (RULE_SAG, "load_kg_m") => "Load (kg/m)",
        (RULE_SAG, "depth_mm") => "Depth (mm)",
        (RULE_SAG, "thickness_mm") => "Thickness (mm)",
        (RULE_SAG, "e_mpa") => "E (MPa)",
        (RULE_SAG, "sag_mm") => "Sag (mm)",
        (RULE_SAG, "limit_mm") => "Allowed sag (mm)",
        (RULE_SCREW_THROUGH, "part") => "Part",
        (RULE_SCREW_THROUGH, "screw_mm") => "Screw (mm)",
        (RULE_SCREW_THROUGH, "thickness_mm") => "Stock (mm)",
        (RULE_SCREW_SHORT, "part") => "Part",
        (RULE_SCREW_SHORT, "screw_mm") => "Screw (mm)",
        (RULE_SCREW_SHORT, "thickness_mm") => "First piece (mm)",
        (RULE_SCREW_SHORT, "grip_mm") => "Grip (mm)",
        (RULE_EDGE_DISTANCE, "part") => "Part",
        (RULE_EDGE_DISTANCE, "fastener") => "Fastener",
        (RULE_EDGE_DISTANCE, "distance_mm") => "Distance (mm)",
        (RULE_EDGE_DISTANCE, "required_mm") => "Required (mm)",
        (RULE_UNSUPPORTED, "part") => "Part",
        (RULE_THIN_WALL, "part") => "Part",
        (RULE_THIN_WALL, "wall_mm") => "Wall (mm)",
        (RULE_THIN_WALL, "minimum_mm") => "Minimum (mm)",
        (RULE_SLOT_SEVERS, "part") => "Part",
        _ => return None,
    };
    Some(text)
}

/// The derivation the app draws for one finding (rules W4 and W5).
pub fn derive(finding: &Finding) -> Derivation {
    // W4.1/W4.2: a Mind finding never consults the table; a rule id only counts when it is
    // in the table, so an unknown id falls through to the generic derivation.
    let builtin_id: Option<&str> = match &finding.source {
        Source::Mind => None,
        Source::Rule(id) => info(id).map(|entry| entry.id),
    };
    let (title, rule) = match builtin_id.and_then(info) {
        Some(entry) => (entry.title.to_string(), entry.rule.to_string()),
        None => (GENERIC_TITLE.to_string(), GENERIC_RULE.to_string()),
    };
    // W4.3: facts in the finding's order; a "source" fact is dropped and only the first one
    // sets the source field.
    let mut inputs: Vec<(String, String)> = Vec::with_capacity(finding.facts.len());
    let mut source: Option<String> = None;
    let mut source_seen = false;
    for (key, value) in &finding.facts {
        if key == "source" {
            if !source_seen {
                source = Some(value.clone());
                source_seen = true;
            }
            continue;
        }
        let shown_key = match builtin_id {
            Some(id) => match label(id, key) {
                Some(text) => text.to_string(),
                None => key.clone(),
            },
            None => key.clone(),
        };
        inputs.push((shown_key, value.clone()));
    }
    // W4.4/W4.5: the sag table's source line is the only fallback, and only for a built-in.
    if !source_seen && builtin_id == Some(RULE_SAG) {
        source = Some(SAG_SOURCE.to_string());
    }
    Derivation {
        title,
        inputs,
        rule,
        source,
    }
}

/// Whether `rule_id` has an entry in the table (rule W1).
pub fn is_builtin(rule_id: &str) -> bool {
    info(rule_id).is_some()
}
