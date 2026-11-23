//! Contract tests for mk_inventory::drift, written by Gordon as a PA (the Crew) from
//! crew/contracts/mk_drift.md. They were the hidden tests of campaign mk-core-28.

//! Hidden tests for `mk_inventory::drift` (contract: crew/contracts/mk_drift.md).
//!
//! One test per rule, then one per row of the contract's worked-examples table.

// Crew-written tests: a helper that builds a whole `Finding` takes one argument per
// field, and worked rows print constants to the contract's digits; neither matters in a
// test.
#![allow(clippy::too_many_arguments, clippy::approx_constant)]

use mk_inventory::drift::{apply, diff, summary, AccessChange, Drift, Entry, QtyChange};

/// An entry from a compact tuple: (key, label, qty, access).
fn e(key: &str, label: &str, qty: u32, access: &str) -> Entry {
    Entry {
        key: key.to_string(),
        label: label.to_string(),
        qty,
        access: access.to_string(),
    }
}

/// A list of owned strings (keys, for `relevant` and expectations).
fn list(items: &[&str]) -> Vec<String> {
    items.iter().map(|k| k.to_string()).collect()
}

fn qty(key: &str, from: u32, to: u32) -> QtyChange {
    QtyChange {
        key: key.to_string(),
        from,
        to,
    }
}

fn access(key: &str, from: &str, to: &str) -> AccessChange {
    AccessChange {
        key: key.to_string(),
        from: from.to_string(),
        to: to.to_string(),
    }
}

/// The contract's snapshot fixture S (deliberately not in key order).
fn snap() -> Vec<Entry> {
    vec![
        e("glue", "Wood glue", 2, "owned"),
        e("ham", "Hammer", 1, "owned"),
        e("saw", "Hand saw", 1, "shared"),
        e("drill", "Drill", 1, "owned"),
    ]
}

/// The contract's current-library fixture C.
fn cur() -> Vec<Entry> {
    vec![
        e("glue", "Wood glue", 3, "owned"),
        e("ham", "Hammer", 1, "owned"),
        e("drill", "Drill", 1, "makerspace"),
        e("clamp", "Clamps", 4, "owned"),
    ]
}

// ---------------------------------------------------------------- A. keys

#[test]
fn rule_a1_empty_key_is_ignored_on_both_sides() {
    let snapshot = vec![
        e("", "Ghost", 1, "owned"),
        e("glue", "Wood glue", 2, "owned"),
    ];
    let current = vec![e("", "Ghost", 9, "wishlist")];
    let drift = diff(&snapshot, &current, None);
    assert!(drift.added.is_empty());
    assert!(drift.removed == list(&["glue"]));
    assert!(drift.qty_changed.is_empty());
    assert!(drift.access_changed.is_empty());
    assert!(apply(&snapshot, &current, None).is_empty());
    assert!(diff(&snapshot, &snapshot, None).is_empty());
}

#[test]
fn rule_a2_duplicate_key_first_occurrence_wins() {
    let snapshot = vec![e("glue", "Wood glue", 2, "owned")];
    let current = vec![
        e("glue", "Wood glue", 3, "owned"),
        e("glue", "Glue again", 9, "wishlist"),
    ];
    let drift = diff(&snapshot, &current, None);
    assert!(drift.qty_changed == vec![qty("glue", 2, 3)]);
    assert!(drift.access_changed.is_empty());
    assert!(drift.count() == 1);
}

#[test]
fn rule_a2_duplicate_key_in_the_snapshot_first_wins_too() {
    let snapshot = vec![
        e("glue", "Wood glue", 2, "owned"),
        e("glue", "Second", 9, "wishlist"),
    ];
    let current = vec![e("glue", "Wood glue", 2, "owned")];
    assert!(diff(&snapshot, &current, None).is_empty());
}

#[test]
fn rule_a3_keys_compare_case_sensitively_and_exactly() {
    let snapshot = vec![e("Glue", "Wood glue", 2, "owned")];
    let current = vec![e("glue", "Wood glue", 2, "owned")];
    let drift = diff(&snapshot, &current, None);
    assert!(drift.added == list(&["glue"]));
    assert!(drift.removed == list(&["Glue"]));
    assert!(drift.qty_changed.is_empty());
}

