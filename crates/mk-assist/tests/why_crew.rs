//! Tests for mk_assist::why written by Hoss (the Crew, campaign mk-core-26) from the
//! contract alone, without seeing the module.

//! Integration tests for `mk_assist::why` module.
//  Covers all rule-info lookups, label lookups, and every worked-example row.

// Crew-written tests: a helper that builds a whole `Finding` takes one argument per
// field, and worked rows print constants to the contract's digits; neither matters in a
// test.
#![allow(clippy::too_many_arguments, clippy::approx_constant)]

use mk_assist::findings::{Finding, Kind, Location, Severity, Source};
use mk_assist::why::{
    derive, info, is_builtin, label, rules, Derivation, RuleInfo, GENERIC_RULE, GENERIC_TITLE,
    RULE_EDGE_DISTANCE, RULE_INTENT_MISSING, RULE_PROMPT_SKIPPED, RULE_SAG, RULE_SCREW_SHORT,
    RULE_SCREW_THROUGH, RULE_SLOT_SEVERS, RULE_THIN_WALL, RULE_TOOLS_ACCURACY, RULE_TOOLS_AWAY,
    RULE_TOOLS_MISSING, RULE_TOOLS_NOISE, RULE_UNIT_SANITY, RULE_UNSUPPORTED, SAG_SOURCE,
};

fn make_finding(source: Source, facts: Vec<(&str, &str)>) -> Finding {
    Finding {
        kind: Kind::Forbidden, // any valid Kind variant works for tests
        severity: Severity::Info,
        source,
        location: Location::Project,
        what: String::new(),
        why: String::new(),
        fix: None,
        facts: facts
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
    }
}

/* ---------- rule-info tests ---------- */

#[test]
fn core_intent_missing() {
    let expected = RuleInfo {
        id: RULE_INTENT_MISSING,
        title: "Intent missing",
        rule: "Sancho starts from your intent: what the piece is for and what done looks like. Until that is written down, no rule pack can judge the plan, so this finding asks for it.",
    };
    assert_eq!(info(RULE_INTENT_MISSING), Some(expected));
    assert!(is_builtin(RULE_INTENT_MISSING));
}

#[test]
fn core_prompt_skipped() {
    let expected = RuleInfo {
        id: RULE_PROMPT_SKIPPED,
        title: "Prompt skipped",
        rule: "A question that decides a later step was skipped. Sancho asks it again instead of guessing, because a guessed answer would spread through every rule that depends on it.",
    };
    assert_eq!(info(RULE_PROMPT_SKIPPED), Some(expected));
    assert!(is_builtin(RULE_PROMPT_SKIPPED));
}

#[test]
fn core_unit_sanity() {
    let expected = RuleInfo {
        id: RULE_UNIT_SANITY,
        title: "Unit sanity",
        rule: "Every length is checked against a plausible size for a home workshop. A value far outside it is more likely a unit slip (millimetres written as metres, or inches as millimetres) than a real size.",
    };
    assert_eq!(info(RULE_UNIT_SANITY), Some(expected));
    assert!(is_builtin(RULE_UNIT_SANITY));
}

#[test]
fn tools_missing() {
    let expected = RuleInfo {
        id: RULE_TOOLS_MISSING,
        title: "No tool for the job",
        rule: "A step needs a capability and none of your tools supplies it. A wishlist tool does not count: it is not in the shop.",
    };
    assert_eq!(info(RULE_TOOLS_MISSING), Some(expected));
    assert!(is_builtin(RULE_TOOLS_MISSING));
}

#[test]
fn tools_away() {
    let expected = RuleInfo {
        id: RULE_TOOLS_AWAY,
        title: "Tool away from home",
        rule: "A step needs a capability you have only at a makerspace or as a rental, so doing it means a trip.",
    };
    assert_eq!(info(RULE_TOOLS_AWAY), Some(expected));
    assert!(is_builtin(RULE_TOOLS_AWAY));
}

