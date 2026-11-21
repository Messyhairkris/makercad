//! Integration tests for `quickadd::quick_add`, written from the contract alone.

use mk_inventory::quickadd::{quick_add, Item, Kind};

const EPS: f64 = 1e-9;

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < EPS
}

fn opt_close(got: Option<f64>, want: Option<f64>) -> bool {
    match (got, want) {
        (None, None) => true,
        (Some(g), Some(w)) => close(g, w),
        _ => false,
    }
}

fn pair_close(got: Option<(f64, f64)>, want: Option<(f64, f64)>) -> bool {
    match (got, want) {
        (None, None) => true,
        (Some((g1, g2)), Some((w1, w2))) => close(g1, w1) && close(g2, w2),
        _ => false,
    }
}

/// An expected `Item`, built field by field; defaults are "one, other, nothing said".
struct W {
    item: Item,
}

impl W {
    fn new(text: &str) -> W {
        W {
            item: Item {
                qty: 1,
                kind: Kind::Other,
                material: String::new(),
                section_mm: None,
                length_mm: None,
                thickness_mm: None,
                sheet_mm: None,
                condition: None,
                notes: Vec::new(),
                text: text.to_string(),
            },
        }
    }

    fn qty(mut self, qty: u32) -> W {
        self.item.qty = qty;
        self
    }

    fn kind(mut self, kind: Kind) -> W {
        self.item.kind = kind;
        self
    }

    fn material(mut self, material: &str) -> W {
        self.item.material = material.to_string();
        self
    }

    fn section(mut self, thin: f64, thick: f64) -> W {
        self.item.section_mm = Some((thin, thick));
        self
    }

    fn length(mut self, mm: f64) -> W {
        self.item.length_mm = Some(mm);
        self
    }

    fn thickness(mut self, mm: f64) -> W {
        self.item.thickness_mm = Some(mm);
        self
    }

    fn sheet(mut self, short: f64, long: f64) -> W {
        self.item.sheet_mm = Some((short, long));
        self
    }

    fn condition(mut self, condition: &str) -> W {
        self.item.condition = Some(condition.to_string());
        self
    }

    fn notes(mut self, notes: &[&str]) -> W {
        self.item.notes = notes.iter().map(|s| s.to_string()).collect();
        self
    }
}

/// Every field of the item, compared whole; lengths within 1e-9 mm.
fn assert_item(got: &Item, want: W) {
    let want = want.item;
    assert_eq!(got.qty, want.qty, "qty for {:?}", want.text);
    assert_eq!(got.kind, want.kind, "kind for {:?}", want.text);
    assert_eq!(got.material, want.material, "material for {:?}", want.text);
    assert!(
        pair_close(got.section_mm, want.section_mm),
        "section_mm for {:?}: got {:?}, want {:?}",
        want.text,
        got.section_mm,
        want.section_mm
    );
    assert!(
        opt_close(got.length_mm, want.length_mm),
        "length_mm for {:?}: got {:?}, want {:?}",
        want.text,
        got.length_mm,
        want.length_mm
    );
    assert!(
        opt_close(got.thickness_mm, want.thickness_mm),
        "thickness_mm for {:?}: got {:?}, want {:?}",
        want.text,
        got.thickness_mm,
        want.thickness_mm
    );
    assert!(
        pair_close(got.sheet_mm, want.sheet_mm),
        "sheet_mm for {:?}: got {:?}, want {:?}",
        want.text,
        got.sheet_mm,
        want.sheet_mm
    );
    assert_eq!(
        got.condition, want.condition,
        "condition for {:?}",
        want.text
    );
    assert_eq!(got.notes, want.notes, "notes for {:?}", want.text);
    assert_eq!(got.text, want.text, "text for {:?}", want.text);
}

fn check(input: &str, want: W) {
    let got = quick_add(input);
    assert_item(&got, want);
}

#[test]
fn rule_1_empty_input_is_zero_other_and_empty() {
    check("", W::new("").qty(0));
    check("   ", W::new("").qty(0));
    check("\t\n ", W::new("").qty(0));
}

#[test]
fn rule_1_text_is_the_trimmed_input_not_lowercased() {
    // rule 1: text keeps the maker's case; only `material` is lower-cased.
    check(
        "  15 pine 2x4s, 3 foot  ",
        W::new("15 pine 2x4s, 3 foot")
            .qty(15)
            .kind(Kind::Board)
            .material("pine")
            .section(38.1, 88.9)
            .length(914.4),
    );
    check(
        "2 yards 500D Cordura",
        W::new("2 yards 500D Cordura").material("2 yards 500d cordura"),
    );
}