#[test]
fn rule_a4_none_means_every_counting_key() {
    let drift = diff(&snap(), &cur(), None);
    assert!(drift.added == list(&["clamp"]));
    assert!(drift.removed == list(&["saw"]));
}

#[test]
fn rule_a4_some_limits_the_check_to_the_listed_keys() {
    let drift = diff(&snap(), &cur(), Some(&list(&["glue"])[..]));
    assert!(drift.added.is_empty());
    assert!(drift.removed.is_empty());
    assert!(drift.qty_changed == vec![qty("glue", 2, 3)]);
    assert!(drift.access_changed.is_empty());
}

#[test]
fn rule_a4_empty_relevant_list_counts_no_key() {
    let drift = diff(&snap(), &cur(), Some(&[] as &[String]));
    assert!(drift.is_empty());
    assert!(summary(&drift) == "nothing changed");
    assert!(apply(&snap(), &cur(), Some(&[] as &[String])) == apply(&snap(), &snap(), None));
}

#[test]
fn rule_a5_label_is_never_compared() {
    let snapshot = vec![e("glue", "Wood glue", 2, "owned")];
    let current = vec![e("glue", "White glue", 2, "owned")];
    assert!(diff(&snapshot, &current, None).is_empty());
}

#[test]
fn rule_a6_drift_lists_are_in_key_order() {
    let current = vec![
        e("b", "B", 1, "owned"),
        e("a", "A", 1, "owned"),
        e("aa", "AA", 1, "owned"),
    ];
    let drift = diff(&[], &current, None);
    assert!(drift.added == list(&["a", "aa", "b"]));
}

#[test]
fn rule_a7_qty_and_access_compare_whole_values() {
    let snapshot = vec![e("k", "K", 0, "")];
    let current = vec![e("k", "K", u32::MAX, "owned")];
    let drift = diff(&snapshot, &current, None);
    assert!(drift.qty_changed == vec![qty("k", 0, u32::MAX)]);
    assert!(drift.access_changed == vec![access("k", "", "owned")]);
    assert!(drift.count() == 2);
}

// ------------------------------------------------------------------ D. diff

#[test]
fn rule_d1_added_holds_the_current_only_keys() {
    let drift = diff(&snap(), &cur(), None);
    assert!(drift.added == list(&["clamp"]));
}

#[test]
fn rule_d2_removed_holds_the_snapshot_only_keys() {
    let drift = diff(&snap(), &cur(), None);
    assert!(drift.removed == list(&["saw"]));
    assert!(drift.count() == 4);
}

#[test]
fn rule_d3_qty_change_runs_snapshot_to_current() {
    let drift = diff(&snap(), &cur(), None);
    assert!(drift.qty_changed == vec![qty("glue", 2, 3)]);
    assert!(drift.qty_changed.len() == 1);
}

#[test]
fn rule_d4_access_change_runs_snapshot_to_current() {
    let drift = diff(&snap(), &cur(), None);
    assert!(drift.access_changed == vec![access("drill", "owned", "makerspace")]);
    assert!(drift.access_changed.len() == 1);
}

#[test]
fn rule_d5_the_same_key_may_change_qty_and_access() {
    let snapshot = vec![e("drill", "Drill", 1, "owned")];
    let current = vec![e("drill", "Drill", 2, "makerspace")];
    let drift = diff(&snapshot, &current, None);
    assert!(drift.qty_changed == vec![qty("drill", 1, 2)]);
    assert!(drift.access_changed == vec![access("drill", "owned", "makerspace")]);
    assert!(drift.added.is_empty());
    assert!(drift.removed.is_empty());
    assert!(drift.count() == 2);
}

#[test]
fn rule_d6_no_other_difference_is_reported() {
    let snapshot = vec![
        e("k", "Old label", 1, "owned"),
        e("gone", "Gone", 1, "owned"),
    ];
    let current = vec![e("k", "New label", 1, "owned")];
    let drift = diff(&snapshot, &current, Some(&list(&["k"])[..]));
    assert!(drift.is_empty());
}

// --------------------------------------------------------------- S. helpers

