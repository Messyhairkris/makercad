//! Contract tests for mk_assist::toolbox, written by Gordon as a PA (the Crew) from
//! crew/contracts/mk_toolbox.md. They were the hidden tests of campaign mk-core-29.

// Crew-written tests: a helper that builds a whole `Finding` takes one argument per
// field, and worked rows print constants to the contract's digits; neither matters in a
// test.
#![allow(clippy::too_many_arguments, clippy::approx_constant)]

use mk_assist::toolbox::{build, is_shown, reason_of, Command, Reason, Toolbox};

// Hidden tests for `mk_assist::toolbox` (contract `crew/contracts/mk_toolbox.md`).
// One test per rule, plus every row of the contract's worked-examples table.

fn cmd(id: &str, group: &str) -> Command {
    Command {
        id: id.to_string(),
        group: group.to_string(),
    }
}

/// The contract's example catalogue `CAT`: seven commands, in this default order.
fn cat() -> Vec<Command> {
    vec![
        cmd("select", "edit"),
        cmd("move", "edit"),
        cmd("box", "solid"),
        cmd("cylinder", "solid"),
        cmd("fillet", "solid"),
        cmd("measure", "inspect"),
        cmd("export", "file"),
    ]
}

fn ids(list: &[&str]) -> Vec<String> {
    list.iter().map(|id| id.to_string()).collect()
}

fn tb(shown: &[(&str, Reason)], rest: &[&str], intent_missing: bool) -> Toolbox {
    Toolbox {
        shown: shown
            .iter()
            .map(|(id, reason)| (id.to_string(), *reason))
            .collect(),
        rest: ids(rest),
        intent_missing,
    }
}

/// The contract's named lists: `P` pinned, `S` suggested, `A` always.
const P: [&str; 3] = ["fillet", "nope", "box"];
const S: [&str; 4] = ["box", "drawer_wizard", "measure", "fillet"];
const A: [&str; 2] = ["select", "slice"];

fn run(pinned: &[&str], suggested: &[&str], always: &[&str], missing: bool, max: usize) -> Toolbox {
    build(
        &cat(),
        &ids(suggested),
        &ids(pinned),
        &ids(always),
        missing,
        max,
    )
}

fn partition_ok(catalogue: &[Command], toolbox: &Toolbox) -> bool {
    let mut all: Vec<String> = toolbox.shown.iter().map(|(id, _)| id.clone()).collect();
    all.extend(toolbox.rest.iter().cloned());
    catalogue.len() == all.len()
        && catalogue
            .iter()
            .all(|command| all.iter().filter(|id| *id == &command.id).count() == 1)
}

// ---- A1: catalogue and ids -------------------------------------------------

#[test]
fn a1_duplicate_catalogue_ids_are_ignored() {
    let catalogue = [
        cmd("box", "solid"),
        cmd("box", "other"),
        cmd("move", "edit"),
    ];
    let got = build(&catalogue, &[], &[], &[], false, 0);
    assert!(got == tb(&[], &["box", "move"], false), "got {:?}", got);

    let got = build(&catalogue, &ids(&["box"]), &[], &[], false, 0);
    let want = tb(&[("box", Reason::Suggested)], &["move"], false);
    assert!(got == want, "got {:?}", got);

    let got = build(&catalogue, &[], &[], &[], true, 0);
    let want = tb(
        &[
            ("box", Reason::IntentMissing),
            ("move", Reason::IntentMissing),
        ],
        &[],
        true,
    );
    assert!(got == want, "got {:?}", got);
}

#[test]
fn a1_group_never_changes_the_result() {
    let one = [cmd("box", "solid"), cmd("move", "edit")];
    let two = [cmd("box", "zzz"), cmd("move", "aaa")];
    let got_one = build(&one, &ids(&["move"]), &ids(&["box"]), &[], false, 0);
    let got_two = build(&two, &ids(&["move"]), &ids(&["box"]), &[], false, 0);
    assert!(got_one == got_two, "{:?} vs {:?}", got_one, got_two);
    let want = tb(
        &[("box", Reason::Pinned), ("move", Reason::Suggested)],
        &[],
        false,
    );
    assert!(got_one == want, "got {:?}", got_one);
}

