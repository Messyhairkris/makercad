//! Contract tests for mk_assist::rules_tools, written by Qwen as a PA (the Crew) from
//! crew/contracts/mk_rules_tools.md. They were the hidden tests of campaign mk-core-19.

use mk_assist::findings::{Finding, Kind, Location, Severity, Source};
use mk_assist::rules_tools::{
    evaluate, family, overlaps, usable, Access, Noise, Slot, Step, Tool, ToolFacts, RULES,
    RULE_ACCURACY, RULE_AWAY, RULE_MISSING, RULE_NOISE,
};

// One argument per Finding field keeps each expected finding readable as the contract's row.
#[allow(clippy::too_many_arguments)]
fn finding(
    kind: Kind,
    severity: Severity,
    rule: &'static str,
    step: u32,
    what: &str,
    why: &str,
    fix: Option<&'static str>,
    facts: Vec<(String, String)>,
) -> Finding {
    Finding {
        kind,
        severity,
        source: Source::Rule(rule.to_string()),
        location: Location::Step(step),
        what: what.to_string(),
        why: why.to_string(),
        fix: fix.map(str::to_string),
        facts,
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

fn slot(day: u8, start_minute: u16, end_minute: u16) -> Slot {
    Slot {
        day,
        start_minute,
        end_minute,
    }
}

fn worked_tools() -> Vec<Tool> {
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

fn worked_facts(steps: Vec<Step>) -> ToolFacts {
    ToolFacts {
        steps,
        tools: worked_tools(),
        quiet_hours: vec![slot(2, 1260, 1440)],
    }
}

// F1: family

#[test]
fn f1_family_plain_capability() {
    assert_eq!(family("cut.straight"), "cut.straight");
}

#[test]
fn f1_family_strips_the_variant() {
    assert_eq!(family("cut.straight:rough"), "cut.straight");
}

#[test]
fn f1_family_keeps_only_the_text_before_the_first_colon() {
    assert_eq!(family("cut.straight:rough:fine"), "cut.straight");
}

#[test]
fn f1_family_of_an_empty_string_is_empty() {
    assert_eq!(family(""), "");
}

// F2: usable

#[test]
fn f2_owned_is_usable() {
    assert!(usable(Access::Owned));
}

#[test]
fn f2_shared_is_usable() {
    assert!(usable(Access::Shared));
}

#[test]
fn f2_makerspace_is_usable() {
    assert!(usable(Access::Makerspace));
}

#[test]
fn f2_rentable_is_usable() {
    assert!(usable(Access::Rentable));
}

#[test]
fn f2_wishlist_is_not_usable() {
    assert!(!usable(Access::Wishlist));
}

// F3: overlaps

#[test]
fn f3_crossing_stretches_on_the_same_day_overlap() {
    assert!(overlaps(slot(2, 1300, 1400), slot(2, 1260, 1440)));
}

#[test]
fn f3_contained_stretch_overlaps() {
    assert!(overlaps(slot(2, 1300, 1320), slot(2, 1260, 1440)));
}

#[test]
fn f3_same_minutes_on_a_different_day_do_not_overlap() {
    assert!(!overlaps(slot(2, 1300, 1400), slot(3, 1300, 1400)));
}

#[test]
fn f3_touching_ends_do_not_overlap() {
    assert!(!overlaps(slot(2, 1200, 1260), slot(2, 1260, 1440)));
}

#[test]
fn f3_touching_ends_do_not_overlap_the_other_way() {
    assert!(!overlaps(slot(2, 1260, 1440), slot(2, 1440, 1500)));
}

// the rule order

#[test]
fn rules_are_listed_in_the_order_they_are_tried() {
    assert_eq!(RULES, [RULE_MISSING, RULE_AWAY, RULE_ACCURACY, RULE_NOISE]);
    assert_eq!(RULE_MISSING, "tools.missing");
    assert_eq!(RULE_AWAY, "tools.away");
    assert_eq!(RULE_ACCURACY, "tools.accuracy");
    assert_eq!(RULE_NOISE, "tools.noise");
}

// T1: tools.missing

#[test]
fn t1_no_tool_for_the_capability() {
    let facts = ToolFacts {
        steps: vec![step(1, &["drill.hole"], None, None)],
        tools: vec![tool(
            "jig",
            &["cut.curve"],
            Access::Owned,
            Some(1.0),
            Noise::Loud,
        )],
        quiet_hours: vec![],
    };
    let expected = vec![finding(
        Kind::Inventory,
        Severity::Warning,
        RULE_MISSING,
        1,
        "No tool for drill.hole",
        "Step 1 needs drill.hole, and none of your tools does that (a wishlist tool does not count).",
        Some("inventory.add_tool"),
        vec![
            ("step".to_string(), "1".to_string()),
            ("capability".to_string(), "drill.hole".to_string()),
        ],
    )];
    assert_eq!(evaluate(&facts), expected);
}

#[test]
fn t1_a_wishlist_tool_does_not_count() {
    let facts = ToolFacts {
        steps: vec![step(2, &["shape.lathe"], None, None)],
        tools: vec![tool(
            "lathe",
            &["shape.lathe"],
            Access::Wishlist,
            Some(0.05),
            Noise::Moderate,
        )],
        quiet_hours: vec![],
    };
    let expected = vec![finding(
        Kind::Inventory,
        Severity::Warning,
        RULE_MISSING,
        2,
        "No tool for shape.lathe",
        "Step 2 needs shape.lathe, and none of your tools does that (a wishlist tool does not count).",
        Some("inventory.add_tool"),
        vec![
            ("step".to_string(), "2".to_string()),
            ("capability".to_string(), "shape.lathe".to_string()),
        ],
    )];
    assert_eq!(evaluate(&facts), expected);
}

// T2: tools.away

#[test]
fn t2_every_tool_at_a_makerspace() {
    let facts = ToolFacts {
        steps: vec![step(3, &["cut.laser"], None, None)],
        tools: vec![tool(
            "laser",
            &["cut.laser"],
            Access::Makerspace,
            Some(0.1),
            Noise::Moderate,
        )],
        quiet_hours: vec![],
    };
    let expected = vec![finding(
        Kind::Tip,
        Severity::Info,
        RULE_AWAY,
        3,
        "cut.laser means a trip",
        "Step 3 needs cut.laser: you have it at a makerspace, not at home.",
        None,
        vec![
            ("step".to_string(), "3".to_string()),
            ("capability".to_string(), "cut.laser".to_string()),
            ("access".to_string(), "makerspace".to_string()),
        ],
    )];
    assert_eq!(evaluate(&facts), expected);
}

#[test]
fn t2_every_tool_rental() {
    let facts = ToolFacts {
        steps: vec![step(4, &["shape.mill"], None, None)],
        tools: vec![tool(
            "cnc",
            &["shape.mill"],
            Access::Rentable,
            None,
            Noise::Loud,
        )],
        quiet_hours: vec![],
    };
    let expected = vec![finding(
        Kind::Tip,
        Severity::Info,
        RULE_AWAY,
        4,
        "shape.mill means a trip",
        "Step 4 needs shape.mill: you have it at a rental, not at home.",
        None,
        vec![
            ("step".to_string(), "4".to_string()),
            ("capability".to_string(), "shape.mill".to_string()),
            ("access".to_string(), "rental".to_string()),
        ],
    )];
    assert_eq!(evaluate(&facts), expected);
}

#[test]
fn t2_makerspace_wins_the_word_when_both_are_away() {
    let facts = ToolFacts {
        steps: vec![step(5, &["cut.laser"], None, None)],
        tools: vec![
            tool(
                "laser",
                &["cut.laser"],
                Access::Makerspace,
                Some(0.1),
                Noise::Moderate,
            ),
            tool(
                "laser2",
                &["cut.laser"],
                Access::Rentable,
                None,
                Noise::Quiet,
            ),
        ],
        quiet_hours: vec![],
    };
    let expected = vec![finding(
        Kind::Tip,
        Severity::Info,
        RULE_AWAY,
        5,
        "cut.laser means a trip",
        "Step 5 needs cut.laser: you have it at a makerspace, not at home.",
        None,
        vec![
            ("step".to_string(), "5".to_string()),
            ("capability".to_string(), "cut.laser".to_string()),
            ("access".to_string(), "makerspace".to_string()),
        ],
    )];
    assert_eq!(evaluate(&facts), expected);
}

#[test]
fn t2_no_away_when_a_home_tool_supplies_the_capability() {
    let facts = ToolFacts {
        steps: vec![step(6, &["cut.laser"], None, None)],
        tools: vec![
            tool(
                "laser",
                &["cut.laser"],
                Access::Makerspace,
                Some(0.1),
                Noise::Moderate,
            ),
            tool(
                "laser_home",
                &["cut.laser"],
                Access::Owned,
                Some(0.2),
                Noise::Quiet,
            ),
        ],
        quiet_hours: vec![],
    };
    assert!(evaluate(&facts).is_empty());
}

// T3: tools.accuracy

#[test]
fn t3_smallest_accuracy_worse_than_the_tolerance() {
    let facts = ToolFacts {
        steps: vec![step(5, &["cut.straight"], Some(0.5), None)],
        tools: vec![tool(
            "jig",
            &["cut.straight"],
            Access::Owned,
            Some(1.0),
            Noise::Quiet,
        )],
        quiet_hours: vec![],
    };
    let expected = vec![finding(
        Kind::NotSuitable,
        Severity::Warning,
        RULE_ACCURACY,
        5,
        "jig is not accurate enough",
        "Step 5 needs cut.straight within 0.5 mm; jig holds about 1 mm. Clamp a guide, or use a finer tool.",
        None,
        vec![
            ("step".to_string(), "5".to_string()),
            ("capability".to_string(), "cut.straight".to_string()),
            ("tool".to_string(), "jig".to_string()),
            ("accuracy_mm".to_string(), "1".to_string()),
            ("tolerance_mm".to_string(), "0.5".to_string()),
        ],
    )];
    assert_eq!(evaluate(&facts), expected);
}

#[test]
fn t3_accuracy_equal_to_the_tolerance_is_fine() {
    let facts = ToolFacts {
        steps: vec![step(6, &["cut.straight"], Some(1.0), None)],
        tools: vec![tool(
            "jig",
            &["cut.straight"],
            Access::Owned,
            Some(1.0),
            Noise::Quiet,
        )],
        quiet_hours: vec![],
    };
    assert!(evaluate(&facts).is_empty());
}

#[test]
fn t3_accuracy_finer_than_the_tolerance_is_fine() {
    let facts = ToolFacts {
        steps: vec![step(7, &["cut.straight"], Some(0.5), None)],
        tools: vec![tool(
            "fine",
            &["cut.straight"],
            Access::Owned,
            Some(0.1),
            Noise::Quiet,
        )],
        quiet_hours: vec![],
    };
    assert!(evaluate(&facts).is_empty());
}

#[test]
fn t3_no_tolerance_means_no_accuracy_check() {
    let facts = ToolFacts {
        steps: vec![step(8, &["cut.straight"], None, None)],
        tools: vec![tool(
            "jig",
            &["cut.straight"],
            Access::Owned,
            Some(1.0),
            Noise::Quiet,
        )],
        quiet_hours: vec![],
    };
    assert!(evaluate(&facts).is_empty());
}

#[test]
fn t3_zero_tolerance_is_not_a_tolerance() {
    let facts = ToolFacts {
        steps: vec![step(9, &["cut.straight"], Some(0.0), None)],
        tools: vec![tool(
            "jig",
            &["cut.straight"],
            Access::Owned,
            Some(1.0),
            Noise::Quiet,
        )],
        quiet_hours: vec![],
    };
    assert!(evaluate(&facts).is_empty());
}

#[test]
fn t3_names_the_first_tool_with_the_smallest_accuracy() {
    let facts = ToolFacts {
        steps: vec![step(10, &["cut.straight"], Some(0.5), None)],
        tools: vec![
            tool(
                "coarse",
                &["cut.straight"],
                Access::Owned,
                Some(2.0),
                Noise::Quiet,
            ),
            tool(
                "mid",
                &["cut.straight"],
                Access::Owned,
                Some(1.0),
                Noise::Quiet,
            ),
            tool(
                "fine",
                &["cut.straight"],
                Access::Owned,
                Some(1.0),
                Noise::Quiet,
            ),
        ],
        quiet_hours: vec![],
    };
    let expected = vec![finding(
        Kind::NotSuitable,
        Severity::Warning,
        RULE_ACCURACY,
        10,
        "mid is not accurate enough",
        "Step 10 needs cut.straight within 0.5 mm; mid holds about 1 mm. Clamp a guide, or use a finer tool.",
        None,
        vec![
            ("step".to_string(), "10".to_string()),
            ("capability".to_string(), "cut.straight".to_string()),
            ("tool".to_string(), "mid".to_string()),
            ("accuracy_mm".to_string(), "1".to_string()),
            ("tolerance_mm".to_string(), "0.5".to_string()),
        ],
    )];
    assert_eq!(evaluate(&facts), expected);
}

#[test]
fn t3_a_tool_without_an_accuracy_does_not_block_the_rule() {
    let facts = ToolFacts {
        steps: vec![step(11, &["cut.straight"], Some(0.5), None)],
        tools: vec![
            tool(
                "blind",
                &["cut.straight"],
                Access::Owned,
                None,
                Noise::Quiet,
            ),
            tool(
                "jig",
                &["cut.straight"],
                Access::Owned,
                Some(1.0),
                Noise::Quiet,
            ),
        ],
        quiet_hours: vec![],
    };
    let expected = vec![finding(
        Kind::NotSuitable,
        Severity::Warning,
        RULE_ACCURACY,
        11,
        "jig is not accurate enough",
        "Step 11 needs cut.straight within 0.5 mm; jig holds about 1 mm. Clamp a guide, or use a finer tool.",
        None,
        vec![
            ("step".to_string(), "11".to_string()),
            ("capability".to_string(), "cut.straight".to_string()),
            ("tool".to_string(), "jig".to_string()),
            ("accuracy_mm".to_string(), "1".to_string()),
            ("tolerance_mm".to_string(), "0.5".to_string()),
        ],
    )];
    assert_eq!(evaluate(&facts), expected);
}

// T4: tools.noise

#[test]
fn t4_loud_tool_planned_in_quiet_hours() {
    let facts = ToolFacts {
        steps: vec![step(8, &["cut.curve"], None, Some(slot(2, 1300, 1400)))],
        tools: vec![tool(
            "jig",
            &["cut.curve"],
            Access::Owned,
            Some(1.0),
            Noise::Loud,
        )],
        quiet_hours: vec![slot(2, 1260, 1440)],
    };
    let expected = vec![finding(
        Kind::Denied,
        Severity::Warning,
        RULE_NOISE,
        8,
        "jig is loud in your quiet hours",
        "Step 8 is planned when you asked for quiet, and jig is loud.",
        Some("plan.move_step"),
        vec![
            ("step".to_string(), "8".to_string()),
            ("capability".to_string(), "cut.curve".to_string()),
            ("tool".to_string(), "jig".to_string()),
            ("day".to_string(), "2".to_string()),
        ],
    )];
    assert_eq!(evaluate(&facts), expected);
}

#[test]
fn t4_not_loud_means_no_noise_finding() {
    let facts = ToolFacts {
        steps: vec![step(9, &["cut.curve"], None, Some(slot(2, 1300, 1400)))],
        tools: vec![tool(
            "saw",
            &["cut.curve"],
            Access::Owned,
            Some(2.0),
            Noise::Moderate,
        )],
        quiet_hours: vec![slot(2, 1260, 1440)],
    };
    assert!(evaluate(&facts).is_empty());
}

#[test]
fn t4_not_planned_means_no_noise_finding() {
    let facts = ToolFacts {
        steps: vec![step(10, &["cut.curve"], None, None)],
        tools: vec![tool(
            "jig",
            &["cut.curve"],
            Access::Owned,
            Some(1.0),
            Noise::Loud,
        )],
        quiet_hours: vec![slot(2, 1260, 1440)],
    };
    assert!(evaluate(&facts).is_empty());
}

#[test]
fn t4_planned_outside_quiet_hours_is_fine() {
    let facts = ToolFacts {
        steps: vec![step(11, &["cut.curve"], None, Some(slot(2, 1000, 1100)))],
        tools: vec![tool(
            "jig",
            &["cut.curve"],
            Access::Owned,
            Some(1.0),
            Noise::Loud,
        )],
        quiet_hours: vec![slot(2, 1260, 1440)],
    };
    assert!(evaluate(&facts).is_empty());
}

#[test]
fn t4_planned_on_another_day_is_fine() {
    let facts = ToolFacts {
        steps: vec![step(12, &["cut.curve"], None, Some(slot(3, 1300, 1400)))],
        tools: vec![tool(
            "jig",
            &["cut.curve"],
            Access::Owned,
            Some(1.0),
            Noise::Loud,
        )],
        quiet_hours: vec![slot(2, 1260, 1440)],
    };
    assert!(evaluate(&facts).is_empty());
}

#[test]
fn t4_names_the_first_tool_for_the_capability() {
    let facts = ToolFacts {
        steps: vec![step(13, &["cut.curve"], None, Some(slot(2, 1300, 1400)))],
        tools: vec![
            tool("first", &["cut.curve"], Access::Owned, None, Noise::Loud),
            tool("second", &["cut.curve"], Access::Owned, None, Noise::Loud),
        ],
        quiet_hours: vec![slot(2, 1260, 1440)],
    };
    let got = evaluate(&facts);
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].what, "first is loud in your quiet hours");
}

// worked examples, one step at a time

#[test]
fn worked_step_1_no_tool_for_drill_hole() {
    let facts = worked_facts(vec![step(1, &["drill.hole"], None, None)]);
    let expected = vec![finding(
        Kind::Inventory,
        Severity::Warning,
        RULE_MISSING,
        1,
        "No tool for drill.hole",
        "Step 1 needs drill.hole, and none of your tools does that (a wishlist tool does not count).",
        Some("inventory.add_tool"),
        vec![
            ("step".to_string(), "1".to_string()),
            ("capability".to_string(), "drill.hole".to_string()),
        ],
    )];
    assert_eq!(evaluate(&facts), expected);
}

#[test]
fn worked_step_2_the_wishlist_lathe_counts_as_missing() {
    let facts = worked_facts(vec![step(2, &["shape.lathe"], None, None)]);
    let expected = vec![finding(
        Kind::Inventory,
        Severity::Warning,
        RULE_MISSING,
        2,
        "No tool for shape.lathe",
        "Step 2 needs shape.lathe, and none of your tools does that (a wishlist tool does not count).",
        Some("inventory.add_tool"),
        vec![
            ("step".to_string(), "2".to_string()),
            ("capability".to_string(), "shape.lathe".to_string()),
        ],
    )];
    assert_eq!(evaluate(&facts), expected);
}

#[test]
fn worked_step_3_the_laser_means_a_trip_to_the_makerspace() {
    let facts = worked_facts(vec![step(3, &["cut.laser"], None, None)]);
    let expected = vec![finding(
        Kind::Tip,
        Severity::Info,
        RULE_AWAY,
        3,
        "cut.laser means a trip",
        "Step 3 needs cut.laser: you have it at a makerspace, not at home.",
        None,
        vec![
            ("step".to_string(), "3".to_string()),
            ("capability".to_string(), "cut.laser".to_string()),
            ("access".to_string(), "makerspace".to_string()),
        ],
    )];
    assert_eq!(evaluate(&facts), expected);
}

#[test]
fn worked_step_4_the_mill_means_a_trip_to_rental() {
    let facts = worked_facts(vec![step(4, &["shape.mill"], None, None)]);
    let expected = vec![finding(
        Kind::Tip,
        Severity::Info,
        RULE_AWAY,
        4,
        "shape.mill means a trip",
        "Step 4 needs shape.mill: you have it at a rental, not at home.",
        None,
        vec![
            ("step".to_string(), "4".to_string()),
            ("capability".to_string(), "shape.mill".to_string()),
            ("access".to_string(), "rental".to_string()),
        ],
    )];
    assert_eq!(evaluate(&facts), expected);
}

#[test]
fn worked_step_5_the_jig_is_not_accurate_enough() {
    let facts = worked_facts(vec![step(5, &["cut.straight"], Some(0.5), None)]);
    let expected = vec![finding(
        Kind::NotSuitable,
        Severity::Warning,
        RULE_ACCURACY,
        5,
        "jig is not accurate enough",
        "Step 5 needs cut.straight within 0.5 mm; jig holds about 1 mm. Clamp a guide, or use a finer tool.",
        None,
        vec![
            ("step".to_string(), "5".to_string()),
            ("capability".to_string(), "cut.straight".to_string()),
            ("tool".to_string(), "jig".to_string()),
            ("accuracy_mm".to_string(), "1".to_string()),
            ("tolerance_mm".to_string(), "0.5".to_string()),
        ],
    )];
    assert_eq!(evaluate(&facts), expected);
}

#[test]
fn worked_step_6_the_tolerance_equal_to_the_accuracy_is_fine() {
    let facts = worked_facts(vec![step(6, &["cut.straight"], Some(1.0), None)]);
    assert!(evaluate(&facts).is_empty());
}

#[test]
fn worked_step_7_the_accuracy_finding_comes_before_noise() {
    let facts = worked_facts(vec![step(
        7,
        &["cut.straight"],
        Some(0.5),
        Some(slot(2, 1300, 1400)),
    )]);
    let expected = vec![finding(
        Kind::NotSuitable,
        Severity::Warning,
        RULE_ACCURACY,
        7,
        "jig is not accurate enough",
        "Step 7 needs cut.straight within 0.5 mm; jig holds about 1 mm. Clamp a guide, or use a finer tool.",
        None,
        vec![
            ("step".to_string(), "7".to_string()),
            ("capability".to_string(), "cut.straight".to_string()),
            ("tool".to_string(), "jig".to_string()),
            ("accuracy_mm".to_string(), "1".to_string()),
            ("tolerance_mm".to_string(), "0.5".to_string()),
        ],
    )];
    assert_eq!(evaluate(&facts), expected);
}

#[test]
fn worked_step_8_the_jig_is_loud_in_quiet_hours() {
    let facts = worked_facts(vec![step(
        8,
        &["cut.curve"],
        None,
        Some(slot(2, 1300, 1400)),
    )]);
    let expected = vec![finding(
        Kind::Denied,
        Severity::Warning,
        RULE_NOISE,
        8,
        "jig is loud in your quiet hours",
        "Step 8 is planned when you asked for quiet, and jig is loud.",
        Some("plan.move_step"),
        vec![
            ("step".to_string(), "8".to_string()),
            ("capability".to_string(), "cut.curve".to_string()),
            ("tool".to_string(), "jig".to_string()),
            ("day".to_string(), "2".to_string()),
        ],
    )];
    assert_eq!(evaluate(&facts), expected);
}

#[test]
fn worked_step_9_touching_ends_are_not_quiet_hours() {
    let facts = worked_facts(vec![step(
        9,
        &["cut.curve"],
        None,
        Some(slot(2, 1200, 1260)),
    )]);
    assert!(evaluate(&facts).is_empty());
}

#[test]
fn worked_step_10_only_the_missing_capability_is_reported() {
    let facts = worked_facts(vec![step(
        10,
        &["cut.crosscut", "drill.hole", "cut.crosscut"],
        None,
        None,
    )]);
    let expected = vec![finding(
        Kind::Inventory,
        Severity::Warning,
        RULE_MISSING,
        10,
        "No tool for drill.hole",
        "Step 10 needs drill.hole, and none of your tools does that (a wishlist tool does not count).",
        Some("inventory.add_tool"),
        vec![
            ("step".to_string(), "10".to_string()),
            ("capability".to_string(), "drill.hole".to_string()),
        ],
    )];
    assert_eq!(evaluate(&facts), expected);
}

#[test]
fn worked_step_11_the_trip_finding_comes_before_accuracy() {
    let facts = worked_facts(vec![step(11, &["cut.laser"], Some(0.05), None)]);
    let expected = vec![finding(
        Kind::Tip,
        Severity::Info,
        RULE_AWAY,
        11,
        "cut.laser means a trip",
        "Step 11 needs cut.laser: you have it at a makerspace, not at home.",
        None,
        vec![
            ("step".to_string(), "11".to_string()),
            ("capability".to_string(), "cut.laser".to_string()),
            ("access".to_string(), "makerspace".to_string()),
        ],
    )];
    assert_eq!(evaluate(&facts), expected);
}

#[test]
fn worked_step_12_no_needs_no_findings() {
    let facts = worked_facts(vec![step(12, &[], None, None)]);
    assert!(evaluate(&facts).is_empty());
}

#[test]
fn worked_all_twelve_steps_together_in_step_order() {
    let steps = vec![
        step(1, &["drill.hole"], None, None),
        step(2, &["shape.lathe"], None, None),
        step(3, &["cut.laser"], None, None),
        step(4, &["shape.mill"], None, None),
        step(5, &["cut.straight"], Some(0.5), None),
        step(6, &["cut.straight"], Some(1.0), None),
        step(7, &["cut.straight"], Some(0.5), Some(slot(2, 1300, 1400))),
        step(8, &["cut.curve"], None, Some(slot(2, 1300, 1400))),
        step(9, &["cut.curve"], None, Some(slot(2, 1200, 1260))),
        step(
            10,
            &["cut.crosscut", "drill.hole", "cut.crosscut"],
            None,
            None,
        ),
        step(11, &["cut.laser"], Some(0.05), None),
        step(12, &[], None, None),
    ];
    let expected = vec![
        finding(
            Kind::Inventory,
            Severity::Warning,
            RULE_MISSING,
            1,
            "No tool for drill.hole",
            "Step 1 needs drill.hole, and none of your tools does that (a wishlist tool does not count).",
            Some("inventory.add_tool"),
            vec![
                ("step".to_string(), "1".to_string()),
                ("capability".to_string(), "drill.hole".to_string()),
            ],
        ),
        finding(
            Kind::Inventory,
            Severity::Warning,
            RULE_MISSING,
            2,
            "No tool for shape.lathe",
            "Step 2 needs shape.lathe, and none of your tools does that (a wishlist tool does not count).",
            Some("inventory.add_tool"),
            vec![
                ("step".to_string(), "2".to_string()),
                ("capability".to_string(), "shape.lathe".to_string()),
            ],
        ),
        finding(
            Kind::Tip,
            Severity::Info,
            RULE_AWAY,
            3,
            "cut.laser means a trip",
            "Step 3 needs cut.laser: you have it at a makerspace, not at home.",
            None,
            vec![
                ("step".to_string(), "3".to_string()),
                ("capability".to_string(), "cut.laser".to_string()),
                ("access".to_string(), "makerspace".to_string()),
            ],
        ),
        finding(
            Kind::Tip,
            Severity::Info,
            RULE_AWAY,
            4,
            "shape.mill means a trip",
            "Step 4 needs shape.mill: you have it at a rental, not at home.",
            None,
            vec![
                ("step".to_string(), "4".to_string()),
                ("capability".to_string(), "shape.mill".to_string()),
                ("access".to_string(), "rental".to_string()),
            ],
        ),
        finding(
            Kind::NotSuitable,
            Severity::Warning,
            RULE_ACCURACY,
            5,
            "jig is not accurate enough",
            "Step 5 needs cut.straight within 0.5 mm; jig holds about 1 mm. Clamp a guide, or use a finer tool.",
            None,
            vec![
                ("step".to_string(), "5".to_string()),
                ("capability".to_string(), "cut.straight".to_string()),
                ("tool".to_string(), "jig".to_string()),
                ("accuracy_mm".to_string(), "1".to_string()),
                ("tolerance_mm".to_string(), "0.5".to_string()),
            ],
        ),
        finding(
            Kind::NotSuitable,
            Severity::Warning,
            RULE_ACCURACY,
            7,
            "jig is not accurate enough",
            "Step 7 needs cut.straight within 0.5 mm; jig holds about 1 mm. Clamp a guide, or use a finer tool.",
            None,
            vec![
                ("step".to_string(), "7".to_string()),
                ("capability".to_string(), "cut.straight".to_string()),
                ("tool".to_string(), "jig".to_string()),
                ("accuracy_mm".to_string(), "1".to_string()),
                ("tolerance_mm".to_string(), "0.5".to_string()),
            ],
        ),
        finding(
            Kind::Denied,
            Severity::Warning,
            RULE_NOISE,
            8,
            "jig is loud in your quiet hours",
            "Step 8 is planned when you asked for quiet, and jig is loud.",
            Some("plan.move_step"),
            vec![
                ("step".to_string(), "8".to_string()),
                ("capability".to_string(), "cut.curve".to_string()),
                ("tool".to_string(), "jig".to_string()),
                ("day".to_string(), "2".to_string()),
            ],
        ),
        finding(
            Kind::Inventory,
            Severity::Warning,
            RULE_MISSING,
            10,
            "No tool for drill.hole",
            "Step 10 needs drill.hole, and none of your tools does that (a wishlist tool does not count).",
            Some("inventory.add_tool"),
            vec![
                ("step".to_string(), "10".to_string()),
                ("capability".to_string(), "drill.hole".to_string()),
            ],
        ),
        finding(
            Kind::Tip,
            Severity::Info,
            RULE_AWAY,
            11,
            "cut.laser means a trip",
            "Step 11 needs cut.laser: you have it at a makerspace, not at home.",
            None,
            vec![
                ("step".to_string(), "11".to_string()),
                ("capability".to_string(), "cut.laser".to_string()),
                ("access".to_string(), "makerspace".to_string()),
            ],
        ),
    ];
    assert_eq!(evaluate(&worked_facts(steps)), expected);
}

// first-match order

#[test]
fn first_match_missing_is_tried_before_away() {
    // With no tools for the capability at all, "every tool is away" is
    // vacuously true, but T1 is tried first and wins.
    let facts = ToolFacts {
        steps: vec![step(1, &["drill.hole"], None, None)],
        tools: vec![],
        quiet_hours: vec![],
    };
    let got = evaluate(&facts);
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].source, Source::Rule(RULE_MISSING.to_string()));
    assert_eq!(got[0].what, "No tool for drill.hole");
}

