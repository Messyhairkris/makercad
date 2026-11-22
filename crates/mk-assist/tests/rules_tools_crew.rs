//! Tests for mk_assist::rules_tools written by Deepak (the Crew, campaign mk-core-19) from the
//! contract alone, without seeing the module.

//! Integration tests for the tools rule pack (doc 07 section 7.5), written from the contract.

use mk_assist::findings::{Finding, Kind, Location, Severity, Source};
use mk_assist::rules_tools::{
    evaluate, family, overlaps, usable, Access, Noise, Slot, Step, Tool, ToolFacts, RULES,
    RULE_ACCURACY, RULE_AWAY, RULE_MISSING, RULE_NOISE,
};

// ---- builders ---------------------------------------------------------------------------

fn slot(day: u8, start_minute: u16, end_minute: u16) -> Slot {
    Slot {
        day,
        start_minute,
        end_minute,
    }
}

fn tool(
    id: &str,
    supplies: &[&str],
    access: Access,
    accuracy_mm: Option<f64>,
    noise: Noise,
) -> Tool {
    Tool {
        id: id.to_string(),
        supplies: supplies.iter().map(|s| s.to_string()).collect(),
        access,
        accuracy_mm,
        noise,
    }
}

fn step(id: u32, needs: &[&str], tolerance_mm: Option<f64>, planned: Option<Slot>) -> Step {
    Step {
        id,
        needs: needs.iter().map(|s| s.to_string()).collect(),
        tolerance_mm,
        planned,
    }
}

fn tool_facts(steps: Vec<Step>, tools: Vec<Tool>, quiet_hours: Vec<Slot>) -> ToolFacts {
    ToolFacts {
        steps,
        tools,
        quiet_hours,
    }
}

#[allow(clippy::too_many_arguments)]
fn finding(
    kind: Kind,
    severity: Severity,
    rule: &str,
    step_id: u32,
    what: &str,
    why: &str,
    fix: Option<&str>,
    facts: &[(&str, &str)],
) -> Finding {
    Finding {
        kind,
        severity,
        source: Source::Rule(rule.to_string()),
        location: Location::Step(step_id),
        what: what.to_string(),
        why: why.to_string(),
        fix: fix.map(|s| s.to_string()),
        facts: facts
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
    }
}

// The contract's words, spelled once each so every expected value is the same text.
fn missing_why(id: u32, c: &str) -> String {
    format!(
        "Step {id} needs {c}, and none of your tools does that (a wishlist tool does not count)."
    )
}

fn away_why(id: u32, c: &str, place: &str) -> String {
    format!("Step {id} needs {c}: you have it at a {place}, not at home.")
}

fn accuracy_why(id: u32, c: &str, t: &str, tool: &str, a: &str) -> String {
    format!(
        "Step {id} needs {c} within {t} mm; {tool} holds about {a} mm. \
         Clamp a guide, or use a finer tool."
    )
}

fn noise_why(id: u32, tool: &str) -> String {
    format!("Step {id} is planned when you asked for quiet, and {tool} is loud.")
}

// ---- the contract's worked example ------------------------------------------------------

fn example_tools() -> Vec<Tool> {
    vec![
        tool(
            "jig",
            &["cut.curve", "cut.straight:rough", "cut.sheet"],
            Access::Owned,
            Some(1.0),
            Noise::Loud,
        ),
        tool(
            "saw",
            &["cut.straight:rough", "cut.crosscut"],
            Access::Owned,
            Some(2.0),
            Noise::Quiet,
        ),
        tool(
            "laser",
            &["cut.laser"],
            Access::Makerspace,
            Some(0.1),
            Noise::Moderate,
        ),
        tool("cnc", &["shape.mill"], Access::Rentable, None, Noise::Loud),
        tool(
            "lathe",
            &["shape.lathe"],
            Access::Wishlist,
            Some(0.05),
            Noise::Moderate,
        ),
    ]
}

fn example_quiet_hours() -> Vec<Slot> {
    vec![slot(2, 1260, 1440)]
}

