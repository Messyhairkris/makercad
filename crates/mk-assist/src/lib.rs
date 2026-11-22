//! Sancho's rules (doc 03's crate list: rules, persona, mind adapter).
//!
//! `interpret`: the project interpreter v1 (doc 05 section 5.4): the six-rule table read from the
//! maker's words and attachments, every environment, tool, template and question with its why,
//! and a conclusion under 60% shown as a question. The persona and the optional AI pass come later.
//!
//! `findings`: what the Rules layer gives (doc 07 sections 7.2, 7.4, 7.9): a finding's kind,
//! severity, source, location and material facts, and the ledger that decides whether Sancho
//! says it, keeps it in the panel, asks for the one confirm an unsafe step needs, or stays quiet
//! (said once; quiet after a dismissal until the facts change; Silence, Mute, Off; a rule
//! overridden three times demoted to the panel). The rule packs that make findings come next.

pub mod findings;
pub mod interpret;
