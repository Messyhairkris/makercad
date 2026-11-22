//! Contract tests for mk_project::atomic, written by Gordon as a PA (the Crew) from
//! crew/contracts/mk_atomic.md v2. They were the hidden tests of campaign mk-core-7.

// CONTRACT NOTE: worked-example row 10 says the backup is "garbage" after the failed rename
// of row 9. Rule S3 copies `target` to the backup before the rename, so from row 9 on the
// backup is "four". This file follows S3; see `worked_example_every_row_in_order`.

use mk_project::atomic::{
    backup_name, interrupted, open, save, temp_name, Disk, DiskError, ErrorKind, Found, Opened,
};

use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};

/// A pretend disk: files, folders, a log of every call, and failures armed by call number.
#[derive(Debug, Default)]
struct FakeDisk {
    files: BTreeMap<String, Vec<u8>>,
    folders: BTreeSet<String>,
    log: Vec<String>,
    seen: BTreeMap<&'static str, usize>,
    failures: Vec<(&'static str, usize)>,
}

impl FakeDisk {
    fn new() -> Self {
        Self::default()
    }

    fn put(&mut self, name: &str, bytes: &[u8]) {
        self.files.insert(name.to_string(), bytes.to_vec());
    }

    fn put_folder(&mut self, name: &str) {
        self.folders.insert(name.to_string());
    }

    fn file(&self, name: &str) -> Option<Vec<u8>> {
        self.files.get(name).cloned()
    }

    fn has_file(&self, name: &str) -> bool {
        self.files.contains_key(name)
    }

    fn calls(&self) -> Vec<String> {
        self.log.clone()
    }

    fn calls_of(&self, kind: &str) -> usize {
        self.seen.get(kind).copied().unwrap_or(0)
    }

    /// Fail the next call of `kind`, counted over every call of that kind on this disk.
    fn fail_next(&mut self, kind: &'static str) {
        let at = self.calls_of(kind) + 1;
        self.failures.push((kind, at));
    }

    /// Fail the `n`th call of `kind` (1 is the first call on this disk).
    fn fail_nth(&mut self, kind: &'static str, n: usize) {
        self.failures.push((kind, n));
    }

    fn saw(&self, line: &str) -> bool {
        self.log.iter().any(|entry| entry == line)
    }

    /// Record one call and say whether it is the armed failure. A failure fires once.
    fn bump(&mut self, kind: &'static str, line: String) -> Option<DiskError> {
        let count = self.seen.entry(kind).or_insert(0);
        *count += 1;
        let count = *count;
        self.log.push(line);
        let at = self
            .failures
            .iter()
            .position(|&(k, n)| k == kind && n == count)?;
        let _ = self.failures.remove(at);
        Some(failure(kind, count))
    }
}

impl Disk for FakeDisk {
    fn found(&mut self, name: &str) -> Found {
        let _ = self.bump("found", format!("found {name}"));
        if self.files.contains_key(name) {
            Found::File
        } else if self.folders.contains(name) {
            Found::Folder
        } else {
            Found::Missing
        }
    }

    fn read(&mut self, name: &str) -> Result<Vec<u8>, DiskError> {
        if let Some(failed) = self.bump("read", format!("read {name}")) {
            return Err(failed);
        }
        match self.files.get(name) {
            Some(bytes) => Ok(bytes.clone()),
            None => Err(not_found(name)),
        }
    }

    fn write(&mut self, name: &str, bytes: &[u8]) -> Result<(), DiskError> {
        if let Some(failed) = self.bump("write", format!("write {name}")) {
            return Err(failed);
        }
        self.files.insert(name.to_string(), bytes.to_vec());
        Ok(())
    }

    fn copy(&mut self, from: &str, to: &str) -> Result<(), DiskError> {
        if let Some(failed) = self.bump("copy", format!("copy {from} {to}")) {
            return Err(failed);
        }
        match self.files.get(from).cloned() {
            Some(bytes) => {
                self.files.insert(to.to_string(), bytes);
                Ok(())
            }
            None => Err(not_found(from)),
        }
    }

