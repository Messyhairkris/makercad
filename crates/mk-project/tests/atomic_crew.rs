//! Tests for mk_project::atomic written by Deepak (the Crew, campaign mk-core-7) from the contract
//! alone, without seeing the module.

//! Integration tests for `mk_project::atomic` (contract v2): a pretend disk in memory.

use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};

use mk_project::atomic::{
    backup_name, interrupted, open, save, temp_name, Disk, DiskError, ErrorKind, Found, Opened,
};

/// A pretend disk: files in a map, folders in a set, every call in a log, and the ability to
/// fail the Nth call of a kind.
struct FakeDisk {
    files: BTreeMap<String, Vec<u8>>,
    folders: BTreeSet<String>,
    log: Vec<String>,
    /// (kind, nth call of that kind) pairs that fail, 1-based.
    fail: Vec<(String, usize)>,
    counts: BTreeMap<String, usize>,
}

impl FakeDisk {
    fn new() -> Self {
        FakeDisk {
            files: BTreeMap::new(),
            folders: BTreeSet::new(),
            log: Vec::new(),
            fail: Vec::new(),
            counts: BTreeMap::new(),
        }
    }

    fn file(&self, name: &str) -> Option<&[u8]> {
        self.files.get(name).map(|b| b.as_slice())
    }

    fn has(&self, name: &str) -> bool {
        self.files.contains_key(name)
    }

    /// Fail the next call of `kind`.
    fn arm_next(&mut self, kind: &str) {
        let n = *self.counts.get(kind).unwrap_or(&0) + 1;
        self.fail.push((kind.to_string(), n));
    }

    fn maybe_fail(&mut self, kind: &str) -> Result<(), DiskError> {
        let n = self.counts.entry(kind.to_string()).or_insert(0);
        *n += 1;
        let n = *n;
        if self.fail.iter().any(|(k, nth)| k == kind && *nth == n) {
            return Err(DiskError {
                kind: ErrorKind::Other,
                message: format!("{kind} failed"),
            });
        }
        Ok(())
    }
}

impl Disk for FakeDisk {
    fn found(&mut self, name: &str) -> Found {
        self.log.push(format!("found({name})"));
        if self.folders.contains(name) {
            Found::Folder
        } else if self.files.contains_key(name) {
            Found::File
        } else {
            Found::Missing
        }
    }

    fn read(&mut self, name: &str) -> Result<Vec<u8>, DiskError> {
        self.log.push(format!("read({name})"));
        self.maybe_fail("read")?;
        match self.files.get(name) {
            Some(b) => Ok(b.clone()),
            None => Err(DiskError {
                kind: ErrorKind::NotFound,
                message: format!("no {name}"),
            }),
        }
    }

    fn write(&mut self, name: &str, bytes: &[u8]) -> Result<(), DiskError> {
        self.log.push(format!("write({name})"));
        self.maybe_fail("write")?;
        self.files.insert(name.to_string(), bytes.to_vec());
        Ok(())
    }

    fn copy(&mut self, from: &str, to: &str) -> Result<(), DiskError> {
        self.log.push(format!("copy({from}, {to})"));
        self.maybe_fail("copy")?;
        match self.files.get(from) {
            Some(b) => {
                let b = b.clone();
                self.files.insert(to.to_string(), b);
                Ok(())
            }
            None => Err(DiskError {
                kind: ErrorKind::NotFound,
                message: format!("no {from}"),
            }),
        }
    }

    fn rename(&mut self, from: &str, to: &str) -> Result<(), DiskError> {
        self.log.push(format!("rename({from}, {to})"));
        self.maybe_fail("rename")?;
        match self.files.remove(from) {
            Some(b) => {
                self.files.insert(to.to_string(), b);
                Ok(())
            }
            None => Err(DiskError {
                kind: ErrorKind::NotFound,
                message: format!("no {from}"),
            }),
        }
    }

