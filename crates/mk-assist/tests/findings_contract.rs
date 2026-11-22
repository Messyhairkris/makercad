//! Contract tests for mk_assist::findings, written by Gordon as a PA (the Crew) from
//! crew/contracts/mk_findings.md. They were the hidden tests of campaign mk-core-13.

use mk_assist::findings::{
    identity, panel_order, speaks, Context, Delivery, Evaluation, Finding, Identity, Kind, Ledger,
    Location, Mode, Override, Severity, Source, DEMOTE_AFTER,
};

// The worked example and the rules agree row for row, so no CONTRACT NOTE was needed.

// ---- fixtures ------------------------------------------------------------

const PLAIN: Context = Context {
    asked: false,
    coach: false,
};
const ASKED: Context = Context {
    asked: true,
    coach: false,
};
const COACH: Context = Context {
    asked: false,
    coach: true,
};

fn rule(id: &str) -> Source {
    Source::Rule(id.to_string())
}

fn entity(name: &str) -> Location {
    Location::Entity(name.to_string())
}

fn pairs(facts: &[(&str, &str)]) -> Vec<(String, String)> {
    facts
        .iter()
        .map(|&(key, value)| (key.to_string(), value.to_string()))
        .collect()
}

fn finding(
    kind: Kind,
    severity: Severity,
    source: Source,
    location: Location,
    facts: &[(&str, &str)],
) -> Finding {
    Finding {
        kind,
        severity,
        source,
        location,
        what: "the app's words".to_string(),
        why: "the app's reason".to_string(),
        fix: None,
        facts: pairs(facts),
    }
}

fn eval(deliveries: Vec<Delivery>, badge: usize) -> Evaluation {
    Evaluation { deliveries, badge }
}

// The worked example's findings, as constructors.

fn stock_short(short: &str) -> Finding {
    finding(
        Kind::Inventory,
        Severity::Warning,
        rule("stock.short"),
        entity("drawer-side"),
        &[("short", short)],
    )
}

fn same_setup(location: Location) -> Finding {
    finding(
        Kind::Tip,
        Severity::Info,
        rule("stock.same-setup"),
        location,
        &[],
    )
}

fn pvc_laser(material: &str) -> Finding {
    finding(
        Kind::Unsafe,
        Severity::Critical,
        rule("safety.pvc-laser"),
        entity("panel-3"),
        &[("material", material)],
    )
}

fn clearance(side: &str) -> Finding {
    finding(
        Kind::Fit,
        Severity::Warning,
        rule("fit.clearance"),
        entity(side),
        &[("over_mm", "6")],
    )
}

fn mind_tip() -> Finding {
    finding(
        Kind::Tip,
        Severity::Info,
        Source::Mind,
        Location::Project,
        &[],
    )
}

// ---- K1: speaks(kind) ----------------------------------------------------

