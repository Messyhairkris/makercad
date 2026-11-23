//! Tests for mk_assist::panel written by Deepak (the Crew, campaign mk-core-30) from the
//! contract alone, without seeing the module.

//! Integration tests for `panel.rs`: the Findings panel's model (contract v1).

// Crew-written tests: a helper that builds a whole `Finding` takes one argument per
// field, and worked rows print constants to the contract's digits; neither matters in a
// test.
#![allow(clippy::too_many_arguments, clippy::approx_constant)]

use mk_assist::findings::{Finding, Kind, Location, Severity, Source};
use mk_assist::panel::{badge, badge_text, filter, group, sort_rows, Badge, Filter, Group, Row};

fn mk(
    kind: Kind,
    severity: Severity,
    rule: &str,
    location: Location,
    what: &str,
    why: &str,
    fix: Option<&str>,
    facts: &[(&str, &str)],
    dismissed: bool,
) -> Row {
    Row {
        finding: Finding {
            kind,
            severity,
            source: Source::Rule(rule.to_string()),
            location,
            what: what.to_string(),
            why: why.to_string(),
            fix: fix.map(str::to_string),
            facts: facts
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        },
        dismissed,
    }
}

/// A row whose only deciding keys are kind, severity and dismissed.
fn s(kind: Kind, severity: Severity, dismissed: bool, what: &str) -> Row {
    mk(
        kind,
        severity,
        "rule.s",
        Location::Project,
        what,
        "",
        None,
        &[],
        dismissed,
    )
}

fn r1() -> Row {
    mk(
        Kind::Forbidden,
        Severity::Critical,
        "panel.a",
        Location::Entity("leg".into()),
        "A",
        "a",
        Some("fix.a"),
        &[("part", "leg")],
        false,
    )
}
fn r2() -> Row {
    mk(
        Kind::Tip,
        Severity::Info,
        "panel.b",
        Location::Project,
        "B",
        "b",
        None,
        &[],
        false,
    )
}
fn r3() -> Row {
    mk(
        Kind::Inventory,
        Severity::Warning,
        "panel.c",
        Location::Step(7),
        "C",
        "c",
        Some("fix.c"),
        &[("step", "7")],
        true,
    )
}
fn r4() -> Row {
    mk(
        Kind::Fit,
        Severity::Warning,
        "panel.d",
        Location::Field("width".into()),
        "D",
        "d",
        None,
        &[],
        false,
    )
}
fn r5() -> Row {
    mk(
        Kind::Forbidden,
        Severity::Info,
        "panel.e",
        Location::Entity("top".into()),
        "E",
        "e",
        None,
        &[("part", "top")],
        false,
    )
}
fn r6() -> Row {
    mk(
        Kind::Denied,
        Severity::Critical,
        "panel.f",
        Location::Step(2),
        "F",
        "f",
        None,
        &[],
        false,
    )
}
fn r7() -> Row {
    mk(
        Kind::Praise,
        Severity::Info,
        "panel.g",
        Location::Project,
        "G",
        "g",
        None,
        &[],
        true,
    )
}
fn all() -> Vec<Row> {
    vec![r1(), r2(), r3(), r4(), r5(), r6(), r7()]
}

// ---- P1

#[test]
fn p1_row_is_finding_plus_dismissed() {
    let row = r1();
    assert_eq!(row.finding.kind, Kind::Forbidden);
    assert_eq!(row.finding.severity, Severity::Critical);
    assert_eq!(row.finding.source, Source::Rule("panel.a".to_string()));
    assert_eq!(row.finding.location, Location::Entity("leg".to_string()));
    assert_eq!(row.finding.what, "A");
    assert_eq!(row.finding.why, "a");
    assert_eq!(row.finding.fix, Some("fix.a".to_string()));
    assert_eq!(
        row.finding.facts,
        vec![("part".to_string(), "leg".to_string())]
    );
    assert!(!row.dismissed);
    assert!(r3().dismissed);
}

#[test]
fn p1_panel_holds_dismissed_rows() {
    assert!(sort_rows(&all()).contains(&r3()));
    assert!(sort_rows(&all()).contains(&r7()));
    assert_eq!(
        group(&all(), Group::Kind)
            .iter()
            .find(|(l, _)| l == "Praise"),
        Some(&("Praise".to_string(), vec![r7()]))
    );
}

// ---- S1