    fn remove(&mut self, name: &str) -> Result<(), DiskError> {
        self.log.push(format!("remove({name})"));
        self.maybe_fail("remove")?;
        self.files.remove(name);
        Ok(())
    }
}

fn disk_error(kind: ErrorKind, message: &str) -> DiskError {
    DiskError {
        kind,
        message: message.to_string(),
    }
}

#[test]
fn temp_name_and_backup_name_append_the_suffix() {
    assert_eq!(temp_name("dresser.maker"), "dresser.maker.saving");
    assert_eq!(backup_name("dresser.maker"), "dresser.maker.backup");
    assert_eq!(temp_name("notes"), "notes.saving");
    assert_eq!(backup_name("notes"), "notes.backup");
}

#[test]
fn worked_example_rows_1_to_11_in_order() {
    let t = "dresser.maker";
    let mut d = FakeDisk::new();

    // row 1
    assert_eq!(save(&mut d, t, b"one"), Ok(()));
    assert_eq!(d.file(t), Some(&b"one"[..]));
    assert!(!d.has(&backup_name(t)));
    assert!(!d.has(&temp_name(t)));

    // row 2
    assert_eq!(save(&mut d, t, b"two"), Ok(()));
    assert_eq!(d.file(t), Some(&b"two"[..]));
    assert_eq!(d.file(&backup_name(t)), Some(&b"one"[..]));

    // row 3
    assert_eq!(save(&mut d, t, b"three"), Ok(()));
    assert_eq!(d.file(t), Some(&b"three"[..]));
    assert_eq!(d.file(&backup_name(t)), Some(&b"two"[..]));

    // row 4: someone writes garbage over the target
    d.files.insert(t.to_string(), b"garbage".to_vec());

    // row 5: the target fails check, the backup passes it
    assert_eq!(
        open(&mut d, t, |b| b.starts_with(b"t")),
        Ok(Opened::Recovered(b"two".to_vec()))
    );
    assert_eq!(d.file(t), Some(&b"garbage"[..]));
    assert_eq!(d.file(&backup_name(t)), Some(&b"two"[..]));

    // row 6: check decides what "whole" means
    assert_eq!(
        open(&mut d, t, |_| true),
        Ok(Opened::Current(b"garbage".to_vec()))
    );

    // row 7: a crash left a .saving file
    d.files.insert(temp_name(t), b"fou".to_vec());
    assert!(interrupted(&mut d, t));
    assert_eq!(
        open(&mut d, t, |_| true),
        Ok(Opened::Current(b"garbage".to_vec()))
    );

    // row 8
    assert_eq!(save(&mut d, t, b"four"), Ok(()));
    assert_eq!(d.file(t), Some(&b"four"[..]));
    assert_eq!(d.file(&backup_name(t)), Some(&b"garbage"[..]));
    assert!(!d.has(&temp_name(t)));
    assert!(!interrupted(&mut d, t));

    // row 9: the disk fails the next rename
    d.arm_next("rename");
    assert_eq!(
        save(&mut d, t, b"five"),
        Err(disk_error(ErrorKind::Other, "rename failed"))
    );
    assert_eq!(d.file(t), Some(&b"four"[..]));
    assert!(!d.has(&temp_name(t)));

    // row 10: the disk fails the next write; row 9's copy had already run
    d.arm_next("write");
    assert_eq!(
        save(&mut d, t, b"five"),
        Err(disk_error(ErrorKind::Other, "write failed"))
    );
    assert_eq!(d.file(t), Some(&b"four"[..]));
    assert_eq!(d.file(&backup_name(t)), Some(&b"four"[..]));

    // row 11: bad names never reach the disk
    d.log.clear();
    assert_eq!(
        save(&mut d, "", b"x").unwrap_err().kind,
        ErrorKind::InvalidInput
    );
    assert_eq!(
        save(&mut d, "a/b", b"x").unwrap_err().kind,
        ErrorKind::InvalidInput
    );
    assert!(d.log.is_empty());
}

