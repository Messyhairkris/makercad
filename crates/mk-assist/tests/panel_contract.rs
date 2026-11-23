//! Contract tests for mk_assist::panel, written by Gordon as a PA (the Crew) from
//! crew/contracts/mk_panel.md. They were the hidden tests of campaign mk-core-30.

//! Hidden tests for panel.rs (contract crew/contracts/mk_panel.md).
//
// r1..r7 are the contract's worked-example fixtures, built as whole `Finding` values.

// Crew-written tests: a helper that builds a whole `Finding` takes one argument per
// field, and worked rows print constants to the contract's digits; neither matters in a
// test.
#![allow(clippy::too_many_arguments, clippy::approx_constant)]

use mk_assist::findings::{Finding, Kind, Location, Severity, Source};
use mk_assist::panel::{badge, badge_text, filter, group, sort_rows, Badge, Filter, Group, Row};

const RULE_A: &str = "panel.a";
const RULE_B: &str = "panel.b";
const RULE_C: &str = "panel.c";
const RULE_D: &str = "panel.d";
const RULE_E: &str = "panel.e";
const RULE_F: &str = "panel.f";
const RULE_G: &str = "panel.g";

fn f1() -> Finding {
    Finding {
        kind: Kind::Forbidden,
        severity: Severity::Critical,
        source: Source::Rule(RULE_A.to_string()),
        location: Location::Entity("leg".to_string()),
        what: "A".to_string(),
        why: "a".to_string(),
        fix: Some("fix.a".to_string()),
        facts: vec![("part".to_string(), "leg".to_string())],
    }
}

fn f2() -> Finding {
    Finding {
        kind: Kind::Tip,
        severity: Severity::Info,
        source: Source::Rule(RULE_B.to_string()),
        location: Location::Project,
        what: "B".to_string(),
        why: "b".to_string(),
        fix: None,
        facts: Vec::new(),
    }
}

fn f3() -> Finding {
    Finding {
        kind: Kind::Inventory,
        severity: Severity::Warning,
        source: Source::Rule(RULE_C.to_string()),
        location: Location::Step(7),
        what: "C".to_string(),
        why: "c".to_string(),
        fix: Some("fix.c".to_string()),
        facts: vec![("step".to_string(), "7".to_string())],
    }
}

fn f4() -> Finding {
    Finding {
        kind: Kind::Fit,
        severity: Severity::Warning,
        source: Source::Rule(RULE_D.to_string()),
        location: Location::Field("width".to_string()),
        what: "D".to_string(),
        why: "d".to_string(),
        fix: None,
        facts: Vec::new(),
    }
}

fn f5() -> Finding {
    Finding {
        kind: Kind::Forbidden,
        severity: Severity::Info,
        source: Source::Rule(RULE_E.to_string()),
        location: Location::Entity("top".to_string()),
        what: "E".to_string(),
        why: "e".to_string(),
        fix: None,
        facts: vec![("part".to_string(), "top".to_string())],
    }
}

fn f6() -> Finding {
    Finding {
        kind: Kind::Denied,
        severity: Severity::Critical,
        source: Source::Rule(RULE_F.to_string()),
        location: Location::Step(2),
        what: "F".to_string(),
        why: "f".to_string(),
        fix: None,
        facts: Vec::new(),
    }
}

fn f7() -> Finding {
    Finding {
        kind: Kind::Praise,
        severity: Severity::Info,
        source: Source::Rule(RULE_G.to_string()),
        location: Location::Project,
        what: "G".to_string(),
        why: "g".to_string(),
        fix: None,
        facts: Vec::new(),
    }
}

fn row(finding: Finding, dismissed: bool) -> Row {
    Row { finding, dismissed }
}

fn r1() -> Row {
    row(f1(), false)
}
fn r2() -> Row {
    row(f2(), false)
}
fn r3() -> Row {
    row(f3(), true)
}
fn r4() -> Row {
    row(f4(), false)
}
fn r5() -> Row {
    row(f5(), false)
}
fn r6() -> Row {
    row(f6(), false)
}
fn r7() -> Row {
    row(f7(), true)
}

fn rows() -> Vec<Row> {
    vec![r1(), r2(), r3(), r4(), r5(), r6(), r7()]
}