#[test]
fn tools_accuracy() {
    let expected = RuleInfo {
        id: RULE_TOOLS_ACCURACY,
        title: "Tool accuracy",
        rule: "A step asks for a tolerance finer than the tool holds. The tool's error, not your care, would decide the result.",
    };
    assert_eq!(info(RULE_TOOLS_ACCURACY), Some(expected));
    assert!(is_builtin(RULE_TOOLS_ACCURACY));
}

#[test]
fn tools_noise() {
    let expected = RuleInfo {
        id: RULE_TOOLS_NOISE,
        title: "Loud tool in quiet hours",
        rule: "A step is planned inside a window you asked to keep quiet, and the only tools that can do it are loud.",
    };
    assert_eq!(info(RULE_TOOLS_NOISE), Some(expected));
    assert!(is_builtin(RULE_TOOLS_NOISE));
}

#[test]
fn structure_sag() {
    let expected = RuleInfo {
        id: RULE_SAG,
        title: "Shelf sag",
        rule: "A shelf bends under its load. The sag at mid-span is sag = 5 w L^4 / (384 E I), with w = load x 9.81 / 1000 N/mm and I = depth x thickness^3 / 12 (mm^4), so w is in N/mm, L in mm, E in MPa and the answer in mm; it is compared with the sag you allow.",
    };
    assert_eq!(info(RULE_SAG), Some(expected));
    assert!(is_builtin(RULE_SAG));
}

#[test]
fn structure_screw_through() {
    let expected = RuleInfo {
        id: RULE_SCREW_THROUGH,
        title: "Screw through the stock",
        rule: "A screw longer than the stock it enters passes through and out the far face: it holds nothing and leaves a point showing. The screw must stop inside the stock.",
    };
    assert_eq!(info(RULE_SCREW_THROUGH), Some(expected));
    assert!(is_builtin(RULE_SCREW_THROUGH));
}

#[test]
fn structure_screw_short() {
    let expected = RuleInfo {
        id: RULE_SCREW_SHORT,
        title: "Screw too short",
        rule: "A screw that does not reach far enough into the second piece grips too little to hold. The screw must enter the second piece far enough to bite.",
    };
    assert_eq!(info(RULE_SCREW_SHORT), Some(expected));
    assert!(is_builtin(RULE_SCREW_SHORT));
}

#[test]
fn structure_edge_distance() {
    let expected = RuleInfo {
        id: RULE_EDGE_DISTANCE,
        title: "Fastener too close to the edge",
        rule: "A screw or dowel close to an edge splits the wood. The hole's centre must stay at least the distance the material asks for, which grows with the fastener's diameter.",
    };
    assert_eq!(info(RULE_EDGE_DISTANCE), Some(expected));
    assert!(is_builtin(RULE_EDGE_DISTANCE));
}

#[test]
fn structure_unsupported() {
    let expected = RuleInfo {
        id: RULE_UNSUPPORTED,
        title: "Part held by nothing",
        rule: "A part is not joined to anything that carries it, so it would fall or flap. Every part needs a path of support back to the frame.",
    };
    assert_eq!(info(RULE_UNSUPPORTED), Some(expected));
    assert!(is_builtin(RULE_UNSUPPORTED));
}

#[test]
fn structure_thin_wall() {
    let expected = RuleInfo {
        id: RULE_THIN_WALL,
        title: "Wall too thin",
        rule: "A wall thinner than the material can hold breaks out under load or when fastened. Thicken it, or change the joint.",
    };
    assert_eq!(info(RULE_THIN_WALL), Some(expected));
    assert!(is_builtin(RULE_THIN_WALL));
}

#[test]
fn structure_slot_severs() {
    let expected = RuleInfo {
        id: RULE_SLOT_SEVERS,
        title: "Slot severs the part",
        rule: "A slot that spans the full width removes the material that held the two halves together, so the part comes apart. Leave a bridge of material.",
    };
    assert_eq!(info(RULE_SLOT_SEVERS), Some(expected));
    assert!(is_builtin(RULE_SLOT_SEVERS));
}

/* ---------- rules order test ---------- */

#[test]
fn rules_order() {
    let ids: Vec<&str> = rules().iter().map(|ri| ri.id).collect();
    assert_eq!(
        ids,
        vec![
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
        ]
    );
}

