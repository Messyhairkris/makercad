//! The `.maker` container itself: entries packed into one zip file and read back (doc 03
//! section 3.6), and the content hash the manifest's index records.
//!
//! Written by Claude for P1's `.maker` file v1, over the Crew's `layout` rules. Every name is
//! checked with `layout::check_name` on the way in and on the way out, so a project never
//! writes, and an opened file never yields, an entry that could escape its folder. JSON is
//! deflated and media stored, as `layout::compression` decides. Every entry carries the same
//! fixed time, so the same project always packs to the same bytes. Reading stops at the limits
//! given, whatever sizes the file claims, so a small file that would inflate to gigabytes is
//! refused rather than unpacked.

use std::collections::HashSet;
use std::io::{Cursor, Read, Write};

use sha2::{Digest, Sha256};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, DateTime, ZipArchive, ZipWriter};

use crate::layout::{self, Compression, NameProblem};

/// How much an opened file may hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    pub max_entries: usize,
    pub max_entry_bytes: u64,
    pub max_total_bytes: u64,
}

impl Default for Limits {
    /// 10,000 entries, 1 GiB an entry, 4 GiB in all. Scans and video are linked rather than
    /// embedded by default (doc 03 section 3.6), so a real project stays far below these.
    fn default() -> Limits {
        Limits {
            max_entries: 10_000,
            max_entry_bytes: 1 << 30,
            max_total_bytes: 4 << 30,
        }
    }
}

/// Why a container could not be written or read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContainerError {
    /// An entry name that is not safe to extract (see `layout::check_name`).
    BadName { name: String, problem: NameProblem },
    /// Two entries whose names differ at most in ASCII case: one Windows folder cannot hold both.
    Duplicate(String),
    /// More entries than the limits allow.
    TooManyEntries,
    /// An entry, or everything read so far, larger than the limits allow.
    TooBig(String),
    /// The zip library refused, in its own words (not a zip file, damaged, encrypted...).
    Zip(String),
}

/// The SHA-256 of `bytes` as 64 lower-case hex digits: what the manifest's hash index holds.
pub fn content_hash(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut hex = String::with_capacity(64);
    for byte in digest {
        hex.push(char::from(HEX[usize::from(byte >> 4)]));
        hex.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    hex
}

const HEX: &[u8; 16] = b"0123456789abcdef";

fn zip_error(error: impl std::fmt::Display) -> ContainerError {
    ContainerError::Zip(error.to_string())
}

/// Checks one name and that no earlier name equals it ignoring ASCII case.
fn admit(name: &str, seen: &mut HashSet<String>) -> Result<(), ContainerError> {
    layout::check_name(name).map_err(|problem| ContainerError::BadName {
        name: name.to_string(),
        problem,
    })?;
    if !seen.insert(name.to_ascii_lowercase()) {
        return Err(ContainerError::Duplicate(name.to_string()));
    }
    Ok(())
}

/// Packs `entries` (name, bytes), in the order given, into the bytes of one `.maker` file.
/// Nothing is written unless every name is safe and unique.
pub fn pack(entries: &[(&str, &[u8])]) -> Result<Vec<u8>, ContainerError> {
    let mut seen = HashSet::new();
    for (name, _) in entries {
        admit(name, &mut seen)?;
    }
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    for (name, bytes) in entries {
        let method = match layout::compression(name) {
            Compression::Store => CompressionMethod::Stored,
            Compression::Deflate => CompressionMethod::Deflated,
        };
        let options = SimpleFileOptions::default()
            .compression_method(method)
            .last_modified_time(DateTime::default())
            .unix_permissions(0o644)
            .large_file(bytes.len() as u64 >= u64::from(u32::MAX));
        writer.start_file(*name, options).map_err(zip_error)?;
        writer.write_all(bytes).map_err(zip_error)?;
    }
    Ok(writer.finish().map_err(zip_error)?.into_inner())
}

/// Reads every file entry of a `.maker` file, in the order stored. Folder entries are skipped.
/// Refuses unsafe or duplicate names and anything past `limits`, before reading further.
pub fn unpack(bytes: &[u8], limits: Limits) -> Result<Vec<(String, Vec<u8>)>, ContainerError> {
    let mut archive = ZipArchive::new(Cursor::new(bytes)).map_err(zip_error)?;
    if archive.len() > limits.max_entries {
        return Err(ContainerError::TooManyEntries);
    }
    let mut seen = HashSet::new();
    let mut entries = Vec::new();
    let mut total: u64 = 0;
    for index in 0..archive.len() {
        let mut file = archive.by_index(index).map_err(zip_error)?;
        if file.is_dir() {
            continue;
        }
        let name = file.name().map_err(zip_error)?.into_owned();
        admit(&name, &mut seen)?;
        if file.size() > limits.max_entry_bytes {
            return Err(ContainerError::TooBig(name));
        }
        // Read at most one byte past the limit: the sizes in the file are only claims.
        let mut content = Vec::new();
        (&mut file)
            .take(limits.max_entry_bytes.saturating_add(1))
            .read_to_end(&mut content)
            .map_err(|error| ContainerError::Zip(format!("{name}: {error}")))?;
        let size = content.len() as u64;
        total = total.saturating_add(size);
        if size > limits.max_entry_bytes || total > limits.max_total_bytes {
            return Err(ContainerError::TooBig(name));
        }
        entries.push((name, content));
    }
    Ok(entries)
}
