//! The project journal: append-only records, edits as superseding records, and their views.
//!
//! Written by Deepak (the Crew, campaign mk-core-8, 2026-11-21) from the contract
//! crew/contracts/mk_journal.md (ATLAS-api-harness), which Claude finished from Deepak's brief.

/// Milliseconds since an epoch the app chooses. The module only compares them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timestamp(pub u64);

/// A record's id: 1 for the first record appended, then 2, 3, ...
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RecordId(pub u64);

/// The kind of a record (doc 10 section 10.4, plus open pages).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RecordKind {
    Intent,
    Decision,
    Step,
    Session,
    Media,
    FindingOverride,
    AsBuilt,
    Outcome,
    OpenPage,
}

/// Where a decision came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecisionSource {
    WhyButton,
    FitSolver,
    InventoryOverride,
}

/// The moments the journal prompts for (doc 10 section 10.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptMoment {
    ProjectCreation,
    FirstMaterial,
    FirstFitToSpace,
    DesignChangeAfterMaterial,
    FirstPhysicalTestFit,
    SessionEnd,
    Conclusion,
}

/// A photo or video, linked or embedded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaRef {
    pub path: String,
    pub hash: Option<String>,
    pub external: bool,
    /// In and out points of a video clip, in milliseconds from the start.
    pub clip: Option<(u64, u64)>,
}

/// What a media record shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MediaTarget {
    Step(RecordId),
    Part(String),
    Unassigned,
}

/// One option a decision considered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecisionOption {
    pub label: String,
    pub thumbnail: Option<MediaRef>,
}

/// A deviation found by an as-built comparison.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Deviation {
    pub part: String,
    pub expected: String,
    pub actual: String,
}

/// What a record says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Body {
    Intent {
        goal: String,
        criteria: Vec<String>,
        why: String,
    },
    Decision {
        question: String,
        options: Vec<DecisionOption>,
        choice: Option<String>,
        reason: String,
        source: DecisionSource,
    },
    Step {
        order: u32,
        text: String,
    },
    Session {
        did: Vec<String>,
        worked: Vec<String>,
        didnt: Vec<String>,
        next: Vec<String>,
        media: Vec<MediaRef>,
        mood: Option<String>,
    },
    Media {
        media: MediaRef,
        caption: Option<String>,
        shows: MediaTarget,
    },
    FindingOverride {
        rule: String,
        reason: String,
    },
    AsBuilt {
        deviations: Vec<Deviation>,
        callouts: Vec<String>,
    },
    Outcome {
        final_photos: Vec<MediaRef>,
        would_change: Vec<String>,
        build_again: Option<bool>,
        tips: Vec<String>,
    },
    OpenPage {
        prompt: PromptMoment,
    },
}

/// One record of the journal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    pub id: RecordId,
    pub at: Timestamp,
    pub supersedes: Option<RecordId>,
    pub body: Body,
}

/// Why a record was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JournalError {
    BadId,
    UnknownSupersede,
    AlreadySuperseded,
    KindMismatch,
    EarlierThanLast,
    EmptyGoal,
    EmptyReason,
    EmptyRule,
    BadClip,
    DuplicateStepOrder,
}

/// The kind of a body.
pub fn kind_of(body: &Body) -> RecordKind {
    match body {
        Body::Intent { .. } => RecordKind::Intent,
        Body::Decision { .. } => RecordKind::Decision,
        Body::Step { .. } => RecordKind::Step,
        Body::Session { .. } => RecordKind::Session,
        Body::Media { .. } => RecordKind::Media,
        Body::FindingOverride { .. } => RecordKind::FindingOverride,
        Body::AsBuilt { .. } => RecordKind::AsBuilt,
        Body::Outcome { .. } => RecordKind::Outcome,
        Body::OpenPage { .. } => RecordKind::OpenPage,
    }
}

/// Rule A5: a clip is well formed when its out point is after its in point.
fn clip_ok(media: &MediaRef) -> bool {
    match media.clip {
        Some((a, b)) => a < b,
        None => true,
    }
}

/// Rule A5: a decision option's thumbnail, when present, must have a well-formed clip.
fn thumbnail_ok(option: &DecisionOption) -> bool {
    match &option.thumbnail {
        Some(media) => clip_ok(media),
        None => true,
    }
}

/// Rule A5: every media ref the body carries must have a well-formed clip.
fn media_refs_ok(body: &Body) -> bool {
    match body {
        Body::Media { media, .. } => clip_ok(media),
        Body::Session { media, .. } => media.iter().all(clip_ok),
        Body::Outcome { final_photos, .. } => final_photos.iter().all(clip_ok),
        Body::Decision { options, .. } => options.iter().all(thumbnail_ok),
        _ => true,
    }
}

/// Rule A5: the field checks, in the order the contract lists them.
fn fields_ok(body: &Body) -> Result<(), JournalError> {
    match body {
        Body::Intent { goal, .. } => {
            if goal.trim().is_empty() {
                return Err(JournalError::EmptyGoal);
            }
        }
        Body::Decision { reason, .. } => {
            if reason.trim().is_empty() {
                return Err(JournalError::EmptyReason);
            }
        }
        Body::FindingOverride { rule, reason } => {
            if reason.trim().is_empty() {
                return Err(JournalError::EmptyReason);
            }
            if rule.trim().is_empty() {
                return Err(JournalError::EmptyRule);
            }
        }
        _ => {}
    }
    if !media_refs_ok(body) {
        return Err(JournalError::BadClip);
    }
    Ok(())
}

