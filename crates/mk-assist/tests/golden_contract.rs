//! Contract tests for mk_assist::golden, written by Gordon as a PA (the Crew) from
//! crew/contracts/mk_golden.md. They were the hidden tests of campaign mk-core-33.

// Crew-written tests: a helper that builds a whole `Finding` takes one argument per
// field, and worked rows print constants to the contract's digits; neither matters in a
// test.
#![allow(clippy::too_many_arguments, clippy::approx_constant)]

use mk_assist::findings::{Finding, Location, Source};
use mk_assist::golden::{
    check, evaluate_all, faulted, location_text, near_misses, seeds, sound, summary, with_faults,
    Golden, Report, Seed,
};

fn rule_of(finding: &Finding) -> String {
    match &finding.source {
        Source::Rule(id) => id.clone(),
        Source::Mind => "mind".to_string(),
    }
}

fn loc_of(finding: &Finding) -> String {
    location_text(&finding.location)
}

fn rule_locs(golden: &Golden) -> Vec<(String, String)> {
    evaluate_all(golden)
        .iter()
        .map(|finding| (rule_of(finding), loc_of(finding)))
        .collect()
}

// Worked example 1, rule A2: the sound chest earns no finding from any pack.
#[test]
fn sound_has_no_findings() {
    let found = evaluate_all(&sound());
    assert!(found.is_empty(), "{:?}", found);
}

// Worked example 5, rule A6.
#[test]
fn sound_check_reports_five_missing() {
    let report = check(&sound(), &seeds());
    assert!(report.matched.is_empty());
    assert!(report.missing.len() == 5);
    assert!(report.false_flags.is_empty());
    assert!(!report.is_perfect());
}

// Worked example 2, rule A3.
#[test]
fn seeds_are_exactly_five_in_order() {
    let found = seeds();
    assert!(found.len() == 5);
    let expected = [
        ("structure.sag", "carcase_top"),
        ("structure.screw_through", "screw_1"),
        ("structure.edge_distance", "screw_2"),
        ("tools.missing", "4"),
        ("safety.anti_tip", "chest"),
    ];
    for (index, (rule, entity)) in expected.iter().enumerate() {
        assert!(found[index].rule.as_str() == *rule);
        assert!(found[index].entity.as_str() == *entity);
    }
}

// Worked example 4, rule A6.
#[test]
fn faulted_flags_exactly_the_five_seeds() {
    let report = check(&faulted(), &seeds());
    assert!(report.is_perfect());
    assert!(report.matched == seeds());
    assert!(report.missing.is_empty());
    assert!(report.false_flags.is_empty());
}

// Worked example 3, rule A5.
#[test]
fn faulted_has_exactly_five_findings() {
    let report = check(&faulted(), &[]);
    assert!(report.false_flags.len() == 5);
    assert!(evaluate_all(&faulted()).len() == 5);
}

// Worked example 3, rule A5: core, then tools, then structure, then safety.
#[test]
fn faulted_findings_come_in_pack_order() {
    let order: Vec<String> = evaluate_all(&faulted()).iter().map(rule_of).collect();
    assert!(
        order
            == vec![
                "tools.missing".to_string(),
                "structure.sag".to_string(),
                "structure.screw_through".to_string(),
                "structure.edge_distance".to_string(),
                "safety.anti_tip".to_string(),
            ],
        "{:?}",
        order
    );
}

// Worked example 8, rule A4.
#[test]
fn seed_zero_alone_flags_only_sag() {
    let found = evaluate_all(&with_faults(&[0]));
    assert!(found.len() == 1, "{:?}", found);
    assert!(rule_of(&found[0]) == "structure.sag");
    assert!(loc_of(&found[0]) == "carcase_top");
}

// Worked example 9, rule A4.
#[test]
fn seed_one_alone_flags_only_screw_through() {
    let found = evaluate_all(&with_faults(&[1]));
    assert!(found.len() == 1, "{:?}", found);
    assert!(rule_of(&found[0]) == "structure.screw_through");
    assert!(loc_of(&found[0]) == "screw_1");
}

