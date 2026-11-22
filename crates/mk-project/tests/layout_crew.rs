//! Tests for mk_project::layout written by Gordon (the Crew, campaign mk-core-5) from the
//! contract alone, without seeing the module.

//! Integration tests for `layout`: entry names, classification, compression and index diff.

use mk_project::layout::{
    check_name, classify, compression, design_id, diff_index, missing_required, Changed,
    Compression, Entry, EntryKind, IndexDiff, NameProblem, MANIFEST, REQUIRED_V1,
};

fn entry(name: &str, hash: &str) -> Entry {
    Entry {
        name: name.to_string(),
        hash: hash.to_string(),
    }
}

fn changed(name: &str, expected: &str, found: &str) -> Changed {
    Changed {
        name: name.to_string(),
        expected: expected.to_string(),
        found: found.to_string(),
    }
}

fn empty_diff() -> IndexDiff {
    IndexDiff {
        missing: Vec::new(),
        changed: Vec::new(),
        extra: Vec::new(),
        duplicates: Vec::new(),
    }
}

#[test]
fn constants_are_the_contract_values() {
    assert_eq!(MANIFEST, "manifest.json");
    assert_eq!(
        REQUIRED_V1,
        [
            "manifest.json",
            "project/intent.json",
            "designs/<id>.sc",
            "log/journal.jsonl"
        ]
    );
}

#[test]
fn n1_empty_name_is_empty() {
    assert_eq!(check_name(""), Err(NameProblem::Empty));
}

#[test]
fn n2_control_characters_are_refused() {
    assert_eq!(check_name("media/a\tb.jpg"), Err(NameProblem::ControlChar));
    assert_eq!(
        check_name("media/a\u{7f}b.jpg"),
        Err(NameProblem::ControlChar)
    );
    assert_eq!(check_name("a\u{1f}"), Err(NameProblem::ControlChar));
    // N2 comes before N3: a control char and a backslash together.
    assert_eq!(check_name("a\\\u{1}b"), Err(NameProblem::ControlChar));
}

#[test]
fn n3_backslash_before_colon() {
    assert_eq!(check_name("C:\\x"), Err(NameProblem::Backslash));
    assert_eq!(check_name("media\\photo.jpg"), Err(NameProblem::Backslash));
    assert_eq!(check_name("a\\b:c"), Err(NameProblem::Backslash));
}

#[test]
fn n4_absolute_before_segment_rules() {
    assert_eq!(check_name("/etc/passwd"), Err(NameProblem::Absolute));
    assert_eq!(check_name("/../x"), Err(NameProblem::Absolute));
    assert_eq!(check_name("/"), Err(NameProblem::Absolute));
}

#[test]
fn n5_colon_before_forbidden_char() {
    assert_eq!(check_name("C:/x"), Err(NameProblem::Colon));
    assert_eq!(check_name("notes.txt:hidden"), Err(NameProblem::Colon));
    assert_eq!(check_name("a:b*"), Err(NameProblem::Colon));
}

#[test]
fn n6_every_forbidden_character() {
    for c in ['<', '>', '"', '|', '?', '*'] {
        let name = format!("media/a{}b.jpg", c);
        assert_eq!(
            check_name(&name),
            Err(NameProblem::ForbiddenChar),
            "char {:?}",
            c
        );
    }
    assert_eq!(
        check_name("designs/<id>.sc"),
        Err(NameProblem::ForbiddenChar)
    );
    assert_eq!(
        check_name("media/what?.jpg"),
        Err(NameProblem::ForbiddenChar)
    );
}

#[test]
fn n7a_empty_segment() {
    assert_eq!(check_name("media/"), Err(NameProblem::EmptySegment));
    assert_eq!(check_name("designs//a.sc"), Err(NameProblem::EmptySegment));
    assert_eq!(check_name("a//b"), Err(NameProblem::EmptySegment));
}

