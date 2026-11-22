//! Tests for mk_assist::findings written by Kimi (the Crew, campaign mk-core-13) from the
//! contract alone, without seeing the module.

//! Integration tests for Sancho's findings and delivery ledger.

use mk_assist::findings::{
    identity, panel_order, speaks, Context, Delivery, Evaluation, Finding, Identity, Kind, Ledger,
    Location, Mode, Override, Severity, Source, DEMOTE_AFTER,
};

fn finding(kind: Kind, source: Source, location: Location) -> Finding {
    Finding {
        kind,
        severity: Severity::Warning,
        source,
        location,
        what: "what".to_owned(),
        why: "why".to_owned(),
        fix: None,
        facts: Vec::new(),
    }
}

fn with_facts(mut finding: Finding, facts: &[(&str, &str)]) -> Finding {
    finding.facts = facts
        .iter()
        .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
        .collect();
    finding
}

fn check(evaluation: Evaluation, deliveries: Vec<Delivery>, badge: usize) {
    assert_eq!(evaluation, Evaluation { deliveries, badge });
}

fn example_a() -> Finding {
    Finding {
        kind: Kind::Inventory,
        severity: Severity::Warning,
        source: Source::Rule("stock.short".to_owned()),
        location: Location::Entity("drawer-side".to_owned()),
        ..with_facts(
            finding(Kind::Inventory, Source::Mind, Location::Project),
            &[("short", "2")],
        )
    }
}

fn example_b() -> Finding {
    Finding {
        kind: Kind::Tip,
        severity: Severity::Info,
        source: Source::Rule("stock.same-setup".to_owned()),
        location: Location::Project,
        ..finding(Kind::Tip, Source::Mind, Location::Project)
    }
}

fn example_c() -> Finding {
    Finding {
        kind: Kind::Unsafe,
        severity: Severity::Critical,
        source: Source::Rule("safety.pvc-laser".to_owned()),
        location: Location::Entity("panel-3".to_owned()),
        ..with_facts(
            finding(Kind::Unsafe, Source::Mind, Location::Project),
            &[("material", "plastic.pvc")],
        )
    }
}

fn example_d() -> Finding {
    Finding {
        kind: Kind::Fit,
        severity: Severity::Warning,
        source: Source::Rule("fit.clearance".to_owned()),
        location: Location::Entity("left-side".to_owned()),
        ..with_facts(
            finding(Kind::Fit, Source::Mind, Location::Project),
            &[("over_mm", "6")],
        )
    }
}

#[test]
fn k1_speaks_matches_the_exact_kind_table() {
    assert!(speaks(Kind::Forbidden));
    assert!(speaks(Kind::Unsafe));
    assert!(speaks(Kind::Denied));
    assert!(!speaks(Kind::NotSuitable));
    assert!(speaks(Kind::Inventory));
    assert!(speaks(Kind::Fit));
    assert!(!speaks(Kind::Schedule));
    assert!(!speaks(Kind::Budget));
    assert!(!speaks(Kind::Tip));
    assert!(speaks(Kind::Praise));
}

#[test]
fn k2_identity_contains_only_source_kind_and_location() {
    let expected = Identity {
        source: Source::Rule("fit.rule".to_owned()),
        kind: Kind::Fit,
        location: Location::Entity("drawer".to_owned()),
    };
    let first = with_facts(
        finding(
            expected.kind,
            expected.source.clone(),
            expected.location.clone(),
        ),
        &[("gap", "6")],
    );
    let mut second = first.clone();
    second.severity = Severity::Critical;
    second.what = "different words".to_owned();
    second.why = "different reason".to_owned();
    second.fix = Some("different fix".to_owned());
    second.facts = vec![("gap".to_owned(), "99".to_owned())];

    assert_eq!(identity(&first), expected);
    assert_eq!(identity(&first), identity(&second));

    let mut different_kind = first.clone();
    different_kind.kind = Kind::Inventory;
    assert_ne!(identity(&first), identity(&different_kind));
    let mut different_source = first.clone();
    different_source.source = Source::Mind;
    assert_ne!(identity(&first), identity(&different_source));
    let mut different_location = first;
    different_location.location = Location::Step(2);
    assert_ne!(identity(&different_kind), identity(&different_location));
}

