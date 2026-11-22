//! Contract tests for mk_assist::rules_core, written by Qwen as a PA (the Crew) from
//! crew/contracts/mk_rules_core.md. They were the hidden tests of campaign mk-core-17.

use mk_assist::findings::{Finding, Kind, Location, Severity, Source};
use mk_assist::rules_core::{
    class_for_mm, evaluate, intent_missing, largest_mm, prompt_skipped, unit_sanity, Part,
    ProjectFacts, Prompt, ScaleClass, RULES, RULE_INTENT_MISSING, RULE_PROMPT_SKIPPED,
    RULE_UNIT_SANITY,
};

// Every rule on its own, every worked example with whole Finding values
// (words, facts, fix, location and source exactly), class_for_mm at every
// boundary, duplicates ignored, and no panics on NaN, infinite, zero or
// negative sizes. No files, randomness or time.

fn facts(
    goal: Option<&str>,
    scale: ScaleClass,
    prompts: &[Prompt],
    parts: &[Part],
) -> ProjectFacts {
    ProjectFacts {
        goal: goal.map(str::to_string),
        scale,
        prompts: prompts.to_vec(),
        parts: parts.to_vec(),
    }
}

fn part(name: &str, size: [f64; 3]) -> Part {
    Part {
        name: name.to_string(),
        size_mm: size,
    }
}

fn prompt(id: &str, skipped: bool, relevant_now: bool) -> Prompt {
    Prompt {
        id: id.to_string(),
        skipped,
        relevant_now,
    }
}

fn r1_finding() -> Finding {
    Finding {
        kind: Kind::Tip,
        severity: Severity::Warning,
        location: Location::Field("intent.goal".to_string()),
        what: "No goal yet".to_string(),
        why: "Sancho checks the design against its goal, and there is none to check against."
            .to_string(),
        fix: Some("intent.edit".to_string()),
        facts: vec![("goal".to_string(), "missing".to_string())],
        source: Source::Rule(RULE_INTENT_MISSING.to_string()),
    }
}

fn r2_finding(id: &str) -> Finding {
    Finding {
        kind: Kind::Tip,
        severity: Severity::Info,
        location: Location::Field(format!("prompt.{id}")),
        what: "A skipped question matters now".to_string(),
        why: "You skipped this question earlier, and what you are doing now depends on it."
            .to_string(),
        fix: Some("prompt.answer".to_string()),
        facts: vec![("prompt".to_string(), id.to_string())],
        source: Source::Rule(RULE_PROMPT_SKIPPED.to_string()),
    }
}

fn r3a_finding(name: &str, size: &str, upper: &str, class_key: &str, suggested: &str) -> Finding {
    Finding {
        kind: Kind::Tip,
        severity: Severity::Warning,
        location: Location::Entity(name.to_string()),
        what: format!("{name} is {size} mm"),
        why: format!(
            "That is large for a {class_key} project (up to {upper} mm). Check the units."
        ),
        fix: Some("project.scale".to_string()),
        facts: vec![
            ("part".to_string(), name.to_string()),
            ("largest_mm".to_string(), size.to_string()),
            ("class".to_string(), class_key.to_string()),
            ("suggested".to_string(), suggested.to_string()),
        ],
        source: Source::Rule(RULE_UNIT_SANITY.to_string()),
    }
}

fn r3b_finding(size: &str, lower: &str, class_key: &str, suggested: &str) -> Finding {
    Finding {
        kind: Kind::Tip,
        severity: Severity::Warning,
        location: Location::Project,
        what: format!("The whole design is {size} mm"),
        why: format!("That is small for a {class_key} project (from {lower} mm). Check the units."),
        fix: Some("project.scale".to_string()),
        facts: vec![
            ("largest_mm".to_string(), size.to_string()),
            ("class".to_string(), class_key.to_string()),
            ("suggested".to_string(), suggested.to_string()),
        ],
        source: Source::Rule(RULE_UNIT_SANITY.to_string()),
    }
}