// Worked example 10, rule A4.
#[test]
fn seed_two_alone_flags_only_edge_distance() {
    let found = evaluate_all(&with_faults(&[2]));
    assert!(found.len() == 1, "{:?}", found);
    assert!(rule_of(&found[0]) == "structure.edge_distance");
    assert!(loc_of(&found[0]) == "screw_2");
}

// Worked example 11, rule A4.
#[test]
fn seed_three_alone_flags_only_tools_missing() {
    let found = evaluate_all(&with_faults(&[3]));
    assert!(found.len() == 1, "{:?}", found);
    assert!(rule_of(&found[0]) == "tools.missing");
    assert!(loc_of(&found[0]) == "4");
}

// Worked example 12, rule A4.
#[test]
fn seed_four_alone_flags_only_anti_tip() {
    let found = evaluate_all(&with_faults(&[4]));
    assert!(found.len() == 1, "{:?}", found);
    assert!(rule_of(&found[0]) == "safety.anti_tip");
    assert!(loc_of(&found[0]) == "chest");
}

// Worked example 13, rule A4: duplicates apply once, unknown indices are ignored.
#[test]
fn duplicate_and_out_of_range_indices_are_ignored() {
    let once = evaluate_all(&with_faults(&[0]));
    let many = evaluate_all(&with_faults(&[0, 0, 0, 99, 500]));
    assert!(many.len() == once.len());
    assert!(rule_of(&many[0]) == rule_of(&once[0]));
    assert!(evaluate_all(&with_faults(&[99, 100])).is_empty());
}

// Worked example 14, rule A4.
#[test]
fn empty_fault_list_is_sound() {
    assert!(evaluate_all(&with_faults(&[])).is_empty());
}

// Worked example 15, rule A4: the order of the indices does not matter.
#[test]
fn faulted_is_all_five_in_any_order() {
    assert!(rule_locs(&faulted()) == rule_locs(&with_faults(&[4, 3, 2, 1, 0])));
}

// Worked example 6, rule A7.
#[test]
fn summary_of_a_perfect_report() {
    let report = check(&faulted(), &seeds());
    assert!(summary(&report) == "5 of 5 seeded faults flagged, 0 false flags");
}

// Worked example 7, rule A7.
#[test]
fn summary_of_sound_is_zero_of_five() {
    let report = check(&sound(), &seeds());
    assert!(summary(&report) == "0 of 5 seeded faults flagged, 0 false flags");
}

// Worked example 18, rule A7: the singular.
#[test]
fn summary_says_one_false_flag_in_the_singular() {
    let report = Report {
        matched: vec![],
        missing: vec![],
        false_flags: vec![("tools.missing".to_string(), "4".to_string())],
    };
    assert!(summary(&report) == "0 of 0 seeded faults flagged, 1 false flag");
    assert!(!report.is_perfect());
}

// Worked example 16, rule A1.
#[test]
fn location_text_spells_every_variant() {
    assert!(location_text(&Location::Project) == "project");
    assert!(location_text(&Location::Entity("chest".to_string())) == "chest");
    assert!(location_text(&Location::Step(7)) == "7");
    assert!(location_text(&Location::Field("prompt.a".to_string())) == "prompt.a");
}

// Worked example 17, rule A8: every near miss is clean.
#[test]
fn near_misses_all_give_no_findings() {
    for (name, golden) in near_misses() {
        let found = evaluate_all(&golden);
        assert!(found.is_empty(), "{} gave {:?}", name, found);
        let report = check(&golden, &seeds());
        assert!(report.missing.len() == 5, "{}", name);
        assert!(report.false_flags.is_empty(), "{}", name);
    }
}

