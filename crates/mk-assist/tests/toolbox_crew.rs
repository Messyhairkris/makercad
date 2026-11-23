//! Tests for mk_assist::toolbox written by Hoss (the Crew, campaign mk-core-29) from the
//! contract alone, without seeing the module.

//! Integration tests for `mk_assist::toolbox` based on the contract.

// Crew-written tests: a helper that builds a whole `Finding` takes one argument per
// field, and worked rows print constants to the contract's digits; neither matters in a
// test.
#![allow(clippy::too_many_arguments, clippy::approx_constant)]

use mk_assist::toolbox::{build, is_shown, reason_of, Command, Reason, Toolbox};

/// Helper to build a catalogue from a slice of ids. `group` is irrelevant.
fn catalogue(ids: &[&str]) -> Vec<Command> {
    ids.iter()
        .map(|&id| Command {
            id: id.to_string(),
            group: String::new(),
        })
        .collect()
}

/// Helper to build a `Toolbox` from shown pairs and rest ids.
fn make_toolbox(shown: &[(&str, Reason)], rest: &[&str], intent_missing: bool) -> Toolbox {
    Toolbox {
        shown: shown.iter().map(|&(id, r)| (id.to_string(), r)).collect(),
        rest: rest.iter().map(|&s| s.to_string()).collect(),
        intent_missing,
    }
}