/* ---------- label tests ---------- */

#[test]
fn label_examples() {
    assert_eq!(label(RULE_INTENT_MISSING, "field"), Some("Field"));
    assert_eq!(label(RULE_PROMPT_SKIPPED, "prompt"), Some("Question"));
    assert_eq!(label(RULE_PROMPT_SKIPPED, "step"), Some("Step"));
    assert_eq!(label(RULE_UNIT_SANITY, "value"), Some("Value"));
    assert_eq!(label(RULE_UNIT_SANITY, "unit"), Some("Unit"));
    assert_eq!(label(RULE_TOOLS_MISSING, "step"), Some("Step"));
    assert_eq!(label(RULE_TOOLS_MISSING, "capability"), Some("Capability"));
    assert_eq!(
        label(RULE_TOOLS_ACCURACY, "accuracy_mm"),
        Some("Tool accuracy (mm)")
    );
    assert_eq!(
        label(RULE_TOOLS_ACCURACY, "tolerance_mm"),
        Some("Tolerance (mm)")
    );
    assert_eq!(label(RULE_SAG, "span_mm"), Some("Span (mm)"));
    assert_eq!(label(RULE_SAG, "sag_mm"), Some("Sag (mm)"));
    // unknown key
    assert_eq!(label(RULE_SAG, "beam"), None);
    // unknown rule
    assert_eq!(label("unknown.id", "any"), None);
}

/* ---------- worked-example derivations ---------- */

#[test]
fn worked_example_1() {
    let f = make_finding(
        Source::Rule("core.intent_missing".into()),
        vec![("field", "intent")],
    );
    let got = derive(&f);
    let want = Derivation {
        title: "Intent missing".into(),
        inputs: vec![("Field".into(),"intent".into())],
        rule: "Sancho starts from your intent: what the piece is for and what done looks like. Until that is written down, no rule pack can judge the plan, so this finding asks for it.".into(),
        source: None,
    };
    assert_eq!(got, want);
}

#[test]
fn worked_example_2() {
    let f = make_finding(
        Source::Rule("core.prompt_skipped".into()),
        vec![("prompt", "How will you finish it?"), ("step", "7")],
    );
    let got = derive(&f);
    let want = Derivation {
        title: "Prompt skipped".into(),
        inputs: vec![("Question".into(),"How will you finish it?".into()),
                     ("Step".into(),"7".into())],
        rule: "A question that decides a later step was skipped. Sancho asks it again instead of guessing, because a guessed answer would spread through every rule that depends on it.".into(),
        source: None,
    };
    assert_eq!(got, want);
}

#[test]
fn worked_example_3() {
    let f = make_finding(
        Source::Rule("core.unit_sanity".into()),
        vec![("value", "3"), ("unit", "m")],
    );
    let got = derive(&f);
    let want = Derivation {
        title: "Unit sanity".into(),
        inputs: vec![("Value".into(),"3".into()),("Unit".into(),"m".into())],
        rule: "Every length is checked against a plausible size for a home workshop. A value far outside it is more likely a unit slip (millimetres written as metres, or inches as millimetres) than a real size.".into(),
        source: None,
    };
    assert_eq!(got, want);
}

#[test]
fn worked_example_4() {
    let f = make_finding(
        Source::Rule("tools.missing".into()),
        vec![("step", "4"), ("capability", "drill.hole")],
    );
    let got = derive(&f);
    let want = Derivation {
        title: "No tool for the job".into(),
        inputs: vec![("Step".into(),"4".into()),("Capability".into(),"drill.hole".into())],
        rule: "A step needs a capability and none of your tools supplies it. A wishlist tool does not count: it is not in the shop.".into(),
        source: None,
    };
    assert_eq!(got, want);
}

#[test]
fn worked_example_5() {
    let f = make_finding(
        Source::Rule("tools.away".into()),
        vec![
            ("step", "3"),
            ("capability", "cut.laser"),
            ("access", "makerspace"),
        ],
    );
    let got = derive(&f);
    let want = Derivation {
        title: "Tool away from home".into(),
        inputs: vec![("Step".into(),"3".into()),("Capability".into(),"cut.laser".into()),("Access".into(),"makerspace".into())],
        rule: "A step needs a capability you have only at a makerspace or as a rental, so doing it means a trip.".into(),
        source: None,
    };
    assert_eq!(got, want);
}