#[test]
fn c1_key() {
    assert!(ScaleClass::Room.key() == "room");
    assert!(ScaleClass::Furniture.key() == "furniture");
    assert!(ScaleClass::Body.key() == "body");
    assert!(ScaleClass::Bench.key() == "bench");
    assert!(ScaleClass::Macro.key() == "macro");
}

#[test]
fn c2_range_mm() {
    assert!(ScaleClass::Room.range_mm() == (500.0, 10000.0));
    assert!(ScaleClass::Furniture.range_mm() == (300.0, 2500.0));
    assert!(ScaleClass::Body.range_mm() == (100.0, 1000.0));
    assert!(ScaleClass::Bench.range_mm() == (10.0, 500.0));
    assert!(ScaleClass::Macro.range_mm() == (0.0, 10.0));
}

#[test]
fn c3_largest_mm() {
    assert!(largest_mm(&part("Pencil", [8.0, 3000.0, 8.0])) == Some(3000.0));
    assert!(largest_mm(&part("Screw", [4.0, 40.0, 4.0])) == Some(40.0));
    assert!(largest_mm(&part("Cube", [5.0, 5.0, 5.0])) == Some(5.0));
}

#[test]
fn c3_largest_mm_invalid() {
    assert!(largest_mm(&part("Pin", [f64::NAN, 1.0, 1.0])).is_none());
    assert!(largest_mm(&part("Pin", [1.0, f64::NAN, 1.0])).is_none());
    assert!(largest_mm(&part("Pin", [1.0, 1.0, f64::NAN])).is_none());
    assert!(largest_mm(&part("Pin", [f64::INFINITY, 1.0, 1.0])).is_none());
    assert!(largest_mm(&part("Pin", [1.0, f64::NEG_INFINITY, 1.0])).is_none());
    assert!(largest_mm(&part("Pin", [0.0, 1.0, 1.0])).is_none());
    assert!(largest_mm(&part("Pin", [1.0, -1.0, 1.0])).is_none());
}

#[test]
fn c4_class_for_mm_boundaries() {
    assert!(class_for_mm(9.99) == Some(ScaleClass::Macro));
    assert!(class_for_mm(10.0) == Some(ScaleClass::Bench));
    assert!(class_for_mm(500.0) == Some(ScaleClass::Bench));
    assert!(class_for_mm(500.1) == Some(ScaleClass::Furniture));
    assert!(class_for_mm(2500.0) == Some(ScaleClass::Furniture));
    assert!(class_for_mm(2500.1) == Some(ScaleClass::Room));
}

#[test]
fn c4_class_for_mm_invalid() {
    assert!(class_for_mm(0.0).is_none());
    assert!(class_for_mm(-1.0).is_none());
    assert!(class_for_mm(f64::NAN).is_none());
    assert!(class_for_mm(f64::INFINITY).is_none());
    assert!(class_for_mm(f64::NEG_INFINITY).is_none());
}

#[test]
fn c4_class_for_mm_never_body() {
    for mm in [
        0.5, 9.99, 10.0, 100.0, 500.0, 500.1, 1000.0, 2500.0, 2500.1, 10000.0, 1e9,
    ] {
        assert!(class_for_mm(mm) != Some(ScaleClass::Body));
    }
}

#[test]
fn r1_intent_missing_goal_none() {
    let got = intent_missing(&facts(None, ScaleClass::Bench, &[], &[]));
    assert!(got == vec![r1_finding()]);
}

#[test]
fn r1_intent_missing_goal_blank() {
    let got = intent_missing(&facts(Some("   "), ScaleClass::Bench, &[], &[]));
    assert!(got == vec![r1_finding()]);
}

#[test]
fn r1_intent_missing_goal_present() {
    let got = intent_missing(&facts(Some("A pencil box"), ScaleClass::Bench, &[], &[]));
    assert!(got.is_empty());
}

#[test]
fn r1_intent_missing_goal_trimmed_present() {
    let got = intent_missing(&facts(
        Some("  A pencil box  "),
        ScaleClass::Bench,
        &[],
        &[],
    ));
    assert!(got.is_empty());
}

#[test]
fn r2_prompt_skipped_single() {
    let prompts = [prompt("a", true, true)];
    let got = prompt_skipped(&facts(Some("Box"), ScaleClass::Bench, &prompts, &[]));
    assert!(got == vec![r2_finding("a")]);
}