#[test]
fn rule_2_symbols_fold_before_splitting() {
    // U+00D7 folds to `x`, so the quantity word is followed by a droppable `x`.
    check(
        "15 \u{00d7} pine 2x4 36\"",
        W::new("15 \u{00d7} pine 2x4 36\"")
            .qty(15)
            .kind(Kind::Board)
            .material("pine")
            .section(38.1, 88.9)
            .length(914.4),
    );
    // U+00BE folds to ` 3/4` and U+2033 to `"`: 3/4 in = 19.05 mm.
    check(
        "1 sheet \u{00be}\u{2033} plywood 4x8",
        W::new("1 sheet \u{00be}\u{2033} plywood 4x8")
            .kind(Kind::Sheet)
            .material("plywood")
            .thickness(19.05)
            .sheet(1219.2, 2438.4)
            .notes(&["plywood is often thinner than its name: caliper it"]),
    );
    // U+2032 folds to `'`: 6 ft = 1828.8 mm.
    check(
        "2x4 6\u{2032}",
        W::new("2x4 6\u{2032}")
            .kind(Kind::Board)
            .section(38.1, 88.9)
            .length(1828.8),
    );
}

#[test]
fn rule_3_digit_quantity_from_first_word() {
    // 3 ft = 914.4 mm.
    check(
        "15 pine 2x4s, 3 foot",
        W::new("15 pine 2x4s, 3 foot")
            .qty(15)
            .kind(Kind::Board)
            .material("pine")
            .section(38.1, 88.9)
            .length(914.4),
    );
    // 10 ft = 3048.0 mm.
    check(
        "3 2x6 x 10'",
        W::new("3 2x6 x 10'")
            .qty(3)
            .kind(Kind::Board)
            .section(38.1, 139.7)
            .length(3048.0),
    );
}

#[test]
fn rule_3_measure_word_after_number_keeps_it_a_word() {
    // "5 kg PLA" is one item: the number stays a word, so material keeps it.
    check("5 kg PLA", W::new("5 kg PLA").material("5 kg pla"));
    check(
        "2 yards 500D Cordura",
        W::new("2 yards 500D Cordura").material("2 yards 500d cordura"),
    );
    // "ft" is a measure word too: qty 1, and "3 ft" becomes the board length.
    check(
        "3 ft 2x4",
        W::new("3 ft 2x4")
            .kind(Kind::Board)
            .section(38.1, 88.9)
            .length(914.4),
    );
}

#[test]
fn rule_3_x_after_digit_quantity_is_dropped() {
    check(
        "15 x pine 2x4 36\"",
        W::new("15 x pine 2x4 36\"")
            .qty(15)
            .kind(Kind::Board)
            .material("pine")
            .section(38.1, 88.9)
            .length(914.4),
    );
}

#[test]
fn rule_3_number_words_one_to_twelve() {
    // 8 ft = 2438.4 mm.
    check(
        "four 2x4s 8' long",
        W::new("four 2x4s 8' long")
            .qty(4)
            .kind(Kind::Board)
            .section(38.1, 88.9)
            .length(2438.4),
    );
    check(
        "twelve 2x4 8ft",
        W::new("twelve 2x4 8ft")
            .qty(12)
            .kind(Kind::Board)
            .section(38.1, 88.9)
            .length(2438.4),
    );
    check(
        "two IKEA Kallax units",
        W::new("two IKEA Kallax units")
            .qty(2)
            .material("ikea kallax units"),
    );
}

#[test]
fn rule_3_dozen_and_a_dozen_are_twelve() {
    // 6 ft = 1828.8 mm.
    check(
        "a dozen cedar 1x6 6 ft",
        W::new("a dozen cedar 1x6 6 ft")
            .qty(12)
            .kind(Kind::Board)
            .material("cedar")
            .section(19.05, 139.7)
            .length(1828.8),
    );
    check(
        "dozen cedar 1x6 6 ft",
        W::new("dozen cedar 1x6 6 ft")
            .qty(12)
            .kind(Kind::Board)
            .material("cedar")
            .section(19.05, 139.7)
            .length(1828.8),
    );
}