#[test]
fn first_match_away_is_tried_before_accuracy() {
    // The makerspace laser holds 0.1 mm and the tolerance is 0.05 mm: T2 and
    // T3 both match, and T2 comes first.
    let facts = ToolFacts {
        steps: vec![step(1, &["cut.laser"], Some(0.05), None)],
        tools: vec![tool(
            "laser",
            &["cut.laser"],
            Access::Makerspace,
            Some(0.1),
            Noise::Moderate,
        )],
        quiet_hours: vec![],
    };
    let got = evaluate(&facts);
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].source, Source::Rule(RULE_AWAY.to_string()));
}

#[test]
fn first_match_accuracy_is_tried_before_noise() {
    // The loud jig holds 1 mm, the tolerance is 0.5 mm and the step is
    // planned in quiet hours: T3 and T4 both match, and T3 comes first.
    let facts = ToolFacts {
        steps: vec![step(
            1,
            &["cut.curve"],
            Some(0.5),
            Some(slot(2, 1300, 1400)),
        )],
        tools: vec![tool(
            "jig",
            &["cut.curve"],
            Access::Owned,
            Some(1.0),
            Noise::Loud,
        )],
        quiet_hours: vec![slot(2, 1260, 1440)],
    };
    let got = evaluate(&facts);
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].source, Source::Rule(RULE_ACCURACY.to_string()));
}