#[test]
fn n7b_parent_segment() {
    assert_eq!(check_name("../evil"), Err(NameProblem::ParentSegment));
    assert_eq!(check_name("media/../x/"), Err(NameProblem::ParentSegment));
    assert_eq!(check_name("media/.."), Err(NameProblem::ParentSegment));
}

#[test]
fn n7c_current_segment() {
    assert_eq!(check_name("./a"), Err(NameProblem::CurrentSegment));
    assert_eq!(
        check_name("media/./photo.jpg"),
        Err(NameProblem::CurrentSegment)
    );
}

#[test]
fn n7d_trailing_dot_or_space() {
    assert_eq!(
        check_name("media/photo."),
        Err(NameProblem::TrailingDotOrSpace)
    );
    assert_eq!(
        check_name("media/photo "),
        Err(NameProblem::TrailingDotOrSpace)
    );
    assert_eq!(
        check_name("media/photo. "),
        Err(NameProblem::TrailingDotOrSpace)
    );
}

#[test]
fn n7e_reserved_names_mixed_case_and_extensions() {
    assert_eq!(check_name("media/con.jpg"), Err(NameProblem::Reserved));
    assert_eq!(check_name("media/Com1"), Err(NameProblem::Reserved));
    assert_eq!(check_name("media/NUL"), Err(NameProblem::Reserved));
    assert_eq!(check_name("media/prn"), Err(NameProblem::Reserved));
    assert_eq!(check_name("media/aux.tar.gz"), Err(NameProblem::Reserved));
    assert_eq!(check_name("media/CoM3.log"), Err(NameProblem::Reserved));
    assert_eq!(check_name("media/lpt9.txt"), Err(NameProblem::Reserved));
    assert_eq!(check_name("media/COM1.txt"), Err(NameProblem::Reserved));
    assert_eq!(check_name("media/LPT1"), Err(NameProblem::Reserved));
    // A reserved name in a later segment still decides.
    assert_eq!(check_name("media/x/aux.json"), Err(NameProblem::Reserved));
}

#[test]
fn n7e_near_misses_are_fine() {
    assert_eq!(check_name("media/com10.jpg"), Ok(()));
    assert_eq!(check_name("media/COM0"), Ok(()));
    assert_eq!(check_name("media/LPT10"), Ok(()));
    assert_eq!(check_name("media/CONS"), Ok(()));
    assert_eq!(check_name("media/con1"), Ok(()));
    assert_eq!(check_name("media/AUXILIARY.txt"), Ok(()));
    assert_eq!(check_name("media/console.txt"), Ok(()));
    assert_eq!(check_name("media/my photo.jpg"), Ok(()));
    assert_eq!(check_name("media/.keep"), Ok(()));
    assert_eq!(check_name("media/caf\u{e9}.jpg"), Ok(()));
    assert_eq!(check_name("manifest.json"), Ok(()));
}

#[test]
fn n_rules_run_in_order_n1_to_n7() {
    // N2 before N3, N3 before N5, N5 before N6, N4 before N7, N7a before N7b.
    assert_eq!(check_name("a\u{1}\\:b"), Err(NameProblem::ControlChar));
    assert_eq!(check_name("a\\b:c"), Err(NameProblem::Backslash));
    assert_eq!(check_name("a:b*"), Err(NameProblem::Colon));
    assert_eq!(check_name("/../x"), Err(NameProblem::Absolute));
    assert_eq!(check_name("a//../b"), Err(NameProblem::EmptySegment));
    assert_eq!(check_name("a/./.."), Err(NameProblem::CurrentSegment));
    // N7d before N7e: "con." ends with a dot, so the trailing dot decides.
    assert_eq!(
        check_name("media/con."),
        Err(NameProblem::TrailingDotOrSpace)
    );
    assert_eq!(
        check_name("media/con "),
        Err(NameProblem::TrailingDotOrSpace)
    );
    // The first failing segment decides.
    assert_eq!(check_name("media/con.jpg/../x"), Err(NameProblem::Reserved));
    assert_eq!(
        check_name("media/../con.jpg"),
        Err(NameProblem::ParentSegment)
    );
}