// ---- A2: candidate order ---------------------------------------------------

#[test]
fn a2_pinned_then_suggested_then_always() {
    let got = run(&P, &S, &A, false, 0);
    let want = tb(
        &[
            ("fillet", Reason::Pinned),
            ("box", Reason::Pinned),
            ("measure", Reason::Suggested),
            ("select", Reason::Always),
        ],
        &["move", "cylinder", "export"],
        false,
    );
    assert!(got == want, "got {:?}", got);
}

#[test]
fn a2_unknown_ids_are_ignored() {
    let got = run(&P, &S, &A, false, 0);
    assert!(
        !got.shown
            .iter()
            .any(|(id, _)| id == "nope" || id == "drawer_wizard" || id == "slice"),
        "got {:?}",
        got
    );
    assert!(
        !got.rest
            .iter()
            .any(|id| id == "nope" || id == "drawer_wizard" || id == "slice"),
        "got {:?}",
        got
    );
}

#[test]
fn a2_each_id_appears_once_with_the_first_reason() {
    let got = build(
        &cat(),
        &ids(&["fillet"]),
        &ids(&["box", "fillet", "box"]),
        &ids(&["box"]),
        false,
        0,
    );
    let want = tb(
        &[("box", Reason::Pinned), ("fillet", Reason::Pinned)],
        &["select", "move", "cylinder", "measure", "export"],
        false,
    );
    assert!(got == want, "got {:?}", got);
}

// ---- A3: the cut, shown and rest -------------------------------------------

#[test]
fn a3_the_cut_keeps_the_first_candidates() {
    let got = run(&P, &S, &A, false, 3);
    let want = tb(
        &[
            ("fillet", Reason::Pinned),
            ("box", Reason::Pinned),
            ("measure", Reason::Suggested),
        ],
        &["select", "move", "cylinder", "export"],
        false,
    );
    assert!(got == want, "got {:?}", got);
}

#[test]
fn a3_zero_means_no_cut() {
    let got = run(&P, &S, &A, false, 0);
    assert!(got.shown.len() == 4, "got {:?}", got);
    let got = run(&P, &S, &A, false, 40);
    assert!(got.shown.len() == 4, "got {:?}", got);
}

#[test]
fn a3_rest_is_every_other_id_in_default_order() {
    let got = build(&cat(), &ids(&["measure"]), &[], &ids(&["select"]), false, 0);
    let want = tb(
        &[("measure", Reason::Suggested), ("select", Reason::Always)],
        &["move", "box", "cylinder", "fillet", "export"],
        false,
    );
    assert!(got == want, "got {:?}", got);
}

#[test]
fn a3_shown_and_rest_partition_the_catalogue() {
    for max in [0usize, 1, 2, 4, 7, 99] {
        let got = run(&P, &S, &A, false, max);
        assert!(partition_ok(&cat(), &got), "max {}: {:?}", max, got);
        let got = run(&P, &S, &A, true, max);
        assert!(partition_ok(&cat(), &got), "max {} missing: {:?}", max, got);
    }
}

// ---- A4: intent missing ----------------------------------------------------

#[test]
fn a4_intent_missing_pinned_first_then_default_order() {
    let got = run(&P, &S, &A, true, 0);
    let want = tb(
        &[
            ("fillet", Reason::Pinned),
            ("box", Reason::Pinned),
            ("select", Reason::IntentMissing),
            ("move", Reason::IntentMissing),
            ("cylinder", Reason::IntentMissing),
            ("measure", Reason::IntentMissing),
            ("export", Reason::IntentMissing),
        ],
        &[],
        true,
    );
    assert!(got == want, "got {:?}", got);
}

