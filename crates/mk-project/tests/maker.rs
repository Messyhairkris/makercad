//! Saving and opening a whole project: it comes back as it went, and what cannot be trusted is
//! refused while what merely changed is reported.

use mk_project::container::{content_hash, pack, unpack, ContainerError, Limits};
use mk_project::layout::IndexDiff;
use mk_project::maker::{open, save, MakerError, INTENT};
use mk_project::project::{
    Budget, Date, Deadline, FormatStatus, Intent, Manifest, ProjectError, Skill, Weekday,
    WorkWindow, FORMAT_VERSION,
};

fn intent() -> Intent {
    let mut intent = Intent::new("Dresser", "A four-drawer dresser for the alcove").unwrap();
    intent.success = vec!["Fits the alcove with 3 mm each side".into()];
    intent.skill = Skill::Comfortable;
    intent.deadline = Some(Deadline {
        due: Date {
            year: 2026,
            month: 12,
            day: 12,
        },
        windows: vec![WorkWindow {
            days: vec![Weekday::Mon, Weekday::Wed],
            start_minute: 19 * 60,
            end_minute: 21 * 60,
        }],
        milestones: vec![],
    });
    intent.budget = Some(Budget {
        cap_minor: 25_000,
        currency: "USD".into(),
        spend_only_if_needed: true,
    });
    intent
}

fn manifest() -> Manifest {
    let mut manifest = Manifest::new("0.1.0");
    manifest.environments = vec!["design".into(), "stock".into()];
    manifest
}

const DESIGN: &[u8] = b"solidcraft design bytes";
const JOURNAL: &[u8] = b"{\"kind\":\"intent\"}\n";

fn others() -> Vec<(&'static str, &'static [u8])> {
    vec![
        ("designs/d1.sc", DESIGN),
        ("log/journal.jsonl", JOURNAL),
        ("media/photo.png", b"\x89PNG"),
    ]
}

#[test]
fn a_saved_project_opens_as_it_was() {
    let bytes = save(&manifest(), &intent(), &others()).unwrap();
    let opened = open(&bytes, Limits::default()).unwrap();
    assert_eq!(opened.intent, intent());
    assert_eq!(opened.format, FormatStatus::Current);
    assert_eq!(opened.manifest.app_version, "0.1.0");
    assert_eq!(opened.manifest.environments, vec!["design", "stock"]);
    assert_eq!(opened.diff, IndexDiff::default());
    let names: Vec<&str> = opened.entries.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(
        names,
        vec!["designs/d1.sc", "log/journal.jsonl", "media/photo.png"]
    );
    assert_eq!(opened.entries[0].1, DESIGN);
}

#[test]
fn the_index_holds_every_entry_but_the_manifest() {
    let bytes = save(&manifest(), &intent(), &others()).unwrap();
    let opened = open(&bytes, Limits::default()).unwrap();
    let names: Vec<&str> = opened
        .manifest
        .hashes()
        .iter()
        .map(|h| h.name.as_str())
        .collect();
    assert_eq!(
        names,
        vec![
            "designs/d1.sc",
            "log/journal.jsonl",
            "media/photo.png",
            "project/intent.json"
        ]
    );
    assert_eq!(
        opened.manifest.hash_of("designs/d1.sc"),
        Some(content_hash(DESIGN).as_str())
    );
}

#[test]
fn the_manifest_and_intent_come_first_and_are_readable_json() {
    let bytes = save(&manifest(), &intent(), &others()).unwrap();
    let raw = unpack(&bytes, Limits::default()).unwrap();
    assert_eq!(raw[0].0, "manifest.json");
    assert_eq!(raw[1].0, INTENT);
    let manifest: serde_json::Value = serde_json::from_slice(&raw[0].1).unwrap();
    assert_eq!(manifest["format_version"], FORMAT_VERSION);
    let intent: serde_json::Value = serde_json::from_slice(&raw[1].1).unwrap();
    assert_eq!(intent["skill"], "comfortable");
    assert_eq!(intent["deadline"]["windows"][0]["days"][1], "wed");
}

#[test]
fn saving_twice_gives_the_same_bytes() {
    assert_eq!(
        save(&manifest(), &intent(), &others()).unwrap(),
        save(&manifest(), &intent(), &others()).unwrap()
    );
}

