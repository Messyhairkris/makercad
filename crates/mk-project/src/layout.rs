//! What each entry inside a `.maker` project file is, and whether its name is safe.
//!
//! Written by Qwen (the Crew, campaign mk-core-5, 2026-11-21) from the contract
//! crew/contracts/mk_layout.md (ATLAS-api-harness), which Claude finished from Deepak's brief.

/// What an entry is (doc 03, section 3.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntryKind {
    Manifest,
    Intent,
    Inventory,
    Space,
    Inspiration,
    Design,
    DesignExtension,
    Plan,
    Journal,
    Session,
    Media,
    Scan,
    Export,
    Unknown,
}

/// How an entry's bytes are kept in the container.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compression {
    Store,
    Deflate,
}

/// Why an entry name is not safe to extract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameProblem {
    Empty,
    ControlChar,
    Backslash,
    Absolute,
    Colon,
    ForbiddenChar,
    EmptySegment,
    ParentSegment,
    CurrentSegment,
    TrailingDotOrSpace,
    Reserved,
}

/// One entry: its name inside the container and its content hash (hex, any case).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub name: String,
    pub hash: String,
}

/// An entry whose hash differs from the one the manifest recorded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Changed {
    pub name: String,
    pub expected: String,
    pub found: String,
}

/// What `diff_index` found.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IndexDiff {
    pub missing: Vec<String>,
    pub changed: Vec<Changed>,
    pub extra: Vec<String>,
    pub duplicates: Vec<String>,
}

/// The manifest's own entry name.
pub const MANIFEST: &str = "manifest.json";

/// What a v1 project must hold, in this order. "designs/<id>.sc" stands for any one design.
pub const REQUIRED_V1: [&str; 4] = [
    "manifest.json",
    "project/intent.json",
    "designs/<id>.sc",
    "log/journal.jsonl",
];

/// Characters Windows refuses in file names (rule N6).
const FORBIDDEN: [char; 6] = ['<', '>', '"', '|', '?', '*'];

/// Device names reserved on Windows (rule N7e), matched ignoring ASCII case.
const RESERVED: [&str; 22] = [
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

/// Whether `name` is safe to extract (rules N1-N7).
pub fn check_name(name: &str) -> Result<(), NameProblem> {
    // N1
    if name.is_empty() {
        return Err(NameProblem::Empty);
    }
    // N2: a control character (below U+0020 or equal to U+007F)
    if name.chars().any(|c| (c as u32) < 0x20 || c == '\u{7f}') {
        return Err(NameProblem::ControlChar);
    }
    // N3
    if name.contains('\\') {
        return Err(NameProblem::Backslash);
    }
    // N4
    if name.starts_with('/') {
        return Err(NameProblem::Absolute);
    }
    // N5
    if name.contains(':') {
        return Err(NameProblem::Colon);
    }
    // N6
    if FORBIDDEN.iter().any(|c| name.contains(*c)) {
        return Err(NameProblem::ForbiddenChar);
    }
    // N7: segments left to right; within a segment, checks a-e in order.
    for seg in name.split('/') {
        if seg.is_empty() {
            return Err(NameProblem::EmptySegment);
        }
        if seg == ".." {
            return Err(NameProblem::ParentSegment);
        }
        if seg == "." {
            return Err(NameProblem::CurrentSegment);
        }
        if seg.ends_with('.') || seg.ends_with(' ') {
            return Err(NameProblem::TrailingDotOrSpace);
        }
        let stem = match seg.find('.') {
            Some(i) => &seg[..i],
            None => seg,
        };
        if is_reserved(stem) {
            return Err(NameProblem::Reserved);
        }
    }
    Ok(())
}

/// Rule N7e: is `stem` a reserved device name, ignoring ASCII case?
/// Compared over the whole stem, byte-wise, so a multi-byte letter can never
/// straddle a byte boundary.
fn is_reserved(stem: &str) -> bool {
    for r in RESERVED {
        if stem.len() == r.len() && stem.eq_ignore_ascii_case(r) {
            return true;
        }
    }
    false
}

/// What the entry `name` is (rules K0-K14).
pub fn classify(name: &str) -> EntryKind {
    // K0
    if check_name(name).is_err() {
        return EntryKind::Unknown;
    }
    // K1-K3
    if name == "manifest.json" {
        return EntryKind::Manifest;
    }
    if name == "project/intent.json" {
        return EntryKind::Intent;
    }
    if name == "project/inventory.json" {
        return EntryKind::Inventory;
    }
    // K4-K5
    if name.starts_with("project/space/") {
        return EntryKind::Space;
    }
    if name.starts_with("project/inspiration/") {
        return EntryKind::Inspiration;
    }
    // K6-K7
    if design_id(name).is_some() {
        if name.ends_with(".ext.json") {
            return EntryKind::DesignExtension;
        }
        if name.ends_with(".sc") {
            return EntryKind::Design;
        }
    }
    // K8-K9
    if name == "plan/plan.plancraft" {
        return EntryKind::Plan;
    }
    if name == "log/journal.jsonl" {
        return EntryKind::Journal;
    }
    // K10
    if let Some(rest) = name.strip_prefix("log/sessions/") {
        if !rest.contains('/') && rest.ends_with(".json") && rest.len() > ".json".len() {
            return EntryKind::Session;
        }
    }
    // K11-K13
    if name.starts_with("media/") {
        return EntryKind::Media;
    }
    if name.starts_with("scans/") {
        return EntryKind::Scan;
    }
    if name.starts_with("exports/") {
        return EntryKind::Export;
    }
    // K14
    EntryKind::Unknown
}

/// The design id in "designs/<id>.sc" or "designs/<id>.ext.json" (rule I1).
pub fn design_id(name: &str) -> Option<&str> {
    if check_name(name).is_err() {
        return None;
    }
    let rest = name.strip_prefix("designs/")?;
    // I1: ".ext.json" is tried before ".sc".
    let id = rest
        .strip_suffix(".ext.json")
        .or_else(|| rest.strip_suffix(".sc"))?;
    if id.is_empty() || id.contains('/') {
        return None;
    }
    Some(id)
}

/// How the entry `name` is compressed (rules C1-C2).
pub fn compression(name: &str) -> Compression {
    // C1: only the last segment counts.
    let last = match name.rfind('/') {
        Some(i) => &name[i + 1..],
        None => name,
    };
    let lower = last.to_ascii_lowercase();
    const SUFFIXES: [&str; 8] = [
        ".json",
        ".jsonl",
        ".sc",
        ".plancraft",
        ".txt",
        ".md",
        ".csv",
        ".svg",
    ];
    for s in SUFFIXES {
        if lower.ends_with(s) {
            return Compression::Deflate;
        }
    }
    Compression::Store
}

/// The items of `REQUIRED_V1` that `present` does not meet, in `REQUIRED_V1` order (rule M1).
pub fn missing_required(present: &[&str]) -> Vec<&'static str> {
    let mut missing = Vec::new();
    for item in REQUIRED_V1.iter() {
        let met = if *item == "designs/<id>.sc" {
            present.iter().any(|n| classify(n) == EntryKind::Design)
        } else {
            present.contains(item)
        };
        if !met {
            missing.push(*item);
        }
    }
    missing
}