/// A plain finding, for tests where only kind and severity matter.
fn plain(kind: Kind, severity: Severity) -> Row {
    row(
        Finding {
            kind,
            severity,
            source: Source::Rule("panel.t".to_string()),
            location: Location::Project,
            what: "t".to_string(),
            why: "t".to_string(),
            fix: None,
            facts: Vec::new(),
        },
        false,
    )
}

/// An open Tip/Info row at `location`, tagged in `what`.
fn at(location: Location, what: &str) -> Row {
    row(
        Finding {
            kind: Kind::Tip,
            severity: Severity::Info,
            source: Source::Rule("panel.t".to_string()),
            location,
            what: what.to_string(),
            why: "t".to_string(),
            fix: None,
            facts: Vec::new(),
        },
        false,
    )
}

#[test]
fn s1_open_before_dismissed() {
    assert_eq!(sort_rows(&[r3(), r1()]), vec![r1(), r3()]);
}

#[test]
fn s1_severity_highest_first() {
    let sorted = sort_rows(&[r2(), r4(), r6(), r5(), r1()]);
    assert_eq!(sorted, vec![r1(), r6(), r4(), r5(), r2()]);
}

#[test]
fn s1_kind_declaration_order() {
    let sorted = sort_rows(&[
        plain(Kind::Praise, Severity::Info),
        plain(Kind::Tip, Severity::Info),
        plain(Kind::NotSuitable, Severity::Info),
        plain(Kind::Forbidden, Severity::Info),
    ]);
    let kinds: Vec<Kind> = sorted.iter().map(|row| row.finding.kind).collect();
    assert_eq!(
        kinds,
        vec![Kind::Forbidden, Kind::NotSuitable, Kind::Tip, Kind::Praise]
    );
}

#[test]
fn s1_ties_keep_input_order() {
    let sorted = sort_rows(&[at(Location::Project, "z"), at(Location::Project, "a")]);
    let whats: Vec<&str> = sorted.iter().map(|row| row.finding.what.as_str()).collect();
    assert_eq!(whats, vec!["z", "a"]);
}

#[test]
fn s2_every_row_once_unchanged() {
    let input = rows();
    let sorted = sort_rows(&input);
    assert_eq!(sorted.len(), input.len());
    let mut by_what_in = input.clone();
    by_what_in.sort_by(|a, b| a.finding.what.cmp(&b.finding.what));
    let mut by_what_out = sorted;
    by_what_out.sort_by(|a, b| a.finding.what.cmp(&b.finding.what));
    assert_eq!(by_what_in, by_what_out);
}

#[test]
fn s2_empty_input_gives_empty_vec() {
    assert!(sort_rows(&[]).is_empty());
}

#[test]
fn g1_groups_appear_in_first_seen_order() {
    let groups = group(&[r5(), r7(), r3(), r1(), r6(), r4(), r2()], Group::Severity);
    let names: Vec<&str> = groups.iter().map(|(name, _)| name.as_str()).collect();
    assert_eq!(names, vec!["Critical", "Warning", "Info"]);
}

#[test]
fn g1_group_members_stay_sorted() {
    let groups = group(&rows(), Group::Severity);
    assert_eq!(groups.len(), 3);
    assert_eq!(groups[0].1, vec![r1(), r6()]);
    assert_eq!(groups[1].1, vec![r4(), r3()]);
    assert_eq!(groups[2].1, vec![r5(), r2(), r7()]);
}

#[test]
fn g2_severity_labels() {
    let groups = group(&rows(), Group::Severity);
    assert_eq!(groups[0].0, "Critical");
    assert_eq!(groups[1].0, "Warning");
    assert_eq!(groups[2].0, "Info");
}

#[test]
fn g2_kind_labels() {
    let input: Vec<Row> = vec![
        plain(Kind::Praise, Severity::Warning),
        plain(Kind::Tip, Severity::Warning),
        plain(Kind::Budget, Severity::Warning),
        plain(Kind::Schedule, Severity::Warning),
        plain(Kind::Fit, Severity::Warning),
        plain(Kind::Inventory, Severity::Warning),
        plain(Kind::NotSuitable, Severity::Warning),
        plain(Kind::Denied, Severity::Warning),
        plain(Kind::Unsafe, Severity::Warning),
        plain(Kind::Forbidden, Severity::Warning),
    ];
    let labels: Vec<String> = group(&input, Group::Kind)
        .into_iter()
        .map(|(label, _)| label)
        .collect();
    assert_eq!(
        labels,
        vec![
            "Forbidden",
            "Unsafe",
            "Denied",
            "Not suitable",
            "Inventory",
            "Fit",
            "Schedule",
            "Budget",
            "Tip",
            "Praise"
        ]
    );
}

