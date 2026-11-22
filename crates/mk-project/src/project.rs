//! The manifest and the intent of a .maker project: types, defaults and rules.
//!
//! Written by Hoss (the Crew, campaign mk-core-9, 2026-11-21) from the contract
//! crew/contracts/mk_project.md (ATLAS-api-harness), which Claude finished from Gordon's brief.

use crate::layout::check_name;
use serde::{Deserialize, Serialize};

/// Milliseconds since an epoch the app chooses. The module only stores them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Timestamp(pub u64);

/// The format version this build writes.
pub const FORMAT_VERSION: u32 = 1;

/// How a manifest's format version compares with `FORMAT_VERSION`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatStatus {
    Current,
    Older(u32),
    Newer(u32),
}

/// One entry of the hash index: an entry name in the container and its content hash.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HashEntry {
    pub name: String,
    pub hash: String,
}

/// The manifest (manifest.json).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    pub format_version: u32,
    pub app_version: String,
    pub environments: Vec<String>,
    hashes: Vec<HashEntry>,
}

/// A calendar date. The module checks it is a real date; it never reads today's date.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Date {
    pub year: u16,
    pub month: u8,
    pub day: u8,
}

/// A day of the week.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Weekday {
    Mon,
    Tue,
    Wed,
    Thu,
    Fri,
    Sat,
    Sun,
}

/// A time the maker can work: some days, from start to end, in minutes after midnight.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkWindow {
    pub days: Vec<Weekday>,
    pub start_minute: u16,
    pub end_minute: u16,
}

/// A milestone on the way to the deadline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Milestone {
    pub label: String,
    pub due: Option<Date>,
}

/// The deadline: a date, the work windows, the milestones (doc 04 section 4.5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Deadline {
    pub due: Date,
    pub windows: Vec<WorkWindow>,
    pub milestones: Vec<Milestone>,
}

/// The budget: a cap in the currency's smallest unit (cents, pence), the currency, and
/// whether to spend only if needed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Budget {
    pub cap_minor: u64,
    pub currency: String,
    pub spend_only_if_needed: bool,
}

/// Where spending stands against the budget (doc 04 section 4.5: warnings at 80% and 100%).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BudgetLevel {
    Ok,
    Warn,
    AtCap,
}

/// The maker's skill (doc 04 section 4.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Skill {
    Beginner,
    Comfortable,
    Experienced,
    Pro,
}

/// One edit of the goal, kept with its reason.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntentEdit {
    pub at: Timestamp,
    pub text: String,
    pub why: String,
}

/// The intent (project/intent.json).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Intent {
    pub name: String,
    pub goal: String,
    pub success: Vec<String>,
    pub constraints: Vec<String>,
    pub skill: Skill,
    pub style_words: Vec<String>,
    pub deadline: Option<Deadline>,
    pub budget: Option<Budget>,
    pub history: Vec<IntentEdit>,
}

/// Why a manifest or intent is refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectError {
    NewerFormat(u32),
    EmptyAppVersion,
    UnknownEnvironment(String),
    BadHashName(String),
    EmptyHash(String),
    EmptyName,
    EmptyGoal,
    BadDate(Date),
    BadWorkWindow(usize),
    BadBudget,
    EmptyCurrency,
    EmptyWhy,
}

/// Whether `date` is a real calendar date (rule D1).
pub fn is_real_date(date: Date) -> bool {
    if date.month == 0 || date.month > 12 {
        return false;
    }
    let max_day = match date.month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            let y = date.year as u32;
            if (y.is_multiple_of(4) && !y.is_multiple_of(100)) || y.is_multiple_of(400) {
                29
            } else {
                28
            }
        }
        _ => unreachable!(),
    };
    date.day >= 1 && date.day <= max_day
}

impl Manifest {
    /// A new project's manifest (rule M1).
    pub fn new(app_version: &str) -> Manifest {
        Manifest {
            format_version: FORMAT_VERSION,
            app_version: app_version.to_string(),
            environments: Vec::new(),
            hashes: Vec::new(),
        }
    }

    /// How this manifest's format compares with `FORMAT_VERSION` (rule M2).
    pub fn format_status(&self) -> FormatStatus {
        if self.format_version == FORMAT_VERSION {
            FormatStatus::Current
        } else if self.format_version < FORMAT_VERSION {
            FormatStatus::Older(self.format_version)
        } else {
            FormatStatus::Newer(self.format_version)
        }
    }

