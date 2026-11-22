//! Contract tests for mk_project::layout, written by Claude from crew/contracts/mk_layout.md
//! (ATLAS-api-harness). They were the hidden tests of campaign mk-core-5; public here now.

use mk_project::layout::{
    check_name, classify, compression, design_id, diff_index, missing_required, Changed,
    Compression, Entry, EntryKind, IndexDiff, NameProblem, MANIFEST, REQUIRED_V1,
};

fn entries(pairs: &[(&str, &str)]) -> Vec<Entry> {
    pairs
        .iter()
        .map(|(n, h)| Entry {
            name: n.to_string(),
            hash: h.to_string(),
        })
        .collect()
}

fn names(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

#[test]
fn constants_are_exact() {
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
fn check_name_table() {
    use NameProblem::*;
    let rows: [(&str, Result<(), NameProblem>); 25] = [
        ("manifest.json", Ok(())),
        ("", Err(Empty)),
        ("media/a\tb.jpg", Err(ControlChar)),
        ("C:\\x", Err(Backslash)),
        ("/etc/passwd", Err(Absolute)),
        ("/../x", Err(Absolute)),
        ("C:/x", Err(Colon)),
        ("notes.txt:hidden", Err(Colon)),
        ("designs/<id>.sc", Err(ForbiddenChar)),
        ("media/what?.jpg", Err(ForbiddenChar)),
        ("a:b*", Err(Colon)),
        ("media/", Err(EmptySegment)),
        ("designs//a.sc", Err(EmptySegment)),
        ("../evil", Err(ParentSegment)),
        ("media/../x/", Err(ParentSegment)),
        ("./a", Err(CurrentSegment)),
        ("media/photo.", Err(TrailingDotOrSpace)),
        ("media/photo ", Err(TrailingDotOrSpace)),
        ("media/con.jpg", Err(Reserved)),
        ("media/Com1", Err(Reserved)),
        ("media/com10.jpg", Ok(())),
        ("media/console.txt", Ok(())),
        ("media/my photo.jpg", Ok(())),
        ("media/.keep", Ok(())),
        ("media/café.jpg", Ok(())),
    ];
    for (name, want) in rows {
        assert_eq!(check_name(name), want, "check_name({name:?})");
    }
}

#[test]
fn n2_control_characters_include_delete_and_newline() {
    assert_eq!(check_name("a\u{7f}b"), Err(NameProblem::ControlChar));
    assert_eq!(check_name("a\nb"), Err(NameProblem::ControlChar));
    assert_eq!(check_name("a\u{0}"), Err(NameProblem::ControlChar));
    assert_eq!(check_name("a\u{1f}"), Err(NameProblem::ControlChar));
    assert_eq!(check_name("a\u{80}b"), Ok(()));
}

#[test]
fn rules_apply_in_order() {
    // N2 before N3, N3 before N4, N4 before N5, N5 before N6, N6 before N7.
    assert_eq!(check_name("\\\t"), Err(NameProblem::ControlChar));
    assert_eq!(check_name("/a\\b"), Err(NameProblem::Backslash));
    assert_eq!(check_name("/a:b"), Err(NameProblem::Absolute));
    assert_eq!(check_name("../a:b"), Err(NameProblem::Colon));
    assert_eq!(check_name("../a|b"), Err(NameProblem::ForbiddenChar));
    for c in ['<', '>', '"', '|', '?', '*'] {
        assert_eq!(
            check_name(&format!("media/a{c}b.jpg")),
            Err(NameProblem::ForbiddenChar)
        );
    }
    assert_eq!(check_name("a//.."), Err(NameProblem::EmptySegment));
    assert_eq!(check_name("../"), Err(NameProblem::ParentSegment));
    assert_eq!(check_name("x/./.."), Err(NameProblem::CurrentSegment));
}

#[test]
fn n6_trailing_dot_or_space_and_dots() {
    assert_eq!(check_name("..."), Err(NameProblem::TrailingDotOrSpace));
    assert_eq!(check_name("a/b.c."), Err(NameProblem::TrailingDotOrSpace));
    assert_eq!(check_name(" a/b"), Ok(()));
    assert_eq!(check_name("a b/c d.txt"), Ok(()));
    assert_eq!(check_name(".hidden/x"), Ok(()));
    assert_eq!(check_name("..x"), Ok(()));
}

#[test]
fn n6_reserved_device_names() {
    for bad in [
        "CON",
        "prn",
        "Aux.txt",
        "nul.tar.gz",
        "COM1",
        "com9.x",
        "LPT1",
        "lpt9",
        "a/Nul",
    ] {
        assert_eq!(check_name(bad), Err(NameProblem::Reserved), "{bad:?}");
    }
    for ok in [
        "COM0",
        "LPT0",
        "COM",
        "CONS",
        "con1",
        "LPT10",
        "xCON",
        "AUXILIARY.txt",
        "a.con",
    ] {
        assert_eq!(check_name(ok), Ok(()), "{ok:?}");
    }
}

#[test]
fn classify_table() {
    use EntryKind::*;
    let rows: [(&str, EntryKind); 21] = [
        ("manifest.json", Manifest),
        ("project/intent.json", Intent),
        ("project/inventory.json", Inventory),
        ("project/space/room.json", Space),
        ("project/inspiration/board/1.jpg", Inspiration),
        ("designs/shelf.sc", Design),
        ("designs/shelf.ext.json", DesignExtension),
        ("designs/a.b.sc", Design),
        ("designs/.sc", Unknown),
        ("designs/x/y.sc", Unknown),
        ("designs/shelf.SC", Unknown),
        ("plan/plan.plancraft", Plan),
        ("log/journal.jsonl", Journal),
        ("log/sessions/2026-11-21.json", Session),
        ("log/sessions/.json", Unknown),
        ("log/sessions/a/b.json", Unknown),
        ("media/photo.jpg", Media),
        ("scans/room.ply", Scan),
        ("exports/pack.json", Export),
        ("Manifest.json", Unknown),
        ("media/../manifest.json", Unknown),
    ];
    for (name, want) in rows {
        assert_eq!(classify(name), want, "classify({name:?})");
    }
}

#[test]
fn k0_unsafe_names_are_unknown_even_with_a_known_shape() {
    assert_eq!(classify("media/con.jpg"), EntryKind::Unknown);
    assert_eq!(classify("designs/a .sc"), EntryKind::Design);
    assert_eq!(classify("designs/x.sc "), EntryKind::Unknown);
    assert_eq!(classify("media/a:b"), EntryKind::Unknown);
    assert_eq!(classify("media\\a.jpg"), EntryKind::Unknown);
    assert_eq!(classify(""), EntryKind::Unknown);
}

#[test]
fn classify_other_near_misses() {
    assert_eq!(classify("project/intent.json.bak"), EntryKind::Unknown);
    assert_eq!(classify("project/space"), EntryKind::Unknown);
    assert_eq!(classify("plan/other.plancraft"), EntryKind::Unknown);
    assert_eq!(classify("log/journal.json"), EntryKind::Unknown);
    assert_eq!(classify("log/sessions/x.jsonl"), EntryKind::Unknown);
    assert_eq!(classify("medias/a.jpg"), EntryKind::Unknown);
    assert_eq!(classify("Media/a.jpg"), EntryKind::Unknown);
    assert_eq!(classify("designs/a.ext.json"), EntryKind::DesignExtension);
    assert_eq!(classify("designs/a.ext.json.sc"), EntryKind::Design);
    assert_eq!(classify("designs/.ext.json"), EntryKind::Unknown);
    assert_eq!(classify("exports/sub/x.pdf"), EntryKind::Export);
}

#[test]
fn design_id_rows() {
    assert_eq!(design_id("designs/shelf.ext.json"), Some("shelf"));
    assert_eq!(design_id("designs/shelf.sc"), Some("shelf"));
    assert_eq!(design_id("designs/a.ext.json.sc"), Some("a.ext.json"));
    assert_eq!(design_id("designs/a.b.sc"), Some("a.b"));
    assert_eq!(design_id("designs/.ext.json"), None);
    assert_eq!(design_id("designs/.sc"), None);
    assert_eq!(design_id("media/shelf.sc"), None);
    assert_eq!(design_id("designs/x/y.sc"), None);
    assert_eq!(design_id("designs/shelf.json"), None);
    assert_eq!(design_id("designs/con.sc"), None);
    assert_eq!(design_id("designs"), None);
}

#[test]
fn design_id_borrows_from_the_name() {
    let name = String::from("designs/dresser.sc");
    let id = design_id(&name).unwrap();
    let start = name.as_ptr() as usize;
    let at = id.as_ptr() as usize;
    assert!(at >= start && at + id.len() <= start + name.len());
}

#[test]
fn compression_table() {
    use Compression::*;
    let rows: [(&str, Compression); 13] = [
        ("manifest.json", Deflate),
        ("log/journal.jsonl", Deflate),
        ("designs/shelf.sc", Deflate),
        ("plan/plan.plancraft", Deflate),
        ("media/NOTES.TXT", Deflate),
        ("project/inspiration/a.md", Deflate),
        ("exports/cuts.csv", Deflate),
        ("media/logo.Svg", Deflate),
        ("media/photo.jpg", Store),
        ("scans/room.ply", Store),
        ("media.json/photo.jpg", Store),
        ("", Store),
        ("media/json", Store),
    ];
    for (name, want) in rows {
        assert_eq!(compression(name), want, "compression({name:?})");
    }
}

#[test]
fn compression_answers_for_unsafe_names_too() {
    assert_eq!(compression("../x.json"), Compression::Deflate);
    assert_eq!(compression("C:\\a\\b.json"), Compression::Deflate);
    assert_eq!(compression("a/"), Compression::Store);
}

#[test]
fn missing_required_rows() {
    assert_eq!(missing_required(&[]), REQUIRED_V1.to_vec());
    let full = [
        "manifest.json",
        "project/intent.json",
        "designs/a.sc",
        "log/journal.jsonl",
    ];
    assert_eq!(missing_required(&full), Vec::<&str>::new());
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
fn missing_required_is_exact_and_ignores_order_and_repeats() {
    let got = missing_required(&[
        "log/journal.jsonl",
        "log/journal.jsonl",
        "designs/b.sc",
        "designs/a.sc",
        "MANIFEST.JSON",
        "project/intent.json",
    ]);
    assert_eq!(got, vec!["manifest.json"]);
    assert_eq!(
        missing_required(&[
            "manifest.json",
            "project/intent.json",
            "designs/<id>.sc",
            "log/journal.jsonl"
        ]),
        vec!["designs/<id>.sc"]
    );
    assert_eq!(
        missing_required(&[
            "manifest.json",
            "project/intent.json",
            "designs/x/y.sc",
            "log/journal.jsonl"
        ]),
        vec!["designs/<id>.sc"]
    );
}

#[test]
fn diff_index_main_row() {
    let got = diff_index(
        &entries(&[("a", "1"), ("b", "2"), ("c", "3")]),
        &entries(&[("a", "1"), ("b", "9"), ("d", "4")]),
    );
    let want = IndexDiff {
        missing: names(&["c"]),
        changed: vec![Changed {
            name: "b".into(),
            expected: "2".into(),
            found: "9".into(),
        }],
        extra: names(&["d"]),
        duplicates: vec![],
    };
    assert_eq!(got, want);
}

#[test]
fn diff_index_hashes_ignore_ascii_case_but_are_reported_as_given() {
    assert_eq!(
        diff_index(&entries(&[("a", "ff")]), &entries(&[("a", "FF")])),
        IndexDiff::default()
    );
    let got = diff_index(&entries(&[("a", "Ab")]), &entries(&[("a", "aC")]));
    assert_eq!(
        got.changed,
        vec![Changed {
            name: "a".into(),
            expected: "Ab".into(),
            found: "aC".into()
        }]
    );
}

#[test]
fn diff_index_never_lists_the_manifest_as_extra() {
    let got = diff_index(
        &entries(&[("a", "1")]),
        &entries(&[("a", "1"), ("manifest.json", "7")]),
    );
    assert_eq!(got, IndexDiff::default());
}

#[test]
fn diff_index_manifest_in_the_index_is_an_ordinary_entry() {
    let got = diff_index(&entries(&[("manifest.json", "1")]), &entries(&[]));
    assert_eq!(got.missing, names(&["manifest.json"]));
}

#[test]
fn diff_index_duplicates_and_first_entries() {
    let got = diff_index(
        &entries(&[("a", "1")]),
        &entries(&[("a", "1"), ("b", "2"), ("a", "5"), ("b", "2"), ("a", "6")]),
    );
    let want = IndexDiff {
        missing: vec![],
        changed: vec![],
        extra: names(&["b"]),
        duplicates: names(&["a", "b"]),
    };
    assert_eq!(got, want);
}

#[test]
fn diff_index_duplicates_follow_second_appearance_order() {
    let got = diff_index(
        &[],
        &entries(&[("x", "1"), ("y", "1"), ("y", "1"), ("x", "1")]),
    );
    assert_eq!(got.duplicates, names(&["y", "x"]));
    assert_eq!(got.extra, names(&["x", "y"]));
}

#[test]
fn diff_index_first_index_entry_counts() {
    let got = diff_index(&entries(&[("a", "1"), ("a", "2")]), &entries(&[("a", "2")]));
    assert_eq!(
        got.changed,
        vec![Changed {
            name: "a".into(),
            expected: "1".into(),
            found: "2".into()
        }]
    );
    assert!(got.missing.is_empty() && got.extra.is_empty() && got.duplicates.is_empty());
}

#[test]
fn diff_index_empty_and_order() {
    assert_eq!(diff_index(&[], &[]), IndexDiff::default());
    let got = diff_index(
        &entries(&[("z", "1"), ("m", "1"), ("a", "1")]),
        &entries(&[("q", "1"), ("b", "1")]),
    );
    assert_eq!(got.missing, names(&["z", "m", "a"]));
    assert_eq!(got.extra, names(&["q", "b"]));
}

#[test]
fn odd_names_never_panic() {
    let odd = [
        "",
        "/",
        "//",
        ".",
        "..",
        "...",
        " ",
        ":",
        "\\",
        "\u{0}",
        "é",
        "aaé",
        "coé",
        "COMé",
        "designs/",
        "designs/é.sc",
        "log/sessions/",
        "log/sessions/é.json",
        "media/\u{1F600}",
        "a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p",
        "ÀÁÂ/ÃÄÅ",
        "lpt\u{1F600}",
        "\u{feff}manifest.json",
    ];
    for name in odd {
        let _ = check_name(name);
        let _ = classify(name);
        let _ = design_id(name);
        let _ = compression(name);
    }
    let _ = missing_required(&odd);
    let list: Vec<Entry> = odd
        .iter()
        .map(|n| Entry {
            name: n.to_string(),
            hash: "é".into(),
        })
        .collect();
    let _ = diff_index(&list, &list);
    assert_eq!(classify("\u{feff}manifest.json"), EntryKind::Unknown);
    assert_eq!(check_name("aaé"), Ok(()));
}