#[test]
fn s1_open_before_dismissed() {
    let d = s(Kind::Forbidden, Severity::Critical, true, "d");
    let o = s(Kind::Praise, Severity::Info, false, "o");
    assert_eq!(sort_rows(&[d.clone(), o.clone()]), vec![o, d]);
}

#[test]
fn s1_severity_highest_first() {
    let rows = vec![
        s(Kind::Fit, Severity::Info, false, "i"),
        s(Kind::Fit, Severity::Critical, false, "c"),
        s(Kind::Fit, Severity::Warning, false, "w"),
    ];
    let got: Vec<Severity> = sort_rows(&rows)
        .iter()
        .map(|r| r.finding.severity)
        .collect();
    assert_eq!(
        got,
        vec![Severity::Critical, Severity::Warning, Severity::Info]
    );
}

#[test]
fn s1_kind_declaration_order() {
    let kinds = [
        Kind::Praise,
        Kind::Tip,
        Kind::Budget,
        Kind::Schedule,
        Kind::Fit,
        Kind::Inventory,
        Kind::NotSuitable,
        Kind::Denied,
        Kind::Unsafe,
        Kind::Forbidden,
    ];
    let rows: Vec<Row> = kinds
        .iter()
        .map(|k| s(*k, Severity::Info, false, "x"))
        .collect();
    let got: Vec<Kind> = sort_rows(&rows).iter().map(|r| r.finding.kind).collect();
    assert_eq!(got, kinds.into_iter().rev().collect::<Vec<Kind>>());
}

#[test]
fn s1_equal_keys_keep_input_order() {
    let a = s(Kind::Fit, Severity::Warning, false, "a");
    let b = s(Kind::Fit, Severity::Warning, false, "b");
    let c = s(Kind::Fit, Severity::Warning, false, "c");
    assert_eq!(sort_rows(&[c.clone(), a.clone(), b.clone()]), vec![c, a, b]);
}

// ---- S2

#[test]
fn s2_every_row_once_unchanged() {
    let input = all();
    let sorted = sort_rows(&input);
    assert_eq!(sorted.len(), input.len());
    for row in &input {
        assert_eq!(sorted.iter().filter(|o| *o == row).count(), 1);
    }
    assert_eq!(sort_rows(&sorted), sorted);
}

#[test]
fn s2_empty_input_empty_vec() {
    assert_eq!(sort_rows(&[]), Vec::<Row>::new());
}

// ---- G1

#[test]
fn g1_groups_in_sorted_first_seen_order() {
    assert_eq!(
        group(&[r2(), r1()], Group::Kind),
        vec![
            ("Forbidden".to_string(), vec![r1()]),
            ("Tip".to_string(), vec![r2()])
        ]
    );
}

// ---- G2

#[test]
fn g2_severity_labels() {
    let labels: Vec<String> = group(&[r2(), r1(), r4()], Group::Severity)
        .into_iter()
        .map(|(l, _)| l)
        .collect();
    assert_eq!(
        labels,
        vec![
            "Critical".to_string(),
            "Warning".to_string(),
            "Info".to_string()
        ]
    );
}

#[test]
fn g2_kind_labels() {
    let kinds = [
        Kind::Praise,
        Kind::Tip,
        Kind::Budget,
        Kind::Schedule,
        Kind::Fit,
        Kind::Inventory,
        Kind::NotSuitable,
        Kind::Denied,
        Kind::Unsafe,
        Kind::Forbidden,
    ];
    let rows: Vec<Row> = kinds
        .iter()
        .map(|k| s(*k, Severity::Info, false, "x"))
        .collect();
    let labels: Vec<String> = group(&rows, Group::Kind)
        .into_iter()
        .map(|(l, _)| l)
        .collect();
    assert_eq!(
        labels,
        vec![
            "Forbidden".to_string(),
            "Unsafe".to_string(),
            "Denied".to_string(),
            "Not suitable".to_string(),
            "Inventory".to_string(),
            "Fit".to_string(),
            "Schedule".to_string(),
            "Budget".to_string(),
            "Tip".to_string(),
            "Praise".to_string(),
        ]
    );
}

