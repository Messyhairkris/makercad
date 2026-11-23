//! Inventory drift between a project's snapshot and the workshop library
//!
//! Written by Hoss (the Crew, campaign mk-core-28, 2026-11-22) from the contract
//! crew/contracts/mk_drift.md (ATLAS-api-harness), which Gordon wrote himself as a PA (the contract, the reference and the hidden tests).
//! (doc 05 section 5.7, doc 03 section 3.6).

use std::collections::{BTreeMap, HashMap};

/// One tool or material line. `key` is the app's id ("tool:jigsaw-1",
/// "mat:pine-2x4-36"); `label` is what the toast shows; `qty` is how many are on
/// hand; `access` is the app's access word ("owned", "shared", ...), compared as
/// text because tool lines and material lines spell it differently.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Entry {
    pub key: String,
    pub label: String,
    pub qty: u32,
    pub access: String,
}

/// One quantity change: `from` is the snapshot's qty, `to` the current library's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QtyChange {
    pub key: String,
    pub from: u32,
    pub to: u32,
}

/// One access change: `from` is the snapshot's access, `to` the current library's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessChange {
    pub key: String,
    pub from: String,
    pub to: String,
}

/// Everything the check found, each list sorted by key (A6).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Drift {
    /// Keys only in the current library.
    pub added: Vec<String>,
    /// Keys only in the snapshot.
    pub removed: Vec<String>,
    /// Keys in both whose qty differs.
    pub qty_changed: Vec<QtyChange>,
    /// Keys in both whose access differs.
    pub access_changed: Vec<AccessChange>,
}

impl Drift {
    /// True when all four lists are empty (S1).
    pub fn is_empty(&self) -> bool {
        self.added.is_empty()
            && self.removed.is_empty()
            && self.qty_changed.is_empty()
            && self.access_changed.is_empty()
    }

    /// The number of findings: the four list lengths added up (S2).
    pub fn count(&self) -> usize {
        self.added.len() + self.removed.len() + self.qty_changed.len() + self.access_changed.len()
    }
}

/// Helper: build a map from key to the first occurrence of an entry,
/// ignoring empty keys (A1, A2).
fn build_map(entries: &[Entry]) -> HashMap<String, &Entry> {
    let mut map = HashMap::new();
    for e in entries {
        if e.key.is_empty() {
            continue;
        }
        map.entry(e.key.clone()).or_insert(e);
    }
    map
}

/// Helper: check if a key counts given the optional relevance list (A4).
fn key_counts(key: &str, relevant: Option<&[String]>) -> bool {
    if key.is_empty() {
        return false;
    }
    match relevant {
        Some(keys) => keys.iter().any(|k| k == key),
        None => true,
    }
}

/// What differs between the snapshot and the current library (D1-D6).
pub fn diff(snapshot: &[Entry], current: &[Entry], relevant: Option<&[String]>) -> Drift {
    let snap_map = build_map(snapshot);
    let cur_map = build_map(current);

    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut qty_changed = Vec::new();
    let mut access_changed = Vec::new();

    // Added and changes
    for (key, cur_entry) in &cur_map {
        if !key_counts(key, relevant) {
            continue;
        }
        match snap_map.get(key) {
            None => added.push(key.clone()),
            Some(snap_entry) => {
                if snap_entry.qty != cur_entry.qty {
                    qty_changed.push(QtyChange {
                        key: key.clone(),
                        from: snap_entry.qty,
                        to: cur_entry.qty,
                    });
                }
                if snap_entry.access != cur_entry.access {
                    access_changed.push(AccessChange {
                        key: key.clone(),
                        from: snap_entry.access.clone(),
                        to: cur_entry.access.clone(),
                    });
                }
            }
        }
    }

    // Removed
    for key in snap_map.keys() {
        if !key_counts(key, relevant) {
            continue;
        }
        if !cur_map.contains_key(key) {
            removed.push(key.clone());
        }
    }

    // Sort according to A6
    added.sort();
    removed.sort();
    qty_changed.sort_by(|a, b| a.key.cmp(&b.key));
    access_changed.sort_by(|a, b| a.key.cmp(&b.key));

    Drift {
        added,
        removed,
        qty_changed,
        access_changed,
    }
}

/// The toast line for a drift (S3).
pub fn summary(drift: &Drift) -> String {
    if drift.is_empty() {
        "nothing changed".to_string()
    } else {
        let n = drift.added.len();
        let r = drift.removed.len();
        let c = drift.qty_changed.len() + drift.access_changed.len();
        format!("{} new, {} gone, {} changed", n, r, c)
    }
}

/// The snapshot brought up to date against the current library (B1-B6).
pub fn apply(snapshot: &[Entry], current: &[Entry], relevant: Option<&[String]>) -> Vec<Entry> {
    let snap_map = build_map(snapshot);
    let cur_map = build_map(current);

    // Use BTreeMap to keep keys sorted (A6)
    let mut result: BTreeMap<String, Entry> = BTreeMap::new();

    // First, handle snapshot entries that are irrelevant (B3)
    for (key, snap_entry) in &snap_map {
        if !key_counts(key, relevant) {
            // Irrelevant keys are copied unchanged
            result.insert(key.clone(), (*snap_entry).clone());
        }
    }

    // Then handle counting keys: current wins (B4, B5)
    for (key, cur_entry) in &cur_map {
        if !key_counts(key, relevant) {
            // Non-relevant current rows are ignored (B1/B3)
            continue;
        }
        // Counting key present in current: include it (overwrites any snapshot entry)
        result.insert(key.clone(), (*cur_entry).clone());
    }

    // Keys that are counting but only in snapshot are omitted automatically
    // because we never inserted them (B5).

    // Collect into a Vec preserving sorted order
    result.into_values().collect()
}