// Rule A8: at least six, all named, and the six the contract lists are there.
#[test]
fn near_misses_are_named_and_at_least_six() {
    let all = near_misses();
    assert!(all.len() >= 6);
    let mut names: Vec<String> = all.iter().map(|(name, _)| name.clone()).collect();
    names.sort();
    names.dedup();
    assert!(names.len() == all.len());
    for (name, _) in &all {
        assert!(!name.is_empty());
    }
    let wanted = [
        "shelf_load_at_ninety_percent",
        "screw_one_mm_short_of_poking_through",
        "screw_just_beyond_edge_distance",
        "step_tool_accurate_enough",
        "chest_not_tall",
        "tall_chest_anchored",
        "wall_load_at_rating",
        "slot_just_above_severing",
    ];
    for name in wanted {
        assert!(
            all.iter().any(|(found, _)| found == name),
            "missing near miss {}",
            name
        );
    }
}

// Worked example 20, rule A6: each seed matches at most one finding.
#[test]
fn check_matches_a_seed_at_most_once() {
    let first = seeds()[0].clone();
    let report = check(&with_faults(&[0]), &[first.clone(), first.clone()]);
    assert!(report.matched == vec![first.clone()]);
    assert!(report.missing == vec![first]);
    assert!(report.false_flags.is_empty());
}

// Worked example 19, rule A6: every finding is a false flag when nothing is expected.
#[test]
fn check_of_faulted_against_no_seeds_flags_every_finding() {
    let report = check(&faulted(), &[]);
    assert!(report.matched.is_empty());
    assert!(report.missing.is_empty());
    assert!(
        report.false_flags
            == vec![
                ("tools.missing".to_string(), "4".to_string()),
                ("structure.sag".to_string(), "carcase_top".to_string()),
                ("structure.screw_through".to_string(), "screw_1".to_string()),
                ("structure.edge_distance".to_string(), "screw_2".to_string()),
                ("safety.anti_tip".to_string(), "chest".to_string()),
            ]
    );
    assert!(!report.is_perfect());
}

// Worked example 21, rule A6: a seed nobody earns is missing, not a false flag.
#[test]
fn check_reports_an_unknown_seed_as_missing() {
    let report = check(
        &sound(),
        &[Seed {
            rule: "nope.rule".to_string(),
            entity: "nothing".to_string(),
        }],
    );
    assert!(report.matched.is_empty());
    assert!(report.missing.len() == 1);
    assert!(report.false_flags.is_empty());
    assert!(!report.is_perfect());
}

// Rule A7: nothing expected and nothing found is perfect.
#[test]
fn check_of_sound_against_no_seeds_is_perfect() {
    let report = check(&sound(), &[]);
    assert!(report.is_perfect());
    assert!(summary(&report) == "0 of 0 seeded faults flagged, 0 false flags");
}

// Rule A2 and the derives: a clone is still the sound chest.
#[test]
fn sound_can_be_cloned_and_formatted() {
    let golden = sound();
    let copy = golden.clone();
    assert!(evaluate_all(&copy).is_empty());
    assert!(format!("{:?}", copy).contains("Golden"));
}

// Rule A3: seeds compare by value.
#[test]
fn seeds_compare_by_value() {
    assert!(seeds()[0] == seeds()[0]);
    assert!(seeds()[0] != seeds()[1]);
    assert!(seeds().len() == 5);
}

// Rule A4: applying faults to one copy never changes a later sound().
#[test]
fn with_faults_does_not_change_later_sound() {
    let _ = with_faults(&[0, 1, 2, 3, 4]);
    assert!(evaluate_all(&sound()).is_empty());
    assert!(evaluate_all(&with_faults(&[0, 1, 2, 3, 4, 5, 6, 1000])).len() == 5);
}

// Rule A9: empty lists and unknown ids are silence, never a panic.
#[test]
fn empty_and_unknown_inputs_do_not_panic() {
    let empty: Vec<Seed> = Vec::new();
    let report = check(&sound(), &empty);
    assert!(report.matched.is_empty());
    assert!(report.missing.is_empty());
    assert!(report.false_flags.is_empty());
    assert!(evaluate_all(&with_faults(&[usize::MAX])).is_empty());
    assert!(location_text(&Location::Field(String::new())).is_empty());
}