fn example_step(n: u32) -> Step {
    match n {
        1 => step(1, &["drill.hole"], None, None),
        2 => step(2, &["shape.lathe"], None, None),
        3 => step(3, &["cut.laser"], None, None),
        4 => step(4, &["shape.mill"], None, None),
        5 => step(5, &["cut.straight"], Some(0.5), None),
        6 => step(6, &["cut.straight"], Some(1.0), None),
        7 => step(7, &["cut.straight"], Some(0.5), Some(slot(2, 1300, 1400))),
        8 => step(8, &["cut.curve"], None, Some(slot(2, 1300, 1400))),
        9 => step(9, &["cut.curve"], None, Some(slot(2, 1200, 1260))),
        10 => step(
            10,
            &["cut.crosscut", "drill.hole", "cut.crosscut"],
            None,
            None,
        ),
        11 => step(11, &["cut.laser"], Some(0.05), None),
        12 => step(12, &[], None, None),
        _ => panic!("no example step {n}"),
    }
}

fn example_facts(steps: Vec<Step>) -> ToolFacts {
    tool_facts(steps, example_tools(), example_quiet_hours())
}

// ---- F1: family -------------------------------------------------------------------------

#[test]
fn f1_family_is_the_text_before_the_first_colon() {
    assert_eq!(family("cut.straight:rough"), "cut.straight");
    assert_eq!(family("cut.straight"), "cut.straight");
    assert_eq!(family("a:b:c"), "a");
    assert_eq!(family(":x"), "");
    assert_eq!(family(""), "");
}

#[test]
fn f1_a_tool_supplies_a_capability_through_its_family() {
    // jig writes "cut.straight:rough"; a step needing "cut.straight" is covered by it.
    let facts = example_facts(vec![step(1, &["cut.straight"], None, None)]);
    assert_eq!(evaluate(&facts), Vec::<Finding>::new());
}

#[test]
fn f1_a_qualified_capability_is_not_supplied_by_the_bare_family() {
    // A tool writing "cut.straight" has family "cut.straight", which is not "cut.straight:rough".
    let tools = vec![tool(
        "knife",
        &["cut.straight"],
        Access::Owned,
        None,
        Noise::Quiet,
    )];
    let facts = tool_facts(
        vec![step(1, &["cut.straight:rough"], None, None)],
        tools,
        vec![],
    );
    assert_eq!(
        evaluate(&facts),
        vec![finding(
            Kind::Inventory,
            Severity::Warning,
            RULE_MISSING,
            1,
            "No tool for cut.straight:rough",
            &missing_why(1, "cut.straight:rough"),
            Some("inventory.add_tool"),
            &[("step", "1"), ("capability", "cut.straight:rough")],
        )]
    );
}

// ---- F2: usable -------------------------------------------------------------------------

#[test]
fn f2_usable_is_true_for_owned_shared_makerspace_and_rentable() {
    assert!(usable(Access::Owned));
    assert!(usable(Access::Shared));
    assert!(usable(Access::Makerspace));
    assert!(usable(Access::Rentable));
}

#[test]
fn f2_usable_is_false_for_wishlist() {
    assert!(!usable(Access::Wishlist));
}

// ---- F3: overlaps -----------------------------------------------------------------------

#[test]
fn f3_overlaps_true_for_partial_overlap_on_the_same_day() {
    assert!(overlaps(slot(2, 1300, 1400), slot(2, 1260, 1440)));
    assert!(overlaps(slot(2, 1260, 1440), slot(2, 1300, 1400)));
}

#[test]
fn f3_overlaps_false_when_ends_touch() {
    assert!(!overlaps(slot(2, 1200, 1260), slot(2, 1260, 1440)));
    assert!(!overlaps(slot(2, 1260, 1440), slot(2, 1200, 1260)));
}

#[test]
fn f3_overlaps_false_on_a_different_day() {
    assert!(!overlaps(slot(1, 1300, 1400), slot(2, 1260, 1440)));
    assert!(!overlaps(slot(2, 1260, 1440), slot(1, 1300, 1400)));
}

