//! Tests for mk_assist::rules_core written by Gordon (the Crew, campaign mk-core-17) from the
//! contract alone, without seeing the module.

//! Integration tests for Sancho's core rule pack (rules_core.rs), written from the contract.

use mk_assist::findings::{Finding, Kind, Location, Severity, Source};
use mk_assist::rules_core::{
    class_for_mm, evaluate, intent_missing, largest_mm, prompt_skipped, unit_sanity, Part,
    ProjectFacts, Prompt, ScaleClass, RULES, RULE_INTENT_MISSING, RULE_PROMPT_SKIPPED,
    RULE_UNIT_SANITY,
};

fn facts_of(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
    pairs
        .iter()
        .map(|&(key, value)| (key.to_string(), value.to_string()))
        .collect()
}

fn part(name: &str, size_mm: [f64; 3]) -> Part {
    Part {
        name: name.to_string(),
        size_mm,
    }
}

fn prompt(id: &str, skipped: bool, relevant_now: bool) -> Prompt {
    Prompt {
        id: id.to_string(),
        skipped,
        relevant_now,
    }
}

fn project(
    goal: Option<&str>,
    scale: ScaleClass,
    prompts: Vec<Prompt>,
    parts: Vec<Part>,
) -> ProjectFacts {
    ProjectFacts {
        goal: goal.map(str::to_string),
        scale,
        prompts,
        parts,
    }
}

// R1's finding, word for word from the contract.
fn r1_finding() -> Finding {
    Finding {
        kind: Kind::Tip,
        severity: Severity::Warning,
        source: Source::Rule(RULE_INTENT_MISSING.to_string()),
        location: Location::Field("intent.goal".to_string()),
        what: "No goal yet".to_string(),
        why: "Sancho checks the design against its goal, and there is none to check against."
            .to_string(),
        fix: Some("intent.edit".to_string()),
        facts: facts_of(&[("goal", "missing")]),
    }
}

// R2's finding for one prompt id.
fn r2_finding(id: &str) -> Finding {
    Finding {
        kind: Kind::Tip,
        severity: Severity::Info,
        source: Source::Rule(RULE_PROMPT_SKIPPED.to_string()),
        location: Location::Field(format!("prompt.{id}")),
        what: "A skipped question matters now".to_string(),
        why: "You skipped this question earlier, and what you are doing now depends on it."
            .to_string(),
        fix: Some("prompt.answer".to_string()),
        facts: facts_of(&[("prompt", id)]),
    }
}

// R3a's finding: `upper` is the class's upper bound, `suggested` the class_for_mm key.
fn r3a_finding(name: &str, size: f64, class: &str, upper: f64, suggested: &str) -> Finding {
    Finding {
        kind: Kind::Tip,
        severity: Severity::Warning,
        source: Source::Rule(RULE_UNIT_SANITY.to_string()),
        location: Location::Entity(name.to_string()),
        what: format!("{name} is {size} mm"),
        why: format!("That is large for a {class} project (up to {upper} mm). Check the units."),
        fix: Some("project.scale".to_string()),
        facts: vec![
            ("part".to_string(), name.to_string()),
            ("largest_mm".to_string(), format!("{size}")),
            ("class".to_string(), class.to_string()),
            ("suggested".to_string(), suggested.to_string()),
        ],
    }
}

// R3b's finding: `lower` is the class's lower bound.
fn r3b_finding(size: f64, class: &str, lower: f64, suggested: &str) -> Finding {
    Finding {
        kind: Kind::Tip,
        severity: Severity::Warning,
        source: Source::Rule(RULE_UNIT_SANITY.to_string()),
        location: Location::Project,
        what: format!("The whole design is {size} mm"),
        why: format!("That is small for a {class} project (from {lower} mm). Check the units."),
        fix: Some("project.scale".to_string()),
        facts: vec![
            ("largest_mm".to_string(), format!("{size}")),
            ("class".to_string(), class.to_string()),
            ("suggested".to_string(), suggested.to_string()),
        ],
    }
}

#[test]
fn c1_scale_class_key() {
    assert_eq!(ScaleClass::Room.key(), "room");
    assert_eq!(ScaleClass::Furniture.key(), "furniture");
    assert_eq!(ScaleClass::Body.key(), "body");
    assert_eq!(ScaleClass::Bench.key(), "bench");
    assert_eq!(ScaleClass::Macro.key(), "macro");
}

#[test]
fn c2_scale_class_range_mm() {
    assert_eq!(ScaleClass::Room.range_mm(), (500.0, 10000.0));
    assert_eq!(ScaleClass::Furniture.range_mm(), (300.0, 2500.0));
    assert_eq!(ScaleClass::Body.range_mm(), (100.0, 1000.0));
    assert_eq!(ScaleClass::Bench.range_mm(), (10.0, 500.0));
    assert_eq!(ScaleClass::Macro.range_mm(), (0.0, 10.0));
}