#[test]
fn rule_s1_is_empty_tracks_all_four_lists() {
    assert!(Drift::default().is_empty());
    assert!(diff(&snap(), &snap(), None).is_empty());
    let one = Drift {
        added: list(&["a"]),
        ..Drift::default()
    };
    assert!(!one.is_empty());
    let one = Drift {
        removed: list(&["a"]),
        ..Drift::default()
    };
    assert!(!one.is_empty());
    let one = Drift {
        qty_changed: vec![qty("a", 1, 2)],
        ..Drift::default()
    };
    assert!(!one.is_empty());
    let one = Drift {
        access_changed: vec![access("a", "x", "y")],
        ..Drift::default()
    };
    assert!(!one.is_empty());
}

#[test]
fn rule_s2_count_adds_the_four_list_lengths() {
    assert!(Drift::default().count() == 0);
    assert!(diff(&snap(), &cur(), None).count() == 4);
    let mixed = Drift {
        added: list(&["a", "b"]),
        removed: list(&["c"]),
        qty_changed: vec![qty("d", 1, 2)],
        access_changed: vec![access("d", "x", "y"), access("e", "x", "y")],
    };
    assert!(mixed.count() == 6);
}

#[test]
fn rule_s3_summary_words_and_numbers() {
    assert!(summary(&diff(&snap(), &cur(), None)) == "1 new, 1 gone, 2 changed");
    assert!(summary(&Drift::default()) == "nothing changed");
    let mixed = Drift {
        added: list(&["a", "b"]),
        removed: Vec::new(),
        qty_changed: vec![qty("c", 1, 2)],
        access_changed: Vec::new(),
    };
    assert!(summary(&mixed) == "2 new, 0 gone, 1 changed");
    let only_access = Drift {
        access_changed: vec![access("c", "owned", "shared")],
        ..Drift::default()
    };
    assert!(summary(&only_access) == "0 new, 0 gone, 1 changed");
}

// ----------------------------------------------------------------- B. apply

#[test]
fn rule_b1_current_rows_that_do_not_count_are_not_applied() {
    let result = apply(&snap(), &cur(), Some(&list(&["ham"])[..]));
    assert!(
        result
            == vec![
                e("drill", "Drill", 1, "owned"),
                e("glue", "Wood glue", 2, "owned"),
                e("ham", "Hammer", 1, "owned"),
                e("saw", "Hand saw", 1, "shared"),
            ]
    );
}

#[test]
fn rule_b2_one_entry_per_key_first_wins() {
    let snapshot = vec![e("glue", "Wood glue", 2, "owned")];
    let current = vec![
        e("glue", "Wood glue", 3, "owned"),
        e("glue", "Glue again", 9, "wishlist"),
    ];
    let result = apply(&snapshot, &current, None);
    assert!(result.len() == 1);
    assert!(result == vec![e("glue", "Wood glue", 3, "owned")]);
}

#[test]
fn rule_b3_irrelevant_snapshot_rows_come_through_unchanged() {
    let result = apply(&snap(), &cur(), Some(&list(&["glue"])[..]));
    assert!(
        result
            == vec![
                e("drill", "Drill", 1, "owned"),
                e("glue", "Wood glue", 3, "owned"),
                e("ham", "Hammer", 1, "owned"),
                e("saw", "Hand saw", 1, "shared"),
            ]
    );
}

#[test]
fn rule_b4_the_current_row_wins_for_a_counting_key() {
    let result = apply(&snap(), &cur(), None);
    assert!(
        result
            == vec![
                e("clamp", "Clamps", 4, "owned"),
                e("drill", "Drill", 1, "makerspace"),
                e("glue", "Wood glue", 3, "owned"),
                e("ham", "Hammer", 1, "owned"),
            ]
    );
}

#[test]
fn rule_b5_added_keys_enter_and_gone_keys_leave() {
    let added = apply(&snap(), &cur(), Some(&list(&["clamp"])[..]));
    assert!(added.len() == 5);
    assert!(added.iter().any(|row| row.key == "clamp" && row.qty == 4));
    let gone = apply(&snap(), &cur(), Some(&list(&["saw"])[..]));
    assert!(
        gone == vec![
            e("drill", "Drill", 1, "owned"),
            e("glue", "Wood glue", 2, "owned"),
            e("ham", "Hammer", 1, "owned"),
        ]
    );
}