#[test]
fn f3_overlaps_true_for_identical_and_contained_slots() {
    assert!(overlaps(slot(3, 600, 700), slot(3, 600, 700)));
    assert!(overlaps(slot(3, 600, 700), slot(3, 650, 660)));
    assert!(overlaps(slot(3, 650, 660), slot(3, 600, 700)));
}

// ---- T1: tools.missing ------------------------------------------------------------------

#[test]
fn t1_no_tool_for_the_capability_is_an_inventory_warning() {
    let tools = vec![tool(
        "jig",
        &["cut.curve"],
        Access::Owned,
        Some(1.0),
        Noise::Loud,
    )];
    let facts = tool_facts(vec![step(7, &["drill.hole"], None, None)], tools, vec![]);
    assert_eq!(
        evaluate(&facts),
        vec![finding(
            Kind::Inventory,
            Severity::Warning,
            RULE_MISSING,
            7,
            "No tool for drill.hole",
            &missing_why(7, "drill.hole"),
            Some("inventory.add_tool"),
            &[("step", "7"), ("capability", "drill.hole")],
        )]
    );
}

#[test]
fn t1_a_wishlist_tool_does_not_count() {
    // The only lathe is on the wishlist, so the capability is missing, not a trip.
    let facts = example_facts(vec![example_step(2)]);
    assert_eq!(
        evaluate(&facts),
        vec![finding(
            Kind::Inventory,
            Severity::Warning,
            RULE_MISSING,
            2,
            "No tool for shape.lathe",
            &missing_why(2, "shape.lathe"),
            Some("inventory.add_tool"),
            &[("step", "2"), ("capability", "shape.lathe")],
        )]
    );
}

// ---- T2: tools.away ---------------------------------------------------------------------

#[test]
fn t2_makerspace_only_is_a_tip_about_a_trip() {
    let facts = example_facts(vec![example_step(3)]);
    assert_eq!(
        evaluate(&facts),
        vec![finding(
            Kind::Tip,
            Severity::Info,
            RULE_AWAY,
            3,
            "cut.laser means a trip",
            &away_why(3, "cut.laser", "makerspace"),
            None,
            &[
                ("step", "3"),
                ("capability", "cut.laser"),
                ("access", "makerspace")
            ],
        )]
    );
}

#[test]
fn t2_rentable_only_says_rental() {
    let facts = example_facts(vec![example_step(4)]);
    assert_eq!(
        evaluate(&facts),
        vec![finding(
            Kind::Tip,
            Severity::Info,
            RULE_AWAY,
            4,
            "shape.mill means a trip",
            &away_why(4, "shape.mill", "rental"),
            None,
            &[
                ("step", "4"),
                ("capability", "shape.mill"),
                ("access", "rental")
            ],
        )]
    );
}

#[test]
fn t2_makerspace_and_rentable_together_say_makerspace() {
    // {where} is "makerspace" when any tool for c is Makerspace, whatever the input order.
    let tools = vec![
        tool("cnc", &["shape.mill"], Access::Rentable, None, Noise::Loud),
        tool(
            "mill",
            &["shape.mill"],
            Access::Makerspace,
            None,
            Noise::Loud,
        ),
    ];
    let facts = tool_facts(vec![step(1, &["shape.mill"], None, None)], tools, vec![]);
    assert_eq!(
        evaluate(&facts),
        vec![finding(
            Kind::Tip,
            Severity::Info,
            RULE_AWAY,
            1,
            "shape.mill means a trip",
            &away_why(1, "shape.mill", "makerspace"),
            None,
            &[
                ("step", "1"),
                ("capability", "shape.mill"),
                ("access", "makerspace")
            ],
        )]
    );
}

#[test]
fn t2_an_owned_tool_means_no_trip_finding() {
    let facts = example_facts(vec![step(1, &["cut.crosscut"], None, None)]);
    assert_eq!(evaluate(&facts), Vec::<Finding>::new());
}

// ---- T3: tools.accuracy -----------------------------------------------------------------