#[test]
fn c3_largest_mm_takes_the_largest_side() {
    // The long side is in the middle: a version that only reads size_mm[0] misses it.
    assert_eq!(
        largest_mm(&part("Pencil", [8.0, 3000.0, 8.0])),
        Some(3000.0)
    );
    assert_eq!(largest_mm(&part("Cube", [5.0, 5.0, 5.0])), Some(5.0));
    assert_eq!(
        largest_mm(&part("Board", [2500.5, 18.0, 200.0])),
        Some(2500.5)
    );
}

#[test]
fn c3_largest_mm_rejects_non_finite_and_non_positive() {
    assert_eq!(largest_mm(&part("Pin", [f64::NAN, 1.0, 1.0])), None);
    assert_eq!(largest_mm(&part("Pin", [1.0, f64::INFINITY, 1.0])), None);
    assert_eq!(
        largest_mm(&part("Pin", [1.0, 1.0, f64::NEG_INFINITY])),
        None
    );
    assert_eq!(largest_mm(&part("Pin", [0.0, 1.0, 1.0])), None);
    assert_eq!(largest_mm(&part("Pin", [1.0, -1.0, 1.0])), None);
}

#[test]
fn c4_class_for_mm_boundaries() {
    assert_eq!(class_for_mm(9.99), Some(ScaleClass::Macro));
    assert_eq!(class_for_mm(10.0), Some(ScaleClass::Bench));
    assert_eq!(class_for_mm(500.0), Some(ScaleClass::Bench));
    assert_eq!(class_for_mm(500.1), Some(ScaleClass::Furniture));
    assert_eq!(class_for_mm(2500.0), Some(ScaleClass::Furniture));
    assert_eq!(class_for_mm(2500.1), Some(ScaleClass::Room));
}

#[test]
fn c4_class_for_mm_rejects_invalid_sizes() {
    assert_eq!(class_for_mm(0.0), None);
    assert_eq!(class_for_mm(-1.0), None);
    assert_eq!(class_for_mm(f64::NAN), None);
    assert_eq!(class_for_mm(f64::INFINITY), None);
    assert_eq!(class_for_mm(f64::NEG_INFINITY), None);
}

#[test]
fn c4_class_for_mm_never_suggests_body() {
    for mm in [
        0.5, 5.0, 9.99, 10.0, 100.0, 500.0, 500.1, 1000.0, 2500.0, 2500.1, 10000.0,
    ] {
        assert_ne!(class_for_mm(mm), Some(ScaleClass::Body));
    }
}

#[test]
fn r1_intent_missing_when_goal_is_none() {
    let facts = project(None, ScaleClass::Bench, Vec::new(), Vec::new());
    assert_eq!(intent_missing(&facts), vec![r1_finding()]);
}

#[test]
fn r1_intent_missing_when_goal_is_blank() {
    let facts = project(Some("   "), ScaleClass::Bench, Vec::new(), Vec::new());
    assert_eq!(intent_missing(&facts), vec![r1_finding()]);
}

#[test]
fn r1_intent_missing_is_silent_when_goal_has_text() {
    let facts = project(
        Some("A pencil box"),
        ScaleClass::Bench,
        Vec::new(),
        Vec::new(),
    );
    assert_eq!(intent_missing(&facts), Vec::<Finding>::new());
}

#[test]
fn r2_prompt_skipped_needs_skipped_and_relevant_now() {
    let facts = project(
        Some("Box"),
        ScaleClass::Bench,
        vec![
            prompt("a", true, true),
            prompt("b", true, false),
            prompt("c", false, true),
            prompt("d", false, false),
        ],
        Vec::new(),
    );
    assert_eq!(prompt_skipped(&facts), vec![r2_finding("a")]);
}

#[test]
fn r2_prompt_skipped_ignores_a_repeated_id() {
    let facts = project(
        Some("Box"),
        ScaleClass::Bench,
        vec![
            prompt("a", true, true),
            prompt("a", true, true),
            prompt("b", true, true),
        ],
        Vec::new(),
    );
    assert_eq!(
        prompt_skipped(&facts),
        vec![r2_finding("a"), r2_finding("b")]
    );
}

#[test]
fn r3a_part_too_big_for_the_project() {
    let facts = project(
        Some("A pencil box"),
        ScaleClass::Bench,
        Vec::new(),
        vec![part("Pencil", [3000.0, 8.0, 8.0])],
    );
    assert_eq!(
        unit_sanity(&facts),
        vec![r3a_finding("Pencil", 3000.0, "bench", 500.0, "room")]
    );
}

#[test]
fn r3a_part_at_the_upper_bound_is_inside() {
    let facts = project(
        Some("Shelf"),
        ScaleClass::Furniture,
        Vec::new(),
        vec![part("Board", [18.0, 200.0, 2500.0])],
    );
    assert_eq!(unit_sanity(&facts), Vec::<Finding>::new());
}