#[test]
fn r2_prompt_skipped_not_relevant() {
    let prompts = [prompt("a", true, false)];
    let got = prompt_skipped(&facts(Some("Box"), ScaleClass::Bench, &prompts, &[]));
    assert!(got.is_empty());
}

#[test]
fn r2_prompt_skipped_answered() {
    let prompts = [prompt("a", false, true)];
    let got = prompt_skipped(&facts(Some("Box"), ScaleClass::Bench, &prompts, &[]));
    assert!(got.is_empty());
}

#[test]
fn r2_prompt_skipped_order() {
    let prompts = [
        prompt("z", true, true),
        prompt("m", true, true),
        prompt("a", true, true),
    ];
    let got = prompt_skipped(&facts(Some("Box"), ScaleClass::Bench, &prompts, &[]));
    assert!(got == vec![r2_finding("z"), r2_finding("m"), r2_finding("a")]);
}

#[test]
fn r2_prompt_skipped_duplicate_id_ignored() {
    // Worked example: a skipped relevant, b skipped not relevant,
    // c answered relevant, a skipped relevant -> only the first a.
    let prompts = [
        prompt("a", true, true),
        prompt("b", true, false),
        prompt("c", false, true),
        prompt("a", true, true),
    ];
    let got = prompt_skipped(&facts(Some("Box"), ScaleClass::Bench, &prompts, &[]));
    assert!(got == vec![r2_finding("a")]);
}

#[test]
fn r2_prompt_skipped_duplicate_id_seen_earlier() {
    // The contract is silent on this case; the module reads "an id already
    // used" as "seen by an earlier prompt", so the first prompt with an id
    // owns it even when it is not flagged.
    let prompts = [prompt("a", true, false), prompt("a", true, true)];
    let got = prompt_skipped(&facts(Some("Box"), ScaleClass::Bench, &prompts, &[]));
    assert!(got.is_empty());
}

#[test]
fn r3a_part_too_big() {
    // Worked example: a Bench project, a pencil 3000 mm long.
    let parts = [part("Pencil", [3000.0, 8.0, 8.0])];
    let got = unit_sanity(&facts(Some("A pencil box"), ScaleClass::Bench, &[], &parts));
    assert!(got == vec![r3a_finding("Pencil", "3000", "500", "bench", "room")]);
}

#[test]
fn r3a_inside_inclusive() {
    // Worked example: 2500 is inside the Furniture range, inclusive.
    let parts = [part("Board", [18.0, 200.0, 2500.0])];
    let got = unit_sanity(&facts(Some("Shelf"), ScaleClass::Furniture, &[], &parts));
    assert!(got.is_empty());
}

#[test]
fn r3a_just_over() {
    let parts = [part("Board", [18.0, 200.0, 2500.5])];
    let got = unit_sanity(&facts(Some("Shelf"), ScaleClass::Furniture, &[], &parts));
    assert!(got == vec![r3a_finding("Board", "2500.5", "2500", "furniture", "room")]);
}

#[test]
fn r3a_macro() {
    let parts = [part("Band", [20.0, 20.0, 2.0])];
    let got = unit_sanity(&facts(Some("Ring"), ScaleClass::Macro, &[], &parts));
    assert!(got == vec![r3a_finding("Band", "20", "10", "macro", "bench")]);
}

#[test]
fn r3a_duplicate_name_ignored() {
    let parts = [
        part("Pencil", [3000.0, 8.0, 8.0]),
        part("Pencil", [4000.0, 8.0, 8.0]),
    ];
    let got = unit_sanity(&facts(Some("A pencil box"), ScaleClass::Bench, &[], &parts));
    assert!(got == vec![r3a_finding("Pencil", "3000", "500", "bench", "room")]);
}

#[test]
fn r3a_invalid_part_ignored() {
    // A part with a NaN side is ignored by R3a, but its name is still used,
    // so the later part with the same name is ignored too. No panic.
    let parts = [
        part("Pin", [f64::NAN, 1.0, 1.0]),
        part("Pin", [4000.0, 8.0, 8.0]),
    ];
    let got = unit_sanity(&facts(Some("A pencil box"), ScaleClass::Bench, &[], &parts));
    assert!(got.is_empty());
}

