//! Tests for mk_inventory::drift written by Deepak (the Crew, campaign mk-core-28) from the
//! contract alone, without seeing the module.

//! Integration tests for `mk_inventory::drift` (contract v1).

// Crew-written tests: a helper that builds a whole `Finding` takes one argument per
// field, and worked rows print constants to the contract's digits; neither matters in a
// test.
#![allow(clippy::too_many_arguments, clippy::approx_constant)]

use mk_inventory::drift::{apply, diff, summary, AccessChange, Drift, Entry, QtyChange};

fn e(key: &str, label: &str, qty: u32, access: &str) -> Entry {
    Entry {
        key: key.to_string(),
        label: label.to_string(),
        qty,
        access: access.to_string(),
    }
}

fn keys(list: &[&str]) -> Vec<String> {
    list.iter().map(|k| k.to_string()).collect()
}

fn qc(key: &str, from: u32, to: u32) -> QtyChange {
    QtyChange {
        key: key.to_string(),
        from,
        to,
    }
}

fn ac(key: &str, from: &str, to: &str) -> AccessChange {
    AccessChange {
        key: key.to_string(),
        from: from.to_string(),
        to: to.to_string(),
    }
}

fn empty_drift() -> Drift {
    Drift {
        added: vec![],
        removed: vec![],
        qty_changed: vec![],
        access_changed: vec![],
    }
}

/// Fixture S of the contract's worked examples.
fn snap() -> Vec<Entry> {
    vec![
        e("glue", "Wood glue", 2, "owned"),
        e("ham", "Hammer", 1, "owned"),
        e("saw", "Hand saw", 1, "shared"),
        e("drill", "Drill", 1, "owned"),
    ]
}

/// Fixture C of the contract's worked examples.
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
fn rule_a1_empty_key_ignored_on_both_sides() {
    let snapshot = vec![
        e("", "Ghost", 1, "owned"),
        e("glue", "Wood glue", 2, "owned"),
    ];
    let current = vec![
        e("", "Ghost", 9, "wishlist"),
        e("glue", "Wood glue", 2, "owned"),
    ];
    assert_eq!(diff(&snapshot, &current, None), empty_drift());
    assert_eq!(
        apply(&snapshot, &current, None),
        vec![e("glue", "Wood glue", 2, "owned")]
    );
}

#[test]
fn rule_a2_duplicate_key_counts_once_as_first_occurrence() {
    // first occurrence in `current` wins: qty 3 / "owned", not 9 / "wishlist"
    let snapshot = vec![e("glue", "Wood glue", 2, "owned")];
    let current = vec![
        e("glue", "Wood glue", 3, "owned"),
        e("glue", "Glue again", 9, "wishlist"),
    ];
    let want = Drift {
        added: vec![],
        removed: vec![],
        qty_changed: vec![qc("glue", 2, 3)],
        access_changed: vec![],
    };
    assert_eq!(diff(&snapshot, &current, None), want);
    assert_eq!(
        apply(&snapshot, &current, None),
        vec![e("glue", "Wood glue", 3, "owned")]
    );

    // first occurrence in `snapshot` wins too: qty 5 / "owned", not 1 / "wishlist"
    let snapshot = vec![e("glue", "A", 5, "owned"), e("glue", "B", 1, "wishlist")];
    let current = vec![e("glue", "C", 5, "owned")];
    assert_eq!(diff(&snapshot, &current, None), empty_drift());
    assert_eq!(
        apply(&snapshot, &current, None),
        vec![e("glue", "C", 5, "owned")]
    );
}

#[test]
fn rule_a3_keys_compare_case_sensitively_and_untrimmed() {
    let snapshot = vec![e("Glue", "Glue", 1, "owned")];
    let current = vec![e("glue", "Glue", 1, "owned")];
    let want = Drift {
        added: keys(&["glue"]),
        removed: keys(&["Glue"]),
        qty_changed: vec![],
        access_changed: vec![],
    };
    assert_eq!(diff(&snapshot, &current, None), want);

    let d = diff(
        &[e(" glue", "G", 1, "owned")],
        &[e("glue", "G", 1, "owned")],
        None,
    );
    assert_eq!(d.added, keys(&["glue"]));
    assert_eq!(d.removed, keys(&[" glue"]));
}