#[test]
fn g2_location_labels() {
    let rows = vec![
        mk(
            Kind::Tip,
            Severity::Info,
            "a",
            Location::Project,
            "p",
            "",
            None,
            &[],
            false,
        ),
        mk(
            Kind::Tip,
            Severity::Info,
            "b",
            Location::Entity("leg".into()),
            "e",
            "",
            None,
            &[],
            false,
        ),
        mk(
            Kind::Tip,
            Severity::Info,
            "c",
            Location::Step(0),
            "s0",
            "",
            None,
            &[],
            false,
        ),
        mk(
            Kind::Tip,
            Severity::Info,
            "d",
            Location::Step(7),
            "s7",
            "",
            None,
            &[],
            false,
        ),
        mk(
            Kind::Tip,
            Severity::Info,
            "e",
            Location::Field("width".into()),
            "f",
            "",
            None,
            &[],
            false,
        ),
    ];
    let labels: Vec<String> = group(&rows, Group::Location)
        .into_iter()
        .map(|(l, _)| l)
        .collect();
    assert_eq!(
        labels,
        vec![
            "Project".to_string(),
            "leg".to_string(),
            "Step 0".to_string(),
            "Step 7".to_string(),
            "width".to_string(),
        ]
    );
}

// ---- G3

#[test]
fn g3_same_label_shares_one_group() {
    let ep = mk(
        Kind::Tip,
        Severity::Info,
        "a",
        Location::Entity("Project".into()),
        "ep",
        "",
        None,
        &[],
        false,
    );
    let pp = mk(
        Kind::Tip,
        Severity::Info,
        "b",
        Location::Project,
        "pp",
        "",
        None,
        &[],
        false,
    );
    let es = mk(
        Kind::Tip,
        Severity::Info,
        "c",
        Location::Entity("Step 7".into()),
        "es",
        "",
        None,
        &[],
        false,
    );
    let ss = mk(
        Kind::Tip,
        Severity::Info,
        "d",
        Location::Step(7),
        "ss",
        "",
        None,
        &[],
        false,
    );
    assert_eq!(
        group(
            &[ep.clone(), pp.clone(), es.clone(), ss.clone()],
            Group::Location
        ),
        vec![
            ("Project".to_string(), vec![ep, pp]),
            ("Step 7".to_string(), vec![es, ss])
        ]
    );
}

#[test]
fn g3_no_rows_no_groups() {
    assert_eq!(
        group(&[], Group::Location),
        Vec::<(String, Vec<Row>)>::new()
    );
}

// ---- F1

#[test]
fn f1_kinds_condition() {
    assert_eq!(
        filter(
            &all(),
            &Filter {
                kinds: vec![Kind::Denied],
                ..Filter::default()
            }
        ),
        vec![r6()]
    );
}

#[test]
fn f1_min_severity_condition() {
    assert_eq!(
        filter(
            &all(),
            &Filter {
                min_severity: Some(Severity::Info),
                ..Filter::default()
            }
        ),
        all()
    );
    assert_eq!(
        filter(
            &all(),
            &Filter {
                min_severity: Some(Severity::Critical),
                ..Filter::default()
            }
        ),
        vec![r1(), r6()]
    );
}

#[test]
fn f1_hide_dismissed_condition() {
    assert_eq!(
        filter(
            &[r3(), r7(), r1()],
            &Filter {
                hide_dismissed: true,
                ..Filter::default()
            }
        ),
        vec![r1()]
    );
}

#[test]
fn f1_all_three_conditions() {
    let f = Filter {
        kinds: vec![Kind::Inventory, Kind::Fit],
        min_severity: Some(Severity::Warning),
        hide_dismissed: true,
    };
    assert_eq!(filter(&all(), &f), vec![r4()]);
}

// ---- F2

#[test]
fn f2_filter_keeps_input_order() {
    let f = Filter {
        kinds: vec![Kind::Praise, Kind::Forbidden],
        ..Filter::default()
    };
    assert_eq!(filter(&all(), &f), vec![r1(), r5(), r7()]);
}

// ---- F3

#[test]
fn f3_empty_kinds_means_every_kind() {
    let f = Filter {
        kinds: Vec::new(),
        min_severity: None,
        hide_dismissed: false,
    };
    assert_eq!(f, Filter::default());
    assert_eq!(filter(&all(), &f), all());
}

// ---- B1

#[test]
fn b1_badge_counts_open_and_serious() {
    assert_eq!(
        badge(&[r3(), r4(), r1()]),
        Badge {
            open: 2,
            serious: 1,
            worst: Some(Severity::Critical)
        }
    );
}