#[test]
fn rule_4_first_condition_word_only() {
    check(
        "6 used oak 1x4",
        W::new("6 used oak 1x4")
            .qty(6)
            .kind(Kind::Board)
            .material("oak")
            .section(19.05, 88.9)
            .condition("used"),
    );
    check(
        "an old bookshelf",
        W::new("an old bookshelf")
            .material("bookshelf")
            .condition("old"),
    );
    check(
        "scrap 2x4 8ft",
        W::new("scrap 2x4 8ft")
            .kind(Kind::Board)
            .section(38.1, 88.9)
            .length(2438.4)
            .condition("scrap"),
    );
    // The second condition word stays a word, so the length cannot be read.
    check(
        "new used 2x4 8ft",
        W::new("new used 2x4 8ft")
            .kind(Kind::Board)
            .section(38.1, 88.9)
            .condition("new")
            .notes(&["could not read the length: used 8ft"]),
    );
}

#[test]
fn rule_5_half_sheet() {
    // 3/4 in = 19.05 mm; a half sheet is 4' x 4' = 1219.2 x 1219.2 mm.
    check(
        "half sheet 3/4 ply",
        W::new("half sheet 3/4 ply")
            .kind(Kind::Sheet)
            .material("plywood")
            .thickness(19.05)
            .sheet(1219.2, 1219.2)
            .notes(&[
                "a half sheet is 4' x 4'",
                "plywood is often thinner than its name: caliper it",
            ]),
    );
}

#[test]
fn rule_5_quarter_sheet() {
    // 6 mm stays 6.0 mm; a quarter sheet is 2' x 4' = 609.6 x 1219.2 mm.
    check(
        "quarter sheet of 6mm birch ply",
        W::new("quarter sheet of 6mm birch ply")
            .kind(Kind::Sheet)
            .material("birch plywood")
            .thickness(6.0)
            .sheet(609.6, 1219.2)
            .notes(&[
                "a quarter sheet is 2' x 4'",
                "plywood is often thinner than its name: caliper it",
            ]),
    );
}

#[test]
fn rule_5_sheet_size_shorter_side_first() {
    // 8x4 is given longer side first: 4' x 8' = 1219.2 x 2438.4 mm.
    check(
        "3 sheets osb 8x4",
        W::new("3 sheets osb 8x4")
            .qty(3)
            .kind(Kind::Sheet)
            .material("osb")
            .sheet(1219.2, 2438.4),
    );
    // 2x4 ft = 609.6 x 1219.2 mm; 1/2 in = 12.7 mm; no material, so no caliper note.
    check(
        "sheet 2x4 1/2\"",
        W::new("sheet 2x4 1/2\"")
            .kind(Kind::Sheet)
            .thickness(12.7)
            .sheet(609.6, 1219.2),
    );
}

#[test]
fn rule_5_full_sheet_when_no_size() {
    // 1/2 in = 12.7 mm; a full sheet is 4' x 8' = 1219.2 x 2438.4 mm.
    check(
        "2 sheets 1/2\" mdf",
        W::new("2 sheets 1/2\" mdf")
            .qty(2)
            .kind(Kind::Sheet)
            .material("mdf")
            .thickness(12.7)
            .sheet(1219.2, 2438.4)
            .notes(&["a full sheet is 4' x 8'"]),
    );
}

#[test]
fn rule_5_acrylic_synonyms() {
    // 1/4 in = 6.35 mm; acrylic is not plywood, so no caliper note.
    check(
        "sheet of plexiglass 1/4\"",
        W::new("sheet of plexiglass 1/4\"")
            .kind(Kind::Sheet)
            .material("acrylic")
            .thickness(6.35)
            .sheet(1219.2, 2438.4)
            .notes(&["a full sheet is 4' x 8'"]),
    );
    check(
        "sheet of perspex 1/4\"",
        W::new("sheet of perspex 1/4\"")
            .kind(Kind::Sheet)
            .material("acrylic")
            .thickness(6.35)
            .sheet(1219.2, 2438.4)
            .notes(&["a full sheet is 4' x 8'"]),
    );
    // With a size given there is no full-sheet note.
    check(
        "sheet of perspex 4x8 1/4\"",
        W::new("sheet of perspex 4x8 1/4\"")
            .kind(Kind::Sheet)
            .material("acrylic")
            .thickness(6.35)
            .sheet(1219.2, 2438.4),
    );
}

#[test]
fn rule_5_thickness_unreadable_note() {
    check(
        "sheet of hardboard thin",
        W::new("sheet of hardboard thin")
            .kind(Kind::Sheet)
            .material("hardboard")
            .sheet(1219.2, 2438.4)
            .notes(&[
                "a full sheet is 4' x 8'",
                "could not read the thickness: thin",
            ]),
    );
    // The note carries the rest of the words, joined by single spaces.
    check(
        "sheet of mdf 1/2\" thick",
        W::new("sheet of mdf 1/2\" thick")
            .kind(Kind::Sheet)
            .material("mdf")
            .sheet(1219.2, 2438.4)
            .notes(&[
                "a full sheet is 4' x 8'",
                "could not read the thickness: 1/2\" thick",
            ]),
    );
}