#[test]
fn r3b_design_too_small() {
    // Worked example: the design's largest part is 1.2 mm; a tenth of the
    // Furniture lower bound is 30 mm.
    let parts = [
        part("Side", [0.018, 0.45, 0.9]),
        part("Top", [0.018, 0.5, 1.2]),
    ];
    let got = unit_sanity(&facts(
        Some("A dresser"),
        ScaleClass::Furniture,
        &[],
        &parts,
    ));
    assert!(got == vec![r3b_finding("1.2", "300", "furniture", "macro")]);
}

#[test]
fn r3b_room_example() {
    // Worked example: 3 mm < 50 (a tenth of the Room lower bound).
    let parts = [part("Wall", [3.0, 2.4, 0.1])];
    let got = unit_sanity(&facts(Some("Room"), ScaleClass::Room, &[], &parts));
    assert!(got == vec![r3b_finding("3", "500", "room", "macro")]);
}

#[test]
fn r3b_macro_never() {
    // Worked example: R3b never for Macro.
    let parts = [part("Band", [2.0, 2.0, 0.5])];
    let got = unit_sanity(&facts(Some("Ring"), ScaleClass::Macro, &[], &parts));
    assert!(got.is_empty());
}

#[test]
fn r3b_small_part_not_flagged() {
    // Worked example: a screw in a dresser is normal; the design's size is
    // the largest part (900 mm), not the smallest.
    let parts = [
        part("Side", [18.0, 450.0, 900.0]),
        part("Screw", [4.0, 4.0, 40.0]),
    ];
    let got = unit_sanity(&facts(
        Some("A dresser"),
        ScaleClass::Furniture,
        &[],
        &parts,
    ));
    assert!(got.is_empty());
}

#[test]
fn r3b_no_parts() {
    // Worked example: any facts with no parts give no R3 finding.
    let got = unit_sanity(&facts(Some("A dresser"), ScaleClass::Furniture, &[], &[]));
    assert!(got.is_empty());
}

#[test]
fn r3b_all_parts_invalid() {
    // No part has a largest_mm, so there is no design size and no R3b.
    let parts = [
        part("Pin", [f64::NAN, 1.0, 1.0]),
        part("Screw", [0.0, 4.0, 4.0]),
        part("Side", [-1.0, 450.0, 900.0]),
    ];
    let got = unit_sanity(&facts(
        Some("A dresser"),
        ScaleClass::Furniture,
        &[],
        &parts,
    ));
    assert!(got.is_empty());
}

#[test]
fn e1_goal_none_bench_empty() {
    // Worked example: goal None, Bench, no prompts, no parts -> [R1].
    let got = evaluate(&facts(None, ScaleClass::Bench, &[], &[]));
    assert!(got == vec![r1_finding()]);
}

#[test]
fn e1_goal_blank() {
    // Worked example: goal Some("   ") -> [R1].
    let got = evaluate(&facts(Some("   "), ScaleClass::Bench, &[], &[]));
    assert!(got == vec![r1_finding()]);
}

#[test]
fn e1_pencil_box() {
    // Worked example: [R3a: "Pencil is 3000 mm", suggested room].
    let parts = [part("Pencil", [3000.0, 8.0, 8.0])];
    let got = evaluate(&facts(Some("A pencil box"), ScaleClass::Bench, &[], &parts));
    assert!(got == vec![r3a_finding("Pencil", "3000", "500", "bench", "room")]);
}

#[test]
fn e1_dresser_clean() {
    // Worked example: a normal dresser -> [].
    let parts = [
        part("Side", [18.0, 450.0, 900.0]),
        part("Screw", [4.0, 4.0, 40.0]),
    ];
    let got = evaluate(&facts(
        Some("A dresser"),
        ScaleClass::Furniture,
        &[],
        &parts,
    ));
    assert!(got.is_empty());
}

