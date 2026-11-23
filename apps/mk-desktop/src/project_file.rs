//! Projects on disk: `.maker` files in one folder (Documents\MakerCAD), saved so that a crash
//! never leaves the maker with nothing and opened with the recovery copy as the fallback (doc 03
//! section 3.6, P1). The rules are mk-project's (`atomic`, `maker`, `container`); this module is
//! the real folder under them, and the app's view of one open project.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use mk_project::atomic::{self, Disk, DiskError, ErrorKind, Found, Opened};
use mk_project::container::Limits;
use mk_project::layout;
use mk_project::maker;
use mk_project::project::{Intent, Manifest};
use mk_units::system::System;

/// A project file's extension.
pub const EXTENSION: &str = "maker";
/// Where the project's one design is kept, in SolidCraft's own design format.
pub const DESIGN_ENTRY: &str = "designs/d1.sc";
/// The journal (doc 10 section 10.4); empty until the journal is wired in.
pub const JOURNAL_ENTRY: &str = "log/journal.jsonl";
/// What the document itself says, apart from its intent: the measuring system it keeps its
/// lengths in (D34). Additive: a project without it measures in the app's default.
pub const DOCUMENT_ENTRY: &str = "project/document.json";
/// The environment every project has.
const DESIGN_ENV: &str = "design";

/// The folder projects live in: `Documents\MakerCAD` under the user's home folder (or
/// `MakerCAD` in the home folder when there is no Documents folder).
pub fn folder() -> PathBuf {
    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let documents = home.join("Documents");
    if documents.is_dir() {
        documents.join("MakerCAD")
    } else {
        home.join("MakerCAD")
    }
}

/// The files of one real folder, for mk-project's `atomic` rules.
pub struct FolderDisk {
    dir: PathBuf,
}

impl FolderDisk {
    pub fn new(dir: &Path) -> Self {
        Self {
            dir: dir.to_path_buf(),
        }
    }
}

fn disk_error(error: std::io::Error) -> DiskError {
    let kind = match error.kind() {
        std::io::ErrorKind::NotFound => ErrorKind::NotFound,
        std::io::ErrorKind::InvalidInput => ErrorKind::InvalidInput,
        std::io::ErrorKind::InvalidData => ErrorKind::InvalidData,
        _ => ErrorKind::Other,
    };
    DiskError {
        kind,
        message: error.to_string(),
    }
}

impl Disk for FolderDisk {
    fn found(&mut self, name: &str) -> Found {
        match fs::metadata(self.dir.join(name)) {
            Ok(m) if m.is_dir() => Found::Folder,
            Ok(_) => Found::File,
            Err(_) => Found::Missing,
        }
    }

    fn read(&mut self, name: &str) -> Result<Vec<u8>, DiskError> {
        fs::read(self.dir.join(name)).map_err(disk_error)
    }

    fn write(&mut self, name: &str, bytes: &[u8]) -> Result<(), DiskError> {
        let mut file = fs::File::create(self.dir.join(name)).map_err(disk_error)?;
        file.write_all(bytes).map_err(disk_error)?;
        file.sync_all().map_err(disk_error)
    }

    fn copy(&mut self, from: &str, to: &str) -> Result<(), DiskError> {
        fs::copy(self.dir.join(from), self.dir.join(to))
            .map(|_| ())
            .map_err(disk_error)
    }

    fn rename(&mut self, from: &str, to: &str) -> Result<(), DiskError> {
        // On Windows, std's rename replaces an existing file (MoveFileExW with REPLACE_EXISTING).
        fs::rename(self.dir.join(from), self.dir.join(to)).map_err(disk_error)
    }

    fn remove(&mut self, name: &str) -> Result<(), DiskError> {
        fs::remove_file(self.dir.join(name)).map_err(disk_error)
    }
}

/// A file name for a project called `name`: its letters, digits, spaces, '-' and '_' (others
/// dropped, runs of spaces made one), never empty and never a name Windows refuses.
pub fn file_name(name: &str) -> String {
    let mut stem = String::new();
    for c in name.chars() {
        let keep = c.is_alphanumeric() || c == '-' || c == '_';
        if keep {
            stem.push(c);
        } else if c.is_whitespace() && !stem.ends_with(' ') && !stem.is_empty() {
            stem.push(' ');
        }
    }
    let stem = stem.trim().chars().take(80).collect::<String>();
    let stem = stem.trim();
    let candidate = format!("{stem}.{EXTENSION}");
    if stem.is_empty() || layout::check_name(&candidate).is_err() {
        format!("Project {stem}.{EXTENSION}").replace("Project .", "Project.")
    } else {
        candidate
    }
}

