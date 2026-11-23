//! The Findings panel's model (doc 07 section 7.4): sort, group, filter and count.
//!
//! Written by Kimi (the Crew, campaign mk-core-30, 2026-11-22) from the contract
//! crew/contracts/mk_panel.md (ATLAS-api-harness), which Gordon wrote himself as a PA (the contract, the reference and the hidden tests).

use crate::findings::{Finding, Kind, Location, Severity};

/// One row of the panel: a finding, and whether the maker dismissed it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub finding: Finding,
    pub dismissed: bool,
}

/// `rows` sorted for the panel: open first, worst first, kind order, else input order.
pub fn sort_rows(rows: &[Row]) -> Vec<Row> {
    let mut sorted = rows.to_vec();
    sorted.sort_by(|left, right| {
        left.dismissed
            .cmp(&right.dismissed)
            .then_with(|| right.finding.severity.cmp(&left.finding.severity))
            .then_with(|| kind_rank(left.finding.kind).cmp(&kind_rank(right.finding.kind)))
    });
    sorted
}

/// What the panel groups by.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Group {
    Severity,
    Kind,
    Location,
}

/// `rows` sorted, then split into (label, rows) in first-seen order.
pub fn group(rows: &[Row], by: Group) -> Vec<(String, Vec<Row>)> {
    let mut groups: Vec<(String, Vec<Row>)> = Vec::new();
    for row in sort_rows(rows) {
        let label = group_label(&row.finding, by);
        // G3: the displayed label, rather than the underlying location, is the key.
        match groups.iter_mut().find(|entry| entry.0 == label) {
            Some((_, grouped_rows)) => grouped_rows.push(row),
            None => groups.push((label, vec![row])),
        }
    }
    groups
}

/// What the maker wants the panel to show. `kinds` empty means every kind.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Filter {
    pub kinds: Vec<Kind>,
    pub min_severity: Option<Severity>,
    pub hide_dismissed: bool,
}

/// The rows `filter` keeps, in input order.
pub fn filter(rows: &[Row], filter: &Filter) -> Vec<Row> {
    rows.iter()
        .filter(|row| {
            let kind_passes = filter.kinds.is_empty() || filter.kinds.contains(&row.finding.kind);
            let severity_passes = match filter.min_severity {
                Some(minimum) => row.finding.severity >= minimum,
                None => true,
            };
            let dismissal_passes = !filter.hide_dismissed || !row.dismissed;
            kind_passes && severity_passes && dismissal_passes
        })
        .cloned()
        .collect()
}

/// The badge on Sancho's portrait: what is open, how many of those are serious, the worst.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Badge {
    pub open: usize,
    pub serious: usize,
    pub worst: Option<Severity>,
}

/// The badge for `rows`.
pub fn badge(rows: &[Row]) -> Badge {
    let mut badge = Badge::default();
    for row in rows.iter().filter(|row| !row.dismissed) {
        badge.open += 1;
        if row.finding.severity == Severity::Critical {
            badge.serious += 1;
        }
        badge.worst = Some(match badge.worst {
            Some(current) => current.max(row.finding.severity),
            None => row.finding.severity,
        });
    }
    badge
}

/// The badge's words, e.g. "3 things to look at, 2 serious".
pub fn badge_text(badge: &Badge) -> String {
    let mut text = match badge.open {
        0 => "Nothing to look at".to_string(),
        1 => "1 thing to look at".to_string(),
        open => format!("{open} things to look at"),
    };
    if badge.serious > 0 {
        text.push_str(&format!(", {} serious", badge.serious));
    }
    text
}

/// S1's third key: the declaration order of `Kind`.
fn kind_rank(kind: Kind) -> u8 {
    match kind {
        Kind::Forbidden => 0,
        Kind::Unsafe => 1,
        Kind::Denied => 2,
        Kind::NotSuitable => 3,
        Kind::Inventory => 4,
        Kind::Fit => 5,
        Kind::Schedule => 6,
        Kind::Budget => 7,
        Kind::Tip => 8,
        Kind::Praise => 9,
    }
}

/// G2's exact label for one finding.
fn group_label(finding: &Finding, by: Group) -> String {
    match by {
        Group::Severity => match finding.severity {
            Severity::Critical => "Critical".to_string(),
            Severity::Warning => "Warning".to_string(),
            Severity::Info => "Info".to_string(),
        },
        Group::Kind => match finding.kind {
            Kind::Forbidden => "Forbidden".to_string(),
            Kind::Unsafe => "Unsafe".to_string(),
            Kind::Denied => "Denied".to_string(),
            Kind::NotSuitable => "Not suitable".to_string(),
            Kind::Inventory => "Inventory".to_string(),
            Kind::Fit => "Fit".to_string(),
            Kind::Schedule => "Schedule".to_string(),
            Kind::Budget => "Budget".to_string(),
            Kind::Tip => "Tip".to_string(),
            Kind::Praise => "Praise".to_string(),
        },
        Group::Location => match &finding.location {
            Location::Project => "Project".to_string(),
            Location::Entity(entity) => entity.clone(),
            Location::Step(step) => format!("Step {step}"),
            Location::Field(field) => field.clone(),
        },
    }
}