#[test]
fn rule_s1_empty_target_is_invalid_input_without_a_disk_call() {
    let mut d = FakeDisk::new();
    let e = save(&mut d, "", b"x").unwrap_err();
    assert_eq!(e.kind, ErrorKind::InvalidInput);
    assert!(d.log.is_empty());
}

#[test]
fn rule_s1_slash_or_backslash_is_invalid_input_without_a_disk_call() {
    for bad in ["a/b", "a\\b", "/", "\\", "dir/", "dir\\file"] {
        let mut d = FakeDisk::new();
        let e = save(&mut d, bad, b"x").unwrap_err();
        assert_eq!(e.kind, ErrorKind::InvalidInput, "target {bad}");
        assert!(d.log.is_empty(), "no disk call for {bad}");
    }
}

#[test]
fn rule_s1_folder_at_target_is_invalid_input_without_another_call() {
    let t = "dresser.maker";
    let mut d = FakeDisk::new();
    d.folders.insert(t.to_string());
    let e = save(&mut d, t, b"x").unwrap_err();
    assert_eq!(e.kind, ErrorKind::InvalidInput);
    assert_eq!(d.log, vec![format!("found({t})")]);
}

#[test]
fn rule_s2_write_failure_removes_temp_and_returns_the_write_error() {
    let t = "dresser.maker";
    let mut d = FakeDisk::new();
    d.files.insert(t.to_string(), b"old".to_vec());
    d.arm_next("write");
    let e = save(&mut d, t, b"new").unwrap_err();
    assert_eq!(e, disk_error(ErrorKind::Other, "write failed"));
    assert_eq!(d.file(t), Some(&b"old"[..]));
    assert!(!d.has(&temp_name(t)));
    assert_eq!(
        d.log,
        vec![
            format!("found({t})"),
            format!("write({})", temp_name(t)),
            format!("remove({})", temp_name(t)),
        ]
    );
}

#[test]
fn rule_s3_copy_failure_removes_temp_and_returns_the_copy_error() {
    let t = "dresser.maker";
    let mut d = FakeDisk::new();
    d.files.insert(t.to_string(), b"old".to_vec());
    d.arm_next("copy");
    let e = save(&mut d, t, b"new").unwrap_err();
    assert_eq!(e, disk_error(ErrorKind::Other, "copy failed"));
    assert_eq!(d.file(t), Some(&b"old"[..]));
    assert!(!d.has(&temp_name(t)));
    assert_eq!(
        d.log,
        vec![
            format!("found({t})"),
            format!("write({})", temp_name(t)),
            format!("found({t})"),
            format!("copy({t}, {})", backup_name(t)),
            format!("remove({})", temp_name(t)),
        ]
    );
}

#[test]
fn rule_s4_rename_failure_removes_temp_and_returns_the_rename_error() {
    let t = "dresser.maker";
    let mut d = FakeDisk::new();
    d.files.insert(t.to_string(), b"old".to_vec());
    d.arm_next("rename");
    let e = save(&mut d, t, b"new").unwrap_err();
    assert_eq!(e, disk_error(ErrorKind::Other, "rename failed"));
    assert_eq!(d.file(t), Some(&b"old"[..]));
    assert_eq!(d.file(&backup_name(t)), Some(&b"old"[..]));
    assert!(!d.has(&temp_name(t)));
    assert_eq!(
        d.log,
        vec![
            format!("found({t})"),
            format!("write({})", temp_name(t)),
            format!("found({t})"),
            format!("copy({t}, {})", backup_name(t)),
            format!("rename({}, {t})", temp_name(t)),
            format!("remove({})", temp_name(t)),
        ]
    );
}

#[test]
fn rule_s5_first_save_leaves_no_backup() {
    let t = "dresser.maker";
    let mut d = FakeDisk::new();
    assert_eq!(save(&mut d, t, b"one"), Ok(()));
    assert_eq!(d.file(t), Some(&b"one"[..]));
    assert!(!d.has(&backup_name(t)));
    assert!(!d.has(&temp_name(t)));
}