#[test]
fn worked_example_6() {
    let f = make_finding(
        Source::Rule("tools.accuracy".into()),
        vec![
            ("step", "5"),
            ("capability", "cut.straight"),
            ("tool", "jig"),
            ("accuracy_mm", "1"),
            ("tolerance_mm", "0.5"),
        ],
    );
    let got = derive(&f);
    let want = Derivation {
        title: "Tool accuracy".into(),
        inputs: vec![
            ("Step".into(),"5".into()),
            ("Capability".into(),"cut.straight".into()),
            ("Tool".into(),"jig".into()),
            ("Tool accuracy (mm)".into(),"1".into()),
            ("Tolerance (mm)".into(),"0.5".into()),
        ],
        rule: "A step asks for a tolerance finer than the tool holds. The tool's error, not your care, would decide the result.".into(),
        source: None,
    };
    assert_eq!(got, want);
}

#[test]
fn worked_example_7() {
    let f = make_finding(
        Source::Rule("tools.noise".into()),
        vec![
            ("step", "8"),
            ("capability", "cut.curve"),
            ("tool", "jig"),
            ("day", "2"),
        ],
    );
    let got = derive(&f);
    let want = Derivation {
        title: "Loud tool in quiet hours".into(),
        inputs: vec![
            ("Step".into(),"8".into()),
            ("Capability".into(),"cut.curve".into()),
            ("Tool".into(),"jig".into()),
            ("Day".into(),"2".into()),
        ],
        rule: "A step is planned inside a window you asked to keep quiet, and the only tools that can do it are loud.".into(),
        source: None,
    };
    assert_eq!(got, want);
}

#[test]
fn worked_example_8() {
    let f = make_finding(
        Source::Rule("structure.sag".into()),
        vec![
            ("part", "shelf-1"),
            ("span_mm", "900"),
            ("load_kg_m", "30"),
            ("depth_mm", "18"),
            ("thickness_mm", "18"),
            ("e_mpa", "9000"),
            ("sag_mm", "9.4"),
            ("limit_mm", "3"),
        ],
    );
    let got = derive(&f);
    let want = Derivation {
        title: "Shelf sag".into(),
        inputs: vec![
            ("Part".into(),"shelf-1".into()),
            ("Span (mm)".into(),"900".into()),
            ("Load (kg/m)".into(),"30".into()),
            ("Depth (mm)".into(),"18".into()),
            ("Thickness (mm)".into(),"18".into()),
            ("E (MPa)".into(),"9000".into()),
            ("Sag (mm)".into(),"9.4".into()),
            ("Allowed sag (mm)".into(),"3".into()),
        ],
        rule: "A shelf bends under its load. The sag at mid-span is sag = 5 w L^4 / (384 E I), with w = load x 9.81 / 1000 N/mm and I = depth x thickness^3 / 12 (mm^4), so w is in N/mm, L in mm, E in MPa and the answer in mm; it is compared with the sag you allow.".into(),
        source: Some(SAG_SOURCE.to_string()),
    };
    assert_eq!(got, want);
}

#[test]
fn worked_example_9() {
    let f = make_finding(
        Source::Rule("structure.screw_through".into()),
        vec![
            ("part", "side-left"),
            ("screw_mm", "40"),
            ("thickness_mm", "18"),
        ],
    );
    let got = derive(&f);
    let want = Derivation {
        title: "Screw through the stock".into(),
        inputs: vec![
            ("Part".into(),"side-left".into()),
            ("Screw (mm)".into(),"40".into()),
            ("Stock (mm)".into(),"18".into()),
        ],
        rule: "A screw longer than the stock it enters passes through and out the far face: it holds nothing and leaves a point showing. The screw must stop inside the stock.".into(),
        source: None,
    };
    assert_eq!(got, want);
}

