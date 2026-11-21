//! Integration tests for `mk_units`, written from the contract.

use mk_units::{format_inches, mm_per, nominal_lumber, parse_length, Unit, UnitError};

/// Parses `text` and checks the result is within 1e-9 mm of `want_mm`.
fn assert_parses(text: &str, default: Unit, want_mm: f64) {
    let got = parse_length(text, default).unwrap();
    assert!(
        (got - want_mm).abs() < 1e-9,
        "{text}: got {got}, want {want_mm}"
    );
}

/// Checks two lengths agree within 1e-9 mm.
fn assert_close(got: f64, want: f64) {
    assert!((got - want).abs() < 1e-9, "got {got}, want {want}");
}

/// Checks a lumber size against the table, within 1e-9 mm per side.
fn assert_lumber(name: &str, want: Option<(f64, f64)>) {
    let got = nominal_lumber(name);
    match (got, want) {
        (Some((a, b)), Some((c, d))) => {
            assert!((a - c).abs() < 1e-9, "{name}: thin side got {a}, want {c}");
            assert!((b - d).abs() < 1e-9, "{name}: wide side got {b}, want {d}");
        }
        (g, w) => assert_eq!(g, w, "{name}"),
    }
}

/// Checks `text` is rejected as `NotANumber` carrying the trimmed original text (rule 9).
fn assert_not_a_number(text: &str, default: Unit) {
    assert_eq!(
        parse_length(text, default),
        Err(UnitError::NotANumber(text.trim().to_string())),
        "{text}"
    );
}

#[test]
fn mm_per_table_values() {
    // API: Mm 1, Cm 10, M 1000, In 25.4, Ft 304.8
    assert_close(mm_per(Unit::Mm), 1.0);
    assert_close(mm_per(Unit::Cm), 10.0);
    assert_close(mm_per(Unit::M), 1000.0);
    assert_close(mm_per(Unit::In), 25.4);
    assert_close(mm_per(Unit::Ft), 304.8);
}

#[test]
fn parse_length_rule_1_trim_and_empty() {
    // nothing but spaces (rule 1)
    assert_eq!(parse_length("  ", Unit::Mm), Err(UnitError::Empty));
    // surrounding spaces are trimmed before parsing (rule 1)
    assert_parses("  600  ", Unit::Mm, 600.0);
}

#[test]
fn parse_length_rule_2_normalise_case_and_prime_chars() {
    // lower-cased before matching (rule 2)
    assert_parses("3 Foot", Unit::Mm, 914.4);
    // U+2033 and U+2032 become " and ' (rule 2)
    assert_parses("6\u{2033}", Unit::Mm, 152.4);
    assert_parses("2\u{2032}6\u{2033}", Unit::Mm, 762.0);
}

#[test]
fn parse_length_rule_3_negative() {
    // text starting with "-" (rule 3)
    assert_eq!(parse_length("-5", Unit::Mm), Err(UnitError::Negative));
    assert_eq!(parse_length("-5 in", Unit::Mm), Err(UnitError::Negative));
}

#[test]
fn parse_length_rule_4_number_forms() {
    // whole number and decimals, including a leading dot (rule 4)
    assert_parses("12", Unit::Mm, 12.0);
    assert_parses("1.5", Unit::Mm, 1.5);
    assert_parses(".5 in", Unit::Mm, 12.7);
    // "12." is not a number: digits must follow the dot (rule 4)
    assert_eq!(
        parse_length("12.", Unit::Mm),
        Err(UnitError::NotANumber("12.".to_string()))
    );
    // plain fraction (rule 4)
    assert_parses("3/4\"", Unit::Mm, 19.05);
    // mixed number with a space and with a hyphen (rule 4)
    assert_parses("35 7/16\"", Unit::Mm, 900.1125);
    assert_parses("35-7/16 in", Unit::Mm, 900.1125);
}