#[test]
fn k3_panel_order_matches_the_worked_sorting_example() {
    let ordered = vec![example_b(), example_a(), example_c(), example_d()];
    assert_eq!(panel_order(&ordered), vec![2, 1, 3, 0]);
}

#[test]
fn k3_panel_order_ties_fall_back_to_input_index() {
    let ordered = vec![
        finding(Kind::Tip, Source::Mind, Location::Project),
        with_facts(
            finding(Kind::Tip, Source::Rule("r2".to_owned()), Location::Step(2)),
            &[("different", "facts")],
        ),
        Finding {
            what: "different words".to_owned(),
            ..finding(Kind::Tip, Source::Mind, Location::Field("f".to_owned()))
        },
    ];
    assert_eq!(panel_order(&ordered), vec![0, 1, 2]);
}

#[test]
fn k3_panel_order_uses_kind_declaration_order() {
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
    let ordered: Vec<Finding> = kinds
        .iter()
        .rev()
        .map(|kind| finding(*kind, Source::Mind, Location::Project))
        .collect();
    let expected: Vec<usize> = (0..kinds.len()).rev().collect();
    assert_eq!(panel_order(&ordered), expected);
}

#[test]
fn e1_only_the_earlier_duplicate_is_delivered() {
    let mut ledger = Ledger::new();
    let first = finding(
        Kind::Tip,
        Source::Rule("same".to_owned()),
        Location::Project,
    );
    let second = with_facts(first.clone(), &[("changed", "facts")]);
    check(
        ledger.evaluate(&[first, second], Context::default()),
        vec![Delivery::Panel, Delivery::Hidden],
        1,
    );
}

#[test]
fn e2_unsafe_confirms_in_all_four_modes() {
    let danger = with_facts(
        finding(
            Kind::Unsafe,
            Source::Rule("safety".to_owned()),
            Location::Project,
        ),
        &[("material", "pvc")],
    );
    let mut ledger = Ledger::new();
    for mode in [Mode::On, Mode::Silence, Mode::Mute, Mode::Off] {
        ledger.set_mode(mode);
        check(
            ledger.evaluate(std::slice::from_ref(&danger), Context::default()),
            vec![Delivery::Confirm],
            0,
        );
    }
}

#[test]
fn e3_remembered_quiet_is_hidden_for_safe_and_unsafe_findings() {
    let mut ledger = Ledger::new();
    let tip = finding(Kind::Tip, Source::Mind, Location::Project);
    let danger = finding(
        Kind::Unsafe,
        Source::Rule("safety".to_owned()),
        Location::Entity("panel".to_owned()),
    );
    assert!(ledger.dismiss(&tip));
    assert!(ledger.override_finding(&danger, None).is_some());
    check(
        ledger.evaluate(&[tip, danger], Context::default()),
        vec![Delivery::Hidden, Delivery::Hidden],
        0,
    );
}

#[test]
fn e4_off_hides_non_safety_findings_without_remembering_them() {
    let tip = finding(Kind::Tip, Source::Mind, Location::Project);
    let inventory = finding(Kind::Inventory, Source::Mind, Location::Step(1));
    let mut ledger = Ledger::new();
    ledger.set_mode(Mode::Off);
    check(
        ledger.evaluate(&[tip.clone(), inventory.clone()], Context::default()),
        vec![Delivery::Hidden, Delivery::Hidden],
        0,
    );
    ledger.set_mode(Mode::On);
    check(
        ledger.evaluate(&[tip, inventory], Context::default()),
        vec![Delivery::Panel, Delivery::Speak],
        1,
    );
}

#[test]
fn e5_mute_panels_non_safety_findings_without_saying_them() {
    let inventory = finding(Kind::Inventory, Source::Mind, Location::Project);
    let mut ledger = Ledger::new();
    ledger.set_mode(Mode::Mute);
    check(
        ledger.evaluate(std::slice::from_ref(&inventory), Context::default()),
        vec![Delivery::Panel],
        0,
    );
    ledger.set_mode(Mode::On);
    check(
        ledger.evaluate(&[inventory], Context::default()),
        vec![Delivery::Speak],
        0,
    );
}