#[test]
fn g2_location_labels() {
    let input = vec![
        at(Location::Project, "p"),
        at(Location::Entity("leg".to_string()), "e"),
        at(Location::Step(7), "s7"),
        at(Location::Step(0), "s0"),
        at(Location::Field("width".to_string()), "f"),
    ];
    let labels: Vec<String> = group(&input, Group::Location)
        .into_iter()
        .map(|(label, _)| label)
        .collect();
    assert_eq!(labels, vec!["Project", "leg", "Step 7", "Step 0", "width"]);
}

#[test]
fn g3_label_is_the_key() {
    let input = vec![
        at(Location::Entity("Project".to_string()), "e"),
        at(Location::Project, "p"),
        at(Location::Entity("Step 7".to_string()), "e7"),
        at(Location::Step(7), "s7"),
    ];
    let groups = group(&input, Group::Location);
    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0].0, "Project");
    assert_eq!(groups[0].1.len(), 2);
    assert_eq!(groups[1].0, "Step 7");
    assert_eq!(groups[1].1.len(), 2);
}

#[test]
fn g3_no_rows_give_no_groups() {
    assert!(group(&[], Group::Severity).is_empty());
    assert!(group(&[], Group::Kind).is_empty());
    assert!(group(&[], Group::Location).is_empty());
}

#[test]
fn f1_kinds_select() {
    let only_tips = Filter {
        kinds: vec![Kind::Tip],
        ..Filter::default()
    };
    assert_eq!(filter(&rows(), &only_tips), vec![r2()]);
}

#[test]
fn f1_min_severity_is_inclusive() {
    let warning_up = Filter {
        min_severity: Some(Severity::Warning),
        ..Filter::default()
    };
    assert_eq!(filter(&rows(), &warning_up), vec![r1(), r3(), r4(), r6()]);
    let critical = Filter {
        min_severity: Some(Severity::Critical),
        ..Filter::default()
    };
    assert_eq!(filter(&rows(), &critical), vec![r1(), r6()]);
}

#[test]
fn f1_hide_dismissed() {
    let open_only = Filter {
        hide_dismissed: true,
        ..Filter::default()
    };
    assert_eq!(
        filter(&rows(), &open_only),
        vec![r1(), r2(), r4(), r5(), r6()]
    );
}

#[test]
fn f1_all_three_together() {
    let narrow = Filter {
        kinds: vec![Kind::Forbidden, Kind::Denied],
        min_severity: Some(Severity::Critical),
        hide_dismissed: true,
    };
    assert_eq!(filter(&rows(), &narrow), vec![r1(), r6()]);
}

#[test]
fn f2_filter_keeps_input_order() {
    let input = vec![r5(), r1(), r4()];
    assert_eq!(filter(&input, &Filter::default()), input);
}

#[test]
fn f3_empty_kinds_means_every_kind() {
    let empty = Filter {
        kinds: Vec::new(),
        min_severity: None,
        hide_dismissed: false,
    };
    assert_eq!(filter(&rows(), &empty).len(), 7);
    assert_eq!(filter(&rows(), &Filter::default()), rows());
}

#[test]
fn b1_badge_counts_open_rows() {
    assert_eq!(
        badge(&rows()),
        Badge {
            open: 5,
            serious: 2,
            worst: Some(Severity::Critical),
        }
    );
}

#[test]
fn b1_badge_empty_and_all_dismissed() {
    assert_eq!(badge(&[]), Badge::default());
    assert_eq!(badge(&[r3(), r7()]), Badge::default());
    assert_eq!(badge(&[row(f1(), true)]), Badge::default());
    assert_eq!(badge(&[r4()]).worst, Some(Severity::Warning));
}

#[test]
fn b2_badge_text_nothing() {
    assert_eq!(badge_text(&badge(&[])), "Nothing to look at");
    assert_eq!(badge_text(&badge(&[r3(), r7()])), "Nothing to look at");
}