#[test]
fn a4_intent_missing_ignores_suggested_and_always() {
    let got = run(&[], &["box"], &["measure"], true, 0);
    let want = tb(
        &[
            ("select", Reason::IntentMissing),
            ("move", Reason::IntentMissing),
            ("box", Reason::IntentMissing),
            ("cylinder", Reason::IntentMissing),
            ("fillet", Reason::IntentMissing),
            ("measure", Reason::IntentMissing),
            ("export", Reason::IntentMissing),
        ],
        &[],
        true,
    );
    assert!(got == want, "got {:?}", got);
}

#[test]
fn a4_intent_missing_still_cuts() {
    let got = run(&P, &S, &A, true, 3);
    let want = tb(
        &[
            ("fillet", Reason::Pinned),
            ("box", Reason::Pinned),
            ("select", Reason::IntentMissing),
        ],
        &["move", "cylinder", "measure", "export"],
        true,
    );
    assert!(got == want, "got {:?}", got);
}

#[test]
fn a4_intent_missing_ignores_unknown_pinned() {
    let got = build(&cat(), &[], &ids(&["ghost", "measure"]), &[], true, 0);
    let want = tb(
        &[
            ("measure", Reason::Pinned),
            ("select", Reason::IntentMissing),
            ("move", Reason::IntentMissing),
            ("box", Reason::IntentMissing),
            ("cylinder", Reason::IntentMissing),
            ("fillet", Reason::IntentMissing),
            ("export", Reason::IntentMissing),
        ],
        &[],
        true,
    );
    assert!(got == want, "got {:?}", got);
}

// ---- A5: the flag ----------------------------------------------------------

#[test]
fn a5_flag_is_echoed() {
    let got = run(&[], &[], &[], false, 0);
    assert!(!got.intent_missing, "got {:?}", got);
    let got = run(&[], &[], &[], true, 0);
    assert!(got.intent_missing, "got {:?}", got);
    let got = build(&[], &[], &[], &[], true, 0);
    assert!(got.intent_missing, "got {:?}", got);
    let got = run(&P, &S, &A, true, 99);
    assert!(got.intent_missing, "got {:?}", got);
    let got = run(&P, &S, &A, false, 99);
    assert!(!got.intent_missing, "got {:?}", got);
}

// ---- A6: lookups -----------------------------------------------------------

#[test]
fn a6_is_shown_reads_shown_only() {
    let toolbox = run(&P, &S, &A, false, 0);
    assert!(is_shown(&toolbox, "fillet"));
    assert!(is_shown(&toolbox, "select"));
    assert!(!is_shown(&toolbox, "move"), "move is only in rest");
    assert!(!is_shown(&toolbox, "nope"));
    assert!(!is_shown(&toolbox, ""));
}

#[test]
fn a6_reason_of_returns_the_pair_reason() {
    let toolbox = run(&P, &S, &A, false, 0);
    assert!(reason_of(&toolbox, "fillet") == Some(Reason::Pinned));
    assert!(reason_of(&toolbox, "box") == Some(Reason::Pinned));
    assert!(reason_of(&toolbox, "measure") == Some(Reason::Suggested));
    assert!(reason_of(&toolbox, "select") == Some(Reason::Always));
    let missing = run(&P, &S, &A, true, 0);
    assert!(reason_of(&missing, "fillet") == Some(Reason::Pinned));
    assert!(reason_of(&missing, "move") == Some(Reason::IntentMissing));
}

#[test]
fn a6_reason_of_absent_is_none() {
    let toolbox = run(&P, &S, &A, false, 0);
    assert!(reason_of(&toolbox, "move").is_none());
    assert!(reason_of(&toolbox, "ghost").is_none());
    let empty = build(
        &[],
        &ids(&["box"]),
        &ids(&["box"]),
        &ids(&["box"]),
        false,
        0,
    );
    assert!(reason_of(&empty, "box").is_none());
    assert!(!is_shown(&empty, "box"));
}

// ---- A7: no panics, no guessing --------------------------------------------

#[test]
fn a7_empty_catalogue() {
    for missing in [false, true] {
        let got = build(
            &[],
            &ids(&["box"]),
            &ids(&["box"]),
            &ids(&["box"]),
            missing,
            0,
        );
        assert!(got.shown.is_empty(), "got {:?}", got);
        assert!(got.rest.is_empty(), "got {:?}", got);
        assert!(got.intent_missing == missing, "got {:?}", got);
        assert!(!is_shown(&got, "box"));
        assert!(reason_of(&got, "box").is_none());
    }
}

