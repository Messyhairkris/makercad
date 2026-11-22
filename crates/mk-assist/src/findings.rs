//! Sancho's findings and the ledger that decides when each one is said.
//!
//! Written by Hoss (the Crew, campaign mk-core-13, 2026-11-22) from the contract
//! crew/contracts/mk_findings.md (ATLAS-api-harness), which Claude finished from Deepak's brief.

use std::collections::{HashMap, HashSet};

/// The ten kinds of finding (doc 07 section 7.4), in the table's order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    Forbidden,
    Unsafe,
    Denied,
    NotSuitable,
    Inventory,
    Fit,
    Schedule,
    Budget,
    Tip,
    Praise,
}

/// How serious a finding is. Declared lowest first, so Info < Warning < Critical.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Severity {
    Info,
    Warning,
    Critical,
}

/// Where a finding came from: a rule (by id) or the optional Mind.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Source {
    Rule(String),
    Mind,
}

/// What a finding is about.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Location {
    Project,
    Entity(String),
    Step(u32),
    Field(String),
}

/// One finding. `what` and `why` are the app's words and never decide anything here.
/// `facts` are the finding's material facts (key, value), in the rule's own order: when they
/// change, the finding is new again.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub kind: Kind,
    pub severity: Severity,
    pub source: Source,
    pub location: Location,
    pub what: String,
    pub why: String,
    pub fix: Option<String>,
    pub facts: Vec<(String, String)>,
}

/// Which finding this is, whatever its facts and words.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identity {
    pub source: Source,
    pub kind: Kind,
    pub location: Location,
}

/// Sancho's switch (doc 07 section 7.4): On, Silence (voice off), Mute (nothing), Off (removed).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mode {
    On,
    Silence,
    Mute,
    Off,
}

/// What happens to one finding in one evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Delivery {
    /// Said aloud (the Voice layer may show a bubble with it).
    Speak,
    /// Popped as a bubble only: Silence is on.
    Bubble,
    /// Waits in the Findings panel.
    Panel,
    /// Needs one explicit confirm before the work goes on (unsafe only).
    Confirm,
    /// Not shown: dismissed, overridden, a duplicate, or Sancho is Off.
    Hidden,
}

/// What the maker did to ask for this evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Context {
    /// The maker asked Sancho ("what do you think?").
    pub asked: bool,
    /// Coach mode is on.
    pub coach: bool,
}

/// One evaluation's result: a delivery per input finding, in input order, and the badge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evaluation {
    pub deliveries: Vec<Delivery>,
    pub badge: usize,
}

/// What the journal's finding_override record needs (doc 10 section 10.4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Override {
    pub rule: String,
    pub reason: Option<String>,
}

/// A rule overridden this many times is demoted to the panel (doc 07 section 7.9).
pub const DEMOTE_AFTER: u32 = 3;

/// What the ledger remembers about one identity.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Memory {
    said: Option<Vec<(String, String)>>,
    quiet: Option<Vec<(String, String)>>,
}

/// The ledger: Sancho's memory of what he has said, what was dismissed and overridden.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ledger {
    mode: Mode,
    memory: HashMap<Identity, Memory>,
    overrides: HashMap<String, u32>,
}

/// Returns the identity of a finding (source, kind, location).
pub fn identity(finding: &Finding) -> Identity {
    Identity {
        source: finding.source.clone(),
        kind: finding.kind,
        location: finding.location.clone(),
    }
}

/// Returns true if the kind is one that Sancho speaks about.
pub fn speaks(kind: Kind) -> bool {
    matches!(
        kind,
        Kind::Forbidden | Kind::Unsafe | Kind::Denied | Kind::Inventory | Kind::Fit | Kind::Praise
    )
}

/// Returns a vector of indices sorted for the panel view.
pub fn panel_order(findings: &[Finding]) -> Vec<usize> {
    let mut idxs: Vec<usize> = (0..findings.len()).collect();
    idxs.sort_by(|&a, &b| {
        let fa = &findings[a];
        let fb = &findings[b];
        // severity: Critical first (reverse order)
        let sev_cmp = fb.severity.cmp(&fa.severity);
        if sev_cmp != std::cmp::Ordering::Equal {
            return sev_cmp;
        }
        // kind order: enum declaration order (as usize)
        let kind_cmp = (fa.kind as usize).cmp(&(fb.kind as usize));
        if kind_cmp != std::cmp::Ordering::Equal {
            return kind_cmp;
        }
        // finally by original index
        a.cmp(&b)
    });
    idxs
}

impl Ledger {
    /// Creates a new ledger with mode On and empty memory.
    pub fn new() -> Ledger {
        Ledger {
            mode: Mode::On,
            memory: HashMap::new(),
            overrides: HashMap::new(),
        }
    }