#[test]
fn t3_accuracy_worse_than_tolerance_is_not_suitable() {
    let facts = example_facts(vec![example_step(5)]);
    assert_eq!(
        evaluate(&facts),
        vec![finding(
            Kind::NotSuitable,
            Severity::Warning,
            RULE_ACCURACY,
            5,
            "jig is not accurate enough",
            &accuracy_why(5, "cut.straight", "0.5", "jig", "1"),
            None,
            &[
                ("step", "5"),
                ("capability", "cut.straight"),
                ("tool", "jig"),
                ("accuracy_mm", "1"),
                ("tolerance_mm", "0.5"),
            ],
        )]
    );
}

#[test]
fn t3_accuracy_equal_to_tolerance_is_fine() {
    // 1 is not > 1.
    let facts = example_facts(vec![example_step(6)]);
    assert_eq!(evaluate(&facts), Vec::<Finding>::new());
}

#[test]
fn t3_uses_the_smallest_accuracy_and_the_first_tool_holding_it() {
    // Smallest accuracy is 0.4, held by fine_a and fine_b; fine_a comes first in input order.
    let tools = vec![
        tool(
            "coarse",
            &["cut.straight"],
            Access::Owned,
            Some(2.0),
            Noise::Quiet,
        ),
        tool(
            "fine_a",
            &["cut.straight"],
            Access::Owned,
            Some(0.4),
            Noise::Quiet,
        ),
        tool(
            "fine_b",
            &["cut.straight"],
            Access::Owned,
            Some(0.4),
            Noise::Quiet,
        ),
    ];
    let facts = tool_facts(
        vec![step(1, &["cut.straight"], Some(0.2), None)],
        tools,
        vec![],
    );
    assert_eq!(
        evaluate(&facts),
        vec![finding(
            Kind::NotSuitable,
            Severity::Warning,
            RULE_ACCURACY,
            1,
            "fine_a is not accurate enough",
            &accuracy_why(1, "cut.straight", "0.2", "fine_a", "0.4"),
            None,
            &[
                ("step", "1"),
                ("capability", "cut.straight"),
                ("tool", "fine_a"),
                ("accuracy_mm", "0.4"),
                ("tolerance_mm", "0.2"),
            ],
        )]
    );
}

#[test]
fn t3_one_accurate_tool_is_enough() {
    // The smallest accuracy (0.1) is within the 0.5 mm tolerance, so no finding.
    let tools = vec![
        tool(
            "jig",
            &["cut.straight"],
            Access::Owned,
            Some(1.0),
            Noise::Loud,
        ),
        tool(
            "knife",
            &["cut.straight"],
            Access::Owned,
            Some(0.1),
            Noise::Quiet,
        ),
    ];
    let facts = tool_facts(
        vec![step(1, &["cut.straight"], Some(0.5), None)],
        tools,
        vec![],
    );
    assert_eq!(evaluate(&facts), Vec::<Finding>::new());
}

#[test]
fn t3_a_tool_without_an_accuracy_is_ignored() {
    let tools = vec![tool(
        "cnc",
        &["shape.mill"],
        Access::Owned,
        None,
        Noise::Quiet,
    )];
    let facts = tool_facts(
        vec![step(1, &["shape.mill"], Some(0.01), None)],
        tools,
        vec![],
    );
    assert_eq!(evaluate(&facts), Vec::<Finding>::new());
}

#[test]
fn t3_nan_and_infinite_tolerances_count_as_not_given() {
    let tools = vec![tool(
        "jig",
        &["cut.straight"],
        Access::Owned,
        Some(1.0),
        Noise::Quiet,
    )];
    for t in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 0.0, -1.0] {
        let facts = tool_facts(
            vec![step(1, &["cut.straight"], Some(t), None)],
            tools.clone(),
            vec![],
        );
        assert_eq!(evaluate(&facts), Vec::<Finding>::new(), "tolerance {t}");
    }
}

#[test]
fn t3_nan_accuracy_counts_as_not_given() {
    let tools = vec![tool(
        "jig",
        &["cut.straight"],
        Access::Owned,
        Some(f64::NAN),
        Noise::Quiet,
    )];
    let facts = tool_facts(
        vec![step(1, &["cut.straight"], Some(0.5), None)],
        tools,
        vec![],
    );
    assert_eq!(evaluate(&facts), Vec::<Finding>::new());
}

