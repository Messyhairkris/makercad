//! Contract tests for mk_assist::why, written by Gordon as a PA (the Crew) from
//! crew/contracts/mk_why.md. They were the hidden tests of campaign mk-core-26.

// Crew-written tests: a helper that builds a whole `Finding` takes one argument per
// field, and worked rows print constants to the contract's digits; neither matters in a
// test.
#![allow(clippy::too_many_arguments, clippy::approx_constant)]

use mk_assist::findings::{Finding, Kind, Location, Severity, Source};
use mk_assist::why::{
    derive, info, is_builtin, label, rules, Derivation, GENERIC_RULE, GENERIC_TITLE, RULES,
    RULE_EDGE_DISTANCE, RULE_INTENT_MISSING, RULE_PROMPT_SKIPPED, RULE_SAG, RULE_SCREW_SHORT,
    RULE_SCREW_THROUGH, RULE_SLOT_SEVERS, RULE_THIN_WALL, RULE_TOOLS_ACCURACY, RULE_TOOLS_AWAY,
    RULE_TOOLS_MISSING, RULE_TOOLS_NOISE, RULE_UNIT_SANITY, RULE_UNSUPPORTED, SAG_SOURCE,
};

// The table's rule words, copied from the contract's W0 table (one string per rule).
const INTENT_RULE: &str = "Sancho starts from your intent: what the piece is for and what done looks like. Until that is written down, no rule pack can judge the plan, so this finding asks for it.";
const PROMPT_RULE: &str = "A question that decides a later step was skipped. Sancho asks it again instead of guessing, because a guessed answer would spread through every rule that depends on it.";
const UNIT_RULE: &str = "Every length is checked against a plausible size for a home workshop. A value far outside it is more likely a unit slip (millimetres written as metres, or inches as millimetres) than a real size.";
const MISSING_RULE: &str = "A step needs a capability and none of your tools supplies it. A wishlist tool does not count: it is not in the shop.";
const AWAY_RULE: &str = "A step needs a capability you have only at a makerspace or as a rental, so doing it means a trip.";
const ACCURACY_RULE: &str = "A step asks for a tolerance finer than the tool holds. The tool's error, not your care, would decide the result.";
const NOISE_RULE: &str = "A step is planned inside a window you asked to keep quiet, and the only tools that can do it are loud.";
const SAG_RULE: &str = "A shelf bends under its load. The sag at mid-span is sag = 5 w L^4 / (384 E I), with w = load x 9.81 / 1000 N/mm and I = depth x thickness^3 / 12 (mm^4), so w is in N/mm, L in mm, E in MPa and the answer in mm; it is compared with the sag you allow.";
const THROUGH_RULE: &str = "A screw longer than the stock it enters passes through and out the far face: it holds nothing and leaves a point showing. The screw must stop inside the stock.";
const SHORT_RULE: &str = "A screw that does not reach far enough into the second piece grips too little to hold. The screw must enter the second piece far enough to bite.";
const EDGE_RULE: &str = "A screw or dowel close to an edge splits the wood. The hole's centre must stay at least the distance the material asks for, which grows with the fastener's diameter.";
const UNSUPPORTED_RULE: &str = "A part is not joined to anything that carries it, so it would fall or flap. Every part needs a path of support back to the frame.";
const WALL_RULE: &str = "A wall thinner than the material can hold breaks out under load or when fastened. Thicken it, or change the joint.";
const SEVERS_RULE: &str = "A slot that spans the full width removes the material that held the two halves together, so the part comes apart. Leave a bridge of material.";

fn rule_finding(rule: &str, facts: &[(&str, &str)]) -> Finding {
    Finding {
        kind: Kind::Tip,
        severity: Severity::Info,
        source: Source::Rule(rule.to_string()),
        location: Location::Entity("shelf-1".to_string()),
        what: "what the app says".to_string(),
        why: "why the app says it".to_string(),
        fix: Some("part.thickness".to_string()),
        facts: inputs(facts),
    }
}

fn mind_finding(facts: &[(&str, &str)]) -> Finding {
    Finding {
        kind: Kind::Tip,
        severity: Severity::Info,
        source: Source::Mind,
        location: Location::Entity("shelf-1".to_string()),
        what: String::new(),
        why: String::new(),
        fix: None,
        facts: inputs(facts),
    }
}

fn inputs(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
    pairs
        .iter()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect()
}