#[test]
fn k1_speaks_the_six_kinds_it_says() {
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
fn k1_speaks_all_ten_kinds() {
    let table = [
        (Kind::Forbidden, true),
        (Kind::Unsafe, true),
        (Kind::Denied, true),
        (Kind::NotSuitable, false),
        (Kind::Inventory, true),
        (Kind::Fit, true),
        (Kind::Schedule, false),
        (Kind::Budget, false),
        (Kind::Tip, false),
        (Kind::Praise, true),
    ];
    for (kind, want) in table {
        assert!(
            speaks(kind) == want,
            "speaks({:?}) should be {}",
            kind,
            want
        );
    }
}

// ---- K2: identity(finding) -----------------------------------------------

#[test]
fn k2_identity_is_source_kind_and_location() {
    let stock = stock_short("2");
    let want = Identity {
        source: rule("stock.short"),
        kind: Kind::Inventory,
        location: entity("drawer-side"),
    };
    assert_eq!(identity(&stock), want);
    // Every location shape is kept as written.
    let step = finding(
        Kind::Fit,
        Severity::Warning,
        rule("fit.step"),
        Location::Step(7),
        &[],
    );
    assert_eq!(
        identity(&step),
        Identity {
            source: rule("fit.step"),
            kind: Kind::Fit,
            location: Location::Step(7),
        }
    );
    let field = finding(
        Kind::Denied,
        Severity::Critical,
        Source::Mind,
        Location::Field("sheet.thickness".to_string()),
        &[],
    );
    assert_eq!(
        identity(&field),
        Identity {
            source: Source::Mind,
            kind: Kind::Denied,
            location: Location::Field("sheet.thickness".to_string()),
        }
    );
}

#[test]
fn k2_identity_ignores_facts_words_severity_and_fix() {
    let base = stock_short("2");
    let mut reworded = base.clone();
    reworded.severity = Severity::Critical;
    reworded.what = "something else entirely".to_string();
    reworded.why = "another reason".to_string();
    reworded.fix = Some("do this".to_string());
    reworded.facts = pairs(&[("short", "99")]);
    assert_eq!(identity(&base), identity(&reworded));
    // But a different source, kind or location is a different finding.
    let mut moved = base.clone();
    moved.location = entity("other-side");
    assert!(identity(&base) != identity(&moved));
    let mut recast = base.clone();
    recast.kind = Kind::Fit;
    assert!(identity(&base) != identity(&recast));
    let mut resourced = base.clone();
    resourced.source = Source::Mind;
    assert!(identity(&base) != identity(&resourced));
}

// ---- K3: panel_order(findings) -------------------------------------------

#[test]
fn k3_panel_order_severity_then_kind_then_index() {
    let b = same_setup(Location::Project);
    let a = stock_short("2");
    let c = pvc_laser("plastic.pvc");
    let d = clearance("left-side");
    // The worked example: c Critical; a and d Warning, Inventory before Fit; b Info.
    assert_eq!(panel_order(&[b, a, c, d]), vec![2, 1, 3, 0]);
}

#[test]
fn k3_panel_order_ties_on_severity_and_kind() {
    let x = finding(Kind::Fit, Severity::Warning, rule("r.1"), entity("a"), &[]);
    let y = finding(Kind::Fit, Severity::Warning, rule("r.2"), entity("b"), &[]);
    let z = finding(Kind::Fit, Severity::Info, rule("r.3"), entity("c"), &[]);
    let w = finding(
        Kind::Forbidden,
        Severity::Warning,
        rule("r.4"),
        entity("d"),
        &[],
    );
    // Warnings first: Forbidden (3) before the two Fit findings, which keep input order;
    // then the Info finding.
    assert_eq!(panel_order(&[x, y, z, w]), vec![3, 0, 1, 2]);
    // Identical findings of the same kind and severity keep input order too.
    let p = finding(
        Kind::Praise,
        Severity::Info,
        rule("r.5"),
        Location::Project,
        &[],
    );
    let q = finding(
        Kind::Praise,
        Severity::Info,
        rule("r.6"),
        Location::Project,
        &[],
    );
    let r = finding(
        Kind::Praise,
        Severity::Info,
        rule("r.7"),
        Location::Project,
        &[],
    );
    assert_eq!(panel_order(&[p, q, r]), vec![0, 1, 2]);
}

#[test]
fn k3_panel_order_returns_every_index_once() {
    let findings = vec![
        finding(
            Kind::Praise,
            Severity::Info,
            rule("r.1"),
            Location::Project,
            &[],
        ),
        finding(
            Kind::Budget,
            Severity::Critical,
            rule("r.2"),
            Location::Step(1),
            &[],
        ),
        finding(
            Kind::Denied,
            Severity::Warning,
            rule("r.3"),
            entity("e.1"),
            &[],
        ),
        finding(
            Kind::Forbidden,
            Severity::Critical,
            rule("r.4"),
            Location::Project,
            &[],
        ),
        finding(
            Kind::Schedule,
            Severity::Warning,
            rule("r.5"),
            entity("e.2"),
            &[],
        ),
        finding(
            Kind::Fit,
            Severity::Warning,
            rule("r.6"),
            entity("e.3"),
            &[],
        ),
    ];
    // Criticals by kind (Forbidden 3, Budget 1), then Warnings by kind (Denied 2,
    // Fit 5, Schedule 4), then the Info Praise.
    assert_eq!(panel_order(&findings), vec![3, 1, 2, 5, 4, 0]);
    let mut seen = panel_order(&findings);
    seen.sort();
    assert_eq!(seen, vec![0, 1, 2, 3, 4, 5]);
}

// ---- E1: an earlier duplicate in the same list is hidden -----------------

#[test]
fn e1_an_earlier_duplicate_identity_is_hidden() {
    let first = stock_short("2");
    let mut copy = stock_short("2");
    copy.severity = Severity::Critical;
    copy.facts = pairs(&[("short", "9")]);
    let mut ledger = Ledger::new();
    assert_eq!(
        ledger.evaluate(&[first, copy], PLAIN),
        eval(vec![Delivery::Speak, Delivery::Hidden], 0)
    );
    // A duplicate of a panel finding does not count twice on the badge either.
    let tip = same_setup(Location::Project);
    let mut ledger = Ledger::new();
    assert_eq!(
        ledger.evaluate(&[tip.clone(), tip], PLAIN),
        eval(vec![Delivery::Panel, Delivery::Hidden], 1)
    );
}

// ---- E2: unsafe needs one explicit confirm -------------------------------

#[test]
fn e2_an_unsafe_finding_needs_one_confirm() {
    let mut ledger = Ledger::new();
    assert_eq!(
        ledger.evaluate(&[pvc_laser("plastic.pvc")], PLAIN),
        eval(vec![Delivery::Confirm], 0)
    );
    // Confirm is not something said: it confirms again next time, and it never counts.
    assert_eq!(
        ledger.evaluate(&[pvc_laser("plastic.pvc")], ASKED),
        eval(vec![Delivery::Confirm], 0)
    );
}

#[test]
fn e2_an_unsafe_finding_confirms_in_every_mode() {
    for mode in [Mode::On, Mode::Silence, Mode::Mute, Mode::Off] {
        let mut ledger = Ledger::new();
        ledger.set_mode(mode);
        assert_eq!(
            ledger.evaluate(&[pvc_laser("plastic.pvc")], PLAIN),
            eval(vec![Delivery::Confirm], 0)
        );
        // The override is its confirm, and it quietens the finding in every mode.
        assert!(ledger
            .override_finding(&pvc_laser("plastic.pvc"), None)
            .is_some());
        assert_eq!(
            ledger.evaluate(&[pvc_laser("plastic.pvc")], PLAIN),
            eval(vec![Delivery::Hidden], 0)
        );
    }
}

// ---- E3: remembered quiet is hidden --------------------------------------

#[test]
fn e3_remembered_quiet_is_hidden() {
    let mut ledger = Ledger::new();
    assert!(ledger.dismiss(&clearance("left-side")));
    // Quiet is checked before Off, Mute and "asked": nothing brings it back.
    assert_eq!(
        ledger.evaluate(&[clearance("left-side")], PLAIN),
        eval(vec![Delivery::Hidden], 0)
    );
    assert_eq!(
        ledger.evaluate(&[clearance("left-side")], ASKED),
        eval(vec![Delivery::Hidden], 0)
    );
    for mode in [Mode::Silence, Mode::Mute, Mode::Off] {
        ledger.set_mode(mode);
        assert_eq!(
            ledger.evaluate(&[clearance("left-side")], COACH),
            eval(vec![Delivery::Hidden], 0)
        );
    }
}

// ---- E4: mode Off hides everything ---------------------------------------

#[test]
fn e4_mode_off_hides_every_finding() {
    let mut ledger = Ledger::new();
    ledger.set_mode(Mode::Off);
    assert_eq!(
        ledger.evaluate(
            &[
                stock_short("2"),
                same_setup(Location::Project),
                clearance("left-side"),
            ],
            ASKED,
        ),
        eval(
            vec![Delivery::Hidden, Delivery::Hidden, Delivery::Hidden,],
            0,
        )
    );
    // Off hides but forgets nothing: switched back on, nothing was said.
    ledger.set_mode(Mode::On);
    assert_eq!(
        ledger.evaluate(&[stock_short("2"), same_setup(Location::Project)], COACH,),
        eval(vec![Delivery::Speak, Delivery::Speak], 0)
    );
}

// ---- E5: mode Mute fills the panel, without a badge ----------------------

#[test]
fn e5_mode_mute_fills_the_panel_without_a_badge() {
    let mut ledger = Ledger::new();
    ledger.set_mode(Mode::Mute);
    assert_eq!(
        ledger.evaluate(
            &[
                stock_short("2"),
                same_setup(Location::Project),
                clearance("left-side"),
            ],
            ASKED,
        ),
        eval(vec![Delivery::Panel, Delivery::Panel, Delivery::Panel], 0)
    );
    // Mute said nothing, so nothing is remembered as said.
    ledger.set_mode(Mode::On);
    assert_eq!(
        ledger.evaluate(&[stock_short("2")], PLAIN),
        eval(vec![Delivery::Speak], 0)
    );
}

// ---- E6: the maker asked ------------------------------------------------

#[test]
fn e6_asked_speaks_and_remembers() {
    let tip = same_setup(Location::Project);
    let mut ledger = Ledger::new();
    assert_eq!(
        ledger.evaluate(std::slice::from_ref(&tip), ASKED),
        eval(vec![Delivery::Speak], 0)
    );
    // Said: the same facts now wait in the panel.
    assert_eq!(
        ledger.evaluate(&[tip], PLAIN),
        eval(vec![Delivery::Panel], 0)
    );
}

#[test]
fn e6_asked_in_silence_is_a_bubble() {
    let mut ledger = Ledger::new();
    ledger.set_mode(Mode::Silence);
    assert_eq!(
        ledger.evaluate(&[same_setup(Location::Project)], ASKED),
        eval(vec![Delivery::Bubble], 0)
    );
}

// ---- E7: remembered said waits in the panel ------------------------------

#[test]
fn e7_remembered_said_waits_in_the_panel() {
    let stock = stock_short("2");
    let mut ledger = Ledger::new();
    assert_eq!(
        ledger.evaluate(std::slice::from_ref(&stock), PLAIN),
        eval(vec![Delivery::Speak], 0)
    );
    // Said once: the panel, and the panel does not count.
    assert_eq!(
        ledger.evaluate(std::slice::from_ref(&stock), PLAIN),
        eval(vec![Delivery::Panel], 0)
    );
    // Asking again says it again, and re-remembers it as said.
    assert_eq!(
        ledger.evaluate(std::slice::from_ref(&stock), ASKED),
        eval(vec![Delivery::Speak], 0)
    );
    assert_eq!(
        ledger.evaluate(&[stock], PLAIN),
        eval(vec![Delivery::Panel], 0)
    );
}

// ---- E8: a demoted rule waits in the panel and counts --------------------

#[test]
fn e8_a_demoted_rule_waits_in_the_panel_and_counts() {
    let mut ledger = Ledger::new();
    // Fit speaks, so a Panel here can only come from the demotion.
    assert_eq!(
        ledger.evaluate(&[clearance("left-side")], PLAIN),
        eval(vec![Delivery::Speak], 0)
    );
    ledger.set_overrides("fit.clearance", DEMOTE_AFTER);
    assert!(ledger.is_demoted("fit.clearance"));
    assert_eq!(
        ledger.evaluate(&[clearance("right-side")], PLAIN),
        eval(vec![Delivery::Panel], 1)
    );
    // E8 does not remember it as said: it is judged, and counted, again.
    assert_eq!(
        ledger.evaluate(&[clearance("right-side")], PLAIN),
        eval(vec![Delivery::Panel], 1)
    );
}

// ---- E9: a speaking kind, or coach mode ---------------------------------

#[test]
fn e9_a_speaking_kind_is_said_aloud() {
    // Unsafe is a speaking kind (K1) but E2 always decides it first, so it is left out.
    let kinds = [
        Kind::Forbidden,
        Kind::Denied,
        Kind::Inventory,
        Kind::Fit,
        Kind::Praise,
    ];
    for kind in kinds {
        let mut ledger = Ledger::new();
        let said = finding(
            kind,
            Severity::Warning,
            rule("r.speak"),
            Location::Project,
            &[],
        );
        assert_eq!(
            ledger.evaluate(&[said], PLAIN),
            eval(vec![Delivery::Speak], 0)
        );
    }
}

#[test]
fn e9_coach_speaks_a_kind_that_does_not_speak() {
    for kind in [Kind::NotSuitable, Kind::Schedule, Kind::Budget, Kind::Tip] {
        let mut ledger = Ledger::new();
        let coachable = finding(kind, Severity::Info, rule("r.quiet"), entity("chair"), &[]);
        assert_eq!(
            ledger.evaluate(&[coachable], COACH),
            eval(vec![Delivery::Speak], 0)
        );
    }
}

#[test]
fn e9_in_silence_is_a_bubble() {
    let mut ledger = Ledger::new();
    ledger.set_mode(Mode::Silence);
    assert_eq!(
        ledger.evaluate(&[stock_short("2")], PLAIN),
        eval(vec![Delivery::Bubble], 0)
    );
    // Silence is a voice setting, not a memory setting: the next one is a panel row.
    assert_eq!(
        ledger.evaluate(&[stock_short("2")], PLAIN),
        eval(vec![Delivery::Panel], 0)
    );
}

// ---- E10: anything else waits in the panel and counts -------------------

#[test]
fn e10_anything_else_waits_in_the_panel_and_counts() {
    for kind in [Kind::NotSuitable, Kind::Schedule, Kind::Budget, Kind::Tip] {
        let mut ledger = Ledger::new();
        let quiet = finding(
            kind,
            Severity::Info,
            rule("r.quiet"),
            Location::Project,
            &[],
        );
        assert_eq!(
            ledger.evaluate(&[quiet], PLAIN),
            eval(vec![Delivery::Panel], 1)
        );
    }
}

// ---- E11: the badge -----------------------------------------------------

#[test]
fn e11_the_badge_counts_only_e8_and_e10() {
    let mut ledger = Ledger::new();
    ledger.set_overrides("fit.clearance", DEMOTE_AFTER);
    let list = vec![
        stock_short("2"),              // E9 Speak
        pvc_laser("plastic.pvc"),      // E2 Confirm
        same_setup(Location::Project), // E10 Panel, counts
        clearance("left-side"),        // E8 Panel, counts
    ];
    assert_eq!(
        ledger.evaluate(&list, PLAIN),
        eval(
            vec![
                Delivery::Speak,
                Delivery::Confirm,
                Delivery::Panel,
                Delivery::Panel,
            ],
            2,
        )
    );
    // E7's panel row and E5's do not count.
    let mut ledger = Ledger::new();
    assert_eq!(
        ledger.evaluate(&[stock_short("2")], PLAIN),
        eval(vec![Delivery::Speak], 0)
    );
    assert_eq!(
        ledger.evaluate(&[stock_short("2")], PLAIN),
        eval(vec![Delivery::Panel], 0)
    );
    ledger.set_mode(Mode::Mute);
    assert_eq!(
        ledger.evaluate(&[stock_short("2"), same_setup(Location::Project)], PLAIN),
        eval(vec![Delivery::Panel, Delivery::Panel], 0)
    );
    // Off's hidden rows do not count, and an empty list has no badge.
    ledger.set_mode(Mode::Off);
    assert_eq!(
        ledger.evaluate(&[same_setup(Location::Project)], PLAIN),
        eval(vec![Delivery::Hidden], 0)
    );
    assert_eq!(ledger.evaluate(&[], PLAIN), eval(vec![], 0));
}

// ---- the worked example, in order, on one ledger ------------------------

#[test]
fn worked_example_every_row_in_order() {
    let b = same_setup(Location::Project);
    let m = mind_tip();
    let mut ledger = Ledger::new();

    // 1
    assert_eq!(
        ledger.evaluate(
            &[stock_short("2"), b.clone(), pvc_laser("plastic.pvc")],
            PLAIN
        ),
        eval(vec![Delivery::Speak, Delivery::Panel, Delivery::Confirm], 1,)
    );
    // 2
    assert_eq!(
        ledger.evaluate(
            &[stock_short("2"), b.clone(), pvc_laser("plastic.pvc")],
            PLAIN
        ),
        eval(vec![Delivery::Panel, Delivery::Panel, Delivery::Confirm], 1,)
    );
    // 3: an unsafe finding cannot be dismissed
    assert!(!ledger.dismiss(&pvc_laser("plastic.pvc")));
    // 4
    assert_eq!(
        ledger.override_finding(&pvc_laser("plastic.pvc"), Some("  cutting it by hand  ")),
        Some(Override {
            rule: "safety.pvc-laser".to_string(),
            reason: Some("cutting it by hand".to_string()),
        })
    );
    // 5
    assert_eq!(
        ledger.evaluate(
            &[stock_short("2"), b.clone(), pvc_laser("plastic.pvc")],
            PLAIN
        ),
        eval(vec![Delivery::Panel, Delivery::Panel, Delivery::Hidden], 1,)
    );
    // 6: a's facts changed, so it is a new finding
    assert_eq!(
        ledger.evaluate(
            &[stock_short("3"), b.clone(), pvc_laser("plastic.pvc")],
            PLAIN
        ),
        eval(vec![Delivery::Speak, Delivery::Panel, Delivery::Hidden], 1,)
    );
    // 7
    assert!(ledger.dismiss(&b));
    // 8
    assert_eq!(
        ledger.evaluate(
            &[stock_short("3"), b.clone(), pvc_laser("plastic.pvc")],
            PLAIN
        ),
        eval(vec![Delivery::Panel, Delivery::Hidden, Delivery::Hidden], 0,)
    );
    // 9: asked says a again
    assert_eq!(
        ledger.evaluate(
            &[stock_short("3"), b.clone(), pvc_laser("plastic.pvc")],
            ASKED
        ),
        eval(vec![Delivery::Speak, Delivery::Hidden, Delivery::Hidden], 0,)
    );
    // 10: Silence turns the voice into a bubble
    ledger.set_mode(Mode::Silence);
    assert_eq!(
        ledger.evaluate(
            &[stock_short("4"), b.clone(), pvc_laser("plastic.pvc")],
            PLAIN
        ),
        eval(
            vec![Delivery::Bubble, Delivery::Hidden, Delivery::Hidden],
            0,
        )
    );
    // 11: Mute fills the panel for both new findings and remembers nothing
    ledger.set_mode(Mode::Mute);
    assert_eq!(
        ledger.evaluate(
            &[
                stock_short("4"),
                b.clone(),
                pvc_laser("plastic.pvc"),
                clearance("left-side"),
            ],
            PLAIN,
        ),
        eval(
            vec![
                Delivery::Panel,
                Delivery::Hidden,
                Delivery::Hidden,
                Delivery::Panel,
            ],
            0,
        )
    );
    // 12: Off hides everything
    ledger.set_mode(Mode::Off);
    assert_eq!(
        ledger.evaluate(&[stock_short("4"), clearance("left-side")], PLAIN),
        eval(vec![Delivery::Hidden, Delivery::Hidden], 0)
    );
    // 13: a new material is not the confirmed one, so it confirms even in Off
    assert_eq!(
        ledger.evaluate(&[pvc_laser("plastic.vinyl")], PLAIN),
        eval(vec![Delivery::Confirm], 0)
    );
    // 14: Mute and Off remembered nothing, so d speaks now
    ledger.set_mode(Mode::On);
    assert_eq!(
        ledger.evaluate(&[clearance("left-side")], PLAIN),
        eval(vec![Delivery::Speak], 0)
    );
    // 15: three overrides demote fit.clearance
    for _ in 0..DEMOTE_AFTER {
        assert!(ledger
            .override_finding(&clearance("left-side"), None)
            .is_some());
    }
    assert!(ledger.is_demoted("fit.clearance"));
    assert_eq!(ledger.overrides("fit.clearance"), 3);
    // 16: the same rule on another side is demoted to the panel and counts
    assert_eq!(
        ledger.evaluate(&[clearance("right-side")], PLAIN),
        eval(vec![Delivery::Panel], 1)
    );
    // 17: a duplicate in the same list is hidden and does not count
    let b2 = same_setup(Location::Step(4));
    assert_eq!(
        ledger.evaluate(&[b2.clone(), b2], PLAIN),
        eval(vec![Delivery::Panel, Delivery::Hidden], 1)
    );
    // 18: the Mind has no rule to demote, but the finding is still quietened
    assert_eq!(ledger.override_finding(&m, Some("x")), None);
    assert_eq!(
        ledger.evaluate(&[m], PLAIN),
        eval(vec![Delivery::Hidden], 0)
    );
    // 19: coach mode speaks a Tip
    assert_eq!(
        ledger.evaluate(&[same_setup(Location::Step(9))], COACH),
        eval(vec![Delivery::Speak], 0)
    );
    // 20
    assert_eq!(ledger.evaluate(&[], PLAIN), eval(vec![], 0));
    // and the panel order of the example's findings
    let b = same_setup(Location::Project);
    let a = stock_short("2");
    let c = pvc_laser("plastic.pvc");
    let d = clearance("left-side");
    assert_eq!(panel_order(&[b, a, c, d]), vec![2, 1, 3, 0]);
}

// ---- A1: dismiss ---------------------------------------------------------

#[test]
fn a1_dismiss_remembers_quiet() {
    let stock = stock_short("2");
    let mut ledger = Ledger::new();
    assert!(ledger.dismiss(&stock));
    // Not just for this list: every later evaluation with the same facts hides it.
    assert_eq!(
        ledger.evaluate(std::slice::from_ref(&stock), PLAIN),
        eval(vec![Delivery::Hidden], 0)
    );
    assert_eq!(
        ledger.evaluate(&[stock], ASKED),
        eval(vec![Delivery::Hidden], 0)
    );
    // A finding with no facts at all can be dismissed too.
    let tip = same_setup(Location::Project);
    assert!(ledger.dismiss(&tip));
    assert_eq!(
        ledger.evaluate(&[tip], PLAIN),
        eval(vec![Delivery::Hidden], 0)
    );
}

#[test]
fn a1_an_unsafe_finding_cannot_be_dismissed() {
    let mut ledger = Ledger::new();
    assert!(!ledger.dismiss(&pvc_laser("plastic.pvc")));
    // Nothing was quietened: it still needs its confirm.
    assert_eq!(
        ledger.evaluate(&[pvc_laser("plastic.pvc")], PLAIN),
        eval(vec![Delivery::Confirm], 0)
    );
}

#[test]
fn a1_a_refused_dismiss_changes_nothing() {
    let list = vec![
        stock_short("2"),
        pvc_laser("plastic.pvc"),
        same_setup(Location::Project),
    ];
    let mut base = Ledger::new();
    // A little history first, so "changes nothing" is not about a fresh ledger.
    assert!(base.dismiss(&stock_short("2")));
    assert_eq!(
        base.evaluate(&list, PLAIN),
        eval(
            vec![Delivery::Hidden, Delivery::Confirm, Delivery::Panel],
            1,
        )
    );

    let mut refused = base.clone();
    assert!(!refused.dismiss(&pvc_laser("plastic.pvc")));
    assert!(
        refused == base,
        "a refused dismiss must not touch the ledger"
    );

    // The next evaluation is the same as without the refused dismiss.
    let without = base.evaluate(&list, ASKED);
    let with = refused.evaluate(&list, ASKED);
    assert_eq!(with, without);
    assert_eq!(
        without,
        eval(
            vec![Delivery::Hidden, Delivery::Confirm, Delivery::Speak],
            0,
        )
    );
}

// ---- A2: override_finding ------------------------------------------------

#[test]
fn a2_override_remembers_quiet_and_counts_the_rule() {
    let mut ledger = Ledger::new();
    assert_eq!(
        ledger.override_finding(&pvc_laser("plastic.pvc"), Some("cutting it by hand")),
        Some(Override {
            rule: "safety.pvc-laser".to_string(),
            reason: Some("cutting it by hand".to_string()),
        })
    );
    assert_eq!(ledger.overrides("safety.pvc-laser"), 1);
    // Quiet is the confirm: nothing to confirm any more.
    assert_eq!(
        ledger.evaluate(&[pvc_laser("plastic.pvc")], PLAIN),
        eval(vec![Delivery::Hidden], 0)
    );
    // Another material is another fact list, so it confirms on its own.
    assert_eq!(
        ledger.evaluate(&[pvc_laser("plastic.vinyl")], PLAIN),
        eval(vec![Delivery::Confirm], 0)
    );
}

#[test]
fn a2_every_override_counts_and_repeats_the_rule() {
    let fitting = clearance("left-side");
    let mut ledger = Ledger::new();
    for expected in 1..=DEMOTE_AFTER {
        assert_eq!(
            ledger.override_finding(&fitting, None),
            Some(Override {
                rule: "fit.clearance".to_string(),
                reason: None,
            })
        );
        assert_eq!(ledger.overrides("fit.clearance"), expected);
    }
    assert!(ledger.is_demoted("fit.clearance"));
}

#[test]
fn a2_the_reason_is_trimmed() {
    let mut ledger = Ledger::new();
    assert_eq!(
        ledger.override_finding(&clearance("a"), Some("  cutting it by hand \t")),
        Some(Override {
            rule: "fit.clearance".to_string(),
            reason: Some("cutting it by hand".to_string()),
        })
    );
    // Inner spacing and non-ASCII text are kept exactly.
    assert_eq!(
        ledger.override_finding(&clearance("b"), Some("  taper 6 mm, caf\u{e9}  ")),
        Some(Override {
            rule: "fit.clearance".to_string(),
            reason: Some("taper 6 mm, caf\u{e9}".to_string()),
        })
    );
}

#[test]
fn a2_an_empty_or_blank_reason_is_none() {
    let mut ledger = Ledger::new();
    for reason in [Some(""), Some("   "), Some("\t\n "), None] {
        assert_eq!(
            ledger.override_finding(&clearance("left-side"), reason),
            Some(Override {
                rule: "fit.clearance".to_string(),
                reason: None,
            })
        );
    }
    // Every one of them still counted.
    assert_eq!(ledger.overrides("fit.clearance"), 4);
}

#[test]
fn a2_the_mind_has_no_rule_to_demote() {
    let mut ledger = Ledger::new();
    assert_eq!(ledger.override_finding(&mind_tip(), Some("x")), None);
    // No rule was counted, but the finding is quiet all the same.
    assert_eq!(ledger.overrides(""), 0);
    assert!(!ledger.is_demoted(""));
    assert_eq!(
        ledger.evaluate(&[mind_tip()], PLAIN),
        eval(vec![Delivery::Hidden], 0)
    );
}

#[test]
fn a2_an_override_count_saturates() {
    let mut ledger = Ledger::new();
    ledger.set_overrides("fit.clearance", u32::MAX - 1);
    assert!(ledger
        .override_finding(&clearance("left-side"), None)
        .is_some());
    assert_eq!(ledger.overrides("fit.clearance"), u32::MAX);
    assert!(ledger
        .override_finding(&clearance("left-side"), None)
        .is_some());
    assert_eq!(ledger.overrides("fit.clearance"), u32::MAX);
    assert!(ledger.is_demoted("fit.clearance"));
}

// ---- A3: overrides, set_overrides, is_demoted ---------------------------

#[test]
fn a3_overrides_get_set_and_demote() {
    let mut ledger = Ledger::new();
    assert_eq!(ledger.overrides("fit.clearance"), 0);
    assert!(!ledger.is_demoted("fit.clearance"));
    assert_eq!(DEMOTE_AFTER, 3);
    ledger.set_overrides("fit.clearance", DEMOTE_AFTER - 1);
    assert_eq!(ledger.overrides("fit.clearance"), DEMOTE_AFTER - 1);
    assert!(!ledger.is_demoted("fit.clearance"));
    ledger.set_overrides("fit.clearance", DEMOTE_AFTER);
    assert_eq!(ledger.overrides("fit.clearance"), DEMOTE_AFTER);
    assert!(ledger.is_demoted("fit.clearance"));
    ledger.set_overrides("fit.clearance", 9);
    assert_eq!(ledger.overrides("fit.clearance"), 9);
    // Rules are counted one by one.
    ledger.set_overrides("stock.short", 1);
    assert_eq!(ledger.overrides("stock.short"), 1);
    assert_eq!(ledger.overrides("fit.clearance"), 9);
    assert!(!ledger.is_demoted("stock.short"));
}

#[test]
fn a3_set_overrides_restores_a_demotion() {
    let fitting = clearance("left-side");
    let mut ledger = Ledger::new();
    for _ in 0..DEMOTE_AFTER {
        assert!(ledger.override_finding(&fitting, None).is_some());
    }
    assert!(ledger.is_demoted("fit.clearance"));
    // The app restores relationship memory: below the threshold the rule is back.
    ledger.set_overrides("fit.clearance", 1);
    assert_eq!(ledger.overrides("fit.clearance"), 1);
    assert!(!ledger.is_demoted("fit.clearance"));
    assert_eq!(
        ledger.evaluate(&[clearance("right-side")], PLAIN),
        eval(vec![Delivery::Speak], 0)
    );
}

// ---- A4: set_mode -------------------------------------------------------

#[test]
fn a4_set_mode_keeps_memory_and_the_counts() {
    let mut ledger = Ledger::new();
    assert!(ledger.dismiss(&clearance("left-side")));
    assert!(ledger.override_finding(&stock_short("2"), None).is_some());
    for mode in [Mode::Silence, Mode::Mute, Mode::Off, Mode::On] {
        ledger.set_mode(mode);
        assert_eq!(ledger.mode(), mode);
        assert_eq!(ledger.overrides("stock.short"), 1);
        // Both findings were quietened before any of this, and stay quietened.
        assert_eq!(
            ledger.evaluate(&[clearance("left-side"), stock_short("2")], ASKED),
            eval(vec![Delivery::Hidden, Delivery::Hidden], 0)
        );
    }
}

#[test]
fn a4_the_mode_is_reported_and_starts_on() {
    let mut ledger = Ledger::new();
    assert_eq!(ledger.mode(), Mode::On);
    for mode in [Mode::Silence, Mode::Mute, Mode::Off, Mode::On] {
        ledger.set_mode(mode);
        assert_eq!(ledger.mode(), mode);
    }
}

// ---- the ledger's memory across evaluations -----------------------------

#[test]
fn absence_from_an_evaluation_forgets_nothing() {
    let stock = stock_short("2");
    let tip = same_setup(Location::Project);
    let mut ledger = Ledger::new();
    assert_eq!(
        ledger.evaluate(std::slice::from_ref(&stock), PLAIN),
        eval(vec![Delivery::Speak], 0)
    );
    assert!(ledger.dismiss(&tip));
    assert_eq!(
        ledger.evaluate(std::slice::from_ref(&tip), PLAIN),
        eval(vec![Delivery::Hidden], 0)
    );
    // Evaluations that leave both out change nothing at all.
    assert_eq!(ledger.evaluate(&[], PLAIN), eval(vec![], 0));
    assert_eq!(
        ledger.evaluate(&[clearance("left-side")], PLAIN),
        eval(vec![Delivery::Speak], 0)
    );
    // Both come back with the same facts: still said, still quiet.
    assert_eq!(
        ledger.evaluate(&[stock, tip], PLAIN),
        eval(vec![Delivery::Panel, Delivery::Hidden], 0)
    );
}

#[test]
fn facts_are_compared_in_order() {
    let in_order = finding(
        Kind::Inventory,
        Severity::Warning,
        rule("stock.short"),
        entity("drawer-side"),
        &[("short", "2"), ("needed", "4")],
    );
    let reversed = finding(
        Kind::Inventory,
        Severity::Warning,
        rule("stock.short"),
        entity("drawer-side"),
        &[("needed", "4"), ("short", "2")],
    );
    let mut ledger = Ledger::new();
    assert_eq!(
        ledger.evaluate(std::slice::from_ref(&in_order), PLAIN),
        eval(vec![Delivery::Speak], 0)
    );
    // The same pairs in another order are a change, so it is said again.
    assert_eq!(
        ledger.evaluate(std::slice::from_ref(&reversed), PLAIN),
        eval(vec![Delivery::Speak], 0)
    );
    // And back again: the first order is no longer what was remembered.
    assert_eq!(
        ledger.evaluate(std::slice::from_ref(&in_order), PLAIN),
        eval(vec![Delivery::Speak], 0)
    );
    assert_eq!(
        ledger.evaluate(std::slice::from_ref(&in_order), PLAIN),
        eval(vec![Delivery::Panel], 0)
    );
    assert_eq!(
        ledger.evaluate(std::slice::from_ref(&reversed), PLAIN),
        eval(vec![Delivery::Speak], 0)
    );

    // Quiet is the facts too: a reordered list is not the dismissed finding.
    let mut ledger = Ledger::new();
    assert!(ledger.dismiss(&in_order));
    assert_eq!(
        ledger.evaluate(std::slice::from_ref(&in_order), PLAIN),
        eval(vec![Delivery::Hidden], 0)
    );
    assert_eq!(
        ledger.evaluate(&[reversed], PLAIN),
        eval(vec![Delivery::Speak], 0)
    );
}

// ---- edges and volume ---------------------------------------------------

#[test]
fn an_empty_list_gives_an_empty_evaluation() {
    let mut ledger = Ledger::new();
    for mode in [Mode::On, Mode::Silence, Mode::Mute, Mode::Off] {
        ledger.set_mode(mode);
        assert_eq!(ledger.evaluate(&[], PLAIN), eval(vec![], 0));
        assert_eq!(ledger.evaluate(&[], ASKED), eval(vec![], 0));
    }
    assert_eq!(panel_order(&[]), Vec::<usize>::new());
}

#[test]
fn a_new_ledger_is_on_and_empty() {
    let mut ledger = Ledger::new();
    assert_eq!(ledger.mode(), Mode::On);
    assert_eq!(ledger.overrides("fit.clearance"), 0);
    assert!(!ledger.is_demoted("fit.clearance"));
    assert_eq!(Context::default(), PLAIN);
    assert_eq!(Ledger::default(), ledger);
    assert_eq!(
        ledger.evaluate(&[same_setup(Location::Project)], PLAIN),
        eval(vec![Delivery::Panel], 1)
    );
}

#[test]
fn a_few_thousand_findings_never_panic() {
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
    let severities = [Severity::Info, Severity::Warning, Severity::Critical];
    let mut findings: Vec<Finding> = Vec::new();
    for i in 0..3_000usize {
        let location = match i % 4 {
            0 => Location::Project,
            1 => entity(&format!("entity-{}", i % 37)),
            2 => Location::Step((i % 11) as u32),
            _ => Location::Field(format!("field-{}", i % 5)),
        };
        let source = if i % 5 == 0 {
            Source::Mind
        } else {
            rule(&format!("rule.{}", i % 23))
        };
        let short = (i % 7).to_string();
        findings.push(finding(
            kinds[i % 10],
            severities[i % 3],
            source,
            location,
            &[("n", short.as_str())],
        ));
    }

    let mut ledger = Ledger::new();
    for mode in [Mode::On, Mode::Silence, Mode::Mute, Mode::Off] {
        ledger.set_mode(mode);
        for context in [PLAIN, ASKED, COACH] {
            let result = ledger.evaluate(&findings, context);
            assert!(result.deliveries.len() == findings.len());
            assert!(result.badge <= findings.len());
        }
    }

    // Every finding dismissed or overridden, then judged again.
    for (i, found) in findings.iter().enumerate() {
        if i % 2 == 0 {
            let _ = ledger.dismiss(found);
        } else {
            let _ = ledger.override_finding(found, Some("  "));
        }
    }
    let again = ledger.evaluate(&findings, PLAIN);
    assert!(again.deliveries.len() == findings.len());

    let order = panel_order(&findings);
    assert!(order.len() == findings.len());
    let mut seen = order;
    seen.sort();
    let every: Vec<usize> = (0..findings.len()).collect();
    assert_eq!(seen, every);
}