#[test]
fn e6_asked_speaks_even_for_a_tip_and_remembers_said() {
    let tip = finding(Kind::Tip, Source::Mind, Location::Project);
    let mut ledger = Ledger::new();
    let asked = Context {
        asked: true,
        coach: false,
    };
    check(
        ledger.evaluate(std::slice::from_ref(&tip), asked),
        vec![Delivery::Speak],
        0,
    );
    check(
        ledger.evaluate(&[tip], Context::default()),
        vec![Delivery::Panel],
        0,
    );
}

#[test]
fn e6_asked_bubbles_in_silence_and_still_remembers_said() {
    let tip = finding(Kind::Tip, Source::Mind, Location::Project);
    let mut ledger = Ledger::new();
    ledger.set_mode(Mode::Silence);
    let asked = Context {
        asked: true,
        coach: false,
    };
    check(
        ledger.evaluate(std::slice::from_ref(&tip), asked),
        vec![Delivery::Bubble],
        0,
    );
    check(
        ledger.evaluate(&[tip], Context::default()),
        vec![Delivery::Panel],
        0,
    );
}

#[test]
fn e7_remembered_said_waits_in_the_panel_without_a_badge() {
    let inventory = finding(Kind::Inventory, Source::Mind, Location::Project);
    let mut ledger = Ledger::new();
    check(
        ledger.evaluate(std::slice::from_ref(&inventory), Context::default()),
        vec![Delivery::Speak],
        0,
    );
    check(
        ledger.evaluate(&[inventory], Context::default()),
        vec![Delivery::Panel],
        0,
    );
}

#[test]
fn e7_facts_are_compared_in_their_original_order() {
    let ordered = with_facts(
        finding(Kind::Inventory, Source::Mind, Location::Project),
        &[("first", "1"), ("second", "2")],
    );
    let reversed = with_facts(ordered.clone(), &[("second", "2"), ("first", "1")]);
    let mut ledger = Ledger::new();
    check(
        ledger.evaluate(std::slice::from_ref(&ordered), Context::default()),
        vec![Delivery::Speak],
        0,
    );
    check(
        ledger.evaluate(std::slice::from_ref(&ordered), Context::default()),
        vec![Delivery::Panel],
        0,
    );
    check(
        ledger.evaluate(&[reversed], Context::default()),
        vec![Delivery::Speak],
        0,
    );
}

#[test]
fn e8_a_demoted_rule_panels_and_counts_toward_the_badge() {
    let inventory = finding(
        Kind::Inventory,
        Source::Rule("stock.rule".to_owned()),
        Location::Project,
    );
    let mut ledger = Ledger::new();
    ledger.set_overrides("stock.rule", DEMOTE_AFTER);
    check(
        ledger.evaluate(&[inventory], Context::default()),
        vec![Delivery::Panel],
        1,
    );
}

#[test]
fn e9_speaking_kinds_and_coach_mode_speak() {
    let inventory = finding(Kind::Inventory, Source::Mind, Location::Project);
    let mut direct = Ledger::new();
    check(
        direct.evaluate(&[inventory], Context::default()),
        vec![Delivery::Speak],
        0,
    );

    let tip = finding(Kind::Tip, Source::Mind, Location::Project);
    let mut coached = Ledger::new();
    check(
        coached.evaluate(
            &[tip],
            Context {
                asked: false,
                coach: true,
            },
        ),
        vec![Delivery::Speak],
        0,
    );
}

#[test]
fn e9_direct_and_coached_speech_both_bubble_in_silence() {
    let inventory = finding(Kind::Inventory, Source::Mind, Location::Project);
    let mut direct = Ledger::new();
    direct.set_mode(Mode::Silence);
    check(
        direct.evaluate(&[inventory], Context::default()),
        vec![Delivery::Bubble],
        0,
    );

    let tip = finding(Kind::Tip, Source::Mind, Location::Project);
    let mut coached = Ledger::new();
    coached.set_mode(Mode::Silence);
    check(
        coached.evaluate(
            &[tip],
            Context {
                asked: false,
                coach: true,
            },
        ),
        vec![Delivery::Bubble],
        0,
    );
}

#[test]
fn e10_an_unspoken_unasked_finding_panels_and_badges() {
    let tip = finding(
        Kind::Tip,
        Source::Rule("tip.rule".to_owned()),
        Location::Project,
    );
    let mut ledger = Ledger::new();
    check(
        ledger.evaluate(&[tip], Context::default()),
        vec![Delivery::Panel],
        1,
    );
}