    fn rename(&mut self, from: &str, to: &str) -> Result<(), DiskError> {
        if let Some(failed) = self.bump("rename", format!("rename {from} {to}")) {
            return Err(failed);
        }
        match self.files.remove(from) {
            Some(bytes) => {
                self.files.insert(to.to_string(), bytes);
                Ok(())
            }
            None => Err(not_found(from)),
        }
    }

    fn remove(&mut self, name: &str) -> Result<(), DiskError> {
        if let Some(failed) = self.bump("remove", format!("remove {name}")) {
            return Err(failed);
        }
        match self.files.remove(name) {
            Some(_) => Ok(()),
            None => Err(not_found(name)),
        }
    }
}

/// The error the pretend disk reports when it is told to fail.
fn failure(kind: &str, n: usize) -> DiskError {
    DiskError {
        kind: ErrorKind::Other,
        message: format!("{kind} {n} failed"),
    }
}

fn not_found(name: &str) -> DiskError {
    DiskError {
        kind: ErrorKind::NotFound,
        message: format!("{name} is not a file"),
    }
}

fn lines(items: &[&str]) -> Vec<String> {
    items.iter().map(|item| item.to_string()).collect()
}

/// The contract's last paragraph: a save names the target only as the source of the copy
/// and as the destination of the one rename.
fn assert_save_only_renames_the_target(d: &FakeDisk, target: &str) {
    let write = format!("write {target}");
    let remove = format!("remove {target}");
    let remove_backup = format!("remove {target}.backup");
    let copy_onto = format!(" {target}");
    for line in d.calls() {
        assert_ne!(line, write, "save wrote the target");
        assert_ne!(line, remove, "save removed the target");
        assert_ne!(line, remove_backup, "save removed the backup");
        if let Some(rest) = line.strip_prefix("copy ") {
            assert!(
                !rest.ends_with(&copy_onto),
                "save copied onto the target: {line}"
            );
        }
        if let Some(rest) = line.strip_prefix("rename ") {
            let (from, to) = rest.split_once(' ').expect("a rename has two names");
            assert_eq!(from, format!("{target}.saving").as_str());
            assert_eq!(to, target);
        }
    }
}

// ---- names --------------------------------------------------------------------

#[test]
fn temp_name_appends_the_suffix_to_the_whole_name() {
    assert_eq!(temp_name("dresser.maker"), "dresser.maker.saving");
    assert_eq!(temp_name("notes"), "notes.saving");
}

#[test]
fn backup_name_appends_the_suffix_to_the_whole_name() {
    assert_eq!(backup_name("dresser.maker"), "dresser.maker.backup");
    assert_eq!(backup_name("notes"), "notes.backup");
}

#[test]
fn temp_and_backup_name_of_an_empty_name() {
    assert_eq!(temp_name(""), ".saving");
    assert_eq!(backup_name(""), ".backup");
}

#[test]
fn a_name_without_an_extension_is_saved_like_any_other() {
    let mut d = FakeDisk::new();
    assert_eq!(save(&mut d, "notes", b"a"), Ok(()));
    assert_eq!(save(&mut d, "notes", b"b"), Ok(()));
    assert_eq!(d.file("notes"), Some(b"b".to_vec()));
    assert_eq!(d.file("notes.backup"), Some(b"a".to_vec()));
}

// ---- save: S1 ----------------------------------------------------------------

#[test]
fn rule_s1_an_empty_name_is_refused_before_any_disk_call() {
    let mut d = FakeDisk::new();
    let got = save(&mut d, "", b"x").unwrap_err();
    assert_eq!(got.kind, ErrorKind::InvalidInput);
    assert!(d.calls().is_empty(), "a refused save called the disk");
}

#[test]
fn rule_s1_a_forward_slash_is_refused_before_any_disk_call() {
    let mut d = FakeDisk::new();
    let got = save(&mut d, "a/b", b"x").unwrap_err();
    assert_eq!(got.kind, ErrorKind::InvalidInput);
    assert!(d.calls().is_empty(), "a refused save called the disk");
}

#[test]
fn rule_s1_a_backslash_is_refused_before_any_disk_call() {
    let mut d = FakeDisk::new();
    let got = save(&mut d, "a\\b", b"x").unwrap_err();
    assert_eq!(got.kind, ErrorKind::InvalidInput);
    assert!(d.calls().is_empty(), "a refused save called the disk");
}

#[test]
fn rule_s1_a_folder_at_the_target_is_refused_after_only_found() {
    let mut d = FakeDisk::new();
    d.put_folder("dresser.maker");
    let got = save(&mut d, "dresser.maker", b"x").unwrap_err();
    assert_eq!(got.kind, ErrorKind::InvalidInput);
    assert_eq!(d.calls(), lines(&["found dresser.maker"]));
}

// ---- save: S2-S5 -------------------------------------------------------------

#[test]
fn rule_s2_a_failing_write_removes_the_temp_and_returns_the_error() {
    let mut d = FakeDisk::new();
    d.put("dresser.maker", b"one");
    d.fail_next("write");
    let want = failure("write", d.calls_of("write") + 1);
    assert_eq!(save(&mut d, "dresser.maker", b"two"), Err(want));
    assert_eq!(d.file("dresser.maker"), Some(b"one".to_vec()));
    assert!(!d.has_file("dresser.maker.saving"));
    assert_eq!(
        d.calls(),
        lines(&[
            "found dresser.maker",
            "write dresser.maker.saving",
            "remove dresser.maker.saving",
        ])
    );
}

#[test]
fn rule_s3_a_failing_copy_removes_the_temp_and_returns_the_error() {
    let mut d = FakeDisk::new();
    d.put("dresser.maker", b"one");
    d.put("dresser.maker.backup", b"zero");
    d.fail_next("copy");
    let want = failure("copy", d.calls_of("copy") + 1);
    assert_eq!(save(&mut d, "dresser.maker", b"two"), Err(want));
    assert_eq!(d.file("dresser.maker"), Some(b"one".to_vec()));
    assert_eq!(d.file("dresser.maker.backup"), Some(b"zero".to_vec()));
    assert!(!d.has_file("dresser.maker.saving"));
    assert_eq!(
        d.calls(),
        lines(&[
            "found dresser.maker",
            "write dresser.maker.saving",
            "found dresser.maker",
            "copy dresser.maker dresser.maker.backup",
            "remove dresser.maker.saving",
        ])
    );
}

#[test]
fn rule_s4_a_failing_rename_removes_the_temp_and_returns_the_error() {
    let mut d = FakeDisk::new();
    d.put("dresser.maker", b"one");
    d.fail_next("rename");
    let want = failure("rename", d.calls_of("rename") + 1);
    assert_eq!(save(&mut d, "dresser.maker", b"two"), Err(want));
    assert_eq!(d.file("dresser.maker"), Some(b"one".to_vec()));
    assert!(!d.has_file("dresser.maker.saving"));
    assert_eq!(d.file("dresser.maker.backup"), Some(b"one".to_vec()));
    assert_eq!(
        d.calls(),
        lines(&[
            "found dresser.maker",
            "write dresser.maker.saving",
            "found dresser.maker",
            "copy dresser.maker dresser.maker.backup",
            "rename dresser.maker.saving dresser.maker",
            "remove dresser.maker.saving",
        ])
    );
}

#[test]
fn rule_s5_a_first_save_has_no_backup() {
    let mut d = FakeDisk::new();
    assert_eq!(save(&mut d, "dresser.maker", b"one"), Ok(()));
    assert_eq!(d.file("dresser.maker"), Some(b"one".to_vec()));
    assert_eq!(d.file("dresser.maker.backup"), None);
    assert!(!d.has_file("dresser.maker.saving"));
}

#[test]
fn rule_s5_a_later_save_keeps_exactly_the_previous_save() {
    let mut d = FakeDisk::new();
    assert_eq!(save(&mut d, "dresser.maker", b"one"), Ok(()));
    assert_eq!(save(&mut d, "dresser.maker", b"two"), Ok(()));
    assert_eq!(d.file("dresser.maker"), Some(b"two".to_vec()));
    assert_eq!(d.file("dresser.maker.backup"), Some(b"one".to_vec()));
    assert!(!d.has_file("dresser.maker.saving"));
    assert_eq!(save(&mut d, "dresser.maker", b"three"), Ok(()));
    assert_eq!(d.file("dresser.maker"), Some(b"three".to_vec()));
    assert_eq!(d.file("dresser.maker.backup"), Some(b"two".to_vec()));
}

// ---- save: call order and the promise ---------------------------------------

#[test]
fn save_call_log_of_a_first_save() {
    let mut d = FakeDisk::new();
    assert_eq!(save(&mut d, "dresser.maker", b"one"), Ok(()));
    assert_eq!(
        d.calls(),
        lines(&[
            "found dresser.maker",
            "write dresser.maker.saving",
            "found dresser.maker",
            "rename dresser.maker.saving dresser.maker",
        ])
    );
}

#[test]
fn save_call_log_of_a_later_save() {
    let mut d = FakeDisk::new();
    d.put("dresser.maker", b"one");
    assert_eq!(save(&mut d, "dresser.maker", b"two"), Ok(()));
    assert_eq!(
        d.calls(),
        lines(&[
            "found dresser.maker",
            "write dresser.maker.saving",
            "found dresser.maker",
            "copy dresser.maker dresser.maker.backup",
            "rename dresser.maker.saving dresser.maker",
        ])
    );
}

#[test]
fn save_only_renames_the_target_and_never_removes_the_backup() {
    let mut d = FakeDisk::new();
    d.put("dresser.maker", b"one");
    d.put("dresser.maker.backup", b"zero");
    assert_eq!(save(&mut d, "dresser.maker", b"two"), Ok(()));
    assert_save_only_renames_the_target(&d, "dresser.maker");
    assert!(!d.saw("remove dresser.maker.backup"));
}

#[test]
fn the_promise_holds_when_each_mutating_call_fails() {
    let target = "dresser.maker";
    for kind in ["write", "copy", "rename"] {
        let mut d = FakeDisk::new();
        d.put(target, b"old");
        d.put("dresser.maker.backup", b"older");
        d.fail_next(kind);
        let want = failure(kind, d.calls_of(kind) + 1);
        assert_eq!(
            save(&mut d, target, b"new"),
            Err(want),
            "a failing {kind} returned other"
        );
        assert_eq!(
            d.file(target),
            Some(b"old".to_vec()),
            "a failing {kind} changed target"
        );
        assert!(
            !d.has_file("dresser.maker.saving"),
            "a failing {kind} left a .saving file"
        );
        assert_save_only_renames_the_target(&d, target);
    }
}

#[test]
fn a_failing_remove_after_a_failed_rename_keeps_the_rename_error() {
    let mut d = FakeDisk::new();
    d.put("dresser.maker", b"old");
    d.fail_next("rename");
    d.fail_next("remove");
    let want = failure("rename", d.calls_of("rename") + 1);
    assert_eq!(save(&mut d, "dresser.maker", b"new"), Err(want));
    assert_eq!(d.file("dresser.maker"), Some(b"old".to_vec()));
    assert!(
        d.saw("remove dresser.maker.saving"),
        "the clean-up did not run"
    );
    assert!(
        d.has_file("dresser.maker.saving"),
        "the armed remove did not fire"
    );
}

#[test]
fn a_failing_remove_after_a_failed_write_keeps_the_write_error() {
    let mut d = FakeDisk::new();
    d.put("dresser.maker", b"old");
    d.fail_next("write");
    d.fail_next("remove");
    let want = failure("write", d.calls_of("write") + 1);
    assert_eq!(save(&mut d, "dresser.maker", b"new"), Err(want));
    assert_eq!(d.file("dresser.maker"), Some(b"old".to_vec()));
    assert!(
        d.saw("remove dresser.maker.saving"),
        "the clean-up did not run"
    );
}

// ---- open: O1-O4 -------------------------------------------------------------

#[test]
fn rule_o1_open_returns_the_current_file_whole() {
    let mut d = FakeDisk::new();
    d.put("dresser.maker", b"whole");
    d.put("dresser.maker.backup", b"older");
    let checks = Cell::new(0);
    let got = open(&mut d, "dresser.maker", |bytes| {
        checks.set(checks.get() + 1);
        bytes == b"whole".as_slice()
    });
    assert_eq!(got, Ok(Opened::Current(b"whole".to_vec())));
    assert_eq!(checks.get(), 1);
}

#[test]
fn rule_o1_the_current_file_wins_over_the_backup() {
    let mut d = FakeDisk::new();
    d.put("dresser.maker", b"new");
    d.put("dresser.maker.backup", b"old");
    assert_eq!(
        open(&mut d, "dresser.maker", |_| true),
        Ok(Opened::Current(b"new".to_vec()))
    );
}

#[test]
fn rule_o2_open_recovers_from_the_backup() {
    let mut d = FakeDisk::new();
    d.put("dresser.maker", b"garbage");
    d.put("dresser.maker.backup", b"two");
    let checks = Cell::new(0);
    let got = open(&mut d, "dresser.maker", |bytes| {
        checks.set(checks.get() + 1);
        bytes == b"two".as_slice()
    });
    assert_eq!(got, Ok(Opened::Recovered(b"two".to_vec())));
    assert_eq!(
        checks.get(),
        2,
        "the target and the backup were each checked once"
    );
}

#[test]
fn rule_o2_a_missing_target_is_read_from_the_backup() {
    let mut d = FakeDisk::new();
    d.put("dresser.maker.backup", b"two");
    let got = open(&mut d, "dresser.maker", |bytes| bytes == b"two".as_slice());
    assert_eq!(got, Ok(Opened::Recovered(b"two".to_vec())));
}

#[test]
fn rule_o2_a_folder_at_the_target_falls_back_to_the_backup() {
    let mut d = FakeDisk::new();
    d.put_folder("dresser.maker");
    d.put("dresser.maker.backup", b"two");
    let got = open(&mut d, "dresser.maker", |bytes| bytes == b"two".as_slice());
    assert_eq!(got, Ok(Opened::Recovered(b"two".to_vec())));
}

#[test]
fn rule_o3_damaged_current_and_damaged_backup_is_invalid_data() {
    let mut d = FakeDisk::new();
    d.put("dresser.maker", b"garbage");
    d.put("dresser.maker.backup", b"rubbish");
    let got = open(&mut d, "dresser.maker", |_| false).unwrap_err();
    assert_eq!(got.kind, ErrorKind::InvalidData);
}

#[test]
fn rule_o3_a_failing_read_is_still_something_there() {
    let mut d = FakeDisk::new();
    d.put("dresser.maker", b"garbage");
    d.fail_nth("read", 1);
    let got = open(&mut d, "dresser.maker", |_| true).unwrap_err();
    assert_eq!(got.kind, ErrorKind::InvalidData);
}

#[test]
fn rule_o4_nothing_there_is_not_found() {
    let mut d = FakeDisk::new();
    let got = open(&mut d, "dresser.maker", |_| true).unwrap_err();
    assert_eq!(got.kind, ErrorKind::NotFound);
}

#[test]
fn rule_o4_a_folder_at_the_target_is_not_a_file() {
    let mut d = FakeDisk::new();
    d.put_folder("dresser.maker");
    let got = open(&mut d, "dresser.maker", |_| true).unwrap_err();
    assert_eq!(got.kind, ErrorKind::NotFound);
}

// ---- open: call discipline ---------------------------------------------------

#[test]
fn open_never_calls_a_mutating_method() {
    let mut d = FakeDisk::new();
    d.put("dresser.maker", b"garbage");
    d.put("dresser.maker.backup", b"two");
    d.put("dresser.maker.saving", b"half");
    let _ = open(&mut d, "dresser.maker", |_| false);
    for line in d.calls() {
        assert!(
            line.starts_with("found ") || line.starts_with("read "),
            "open called {line}"
        );
    }
}

#[test]
fn open_never_reads_the_saving_file() {
    let mut d = FakeDisk::new();
    d.put("dresser.maker", b"garbage");
    d.put("dresser.maker.saving", b"half");
    let _ = open(&mut d, "dresser.maker", |_| false);
    assert!(!d.saw("read dresser.maker.saving"));
}

#[test]
fn open_calls_check_at_most_once_per_file_read() {
    let mut d = FakeDisk::new();
    d.put("dresser.maker", b"damaged");
    d.put("dresser.maker.backup", b"whole");
    let checks = Cell::new(0);
    let got = open(&mut d, "dresser.maker", |bytes| {
        checks.set(checks.get() + 1);
        bytes == b"whole".as_slice()
    });
    assert_eq!(got, Ok(Opened::Recovered(b"whole".to_vec())));
    assert_eq!(checks.get(), 2, "one check per file read");
}

#[test]
fn open_treats_a_failing_read_as_damaged() {
    let mut d = FakeDisk::new();
    d.put("dresser.maker", b"damaged");
    d.put("dresser.maker.backup", b"whole");
    d.fail_nth("read", 1);
    let checks = Cell::new(0);
    let got = open(&mut d, "dresser.maker", |bytes| {
        checks.set(checks.get() + 1);
        bytes == b"whole".as_slice()
    });
    assert_eq!(got, Ok(Opened::Recovered(b"whole".to_vec())));
    assert_eq!(checks.get(), 1, "a failed read is not checked");
}

// ---- interrupted: I1 ---------------------------------------------------------

#[test]
fn rule_i1_interrupted_is_true_only_for_a_file_at_the_saving_name() {
    let mut d = FakeDisk::new();
    assert!(!interrupted(&mut d, "dresser.maker"));
    d.put_folder("dresser.maker.saving");
    assert!(
        !interrupted(&mut d, "dresser.maker"),
        "a folder is not an interrupted save"
    );
    d.put("dresser.maker.saving", b"fou");
    assert!(interrupted(&mut d, "dresser.maker"));
}

#[test]
fn rule_i1_a_successful_save_clears_it() {
    let mut d = FakeDisk::new();
    d.put("dresser.maker", b"one");
    d.put("dresser.maker.saving", b"hal");
    assert!(interrupted(&mut d, "dresser.maker"));
    assert_eq!(save(&mut d, "dresser.maker", b"two"), Ok(()));
    assert!(!interrupted(&mut d, "dresser.maker"));
    assert_eq!(d.file("dresser.maker"), Some(b"two".to_vec()));
    assert_eq!(d.file("dresser.maker.backup"), Some(b"one".to_vec()));
}

// ---- the worked example ------------------------------------------------------

#[test]
fn worked_example_every_row_in_order() {
    let mut d = FakeDisk::new();
    let t = "dresser.maker";

    // 1
    assert_eq!(save(&mut d, t, b"one"), Ok(()));
    assert_eq!(d.file(t), Some(b"one".to_vec()));
    assert_eq!(d.file("dresser.maker.backup"), None);
    assert!(!d.has_file("dresser.maker.saving"));

    // 2
    assert_eq!(save(&mut d, t, b"two"), Ok(()));
    assert_eq!(d.file(t), Some(b"two".to_vec()));
    assert_eq!(d.file("dresser.maker.backup"), Some(b"one".to_vec()));

    // 3
    assert_eq!(save(&mut d, t, b"three"), Ok(()));
    assert_eq!(d.file(t), Some(b"three".to_vec()));
    assert_eq!(d.file("dresser.maker.backup"), Some(b"two".to_vec()));

    // 4: someone writes garbage over the target.
    d.put(t, b"garbage");

    // 5
    assert_eq!(
        open(&mut d, t, |bytes| bytes.starts_with(b"t")),
        Ok(Opened::Recovered(b"two".to_vec()))
    );
    assert_eq!(
        d.file(t),
        Some(b"garbage".to_vec()),
        "open changed the target"
    );
    assert_eq!(d.file("dresser.maker.backup"), Some(b"two".to_vec()));

    // 6: the caller's check decides what "whole" means.
    assert_eq!(
        open(&mut d, t, |_| true),
        Ok(Opened::Current(b"garbage".to_vec()))
    );

    // 7: a crash left a temp file; open still reads the target.
    d.put("dresser.maker.saving", b"fou");
    assert!(interrupted(&mut d, t));
    assert_eq!(
        open(&mut d, t, |_| true),
        Ok(Opened::Current(b"garbage".to_vec()))
    );

    // 8
    assert_eq!(save(&mut d, t, b"four"), Ok(()));
    assert_eq!(d.file(t), Some(b"four".to_vec()));
    assert_eq!(d.file("dresser.maker.backup"), Some(b"garbage".to_vec()));
    assert!(!interrupted(&mut d, t));

    // 9: the next rename fails. S3 copied before it, so the backup is now "four".
    d.fail_next("rename");
    let want = failure("rename", d.calls_of("rename") + 1);
    assert_eq!(save(&mut d, t, b"five"), Err(want));
    assert_eq!(d.file(t), Some(b"four".to_vec()));
    assert_eq!(d.file("dresser.maker.backup"), Some(b"four".to_vec()));
    assert!(!interrupted(&mut d, t));

    // 10: the next write fails.
    d.fail_next("write");
    let want = failure("write", d.calls_of("write") + 1);
    assert_eq!(save(&mut d, t, b"five"), Err(want));
    assert_eq!(d.file(t), Some(b"four".to_vec()));
    // CONTRACT NOTE: row 10 says "garbage"; S3 copied "four" at row 9, so the rule says "four".
    assert_eq!(d.file("dresser.maker.backup"), Some(b"four".to_vec()));

    // 11: a refused name makes no disk call at all.
    let before = d.calls().len();
    assert_eq!(
        save(&mut d, "", b"x").unwrap_err().kind,
        ErrorKind::InvalidInput
    );
    assert_eq!(
        save(&mut d, "a/b", b"x").unwrap_err().kind,
        ErrorKind::InvalidInput
    );
    assert_eq!(d.calls().len(), before, "a refused save called the disk");
}

// ---- sizes -------------------------------------------------------------------

#[test]
fn empty_bytes_are_a_whole_file() {
    let mut d = FakeDisk::new();
    assert_eq!(save(&mut d, "notes", b""), Ok(()));
    assert_eq!(d.file("notes"), Some(Vec::new()));
    let checks = Cell::new(0);
    let got = open(&mut d, "notes", |bytes| {
        checks.set(checks.get() + 1);
        bytes.is_empty()
    });
    assert_eq!(got, Ok(Opened::Current(Vec::new())));
    assert_eq!(checks.get(), 1);
}

#[test]
fn one_megabyte_round_trips() {
    let bytes: Vec<u8> = (0..1_048_576).map(|i| (i % 251) as u8).collect();
    let mut d = FakeDisk::new();
    assert_eq!(save(&mut d, "big.maker", &bytes), Ok(()));
    assert_eq!(d.file("big.maker"), Some(bytes.clone()));
    assert_eq!(
        open(&mut d, "big.maker", |whole| whole == bytes.as_slice()),
        Ok(Opened::Current(bytes.clone()))
    );
    assert_eq!(save(&mut d, "big.maker", b"tiny"), Ok(()));
    assert_eq!(d.file("big.maker.backup"), Some(bytes));
}