// ---- T4: tools.noise --------------------------------------------------------------------

#[test]
fn t4_a_loud_tool_in_quiet_hours_is_denied() {
    let facts = example_facts(vec![example_step(8)]);
    assert_eq!(
        evaluate(&facts),
        vec![finding(
            Kind::Denied,
            Severity::Warning,
            RULE_NOISE,
            8,
            "jig is loud in your quiet hours",
            &noise_why(8, "jig"),
            Some("plan.move_step"),
            &[
                ("step", "8"),
                ("capability", "cut.curve"),
                ("tool", "jig"),
                ("day", "2"),
            ],
        )]
    );
}

#[test]
fn t4_touching_quiet_hours_is_not_an_overlap() {
    let facts = example_facts(vec![example_step(9)]);
    assert_eq!(evaluate(&facts), Vec::<Finding>::new());
}

#[test]
fn t4_a_quiet_tool_in_quiet_hours_is_fine() {
    // saw is Quiet, so a planned cut.crosscut in the quiet hours gives nothing.
    let facts = example_facts(vec![step(
        1,
        &["cut.crosscut"],
        None,
        Some(slot(2, 1300, 1400)),
    )]);
    assert_eq!(evaluate(&facts), Vec::<Finding>::new());
}

#[test]
fn t4_a_mixed_noise_set_is_not_every_tool_loud() {
    // jig is Loud but saw is Quiet, so not every tool for cut.straight is loud.
    let facts = example_facts(vec![step(
        1,
        &["cut.straight"],
        None,
        Some(slot(2, 1300, 1400)),
    )]);
    assert_eq!(evaluate(&facts), Vec::<Finding>::new());
}

#[test]
fn t4_no_planned_slot_means_no_noise_finding() {
    let facts = example_facts(vec![step(1, &["cut.curve"], None, None)]);
    assert_eq!(evaluate(&facts), Vec::<Finding>::new());
}

#[test]
fn t4_names_the_first_tool_for_the_capability() {
    let tools = vec![
        tool("router", &["cut.curve"], Access::Owned, None, Noise::Loud),
        tool("jig", &["cut.curve"], Access::Owned, None, Noise::Loud),
    ];
    let facts = tool_facts(
        vec![step(1, &["cut.curve"], None, Some(slot(4, 60, 120)))],
        tools,
        vec![slot(4, 0, 600)],
    );
    assert_eq!(
        evaluate(&facts),
        vec![finding(
            Kind::Denied,
            Severity::Warning,
            RULE_NOISE,
            1,
            "router is loud in your quiet hours",
            &noise_why(1, "router"),
            Some("plan.move_step"),
            &[
                ("step", "1"),
                ("capability", "cut.curve"),
                ("tool", "router"),
                ("day", "4"),
            ],
        )]
    );
}

// ---- E1: order, first match, repeated capabilities --------------------------------------

#[test]
fn e1_accuracy_is_tried_before_noise() {
    // Step 7: jig is both too coarse and loud, but T3 comes first.
    let facts = example_facts(vec![example_step(7)]);
    assert_eq!(
        evaluate(&facts),
        vec![finding(
            Kind::NotSuitable,
            Severity::Warning,
            RULE_ACCURACY,
            7,
            "jig is not accurate enough",
            &accuracy_why(7, "cut.straight", "0.5", "jig", "1"),
            None,
            &[
                ("step", "7"),
                ("capability", "cut.straight"),
                ("tool", "jig"),
                ("accuracy_mm", "1"),
                ("tolerance_mm", "0.5"),
            ],
        )]
    );
}

#[test]
fn e1_away_is_tried_before_accuracy() {
    // Step 11: the laser is accurate enough, but T2 comes first.
    let facts = example_facts(vec![example_step(11)]);
    assert_eq!(
        evaluate(&facts),
        vec![finding(
            Kind::Tip,
            Severity::Info,
            RULE_AWAY,
            11,
            "cut.laser means a trip",
            &away_why(11, "cut.laser", "makerspace"),
            None,
            &[
                ("step", "11"),
                ("capability", "cut.laser"),
                ("access", "makerspace")
            ],
        )]
    );
}