#[test]
fn e11_badge_counts_only_findings_reaching_e8_or_e10() {
    let ordinary = finding(
        Kind::Tip,
        Source::Rule("ordinary".to_owned()),
        Location::Project,
    );
    let demoted = finding(
        Kind::Inventory,
        Source::Rule("demoted".to_owned()),
        Location::Entity("stock".to_owned()),
    );
    let spoken = finding(Kind::Fit, Source::Mind, Location::Step(4));
    let danger = finding(
        Kind::Unsafe,
        Source::Rule("safety".to_owned()),
        Location::Project,
    );
    let mut ledger = Ledger::new();
    ledger.set_overrides("demoted", 3);
    check(
        ledger.evaluate(
            &[ordinary.clone(), ordinary, demoted, spoken, danger],
            Context::default(),
        ),
        vec![
            Delivery::Panel,
            Delivery::Hidden,
            Delivery::Panel,
            Delivery::Speak,
            Delivery::Confirm,
        ],
        2,
    );
}

#[test]
fn e11_absence_from_an_evaluation_forgets_nothing() {
    let said = finding(Kind::Inventory, Source::Mind, Location::Project);
    let quiet = finding(Kind::Tip, Source::Mind, Location::Step(2));
    let mut ledger = Ledger::new();
    check(
        ledger.evaluate(std::slice::from_ref(&said), Context::default()),
        vec![Delivery::Speak],
        0,
    );
    assert!(ledger.dismiss(&quiet));
    check(ledger.evaluate(&[], Context::default()), vec![], 0);
    check(
        ledger.evaluate(&[said, quiet], Context::default()),
        vec![Delivery::Panel, Delivery::Hidden],
        0,
    );
}

#[test]
fn e11_an_empty_list_has_no_deliveries_and_no_badge() {
    let mut ledger = Ledger::new();
    check(ledger.evaluate(&[], Context::default()), vec![], 0);
}

#[test]
fn a1_dismiss_remembers_a_non_unsafe_finding_as_quiet() {
    let tip = with_facts(
        finding(Kind::Tip, Source::Mind, Location::Project),
        &[("topic", "setup")],
    );
    let mut ledger = Ledger::new();
    assert!(ledger.dismiss(&tip));
    check(
        ledger.evaluate(&[tip], Context::default()),
        vec![Delivery::Hidden],
        0,
    );
}

#[test]
fn a1_a_refused_unsafe_dismiss_changes_nothing() {
    let danger = with_facts(
        finding(
            Kind::Unsafe,
            Source::Rule("safety".to_owned()),
            Location::Project,
        ),
        &[("material", "pvc")],
    );
    let mut ledger = Ledger::new();
    let before = ledger.clone();
    assert!(!ledger.dismiss(&danger));
    assert_eq!(ledger, before);
    check(
        ledger.evaluate(&[danger], Context::default()),
        vec![Delivery::Confirm],
        0,
    );
}

#[test]
fn a2_override_trims_the_reason_and_increments_its_rule() {
    let item = finding(
        Kind::Fit,
        Source::Rule("fit.rule".to_owned()),
        Location::Project,
    );
    let mut ledger = Ledger::new();
    assert_eq!(
        ledger.override_finding(&item, Some("  cut by hand\t\n")),
        Some(Override {
            rule: "fit.rule".to_owned(),
            reason: Some("cut by hand".to_owned()),
        })
    );
    assert_eq!(ledger.overrides("fit.rule"), 1);
    check(
        ledger.evaluate(&[item], Context::default()),
        vec![Delivery::Hidden],
        0,
    );
}

#[test]
fn a2_empty_and_whitespace_reasons_become_none() {
    let item = finding(
        Kind::Tip,
        Source::Rule("tip.rule".to_owned()),
        Location::Project,
    );
    let expected = Some(Override {
        rule: "tip.rule".to_owned(),
        reason: None,
    });
    let mut ledger = Ledger::new();
    assert_eq!(ledger.override_finding(&item, Some(" \t\r\n ")), expected);
    assert_eq!(ledger.override_finding(&item, None), expected);
    assert_eq!(ledger.overrides("tip.rule"), 2);
}

