//! Contract tests for mk_inventory's quick-add, written by Claude from crew/contracts/mk_quickadd.md
//! (ATLAS-api-harness): the hidden tests of campaign mk-core-3, public now.

use mk_inventory::quickadd::{quick_add, Item, Kind};

fn near(a: Option<f64>, b: Option<f64>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(x), Some(y)) => (x - y).abs() < 1e-9,
        _ => false,
    }
}

fn near2(a: Option<(f64, f64)>, b: Option<(f64, f64)>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(x), Some(y)) => (x.0 - y.0).abs() < 1e-9 && (x.1 - y.1).abs() < 1e-9,
        _ => false,
    }
}

#[allow(clippy::too_many_arguments)]
fn check(
    text: &str,
    qty: u32,
    kind: Kind,
    material: &str,
    section: Option<(f64, f64)>,
    length: Option<f64>,
    thickness: Option<f64>,
    sheet: Option<(f64, f64)>,
    condition: Option<&str>,
    notes: &[&str],
) {
    let got: Item = quick_add(text);
    let ctx = format!("{text:?} -> {got:?}");
    assert_eq!(got.qty, qty, "qty: {ctx}");
    assert_eq!(got.kind, kind, "kind: {ctx}");
    assert_eq!(got.material, material, "material: {ctx}");
    assert!(near2(got.section_mm, section), "section_mm: {ctx}");
    assert!(near(got.length_mm, length), "length_mm: {ctx}");
    assert!(near(got.thickness_mm, thickness), "thickness_mm: {ctx}");
    assert!(near2(got.sheet_mm, sheet), "sheet_mm: {ctx}");
    assert_eq!(got.condition.as_deref(), condition, "condition: {ctx}");
    let want: Vec<String> = notes.iter().map(|s| s.to_string()).collect();
    assert_eq!(got.notes, want, "notes: {ctx}");
    assert_eq!(got.text, text.trim(), "text: {ctx}");
}

const B24: Option<(f64, f64)> = Some((38.1, 88.9));
const FULL: Option<(f64, f64)> = Some((1219.2, 2438.4));
const CALIPER: &str = "plywood is often thinner than its name: caliper it";

#[test]
fn boards_the_way_makers_say_them() {
    check(
        "15 pine 2x4s, 3 foot",
        15,
        Kind::Board,
        "pine",
        B24,
        Some(914.4),
        None,
        None,
        None,
        &[],
    );
    check(
        "15 \u{d7} pine 2x4 36\"",
        15,
        Kind::Board,
        "pine",
        B24,
        Some(914.4),
        None,
        None,
        None,
        &[],
    );
    check(
        "2x4 8ft",
        1,
        Kind::Board,
        "",
        B24,
        Some(2438.4),
        None,
        None,
        None,
        &[],
    );
    check(
        "3 2x6 x 10'",
        3,
        Kind::Board,
        "",
        Some((38.1, 139.7)),
        Some(3048.0),
        None,
        None,
        None,
        &[],
    );
    check(
        "four 2x4s 8' long",
        4,
        Kind::Board,
        "",
        B24,
        Some(2438.4),
        None,
        None,
        None,
        &[],
    );
}

#[test]
fn a_dozen_is_twelve() {
    check(
        "a dozen cedar 1x6 6 ft",
        12,
        Kind::Board,
        "cedar",
        Some((19.05, 139.7)),
        Some(1828.8),
        None,
        None,
        None,
        &[],
    );
    check(
        "dozen 2x4",
        12,
        Kind::Board,
        "",
        B24,
        None,
        None,
        None,
        None,
        &[],
    );
}

#[test]
fn a_bare_length_is_inches_and_says_so() {
    check(
        "2x4 96",
        1,
        Kind::Board,
        "",
        B24,
        Some(2438.4),
        None,
        None,
        None,
        &["a length without a unit is read as inches"],
    );
}

#[test]
fn condition_and_no_length() {
    check(
        "6 used oak 1x4",
        6,
        Kind::Board,
        "oak",
        Some((19.05, 88.9)),
        None,
        None,
        None,
        Some("used"),
        &[],
    );
}

#[test]
fn an_unreadable_length_is_a_note_not_a_failure() {
    check(
        "2x4 about 8 feet",
        1,
        Kind::Board,
        "",
        B24,
        None,
        None,
        None,
        None,
        &["could not read the length: about 8 feet"],
    );
}

#[test]
fn half_and_quarter_sheets() {
    check(
        "half sheet 3/4 ply",
        1,
        Kind::Sheet,
        "plywood",
        None,
        None,
        Some(19.05),
        Some((1219.2, 1219.2)),
        None,
        &["a half sheet is 4' x 4'", CALIPER],
    );
    check(
        "quarter sheet of 6mm birch ply",
        1,
        Kind::Sheet,
        "birch plywood",
        None,
        None,
        Some(6.0),
        Some((609.6, 1219.2)),
        None,
        &["a quarter sheet is 2' x 4'", CALIPER],
    );
}