#[test]
fn k0_unsafe_names_are_unknown() {
    assert_eq!(classify("media/../manifest.json"), EntryKind::Unknown);
    assert_eq!(classify(""), EntryKind::Unknown);
    assert_eq!(classify("media/con.jpg"), EntryKind::Unknown);
    assert_eq!(classify("designs/<id>.sc"), EntryKind::Unknown);
    assert_eq!(classify("manifest.json/"), EntryKind::Unknown);
}

#[test]
fn k1_k2_k3_manifest_intent_inventory() {
    assert_eq!(classify("manifest.json"), EntryKind::Manifest);
    assert_eq!(classify("project/intent.json"), EntryKind::Intent);
    assert_eq!(classify("project/inventory.json"), EntryKind::Inventory);
    assert_eq!(classify("Manifest.json"), EntryKind::Unknown);
    assert_eq!(classify("project/Intent.json"), EntryKind::Unknown);
    assert_eq!(classify("project/inventory.json.bak"), EntryKind::Unknown);
}

#[test]
fn k4_k5_space_and_inspiration() {
    assert_eq!(classify("project/space/room.json"), EntryKind::Space);
    assert_eq!(classify("project/space/a/b.json"), EntryKind::Space);
    assert_eq!(
        classify("project/inspiration/board/1.jpg"),
        EntryKind::Inspiration
    );
    assert_eq!(classify("project/space"), EntryKind::Unknown);
    assert_eq!(classify("project/spaces/room.json"), EntryKind::Unknown);
}

#[test]
fn k6_k7_design_and_design_extension() {
    assert_eq!(classify("designs/shelf.sc"), EntryKind::Design);
    assert_eq!(
        classify("designs/shelf.ext.json"),
        EntryKind::DesignExtension
    );
    assert_eq!(classify("designs/a.b.sc"), EntryKind::Design);
    assert_eq!(classify("designs/a.ext.json.sc"), EntryKind::Design);
    assert_eq!(classify("designs/.sc"), EntryKind::Unknown);
    assert_eq!(classify("designs/.ext.json"), EntryKind::Unknown);
    assert_eq!(classify("designs/x/y.sc"), EntryKind::Unknown);
    assert_eq!(classify("designs/shelf.SC"), EntryKind::Unknown);
}

#[test]
fn k8_to_k13_plan_journal_session_media_scan_export() {
    assert_eq!(classify("plan/plan.plancraft"), EntryKind::Plan);
    assert_eq!(classify("log/journal.jsonl"), EntryKind::Journal);
    assert_eq!(classify("log/sessions/2026-11-21.json"), EntryKind::Session);
    assert_eq!(classify("log/sessions/.json"), EntryKind::Unknown);
    assert_eq!(classify("log/sessions/a/b.json"), EntryKind::Unknown);
    assert_eq!(classify("log/sessions/a.txt"), EntryKind::Unknown);
    assert_eq!(classify("media/photo.jpg"), EntryKind::Media);
    assert_eq!(classify("scans/room.ply"), EntryKind::Scan);
    assert_eq!(classify("exports/pack.json"), EntryKind::Export);
    assert_eq!(classify("plan/plan.plancraft2"), EntryKind::Unknown);
    assert_eq!(classify("media"), EntryKind::Unknown);
}

#[test]
fn k14_anything_else_is_unknown() {
    assert_eq!(classify("notes.txt"), EntryKind::Unknown);
    assert_eq!(classify("designs/shelf.sc.bak"), EntryKind::Unknown);
    assert_eq!(classify("project/other.json"), EntryKind::Unknown);
}