#[test]
fn rule_s5_later_save_backup_holds_the_previous_bytes() {
    let t = "dresser.maker";
    let mut d = FakeDisk::new();
    assert_eq!(save(&mut d, t, b"one"), Ok(()));
    assert_eq!(save(&mut d, t, b"two"), Ok(()));
    assert_eq!(d.file(t), Some(&b"two"[..]));
    assert_eq!(d.file(&backup_name(t)), Some(&b"one"[..]));
    assert_eq!(save(&mut d, t, b"three"), Ok(()));
    assert_eq!(d.file(t), Some(&b"three"[..]));
    assert_eq!(d.file(&backup_name(t)), Some(&b"two"[..]));
    assert!(!d.has(&temp_name(t)));
}

#[test]
fn rule_s5_successful_first_save_call_log() {
    let t = "dresser.maker";
    let mut d = FakeDisk::new();
    assert_eq!(save(&mut d, t, b"one"), Ok(()));
    assert_eq!(
        d.log,
        vec![
            format!("found({t})"),
            format!("write({})", temp_name(t)),
            format!("found({t})"),
            format!("rename({}, {t})", temp_name(t)),
        ]
    );
}

#[test]
fn rule_s5_successful_later_save_call_log() {
    let t = "dresser.maker";
    let mut d = FakeDisk::new();
    d.files.insert(t.to_string(), b"old".to_vec());
    assert_eq!(save(&mut d, t, b"new"), Ok(()));
    assert_eq!(
        d.log,
        vec![
            format!("found({t})"),
            format!("write({})", temp_name(t)),
            format!("found({t})"),
            format!("copy({t}, {})", backup_name(t)),
            format!("rename({}, {t})", temp_name(t)),
        ]
    );
}

#[test]
fn save_never_writes_copies_onto_or_removes_target() {
    let t = "dresser.maker";
    let mut d = FakeDisk::new();
    assert_eq!(save(&mut d, t, b"one"), Ok(()));
    assert_eq!(save(&mut d, t, b"two"), Ok(()));
    assert_eq!(save(&mut d, t, b"three"), Ok(()));
    for call in &d.log {
        assert_ne!(call, &format!("write({t})"), "wrote the target: {call}");
        assert_ne!(call, &format!("remove({t})"), "removed the target: {call}");
        assert_ne!(
            call,
            &format!("remove({})", backup_name(t)),
            "removed the backup: {call}"
        );
        if let Some(rest) = call.strip_prefix("copy(") {
            assert!(
                !rest.ends_with(&format!(", {t})")),
                "copied onto the target: {call}"
            );
        }
    }
}

#[test]
fn rule_o1_open_returns_current_when_target_is_whole() {
    let t = "dresser.maker";
    let mut d = FakeDisk::new();
    d.files.insert(t.to_string(), b"whole".to_vec());
    assert_eq!(
        open(&mut d, t, |b| b == b"whole"),
        Ok(Opened::Current(b"whole".to_vec()))
    );
}

#[test]
fn rule_o2_open_returns_recovered_when_target_is_missing() {
    let t = "dresser.maker";
    let mut d = FakeDisk::new();
    d.files.insert(backup_name(t), b"older".to_vec());
    assert_eq!(
        open(&mut d, t, |b| b == b"older"),
        Ok(Opened::Recovered(b"older".to_vec()))
    );
}

#[test]
fn rule_o3_open_returns_invalid_data_when_nothing_passes_check() {
    let t = "dresser.maker";
    let mut d = FakeDisk::new();
    d.files.insert(t.to_string(), b"junk".to_vec());
    d.files.insert(backup_name(t), b"junk2".to_vec());
    let e = open(&mut d, t, |_| false).unwrap_err();
    assert_eq!(e.kind, ErrorKind::InvalidData);
}

