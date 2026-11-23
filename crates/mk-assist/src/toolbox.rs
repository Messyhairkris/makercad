//! The side toolbox's contents (doc 05 section 5.4): what is shown, in what order, and why.
//!
//! Written by Deepak (the Crew, campaign mk-core-29, 2026-11-22) from the contract
//! crew/contracts/mk_toolbox.md (ATLAS-api-harness), which Gordon wrote himself as a PA (the contract, the reference and the hidden tests).

/// The toolbox size a caller uses when it has no opinion.
///
/// (v1 starting value, to tune)
pub const DEFAULT_MAX_SHOWN: usize = 12;

/// One command of the catalogue, in the caller's default order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    /// The command's identity, compared byte for byte (A1).
    pub id: String,
    /// The UI group; carried for the caller, never read here.
    pub group: String,
}

/// Why a command is in the side toolbox.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Reason {
    /// The maker pinned it.
    Pinned,
    /// The interpreter suggested it.
    Suggested,
    /// It is marked always-show.
    Always,
    /// The Quest was skipped, so the catalogue's default order fills the toolbox.
    IntentMissing,
}

/// What the side toolbox shows, in order, and what is left to search.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Toolbox {
    /// The kept commands in candidate order, each with the reason it is shown.
    pub shown: Vec<(String, Reason)>,
    /// Every catalogue id not shown, in catalogue default order.
    pub rest: Vec<String>,
    /// The caller's flag, echoed (A5).
    pub intent_missing: bool,
}

/// A1/A2: add `id` as a candidate when it is in the catalogue and not yet a candidate.
///
/// The first place an id reaches decides both its position and its reason.
fn add_candidate(candidates: &mut Vec<(String, Reason)>, ids: &[&str], id: &str, reason: Reason) {
    if !ids.contains(&id) {
        return;
    }
    if candidates.iter().any(|(known, _)| known.as_str() == id) {
        return;
    }
    candidates.push((id.to_string(), reason));
}

/// Builds the side toolbox from the catalogue and the three id lists.
pub fn build(
    catalogue: &[Command],
    suggested: &[String],
    pinned: &[String],
    always: &[String],
    intent_missing: bool,
    max_shown: usize,
) -> Toolbox {
    // A1: the first occurrence of an id is the command; later ones are ignored.
    let mut ids: Vec<&str> = Vec::new();
    for command in catalogue {
        if !ids.contains(&command.id.as_str()) {
            ids.push(command.id.as_str());
        }
    }

    let mut candidates: Vec<(String, Reason)> = Vec::new();
    if intent_missing {
        // A4: suggested and always are ignored; pinned first, then the catalogue order.
        for id in pinned {
            add_candidate(&mut candidates, &ids, id, Reason::Pinned);
        }
        for &id in &ids {
            add_candidate(&mut candidates, &ids, id, Reason::IntentMissing);
        }
    } else {
        // A2: pinned, then suggested, then always; the first place wins.
        for id in pinned {
            add_candidate(&mut candidates, &ids, id, Reason::Pinned);
        }
        for id in suggested {
            add_candidate(&mut candidates, &ids, id, Reason::Suggested);
        }
        for id in always {
            add_candidate(&mut candidates, &ids, id, Reason::Always);
        }
    }

    // A3: 0 means no cut; any other value keeps at most that many candidates.
    if max_shown != 0 && candidates.len() > max_shown {
        candidates.truncate(max_shown);
    }

    // A3: rest is every catalogue id not shown, in catalogue default order.
    let mut rest: Vec<String> = Vec::new();
    for &id in &ids {
        if !candidates.iter().any(|(known, _)| known.as_str() == id) {
            rest.push(id.to_string());
        }
    }

    Toolbox {
        shown: candidates,
        rest,
        intent_missing,
    }
}

/// A6: true exactly when `shown` holds a pair whose id equals `id`.
pub fn is_shown(toolbox: &Toolbox, id: &str) -> bool {
    toolbox.shown.iter().any(|(shown_id, _)| shown_id == id)
}

/// A6: the reason of the first shown pair with this id, or `None` when it is not shown.
pub fn reason_of(toolbox: &Toolbox, id: &str) -> Option<Reason> {
    toolbox
        .shown
        .iter()
        .find(|(shown_id, _)| shown_id == id)
        .map(|(_, reason)| *reason)
}
