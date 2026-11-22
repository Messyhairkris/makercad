//! The project's log (doc 03's crate list: journal records, prompt schedule, session interviews).
//!
//! `journal`: the project journal (log/journal.jsonl): append-only records, edits as records
//! that supersede old ones, open pages answered later, and the views the app shows.
//!
//! `quest`: the new-project questions as data (doc 05 section 5.2): the steps, answers checked
//! by kind, skipping that moves the flow on, and the facts the app reads from the answers.
//!
//! `interview`: the session-end interview (doc 07 section 7.7): when it is offered, the six
//! questions, the draft of what was done from the command log, cause chips, skippable answers,
//! and the journal bodies it becomes (a session record, and an open page when one was skipped).
//!
//! `prompts`: the journal's seven prompt moments (doc 10 section 10.4): which events make each
//! one due, skipping one into an open page, answering it (due or open), and the lists the app
//! shows.

pub mod interview;
pub mod journal;
pub mod prompts;
pub mod quest;