/// Convert a slice of `&str` to a `Vec<String>`.
fn strs(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

// ---------- Rule A1 ----------
#[test]
fn rule_a1_catalogue_and_ids() {
    // catalogue with duplicate ids; later duplicates ignored.
    let cat = catalogue(&["a", "b", "a"]);
    let pinned = strs(&["a"]);
    let t = build(&cat, &[], &pinned, &[], false, 0);
    let expected = make_toolbox(&[("a", Reason::Pinned)], &["b"], false);
    assert_eq!(t, expected);
}

// ---------- Rule A2 ----------
#[test]
fn rule_a2_candidate_order_intent_missing_false() {
    let cat = catalogue(&["x", "y", "z"]);
    let suggested = strs(&["y"]);
    let pinned = strs(&["z"]);
    let always = strs(&["x"]);
    let t = build(&cat, &suggested, &pinned, &always, false, 0);
    let expected = make_toolbox(
        &[
            ("z", Reason::Pinned),
            ("y", Reason::Suggested),
            ("x", Reason::Always),
        ],
        &[],
        false,
    );
    assert_eq!(t, expected);
}

// ---------- Rule A3 ----------
#[test]
fn rule_a3_cut_shown_and_rest() {
    let cat = catalogue(&["a", "b", "c", "d"]);
    let suggested = strs(&["b"]);
    let pinned = strs(&["a"]);
    let always = strs(&["c"]);
    let t = build(&cat, &suggested, &pinned, &always, false, 2);
    let expected = make_toolbox(
        &[("a", Reason::Pinned), ("b", Reason::Suggested)],
        &["c", "d"],
        false,
    );
    assert_eq!(t, expected);
}

// ---------- Rule A4 ----------
#[test]
fn rule_a4_intent_missing_true() {
    let cat = catalogue(&["a", "b", "c"]);
    let suggested = strs(&["b"]);
    let pinned = strs(&["a"]);
    let always = strs(&["c"]);
    let t = build(&cat, &suggested, &pinned, &always, true, 0);
    let expected = make_toolbox(
        &[
            ("a", Reason::Pinned),
            ("b", Reason::IntentMissing),
            ("c", Reason::IntentMissing),
        ],
        &[],
        true,
    );
    assert_eq!(t, expected);
}

// ---------- Rule A5 ----------
#[test]
fn rule_a5_intent_missing_flag_echoed() {
    let cat = catalogue(&["a"]);
    let t = build(&cat, &[], &[], &[], true, 5);
    assert!(t.intent_missing);
}

// ---------- Rule A6 ----------
#[test]
fn rule_a6_lookups_is_shown_and_reason_of() {
    let cat = catalogue(&["a", "b"]);
    let suggested = strs(&["b"]);
    let pinned = strs(&["a"]);
    let t = build(&cat, &suggested, &pinned, &[], false, 0);
    assert!(is_shown(&t, "a"));
    assert_eq!(reason_of(&t, "a"), Some(Reason::Pinned));
    assert!(!is_shown(&t, "c"));
    assert_eq!(reason_of(&t, "c"), None);
}

// ---------- Rule A7 ----------
#[test]
fn rule_a7_no_panic_on_empty_and_unknown() {
    let cat: Vec<Command> = vec![];
    let suggested = strs(&["unknown"]);
    let pinned = strs(&[""]);
    let t = build(&cat, &suggested, &pinned, &[], false, usize::MAX);
    assert_eq!(t.shown.len(), 0);
    assert_eq!(t.rest.len(), 0);
    assert!(!is_shown(&t, "unknown"));
    assert_eq!(reason_of(&t, "unknown"), None);
}

// ---------- Worked example 1 ----------
#[test]
fn example_1_row_1() {
    let cat = catalogue(&[
        "select", "move", "box", "cylinder", "fillet", "measure", "export",
    ]);
    let pinned = strs(&["fillet", "nope", "box"]);
    let suggested = strs(&["box", "drawer_wizard", "measure", "fillet"]);
    let always = strs(&["select", "slice"]);
    let t = build(&cat, &suggested, &pinned, &always, false, 5);
    let expected = make_toolbox(
        &[
            ("fillet", Reason::Pinned),
            ("box", Reason::Pinned),
            ("measure", Reason::Suggested),
            ("select", Reason::Always),
        ],
        &["move", "cylinder", "export"],
        false,
    );
    assert_eq!(t, expected);
}

// ---------- Worked example 2 ----------
#[test]
fn example_2_row_2() {
    let cat = catalogue(&[
        "select", "move", "box", "cylinder", "fillet", "measure", "export",
    ]);
    let pinned = strs(&["fillet", "nope", "box"]);
    let suggested = strs(&["box", "drawer_wizard", "measure", "fillet"]);
    let always = strs(&["select", "slice"]);
    let t = build(&cat, &suggested, &pinned, &always, false, 2);
    let expected = make_toolbox(
        &[("fillet", Reason::Pinned), ("box", Reason::Pinned)],
        &["select", "move", "cylinder", "measure", "export"],
        false,
    );
    assert_eq!(t, expected);
}

// ---------- Worked example 3 ----------
#[test]
fn example_3_row_3() {
    let cat = catalogue(&[
        "select", "move", "box", "cylinder", "fillet", "measure", "export",
    ]);
    let pinned = strs(&["fillet", "nope", "box"]);
    let suggested = strs(&["box", "drawer_wizard", "measure", "fillet"]);
    let always = strs(&["select", "slice"]);
    let t = build(&cat, &suggested, &pinned, &always, false, 0);
    let expected = make_toolbox(
        &[
            ("fillet", Reason::Pinned),
            ("box", Reason::Pinned),
            ("measure", Reason::Suggested),
            ("select", Reason::Always),
        ],
        &["move", "cylinder", "export"],
        false,
    );
    assert_eq!(t, expected);
}

// ---------- Worked example 4 ----------
#[test]
fn example_4_row_4() {
    let cat = catalogue(&[
        "select", "move", "box", "cylinder", "fillet", "measure", "export",
    ]);
    let suggested = strs(&["measure", "box", "measure"]);
    let t = build(&cat, &suggested, &[], &[], false, 0);
    let expected = make_toolbox(
        &[("measure", Reason::Suggested), ("box", Reason::Suggested)],
        &["select", "move", "cylinder", "fillet", "export"],
        false,
    );
    assert_eq!(t, expected);
}

// ---------- Worked example 5 ----------
#[test]
fn example_5_row_5() {
    let cat = catalogue(&[
        "select", "move", "box", "cylinder", "fillet", "measure", "export",
    ]);
    let pinned = strs(&["box", "box"]);
    let suggested = strs(&["box"]);
    let always = strs(&["box"]);
    let t = build(&cat, &suggested, &pinned, &always, false, 0);
    let expected = make_toolbox(
        &[("box", Reason::Pinned)],
        &["select", "move", "cylinder", "fillet", "measure", "export"],
        false,
    );
    assert_eq!(t, expected);
}

// ---------- Worked example 6 ----------
#[test]
fn example_6_row_6() {
    let cat = catalogue(&[
        "select", "move", "box", "cylinder", "fillet", "measure", "export",
    ]);
    let pinned = strs(&["fillet", "nope", "box"]);
    let suggested = strs(&["box", "drawer_wizard", "measure", "fillet"]);
    let always = strs(&["select", "slice"]);
    let t = build(&cat, &suggested, &pinned, &always, true, 0);
    let expected = make_toolbox(
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
    assert_eq!(t, expected);
}

// ---------- Worked example 7 ----------
#[test]
fn example_7_row_7() {
    let cat = catalogue(&[
        "select", "move", "box", "cylinder", "fillet", "measure", "export",
    ]);
    let pinned = strs(&["fillet", "nope", "box"]);
    let suggested = strs(&["box", "drawer_wizard", "measure", "fillet"]);
    let always = strs(&["select", "slice"]);
    let t = build(&cat, &suggested, &pinned, &always, true, 3);
    let expected = make_toolbox(
        &[
            ("fillet", Reason::Pinned),
            ("box", Reason::Pinned),
            ("select", Reason::IntentMissing),
        ],
        &["move", "cylinder", "measure", "export"],
        true,
    );
    assert_eq!(t, expected);
}

// ---------- Worked example 8 ----------
#[test]
fn example_8_row_8() {
    let cat = catalogue(&[
        "select", "move", "box", "cylinder", "fillet", "measure", "export",
    ]);
    let suggested = strs(&["box"]);
    let always = strs(&["measure"]);
    let t = build(&cat, &suggested, &[], &always, true, 0);
    let expected = make_toolbox(
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
    assert_eq!(t, expected);
}

// ---------- Worked example 9 ----------
#[test]
fn example_9_row_9() {
    let cat = catalogue(&[
        "select", "move", "box", "cylinder", "fillet", "measure", "export",
    ]);
    let pinned = strs(&["ghost"]);
    let suggested = strs(&["phantom"]);
    let always = strs(&["slice"]);
    let t = build(&cat, &suggested, &pinned, &always, false, 0);
    let expected = make_toolbox(
        &[],
        &[
            "select", "move", "box", "cylinder", "fillet", "measure", "export",
        ],
        false,
    );
    assert_eq!(t, expected);
}

// ---------- Worked example 10 ----------
#[test]
fn example_10_row_10() {
    let cat: Vec<Command> = vec![];
    let pinned = strs(&["box"]);
    let suggested = strs(&["box"]);
    let always = strs(&["box"]);
    let t = build(&cat, &suggested, &pinned, &always, false, 0);
    let expected = make_toolbox(&[], &[], false);
    assert_eq!(t, expected);
}

// ---------- Worked example 11 ----------
#[test]
fn example_11_row_11() {
    let cat = catalogue(&[
        "select", "move", "box", "cylinder", "fillet", "measure", "export",
    ]);
    let pinned = strs(&["measure"]);
    let suggested = strs(&["box"]);
    let always = strs(&["select"]);
    let t = build(&cat, &suggested, &pinned, &always, false, 10);
    let expected = make_toolbox(
        &[
            ("measure", Reason::Pinned),
            ("box", Reason::Suggested),
            ("select", Reason::Always),
        ],
        &["move", "cylinder", "fillet", "export"],
        false,
    );
    assert_eq!(t, expected);
}

// ---------- Worked example 12 ----------
#[test]
fn example_12_row_12() {
    let cat: Vec<Command> = vec![];
    let pinned = strs(&["box"]);
    let suggested = strs(&["box"]);
    let always = strs(&["box"]);
    let t = build(&cat, &suggested, &pinned, &always, true, 0);
    let expected = make_toolbox(&[], &[], true);
    assert_eq!(t, expected);
}

// ---------- Worked example 13 ----------
#[test]
fn example_13_row_13() {
    let cat = catalogue(&[
        "select", "move", "box", "cylinder", "fillet", "measure", "export",
    ]);
    let pinned = strs(&["fillet"]);
    let t = build(&cat, &[], &pinned, &[], false, 1);
    let expected = make_toolbox(
        &[("fillet", Reason::Pinned)],
        &["select", "move", "box", "cylinder", "measure", "export"],
        false,
    );
    assert_eq!(t, expected);
}

// ---------- Lookup tests from row 1 ----------
#[test]
fn lookup_row_1_examples() {
    let cat = catalogue(&[
        "select", "move", "box", "cylinder", "fillet", "measure", "export",
    ]);
    let pinned = strs(&["fillet", "nope", "box"]);
    let suggested = strs(&["box", "drawer_wizard", "measure", "fillet"]);
    let always = strs(&["select", "slice"]);
    let t = build(&cat, &suggested, &pinned, &always, false, 5);
    assert!(is_shown(&t, "fillet"));
    assert_eq!(reason_of(&t, "fillet"), Some(Reason::Pinned));
    assert!(is_shown(&t, "select"));
    assert_eq!(reason_of(&t, "select"), Some(Reason::Always));
    assert!(!is_shown(&t, "move"));
    assert_eq!(reason_of(&t, "move"), None);
}
