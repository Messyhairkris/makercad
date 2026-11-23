//! Sancho's Voice layer for the greeting (doc 07 sections 7.2 and 7.9).
//!
//! Written by Hoss (the Crew, campaign mk-core-25, 2026-11-22) from the contract
//! crew/contracts/mk_voice.md (ATLAS-api-harness), which Gordon wrote himself as a PA (the contract, the reference and the hidden tests).

use crate::memory::Fact;

/// At most this many lines in one greeting (v1 starting value, to tune)
pub const MAX_LINES: usize = 2;

/// Returns a string with a number and the appropriate singular or plural word.
pub fn plural(n: u32, one: &str, many: &str) -> String {
    if n == 1 {
        format!("{} {}", n, one)
    } else {
        format!("{} {}", n, many)
    }
}

/// Returns a phrase describing how many days ago something happened.
pub fn days_phrase(days: u32) -> String {
    match days {
        0 => "today".to_string(),
        1 => "yesterday".to_string(),
        _ => plural(days, "day", "days"),
    }
}

/// Builds greeting lines from a slice of `Fact`s according to the contract.
pub fn greeting(facts: &[Fact]) -> Vec<String> {
    // First-occurrence capture, respecting absence rules.
    let mut days_opt: Option<u32> = None;
    let mut streak_opt: Option<u32> = None;
    let mut completed_opt: Option<u32> = None;
    let mut favourite_opt: Option<String> = None;
    let mut waiting_opt: Option<String> = None;

    for fact in facts {
        match fact {
            Fact::DaysSinceLast(n) => {
                if days_opt.is_none() {
                    days_opt = Some(*n);
                }
            }
            Fact::Streak(n) => {
                if *n != 0 && streak_opt.is_none() {
                    streak_opt = Some(*n);
                }
            }
            Fact::CompletedProjects(n) => {
                if *n != 0 && completed_opt.is_none() {
                    completed_opt = Some(*n);
                }
            }
            Fact::Favourite(text) => {
                if favourite_opt.is_none() && !text.trim().is_empty() {
                    favourite_opt = Some(text.clone());
                }
            }
            Fact::Waiting(text) => {
                if waiting_opt.is_none() && !text.trim().is_empty() {
                    waiting_opt = Some(text.clone());
                }
            }
        }
    }

    // No facts at all that produce a line.
    if days_opt.is_none()
        && streak_opt.is_none()
        && completed_opt.is_none()
        && favourite_opt.is_none()
        && waiting_opt.is_none()
    {
        return Vec::new();
    }

    // Determine lead line and its kind.
    enum LeadKind {
        Days,
        Streak,
        Completed,
        Favourite,
        None,
    }

    let mut lines: Vec<String> = Vec::new();
    let mut lead_kind = LeadKind::None;

    if let Some(days) = days_opt {
        lead_kind = LeadKind::Days;
        if days == 0 {
            lines.push("Welcome back.".to_string());
        } else {
            let phrase = days_phrase(days);
            lines.push(format!("Welcome back - it's been {}.", phrase));
        }
    } else if let Some(streak) = streak_opt {
        lead_kind = LeadKind::Streak;
        lines.push(format!(
            "That's {} in a row.",
            plural(streak, "day", "days")
        ));
    } else if let Some(comp) = completed_opt {
        lead_kind = LeadKind::Completed;
        lines.push(format!(
            "You've finished {}.",
            plural(comp, "project", "projects")
        ));
    } else if let Some(name) = favourite_opt.clone() {
        lead_kind = LeadKind::Favourite;
        lines.push(format!("Still loving the {}.", name));
    }

    // If a waiting line exists, it is always the last line.
    if let Some(wait) = waiting_opt {
        if lines.len() < MAX_LINES {
            lines.push(format!("Waiting on {}.", wait));
        }
        return lines;
    }

    // No waiting line - possibly add a tail line based on the lead.
    if lines.len() < MAX_LINES {
        match lead_kind {
            LeadKind::Days => {
                if let Some(streak) = streak_opt {
                    lines.push(format!(
                        "That's {} in a row.",
                        plural(streak, "day", "days")
                    ));
                } else if let Some(comp) = completed_opt {
                    lines.push(format!(
                        "You've finished {}.",
                        plural(comp, "project", "projects")
                    ));
                } else if let Some(name) = favourite_opt.clone() {
                    lines.push(format!("Still loving the {}.", name));
                }
            }
            LeadKind::Streak => {
                if let Some(comp) = completed_opt {
                    lines.push(format!(
                        "You've finished {}.",
                        plural(comp, "project", "projects")
                    ));
                } else if let Some(name) = favourite_opt.clone() {
                    lines.push(format!("Still loving the {}.", name));
                }
            }
            LeadKind::Completed => {
                if let Some(name) = favourite_opt.clone() {
                    lines.push(format!("Still loving the {}.", name));
                }
            }
            LeadKind::Favourite | LeadKind::None => {
                // No tail line.
            }
        }
    }

    lines
}