#[test]
fn i1_design_id_tries_ext_json_first() {
    assert_eq!(design_id("designs/shelf.ext.json"), Some("shelf"));
    assert_eq!(design_id("designs/a.ext.json.sc"), Some("a.ext.json"));
    assert_eq!(design_id("designs/shelf.sc"), Some("shelf"));
    assert_eq!(design_id("designs/a.b.sc"), Some("a.b"));
    assert_eq!(design_id("designs/.ext.json"), None);
    assert_eq!(design_id("designs/.sc"), None);
    assert_eq!(design_id("designs/x/y.sc"), None);
    assert_eq!(design_id("designs/shelf.SC"), None);
    assert_eq!(design_id("media/shelf.sc"), None);
    assert_eq!(design_id("designs/../x.sc"), None);
    assert_eq!(design_id("designs/shelf.ext.jsonx"), None);
}

#[test]
fn c1_deflate_for_text_extensions() {
    assert_eq!(compression("manifest.json"), Compression::Deflate);
    assert_eq!(compression("log/journal.jsonl"), Compression::Deflate);
    assert_eq!(compression("designs/shelf.sc"), Compression::Deflate);
    assert_eq!(compression("plan/plan.plancraft"), Compression::Deflate);
    assert_eq!(compression("media/NOTES.TXT"), Compression::Deflate);
    assert_eq!(compression("media/notes.md"), Compression::Deflate);
    assert_eq!(compression("exports/table.csv"), Compression::Deflate);
    assert_eq!(compression("exports/pack.svg"), Compression::Deflate);
    assert_eq!(compression("media/a.Json"), Compression::Deflate);
}

#[test]
fn c2_store_for_everything_else() {
    assert_eq!(compression("media/photo.jpg"), Compression::Store);
    assert_eq!(compression("scans/room.ply"), Compression::Store);
    assert_eq!(compression("media.json/photo.jpg"), Compression::Store);
    assert_eq!(compression(""), Compression::Store);
    assert_eq!(compression("media/a.jsonx"), Compression::Store);
    assert_eq!(compression("media/photo "), Compression::Store);
}

#[test]
fn m1_missing_required_in_required_order() {
    assert_eq!(missing_required(&[]), REQUIRED_V1.to_vec());
    assert_eq!(
        missing_required(&[
            "manifest.json",
            "project/intent.json",
            "designs/a.sc",
            "log/journal.jsonl",
        ]),
        Vec::<&'static str>::new()
    );
    assert_eq!(
        missing_required(&["manifest.json", "designs/a.ext.json"]),
        vec![
            "project/intent.json",
            "designs/<id>.sc",
            "log/journal.jsonl"
        ]
    );
}

#[test]
fn m1_repeated_and_reordered_names_change_nothing() {
    assert_eq!(
        missing_required(&[
            "log/journal.jsonl",
            "manifest.json",
            "manifest.json",
            "designs/b.sc",
            "designs/a.sc",
            "project/intent.json",
        ]),
        Vec::<&'static str>::new()
    );
    // A design extension is not a design.
    assert_eq!(
        missing_required(&["designs/a.ext.json"]),
        vec![
            "manifest.json",
            "project/intent.json",
            "designs/<id>.sc",
            "log/journal.jsonl"
        ]
    );
    // An unsafe name that looks like a design classifies as Unknown, so it does not count.
    assert_eq!(
        missing_required(&["designs/../a.sc"]),
        vec![
            "manifest.json",
            "project/intent.json",
            "designs/<id>.sc",
            "log/journal.jsonl"
        ]
    );
}

#[test]
fn d2_d3_d4_missing_changed_extra() {
    let index = [entry("a", "1"), entry("b", "2"), entry("c", "3")];
    let present = [entry("a", "1"), entry("b", "9"), entry("d", "4")];
    assert_eq!(
        diff_index(&index, &present),
        IndexDiff {
            missing: vec!["c".to_string()],
            changed: vec![changed("b", "2", "9")],
            extra: vec!["d".to_string()],
            duplicates: Vec::new(),
        }
    );
}

