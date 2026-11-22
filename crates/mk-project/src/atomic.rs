//! Saving a file so that a crash never leaves the maker with nothing.
//!
//! Written by Hoss (the Crew, campaign mk-core-7, 2026-11-21) from the contract
//! crew/contracts/mk_atomic.md v2 (ATLAS-api-harness): every file operation goes through a
//! `Disk` the caller passes in. The app gives it the real file system.

/// What kind of failure a `Disk` reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    NotFound,
    InvalidInput,
    InvalidData,
    Other,
}

/// A failure, from the `Disk` or from this module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiskError {
    pub kind: ErrorKind,
    pub message: String,
}

/// What is at a name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Found {
    Missing,
    File,
    Folder,
}

/// The files of one folder, by name. The app implements it over the real folder.
pub trait Disk {
    /// What is at `name`.
    fn found(&mut self, name: &str) -> Found;
    /// The whole file.
    fn read(&mut self, name: &str) -> Result<Vec<u8>, DiskError>;
    /// Creates or truncates `name`, writes all of `bytes`, and syncs it to the device.
    fn write(&mut self, name: &str, bytes: &[u8]) -> Result<(), DiskError>;
    /// Copies the file `from` over `to`, replacing `to`.
    fn copy(&mut self, from: &str, to: &str) -> Result<(), DiskError>;
    /// Renames the file `from` to `to` in one step, replacing `to`.
    fn rename(&mut self, from: &str, to: &str) -> Result<(), DiskError>;
    /// Removes the file `name`.
    fn remove(&mut self, name: &str) -> Result<(), DiskError>;
}

/// What `open` found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Opened {
    /// The target itself, whole.
    Current(Vec<u8>),
    /// The target was missing or damaged; this is the recovery copy.
    Recovered(Vec<u8>),
}

/// The file a save writes first: `target` + ".saving".
pub fn temp_name(target: &str) -> String {
    format!("{}{}", target, ".saving")
}

/// The recovery copy: `target` + ".backup".
pub fn backup_name(target: &str) -> String {
    format!("{}{}", target, ".backup")
}

/// Saves `bytes` as `target` (rules S1-S5).
pub fn save(disk: &mut impl Disk, target: &str, bytes: &[u8]) -> Result<(), DiskError> {
    // S1 - validate name
    if target.is_empty() || target.contains('/') || target.contains('\\') {
        return Err(DiskError {
            kind: ErrorKind::InvalidInput,
            message: "Invalid input".into(),
        });
    }
    // After name checks, ensure target is not a folder.
    if let Found::Folder = disk.found(target) {
        return Err(DiskError {
            kind: ErrorKind::InvalidInput,
            message: "Target is a folder".into(),
        });
    }

    // S2 - write temporary file.
    let temp = temp_name(target);
    if let Err(e) = disk.write(&temp, bytes) {
        let _ = disk.remove(&temp);
        return Err(e);
    }

    // S3 - copy existing target to backup if it exists.
    if let Found::File = disk.found(target) {
        let backup = backup_name(target);
        if let Err(e) = disk.copy(target, &backup) {
            let _ = disk.remove(&temp);
            return Err(e);
        }
    }

    // S4 - rename temporary file to the final name.
    if let Err(e) = disk.rename(&temp, target) {
        let _ = disk.remove(&temp);
        return Err(e);
    }

    // S5 - success.
    Ok(())
}

/// Reads `target`, or its recovery copy when `target` is missing or damaged (rules O1-O4).
/// `check` decides whether a byte slice represents a whole file.
pub fn open(
    disk: &mut impl Disk,
    target: &str,
    check: impl Fn(&[u8]) -> bool,
) -> Result<Opened, DiskError> {
    // Record the presence of target and backup once each.
    let target_found = disk.found(target);
    let backup_name = backup_name(target);
    let backup_found = disk.found(&backup_name);

    // O1 - try the current file if it exists.
    if let Found::File = target_found {
        match disk.read(target) {
            Ok(data) if check(&data) => return Ok(Opened::Current(data)),
            // Any failure (read error or check false) falls through to O2.
            _ => {}
        }
    }

    // O2 - try the backup if it exists.
    if let Found::File = backup_found {
        match disk.read(&backup_name) {
            Ok(data) if check(&data) => return Ok(Opened::Recovered(data)),
            // Failure falls through to O3.
            _ => {}
        }
    }

    // O3 - a file was present but invalid or unreadable.
    if matches!(target_found, Found::File) || matches!(backup_found, Found::File) {
        return Err(DiskError {
            kind: ErrorKind::InvalidData,
            message: "File present but invalid".into(),
        });
    }

    // O4 - nothing found.
    Err(DiskError {
        kind: ErrorKind::NotFound,
        message: "File not found".into(),
    })
}

/// Whether an interrupted save left its ".saving" file (rule I1).
pub fn interrupted(disk: &mut impl Disk, target: &str) -> bool {
    matches!(disk.found(&temp_name(target)), Found::File)
}