/// A file name for a new project called `name` that no file in `dir` has yet: "Desk.maker",
/// else "Desk 2.maker", "Desk 3.maker" and so on, so a new project never replaces an old one.
pub fn free_file_name(dir: &Path, name: &str) -> String {
    let first = file_name(name);
    if !dir.join(&first).exists() {
        return first;
    }
    let base = stem(&first).to_string();
    (2..10_000)
        .map(|n| file_name(&format!("{base} {n}")))
        .find(|candidate| !dir.join(candidate).exists())
        .unwrap_or(first)
}

/// One open project: the file it came from or goes to, and what it holds besides the design.
#[derive(Debug, Clone)]
pub struct Project {
    /// The file's name inside [`folder`].
    pub file: String,
    pub manifest: Manifest,
    pub intent: Intent,
    /// Every entry but the manifest, the intent and the design, kept as they were read.
    pub others: Vec<(String, Vec<u8>)>,
}

impl Project {
    /// A new project for a design called `name` (an example's name and its line as the goal).
    pub fn new(name: &str, goal: &str) -> Result<Self, String> {
        let goal = if goal.trim().is_empty() {
            "A design made in MakerCAD"
        } else {
            goal
        };
        let intent = Intent::new(name, goal).map_err(|e| format!("{e:?}"))?;
        let mut manifest = Manifest::new(env!("CARGO_PKG_VERSION"));
        manifest.environments = vec![DESIGN_ENV.into()];
        Ok(Self {
            file: file_name(name),
            manifest,
            intent,
            others: vec![(JOURNAL_ENTRY.into(), Vec::new())],
        })
    }

    /// The measuring system this document keeps its lengths in, when it says (an older project,
    /// or a file whose note cannot be read, says nothing: the app's default stands).
    pub fn units(&self) -> Option<System> {
        let (_, bytes) = self.others.iter().find(|(n, _)| n == DOCUMENT_ENTRY)?;
        let note: serde_json::Value = serde_json::from_slice(bytes).ok()?;
        System::from_key(note.get("units")?.as_str()?)
    }

    /// Sets the measuring system. Lengths are millimetres underneath, so nothing else changes:
    /// only how they are written and read. Any other thing the document note holds is kept.
    pub fn set_units(&mut self, system: System) {
        let existing = self
            .others
            .iter()
            .find(|(n, _)| n == DOCUMENT_ENTRY)
            .and_then(|(_, bytes)| serde_json::from_slice::<serde_json::Value>(bytes).ok())
            .filter(serde_json::Value::is_object);
        let mut note = existing.unwrap_or_else(|| serde_json::json!({ "version": 1 }));
        note["units"] = serde_json::Value::from(system.key());
        let bytes = serde_json::to_vec_pretty(&note).unwrap_or_default();
        match self.others.iter_mut().find(|(n, _)| n == DOCUMENT_ENTRY) {
            Some((_, old)) => *old = bytes,
            None => self.others.push((DOCUMENT_ENTRY.to_string(), bytes)),
        }
    }

    /// The bytes of the project file holding `design` (SolidCraft design bytes).
    pub fn to_bytes(&self, design: &[u8]) -> Result<Vec<u8>, String> {
        let mut entries: Vec<(&str, &[u8])> = vec![(DESIGN_ENTRY, design)];
        entries.extend(
            self.others
                .iter()
                .filter(|(n, _)| n != DESIGN_ENTRY)
                .map(|(n, b)| (n.as_str(), b.as_slice())),
        );
        maker::save(&self.manifest, &self.intent, &entries).map_err(|e| format!("{e:?}"))
    }

    /// Saves the project with `design` into `dir` (made if missing): the new file is written
    /// beside the old, the old kept as the recovery copy, then put in its place.
    pub fn save(&self, dir: &Path, design: &[u8]) -> Result<PathBuf, String> {
        let bytes = self.to_bytes(design)?;
        fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        atomic::save(&mut FolderDisk::new(dir), &self.file, &bytes).map_err(|e| e.message)?;
        Ok(dir.join(&self.file))
    }
}

/// What opening a project file gave.
#[derive(Debug)]
pub struct Loaded {
    pub project: Project,
    /// The design's SolidCraft bytes.
    pub design: Vec<u8>,
    /// The file was missing or damaged and its recovery copy was opened.
    pub recovered: bool,
    /// Entries that changed, appeared or went missing since MakerCAD last saved it.
    pub changed_outside: usize,
}