#[test]
fn worked_example_10() {
    let f = make_finding(
        Source::Rule("structure.screw_short".into()),
        vec![
            ("part", "side-left"),
            ("screw_mm", "30"),
            ("thickness_mm", "18"),
            ("grip_mm", "5"),
        ],
    );
    let got = derive(&f);
    let want = Derivation {
        title: "Screw too short".into(),
        inputs: vec![
            ("Part".into(),"side-left".into()),
            ("Screw (mm)".into(),"30".into()),
            ("First piece (mm)".into(),"18".into()),
            ("Grip (mm)".into(),"5".into()),
        ],
        rule: "A screw that does not reach far enough into the second piece grips too little to hold. The screw must enter the second piece far enough to bite.".into(),
        source: None,
    };
    assert_eq!(got, want);
}

#[test]
fn worked_example_11() {
    let f = make_finding(
        Source::Rule("structure.edge_distance".into()),
        vec![
            ("part", "shelf-1"),
            ("fastener", "M4"),
            ("distance_mm", "5"),
            ("required_mm", "8"),
        ],
    );
    let got = derive(&f);
    let want = Derivation {
        title: "Fastener too close to the edge".into(),
        inputs: vec![
            ("Part".into(),"shelf-1".into()),
            ("Fastener".into(),"M4".into()),
            ("Distance (mm)".into(),"5".into()),
            ("Required (mm)".into(),"8".into()),
        ],
        rule: "A screw or dowel close to an edge splits the wood. The hole's centre must stay at least the distance the material asks for, which grows with the fastener's diameter.".into(),
        source: None,
    };
    assert_eq!(got, want);
}

#[test]
fn worked_example_12() {
    let f = make_finding(
        Source::Rule("structure.unsupported".into()),
        vec![("part", "back-panel")],
    );
    let got = derive(&f);
    let want = Derivation {
        title: "Part held by nothing".into(),
        inputs: vec![("Part".into(),"back-panel".into())],
        rule: "A part is not joined to anything that carries it, so it would fall or flap. Every part needs a path of support back to the frame.".into(),
        source: None,
    };
    assert_eq!(got, want);
}

#[test]
fn worked_example_13() {
    let f = make_finding(
        Source::Rule("structure.thin_wall".into()),
        vec![
            ("part", "drawer-side"),
            ("wall_mm", "4"),
            ("minimum_mm", "6"),
        ],
    );
    let got = derive(&f);
    let want = Derivation {
        title: "Wall too thin".into(),
        inputs: vec![
            ("Part".into(),"drawer-side".into()),
            ("Wall (mm)".into(),"4".into()),
            ("Minimum (mm)".into(),"6".into()),
        ],
        rule: "A wall thinner than the material can hold breaks out under load or when fastened. Thicken it, or change the joint.".into(),
        source: None,
    };
    assert_eq!(got, want);
}

#[test]
fn worked_example_14() {
    let f = make_finding(
        Source::Rule("structure.slot_severs".into()),
        vec![("part", "divider")],
    );
    let got = derive(&f);
    let want = Derivation {
        title: "Slot severs the part".into(),
        inputs: vec![("Part".into(),"divider".into())],
        rule: "A slot that spans the full width removes the material that held the two halves together, so the part comes apart. Leave a bridge of material.".into(),
        source: None,
    };
    assert_eq!(got, want);
}

/* ---------- generic / edge cases ---------- */

#[test]
fn worked_example_15_unknown_rule() {
    let f = make_finding(
        Source::Rule("stock.kerf".into()),
        vec![("part", "shelf-1"), ("kerf_mm", "3")],
    );
    let got = derive(&f);
    let want = Derivation {
        title: GENERIC_TITLE.into(),
        inputs: vec![
            ("part".into(), "shelf-1".into()),
            ("kerf_mm".into(), "3".into()),
        ],
        rule: GENERIC_RULE.into(),
        source: None,
    };
    assert_eq!(got, want);
}

#[test]
fn worked_example_16_mind_source() {
    let f = make_finding(Source::Mind, vec![("part", "shelf-1")]);
    let got = derive(&f);
    let want = Derivation {
        title: GENERIC_TITLE.into(),
        inputs: vec![("part".into(), "shelf-1".into())],
        rule: GENERIC_RULE.into(),
        source: None,
    };
    assert_eq!(got, want);
}