#[test]
fn e1_missing_is_tried_before_away() {
    // Step 2: the only lathe is on the wishlist, so it is missing, not a trip.
    let facts = example_facts(vec![example_step(2)]);
    assert_eq!(
        evaluate(&facts),
        vec![finding(
            Kind::Inventory,
            Severity::Warning,
            RULE_MISSING,
            2,
            "No tool for shape.lathe",
            &missing_why(2, "shape.lathe"),
            Some("inventory.add_tool"),
            &[("step", "2"), ("capability", "shape.lathe")],
        )]
    );
}

#[test]
fn e1_a_repeated_capability_is_checked_once() {
    // Step 10: cut.crosscut twice (fine both times) and drill.hole once (missing).
    let facts = example_facts(vec![example_step(10)]);
    assert_eq!(
        evaluate(&facts),
        vec![finding(
            Kind::Inventory,
            Severity::Warning,
            RULE_MISSING,
            10,
            "No tool for drill.hole",
            &missing_why(10, "drill.hole"),
            Some("inventory.add_tool"),
            &[("step", "10"), ("capability", "drill.hole")],
        )]
    );
}

#[test]
fn e1_a_repeated_capability_gives_one_finding_not_two() {
    let facts = example_facts(vec![step(1, &["cut.laser", "cut.laser"], None, None)]);
    assert_eq!(
        evaluate(&facts),
        vec![finding(
            Kind::Tip,
            Severity::Info,
            RULE_AWAY,
            1,
            "cut.laser means a trip",
            &away_why(1, "cut.laser", "makerspace"),
            None,
            &[
                ("step", "1"),
                ("capability", "cut.laser"),
                ("access", "makerspace")
            ],
        )]
    );
}

#[test]
fn e1_steps_are_visited_in_input_order() {
    let facts = example_facts(vec![example_step(3), example_step(1)]);
    assert_eq!(
        evaluate(&facts),
        vec![
            finding(
                Kind::Tip,
                Severity::Info,
                RULE_AWAY,
                3,
                "cut.laser means a trip",
                &away_why(3, "cut.laser", "makerspace"),
                None,
                &[
                    ("step", "3"),
                    ("capability", "cut.laser"),
                    ("access", "makerspace")
                ],
            ),
            finding(
                Kind::Inventory,
                Severity::Warning,
                RULE_MISSING,
                1,
                "No tool for drill.hole",
                &missing_why(1, "drill.hole"),
                Some("inventory.add_tool"),
                &[("step", "1"), ("capability", "drill.hole")],
            ),
        ]
    );
}

#[test]
fn e1_no_steps_gives_no_findings() {
    assert_eq!(evaluate(&ToolFacts::default()), Vec::<Finding>::new());
}

#[test]
fn rules_are_listed_in_the_order_they_are_tried() {
    assert_eq!(RULES, [RULE_MISSING, RULE_AWAY, RULE_ACCURACY, RULE_NOISE]);
    assert_eq!(RULE_MISSING, "tools.missing");
    assert_eq!(RULE_AWAY, "tools.away");
    assert_eq!(RULE_ACCURACY, "tools.accuracy");
    assert_eq!(RULE_NOISE, "tools.noise");
}

// ---- the worked-example table, one row per test -----------------------------------------

#[test]
fn worked_example_step_1_no_tool_for_drill_hole() {
    let facts = example_facts(vec![example_step(1)]);
    assert_eq!(
        evaluate(&facts),
        vec![finding(
            Kind::Inventory,
            Severity::Warning,
            RULE_MISSING,
            1,
            "No tool for drill.hole",
            &missing_why(1, "drill.hole"),
            Some("inventory.add_tool"),
            &[("step", "1"), ("capability", "drill.hole")],
        )]
    );
}

#[test]
fn worked_example_step_2_lathe_is_only_on_the_wishlist() {
    let facts = example_facts(vec![example_step(2)]);
    assert_eq!(
        evaluate(&facts),
        vec![finding(
            Kind::Inventory,
            Severity::Warning,
            RULE_MISSING,
            2,
            "No tool for shape.lathe",
            &missing_why(2, "shape.lathe"),
            Some("inventory.add_tool"),
            &[("step", "2"), ("capability", "shape.lathe")],
        )]
    );
}