#[test]
fn a_stale_index_in_the_given_manifest_is_replaced() {
    let mut stale = manifest();
    stale.set_hash("designs/gone.sc", "abc").unwrap();
    let bytes = save(&stale, &intent(), &others()).unwrap();
    let opened = open(&bytes, Limits::default()).unwrap();
    assert_eq!(opened.manifest.hash_of("designs/gone.sc"), None);
    assert_eq!(opened.diff, IndexDiff::default());
}

#[test]
fn an_intent_that_breaks_its_rules_is_not_saved() {
    let mut bad = intent();
    bad.goal = "  ".into();
    assert_eq!(
        save(&manifest(), &bad, &others()),
        Err(MakerError::Project(ProjectError::EmptyGoal))
    );
}

#[test]
fn the_manifest_and_intent_names_are_reserved() {
    let mut entries = others();
    entries.push((INTENT, b"{}"));
    assert_eq!(
        save(&manifest(), &intent(), &entries),
        Err(MakerError::Reserved(INTENT.into()))
    );
}

#[test]
fn an_unsafe_entry_name_is_not_saved() {
    let mut entries = others();
    entries.push(("../outside.txt", b"x"));
    assert_eq!(
        save(&manifest(), &intent(), &entries),
        Err(MakerError::Project(ProjectError::BadHashName(
            "../outside.txt".into()
        )))
    );
}

/// The entries of a saved project, with one replaced, removed or added, packed again.
fn edited(change: impl FnOnce(&mut Vec<(String, Vec<u8>)>)) -> Vec<u8> {
    let bytes = save(&manifest(), &intent(), &others()).unwrap();
    let mut entries = unpack(&bytes, Limits::default()).unwrap();
    change(&mut entries);
    let borrowed: Vec<(&str, &[u8])> = entries
        .iter()
        .map(|(n, b)| (n.as_str(), b.as_slice()))
        .collect();
    pack(&borrowed).unwrap()
}

#[test]
fn an_entry_changed_outside_makercad_opens_and_is_reported() {
    let bytes = edited(|e| e[2].1 = b"edited elsewhere".to_vec());
    let opened = open(&bytes, Limits::default()).unwrap();
    assert_eq!(opened.diff.changed.len(), 1);
    assert_eq!(opened.diff.changed[0].name, "designs/d1.sc");
    assert_eq!(
        opened.diff.changed[0].found,
        content_hash(b"edited elsewhere")
    );
}

#[test]
fn an_added_entry_opens_and_is_reported_as_extra() {
    let bytes = edited(|e| e.push(("media/new.png".into(), b"png".to_vec())));
    let opened = open(&bytes, Limits::default()).unwrap();
    assert_eq!(opened.diff.extra, vec!["media/new.png".to_string()]);
}

#[test]
fn a_project_without_its_journal_is_refused() {
    let bytes = edited(|e| e.retain(|(n, _)| n != "log/journal.jsonl"));
    assert_eq!(
        open(&bytes, Limits::default()),
        Err(MakerError::Missing(vec!["log/journal.jsonl"]))
    );
}

#[test]
fn a_newer_format_is_refused() {
    let bytes = edited(|e| {
        let mut m: serde_json::Value = serde_json::from_slice(&e[0].1).unwrap();
        m["format_version"] = serde_json::json!(FORMAT_VERSION + 1);
        e[0].1 = serde_json::to_vec(&m).unwrap();
    });
    assert_eq!(
        open(&bytes, Limits::default()),
        Err(MakerError::NewerFormat(FORMAT_VERSION + 1))
    );
}

#[test]
fn broken_json_is_named() {
    let bytes = edited(|e| e[1].1 = b"{ not json".to_vec());
    assert!(matches!(
        open(&bytes, Limits::default()),
        Err(MakerError::Json { name, .. }) if name == INTENT
    ));
}

#[test]
fn a_tampered_index_name_is_refused() {
    let bytes = edited(|e| {
        let mut m: serde_json::Value = serde_json::from_slice(&e[0].1).unwrap();
        m["hashes"][0]["name"] = serde_json::json!("../x");
        e[0].1 = serde_json::to_vec(&m).unwrap();
    });
    assert_eq!(
        open(&bytes, Limits::default()),
        Err(MakerError::Project(ProjectError::BadHashName(
            "../x".into()
        )))
    );
}

#[test]
fn not_a_project_file_is_a_container_error() {
    assert!(matches!(
        open(b"hello", Limits::default()),
        Err(MakerError::Container(ContainerError::Zip(_)))
    ));
}