#[test]
fn a7_empty_lists() {
    let got = build(&cat(), &[], &[], &[], false, 0);
    assert!(got.shown.is_empty(), "got {:?}", got);
    let want_rest = [
        "select", "move", "box", "cylinder", "fillet", "measure", "export",
    ];
    assert!(got.rest == ids(&want_rest), "got {:?}", got);
    let got = build(&cat(), &[], &[], &[], false, 3);
    assert!(got.shown.is_empty(), "got {:?}", got);
    assert!(got.rest == ids(&want_rest), "got {:?}", got);
}

#[test]
fn a7_huge_max_shown() {
    let got = run(&P, &S, &A, false, usize::MAX);
    assert!(got.shown.len() == 4, "got {:?}", got);
    assert!(
        got.rest == ids(&["move", "cylinder", "export"]),
        "got {:?}",
        got
    );
    let got = run(&[], &[], &[], true, usize::MAX);
    assert!(got.shown.len() == 7, "got {:?}", got);
    assert!(got.rest.is_empty(), "got {:?}", got);
}

#[test]
fn a7_mixed_junk_never_panics() {
    let catalogue = [
        cmd("", "g"),
        cmd("Box", "g"),
        cmd("box", "g"),
        cmd("box ", "g"),
    ];
    let pinned = ids(&["", "Box", "BOX", "box", "box "]);
    let suggested = ids(&["box", ""]);
    let always = ids(&["\u{0}", "box "]);
    let got = build(&catalogue, &suggested, &pinned, &always, false, usize::MAX);
    assert!(got.shown.len() == 4, "got {:?}", got);
    assert!(
        got.shown
            .iter()
            .all(|(_, reason)| *reason == Reason::Pinned),
        "got {:?}",
        got
    );
    assert!(got.rest.is_empty(), "got {:?}", got);
    let got = build(&catalogue, &suggested, &pinned, &always, true, 0);
    assert!(partition_ok(&catalogue, &got), "got {:?}", got);
    assert!(!is_shown(&got, "BOX"));
    assert!(reason_of(&got, "BOX").is_none());
}

// ---- The contract's worked examples, one test per row -----------------------

#[test]
fn row01_pinned_then_suggested_then_always() {
    let got = run(&P, &S, &A, false, 5);
    let want = tb(
        &[
            ("fillet", Reason::Pinned),
            ("box", Reason::Pinned),
            ("measure", Reason::Suggested),
            ("select", Reason::Always),
        ],
        &["move", "cylinder", "export"],
        false,
    );
    assert!(got == want, "got {:?}", got);
}

#[test]
fn row02_cut_at_two() {
    let got = run(&P, &S, &A, false, 2);
    let want = tb(
        &[("fillet", Reason::Pinned), ("box", Reason::Pinned)],
        &["select", "move", "cylinder", "measure", "export"],
        false,
    );
    assert!(got == want, "got {:?}", got);
}

#[test]
fn row03_zero_is_no_cut() {
    let got = run(&P, &S, &A, false, 0);
    let want = tb(
        &[
            ("fillet", Reason::Pinned),
            ("box", Reason::Pinned),
            ("measure", Reason::Suggested),
            ("select", Reason::Always),
        ],
        &["move", "cylinder", "export"],
        false,
    );
    assert!(got == want, "got {:?}", got);
}

#[test]
fn row04_suggested_repeated() {
    let got = build(
        &cat(),
        &ids(&["measure", "box", "measure"]),
        &[],
        &[],
        false,
        0,
    );
    let want = tb(
        &[("measure", Reason::Suggested), ("box", Reason::Suggested)],
        &["select", "move", "cylinder", "fillet", "export"],
        false,
    );
    assert!(got == want, "got {:?}", got);
}