#[test]
fn worked_example_step_3_laser_means_a_trip() {
    let facts = example_facts(vec![example_step(3)]);
    assert_eq!(
        evaluate(&facts),
        vec![finding(
            Kind::Tip,
            Severity::Info,
            RULE_AWAY,
            3,
            "cut.laser means a trip",
            &away_why(3, "cut.laser", "makerspace"),
            None,
            &[
                ("step", "3"),
                ("capability", "cut.laser"),
                ("access", "makerspace")
            ],
        )]
    );
}

#[test]
fn worked_example_step_4_mill_is_a_rental() {
    let facts = example_facts(vec![example_step(4)]);
    assert_eq!(
        evaluate(&facts),
        vec![finding(
            Kind::Tip,
            Severity::Info,
            RULE_AWAY,
            4,
            "shape.mill means a trip",
            &away_why(4, "shape.mill", "rental"),
            None,
            &[
                ("step", "4"),
                ("capability", "shape.mill"),
                ("access", "rental")
            ],
        )]
    );
}

#[test]
fn worked_example_step_5_jig_is_too_coarse_for_half_a_millimetre() {
    let facts = example_facts(vec![example_step(5)]);
    assert_eq!(
        evaluate(&facts),
        vec![finding(
            Kind::NotSuitable,
            Severity::Warning,
            RULE_ACCURACY,
            5,
            "jig is not accurate enough",
            &accuracy_why(5, "cut.straight", "0.5", "jig", "1"),
            None,
            &[
                ("step", "5"),
                ("capability", "cut.straight"),
                ("tool", "jig"),
                ("accuracy_mm", "1"),
                ("tolerance_mm", "0.5"),
            ],
        )]
    );
}

#[test]
fn worked_example_step_6_one_millimetre_tolerance_is_met() {
    let facts = example_facts(vec![example_step(6)]);
    assert_eq!(evaluate(&facts), Vec::<Finding>::new());
}

#[test]
fn worked_example_step_7_accuracy_wins_over_noise() {
    let facts = example_facts(vec![example_step(7)]);
    assert_eq!(
        evaluate(&facts),
        vec![finding(
            Kind::NotSuitable,
            Severity::Warning,
            RULE_ACCURACY,
            7,
            "jig is not accurate enough",
            &accuracy_why(7, "cut.straight", "0.5", "jig", "1"),
            None,
            &[
                ("step", "7"),
                ("capability", "cut.straight"),
                ("tool", "jig"),
                ("accuracy_mm", "1"),
                ("tolerance_mm", "0.5"),
            ],
        )]
    );
}

#[test]
fn worked_example_step_8_jig_is_loud_in_quiet_hours() {
    let facts = example_facts(vec![example_step(8)]);
    assert_eq!(
        evaluate(&facts),
        vec![finding(
            Kind::Denied,
            Severity::Warning,
            RULE_NOISE,
            8,
            "jig is loud in your quiet hours",
            &noise_why(8, "jig"),
            Some("plan.move_step"),
            &[
                ("step", "8"),
                ("capability", "cut.curve"),
                ("tool", "jig"),
                ("day", "2"),
            ],
        )]
    );
}

#[test]
fn worked_example_step_9_touching_ends_do_not_overlap() {
    let facts = example_facts(vec![example_step(9)]);
    assert_eq!(evaluate(&facts), Vec::<Finding>::new());
}

#[test]
fn worked_example_step_10_repeated_crosscut_is_checked_once() {
    let facts = example_facts(vec![example_step(10)]);
    assert_eq!(
        evaluate(&facts),
        vec![finding(
            Kind::Inventory,
            Severity::Warning,
            RULE_MISSING,
            10,
            "No tool for drill.hole",
            &missing_why(10, "drill.hole"),
            Some("inventory.add_tool"),
            &[("step", "10"), ("capability", "drill.hole")],
        )]
    );
}