#[test]
fn rule_b6_apply_result_is_sorted_and_inputs_are_untouched() {
    let snapshot = snap();
    let current = cur();
    let result = apply(&snapshot, &current, None);
    let mut sorted = true;
    for pair in result.windows(2) {
        if pair[0].key >= pair[1].key {
            sorted = false;
        }
    }
    assert!(sorted);
    assert!(snapshot == snap());
    assert!(current == cur());
}

// ------------------------------------------------------------------- N. none

#[test]
fn rule_n1_empty_inputs_do_not_panic() {
    let drift = diff(&[], &[], None);
    assert!(drift.is_empty());
    assert!(drift.count() == 0);
    assert!(summary(&drift) == "nothing changed");
    assert!(apply(&[], &[], None).is_empty());
    assert!(diff(&[], &[], Some(&[] as &[String])).is_empty());
    assert!(apply(&[], &[], Some(&list(&["a", "b"])[..])).is_empty());
}

#[test]
fn rule_n1_odd_strings_do_not_panic() {
    let long = "k".repeat(10_000);
    let snapshot = vec![e(&long, "", 0, ""), e("glühen", "Ü", u32::MAX, "ü")];
    let current = vec![e("glühen", "Ü", 0, ""), e(&long, "x", u32::MAX, "y")];
    let keys = list(&[long.as_str(), "glühen", "glühen"]);
    let drift = diff(&snapshot, &current, Some(keys.as_slice()));
    assert!(drift.count() == 4);
    assert!(apply(&snapshot, &current, None).len() == 2);
}

#[test]
fn rule_n1_single_row_inputs_do_not_panic() {
    let only = vec![e("k", "K", 1, "owned")];
    assert!(diff(&only, &[], None).removed == list(&["k"]));
    assert!(diff(&[], &only, None).added == list(&["k"]));
    assert!(apply(&only, &[], None).is_empty());
    assert!(apply(&[], &only, None) == only);
}

// -------------------------------------------------------- worked examples
// One test per row of the contract's table, in the table's order.

#[test]
fn example_r01_diff_all_keys() {
    let drift = diff(&snap(), &cur(), None);
    assert!(drift.added == list(&["clamp"]));
    assert!(drift.removed == list(&["saw"]));
    assert!(drift.qty_changed == vec![qty("glue", 2, 3)]);
    assert!(drift.access_changed == vec![access("drill", "owned", "makerspace")]);
    assert!(drift.count() == 4);
    assert!(summary(&drift) == "1 new, 1 gone, 2 changed");
}

#[test]
fn example_r02_relevant_glue_only() {
    let drift = diff(&snap(), &cur(), Some(&list(&["glue"])[..]));
    assert!(drift.added.is_empty());
    assert!(drift.removed.is_empty());
    assert!(drift.qty_changed == vec![qty("glue", 2, 3)]);
    assert!(drift.access_changed.is_empty());
    assert!(drift.count() == 1);
}

#[test]
fn example_r03_relevant_saw_and_clamp() {
    let drift = diff(&snap(), &cur(), Some(&list(&["saw", "clamp"])[..]));
    assert!(drift.added == list(&["clamp"]));
    assert!(drift.removed == list(&["saw"]));
    assert!(drift.qty_changed.is_empty());
    assert!(drift.access_changed.is_empty());
    assert!(drift.count() == 2);
}

#[test]
fn example_r04_relevant_ham_only() {
    let drift = diff(&snap(), &cur(), Some(&list(&["ham"])[..]));
    assert!(drift.is_empty());
    assert!(drift.count() == 0);
    assert!(summary(&drift) == "nothing changed");
}

#[test]
fn example_r05_snapshot_against_itself() {
    let drift = diff(&snap(), &snap(), None);
    assert!(drift.is_empty());
    assert!(drift.count() == 0);
    assert!(summary(&drift) == "nothing changed");
}

#[test]
fn example_r06_empty_key_ignored() {
    let snapshot = vec![
        e("", "Ghost", 1, "owned"),
        e("glue", "Wood glue", 2, "owned"),
    ];
    let current = vec![e("", "Ghost", 9, "wishlist")];
    let drift = diff(&snapshot, &current, None);
    assert!(drift.added.is_empty());
    assert!(drift.removed == list(&["glue"]));
    assert!(drift.qty_changed.is_empty());
    assert!(drift.access_changed.is_empty());
    assert!(summary(&drift) == "0 new, 1 gone, 0 changed");
}