    /// Checks the manifest (rules M3-M5).
    pub fn validate(&self, known_environments: &[&str]) -> Result<(), ProjectError> {
        // M3: newer format
        if self.format_version > FORMAT_VERSION {
            return Err(ProjectError::NewerFormat(self.format_version));
        }
        // blank app_version
        if self.app_version.trim().is_empty() {
            return Err(ProjectError::EmptyAppVersion);
        }
        // first unknown environment
        for env in &self.environments {
            if !known_environments.iter().any(|&k| k == env) {
                return Err(ProjectError::UnknownEnvironment(env.clone()));
            }
        }
        // M4: each hash entry
        for entry in &self.hashes {
            if check_name(&entry.name).is_err() {
                return Err(ProjectError::BadHashName(entry.name.clone()));
            }
            if entry.hash.trim().is_empty() {
                return Err(ProjectError::EmptyHash(entry.name.clone()));
            }
        }
        // M5: ok
        Ok(())
    }

    /// Adds or replaces one hash index entry (rule H1).
    pub fn set_hash(&mut self, name: &str, hash: &str) -> Result<(), ProjectError> {
        // name check
        if check_name(name).is_err() {
            return Err(ProjectError::BadHashName(name.to_string()));
        }
        // blank hash
        if hash.trim().is_empty() {
            return Err(ProjectError::EmptyHash(name.to_string()));
        }
        // binary search for insertion point
        match self.hashes.binary_search_by(|e| e.name.as_str().cmp(name)) {
            Ok(idx) => {
                self.hashes[idx].hash = hash.to_string();
            }
            Err(idx) => {
                self.hashes.insert(
                    idx,
                    HashEntry {
                        name: name.to_string(),
                        hash: hash.to_string(),
                    },
                );
            }
        }
        Ok(())
    }

    /// The hash recorded for `name`, if any.
    pub fn hash_of(&self, name: &str) -> Option<&str> {
        self.hashes
            .binary_search_by(|e| e.name.as_str().cmp(name))
            .ok()
            .map(|idx| self.hashes[idx].hash.as_str())
    }

    /// The hash index, sorted by name (rule H2).
    pub fn hashes(&self) -> &[HashEntry] {
        &self.hashes
    }
}

impl Intent {
    /// A new project's intent (rule I1).
    pub fn new(name: &str, goal: &str) -> Result<Intent, ProjectError> {
        if name.trim().is_empty() {
            return Err(ProjectError::EmptyName);
        }
        if goal.trim().is_empty() {
            return Err(ProjectError::EmptyGoal);
        }
        Ok(Intent {
            name: name.to_string(),
            goal: goal.to_string(),
            success: Vec::new(),
            constraints: Vec::new(),
            skill: Skill::Beginner,
            style_words: Vec::new(),
            deadline: None,
            budget: None,
            history: Vec::new(),
        })
    }

    /// Checks the intent (rules I2-I3).
    pub fn validate(&self) -> Result<(), ProjectError> {
        // blank name
        if self.name.trim().is_empty() {
            return Err(ProjectError::EmptyName);
        }
        // blank goal
        if self.goal.trim().is_empty() {
            return Err(ProjectError::EmptyGoal);
        }
        // deadline checks
        if let Some(dl) = &self.deadline {
            if !is_real_date(dl.due) {
                return Err(ProjectError::BadDate(dl.due));
            }
            // milestones
            for ms in &dl.milestones {
                if let Some(d) = ms.due {
                    if !is_real_date(d) {
                        return Err(ProjectError::BadDate(d));
                    }
                }
            }
            // work windows
            for (i, ww) in dl.windows.iter().enumerate() {
                if ww.days.is_empty() || ww.start_minute >= ww.end_minute || ww.end_minute > 1440 {
                    return Err(ProjectError::BadWorkWindow(i));
                }
            }
        }
        // budget checks
        if let Some(b) = &self.budget {
            if b.cap_minor == 0 {
                return Err(ProjectError::BadBudget);
            }
            if b.currency.trim().is_empty() {
                return Err(ProjectError::EmptyCurrency);
            }
        }
        Ok(())
    }

    /// Changes the goal and keeps the edit with its reason (rule I4).
    pub fn edit_goal(&mut self, at: Timestamp, text: &str, why: &str) -> Result<(), ProjectError> {
        if text.trim().is_empty() {
            return Err(ProjectError::EmptyGoal);
        }
        if why.trim().is_empty() {
            return Err(ProjectError::EmptyWhy);
        }
        self.history.push(IntentEdit {
            at,
            text: text.to_string(),
            why: why.to_string(),
        });
        self.goal = text.to_string();
        Ok(())
    }

    /// Where `spent_minor` stands against the budget (rule B1).
    pub fn budget_level(&self, spent_minor: u64) -> Option<BudgetLevel> {
        let b = self.budget.as_ref()?;
        if spent_minor >= b.cap_minor {
            return Some(BudgetLevel::AtCap);
        }
        let spent = spent_minor as u128;
        let cap = b.cap_minor as u128;
        if spent * 5 >= cap * 4 {
            return Some(BudgetLevel::Warn);
        }
        Some(BudgetLevel::Ok)
    }
}