#[test]
fn rule_o3_open_returns_invalid_data_when_the_only_file_cannot_be_read() {
    let t = "dresser.maker";
    let mut d = FakeDisk::new();
    d.files.insert(t.to_string(), b"whole".to_vec());
    d.arm_next("read");
    let e = open(&mut d, t, |_| true).unwrap_err();
    assert_eq!(e.kind, ErrorKind::InvalidData);
}

#[test]
fn rule_o4_open_returns_not_found_when_nothing_is_there() {
    let t = "dresser.maker";
    let mut d = FakeDisk::new();
    let e = open(&mut d, t, |_| true).unwrap_err();
    assert_eq!(e.kind, ErrorKind::NotFound);
}

#[test]
fn open_never_calls_a_mutating_method() {
    let t = "dresser.maker";
    let mut d = FakeDisk::new();
    d.files.insert(t.to_string(), b"junk".to_vec());
    d.files.insert(backup_name(t), b"junk".to_vec());
    d.files.insert(temp_name(t), b"junk".to_vec());
    let _ = open(&mut d, t, |_| false);
    for call in &d.log {
        assert!(
            call.starts_with("found(") || call.starts_with("read("),
            "open made a mutating call: {call}"
        );
    }
}

#[test]
fn open_never_reads_the_saving_file() {
    let t = "dresser.maker";
    let mut d = FakeDisk::new();
    d.files.insert(temp_name(t), b"half".to_vec());
    let e = open(&mut d, t, |_| true).unwrap_err();
    assert_eq!(e.kind, ErrorKind::NotFound);
    assert!(
        !d.log
            .iter()
            .any(|c| c == &format!("read({})", temp_name(t))),
        "open read the .saving file: {:?}",
        d.log
    );
}

#[test]
fn open_calls_check_at_most_once_per_file_read() {
    let t = "dresser.maker";

    let mut d = FakeDisk::new();
    d.files.insert(t.to_string(), b"whole".to_vec());
    let n = Cell::new(0);
    let r = open(&mut d, t, |b| {
        n.set(n.get() + 1);
        b == b"whole"
    });
    assert_eq!(r, Ok(Opened::Current(b"whole".to_vec())));
    assert_eq!(n.get(), 1);

    let mut d = FakeDisk::new();
    d.files.insert(t.to_string(), b"junk".to_vec());
    d.files.insert(backup_name(t), b"whole".to_vec());
    let n = Cell::new(0);
    let r = open(&mut d, t, |b| {
        n.set(n.get() + 1);
        b == b"whole"
    });
    assert_eq!(r, Ok(Opened::Recovered(b"whole".to_vec())));
    assert_eq!(n.get(), 2);
}

#[test]
fn open_treats_a_failing_read_as_damaged() {
    let t = "dresser.maker";
    let mut d = FakeDisk::new();
    d.files.insert(t.to_string(), b"whole".to_vec());
    d.files.insert(backup_name(t), b"older".to_vec());
    d.arm_next("read");
    assert_eq!(
        open(&mut d, t, |_| true),
        Ok(Opened::Recovered(b"older".to_vec()))
    );
}

#[test]
fn open_counts_a_folder_at_target_as_not_a_file() {
    let t = "dresser.maker";
    let mut d = FakeDisk::new();
    d.folders.insert(t.to_string());
    d.files.insert(backup_name(t), b"older".to_vec());
    assert_eq!(
        open(&mut d, t, |_| true),
        Ok(Opened::Recovered(b"older".to_vec()))
    );

    let mut d = FakeDisk::new();
    d.folders.insert(t.to_string());
    let e = open(&mut d, t, |_| true).unwrap_err();
    assert_eq!(e.kind, ErrorKind::NotFound);
}

