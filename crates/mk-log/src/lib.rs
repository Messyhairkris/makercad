//! The project's log (doc 03's crate list: journal records, prompt schedule, session interviews).
//!
//! `journal`: the project journal (log/journal.jsonl): append-only records, edits as records
//! that supersede old ones, open pages answered later, and the views the app shows.
//!
//! `quest`: the new-project questions as data (doc 05 section 5.2): the steps, answers checked
//! by kind, skipping that moves the flow on, and the facts the app reads from the answers.

pub mod journal;
pub mod quest;