#[test]
fn worked_example_step_11_trip_comes_before_accuracy() {
    let facts = example_facts(vec![example_step(11)]);
    assert_eq!(
        evaluate(&facts),
        vec![finding(
            Kind::Tip,
            Severity::Info,
            RULE_AWAY,
            11,
            "cut.laser means a trip",
            &away_why(11, "cut.laser", "makerspace"),
            None,
            &[
                ("step", "11"),
                ("capability", "cut.laser"),
                ("access", "makerspace")
            ],
        )]
    );
}

#[test]
fn worked_example_step_12_no_needs_gives_nothing() {
    let facts = example_facts(vec![example_step(12)]);
    assert_eq!(evaluate(&facts), Vec::<Finding>::new());
}

#[test]
fn worked_example_all_twelve_steps_together() {
    let steps: Vec<Step> = (1..=12).map(example_step).collect();
    let facts = example_facts(steps);
    assert_eq!(
        evaluate(&facts),
        vec![
            finding(
                Kind::Inventory,
                Severity::Warning,
                RULE_MISSING,
                1,
                "No tool for drill.hole",
                &missing_why(1, "drill.hole"),
                Some("inventory.add_tool"),
                &[("step", "1"), ("capability", "drill.hole")],
            ),
            finding(
                Kind::Inventory,
                Severity::Warning,
                RULE_MISSING,
                2,
                "No tool for shape.lathe",
                &missing_why(2, "shape.lathe"),
                Some("inventory.add_tool"),
                &[("step", "2"), ("capability", "shape.lathe")],
            ),
            finding(
                Kind::Tip,
                Severity::Info,
                RULE_AWAY,
                3,
                "cut.laser means a trip",
                &away_why(3, "cut.laser", "makerspace"),
                None,
                &[
                    ("step", "3"),
                    ("capability", "cut.laser"),
                    ("access", "makerspace")
                ],
            ),
            finding(
                Kind::Tip,
                Severity::Info,
                RULE_AWAY,
                4,
                "shape.mill means a trip",
                &away_why(4, "shape.mill", "rental"),
                None,
                &[
                    ("step", "4"),
                    ("capability", "shape.mill"),
                    ("access", "rental")
                ],
            ),
            finding(
                Kind::NotSuitable,
                Severity::Warning,
                RULE_ACCURACY,
                5,
                "jig is not accurate enough",
                &accuracy_why(5, "cut.straight", "0.5", "jig", "1"),
                None,
                &[
                    ("step", "5"),
                    ("capability", "cut.straight"),
                    ("tool", "jig"),
                    ("accuracy_mm", "1"),
                    ("tolerance_mm", "0.5"),
                ],
            ),
            finding(
                Kind::NotSuitable,
                Severity::Warning,
                RULE_ACCURACY,
                7,
                "jig is not accurate enough",
                &accuracy_why(7, "cut.straight", "0.5", "jig", "1"),
                None,
                &[
                    ("step", "7"),
                    ("capability", "cut.straight"),
                    ("tool", "jig"),
                    ("accuracy_mm", "1"),
                    ("tolerance_mm", "0.5"),
                ],
            ),
            finding(
                Kind::Denied,
                Severity::Warning,
                RULE_NOISE,
                8,
                "jig is loud in your quiet hours",
                &noise_why(8, "jig"),
                Some("plan.move_step"),
                &[
                    ("step", "8"),
                    ("capability", "cut.curve"),
                    ("tool", "jig"),
                    ("day", "2"),
                ],
            ),
            finding(
                Kind::Inventory,
                Severity::Warning,
                RULE_MISSING,
                10,
                "No tool for drill.hole",
                &missing_why(10, "drill.hole"),
                Some("inventory.add_tool"),
                &[("step", "10"), ("capability", "drill.hole")],
            ),
            finding(
                Kind::Tip,
                Severity::Info,
                RULE_AWAY,
                11,
                "cut.laser means a trip",
                &away_why(11, "cut.laser", "makerspace"),
                None,
                &[
                    ("step", "11"),
                    ("capability", "cut.laser"),
                    ("access", "makerspace")
                ],
            ),
        ]
    );
}