fn derivation(
    title: &str,
    input_pairs: &[(&str, &str)],
    rule: &str,
    source: Option<&str>,
) -> Derivation {
    Derivation {
        title: title.to_string(),
        inputs: inputs(input_pairs),
        rule: rule.to_string(),
        source: source.map(|value| value.to_string()),
    }
}

// Worked example 1, rule core.intent_missing.
#[test]
fn core_intent_missing() {
    let found = derive(&rule_finding("core.intent_missing", &[("field", "intent")]));
    assert_eq!(
        found,
        derivation("Intent missing", &[("Field", "intent")], INTENT_RULE, None)
    );
}

// Worked example 2, rule core.prompt_skipped.
#[test]
fn core_prompt_skipped() {
    let found = derive(&rule_finding(
        "core.prompt_skipped",
        &[("prompt", "How will you finish it?"), ("step", "7")],
    ));
    assert_eq!(
        found,
        derivation(
            "Prompt skipped",
            &[("Question", "How will you finish it?"), ("Step", "7")],
            PROMPT_RULE,
            None
        )
    );
}

// Worked example 3, rule core.unit_sanity.
#[test]
fn core_unit_sanity() {
    let found = derive(&rule_finding(
        "core.unit_sanity",
        &[("value", "3"), ("unit", "m")],
    ));
    assert_eq!(
        found,
        derivation(
            "Unit sanity",
            &[("Value", "3"), ("Unit", "m")],
            UNIT_RULE,
            None
        )
    );
}

// Worked example 4, rule tools.missing.
#[test]
fn tools_missing() {
    let found = derive(&rule_finding(
        "tools.missing",
        &[("step", "4"), ("capability", "drill.hole")],
    ));
    assert_eq!(
        found,
        derivation(
            "No tool for the job",
            &[("Step", "4"), ("Capability", "drill.hole")],
            MISSING_RULE,
            None
        )
    );
}

// Worked example 5, rule tools.away.
#[test]
fn tools_away() {
    let found = derive(&rule_finding(
        "tools.away",
        &[
            ("step", "3"),
            ("capability", "cut.laser"),
            ("access", "makerspace"),
        ],
    ));
    assert_eq!(
        found,
        derivation(
            "Tool away from home",
            &[
                ("Step", "3"),
                ("Capability", "cut.laser"),
                ("Access", "makerspace"),
            ],
            AWAY_RULE,
            None
        )
    );
}

// Worked example 6, rule tools.accuracy.
#[test]
fn tools_accuracy() {
    let found = derive(&rule_finding(
        "tools.accuracy",
        &[
            ("step", "5"),
            ("capability", "cut.straight"),
            ("tool", "jig"),
            ("accuracy_mm", "1"),
            ("tolerance_mm", "0.5"),
        ],
    ));
    assert_eq!(
        found,
        derivation(
            "Tool accuracy",
            &[
                ("Step", "5"),
                ("Capability", "cut.straight"),
                ("Tool", "jig"),
                ("Tool accuracy (mm)", "1"),
                ("Tolerance (mm)", "0.5"),
            ],
            ACCURACY_RULE,
            None
        )
    );
}

// Worked example 7, rule tools.noise.
#[test]
fn tools_noise() {
    let found = derive(&rule_finding(
        "tools.noise",
        &[
            ("step", "8"),
            ("capability", "cut.curve"),
            ("tool", "jig"),
            ("day", "2"),
        ],
    ));
    assert_eq!(
        found,
        derivation(
            "Loud tool in quiet hours",
            &[
                ("Step", "8"),
                ("Capability", "cut.curve"),
                ("Tool", "jig"),
                ("Day", "2"),
            ],
            NOISE_RULE,
            None
        )
    );
}

// Worked example 8, rule structure.sag: the doc's own source line is the default.
#[test]
fn structure_sag() {
    let found = derive(&rule_finding(
        "structure.sag",
        &[
            ("part", "shelf-1"),
            ("span_mm", "900"),
            ("load_kg_m", "30"),
            ("depth_mm", "18"),
            ("thickness_mm", "18"),
            ("e_mpa", "9000"),
            ("sag_mm", "9.4"),
            ("limit_mm", "3"),
        ],
    ));
    assert_eq!(
        found,
        derivation(
            "Shelf sag",
            &[
                ("Part", "shelf-1"),
                ("Span (mm)", "900"),
                ("Load (kg/m)", "30"),
                ("Depth (mm)", "18"),
                ("Thickness (mm)", "18"),
                ("E (MPa)", "9000"),
                ("Sag (mm)", "9.4"),
                ("Allowed sag (mm)", "3"),
            ],
            SAG_RULE,
            Some(SAG_SOURCE)
        )
    );
}