#[test]
fn worked_example_17_source_fact_overrides() {
    let f = make_finding(
        Source::Rule("structure.sag".into()),
        vec![
            ("part", "shelf-1"),
            ("source", "FPL span table, pine 30 kg/m, 2026 print"),
            ("span_mm", "900"),
        ],
    );
    let got = derive(&f);
    let want = Derivation {
        title: "Shelf sag".into(),
        inputs: vec![
            ("Part".into(),"shelf-1".into()),
            ("Span (mm)".into(),"900".into()),
        ],
        rule: "A shelf bends under its load. The sag at mid-span is sag = 5 w L^4 / (384 E I), with w = load x 9.81 / 1000 N/mm and I = depth x thickness^3 / 12 (mm^4), so w is in N/mm, L in mm, E in MPa and the answer in mm; it is compared with the sag you allow.".into(),
        source: Some("FPL span table, pine 30 kg/m, 2026 print".into()),
    };
    assert_eq!(got, want);
}

#[test]
fn worked_example_18_generic_with_source_fact() {
    let f = make_finding(
        Source::Rule("stock.grain".into()),
        vec![("source", "Grain chart, 2026"), ("part", "shelf-1")],
    );
    let got = derive(&f);
    let want = Derivation {
        title: GENERIC_TITLE.into(),
        inputs: vec![("part".into(), "shelf-1".into())],
        rule: GENERIC_RULE.into(),
        source: Some("Grain chart, 2026".into()),
    };
    assert_eq!(got, want);
}

#[test]
fn worked_example_19_unlabelled_key() {
    let f = make_finding(
        Source::Rule("structure.sag".into()),
        vec![("part", "shelf-1"), ("beam", "pine")],
    );
    let got = derive(&f);
    let want = Derivation {
        title: "Shelf sag".into(),
        inputs: vec![
            ("Part".into(),"shelf-1".into()),
            ("beam".into(),"pine".into()),
        ],
        rule: "A shelf bends under its load. The sag at mid-span is sag = 5 w L^4 / (384 E I), with w = load x 9.81 / 1000 N/mm and I = depth x thickness^3 / 12 (mm^4), so w is in N/mm, L in mm, E in MPa and the answer in mm; it is compared with the sag you allow.".into(),
        source: Some(SAG_SOURCE.to_string()),
    };
    assert_eq!(got, want);
}

#[test]
fn worked_example_20_empty_facts() {
    let f = make_finding(Source::Rule("tools.missing".into()), vec![]);
    let got = derive(&f);
    let want = Derivation {
        title: "No tool for the job".into(),
        inputs: vec![],
        rule: "A step needs a capability and none of your tools supplies it. A wishlist tool does not count: it is not in the shop.".into(),
        source: None,
    };
    assert_eq!(got, want);
}

/* ---------- robustness tests (no panic) ---------- */

#[test]
fn derive_nan_inf_and_empty_rule() {
    // NaN and infinities as string values - should be treated as ordinary strings.
    let f = make_finding(
        Source::Rule("core.unit_sanity".into()),
        vec![("value", "NaN"), ("unit", "inf")],
    );
    let got = derive(&f);
    let want = Derivation {
        title: "Unit sanity".into(),
        inputs: vec![("Value".into(),"NaN".into()),("Unit".into(),"inf".into())],
        rule: "Every length is checked against a plausible size for a home workshop. A value far outside it is more likely a unit slip (millimetres written as metres, or inches as millimetres) than a real size.".into(),
        source: None,
    };
    assert_eq!(got, want);
}

#[test]
fn derive_unknown_rule_id() {
    let f = make_finding(Source::Rule("".into()), vec![("foo", "bar")]);
    let got = derive(&f);
    let want = Derivation {
        title: GENERIC_TITLE.into(),
        inputs: vec![("foo".into(), "bar".into())],
        rule: GENERIC_RULE.into(),
        source: None,
    };
    assert_eq!(got, want);
}