#[test]
fn b1_badge_worst_is_highest_open() {
    assert_eq!(
        badge(&[r2(), r4()]),
        Badge {
            open: 2,
            serious: 0,
            worst: Some(Severity::Warning)
        }
    );
    assert_eq!(
        badge(&[r3(), r7()]),
        Badge {
            open: 0,
            serious: 0,
            worst: None
        }
    );
}

// ---- B2

#[test]
fn b2_badge_text_zero_open() {
    assert_eq!(
        badge_text(&Badge {
            open: 0,
            serious: 0,
            worst: None
        }),
        "Nothing to look at"
    );
}

#[test]
fn b2_badge_text_one_open() {
    assert_eq!(
        badge_text(&Badge {
            open: 1,
            serious: 0,
            worst: Some(Severity::Info)
        }),
        "1 thing to look at"
    );
}

#[test]
fn b2_badge_text_many_open() {
    assert_eq!(
        badge_text(&Badge {
            open: 2,
            serious: 0,
            worst: Some(Severity::Warning)
        }),
        "2 things to look at"
    );
    assert_eq!(
        badge_text(&Badge {
            open: 10,
            serious: 0,
            worst: Some(Severity::Warning)
        }),
        "10 things to look at"
    );
}

#[test]
fn b2_badge_text_serious_suffix() {
    assert_eq!(
        badge_text(&Badge {
            open: 3,
            serious: 1,
            worst: Some(Severity::Critical)
        }),
        "3 things to look at, 1 serious"
    );
    assert_eq!(
        badge_text(&Badge {
            open: 5,
            serious: 2,
            worst: Some(Severity::Critical)
        }),
        "5 things to look at, 2 serious"
    );
}

// ---- B3

#[test]
fn b3_worst_never_changes_words() {
    let a = badge_text(&Badge {
        open: 2,
        serious: 1,
        worst: Some(Severity::Info),
    });
    let b = badge_text(&Badge {
        open: 2,
        serious: 1,
        worst: None,
    });
    assert_eq!(a, "2 things to look at, 1 serious");
    assert_eq!(b, a);
}

// ---- N1

#[test]
fn n1_empty_inputs_never_panic() {
    assert_eq!(sort_rows(&[]), Vec::<Row>::new());
    assert_eq!(
        group(&[], Group::Severity),
        Vec::<(String, Vec<Row>)>::new()
    );
    assert_eq!(group(&[], Group::Kind), Vec::<(String, Vec<Row>)>::new());
    assert_eq!(
        group(&[], Group::Location),
        Vec::<(String, Vec<Row>)>::new()
    );
    assert_eq!(filter(&[], &Filter::default()), Vec::<Row>::new());
    assert_eq!(
        badge(&[]),
        Badge {
            open: 0,
            serious: 0,
            worst: None
        }
    );
    assert_eq!(badge_text(&Badge::default()), "Nothing to look at");
}

#[test]
fn n1_extreme_inputs_never_panic() {
    let huge = mk(
        Kind::Schedule,
        Severity::Warning,
        "a",
        Location::Step(u32::MAX),
        "m",
        "",
        None,
        &[],
        false,
    );
    let empty = mk(
        Kind::NotSuitable,
        Severity::Info,
        "b",
        Location::Field(String::new()),
        "e",
        "",
        None,
        &[],
        false,
    );
    let long = mk(
        Kind::Unsafe,
        Severity::Critical,
        "c",
        Location::Entity("e".repeat(256)),
        "l",
        "",
        None,
        &[],
        false,
    );
    let rows = vec![huge, empty, long];
    assert_eq!(sort_rows(&rows).len(), 3);
    let labels: Vec<String> = group(&rows, Group::Location)
        .into_iter()
        .map(|(l, _)| l)
        .collect();
    assert_eq!(
        labels,
        vec![
            "e".repeat(256),
            "Step 4294967295".to_string(),
            String::new()
        ]
    );
    let repeated = filter(
        &all(),
        &Filter {
            kinds: vec![Kind::Tip, Kind::Tip, Kind::Tip],
            ..Filter::default()
        },
    );
    assert_eq!(repeated, vec![r2()]);
}

// ---- Worked examples E1-E20

#[test]
fn e1_e3_sort_rows_examples() {
    assert_eq!(sort_rows(&[]), Vec::<Row>::new());
    assert_eq!(
        sort_rows(&all()),
        vec![r1(), r6(), r4(), r5(), r2(), r3(), r7()]
    );
    assert_eq!(sort_rows(&[r3(), r7(), r1()]), vec![r1(), r3(), r7()]);
}