// Worked example 9, rule structure.screw_through.
#[test]
fn structure_screw_through() {
    let found = derive(&rule_finding(
        "structure.screw_through",
        &[
            ("part", "side-left"),
            ("screw_mm", "40"),
            ("thickness_mm", "18"),
        ],
    ));
    assert_eq!(
        found,
        derivation(
            "Screw through the stock",
            &[
                ("Part", "side-left"),
                ("Screw (mm)", "40"),
                ("Stock (mm)", "18"),
            ],
            THROUGH_RULE,
            None
        )
    );
}

// Worked example 10, rule structure.screw_short.
#[test]
fn structure_screw_short() {
    let found = derive(&rule_finding(
        "structure.screw_short",
        &[
            ("part", "side-left"),
            ("screw_mm", "30"),
            ("thickness_mm", "18"),
            ("grip_mm", "5"),
        ],
    ));
    assert_eq!(
        found,
        derivation(
            "Screw too short",
            &[
                ("Part", "side-left"),
                ("Screw (mm)", "30"),
                ("First piece (mm)", "18"),
                ("Grip (mm)", "5"),
            ],
            SHORT_RULE,
            None
        )
    );
}

// Worked example 11, rule structure.edge_distance.
#[test]
fn structure_edge_distance() {
    let found = derive(&rule_finding(
        "structure.edge_distance",
        &[
            ("part", "shelf-1"),
            ("fastener", "M4"),
            ("distance_mm", "5"),
            ("required_mm", "8"),
        ],
    ));
    assert_eq!(
        found,
        derivation(
            "Fastener too close to the edge",
            &[
                ("Part", "shelf-1"),
                ("Fastener", "M4"),
                ("Distance (mm)", "5"),
                ("Required (mm)", "8"),
            ],
            EDGE_RULE,
            None
        )
    );
}

// Worked example 12, rule structure.unsupported.
#[test]
fn structure_unsupported() {
    let found = derive(&rule_finding(
        "structure.unsupported",
        &[("part", "back-panel")],
    ));
    assert_eq!(
        found,
        derivation(
            "Part held by nothing",
            &[("Part", "back-panel")],
            UNSUPPORTED_RULE,
            None
        )
    );
}

// Worked example 13, rule structure.thin_wall.
#[test]
fn structure_thin_wall() {
    let found = derive(&rule_finding(
        "structure.thin_wall",
        &[
            ("part", "drawer-side"),
            ("wall_mm", "4"),
            ("minimum_mm", "6"),
        ],
    ));
    assert_eq!(
        found,
        derivation(
            "Wall too thin",
            &[
                ("Part", "drawer-side"),
                ("Wall (mm)", "4"),
                ("Minimum (mm)", "6"),
            ],
            WALL_RULE,
            None
        )
    );
}

// Worked example 14, rule structure.slot_severs.
#[test]
fn structure_slot_severs() {
    let found = derive(&rule_finding(
        "structure.slot_severs",
        &[("part", "divider")],
    ));
    assert_eq!(
        found,
        derivation(
            "Slot severs the part",
            &[("Part", "divider")],
            SEVERS_RULE,
            None
        )
    );
}

// Worked example 15, an unknown rule id: generic, raw fact keys.
#[test]
fn unknown_rule_is_generic() {
    let found = derive(&rule_finding(
        "stock.kerf",
        &[("part", "shelf-1"), ("kerf_mm", "3")],
    ));
    assert_eq!(
        found,
        derivation(
            GENERIC_TITLE,
            &[("part", "shelf-1"), ("kerf_mm", "3")],
            GENERIC_RULE,
            None
        )
    );
}

// Worked example 16, a Mind finding: generic, no source.
#[test]
fn mind_finding_is_generic() {
    let found = derive(&mind_finding(&[("part", "shelf-1")]));
    assert_eq!(
        found,
        derivation(GENERIC_TITLE, &[("part", "shelf-1")], GENERIC_RULE, None)
    );
}

// Worked example 17, a source fact is not an input and overrides the sag table line.
#[test]
fn sag_uses_its_source_fact() {
    let found = derive(&rule_finding(
        "structure.sag",
        &[
            ("part", "shelf-1"),
            ("source", "FPL span table, pine 30 kg/m, 2026 print"),
            ("span_mm", "900"),
        ],
    ));
    assert_eq!(
        found,
        derivation(
            "Shelf sag",
            &[("Part", "shelf-1"), ("Span (mm)", "900")],
            SAG_RULE,
            Some("FPL span table, pine 30 kg/m, 2026 print")
        )
    );
}