#[test]
fn e1_dresser_mm() {
    // Worked example: a millimetre-sized dresser -> [R3b, suggested macro].
    let parts = [
        part("Side", [0.018, 0.45, 0.9]),
        part("Top", [0.018, 0.5, 1.2]),
    ];
    let got = evaluate(&facts(
        Some("A dresser"),
        ScaleClass::Furniture,
        &[],
        &parts,
    ));
    assert!(got == vec![r3b_finding("1.2", "300", "furniture", "macro")]);
}

#[test]
fn e1_shelf_inclusive() {
    // Worked example: 2500 is inside, inclusive -> [].
    let parts = [part("Board", [18.0, 200.0, 2500.0])];
    let got = evaluate(&facts(Some("Shelf"), ScaleClass::Furniture, &[], &parts));
    assert!(got.is_empty());
}

#[test]
fn e1_shelf_over() {
    // Worked example: 2500.5 is over -> [R3a, suggested room].
    let parts = [part("Board", [18.0, 200.0, 2500.5])];
    let got = evaluate(&facts(Some("Shelf"), ScaleClass::Furniture, &[], &parts));
    assert!(got == vec![r3a_finding("Board", "2500.5", "2500", "furniture", "room")]);
}

#[test]
fn e1_ring_macro_clean() {
    // Worked example: a 2 mm ring band -> [] (R3b never for Macro).
    let parts = [part("Band", [2.0, 2.0, 0.5])];
    let got = evaluate(&facts(Some("Ring"), ScaleClass::Macro, &[], &parts));
    assert!(got.is_empty());
}

#[test]
fn e1_ring_macro_big() {
    // Worked example: a 20 mm band on a Macro project -> [R3a, suggested bench].
    let parts = [part("Band", [20.0, 20.0, 2.0])];
    let got = evaluate(&facts(Some("Ring"), ScaleClass::Macro, &[], &parts));
    assert!(got == vec![r3a_finding("Band", "20", "10", "macro", "bench")]);
}

#[test]
fn e1_prompts_worked() {
    // Worked example: -> [R2 for a].
    let prompts = [
        prompt("a", true, true),
        prompt("b", true, false),
        prompt("c", false, true),
        prompt("a", true, true),
    ];
    let got = evaluate(&facts(Some("Box"), ScaleClass::Bench, &prompts, &[]));
    assert!(got == vec![r2_finding("a")]);
}

#[test]
fn e1_mixed_all() {
    // Worked example: -> [R1, R2 for p, R3a for the first Pencil].
    let prompts = [prompt("p", true, true)];
    let parts = [
        part("Pin", [f64::NAN, 1.0, 1.0]),
        part("Pencil", [3000.0, 8.0, 8.0]),
        part("Pencil", [4000.0, 8.0, 8.0]),
    ];
    let got = evaluate(&facts(None, ScaleClass::Bench, &prompts, &parts));
    assert!(
        got == vec![
            r1_finding(),
            r2_finding("p"),
            r3a_finding("Pencil", "3000", "500", "bench", "room"),
        ]
    );
}

#[test]
fn e1_room_small() {
    // Worked example: -> [R3b: 3 mm < 50].
    let parts = [part("Wall", [3.0, 2.4, 0.1])];
    let got = evaluate(&facts(Some("Room"), ScaleClass::Room, &[], &parts));
    assert!(got == vec![r3b_finding("3", "500", "room", "macro")]);
}

#[test]
fn e1_no_parts_no_r3() {
    // Worked example: any facts with no parts -> no R3 finding.
    let got = evaluate(&facts(Some("A dresser"), ScaleClass::Furniture, &[], &[]));
    assert!(got.is_empty());
}

#[test]
fn e1_order_r1_before_r3b() {
    // R1's finding comes before R3b's.
    let parts = [part("Wall", [3.0, 2.4, 0.1])];
    let got = evaluate(&facts(None, ScaleClass::Room, &[], &parts));
    assert!(got == vec![r1_finding(), r3b_finding("3", "500", "room", "macro")]);
}

#[test]
fn e1_rules_constant_order() {
    assert!(RULES == [RULE_INTENT_MISSING, RULE_PROMPT_SKIPPED, RULE_UNIT_SANITY]);
}