// repeated capabilities

#[test]
fn a_capability_repeated_in_one_step_is_checked_once() {
    let facts = ToolFacts {
        steps: vec![step(1, &["cut.straight", "cut.straight"], Some(0.5), None)],
        tools: vec![tool(
            "jig",
            &["cut.straight"],
            Access::Owned,
            Some(1.0),
            Noise::Quiet,
        )],
        quiet_hours: vec![],
    };
    let got = evaluate(&facts);
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].source, Source::Rule(RULE_ACCURACY.to_string()));
}

#[test]
fn a_repeated_capability_still_checks_each_distinct_one() {
    // ["cut.crosscut", "drill.hole", "cut.crosscut"]: crosscut once, drill.hole once.
    let facts = worked_facts(vec![step(
        1,
        &["cut.crosscut", "drill.hole", "cut.crosscut"],
        None,
        None,
    )]);
    let got = evaluate(&facts);
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].source, Source::Rule(RULE_MISSING.to_string()));
    assert_eq!(got[0].what, "No tool for drill.hole");
}

// NaN and infinite lengths never panic

#[test]
fn nan_accuracy_is_treated_as_not_given() {
    let facts = ToolFacts {
        steps: vec![step(1, &["cut.straight"], Some(0.5), None)],
        tools: vec![tool(
            "jig",
            &["cut.straight"],
            Access::Owned,
            Some(f64::NAN),
            Noise::Quiet,
        )],
        quiet_hours: vec![],
    };
    assert!(evaluate(&facts).is_empty());
}