#[test]
fn parse_length_rule_5_all_unit_words() {
    // every accepted spelling; the default unit is chosen so that ignoring
    // the word would give a different answer (rule 5)
    for w in [
        "mm",
        "millimeter",
        "millimeters",
        "millimetre",
        "millimetres",
    ] {
        assert_parses(&format!("10 {w}"), Unit::In, 10.0);
    }
    for w in [
        "cm",
        "centimeter",
        "centimeters",
        "centimetre",
        "centimetres",
    ] {
        assert_parses(&format!("10 {w}"), Unit::In, 100.0);
    }
    for w in ["m", "meter", "meters", "metre", "metres"] {
        assert_parses(&format!("1 {w}"), Unit::Mm, 1000.0);
    }
    for w in ["in", "inch", "inches"] {
        assert_parses(&format!("1 {w}"), Unit::Mm, 25.4);
    }
    assert_parses("1\"", Unit::Mm, 25.4);
    for w in ["ft", "foot", "feet"] {
        assert_parses(&format!("1 {w}"), Unit::Mm, 304.8);
    }
    assert_parses("1'", Unit::Mm, 304.8);
    // a unit may follow a number with no space between them (rule 5)
    assert_parses("600mm", Unit::In, 600.0);
    assert_parses("1in", Unit::Mm, 25.4);
}

#[test]
fn parse_length_rule_6_one_quantity_default_unit() {
    // a number with no unit is in default_unit (rule 6)
    assert_parses("600", Unit::Mm, 600.0);
    assert_parses("36", Unit::In, 914.4);
    assert_parses("1 1/2", Unit::In, 38.1);
}

#[test]
fn parse_length_rule_7_feet_then_inches() {
    // feet then inches, with and without spaces (rule 7)
    assert_parses("2'6\"", Unit::Mm, 762.0);
    assert_parses("2' 6\"", Unit::Mm, 762.0);
    assert_parses("2 ft 6 in", Unit::Mm, 762.0);
    assert_parses("2ft6in", Unit::Mm, 762.0);
    // a bare number after feet is inches even with default_unit Mm (rule 7)
    assert_parses("2' 6", Unit::Mm, 762.0);
    // one quantity in feet (rule 6)
    assert_parses("3 ft", Unit::Mm, 914.4);
}

#[test]
fn parse_length_rule_8_divide_by_zero() {
    // a fraction with 0 below the line (rule 8)
    assert_eq!(parse_length("3/0", Unit::In), Err(UnitError::DivideByZero));
    assert_eq!(
        parse_length("3/0 in", Unit::In),
        Err(UnitError::DivideByZero)
    );
}

#[test]
fn parse_length_rule_9_not_a_number() {
    // unknown text and unknown unit (rule 9)
    assert_eq!(
        parse_length("abc", Unit::Mm),
        Err(UnitError::NotANumber("abc".to_string()))
    );
    assert_eq!(
        parse_length("5 Parsecs", Unit::Mm),
        Err(UnitError::NotANumber("5 Parsecs".to_string()))
    );
    // malformed two-quantity texts: inches before feet, two feet quantities
    assert_eq!(
        parse_length("6\" 2'", Unit::Mm),
        Err(UnitError::NotANumber("6\" 2'".to_string()))
    );
    assert_eq!(
        parse_length("2 ft 3 ft", Unit::Mm),
        Err(UnitError::NotANumber("2 ft 3 ft".to_string()))
    );
    // stray characters (rule 9)
    assert_eq!(
        parse_length("1.5.2", Unit::Mm),
        Err(UnitError::NotANumber("1.5.2".to_string()))
    );
    // the error carries the trimmed text, not lower-cased (rule 9)
    assert_eq!(
        parse_length(" 5 PARSECS ", Unit::Mm),
        Err(UnitError::NotANumber("5 PARSECS".to_string()))
    );
}