#[test]
fn b2_badge_text_one_thing() {
    assert_eq!(badge_text(&badge(&[r2()])), "1 thing to look at");
    assert_eq!(
        badge_text(&Badge {
            open: 1,
            serious: 1,
            worst: Some(Severity::Critical),
        }),
        "1 thing to look at, 1 serious"
    );
}

#[test]
fn b2_badge_text_many_things() {
    assert_eq!(
        badge_text(&badge(&rows())),
        "5 things to look at, 2 serious"
    );
    assert_eq!(
        badge_text(&Badge {
            open: 10,
            serious: 0,
            worst: Some(Severity::Warning),
        }),
        "10 things to look at"
    );
    assert_eq!(
        badge_text(&Badge {
            open: 3,
            serious: 0,
            worst: None,
        }),
        "3 things to look at"
    );
}

#[test]
fn b3_worst_does_not_change_the_words() {
    let critical = Badge {
        open: 2,
        serious: 0,
        worst: Some(Severity::Critical),
    };
    let info = Badge {
        open: 2,
        serious: 0,
        worst: Some(Severity::Info),
    };
    assert_eq!(badge_text(&critical), badge_text(&info));
    assert_eq!(badge_text(&critical), "2 things to look at");
}

#[test]
fn n1_empty_inputs_do_not_panic() {
    let none: Vec<Row> = Vec::new();
    assert!(sort_rows(&none).is_empty());
    assert!(group(&none, Group::Severity).is_empty());
    assert!(filter(&none, &Filter::default()).is_empty());
    assert_eq!(badge(&none), Badge::default());
    assert_eq!(badge_text(&badge(&none)), "Nothing to look at");
}

#[test]
fn n1_extreme_values_do_not_panic() {
    let kinds = [
        Kind::Forbidden,
        Kind::Unsafe,
        Kind::Denied,
        Kind::NotSuitable,
        Kind::Inventory,
        Kind::Fit,
        Kind::Schedule,
        Kind::Budget,
        Kind::Tip,
        Kind::Praise,
    ];
    let mut input: Vec<Row> = kinds
        .iter()
        .map(|kind| plain(*kind, Severity::Warning))
        .collect();
    input.push(at(Location::Step(u32::MAX), "huge"));
    input.push(at(Location::Field(String::new()), "empty"));
    input.push(at(Location::Entity("x".repeat(256)), "long"));
    let some_kinds = Filter {
        kinds: vec![Kind::Tip, Kind::Tip, Kind::Praise],
        min_severity: Some(Severity::Info),
        hide_dismissed: false,
    };
    assert_eq!(sort_rows(&input).len(), input.len());
    assert_eq!(group(&input, Group::Kind).len(), 10);
    assert!(group(&input, Group::Location).len() >= 3);
    assert_eq!(filter(&input, &some_kinds).len(), 5);
    assert_eq!(badge(&input).open, 13);
    assert_eq!(badge(&[r1()]).serious, 1);
}

#[test]
fn worked_e1_sort_empty() {
    let none: Vec<Row> = Vec::new();
    assert_eq!(sort_rows(&none), none);
}

#[test]
fn worked_e2_sort_every_row() {
    assert_eq!(
        sort_rows(&rows()),
        vec![r1(), r6(), r4(), r5(), r2(), r3(), r7()]
    );
}

#[test]
fn worked_e3_sort_dismissed_last() {
    assert_eq!(sort_rows(&[r3(), r7(), r1()]), vec![r1(), r3(), r7()]);
}

#[test]
fn worked_e4_group_by_severity() {
    assert_eq!(
        group(&rows(), Group::Severity),
        vec![
            ("Critical".to_string(), vec![r1(), r6()]),
            ("Warning".to_string(), vec![r4(), r3()]),
            ("Info".to_string(), vec![r5(), r2(), r7()]),
        ]
    );
}

#[test]
fn worked_e5_group_by_kind() {
    assert_eq!(
        group(&rows(), Group::Kind),
        vec![
            ("Forbidden".to_string(), vec![r1(), r5()]),
            ("Denied".to_string(), vec![r6()]),
            ("Fit".to_string(), vec![r4()]),
            ("Tip".to_string(), vec![r2()]),
            ("Inventory".to_string(), vec![r3()]),
            ("Praise".to_string(), vec![r7()]),
        ]
    );
}