#[test]
fn rule_i1_interrupted_is_true_only_when_saving_is_a_file() {
    let t = "dresser.maker";
    let mut d = FakeDisk::new();
    assert!(!interrupted(&mut d, t));

    d.folders.insert(temp_name(t));
    assert!(!interrupted(&mut d, t));

    d.folders.remove(&temp_name(t));
    d.files.insert(temp_name(t), b"fou".to_vec());
    assert!(interrupted(&mut d, t));

    assert_eq!(save(&mut d, t, b"four"), Ok(()));
    assert!(!interrupted(&mut d, t));
}

#[test]
fn failing_remove_during_clean_up_is_ignored() {
    let t = "dresser.maker";

    // write fails, and the clean-up remove fails too: the write's error is returned
    let mut d = FakeDisk::new();
    d.files.insert(t.to_string(), b"old".to_vec());
    d.arm_next("write");
    d.arm_next("remove");
    let e = save(&mut d, t, b"new").unwrap_err();
    assert_eq!(e, disk_error(ErrorKind::Other, "write failed"));
    assert_eq!(d.file(t), Some(&b"old"[..]));

    // copy fails, and the clean-up remove fails too: the copy's error is returned
    let mut d = FakeDisk::new();
    d.files.insert(t.to_string(), b"old".to_vec());
    d.arm_next("copy");
    d.arm_next("remove");
    let e = save(&mut d, t, b"new").unwrap_err();
    assert_eq!(e, disk_error(ErrorKind::Other, "copy failed"));
    assert_eq!(d.file(t), Some(&b"old"[..]));
}

#[test]
fn promise_target_is_old_or_new_after_a_failure_at_each_step() {
    let t = "dresser.maker";
    for kind in ["write", "copy", "rename"] {
        let mut d = FakeDisk::new();
        d.files.insert(t.to_string(), b"old".to_vec());
        d.arm_next(kind);
        let e = save(&mut d, t, b"new").unwrap_err();
        assert_eq!(e, disk_error(ErrorKind::Other, &format!("{kind} failed")));
        let now = d.file(t).map(|b| b.to_vec());
        assert_eq!(now, Some(b"old".to_vec()), "{kind} must leave the old file");
        assert!(!d.has(&temp_name(t)), "{kind} left a .saving file");
    }

    // With nothing at the target, S3 never copies, so only write and rename can fail.
    for kind in ["write", "rename"] {
        let mut d = FakeDisk::new();
        d.arm_next(kind);
        let e = save(&mut d, t, b"new").unwrap_err();
        assert_eq!(e, disk_error(ErrorKind::Other, &format!("{kind} failed")));
        assert!(!d.has(t), "{kind} created the target");
        assert!(!d.has(&temp_name(t)), "{kind} left a .saving file");
    }
}

#[test]
fn save_and_open_empty_bytes() {
    let t = "dresser.maker";
    let mut d = FakeDisk::new();
    assert_eq!(save(&mut d, t, b""), Ok(()));
    assert_eq!(d.file(t), Some(&b""[..]));
    assert_eq!(open(&mut d, t, |_| true), Ok(Opened::Current(Vec::new())));

    assert_eq!(save(&mut d, t, b"x"), Ok(()));
    assert_eq!(d.file(&backup_name(t)), Some(&b""[..]));
    assert_eq!(d.file(t), Some(&b"x"[..]));
}

#[test]
fn save_and_open_one_megabyte() {
    let t = "dresser.maker";
    let big: Vec<u8> = (0..1024 * 1024).map(|i| (i % 251) as u8).collect();
    let mut d = FakeDisk::new();
    assert_eq!(save(&mut d, t, &big), Ok(()));
    assert_eq!(d.file(t), Some(big.as_slice()));
    assert_eq!(
        open(&mut d, t, |b| b.len() == big.len()),
        Ok(Opened::Current(big.clone()))
    );

    let small = b"small".to_vec();
    assert_eq!(save(&mut d, t, &small), Ok(()));
    assert_eq!(d.file(t), Some(small.as_slice()));
    assert_eq!(d.file(&backup_name(t)), Some(big.as_slice()));
}