#[test]
fn format_inches_worked_examples() {
    // reduced fractions and whole inches (rules 4-5)
    assert_eq!(format_inches(900.1125, 16), "35 7/16\"");
    assert_eq!(format_inches(914.4, 16), "36\"");
    assert_eq!(format_inches(19.05, 16), "3/4\"");
    assert_eq!(format_inches(38.1, 16), "1 1/2\"");
    // zero, and rounding down to zero (rules 2-3)
    assert_eq!(format_inches(0.0, 16), "0\"");
    assert_eq!(format_inches(1.0, 16), "1/16\"");
    assert_eq!(format_inches(0.5, 16), "0\"");
    // negative values, never "-0\"" (rule 5)
    assert_eq!(format_inches(-19.05, 16), "-3/4\"");
    assert_eq!(format_inches(-0.3, 16), "0\"");
    // other allowed denominators
    assert_eq!(format_inches(100.0, 8), "3 7/8\"");
    assert_eq!(format_inches(100.0, 32), "3 15/16\"");
    assert_eq!(format_inches(2000.0, 64), "78 47/64\"");
}

#[test]
fn format_inches_rule_1_denominator_fallback() {
    // any denom not in {1, 2, 4, 8, 16, 32, 64} is treated as 16 (rule 1)
    assert_eq!(format_inches(100.0, 3), "3 15/16\"");
    assert_eq!(format_inches(100.0, 0), "3 15/16\"");
    assert_eq!(format_inches(100.0, 128), "3 15/16\"");
    // allowed denoms are used as given (rule 1)
    assert_eq!(format_inches(25.4, 1), "1\"");
    assert_eq!(format_inches(12.7, 2), "1/2\"");
    assert_eq!(format_inches(6.35, 4), "1/4\"");
}

#[test]
fn nominal_lumber_rule_1_spellings() {
    // x, x, by, optional spaces, case, one trailing s (rule 1)
    for name in [
        "2x4",
        "2 x 4",
        "2X4",
        "2\u{d7}4",
        "2by4",
        "2 by 4",
        "2x4s",
        " 2 By 4s ",
    ] {
        assert_lumber(name, Some((38.1, 88.9)));
    }
    // the two numbers may come in either order (rule 2)
    assert_lumber("4x2", Some((38.1, 88.9)));
}

#[test]
fn nominal_lumber_rule_3_table() {
    assert_lumber("1x2", Some((19.05, 38.1)));
    assert_lumber("1x3", Some((19.05, 63.5)));
    assert_lumber("1x4", Some((19.05, 88.9)));
    assert_lumber("1x6", Some((19.05, 139.7)));
    assert_lumber("1x8", Some((19.05, 184.15)));
    assert_lumber("1x10", Some((19.05, 234.95)));
    assert_lumber("1x12", Some((19.05, 285.75)));
    assert_lumber("2x2", Some((38.1, 38.1)));
    assert_lumber("2x3", Some((38.1, 63.5)));
    assert_lumber("2x4", Some((38.1, 88.9)));
    assert_lumber("2x6", Some((38.1, 139.7)));
    assert_lumber("2x8", Some((38.1, 184.15)));
    assert_lumber("2x10", Some((38.1, 234.95)));
    assert_lumber("2x12", Some((38.1, 285.75)));
    assert_lumber("4x4", Some((88.9, 88.9)));
    assert_lumber("4x6", Some((88.9, 139.7)));
    assert_lumber("6x6", Some((139.7, 139.7)));
}

#[test]
fn nominal_lumber_unknown_size() {
    // anything not in the table is None (rule 3)
    assert_lumber("2x5", None);
    assert_lumber("plywood", None);
    assert_lumber("", None);
}

