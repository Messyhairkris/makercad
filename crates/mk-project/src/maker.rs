//! Saving and opening a whole project: the manifest and the intent as JSON, the hash index, and
//! the container (doc 03 section 3.6).
//!
//! Written by Claude for P1's `.maker` file v1. A save writes `manifest.json` first and
//! `project/intent.json` second, then the other entries in the order given, and records every
//! entry but the manifest itself in the manifest's hash index. An open refuses what it cannot
//! trust (an unsafe or missing entry, a newer format, an intent that breaks its rules) and
//! reports, without refusing, what changed since the index was written: a project edited
//! outside MakerCAD still opens, and the app says what changed.

use serde::de::DeserializeOwned;

use crate::container::{self, content_hash, ContainerError, Limits};
use crate::layout::{self, Entry, IndexDiff, MANIFEST};
use crate::project::{FormatStatus, Intent, Manifest, ProjectError};

/// The intent's own entry name.
pub const INTENT: &str = "project/intent.json";

/// Why a project could not be saved or opened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MakerError {
    /// The container refused (an unsafe or duplicate name, a limit, not a zip file).
    Container(ContainerError),
    /// Save: one of the other entries is named `manifest.json` or `project/intent.json`.
    Reserved(String),
    /// Open: entries every v1 project holds are absent (see `layout::REQUIRED_V1`).
    Missing(Vec<&'static str>),
    /// An entry that should be JSON is not, or not the expected shape (serde's words).
    Json { name: String, message: String },
    /// Open: written by a newer MakerCAD, whose format this build does not know.
    NewerFormat(u32),
    /// The manifest or the intent breaks one of its rules.
    Project(ProjectError),
}

/// A project read back from its bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Opened {
    pub manifest: Manifest,
    pub intent: Intent,
    /// How the file's format compares with this build's (Older opens as it is).
    pub format: FormatStatus,
    /// Every entry but the manifest and the intent, in the order stored.
    pub entries: Vec<(String, Vec<u8>)>,
    /// What differs from the manifest's hash index: empty when nothing changed outside MakerCAD.
    pub diff: IndexDiff,
}

fn json_error(name: &str, error: serde_json::Error) -> MakerError {
    MakerError::Json {
        name: name.to_string(),
        message: error.to_string(),
    }
}

/// A copy of `manifest` whose index was rebuilt through `set_hash`: sorted, one entry a name,
/// every name checked.
fn normalised(manifest: &Manifest, keep_index: bool) -> Result<Manifest, MakerError> {
    let mut out = Manifest::new(&manifest.app_version);
    out.format_version = manifest.format_version;
    out.environments = manifest.environments.clone();
    if keep_index {
        for entry in manifest.hashes() {
            out.set_hash(&entry.name, &entry.hash)
                .map_err(MakerError::Project)?;
        }
    }
    Ok(out)
}

/// The bytes of a `.maker` file holding `manifest`, `intent` and the other `entries`.
///
/// The manifest is written at this build's format version with a hash index of exactly what is
/// saved; whatever index `manifest` held is replaced.
pub fn save(
    manifest: &Manifest,
    intent: &Intent,
    entries: &[(&str, &[u8])],
) -> Result<Vec<u8>, MakerError> {
    intent.validate().map_err(MakerError::Project)?;
    if let Some((name, _)) = entries
        .iter()
        .find(|(name, _)| *name == MANIFEST || *name == INTENT)
    {
        return Err(MakerError::Reserved(name.to_string()));
    }
    let intent_json = serde_json::to_vec_pretty(intent).map_err(|e| json_error(INTENT, e))?;
    let mut index = normalised(manifest, false)?;
    index.format_version = crate::project::FORMAT_VERSION;
    index
        .set_hash(INTENT, &content_hash(&intent_json))
        .map_err(MakerError::Project)?;
    for (name, bytes) in entries {
        index
            .set_hash(name, &content_hash(bytes))
            .map_err(MakerError::Project)?;
    }
    let manifest_json = serde_json::to_vec_pretty(&index).map_err(|e| json_error(MANIFEST, e))?;
    let mut all: Vec<(&str, &[u8])> = Vec::with_capacity(entries.len() + 2);
    all.push((MANIFEST, &manifest_json));
    all.push((INTENT, &intent_json));
    all.extend_from_slice(entries);
    container::pack(&all).map_err(MakerError::Container)
}

fn parse<T: DeserializeOwned>(entries: &[(String, Vec<u8>)], name: &str) -> Result<T, MakerError> {
    let bytes = entries
        .iter()
        .find(|(n, _)| n == name)
        .map(|(_, b)| b.as_slice())
        .unwrap_or_default();
    serde_json::from_slice(bytes).map_err(|e| json_error(name, e))
}

/// Reads a `.maker` file.
pub fn open(bytes: &[u8], limits: Limits) -> Result<Opened, MakerError> {
    let entries = container::unpack(bytes, limits).map_err(MakerError::Container)?;
    let names: Vec<&str> = entries.iter().map(|(n, _)| n.as_str()).collect();
    let missing = layout::missing_required(&names);
    if !missing.is_empty() {
        return Err(MakerError::Missing(missing));
    }
    let stored: Manifest = parse(&entries, MANIFEST)?;
    let format = stored.format_status();
    if let FormatStatus::Newer(version) = format {
        return Err(MakerError::NewerFormat(version));
    }
    let manifest = normalised(&stored, true)?;
    let intent: Intent = parse(&entries, INTENT)?;
    intent.validate().map_err(MakerError::Project)?;
    let index: Vec<Entry> = manifest
        .hashes()
        .iter()
        .map(|h| Entry {
            name: h.name.clone(),
            hash: h.hash.clone(),
        })
        .collect();
    let present: Vec<Entry> = entries
        .iter()
        .filter(|(n, _)| n != MANIFEST)
        .map(|(n, b)| Entry {
            name: n.clone(),
            hash: content_hash(b),
        })
        .collect();
    let diff = layout::diff_index(&index, &present);
    let others = entries
        .into_iter()
        .filter(|(n, _)| n != MANIFEST && n != INTENT)
        .collect();
    Ok(Opened {
        manifest,
        intent,
        format,
        entries: others,
        diff,
    })
}