#[test]
fn r3a_ignores_a_repeated_part_name() {
    let facts = project(
        Some("A pencil box"),
        ScaleClass::Bench,
        Vec::new(),
        vec![
            part("Pencil", [3000.0, 8.0, 8.0]),
            part("Pencil", [4000.0, 8.0, 8.0]),
        ],
    );
    assert_eq!(
        unit_sanity(&facts),
        vec![r3a_finding("Pencil", 3000.0, "bench", 500.0, "room")]
    );
}

#[test]
fn r3a_body_project_uses_its_own_upper_bound() {
    let facts = project(
        Some("A watch strap"),
        ScaleClass::Body,
        Vec::new(),
        vec![part("Strap", [1500.0, 20.0, 2.0])],
    );
    assert_eq!(
        unit_sanity(&facts),
        vec![r3a_finding("Strap", 1500.0, "body", 1000.0, "furniture")]
    );
}

#[test]
fn r3b_whole_design_far_too_small() {
    let facts = project(
        Some("A dresser"),
        ScaleClass::Furniture,
        Vec::new(),
        vec![
            part("Side", [0.018, 0.45, 0.9]),
            part("Top", [0.018, 0.5, 1.2]),
        ],
    );
    assert_eq!(
        unit_sanity(&facts),
        vec![r3b_finding(1.2, "furniture", 300.0, "macro")]
    );
}

#[test]
fn r3b_never_fires_for_macro() {
    let facts = project(
        Some("Ring"),
        ScaleClass::Macro,
        Vec::new(),
        vec![part("Band", [2.0, 2.0, 0.5])],
    );
    assert_eq!(unit_sanity(&facts), Vec::<Finding>::new());
}

#[test]
fn r3b_needs_at_least_one_usable_part() {
    let facts = project(
        Some("Box"),
        ScaleClass::Bench,
        Vec::new(),
        vec![part("Pin", [f64::NAN, 1.0, 1.0])],
    );
    assert_eq!(unit_sanity(&facts), Vec::<Finding>::new());
}

#[test]
fn r3b_at_a_tenth_of_the_lower_bound_is_not_flagged() {
    // Furniture's lower bound is 300 mm, so 30 mm is exactly a tenth: not "less than".
    let facts = project(
        Some("Shelf"),
        ScaleClass::Furniture,
        Vec::new(),
        vec![part("Board", [30.0, 20.0, 10.0])],
    );
    assert_eq!(unit_sanity(&facts), Vec::<Finding>::new());
}

#[test]
fn r3b_just_below_a_tenth_is_flagged() {
    let facts = project(
        Some("Shelf"),
        ScaleClass::Furniture,
        Vec::new(),
        vec![part("Board", [29.9, 20.0, 10.0])],
    );
    assert_eq!(
        unit_sanity(&facts),
        vec![r3b_finding(29.9, "furniture", 300.0, "bench")]
    );
}

#[test]
fn e1_evaluate_runs_r1_then_r2_then_r3a_then_r3b() {
    // Worked example: goal None, Bench, prompt p, Pin NaN, Pencil 3000, Pencil 4000.
    let facts = project(
        None,
        ScaleClass::Bench,
        vec![prompt("p", true, true)],
        vec![
            part("Pin", [f64::NAN, 1.0, 1.0]),
            part("Pencil", [3000.0, 8.0, 8.0]),
            part("Pencil", [4000.0, 8.0, 8.0]),
        ],
    );
    assert_eq!(
        evaluate(&facts),
        vec![
            r1_finding(),
            r2_finding("p"),
            r3a_finding("Pencil", 3000.0, "bench", 500.0, "room"),
        ]
    );
}

#[test]
fn e1_evaluate_puts_r3b_last() {
    let facts = project(
        None,
        ScaleClass::Bench,
        vec![prompt("p", true, true)],
        vec![part("Pin", [0.5, 0.5, 0.5])],
    );
    assert_eq!(
        evaluate(&facts),
        vec![
            r1_finding(),
            r2_finding("p"),
            r3b_finding(0.5, "bench", 10.0, "macro"),
        ]
    );
}

#[test]
fn rules_lists_the_three_rule_ids_in_run_order() {
    assert_eq!(RULE_INTENT_MISSING, "core.intent_missing");
    assert_eq!(RULE_PROMPT_SKIPPED, "core.prompt_skipped");
    assert_eq!(RULE_UNIT_SANITY, "core.unit_sanity");
    assert_eq!(
        RULES,
        [RULE_INTENT_MISSING, RULE_PROMPT_SKIPPED, RULE_UNIT_SANITY]
    );
}