#[test]
fn parse_length_rule_7_dash_join_accepted() {
    // v2: ONE `-` may join feet and inches (rule 7).
    // 5 ft = 5 * 304.8 = 1524 mm; 6 in = 6 * 25.4 = 152.4 mm; sum = 1676.4 mm.
    assert_parses("5'-6\"", Unit::Mm, 1676.4);
    assert_parses("5' - 6\"", Unit::Mm, 1676.4);
    assert_parses("5 ft-6 in", Unit::Mm, 1676.4);
    // the second quantity is inches even when default_unit is Cm (rule 7)
    assert_parses("5'-6", Unit::Cm, 1676.4);
    // 6 1/2 in = 6.5 * 25.4 = 165.1 mm, so 1524 + 165.1 = 1689.1 mm
    assert_parses("5'-6 1/2\"", Unit::Mm, 1689.1);
    // dash join, then a mixed number joined by its own `-` (rule 4)
    assert_parses("5'-6-1/2\"", Unit::Mm, 1689.1);
}

#[test]
fn parse_length_rule_7_dash_join_rejected() {
    // the `-` needs an inches quantity after it (rule 7)
    assert_not_a_number("5'-", Unit::Mm);
    // only ONE `-` is allowed there (rule 7)
    assert_not_a_number("5'--6\"", Unit::Mm);
    // the quantity before the `-` must be feet (rule 7)
    assert_not_a_number("5 - 6", Unit::In);
    // inches before feet is not a length (rule 7)
    assert_not_a_number("6\"-2'", Unit::Mm);
}

#[test]
fn parse_length_rule_7_dash_join_is_not_a_mixed_number() {
    // `5-1/2'` is a mixed number of feet, not a dash join: 5.5 * 304.8 = 1676.4 mm
    assert_parses("5-1/2'", Unit::Mm, 1676.4);
    // the same length as the dash-joined 5'-6"
    assert_parses("5'-6\"", Unit::Mm, 1676.4);
}

#[test]
fn parse_length_rule_4_digit_limit_whole_number() {
    // 15 digits is a whole number (rule 4)
    assert_parses("123456789012345", Unit::Mm, 123456789012345.0);
    // 16 digits is NotANumber, never a number (rule 4)
    assert_not_a_number("1234567890123456", Unit::Mm);
}

#[test]
fn parse_length_rule_4_digit_limit_before_decimal_point() {
    // 15 digits before the `.` is fine (rule 4)
    assert_parses("123456789012345.5", Unit::Mm, 123456789012345.5);
    // 16 digits before the `.` is NotANumber (rule 4)
    assert_not_a_number("1234567890123456.5", Unit::Mm);
}

#[test]
fn parse_length_rule_4_digit_limit_each_side_of_slash() {
    // 15 digits on each side of the `/` is a fraction: 123456789012345/123456789012345 = 1
    assert_parses("123456789012345/123456789012345", Unit::Mm, 1.0);
    // 16 digits above the line (rule 4)
    assert_not_a_number("1234567890123456/2", Unit::Mm);
    // 16 digits below the line (rule 4)
    assert_not_a_number("1/1234567890123456", Unit::Mm);
    // a long run of digits is NotANumber, never DivideByZero (rule 4)
    assert_not_a_number("1234567890123456/0", Unit::Mm);
}

#[test]
fn parse_length_rule_4_digit_limit_inside_mixed_number() {
    // 15 digits in the fraction of a mixed number is fine: 1 + 1 = 2 (rule 4)
    assert_parses("1 123456789012345/123456789012345", Unit::Mm, 2.0);
    // 16 digits in the fraction of a mixed number is NotANumber (rule 4)
    assert_not_a_number("1 1234567890123456/2", Unit::Mm);
}

#[test]
fn parse_length_rule_4_digits_after_dot_unlimited() {
    // the digits after a `.` have no limit (rule 4)
    assert_parses("25.400000000000002 mm", Unit::In, 25.400000000000002);
    assert_parses("25.400000000000002", Unit::Mm, 25.400000000000002);
}

#[test]
fn parse_length_rule_9_text_after_second_quantity() {
    // anything after the second quantity is NotANumber (rule 9)
    assert_not_a_number("2' 6\" x", Unit::Mm);
    // three quantities is NotANumber too (rule 9)
    assert_not_a_number("2' 6\" 3", Unit::Mm);
}