#[test]
fn a2_mind_override_returns_none_but_still_quiets_the_finding() {
    let item = finding(Kind::Tip, Source::Mind, Location::Project);
    let mut ledger = Ledger::new();
    assert_eq!(ledger.override_finding(&item, Some("x")), None);
    check(
        ledger.evaluate(&[item], Context::default()),
        vec![Delivery::Hidden],
        0,
    );
}

#[test]
fn a2_rule_override_count_saturates() {
    let item = finding(
        Kind::Fit,
        Source::Rule("fit.rule".to_owned()),
        Location::Project,
    );
    let mut ledger = Ledger::new();
    ledger.set_overrides("fit.rule", u32::MAX);
    assert!(ledger.override_finding(&item, None).is_some());
    assert_eq!(ledger.overrides("fit.rule"), u32::MAX);
}

#[test]
fn a3_counts_are_replaced_and_demotion_starts_at_exactly_three() {
    let mut ledger = Ledger::new();
    assert_eq!(DEMOTE_AFTER, 3);
    assert_eq!(ledger.overrides("unknown"), 0);
    ledger.set_overrides("rule", 2);
    assert_eq!(ledger.overrides("rule"), 2);
    assert!(!ledger.is_demoted("rule"));
    ledger.set_overrides("rule", 3);
    assert!(ledger.is_demoted("rule"));
    ledger.set_overrides("rule", 8);
    assert_eq!(ledger.overrides("rule"), 8);
    assert!(ledger.is_demoted("rule"));
    ledger.set_overrides("rule", 1);
    assert_eq!(ledger.overrides("rule"), 1);
    assert!(!ledger.is_demoted("rule"));
}

#[test]
fn a4_set_mode_preserves_memories_and_override_counts() {
    let said = finding(
        Kind::Inventory,
        Source::Rule("stock".to_owned()),
        Location::Project,
    );
    let quiet = finding(Kind::Tip, Source::Mind, Location::Step(2));
    let mut ledger = Ledger::new();
    assert_eq!(ledger.mode(), Mode::On);
    check(
        ledger.evaluate(std::slice::from_ref(&said), Context::default()),
        vec![Delivery::Speak],
        0,
    );
    assert!(ledger.dismiss(&quiet));
    ledger.set_overrides("stock", 2);
    for mode in [Mode::Off, Mode::Mute, Mode::Silence, Mode::On] {
        ledger.set_mode(mode);
        assert_eq!(ledger.mode(), mode);
        assert_eq!(ledger.overrides("stock"), 2);
    }
    check(
        ledger.evaluate(&[said, quiet], Context::default()),
        vec![Delivery::Panel, Delivery::Hidden],
        0,
    );
}