#[test]
fn worked_example_no_goal_bench_is_intent_missing() {
    let facts = project(None, ScaleClass::Bench, Vec::new(), Vec::new());
    assert_eq!(evaluate(&facts), vec![r1_finding()]);
}

#[test]
fn worked_example_blank_goal_is_intent_missing() {
    let facts = project(Some("   "), ScaleClass::Bench, Vec::new(), Vec::new());
    assert_eq!(evaluate(&facts), vec![r1_finding()]);
}

#[test]
fn worked_example_pencil_3000_in_a_bench_project() {
    let facts = project(
        Some("A pencil box"),
        ScaleClass::Bench,
        Vec::new(),
        vec![part("Pencil", [3000.0, 8.0, 8.0])],
    );
    assert_eq!(
        evaluate(&facts),
        vec![r3a_finding("Pencil", 3000.0, "bench", 500.0, "room")]
    );
}

#[test]
fn worked_example_dresser_in_mm_is_quiet() {
    let facts = project(
        Some("A dresser"),
        ScaleClass::Furniture,
        Vec::new(),
        vec![
            part("Side", [18.0, 450.0, 900.0]),
            part("Screw", [4.0, 4.0, 40.0]),
        ],
    );
    assert_eq!(evaluate(&facts), Vec::<Finding>::new());
}

#[test]
fn worked_example_dresser_in_metres_is_too_small() {
    let facts = project(
        Some("A dresser"),
        ScaleClass::Furniture,
        Vec::new(),
        vec![
            part("Side", [0.018, 0.45, 0.9]),
            part("Top", [0.018, 0.5, 1.2]),
        ],
    );
    assert_eq!(
        evaluate(&facts),
        vec![r3b_finding(1.2, "furniture", 300.0, "macro")]
    );
}

#[test]
fn worked_example_board_2500_is_inside() {
    let facts = project(
        Some("Shelf"),
        ScaleClass::Furniture,
        Vec::new(),
        vec![part("Board", [18.0, 200.0, 2500.0])],
    );
    assert_eq!(evaluate(&facts), Vec::<Finding>::new());
}

#[test]
fn worked_example_board_2500_5_is_too_big() {
    let facts = project(
        Some("Shelf"),
        ScaleClass::Furniture,
        Vec::new(),
        vec![part("Board", [18.0, 200.0, 2500.5])],
    );
    assert_eq!(
        evaluate(&facts),
        vec![r3a_finding("Board", 2500.5, "furniture", 2500.0, "room")]
    );
}

#[test]
fn worked_example_macro_band_2_is_quiet() {
    let facts = project(
        Some("Ring"),
        ScaleClass::Macro,
        Vec::new(),
        vec![part("Band", [2.0, 2.0, 0.5])],
    );
    assert_eq!(evaluate(&facts), Vec::<Finding>::new());
}

#[test]
fn worked_example_macro_band_20_is_too_big() {
    let facts = project(
        Some("Ring"),
        ScaleClass::Macro,
        Vec::new(),
        vec![part("Band", [20.0, 20.0, 2.0])],
    );
    assert_eq!(
        evaluate(&facts),
        vec![r3a_finding("Band", 20.0, "macro", 10.0, "bench")]
    );
}

#[test]
fn worked_example_prompts_report_only_the_first_skipped_relevant() {
    let facts = project(
        Some("Box"),
        ScaleClass::Bench,
        vec![
            prompt("a", true, true),
            prompt("b", true, false),
            prompt("c", false, true),
            prompt("a", true, true),
        ],
        Vec::new(),
    );
    assert_eq!(evaluate(&facts), vec![r2_finding("a")]);
}

#[test]
fn worked_example_room_wall_3_mm_is_too_small() {
    let facts = project(
        Some("Room"),
        ScaleClass::Room,
        Vec::new(),
        vec![part("Wall", [3.0, 2.4, 0.1])],
    );
    assert_eq!(
        evaluate(&facts),
        vec![r3b_finding(3.0, "room", 500.0, "macro")]
    );
}

#[test]
fn worked_example_no_parts_gives_no_unit_finding() {
    let facts = project(Some("Box"), ScaleClass::Bench, Vec::new(), Vec::new());
    assert_eq!(evaluate(&facts), Vec::<Finding>::new());
}

#[test]
fn odd_sizes_never_panic_and_are_ignored() {
    let facts = project(
        Some("Box"),
        ScaleClass::Bench,
        Vec::new(),
        vec![
            part("A", [f64::NAN, f64::NAN, f64::NAN]),
            part("B", [f64::INFINITY, 1.0, 1.0]),
            part("C", [f64::NEG_INFINITY, 1.0, 1.0]),
            part("D", [0.0, 0.0, 0.0]),
            part("E", [-5.0, -5.0, -5.0]),
        ],
    );
    assert_eq!(evaluate(&facts), Vec::<Finding>::new());
}