/// Rule A6: a current step this record does not supersede already has this order.
fn step_order_taken(records: &[Record], supersedes: Option<RecordId>, order: u32) -> bool {
    records.iter().any(|record| {
        if Some(record.id) == supersedes {
            return false;
        }
        if records
            .iter()
            .any(|other| other.supersedes == Some(record.id))
        {
            return false;
        }
        match &record.body {
            Body::Step {
                order: existing, ..
            } => *existing == order,
            _ => false,
        }
    })
}

/// Rules A1-A6, checked against the journal as it is before the new record.
fn check_append(
    records: &[Record],
    at: Timestamp,
    supersedes: Option<RecordId>,
    body: &Body,
) -> Result<(), JournalError> {
    if let Some(old) = supersedes {
        let old_record = match records.iter().find(|record| record.id == old) {
            Some(record) => record,
            None => return Err(JournalError::UnknownSupersede),
        };
        if records.iter().any(|record| record.supersedes == Some(old)) {
            return Err(JournalError::AlreadySuperseded);
        }
        let old_kind = kind_of(&old_record.body);
        if old_kind != RecordKind::OpenPage && old_kind != kind_of(body) {
            return Err(JournalError::KindMismatch);
        }
    }
    if records.last().is_some_and(|last| at < last.at) {
        return Err(JournalError::EarlierThanLast);
    }
    fields_ok(body)?;
    let duplicate_step = match body {
        Body::Step { order, .. } => step_order_taken(records, supersedes, *order),
        _ => false,
    };
    if duplicate_step {
        return Err(JournalError::DuplicateStepOrder);
    }
    Ok(())
}

/// The journal: the append-only list of records.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Journal {
    records: Vec<Record>,
}

impl Journal {
    /// An empty journal.
    pub fn new() -> Journal {
        Journal {
            records: Vec::new(),
        }
    }

    /// Rebuilds a journal from records read back, checking each in order as `append` would
    /// (rules A1-A7); the first refused record's index and error are returned.
    pub fn from_records(records: Vec<Record>) -> Result<Journal, (usize, JournalError)> {
        let mut built: Vec<Record> = Vec::with_capacity(records.len());
        for (index, record) in records.into_iter().enumerate() {
            let expected = match built.last() {
                Some(last) => last.id.0 + 1,
                None => 1,
            };
            if record.id.0 != expected {
                return Err((index, JournalError::BadId));
            }
            check_append(&built, record.at, record.supersedes, &record.body)
                .map_err(|error| (index, error))?;
            built.push(record);
        }
        Ok(Journal { records: built })
    }

    /// Appends a record (rules A1-A8) and returns its id.
    pub fn append(
        &mut self,
        at: Timestamp,
        supersedes: Option<RecordId>,
        body: Body,
    ) -> Result<RecordId, JournalError> {
        check_append(&self.records, at, supersedes, &body)?;
        let id = RecordId(match self.records.last() {
            Some(last) => last.id.0 + 1,
            None => 1,
        });
        self.records.push(Record {
            id,
            at,
            supersedes,
            body,
        });
        Ok(id)
    }

    /// Every record, in append order, superseded ones included.
    pub fn records(&self) -> &[Record] {
        &self.records
    }

    /// Whether some record supersedes `id`.
    fn is_superseded(&self, id: RecordId) -> bool {
        self.records
            .iter()
            .any(|record| record.supersedes == Some(id))
    }

    /// The records no other record supersedes, in append order (rule V1).
    pub fn current(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| !self.is_superseded(record.id))
            .collect()
    }

    /// The edit history that `id` belongs to, oldest first (rule V2).
    pub fn history_of(&self, id: RecordId) -> Vec<&Record> {
        let mut first = match self.records.iter().find(|record| record.id == id) {
            Some(record) => record,
            None => return Vec::new(),
        };
        // Rule V2: walk back to the record whose `supersedes` is None.
        while let Some(previous) = first.supersedes {
            match self.records.iter().find(|record| record.id == previous) {
                Some(record) => first = record,
                None => break,
            }
        }
        // Rule V2: then forward, each step taking the one record that supersedes the previous.
        let mut chain = vec![first];
        let mut cursor = first;
        loop {
            let next = self
                .records
                .iter()
                .find(|record| record.supersedes == Some(cursor.id));
            match next {
                Some(record) => {
                    chain.push(record);
                    cursor = record;
                }
                None => break,
            }
        }
        chain
    }

    /// The last current session record (rule V3).
    pub fn last_session(&self) -> Option<&Record> {
        self.current()
            .into_iter()
            .rev()
            .find(|record| kind_of(&record.body) == RecordKind::Session)
    }

    /// The current open pages, in append order (rule V4).
    pub fn open_pages(&self) -> Vec<&Record> {
        self.current()
            .into_iter()
            .filter(|record| kind_of(&record.body) == RecordKind::OpenPage)
            .collect()
    }
}