#[test]
fn rule_6_board_section_and_length() {
    // 8 ft = 2438.4 mm.
    check(
        "2x4 8ft",
        W::new("2x4 8ft")
            .kind(Kind::Board)
            .section(38.1, 88.9)
            .length(2438.4),
    );
    // 10 ft = 3048.0 mm; the `x` filler is removed.
    check(
        "3 2x6 x 10'",
        W::new("3 2x6 x 10'")
            .qty(3)
            .kind(Kind::Board)
            .section(38.1, 139.7)
            .length(3048.0),
    );
    // 6 ft = 1828.8 mm.
    check(
        "a dozen cedar 1x6 6 ft",
        W::new("a dozen cedar 1x6 6 ft")
            .qty(12)
            .kind(Kind::Board)
            .material("cedar")
            .section(19.05, 139.7)
            .length(1828.8),
    );
}

#[test]
fn rule_6_length_without_unit_note() {
    // 96 in = 2438.4 mm, read as inches with a note.
    check(
        "2x4 96",
        W::new("2x4 96")
            .kind(Kind::Board)
            .section(38.1, 88.9)
            .length(2438.4)
            .notes(&["a length without a unit is read as inches"]),
    );
    // 8 in = 203.2 mm.
    check(
        "2x4 8",
        W::new("2x4 8")
            .kind(Kind::Board)
            .section(38.1, 88.9)
            .length(203.2)
            .notes(&["a length without a unit is read as inches"]),
    );
}

#[test]
fn rule_6_length_unreadable_note() {
    check(
        "2x4 about 8 feet",
        W::new("2x4 about 8 feet")
            .kind(Kind::Board)
            .section(38.1, 88.9)
            .notes(&["could not read the length: about 8 feet"]),
    );
}

#[test]
fn rule_7_other_keeps_remaining_words() {
    // No fillers are removed for Other: "of" stays.
    check(
        "a stack of newspapers",
        W::new("a stack of newspapers").material("stack of newspapers"),
    );
    check("5 kg PLA", W::new("5 kg PLA").material("5 kg pla"));
    check(
        "2 yards 500D Cordura",
        W::new("2 yards 500D Cordura").material("2 yards 500d cordura"),
    );
    check(
        "two IKEA Kallax units",
        W::new("two IKEA Kallax units")
            .qty(2)
            .material("ikea kallax units"),
    );
    check(
        "an old bookshelf",
        W::new("an old bookshelf")
            .material("bookshelf")
            .condition("old"),
    );
}