#[test]
fn worked_e6_group_by_location() {
    assert_eq!(
        group(&rows(), Group::Location),
        vec![
            ("leg".to_string(), vec![r1()]),
            ("Step 2".to_string(), vec![r6()]),
            ("width".to_string(), vec![r4()]),
            ("top".to_string(), vec![r5()]),
            ("Project".to_string(), vec![r2(), r7()]),
            ("Step 7".to_string(), vec![r3()]),
        ]
    );
}

#[test]
fn worked_e7_group_empty() {
    let none: Vec<Row> = Vec::new();
    let no_groups: Vec<(String, Vec<Row>)> = Vec::new();
    assert_eq!(group(&none, Group::Severity), no_groups);
}

#[test]
fn worked_e8_filter_default() {
    assert_eq!(filter(&rows(), &Filter::default()), rows());
}

#[test]
fn worked_e9_filter_tips() {
    let tips = Filter {
        kinds: vec![Kind::Tip],
        ..Filter::default()
    };
    assert_eq!(filter(&rows(), &tips), vec![r2()]);
}

#[test]
fn worked_e10_filter_warning_up() {
    let warning_up = Filter {
        min_severity: Some(Severity::Warning),
        ..Filter::default()
    };
    assert_eq!(filter(&rows(), &warning_up), vec![r1(), r3(), r4(), r6()]);
}

#[test]
fn worked_e11_filter_hide_dismissed() {
    let open_only = Filter {
        hide_dismissed: true,
        ..Filter::default()
    };
    assert_eq!(
        filter(&rows(), &open_only),
        vec![r1(), r2(), r4(), r5(), r6()]
    );
}

#[test]
fn worked_e12_filter_narrow() {
    let narrow = Filter {
        kinds: vec![Kind::Forbidden, Kind::Denied],
        min_severity: Some(Severity::Critical),
        hide_dismissed: true,
    };
    assert_eq!(filter(&rows(), &narrow), vec![r1(), r6()]);
}

#[test]
fn worked_e13_badge_empty() {
    let none: Vec<Row> = Vec::new();
    assert_eq!(badge(&none), Badge::default());
    assert_eq!(badge_text(&Badge::default()), "Nothing to look at");
}

#[test]
fn worked_e14_badge_all_dismissed() {
    let none_open = badge(&[r3(), r7()]);
    assert_eq!(none_open, Badge::default());
    assert_eq!(badge_text(&none_open), "Nothing to look at");
}

#[test]
fn worked_e15_badge_one_thing() {
    let one = badge(&[r2()]);
    assert_eq!(
        one,
        Badge {
            open: 1,
            serious: 0,
            worst: Some(Severity::Info),
        }
    );
    assert_eq!(badge_text(&one), "1 thing to look at");
}

#[test]
fn worked_e16_badge_all_rows() {
    let all = badge(&rows());
    assert_eq!(
        all,
        Badge {
            open: 5,
            serious: 2,
            worst: Some(Severity::Critical),
        }
    );
    assert_eq!(badge_text(&all), "5 things to look at, 2 serious");
}

#[test]
fn worked_e17_badge_three_one_serious() {
    let three = badge(&[r1(), r2(), r4()]);
    assert_eq!(
        three,
        Badge {
            open: 3,
            serious: 1,
            worst: Some(Severity::Critical),
        }
    );
    assert_eq!(badge_text(&three), "3 things to look at, 1 serious");
}

#[test]
fn worked_e18_badge_text_two_serious() {
    let two = Badge {
        open: 2,
        serious: 2,
        worst: Some(Severity::Critical),
    };
    assert_eq!(badge_text(&two), "2 things to look at, 2 serious");
}

#[test]
fn worked_e19_badge_text_ten() {
    let ten = Badge {
        open: 10,
        serious: 0,
        worst: Some(Severity::Warning),
    };
    assert_eq!(badge_text(&ten), "10 things to look at");
}

#[test]
fn worked_e20_badge_text_no_worst() {
    let three = Badge {
        open: 3,
        serious: 0,
        worst: None,
    };
    assert_eq!(badge_text(&three), "3 things to look at");
}
