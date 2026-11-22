//! Sancho's relationship memory (doc 07 section 7.9): local, small and opt-out.
//!
//! Written by Qwen (the Crew, campaign mk-core-20, 2026-11-22) from the contract
//! crew/contracts/mk_memory.md (ATLAS-api-harness), which Claude finished from Gordon's brief.

use std::collections::{BTreeMap, BTreeSet};

/// A calendar day as a day number; the app chooses the epoch (consecutive days differ by 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Day(pub i64);

/// One session, as the app reports it when the session ends.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    pub day: Day,
    pub materials: Vec<String>,
    /// The project finished in this session, if one was.
    pub completed: Option<String>,
}

/// A fact the Voice layer may word. No sentences here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fact {
    DaysSinceLast(u32),
    Streak(u32),
    CompletedProjects(u32),
    Favourite(String),
    Waiting(String),
}

/// A rule overridden this many times is demoted to the panel (the same as findings.rs).
pub const DEMOTE_AFTER: u32 = 3;
/// How many favourite materials are kept for the greeting.
pub const FAVOURITES: usize = 3;

/// The relationship memory.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Memory {
    opted_out: bool,
    days: BTreeSet<i64>,
    completed: BTreeSet<String>,
    materials: BTreeMap<String, u32>,
    overrides: BTreeMap<String, u32>,
}

impl Memory {
    /// An empty memory, not opted out.
    pub fn new() -> Memory {
        Memory::default()
    }

    /// Whether the user has opted out (rule M1).
    pub fn opted_out(&self) -> bool {
        self.opted_out
    }

    /// Opt out, clearing everything, or opt back in, starting empty (rule M1).
    pub fn set_opted_out(&mut self, out: bool) {
        if out {
            self.days.clear();
            self.completed.clear();
            self.materials.clear();
            self.overrides.clear();
        }
        self.opted_out = out;
    }

    /// Record a finished session (rule M2); false while opted out.
    pub fn record(&mut self, session: &Session) -> bool {
        if self.opted_out {
            return false;
        }
        self.days.insert(session.day.0);
        let mut seen: BTreeSet<&str> = BTreeSet::new();
        for material in &session.materials {
            if material.trim().is_empty() {
                continue; // M2: empty-after-trim names are skipped
            }
            if seen.insert(material.as_str()) {
                // M2: a name repeated in one session is counted once
                let count = self.materials.entry(material.clone()).or_insert(0);
                *count = count.saturating_add(1);
            }
        }
        if let Some(completed) = &session.completed {
            if !completed.trim().is_empty() {
                self.completed.insert(completed.clone());
            }
        }
        true
    }

    /// The latest day with a session (rule M3).
    pub fn last_day(&self) -> Option<Day> {
        self.days.iter().next_back().map(|&day| Day(day))
    }

    /// Days since the last session: None with no session, 0 when today is not
    /// after the last day, saturating at u32::MAX (rule M3).
    pub fn days_since_last(&self, today: Day) -> Option<u32> {
        let last = *self.days.iter().next_back()?;
        // saturating_sub floors at i64::MIN, so floor at 0 explicitly:
        // today not after the last day means 0 days.
        let diff = today.0.saturating_sub(last);
        if diff <= 0 {
            return Some(0);
        }
        Some(diff.min(u32::MAX as i64) as u32)
    }

    /// Consecutive days with a session, ending at today or today - 1 (rule M4).
    pub fn streak(&self, today: Day) -> u32 {
        let end = if self.days.contains(&today.0) {
            Some(today.0)
        } else {
            today
                .0
                .checked_sub(1)
                .filter(|&yesterday| self.days.contains(&yesterday))
        };
        let Some(end) = end else {
            return 0;
        };
        let mut count = 0u32;
        let mut day = end;
        while self.days.contains(&day) {
            count = count.saturating_add(1);
            match day.checked_sub(1) {
                Some(previous) => day = previous,
                None => break,
            }
        }
        count
    }

    /// The number of distinct completed projects (rule M5).
    pub fn completed_projects(&self) -> u32 {
        self.completed.len() as u32
    }

    /// How many sessions used this material, 0 when unknown (rule M5).
    pub fn material_count(&self, material: &str) -> u32 {
        self.materials.get(material).copied().unwrap_or(0)
    }

    /// Up to FAVOURITES material names, count descending then name ascending (rule M5).
    pub fn favourites(&self) -> Vec<String> {
        let mut ranked: Vec<(&String, u32)> = self
            .materials
            .iter()
            .map(|(name, &count)| (name, count))
            .collect();
        ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));
        ranked
            .into_iter()
            .take(FAVOURITES)
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// How many times a rule was overridden, 0 when unknown (rule M6).
    pub fn overrides(&self, rule: &str) -> u32 {
        self.overrides.get(rule).copied().unwrap_or(0)
    }

    /// Add one override, saturating, and return the new count; 0 while opted out.
    pub fn add_override(&mut self, rule: &str) -> u32 {
        if self.opted_out {
            return 0;
        }
        let count = self.overrides.entry(rule.to_string()).or_insert(0);
        *count = count.saturating_add(1);
        *count
    }

    /// Replace a rule's override count; 0 removes the rule (rule M6).
    pub fn set_overrides(&mut self, rule: &str, count: u32) {
        if self.opted_out {
            return;
        }
        if count == 0 {
            self.overrides.remove(rule);
        } else {
            self.overrides.insert(rule.to_string(), count);
        }
    }

    /// Rules overridden at least DEMOTE_AFTER times, name ascending (rule M6).
    pub fn demoted(&self) -> Vec<String> {
        self.overrides
            .iter()
            .filter(|(_, &count)| count >= DEMOTE_AFTER)
            .map(|(rule, _)| rule.clone())
            .collect()
    }

    /// The facts a greeting may use, in order, each only when it says something
    /// (rule M7); empty while opted out (rule M1).
    pub fn greeting(&self, today: Day, waiting: Option<&str>) -> Vec<Fact> {
        if self.opted_out {
            return Vec::new();
        }
        let mut facts: Vec<Fact> = Vec::new();
        if let Some(days) = self.days_since_last(today).filter(|&days| days >= 1) {
            facts.push(Fact::DaysSinceLast(days));
        }
        let streak = self.streak(today);
        if streak >= 2 {
            facts.push(Fact::Streak(streak));
        }
        let completed = self.completed_projects();
        if completed >= 1 {
            facts.push(Fact::CompletedProjects(completed));
        }
        if let Some(favourite) = self.favourites().into_iter().next() {
            facts.push(Fact::Favourite(favourite));
        }
        if let Some(text) = waiting.map(str::trim).filter(|text| !text.is_empty()) {
            facts.push(Fact::Waiting(text.to_string()));
        }
        facts
    }
}