// Worked example 18, a generic derivation still honours a source fact.
#[test]
fn generic_uses_its_source_fact() {
    let found = derive(&rule_finding(
        "stock.grain",
        &[("source", "Grain chart, 2026"), ("part", "shelf-1")],
    ));
    assert_eq!(
        found,
        derivation(
            GENERIC_TITLE,
            &[("part", "shelf-1")],
            GENERIC_RULE,
            Some("Grain chart, 2026")
        )
    );
}

// Worked example 19, a fact key with no label keeps its raw name.
#[test]
fn unlabelled_fact_keeps_raw_key() {
    let found = derive(&rule_finding(
        "structure.sag",
        &[("part", "shelf-1"), ("beam", "pine")],
    ));
    assert_eq!(
        found,
        derivation(
            "Shelf sag",
            &[("Part", "shelf-1"), ("beam", "pine")],
            SAG_RULE,
            Some(SAG_SOURCE)
        )
    );
}

// Worked example 20, no facts at all.
#[test]
fn no_facts_still_derives() {
    let found = derive(&rule_finding("tools.missing", &[]));
    assert_eq!(
        found,
        derivation("No tool for the job", &[], MISSING_RULE, None)
    );
    assert!(found.inputs.is_empty());
}

// Repeated fact keys are all kept, in the finding's order, each labelled.
#[test]
fn repeated_fact_keys_kept_in_order() {
    let found = derive(&rule_finding(
        "tools.missing",
        &[("step", "4"), ("capability", "cut.straight"), ("step", "5")],
    ));
    assert_eq!(
        found,
        derivation(
            "No tool for the job",
            &[("Step", "4"), ("Capability", "cut.straight"), ("Step", "5"),],
            MISSING_RULE,
            None
        )
    );
}

// Several source facts: the first one wins, and none of them is an input.
#[test]
fn first_source_fact_wins() {
    let found = derive(&rule_finding(
        "structure.sag",
        &[
            ("source", "first table"),
            ("span_mm", "900"),
            ("source", "second table"),
        ],
    ));
    assert_eq!(
        found,
        derivation(
            "Shelf sag",
            &[("Span (mm)", "900")],
            SAG_RULE,
            Some("first table")
        )
    );
}

// An empty source fact is a source all the same (W4.4).
#[test]
fn empty_source_fact_is_kept() {
    let found = derive(&rule_finding("tools.missing", &[("source", "")]));
    assert_eq!(found.source, Some(String::new()));
    assert!(found.inputs.is_empty());
}

// An empty rule id is not built in, so it gets the generic derivation.
#[test]
fn empty_rule_id_is_generic() {
    let found = derive(&rule_finding("", &[("part", "shelf-1")]));
    assert_eq!(
        found,
        derivation(GENERIC_TITLE, &[("part", "shelf-1")], GENERIC_RULE, None)
    );
}

// A Mind finding never borrows a built-in rule's labels, even for a familiar key.
#[test]
fn mind_finding_keeps_raw_keys() {
    let found = derive(&mind_finding(&[("span_mm", "900"), ("part", "shelf-1")]));
    assert_eq!(
        found,
        derivation(
            GENERIC_TITLE,
            &[("span_mm", "900"), ("part", "shelf-1")],
            GENERIC_RULE,
            None
        )
    );
}

// W1: every rule id has an entry with the table's title and a rule.
#[test]
fn info_has_every_rule_id() {
    let expected = [
        ("core.intent_missing", "Intent missing"),
        ("core.prompt_skipped", "Prompt skipped"),
        ("core.unit_sanity", "Unit sanity"),
        ("tools.missing", "No tool for the job"),
        ("tools.away", "Tool away from home"),
        ("tools.accuracy", "Tool accuracy"),
        ("tools.noise", "Loud tool in quiet hours"),
        ("structure.sag", "Shelf sag"),
        ("structure.screw_through", "Screw through the stock"),
        ("structure.screw_short", "Screw too short"),
        ("structure.edge_distance", "Fastener too close to the edge"),
        ("structure.unsupported", "Part held by nothing"),
        ("structure.thin_wall", "Wall too thin"),
        ("structure.slot_severs", "Slot severs the part"),
    ];
    for &(id, title) in expected.iter() {
        let found = info(id);
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.id, id);
        assert_eq!(found.title, title);
        assert!(!found.rule.is_empty());
        assert!(is_builtin(id));
    }
    assert!(!is_builtin("stock.kerf"));
}