/// Reads the project bytes of a whole `.maker` file into a [`Loaded`] (not from disk).
pub fn from_bytes(file: &str, bytes: &[u8], recovered: bool) -> Result<Loaded, String> {
    let opened = maker::open(bytes, Limits::default()).map_err(|e| format!("{e:?}"))?;
    let changed_outside =
        opened.diff.changed.len() + opened.diff.extra.len() + opened.diff.missing.len();
    let mut design = None;
    let mut others = Vec::new();
    for (name, content) in opened.entries {
        if design.is_none() && layout::design_id(&name).is_some() && name.ends_with(".sc") {
            design = Some(content);
        } else {
            others.push((name, content));
        }
    }
    let design = design.ok_or("the project holds no design")?;
    Ok(Loaded {
        project: Project {
            file: file.to_string(),
            manifest: opened.manifest,
            intent: opened.intent,
            others,
        },
        design,
        recovered,
        changed_outside,
    })
}

/// Opens `file` in `dir`, or its recovery copy when the file is missing or damaged.
pub fn open(dir: &Path, file: &str) -> Result<Loaded, String> {
    let whole = |bytes: &[u8]| maker::open(bytes, Limits::default()).is_ok();
    match atomic::open(&mut FolderDisk::new(dir), file, whole) {
        Ok(Opened::Current(bytes)) => from_bytes(file, &bytes, false),
        Ok(Opened::Recovered(bytes)) => from_bytes(file, &bytes, true),
        Err(e) if e.kind == ErrorKind::NotFound => {
            Err(format!("{file} is not in {}", dir.display()))
        }
        Err(_) => {
            // Neither the file nor its copy is whole: say why the file itself is refused.
            let bytes = fs::read(dir.join(file)).map_err(|e| e.to_string())?;
            from_bytes(file, &bytes, false)
        }
    }
}

/// A project file in the folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Listed {
    pub file: String,
    pub modified: Option<SystemTime>,
}

/// The `.maker` files in `dir`, the most recently saved first (none when it does not exist).
pub fn list(dir: &Path) -> Vec<Listed> {
    let Ok(read) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut found: Vec<Listed> = read
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|x| x == EXTENSION))
        .filter(|e| e.file_type().is_ok_and(|t| t.is_file()))
        .filter_map(|e| {
            Some(Listed {
                file: e.file_name().into_string().ok()?,
                modified: e.metadata().ok().and_then(|m| m.modified().ok()),
            })
        })
        .collect();
    found.sort_by(|a, b| b.modified.cmp(&a.modified).then(a.file.cmp(&b.file)));
    found
}