#[test]
fn d3_hashes_compare_ignoring_ascii_case() {
    let index = [entry("a", "ff")];
    let present = [entry("a", "FF")];
    assert_eq!(diff_index(&index, &present), empty_diff());
    // Hashes are reported exactly as given.
    let index = [entry("a", "AbC")];
    let present = [entry("a", "xyz")];
    assert_eq!(
        diff_index(&index, &present),
        IndexDiff {
            missing: Vec::new(),
            changed: vec![changed("a", "AbC", "xyz")],
            extra: Vec::new(),
            duplicates: Vec::new(),
        }
    );
}

#[test]
fn d4_manifest_is_never_extra() {
    let index = [entry("a", "1")];
    let present = [entry("a", "1"), entry(MANIFEST, "7")];
    assert_eq!(diff_index(&index, &present), empty_diff());
    // D4 exempts the manifest from `extra` only; D3 still compares it.
    let index = [entry(MANIFEST, "1")];
    let present = [entry(MANIFEST, "2")];
    assert_eq!(
        diff_index(&index, &present),
        IndexDiff {
            missing: Vec::new(),
            changed: vec![changed(MANIFEST, "1", "2")],
            extra: Vec::new(),
            duplicates: Vec::new(),
        }
    );
}

#[test]
fn d0_d1_duplicates_in_second_appearance_order() {
    let index = [entry("a", "1")];
    let present = [
        entry("a", "1"),
        entry("b", "2"),
        entry("a", "5"),
        entry("b", "2"),
        entry("a", "6"),
    ];
    assert_eq!(
        diff_index(&index, &present),
        IndexDiff {
            missing: Vec::new(),
            changed: Vec::new(),
            extra: vec!["b".to_string()],
            duplicates: vec!["a".to_string(), "b".to_string()],
        }
    );
}

#[test]
fn d0_first_entry_counts_in_both_lists() {
    let index = [entry("a", "1"), entry("a", "2")];
    let present = [entry("a", "2")];
    assert_eq!(
        diff_index(&index, &present),
        IndexDiff {
            missing: Vec::new(),
            changed: vec![changed("a", "1", "2")],
            extra: Vec::new(),
            duplicates: Vec::new(),
        }
    );
    // The first present entry counts, so the later "a=1" is ignored.
    let index = [entry("a", "1")];
    let present = [entry("a", "2"), entry("a", "1")];
    assert_eq!(
        diff_index(&index, &present),
        IndexDiff {
            missing: Vec::new(),
            changed: vec![changed("a", "1", "2")],
            extra: Vec::new(),
            duplicates: vec!["a".to_string()],
        }
    );
}

#[test]
fn d0_empty_lists_give_an_empty_diff() {
    assert_eq!(diff_index(&[], &[]), empty_diff());
}

#[test]
fn odd_names_never_panic_in_any_function() {
    let odd = [
        "",
        "/",
        "//",
        ".",
        "..",
        "...",
        "a//b",
        "media/",
        "media/../x/",
        "designs//a.sc",
        "C:\\x",
        "notes.txt:hidden",
        "media/what?.jpg",
        "media/photo ",
        "media/con.jpg",
        "log/sessions/.json",
        "designs/.sc",
        "designs/x/y.sc",
        "media.json/photo.jpg",
        "media/NOTES.TXT",
        "manifest.json",
        "designs/shelf.ext.json",
        "caf\u{e9}.jpg",
        "\u{1F600}.jpg",
        "\u{e9}\u{e9}",
        "media/\u{1F600}\u{e9}",
        "a\u{1}b",
        "media/a\u{7f}",
    ];
    for name in odd {
        let checked = check_name(name);
        // K0: an unsafe name is Unknown.
        if checked.is_err() {
            assert_eq!(classify(name), EntryKind::Unknown, "name {:?}", name);
        }
        let _ = design_id(name);
        let _ = compression(name);
        let _ = missing_required(&[name]);
        let index = [entry(name, "1")];
        let present = [entry(name, "1"), entry(name, "2")];
        let diff = diff_index(&index, &present);
        assert_eq!(diff.duplicates, vec![name.to_string()], "name {:?}", name);
    }
}