// W1: an empty, unknown or near-miss id has no entry.
#[test]
fn info_near_miss_is_none() {
    assert!(info("").is_none());
    assert!(info("stock.kerf").is_none());
    assert!(info("tools.Missing").is_none());
    assert!(info("structure.sag ").is_none());
    assert!(!is_builtin(""));
}

// W2: rules() is the fourteen table entries, in the order of RULES.
#[test]
fn rules_are_the_fourteen_in_table_order() {
    let ids: Vec<&str> = rules().iter().map(|entry| entry.id).collect();
    assert_eq!(
        ids,
        vec![
            "core.intent_missing",
            "core.prompt_skipped",
            "core.unit_sanity",
            "tools.missing",
            "tools.away",
            "tools.accuracy",
            "tools.noise",
            "structure.sag",
            "structure.screw_through",
            "structure.screw_short",
            "structure.edge_distance",
            "structure.unsupported",
            "structure.thin_wall",
            "structure.slot_severs",
        ]
    );
    assert_eq!(ids.len(), 14);
    assert_eq!(rules().len(), RULES.len());
}

// W3: labels for the keys the rule packs emit.
#[test]
fn label_known_keys() {
    assert_eq!(label("structure.sag", "span_mm"), Some("Span (mm)"));
    assert_eq!(label("structure.sag", "limit_mm"), Some("Allowed sag (mm)"));
    assert_eq!(
        label("tools.accuracy", "tolerance_mm"),
        Some("Tolerance (mm)")
    );
    assert_eq!(label("tools.missing", "capability"), Some("Capability"));
    assert_eq!(label("core.prompt_skipped", "prompt"), Some("Question"));
}

// W3: no label for an unlisted key or an unknown rule.
#[test]
fn label_unlabelled_and_unknown_is_none() {
    assert_eq!(label("structure.sag", "beam"), None);
    assert_eq!(label("structure.unsupported", "span_mm"), None);
    assert_eq!(label("", "part"), None);
    assert_eq!(label("stock.kerf", "part"), None);
}

// The public constants are the contract's ids and the doc's source line.
#[test]
fn consts_are_the_rule_ids() {
    assert_eq!(RULE_INTENT_MISSING, "core.intent_missing");
    assert_eq!(RULE_PROMPT_SKIPPED, "core.prompt_skipped");
    assert_eq!(RULE_UNIT_SANITY, "core.unit_sanity");
    assert_eq!(RULE_TOOLS_MISSING, "tools.missing");
    assert_eq!(RULE_TOOLS_AWAY, "tools.away");
    assert_eq!(RULE_TOOLS_ACCURACY, "tools.accuracy");
    assert_eq!(RULE_TOOLS_NOISE, "tools.noise");
    assert_eq!(RULE_SAG, "structure.sag");
    assert_eq!(RULE_SCREW_THROUGH, "structure.screw_through");
    assert_eq!(RULE_SCREW_SHORT, "structure.screw_short");
    assert_eq!(RULE_EDGE_DISTANCE, "structure.edge_distance");
    assert_eq!(RULE_UNSUPPORTED, "structure.unsupported");
    assert_eq!(RULE_THIN_WALL, "structure.thin_wall");
    assert_eq!(RULE_SLOT_SEVERS, "structure.slot_severs");
    assert_eq!(RULES.len(), 14);
    assert_eq!(RULES[7], "structure.sag");
    assert_eq!(
        SAG_SOURCE,
        "Sag table: American Wood Council span data for pine, 30 kg/m load"
    );
    assert_eq!(GENERIC_TITLE, "Not a built-in rule");
    assert_eq!(
        GENERIC_RULE,
        "This finding does not come from a built-in rule, so Sancho has no formula or rule table to show; the inputs below are the facts the finding carried."
    );
}

// W4: odd but legal input never panics and never loses a fact.
#[test]
fn odd_input_never_panics() {
    let long = "x".repeat(4096);
    let found = derive(&rule_finding(
        "",
        &[("", ""), ("source", ""), ("", long.as_str())],
    ));
    assert_eq!(found.title, GENERIC_TITLE);
    assert_eq!(found.inputs, inputs(&[("", ""), ("", long.as_str())]));
    assert_eq!(found.source, Some(String::new()));

    let found = derive(&mind_finding(&[]));
    assert!(found.inputs.is_empty());
    assert_eq!(found.source, None);

    let found = derive(&rule_finding("structure.sag", &[("source", "only")]));
    assert!(found.inputs.is_empty());
    assert_eq!(found.source, Some("only".to_string()));
}