/// The project's name as the file says it, without opening it ("Dresser.maker" -> "Dresser").
pub fn stem(file: &str) -> &str {
    file.strip_suffix(".maker").unwrap_or(file)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "makercad-project-test-{tag}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn a_new_project_never_takes_an_old_ones_file() {
        let dir = scratch("free");
        fs::create_dir_all(&dir).unwrap();
        assert_eq!(free_file_name(&dir, "Desk"), "Desk.maker");
        fs::write(dir.join("Desk.maker"), b"old").unwrap();
        assert_eq!(free_file_name(&dir, "Desk"), "Desk 2.maker");
        fs::write(dir.join("Desk 2.maker"), b"old").unwrap();
        assert_eq!(free_file_name(&dir, "Desk"), "Desk 3.maker");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn file_names_are_safe_and_never_empty() {
        assert_eq!(file_name("Alcove dresser"), "Alcove dresser.maker");
        assert_eq!(file_name("  Hike   pack: v2?  "), "Hike pack v2.maker");
        assert_eq!(file_name("???"), "Project.maker");
        assert_eq!(file_name("CON"), "Project CON.maker");
        assert_eq!(file_name("../../evil"), "evil.maker");
    }

    #[test]
    fn a_saved_project_opens_with_its_design_and_journal() {
        let dir = scratch("roundtrip");
        let project = Project::new("Test plate", "A plate with two holes").unwrap();
        let path = project.save(&dir, b"design bytes").unwrap();
        assert_eq!(path, dir.join("Test plate.maker"));
        let loaded = open(&dir, "Test plate.maker").unwrap();
        assert_eq!(loaded.design, b"design bytes");
        assert_eq!(loaded.project.intent.name, "Test plate");
        assert_eq!(loaded.project.manifest.environments, vec!["design"]);
        assert_eq!(
            loaded.project.others,
            vec![(JOURNAL_ENTRY.to_string(), Vec::new())]
        );
        assert!(!loaded.recovered);
        assert_eq!(loaded.changed_outside, 0);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_project_says_its_measuring_system_and_keeps_saying_it() {
        let dir = scratch("units");
        let mut project = Project::new("Shelf", "A shelf").unwrap();
        assert_eq!(project.units(), None, "a new project has not said yet");
        project.set_units(System::Imperial);
        assert_eq!(project.units(), Some(System::Imperial));
        project.set_units(System::Metric);
        assert_eq!(project.units(), Some(System::Metric));
        project.set_units(System::Imperial);
        assert_eq!(
            project
                .others
                .iter()
                .filter(|(n, _)| n == DOCUMENT_ENTRY)
                .count(),
            1,
            "setting it again replaces the note, never adds a second"
        );
        project.save(&dir, b"design").unwrap();
        let loaded = open(&dir, "Shelf.maker").unwrap();
        assert_eq!(loaded.project.units(), Some(System::Imperial));
        assert_eq!(loaded.changed_outside, 0);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_document_note_that_cannot_be_read_says_nothing_and_a_newer_ones_other_words_stay() {
        let mut project = Project::new("Shelf", "").unwrap();
        for bad in [
            &b"not json"[..],
            b"[]",
            b"{}",
            br#"{"units": 3}"#,
            br#"{"units": "cubits"}"#,
        ] {
            project.others.retain(|(n, _)| n != DOCUMENT_ENTRY);
            project.others.push((DOCUMENT_ENTRY.into(), bad.to_vec()));
            assert_eq!(project.units(), None, "{}", String::from_utf8_lossy(bad));
        }
        // Changing it leaves what a newer MakerCAD wrote beside "units" alone.
        project.others.retain(|(n, _)| n != DOCUMENT_ENTRY);
        project.others.push((
            DOCUMENT_ENTRY.into(),
            br#"{"version": 2, "grid": "fine", "units": "metric"}"#.to_vec(),
        ));
        project.set_units(System::Imperial);
        let (_, bytes) = project
            .others
            .iter()
            .find(|(n, _)| n == DOCUMENT_ENTRY)
            .unwrap();
        let note: serde_json::Value = serde_json::from_slice(bytes).unwrap();
        assert_eq!(note["units"], "imperial");
        assert_eq!(note["grid"], "fine");
        assert_eq!(note["version"], 2);
        // A note that is not an object is replaced, not patched.
        project.others.retain(|(n, _)| n != DOCUMENT_ENTRY);
        project
            .others
            .push((DOCUMENT_ENTRY.into(), b"[1]".to_vec()));
        project.set_units(System::Metric);
        assert_eq!(project.units(), Some(System::Metric));
    }

    #[test]
    fn saving_again_keeps_the_last_save_as_the_recovery_copy() {
        let dir = scratch("backup");
        let project = Project::new("Box", "").unwrap();
        project.save(&dir, b"first").unwrap();
        project.save(&dir, b"second").unwrap();
        assert!(dir.join("Box.maker.backup").is_file());
        assert!(!dir.join("Box.maker.saving").exists());
        // The file is damaged: the recovery copy (the first save) opens instead.
        fs::write(dir.join("Box.maker"), b"half a file").unwrap();
        let loaded = open(&dir, "Box.maker").unwrap();
        assert!(loaded.recovered);
        assert_eq!(loaded.design, b"first");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_missing_project_and_a_damaged_one_are_said_plainly() {
        let dir = scratch("missing");
        fs::create_dir_all(&dir).unwrap();
        assert!(open(&dir, "Nothing.maker")
            .unwrap_err()
            .contains("is not in"));
        fs::write(dir.join("Bad.maker"), b"not a zip").unwrap();
        assert!(open(&dir, "Bad.maker").unwrap_err().contains("Zip"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_folder_lists_project_files_newest_first() {
        let dir = scratch("list");
        fs::create_dir_all(&dir).unwrap();
        Project::new("Old", "").unwrap().save(&dir, b"a").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(20));
        Project::new("New", "").unwrap().save(&dir, b"b").unwrap();
        Project::new("New", "").unwrap().save(&dir, b"c").unwrap();
        let files: Vec<String> = list(&dir).into_iter().map(|l| l.file).collect();
        assert_eq!(files, vec!["New.maker", "Old.maker"]);
        assert!(list(&dir.join("nowhere")).is_empty());
        let _ = fs::remove_dir_all(&dir);
    }
}
