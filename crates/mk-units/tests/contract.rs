//! Contract tests for mk_units, written by Claude from crew/contracts/mk_units.md version 2
//! (ATLAS-api-harness). They were the hidden tests of campaigns mk-core-1 and mk-core-2; the
//! contract is final for now, so they are public here. New rules get new hidden tests first.

use mk_units::{format_inches, mm_per, nominal_lumber, parse_length, Unit, UnitError};

fn ok(text: &str, unit: Unit, want: f64) {
    match parse_length(text, unit) {
        Ok(got) => assert!(
            (got - want).abs() < 1e-9,
            "{text:?}: got {got}, want {want}"
        ),
        Err(e) => panic!("{text:?}: got {e:?}, want {want}"),
    }
}

#[test]
fn mm_per_unit_is_the_table() {
    assert_eq!(mm_per(Unit::Mm), 1.0);
    assert_eq!(mm_per(Unit::Cm), 10.0);
    assert_eq!(mm_per(Unit::M), 1000.0);
    assert_eq!(mm_per(Unit::In), 25.4);
    assert_eq!(mm_per(Unit::Ft), 304.8);
}

#[test]
fn a_bare_number_uses_the_default_unit() {
    ok("600", Unit::Mm, 600.0);
    ok("36", Unit::In, 914.4);
    ok("1 1/2", Unit::In, 38.1);
}

#[test]
fn an_explicit_unit_beats_the_default() {
    ok("600mm", Unit::In, 600.0);
    ok("60 cm", Unit::Mm, 600.0);
    ok("1.2 m", Unit::Mm, 1200.0);
    ok("3 ft", Unit::Mm, 914.4);
    ok("3 Foot", Unit::Mm, 914.4);
    ok("2 metres", Unit::In, 2000.0);
    ok("4 inches", Unit::Mm, 101.6);
}

#[test]
fn feet_and_inches_add_up() {
    ok("2'6\"", Unit::Mm, 762.0);
    ok("2' 6\"", Unit::Mm, 762.0);
    ok("2 ft 6 in", Unit::Mm, 762.0);
    ok("2ft6in", Unit::Mm, 762.0);
}

#[test]
fn a_bare_number_after_feet_is_inches_whatever_the_default() {
    ok("2' 6", Unit::Mm, 762.0);
    ok("2' 6", Unit::Cm, 762.0);
}

#[test]
fn fractions_and_mixed_numbers() {
    ok("35 7/16\"", Unit::Mm, 900.1125);
    ok("35-7/16 in", Unit::Mm, 900.1125);
    ok("3/4\"", Unit::Mm, 19.05);
    ok(".5 in", Unit::Mm, 12.7);
    ok("5' 3 1/2\"", Unit::Mm, 1612.9);
}

#[test]
fn unicode_primes_are_inch_and_foot_marks() {
    ok("6\u{2033}", Unit::Mm, 152.4);
    ok("1\u{2032} 1\u{2033}", Unit::Mm, 330.2);
}

#[test]
fn surrounding_spaces_are_ignored() {
    ok("   12 mm  ", Unit::In, 12.0);
}

#[test]
fn errors_say_what_went_wrong() {
    assert_eq!(parse_length("  ", Unit::Mm), Err(UnitError::Empty));
    assert_eq!(parse_length("", Unit::Mm), Err(UnitError::Empty));
    assert_eq!(
        parse_length("abc", Unit::Mm),
        Err(UnitError::NotANumber("abc".into()))
    );
    assert_eq!(
        parse_length(" 5 Parsecs ", Unit::Mm),
        Err(UnitError::NotANumber("5 Parsecs".into()))
    );
    assert_eq!(parse_length("3/0", Unit::In), Err(UnitError::DivideByZero));
    assert_eq!(parse_length("-5", Unit::Mm), Err(UnitError::Negative));
}

#[test]
fn malformed_lengths_are_not_numbers() {
    for bad in [
        "6\" 2'",
        "1.5.2",
        "12.",
        "2' 3' ",
        "1 2 3",
        "3 ft 4 ft",
        "2 6",
        "in",
        "/4",
    ] {
        let got = parse_length(bad, Unit::Mm);
        assert_eq!(
            got,
            Err(UnitError::NotANumber(bad.trim().into())),
            "{bad:?}"
        );
    }
}

#[test]
fn format_inches_rounds_to_the_nearest_fraction_and_reduces_it() {
    assert_eq!(format_inches(900.1125, 16), "35 7/16\"");
    assert_eq!(format_inches(914.4, 16), "36\"");
    assert_eq!(format_inches(19.05, 16), "3/4\"");
    assert_eq!(format_inches(38.1, 16), "1 1/2\"");
    assert_eq!(format_inches(25.4, 16), "1\"");
    assert_eq!(format_inches(1.0, 16), "1/16\"");
    assert_eq!(format_inches(100.0, 8), "3 7/8\"");
    assert_eq!(format_inches(100.0, 32), "3 15/16\"");
    assert_eq!(format_inches(2000.0, 64), "78 47/64\"");
}