#[test]
fn full_sheets_and_sizes() {
    check(
        "1 sheet \u{be}\u{2033} plywood 4x8",
        1,
        Kind::Sheet,
        "plywood",
        None,
        None,
        Some(19.05),
        FULL,
        None,
        &[CALIPER],
    );
    check(
        "2 sheets 1/2\" mdf",
        2,
        Kind::Sheet,
        "mdf",
        None,
        None,
        Some(12.7),
        FULL,
        None,
        &["a full sheet is 4' x 8'"],
    );
    check(
        "3 sheets osb 8x4",
        3,
        Kind::Sheet,
        "osb",
        None,
        None,
        None,
        FULL,
        None,
        &[],
    );
    check(
        "sheet of hardboard thin",
        1,
        Kind::Sheet,
        "hardboard",
        None,
        None,
        None,
        FULL,
        None,
        &[
            "a full sheet is 4' x 8'",
            "could not read the thickness: thin",
        ],
    );
}

#[test]
fn sheet_beats_lumber_for_a_size_like_2x4() {
    check(
        "sheet 2x4 1/4 hardboard",
        1,
        Kind::Sheet,
        "hardboard",
        None,
        None,
        Some(6.35),
        Some((609.6, 1219.2)),
        None,
        &[],
    );
}

#[test]
fn plexiglass_is_acrylic() {
    check(
        "2 sheets plexiglass 3mm 2x2",
        2,
        Kind::Sheet,
        "acrylic",
        None,
        None,
        Some(3.0),
        Some((609.6, 609.6)),
        None,
        &[],
    );
}

#[test]
fn everything_else_is_kept_in_words() {
    check(
        "an old bookshelf",
        1,
        Kind::Other,
        "bookshelf",
        None,
        None,
        None,
        None,
        Some("old"),
        &[],
    );
    check(
        "a stack of newspapers",
        1,
        Kind::Other,
        "stack of newspapers",
        None,
        None,
        None,
        None,
        None,
        &[],
    );
    check(
        "two IKEA Kallax units",
        2,
        Kind::Other,
        "ikea kallax units",
        None,
        None,
        None,
        None,
        None,
        &[],
    );
}

#[test]
fn a_number_before_a_measure_word_is_not_a_quantity() {
    check(
        "5 kg PLA",
        1,
        Kind::Other,
        "5 kg pla",
        None,
        None,
        None,
        None,
        None,
        &[],
    );
    check(
        "2 yards 500D Cordura",
        1,
        Kind::Other,
        "2 yards 500d cordura",
        None,
        None,
        None,
        None,
        None,
        &[],
    );
}

#[test]
fn empty_is_nothing() {
    check("   ", 0, Kind::Other, "", None, None, None, None, None, &[]);
    check("", 0, Kind::Other, "", None, None, None, None, None, &[]);
}

#[test]
fn only_the_first_condition_is_taken() {
    check(
        "old scrap tent",
        1,
        Kind::Other,
        "scrap tent",
        None,
        None,
        None,
        None,
        Some("old"),
        &[],
    );
}

#[test]
fn junk_never_crashes() {
    for t in [
        "x",
        "sheet",
        "sheets",
        "half",
        "half sheet",
        "a",
        "a dozen",
        "12345 2x4",
        "2x4 x",
        "\u{bd}",
        "\u{d7}\u{d7}\u{d7}",
        "9999 sheets",
        "sheet 0x0",
        "sheet 13x2",
        "2x4s 2x6 3'",
        "@ @ @",
        ", , ,",
        "1/0 2x4",
        "2x4 -5'",
        "\u{2032}\u{2033}",
    ] {
        let item = quick_add(t);
        assert_eq!(item.text, t.trim());
    }
}

#[test]
fn every_number_word_counts() {
    let words = [
        ("one", 1),
        ("two", 2),
        ("three", 3),
        ("four", 4),
        ("five", 5),
        ("six", 6),
        ("seven", 7),
        ("eight", 8),
        ("nine", 9),
        ("ten", 10),
        ("eleven", 11),
        ("twelve", 12),
        ("an", 1),
    ];
    for (w, n) in words {
        assert_eq!(quick_add(&format!("{w} 2x4")).qty, n, "{w}");
    }
}

#[test]
fn quantities_have_at_most_four_digits() {
    assert_eq!(quick_add("9999 2x4").qty, 9999);
    let five = quick_add("12345 2x4");
    assert_eq!(five.qty, 1);
    assert_eq!(
        five.notes,
        vec!["a length without a unit is read as inches".to_string()]
    );
}

#[test]
fn sheet_sizes_run_from_1_to_12_feet() {
    let ok = quick_add("sheet ply 1x12");
    assert!(
        near2(ok.sheet_mm, Some((304.8, 3657.6))),
        "{:?}",
        ok.sheet_mm
    );
    assert!(ok.notes.is_empty(), "{:?}", ok.notes);
    for bad in ["13x2", "0x4", "4xa", "x4"] {
        let item = quick_add(&format!("sheet ply {bad}"));
        assert!(near2(item.sheet_mm, FULL), "{bad}");
        assert_eq!(
            item.notes,
            vec![
                "a full sheet is 4' x 8'".to_string(),
                format!("could not read the thickness: {bad}")
            ],
            "{bad}"
        );
    }
}