#[test]
fn nan_tolerance_is_treated_as_not_given() {
    let facts = ToolFacts {
        steps: vec![step(1, &["cut.straight"], Some(f64::NAN), None)],
        tools: vec![tool(
            "jig",
            &["cut.straight"],
            Access::Owned,
            Some(1.0),
            Noise::Quiet,
        )],
        quiet_hours: vec![],
    };
    assert!(evaluate(&facts).is_empty());
}

#[test]
fn infinite_accuracy_is_treated_as_not_given() {
    let facts = ToolFacts {
        steps: vec![step(1, &["cut.straight"], Some(0.5), None)],
        tools: vec![tool(
            "jig",
            &["cut.straight"],
            Access::Owned,
            Some(f64::INFINITY),
            Noise::Quiet,
        )],
        quiet_hours: vec![],
    };
    assert!(evaluate(&facts).is_empty());
}

#[test]
fn infinite_tolerance_is_treated_as_not_given() {
    let facts = ToolFacts {
        steps: vec![step(1, &["cut.straight"], Some(f64::INFINITY), None)],
        tools: vec![tool(
            "jig",
            &["cut.straight"],
            Access::Owned,
            Some(1.0),
            Noise::Quiet,
        )],
        quiet_hours: vec![],
    };
    assert!(evaluate(&facts).is_empty());
}

#[test]
fn nan_lengths_never_panic_even_with_a_plan() {
    // T3 is skipped (NaN tolerance and accuracy), T4 still fires: the jig is
    // loud in quiet hours.
    let facts = ToolFacts {
        steps: vec![step(
            1,
            &["cut.curve"],
            Some(f64::NAN),
            Some(slot(2, 1300, 1400)),
        )],
        tools: vec![tool(
            "jig",
            &["cut.curve"],
            Access::Owned,
            Some(f64::NAN),
            Noise::Loud,
        )],
        quiet_hours: vec![slot(2, 1260, 1440)],
    };
    let got = evaluate(&facts);
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].source, Source::Rule(RULE_NOISE.to_string()));
}