/// Compares the manifest's hash index with the entries present (rules D0-D4).
pub fn diff_index(index: &[Entry], present: &[Entry]) -> IndexDiff {
    // D1: names appearing more than once in `present`, in order of second appearance.
    let mut duplicates: Vec<String> = Vec::new();
    {
        let mut seen: Vec<&str> = Vec::new();
        for e in present {
            if seen.contains(&e.name.as_str()) {
                if !duplicates.contains(&e.name) {
                    duplicates.push(e.name.clone());
                }
            } else {
                seen.push(e.name.as_str());
            }
        }
    }

    // D0: first entry per name, in list order, for each list.
    let index_first = first_entries(index);
    let present_first = first_entries(present);

    // D2: in the index (first entries, index order) but not present.
    let mut missing: Vec<String> = Vec::new();
    for e in &index_first {
        if !present_first.iter().any(|p| p.name == e.name) {
            missing.push(e.name.clone());
        }
    }

    // D3: in both, hashes differ ignoring ASCII case, in index order.
    let mut changed: Vec<Changed> = Vec::new();
    for e in &index_first {
        if let Some(p) = present_first.iter().find(|p| p.name == e.name) {
            if !e.hash.eq_ignore_ascii_case(&p.hash) {
                changed.push(Changed {
                    name: e.name.clone(),
                    expected: e.hash.clone(),
                    found: p.hash.clone(),
                });
            }
        }
    }

    // D4: in present (first entries, present order) but not in the index, except MANIFEST.
    let mut extra: Vec<String> = Vec::new();
    for e in &present_first {
        if e.name != MANIFEST && !index_first.iter().any(|i| i.name == e.name) {
            extra.push(e.name.clone());
        }
    }

    IndexDiff {
        missing,
        changed,
        extra,
        duplicates,
    }
}

/// D0: the first entry for each name, in list order.
fn first_entries(list: &[Entry]) -> Vec<&Entry> {
    let mut out: Vec<&Entry> = Vec::new();
    for e in list {
        if !out.iter().any(|f| f.name == e.name) {
            out.push(e);
        }
    }
    out
}