#[test]
fn rule_a4_relevant_filters_both_sides() {
    let rel = keys(&["glue"]);
    let want = Drift {
        added: vec![],
        removed: vec![],
        qty_changed: vec![qc("glue", 2, 3)],
        access_changed: vec![],
    };
    assert_eq!(diff(&snap(), &cur(), Some(&rel)), want);

    // Some(&[]) means no key counts at all
    let none: Vec<String> = vec![];
    assert_eq!(diff(&snap(), &cur(), Some(&none)), empty_drift());
    assert_eq!(
        apply(&snap(), &cur(), Some(&none)),
        vec![
            e("drill", "Drill", 1, "owned"),
            e("glue", "Wood glue", 2, "owned"),
            e("ham", "Hammer", 1, "owned"),
            e("saw", "Hand saw", 1, "shared"),
        ]
    );

    // repeats inside `keys` change nothing
    let repeated = keys(&["glue", "glue", "glue"]);
    assert_eq!(
        diff(&snap(), &cur(), Some(&repeated)),
        diff(&snap(), &cur(), Some(&rel))
    );
}

#[test]
fn rule_a5_label_is_never_compared() {
    let snapshot = vec![e("glue", "Wood glue", 2, "owned")];
    let current = vec![e("glue", "White glue", 2, "owned")];
    assert_eq!(diff(&snapshot, &current, None), empty_drift());
    // apply still takes the current label (B4)
    assert_eq!(
        apply(&snapshot, &current, None),
        vec![e("glue", "White glue", 2, "owned")]
    );
}

#[test]
fn rule_a6_every_list_is_sorted_by_key() {
    let snapshot = vec![e("b", "B", 1, "owned"), e("a", "A", 1, "owned")];
    let current = vec![e("d", "D", 1, "owned"), e("c", "C", 1, "owned")];
    let d = diff(&snapshot, &current, None);
    assert_eq!(d.added, keys(&["c", "d"]));
    assert_eq!(d.removed, keys(&["a", "b"]));
    assert_eq!(
        apply(&snapshot, &current, None),
        vec![e("c", "C", 1, "owned"), e("d", "D", 1, "owned")]
    );

    // qty_changed is sorted too, not in input order
    let snapshot = vec![e("z", "Z", 1, "owned"), e("a", "A", 1, "owned")];
    let current = vec![e("z", "Z", 2, "owned"), e("a", "A", 2, "owned")];
    assert_eq!(
        diff(&snapshot, &current, None).qty_changed,
        vec![qc("a", 1, 2), qc("z", 1, 2)]
    );
}

#[test]
fn rule_a7_qty_and_access_compared_as_whole_values() {
    let snapshot = vec![e("a", "A", 0, ""), e("b", "B", u32::MAX, "owned")];
    let current = vec![e("a", "A", 0, ""), e("b", "B", 0, "owned")];
    let d = diff(&snapshot, &current, None);
    assert_eq!(d.qty_changed, vec![qc("b", u32::MAX, 0)]);
    assert_eq!(d.access_changed, vec![]);

    // empty access vs "owned" is a whole-string change
    let d2 = diff(&[e("k", "K", 1, "")], &[e("k", "K", 1, "owned")], None);
    assert_eq!(d2.access_changed, vec![ac("k", "", "owned")]);
}

// ---------------------------------------------------------------- D. diff

#[test]
fn rule_d1_added_holds_keys_only_in_current() {
    let snapshot = vec![e("glue", "Wood glue", 2, "owned")];
    let current = vec![
        e("glue", "Wood glue", 2, "owned"),
        e("clamp", "Clamps", 4, "owned"),
    ];
    let want = Drift {
        added: keys(&["clamp"]),
        removed: vec![],
        qty_changed: vec![],
        access_changed: vec![],
    };
    assert_eq!(diff(&snapshot, &current, None), want);
}

#[test]
fn rule_d2_removed_holds_keys_only_in_snapshot() {
    let snapshot = vec![
        e("glue", "Wood glue", 2, "owned"),
        e("saw", "Hand saw", 1, "shared"),
    ];
    let current = vec![e("glue", "Wood glue", 2, "owned")];
    let want = Drift {
        added: vec![],
        removed: keys(&["saw"]),
        qty_changed: vec![],
        access_changed: vec![],
    };
    assert_eq!(diff(&snapshot, &current, None), want);
}

#[test]
fn rule_d3_qty_changed_from_snapshot_to_current() {
    let snapshot = vec![e("glue", "Wood glue", 2, "owned")];
    let current = vec![e("glue", "Wood glue", 3, "owned")];
    let want = Drift {
        added: vec![],
        removed: vec![],
        qty_changed: vec![qc("glue", 2, 3)],
        access_changed: vec![],
    };
    assert_eq!(diff(&snapshot, &current, None), want);
}