#[test]
fn worked_example_all_twenty_rows_in_order_on_one_ledger() {
    let mut ledger = Ledger::new();
    assert_eq!(ledger.mode(), Mode::On);
    let mut a = example_a();
    let b = example_b();
    let mut c = example_c();
    let d = example_d();

    // 1: Inventory and Fit kinds speak; an unasked Tip waits in the panel.
    check(
        ledger.evaluate(&[a.clone(), b.clone(), c.clone()], Context::default()),
        vec![Delivery::Speak, Delivery::Panel, Delivery::Confirm],
        1,
    );
    // 2: a is remembered said, while c was not quieted by its confirm prompt.
    check(
        ledger.evaluate(&[a.clone(), b.clone(), c.clone()], Context::default()),
        vec![Delivery::Panel, Delivery::Panel, Delivery::Confirm],
        1,
    );
    // 3 and 4: Unsafe refuses dismissal and takes the override as its confirm.
    assert!(!ledger.dismiss(&c));
    assert_eq!(
        ledger.override_finding(&c, Some("  cutting it by hand  ")),
        Some(Override {
            rule: "safety.pvc-laser".to_owned(),
            reason: Some("cutting it by hand".to_owned()),
        })
    );
    check(
        ledger.evaluate(&[a.clone(), b.clone(), c.clone()], Context::default()),
        vec![Delivery::Panel, Delivery::Panel, Delivery::Hidden],
        1,
    );

    // 5 and 6: material facts changing from short=2 to short=3 make a new again.
    a.facts = vec![("short".to_owned(), "3".to_owned())];
    check(
        ledger.evaluate(&[a.clone(), b.clone(), c.clone()], Context::default()),
        vec![Delivery::Speak, Delivery::Panel, Delivery::Hidden],
        1,
    );
    // 7 and 8: b is now quiet; neither a's said panel nor quiet findings badge.
    assert!(ledger.dismiss(&b));
    check(
        ledger.evaluate(&[a.clone(), b.clone(), c.clone()], Context::default()),
        vec![Delivery::Panel, Delivery::Hidden, Delivery::Hidden],
        0,
    );
    // 9: asked precedes remembered said, so a speaks again.
    check(
        ledger.evaluate(
            &[a.clone(), b.clone(), c.clone()],
            Context {
                asked: true,
                coach: false,
            },
        ),
        vec![Delivery::Speak, Delivery::Hidden, Delivery::Hidden],
        0,
    );
    // 10: new facts bubble in Silence, while the other two remain quiet.
    ledger.set_mode(Mode::Silence);
    a.facts = vec![("short".to_owned(), "4".to_owned())];
    check(
        ledger.evaluate(&[a.clone(), b.clone(), c.clone()], Context::default()),
        vec![Delivery::Bubble, Delivery::Hidden, Delivery::Hidden],
        0,
    );
    // 11: quiet precedes Mute; Mute itself remembers nothing.
    ledger.set_mode(Mode::Mute);
    check(
        ledger.evaluate(
            &[a.clone(), b.clone(), c.clone(), d.clone()],
            Context::default(),
        ),
        vec![
            Delivery::Panel,
            Delivery::Hidden,
            Delivery::Hidden,
            Delivery::Panel,
        ],
        0,
    );
    // 12 and 13: Off hides ordinary findings but never the Unsafe confirm.
    ledger.set_mode(Mode::Off);
    check(
        ledger.evaluate(&[a.clone(), d.clone()], Context::default()),
        vec![Delivery::Hidden, Delivery::Hidden],
        0,
    );
    c.facts = vec![("material".to_owned(), "plastic.vinyl".to_owned())];
    check(
        ledger.evaluate(std::slice::from_ref(&c), Context::default()),
        vec![Delivery::Confirm],
        0,
    );
    // 14: d speaks because Mute and Off did not mark it said.
    ledger.set_mode(Mode::On);
    check(
        ledger.evaluate(std::slice::from_ref(&d), Context::default()),
        vec![Delivery::Speak],
        0,
    );
    // 15: the third override is exactly the demotion boundary.
    for _ in 0..3 {
        assert_eq!(
            ledger.override_finding(&d, None),
            Some(Override {
                rule: "fit.clearance".to_owned(),
                reason: None,
            })
        );
    }
    assert_eq!(ledger.overrides("fit.clearance"), 3);
    assert!(ledger.is_demoted("fit.clearance"));
    // 16: a different location has no said memory, but the rule is demoted.
    let d_right = Finding {
        location: Location::Entity("right-side".to_owned()),
        ..d.clone()
    };
    check(
        ledger.evaluate(&[d_right], Context::default()),
        vec![Delivery::Panel],
        1,
    );
    // 17: the second identical identity in one list is a duplicate.
    let b2 = Finding {
        location: Location::Step(4),
        ..b.clone()
    };
    check(
        ledger.evaluate(&[b2.clone(), b2], Context::default()),
        vec![Delivery::Panel, Delivery::Hidden],
        1,
    );
    // 18: a Mind override has no rule record to return, but quiets the finding.
    let m = Finding {
        severity: Severity::Info,
        ..finding(Kind::Tip, Source::Mind, Location::Project)
    };
    assert_eq!(ledger.override_finding(&m, Some("x")), None);
    check(
        ledger.evaluate(&[m], Context::default()),
        vec![Delivery::Hidden],
        0,
    );
    // 19: coach lets a fresh Tip identity speak without being asked.
    let b_step_9 = Finding {
        location: Location::Step(9),
        ..b
    };
    check(
        ledger.evaluate(
            &[b_step_9],
            Context {
                asked: false,
                coach: true,
            },
        ),
        vec![Delivery::Speak],
        0,
    );
    // 20: an empty evaluation remains empty after all that history.
    check(ledger.evaluate(&[], Context::default()), vec![], 0);
}