#[test]
fn worked_examples_table_all_rows() {
    check(
        "15 pine 2x4s, 3 foot",
        W::new("15 pine 2x4s, 3 foot")
            .qty(15)
            .kind(Kind::Board)
            .material("pine")
            .section(38.1, 88.9)
            .length(914.4),
    );
    check(
        "15 \u{00d7} pine 2x4 36\"",
        W::new("15 \u{00d7} pine 2x4 36\"")
            .qty(15)
            .kind(Kind::Board)
            .material("pine")
            .section(38.1, 88.9)
            .length(914.4),
    );
    check(
        "2x4 8ft",
        W::new("2x4 8ft")
            .kind(Kind::Board)
            .section(38.1, 88.9)
            .length(2438.4),
    );
    check(
        "3 2x6 x 10'",
        W::new("3 2x6 x 10'")
            .qty(3)
            .kind(Kind::Board)
            .section(38.1, 139.7)
            .length(3048.0),
    );
    check(
        "four 2x4s 8' long",
        W::new("four 2x4s 8' long")
            .qty(4)
            .kind(Kind::Board)
            .section(38.1, 88.9)
            .length(2438.4),
    );
    check(
        "a dozen cedar 1x6 6 ft",
        W::new("a dozen cedar 1x6 6 ft")
            .qty(12)
            .kind(Kind::Board)
            .material("cedar")
            .section(19.05, 139.7)
            .length(1828.8),
    );
    check(
        "2x4 96",
        W::new("2x4 96")
            .kind(Kind::Board)
            .section(38.1, 88.9)
            .length(2438.4)
            .notes(&["a length without a unit is read as inches"]),
    );
    check(
        "6 used oak 1x4",
        W::new("6 used oak 1x4")
            .qty(6)
            .kind(Kind::Board)
            .material("oak")
            .section(19.05, 88.9)
            .condition("used"),
    );
    check(
        "2x4 about 8 feet",
        W::new("2x4 about 8 feet")
            .kind(Kind::Board)
            .section(38.1, 88.9)
            .notes(&["could not read the length: about 8 feet"]),
    );
    check(
        "half sheet 3/4 ply",
        W::new("half sheet 3/4 ply")
            .kind(Kind::Sheet)
            .material("plywood")
            .thickness(19.05)
            .sheet(1219.2, 1219.2)
            .notes(&[
                "a half sheet is 4' x 4'",
                "plywood is often thinner than its name: caliper it",
            ]),
    );
    check(
        "1 sheet \u{00be}\u{2033} plywood 4x8",
        W::new("1 sheet \u{00be}\u{2033} plywood 4x8")
            .kind(Kind::Sheet)
            .material("plywood")
            .thickness(19.05)
            .sheet(1219.2, 2438.4)
            .notes(&["plywood is often thinner than its name: caliper it"]),
    );
    check(
        "2 sheets 1/2\" mdf",
        W::new("2 sheets 1/2\" mdf")
            .qty(2)
            .kind(Kind::Sheet)
            .material("mdf")
            .thickness(12.7)
            .sheet(1219.2, 2438.4)
            .notes(&["a full sheet is 4' x 8'"]),
    );
    check(
        "quarter sheet of 6mm birch ply",
        W::new("quarter sheet of 6mm birch ply")
            .kind(Kind::Sheet)
            .material("birch plywood")
            .thickness(6.0)
            .sheet(609.6, 1219.2)
            .notes(&[
                "a quarter sheet is 2' x 4'",
                "plywood is often thinner than its name: caliper it",
            ]),
    );
    check(
        "3 sheets osb 8x4",
        W::new("3 sheets osb 8x4")
            .qty(3)
            .kind(Kind::Sheet)
            .material("osb")
            .sheet(1219.2, 2438.4),
    );
    check(
        "sheet of hardboard thin",
        W::new("sheet of hardboard thin")
            .kind(Kind::Sheet)
            .material("hardboard")
            .sheet(1219.2, 2438.4)
            .notes(&[
                "a full sheet is 4' x 8'",
                "could not read the thickness: thin",
            ]),
    );
    check(
        "an old bookshelf",
        W::new("an old bookshelf")
            .material("bookshelf")
            .condition("old"),
    );
    check(
        "a stack of newspapers",
        W::new("a stack of newspapers").material("stack of newspapers"),
    );
    check("5 kg PLA", W::new("5 kg PLA").material("5 kg pla"));
    check(
        "2 yards 500D Cordura",
        W::new("2 yards 500D Cordura").material("2 yards 500d cordura"),
    );
    check(
        "two IKEA Kallax units",
        W::new("two IKEA Kallax units")
            .qty(2)
            .material("ikea kallax units"),
    );
    check("   ", W::new("").qty(0));
}

#[test]
fn junk_inputs_never_panic_and_keep_text() {
    let junk = [
        "",
        "   ",
        "\t\n",
        "!!!",
        "2x4",
        "sheet",
        "sheets",
        "half sheet",
        "quarter sheet",
        "a",
        "an",
        "one",
        "dozen",
        "a dozen",
        "x",
        "15 x",
        "\u{00bd}",
        "\u{00bc}",
        "\u{00be}",
        "\u{00d7}",
        "\u{2033}",
        "\u{2032}",
        ",",
        ",,,",
        "2x4 2x4 2x4",
        "sheet sheet",
        "half half sheet",
        "0 2x4",
        "9999 2x4",
        "12345 2x4",
        "2x4 -5",
        "sheet of",
        "of",
        "long each piece",
        "@",
        "1/0",
        "2x4 1/0",
        "sheet 1/0",
        "twelve",
        "eleven 2x4",
        "13x13 sheet",
        "0x0 sheet",
        "sheet 12x12",
        "PLYWOOD SHEET",
        "  half   sheet   3/4   ply  ",
        "2x4 8ft 9ft",
        "sheet of ply 1/2\" 3/4\"",
        "a a a",
        "used used 2x4",
        "2x4s 2x6s",
    ];
    for input in junk {
        let got = quick_add(input);
        // rule 1: text is always the trimmed input, whatever else happened.
        assert_eq!(got.text, input.trim(), "text for {:?}", input);
        if input.trim().is_empty() {
            assert_eq!(got.qty, 0, "qty for {:?}", input);
            assert_eq!(got.kind, Kind::Other, "kind for {:?}", input);
            assert_eq!(got.material, "", "material for {:?}", input);
            assert_eq!(got.notes, Vec::<String>::new(), "notes for {:?}", input);
        }
    }
}