#[test]
fn rule_d4_access_changed_from_snapshot_to_current() {
    let snapshot = vec![e("drill", "Drill", 1, "owned")];
    let current = vec![e("drill", "Drill", 1, "makerspace")];
    let want = Drift {
        added: vec![],
        removed: vec![],
        qty_changed: vec![],
        access_changed: vec![ac("drill", "owned", "makerspace")],
    };
    assert_eq!(diff(&snapshot, &current, None), want);
}

#[test]
fn rule_d5_added_and_removed_disjoint_both_changes_independent() {
    let d = diff(
        &[e("drill", "Drill", 1, "owned")],
        &[e("drill", "Drill", 2, "makerspace")],
        None,
    );
    assert_eq!(d.qty_changed, vec![qc("drill", 1, 2)]);
    assert_eq!(d.access_changed, vec![ac("drill", "owned", "makerspace")]);
    assert_eq!(d.count(), 2);

    let d2 = diff(&snap(), &cur(), None);
    for k in &d2.added {
        assert!(!d2.removed.contains(k));
    }
}

#[test]
fn rule_d6_nothing_else_is_reported() {
    // empty inputs are not an error
    assert_eq!(diff(&[], &[], None), empty_drift());
    // a label-only change is not a finding
    let d = diff(
        &[e("glue", "Wood glue", 2, "owned")],
        &[e("glue", "White glue", 2, "owned")],
        None,
    );
    assert_eq!(d, empty_drift());
    // no duplicates and no empty keys leak into the lists
    let snapshot = vec![
        e("", "Ghost", 1, "owned"),
        e("a", "A", 1, "owned"),
        e("a", "A2", 1, "owned"),
    ];
    let current = vec![
        e("", "Ghost", 5, "owned"),
        e("b", "B", 1, "owned"),
        e("b", "B2", 1, "owned"),
    ];
    let d2 = diff(&snapshot, &current, None);
    assert_eq!(d2.added, keys(&["b"]));
    assert_eq!(d2.removed, keys(&["a"]));
}

// ---------------------------------------------------------------- S. helpers

#[test]
fn rule_s1_is_empty_true_only_when_all_four_lists_empty() {
    assert!(empty_drift().is_empty());
    let only_added = Drift {
        added: keys(&["a"]),
        removed: vec![],
        qty_changed: vec![],
        access_changed: vec![],
    };
    assert!(!only_added.is_empty());
    let only_access = Drift {
        added: vec![],
        removed: vec![],
        qty_changed: vec![],
        access_changed: vec![ac("k", "x", "y")],
    };
    assert!(!only_access.is_empty());
    assert!(diff(&snap(), &snap(), None).is_empty());
}

#[test]
fn rule_s2_count_sums_the_four_list_lengths() {
    let d = Drift {
        added: keys(&["a", "b"]),
        removed: keys(&["c"]),
        qty_changed: vec![qc("d", 1, 2)],
        access_changed: vec![ac("e", "x", "y")],
    };
    assert_eq!(d.count(), 5);
    // a key in both change lists counts twice (D5)
    let both = diff(
        &[e("drill", "Drill", 1, "owned")],
        &[e("drill", "Drill", 2, "makerspace")],
        None,
    );
    assert_eq!(both.count(), 2);
    assert_eq!(empty_drift().count(), 0);
}

#[test]
fn rule_s3_summary_wording() {
    assert_eq!(summary(&empty_drift()), "nothing changed");

    let one = Drift {
        added: vec![],
        removed: vec![],
        qty_changed: vec![qc("c", 1, 2)],
        access_changed: vec![],
    };
    assert_eq!(summary(&one), "0 new, 0 gone, 1 changed");

    let mixed = Drift {
        added: keys(&["a"]),
        removed: keys(&["b"]),
        qty_changed: vec![qc("c", 1, 2)],
        access_changed: vec![ac("d", "x", "y")],
    };
    assert_eq!(summary(&mixed), "1 new, 1 gone, 2 changed");

    let added_only = Drift {
        added: keys(&["a"]),
        removed: vec![],
        qty_changed: vec![],
        access_changed: vec![],
    };
    assert_eq!(summary(&added_only), "1 new, 0 gone, 0 changed");
}

// ---------------------------------------------------------------- B. apply