#[test]
fn format_inches_zero_and_negative() {
    assert_eq!(format_inches(0.0, 16), "0\"");
    assert_eq!(format_inches(0.5, 16), "0\"");
    assert_eq!(format_inches(-19.05, 16), "-3/4\"");
    assert_eq!(format_inches(-0.3, 16), "0\"");
}

#[test]
fn a_denominator_that_is_not_a_power_of_two_up_to_64_means_16() {
    assert_eq!(format_inches(100.0, 3), "3 15/16\"");
    assert_eq!(format_inches(100.0, 0), "3 15/16\"");
    assert_eq!(format_inches(100.0, 128), "3 15/16\"");
    assert_eq!(format_inches(100.0, 1), "4\"");
}

#[test]
fn parse_then_format_round_trips_sixteenths() {
    for text in ["35 7/16\"", "3/4\"", "12\"", "1 1/16\"", "7 15/16\""] {
        let mm = parse_length(text, Unit::Mm).expect("parses");
        assert_eq!(format_inches(mm, 16), text);
    }
}

#[test]
fn nominal_lumber_is_the_dressed_size() {
    assert_eq!(nominal_lumber("2x4"), Some((38.1, 88.9)));
    assert_eq!(nominal_lumber("1x6"), Some((19.05, 139.7)));
    assert_eq!(nominal_lumber("2x12"), Some((38.1, 285.75)));
    assert_eq!(nominal_lumber("4x4"), Some((88.9, 88.9)));
    assert_eq!(nominal_lumber("6x6"), Some((139.7, 139.7)));
    assert_eq!(nominal_lumber("1x8"), Some((19.05, 184.15)));
}

#[test]
fn nominal_lumber_accepts_the_ways_people_write_it() {
    for s in [
        "2x4",
        "2 x 4",
        "2X4",
        "2\u{00d7}4",
        "2by4",
        " 2 By 4s ",
        "4x2",
        "2x4s",
    ] {
        assert_eq!(nominal_lumber(s), Some((38.1, 88.9)), "{s:?}");
    }
}

#[test]
fn nominal_lumber_unknown_is_none() {
    for s in ["2x5", "plywood", "", "2x", "x4", "2x4x8", "3x3"] {
        assert_eq!(nominal_lumber(s), None, "{s:?}");
    }
}

// ---- v2 (wave 2, 2026-11-19) ----

fn not_a_number(text: &str, unit: Unit) {
    assert_eq!(
        parse_length(text, unit),
        Err(UnitError::NotANumber(text.trim().into())),
        "{text:?}"
    );
}

#[test]
fn v2_a_dash_may_join_feet_and_inches() {
    ok("5'-6\"", Unit::Mm, 1676.4);
    ok("5' - 6\"", Unit::Mm, 1676.4);
    ok("5' -6\"", Unit::Mm, 1676.4);
    ok("5 ft-6 in", Unit::Mm, 1676.4);
    ok("5ft - 6in", Unit::Mm, 1676.4);
    ok("5'-6", Unit::Mm, 1676.4);
    ok("5'-6", Unit::Cm, 1676.4);
    ok("5'-6 1/2\"", Unit::Mm, 1689.1);
    ok("5'-6-1/2\"", Unit::Mm, 1689.1);
    ok("5'-1/2\"", Unit::Mm, 1536.7);
    ok("5\u{2032}-6\u{2033}", Unit::Mm, 1676.4);
}

#[test]
fn v2_a_dash_inside_a_mixed_number_is_unchanged() {
    ok("5-1/2'", Unit::Mm, 1676.4);
    ok("35-7/16 in", Unit::Mm, 900.1125);
}

#[test]
fn v2_a_dash_anywhere_else_is_not_a_number() {
    for bad in [
        "5'-",
        "5' - ",
        "5'--6\"",
        "5' - - 6\"",
        "5 - 6",
        "5-6",
        "6\"-2'",
        "5'-6\"-2",
        "5 mm-6",
    ] {
        not_a_number(bad, Unit::In);
    }
}

#[test]
fn v2_fifteen_digits_is_the_limit() {
    ok("123456789012345", Unit::Mm, 123456789012345.0);
    ok("999999999999999/3", Unit::Mm, 333333333333333.0);
    ok("25.400000000000002 mm", Unit::In, 25.400000000000002);
    ok("1.1234567890123456789 mm", Unit::In, 1.1234567890123457);
    for bad in [
        "1234567890123456",
        "123456789012345678901234",
        "1234567890123456/2",
        "1/1234567890123456",
        "1/0000000000000000",
        "1234567890123456.5",
        "1 1234567890123456/2",
        "2' 1234567890123456\"",
    ] {
        not_a_number(bad, Unit::Mm);
    }
}

#[test]
fn v2_nothing_may_follow_the_second_quantity() {
    for bad in ["2' 6\" x", "2'6\"x", "2' 6 7", "2 ft 6 in 7"] {
        not_a_number(bad, Unit::Mm);
    }
}