#[test]
fn e4_e7_group_examples() {
    assert_eq!(
        group(&all(), Group::Severity),
        vec![
            ("Critical".to_string(), vec![r1(), r6()]),
            ("Warning".to_string(), vec![r4(), r3()]),
            ("Info".to_string(), vec![r5(), r2(), r7()]),
        ]
    );
    assert_eq!(
        group(&all(), Group::Kind),
        vec![
            ("Forbidden".to_string(), vec![r1(), r5()]),
            ("Denied".to_string(), vec![r6()]),
            ("Fit".to_string(), vec![r4()]),
            ("Tip".to_string(), vec![r2()]),
            ("Inventory".to_string(), vec![r3()]),
            ("Praise".to_string(), vec![r7()]),
        ]
    );
    assert_eq!(
        group(&all(), Group::Location),
        vec![
            ("leg".to_string(), vec![r1()]),
            ("Step 2".to_string(), vec![r6()]),
            ("width".to_string(), vec![r4()]),
            ("top".to_string(), vec![r5()]),
            ("Project".to_string(), vec![r2(), r7()]),
            ("Step 7".to_string(), vec![r3()]),
        ]
    );
    assert_eq!(
        group(&[], Group::Severity),
        Vec::<(String, Vec<Row>)>::new()
    );
}

#[test]
fn e8_e12_filter_examples() {
    assert_eq!(filter(&all(), &Filter::default()), all());
    assert_eq!(
        filter(
            &all(),
            &Filter {
                kinds: vec![Kind::Tip],
                ..Filter::default()
            }
        ),
        vec![r2()]
    );
    assert_eq!(
        filter(
            &all(),
            &Filter {
                min_severity: Some(Severity::Warning),
                ..Filter::default()
            }
        ),
        vec![r1(), r3(), r4(), r6()]
    );
    assert_eq!(
        filter(
            &all(),
            &Filter {
                hide_dismissed: true,
                ..Filter::default()
            }
        ),
        vec![r1(), r2(), r4(), r5(), r6()]
    );
    let f = Filter {
        kinds: vec![Kind::Forbidden, Kind::Denied],
        min_severity: Some(Severity::Critical),
        hide_dismissed: true,
    };
    assert_eq!(filter(&all(), &f), vec![r1(), r6()]);
}

#[test]
fn e13_e17_badge_examples() {
    assert_eq!(
        badge(&[]),
        Badge {
            open: 0,
            serious: 0,
            worst: None
        }
    );
    assert_eq!(badge_text(&badge(&[])), "Nothing to look at");
    assert_eq!(
        badge(&[r3(), r7()]),
        Badge {
            open: 0,
            serious: 0,
            worst: None
        }
    );
    assert_eq!(badge_text(&badge(&[r3(), r7()])), "Nothing to look at");
    assert_eq!(
        badge(&[r2()]),
        Badge {
            open: 1,
            serious: 0,
            worst: Some(Severity::Info)
        }
    );
    assert_eq!(badge_text(&badge(&[r2()])), "1 thing to look at");
    assert_eq!(
        badge(&all()),
        Badge {
            open: 5,
            serious: 2,
            worst: Some(Severity::Critical)
        }
    );
    assert_eq!(badge_text(&badge(&all())), "5 things to look at, 2 serious");
    assert_eq!(
        badge(&[r1(), r2(), r4()]),
        Badge {
            open: 3,
            serious: 1,
            worst: Some(Severity::Critical)
        }
    );
    assert_eq!(
        badge_text(&badge(&[r1(), r2(), r4()])),
        "3 things to look at, 1 serious"
    );
}

#[test]
fn e18_e20_badge_text_examples() {
    assert_eq!(
        badge_text(&Badge {
            open: 2,
            serious: 2,
            worst: Some(Severity::Critical)
        }),
        "2 things to look at, 2 serious"
    );
    assert_eq!(
        badge_text(&Badge {
            open: 10,
            serious: 0,
            worst: Some(Severity::Warning)
        }),
        "10 things to look at"
    );
    assert_eq!(
        badge_text(&Badge {
            open: 3,
            serious: 0,
            worst: None
        }),
        "3 things to look at"
    );
}