#[test]
fn rule_b1_irrelevant_current_row_never_applied() {
    let snapshot = vec![
        e("glue", "Wood glue", 2, "owned"),
        e("saw", "Hand saw", 1, "shared"),
    ];
    let current = vec![
        e("glue", "Wood glue", 3, "owned"),
        e("clamp", "Clamps", 4, "owned"),
    ];
    let rel = keys(&["glue"]);
    assert_eq!(
        apply(&snapshot, &current, Some(&rel)),
        vec![
            e("glue", "Wood glue", 3, "owned"),
            e("saw", "Hand saw", 1, "shared")
        ]
    );
}

#[test]
fn rule_b2_one_entry_per_key_merged_not_concatenated() {
    let snapshot = vec![e("a", "A", 1, "owned"), e("a", "A2", 5, "wishlist")];
    let current = vec![e("a", "A3", 2, "owned"), e("a", "A4", 7, "wishlist")];
    assert_eq!(
        apply(&snapshot, &current, None),
        vec![e("a", "A3", 2, "owned")]
    );
}

#[test]
fn rule_b3_irrelevant_snapshot_rows_kept_unchanged() {
    let snapshot = vec![
        e("glue", "Wood glue", 2, "owned"),
        e("saw", "Hand saw", 1, "shared"),
    ];
    let current = vec![
        e("glue", "Wood glue", 3, "owned"),
        e("saw", "Hand saw", 9, "wishlist"),
    ];
    let rel = keys(&["glue"]);
    // B3 runs before B4/B5: "saw" keeps every snapshot field, the current row is ignored
    assert_eq!(
        apply(&snapshot, &current, Some(&rel)),
        vec![
            e("glue", "Wood glue", 3, "owned"),
            e("saw", "Hand saw", 1, "shared")
        ]
    );
}

#[test]
fn rule_b4_counting_key_in_both_takes_the_current_row() {
    let snapshot = vec![e("glue", "Wood glue", 2, "owned")];
    let current = vec![e("glue", "White glue", 3, "shared")];
    assert_eq!(
        apply(&snapshot, &current, None),
        vec![e("glue", "White glue", 3, "shared")]
    );
}

#[test]
fn rule_b5_current_only_added_snapshot_only_dropped() {
    let snapshot = vec![e("saw", "Hand saw", 1, "shared")];
    let current = vec![e("clamp", "Clamps", 4, "owned")];
    assert_eq!(
        apply(&snapshot, &current, None),
        vec![e("clamp", "Clamps", 4, "owned")]
    );
}

#[test]
fn rule_b6_result_sorted_and_inputs_untouched() {
    let snapshot = vec![e("b", "B", 1, "owned"), e("a", "A", 1, "owned")];
    let current = vec![e("d", "D", 1, "owned"), e("c", "C", 1, "owned")];
    let out = apply(&snapshot, &current, None);
    assert_eq!(out, vec![e("c", "C", 1, "owned"), e("d", "D", 1, "owned")]);
    assert_eq!(
        snapshot,
        vec![e("b", "B", 1, "owned"), e("a", "A", 1, "owned")]
    );
    assert_eq!(
        current,
        vec![e("d", "D", 1, "owned"), e("c", "C", 1, "owned")]
    );
}

// ---------------------------------------------------------------- N. never panics

#[test]
fn rule_n1_no_input_can_panic() {
    let empty: Vec<Entry> = vec![];
    let none: Vec<String> = vec![];
    assert!(diff(&empty, &empty, None).is_empty());
    assert!(diff(&empty, &empty, Some(&none)).is_empty());
    assert_eq!(apply(&empty, &empty, None), Vec::<Entry>::new());
    assert_eq!(apply(&empty, &empty, Some(&none)), Vec::<Entry>::new());

    // a key neither list has, a 10 000-character key, qty 0 and u32::MAX, empty label/access
    let long = "tool:".repeat(2500);
    let snapshot = vec![e(&long, "", 0, ""), e("k", "", u32::MAX, "")];
    let current = vec![e(&long, "", u32::MAX, ""), e("k", "", 0, "")];
    let d = diff(&snapshot, &current, None);
    assert_eq!(d.count(), 2);
    assert_eq!(apply(&snapshot, &current, None).len(), 2);

    // a key neither list has is not a finding on either side
    let d2 = diff(&[e("a", "A", 1, "owned")], &[e("a", "A", 1, "owned")], None);
    assert!(d2.is_empty());
}