#[test]
fn example_r07_duplicate_current_key_first_wins() {
    let snapshot = vec![e("glue", "Wood glue", 2, "owned")];
    let current = vec![
        e("glue", "Wood glue", 3, "owned"),
        e("glue", "Glue again", 9, "wishlist"),
    ];
    let drift = diff(&snapshot, &current, None);
    assert!(drift.qty_changed == vec![qty("glue", 2, 3)]);
    assert!(drift.access_changed.is_empty());
    assert!(drift.count() == 1);
}

#[test]
fn example_r08_added_keys_sorted() {
    let current = vec![
        e("b", "B", 1, "owned"),
        e("a", "A", 1, "owned"),
        e("aa", "AA", 1, "owned"),
    ];
    let drift = diff(&[], &current, None);
    assert!(drift.added == list(&["a", "aa", "b"]));
    assert!(drift.count() == 3);
}

#[test]
fn example_r09_label_change_only() {
    let snapshot = vec![e("glue", "Wood glue", 2, "owned")];
    let current = vec![e("glue", "White glue", 2, "owned")];
    assert!(diff(&snapshot, &current, None).is_empty());
}

#[test]
fn example_r10_apply_all_keys() {
    let result = apply(&snap(), &cur(), None);
    assert!(
        result
            == vec![
                e("clamp", "Clamps", 4, "owned"),
                e("drill", "Drill", 1, "makerspace"),
                e("glue", "Wood glue", 3, "owned"),
                e("ham", "Hammer", 1, "owned"),
            ]
    );
}

#[test]
fn example_r11_apply_relevant_glue() {
    let result = apply(&snap(), &cur(), Some(&list(&["glue"])[..]));
    assert!(
        result
            == vec![
                e("drill", "Drill", 1, "owned"),
                e("glue", "Wood glue", 3, "owned"),
                e("ham", "Hammer", 1, "owned"),
                e("saw", "Hand saw", 1, "shared"),
            ]
    );
}

#[test]
fn example_r12_apply_relevant_saw() {
    let result = apply(&snap(), &cur(), Some(&list(&["saw"])[..]));
    assert!(
        result
            == vec![
                e("drill", "Drill", 1, "owned"),
                e("glue", "Wood glue", 2, "owned"),
                e("ham", "Hammer", 1, "owned"),
            ]
    );
}

#[test]
fn example_r13_apply_relevant_empty() {
    let result = apply(&snap(), &cur(), Some(&[] as &[String]));
    assert!(
        result
            == vec![
                e("drill", "Drill", 1, "owned"),
                e("glue", "Wood glue", 2, "owned"),
                e("ham", "Hammer", 1, "owned"),
                e("saw", "Hand saw", 1, "shared"),
            ]
    );
}

#[test]
fn example_r14_apply_drops_the_empty_key() {
    let snapshot = vec![e("", "Ghost", 1, "owned")];
    let current = vec![e("", "Ghost", 2, "owned")];
    assert!(apply(&snapshot, &current, None).is_empty());
}

#[test]
fn example_r15_a_key_in_both_change_lists() {
    let snapshot = vec![e("drill", "Drill", 1, "owned")];
    let current = vec![e("drill", "Drill", 2, "makerspace")];
    let drift = diff(&snapshot, &current, None);
    assert!(drift.qty_changed == vec![qty("drill", 1, 2)]);
    assert!(drift.access_changed == vec![access("drill", "owned", "makerspace")]);
    assert!(drift.count() == 2);
    assert!(summary(&drift) == "0 new, 0 gone, 2 changed");
}

#[test]
fn example_r16_apply_duplicate_snapshot_key_first_wins() {
    let snapshot = vec![
        e("glue", "Wood glue", 2, "owned"),
        e("glue", "Second", 9, "wishlist"),
    ];
    let current = vec![e("glue", "Wood glue", 3, "owned")];
    assert!(apply(&snapshot, &current, None) == vec![e("glue", "Wood glue", 3, "owned")]);
}

#[test]
fn example_r17_apply_takes_the_current_label() {
    let snapshot = vec![e("glue", "Wood glue", 2, "owned")];
    let current = vec![e("glue", "White glue", 3, "owned")];
    assert!(apply(&snapshot, &current, None) == vec![e("glue", "White glue", 3, "owned")]);
}