    /// Returns the current mode.
    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// Sets the current mode.
    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    /// Evaluates a list of findings under the given context.
    pub fn evaluate(&mut self, findings: &[Finding], context: Context) -> Evaluation {
        let mut deliveries = Vec::with_capacity(findings.len());
        let mut badge = 0usize;
        let mut seen: HashSet<Identity> = HashSet::new();

        for finding in findings {
            let id = identity(finding);

            // E1: duplicate identity earlier in same list
            if seen.contains(&id) {
                deliveries.push(Delivery::Hidden);
                continue;
            }
            seen.insert(id.clone());

            // Helper closures for memory checks
            let is_quiet = |ledger: &Ledger, id: &Identity, f: &Finding| {
                ledger.memory.get(id).and_then(|m| m.quiet.as_ref()) == Some(&f.facts)
            };
            let is_said = |ledger: &Ledger, id: &Identity, f: &Finding| {
                ledger.memory.get(id).and_then(|m| m.said.as_ref()) == Some(&f.facts)
            };

            // E2: Unsafe and not remembered quiet
            if matches!(finding.kind, Kind::Unsafe) && !is_quiet(self, &id, finding) {
                deliveries.push(Delivery::Confirm);
                continue;
            }

            // E3: remembered quiet
            if is_quiet(self, &id, finding) {
                deliveries.push(Delivery::Hidden);
                continue;
            }

            // E4: mode Off
            if self.mode == Mode::Off {
                deliveries.push(Delivery::Hidden);
                continue;
            }

            // E5: mode Mute
            if self.mode == Mode::Mute {
                deliveries.push(Delivery::Panel);
                // badge stays zero for Mute/Off (handled later)
                continue;
            }

            // E6: context asked
            if context.asked {
                let delivery = if self.mode == Mode::Silence {
                    Delivery::Bubble
                } else {
                    Delivery::Speak
                };
                deliveries.push(delivery);
                self.memory.entry(id.clone()).or_default().said = Some(finding.facts.clone());
                continue;
            }

            // E7: remembered said
            if is_said(self, &id, finding) {
                deliveries.push(Delivery::Panel);
                continue;
            }

            // E8: source is Rule and demoted
            if let Source::Rule(ref rule_id) = finding.source {
                if self.is_demoted(rule_id) {
                    deliveries.push(Delivery::Panel);
                    badge += 1;
                    continue;
                }
            }

            // E9: speaks(kind) or coach mode
            if speaks(finding.kind) || context.coach {
                let delivery = if self.mode == Mode::Silence {
                    Delivery::Bubble
                } else {
                    Delivery::Speak
                };
                deliveries.push(delivery);
                self.memory.entry(id.clone()).or_default().said = Some(finding.facts.clone());
                continue;
            }

            // E10: otherwise
            deliveries.push(Delivery::Panel);
            badge += 1;
        }

        // Badge is zero when mode is Mute or Off
        if matches!(self.mode, Mode::Mute | Mode::Off) {
            badge = 0;
        }

        Evaluation { deliveries, badge }
    }

    /// Dismisses a finding unless it is Unsafe.
    pub fn dismiss(&mut self, finding: &Finding) -> bool {
        if matches!(finding.kind, Kind::Unsafe) {
            return false;
        }
        let id = identity(finding);
        self.memory.entry(id).or_default().quiet = Some(finding.facts.clone());
        true
    }

    /// Overrides a finding, remembering it as quiet and tracking rule overrides.
    pub fn override_finding(
        &mut self,
        finding: &Finding,
        reason: Option<&str>,
    ) -> Option<Override> {
        // Remember quiet
        let id = identity(finding);
        self.memory.entry(id).or_default().quiet = Some(finding.facts.clone());

        // Trim reason
        let trimmed = reason
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        match &finding.source {
            Source::Rule(rule_id) => {
                let count = self.overrides.entry(rule_id.clone()).or_insert(0);
                *count = count.saturating_add(1);
                Some(Override {
                    rule: rule_id.clone(),
                    reason: trimmed,
                })
            }
            Source::Mind => None,
        }
    }

    /// Returns the number of overrides for a rule.
    pub fn overrides(&self, rule: &str) -> u32 {
        *self.overrides.get(rule).unwrap_or(&0)
    }

    /// Sets the override count for a rule.
    pub fn set_overrides(&mut self, rule: &str, count: u32) {
        self.overrides.insert(rule.to_string(), count);
    }

    /// Returns true if a rule is demoted (overridden enough times).
    pub fn is_demoted(&self, rule: &str) -> bool {
        self.overrides(rule) >= DEMOTE_AFTER
    }
}

impl Default for Ledger {
    fn default() -> Ledger {
        Ledger::new()
    }
}