#[test]
fn row05_box_in_every_list_is_pinned_once() {
    let got = build(
        &cat(),
        &ids(&["box"]),
        &ids(&["box", "box"]),
        &ids(&["box"]),
        false,
        0,
    );
    let want = tb(
        &[("box", Reason::Pinned)],
        &["select", "move", "cylinder", "fillet", "measure", "export"],
        false,
    );
    assert!(got == want, "got {:?}", got);
}

#[test]
fn row06_intent_missing_whole_catalogue() {
    let got = run(&P, &S, &A, true, 0);
    let want = tb(
        &[
            ("fillet", Reason::Pinned),
            ("box", Reason::Pinned),
            ("select", Reason::IntentMissing),
            ("move", Reason::IntentMissing),
            ("cylinder", Reason::IntentMissing),
            ("measure", Reason::IntentMissing),
            ("export", Reason::IntentMissing),
        ],
        &[],
        true,
    );
    assert!(got == want, "got {:?}", got);
}

#[test]
fn row07_intent_missing_cut_at_three() {
    let got = run(&P, &S, &A, true, 3);
    let want = tb(
        &[
            ("fillet", Reason::Pinned),
            ("box", Reason::Pinned),
            ("select", Reason::IntentMissing),
        ],
        &["move", "cylinder", "measure", "export"],
        true,
    );
    assert!(got == want, "got {:?}", got);
}

#[test]
fn row08_intent_missing_ignores_suggested_and_always() {
    let got = run(&[], &["box"], &["measure"], true, 0);
    let want = tb(
        &[
            ("select", Reason::IntentMissing),
            ("move", Reason::IntentMissing),
            ("box", Reason::IntentMissing),
            ("cylinder", Reason::IntentMissing),
            ("fillet", Reason::IntentMissing),
            ("measure", Reason::IntentMissing),
            ("export", Reason::IntentMissing),
        ],
        &[],
        true,
    );
    assert!(got == want, "got {:?}", got);
}

#[test]
fn row09_unknown_ids_only() {
    let got = run(&["ghost"], &["phantom"], &["slice"], false, 0);
    let want = tb(
        &[],
        &[
            "select", "move", "box", "cylinder", "fillet", "measure", "export",
        ],
        false,
    );
    assert!(got == want, "got {:?}", got);
}

#[test]
fn row10_empty_catalogue_names_nothing() {
    let got = build(
        &[],
        &ids(&["box"]),
        &ids(&["box"]),
        &ids(&["box"]),
        false,
        0,
    );
    assert!(got == tb(&[], &[], false), "got {:?}", got);
}

#[test]
fn row11_always_last_and_no_cut() {
    let got = build(
        &cat(),
        &ids(&["box"]),
        &ids(&["measure"]),
        &ids(&["select"]),
        false,
        10,
    );
    let want = tb(
        &[
            ("measure", Reason::Pinned),
            ("box", Reason::Suggested),
            ("select", Reason::Always),
        ],
        &["move", "cylinder", "fillet", "export"],
        false,
    );
    assert!(got == want, "got {:?}", got);
}

#[test]
fn row12_empty_catalogue_intent_missing() {
    let got = build(&[], &[], &ids(&["box"]), &[], true, 0);
    assert!(got == tb(&[], &[], true), "got {:?}", got);
}

#[test]
fn row13_cut_at_one() {
    let got = build(&cat(), &[], &ids(&["fillet"]), &[], false, 1);
    let want = tb(
        &[("fillet", Reason::Pinned)],
        &["select", "move", "box", "cylinder", "measure", "export"],
        false,
    );
    assert!(got == want, "got {:?}", got);
}

#[test]
fn lookup_rows_on_row01() {
    let got = run(&P, &S, &A, false, 5);
    assert!(is_shown(&got, "fillet"));
    assert!(reason_of(&got, "fillet") == Some(Reason::Pinned));
    assert!(is_shown(&got, "select"));
    assert!(reason_of(&got, "select") == Some(Reason::Always));
    assert!(!is_shown(&got, "move"));
    assert!(reason_of(&got, "move").is_none());
}
