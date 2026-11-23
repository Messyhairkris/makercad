//! The measuring system a document keeps its lengths in (D34), and what happens to a length
//! typed into it: it is read however it was written, shown in the document's own system, and,
//! once the maker confirms it, kept at the document's precision.
//!
//! Underneath every length is still millimetres (`parse_length`'s own unit), so changing a
//! document's system later changes how lengths are written and read, never what they are.

use crate::{format_inches, mm_per, parse_length, Unit, UnitError};

/// The finest inch fraction an imperial document keeps: 1/64".
pub const IMPERIAL_DENOM: u32 = 64;

/// A length rounded by more than this many millimetres is worth saying so (a hundredth of a
/// millimetre is finer than a maker can measure).
const WORTH_SAYING_MM: f64 = 0.01;

/// The system a document measures in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum System {
    /// Millimetres; a bare number is millimetres.
    #[default]
    Metric,
    /// Inches and fractions of an inch; a bare number is inches.
    Imperial,
}

impl System {
    /// Both systems, metric first.
    pub const ALL: [System; 2] = [System::Metric, System::Imperial];

    /// The key a file stores. Never change one: files already hold it.
    pub fn key(self) -> &'static str {
        match self {
            System::Metric => "metric",
            System::Imperial => "imperial",
        }
    }

    /// The system stored under `key`, if it is one.
    pub fn from_key(key: &str) -> Option<System> {
        System::ALL.into_iter().find(|s| s.key() == key)
    }

    /// What to call it for a person.
    pub fn name(self) -> &'static str {
        match self {
            System::Metric => "Metric (millimetres)",
            System::Imperial => "Imperial (inches)",
        }
    }

    /// The unit a bare number is read in.
    pub fn bare_unit(self) -> Unit {
        match self {
            System::Metric => Unit::Mm,
            System::Imperial => Unit::In,
        }
    }

    /// Ways of writing a length that this system's people use, for "try ..." hints.
    pub fn examples(self) -> &'static str {
        match self {
            System::Metric => "1200 mm, 120 cm or 1.2 m",
            System::Imperial => "48 in, 4 ft or 3 1/2\"",
        }
    }

    /// What rounding to the system's precision is called, for "(rounded to ...)".
    pub fn precision(self) -> &'static str {
        match self {
            System::Metric => "0.01 mm",
            System::Imperial => "1/64\"",
        }
    }
}

/// A typed length, read and put into a document's system.
#[derive(Debug, Clone, PartialEq)]
pub struct Reading {
    /// What was typed, trimmed.
    pub typed: String,
    /// The length in millimetres as typed, before the document's precision was applied.
    pub exact_mm: f64,
    /// The length in millimetres at the document's precision: what is kept.
    pub kept_mm: f64,
    /// `kept_mm` written in the document's system ("3 1/2\"" or "88.9 mm"): what replaces the
    /// typing when it is confirmed.
    pub shown: String,
    /// Whether the document's precision moved the length by more than a hundredth of a mm.
    pub rounded: bool,
    /// Whether confirming changes the text (it was written another way, or with no unit).
    pub changed: bool,
}

/// Rounds a length to what a document in `system` keeps: hundredths of a millimetre, or 64ths
/// of an inch.
pub fn keep(mm: f64, system: System) -> f64 {
    match system {
        System::Metric => (mm * 100.0).round() / 100.0,
        System::Imperial => {
            let inch = mm_per(Unit::In);
            let steps = IMPERIAL_DENOM as f64;
            (mm / inch * steps).round() / steps * inch
        }
    }
}

/// Writes a length in `system`, at its precision: "88.9 mm", "1200 mm", `3 1/2"`, `12"`.
pub fn show(mm: f64, system: System) -> String {
    match system {
        System::Metric => {
            let text = format!("{:.2}", keep(mm, system));
            let text = text.trim_end_matches('0').trim_end_matches('.');
            format!("{text} mm")
        }
        System::Imperial => format_inches(mm, IMPERIAL_DENOM),
    }
}

/// Reads a length typed in any way `parse_length` knows (millimetres, centimetres, metres,
/// inches, fractions, feet and inches) and puts it into `system`. A bare number is in the
/// system's own unit.
pub fn read(text: &str, system: System) -> Result<Reading, UnitError> {
    let exact_mm = parse_length(text, system.bare_unit())?;
    let kept_mm = keep(exact_mm, system);
    let shown = show(kept_mm, system);
    let typed = text.trim().to_string();
    Ok(Reading {
        rounded: (exact_mm - kept_mm).abs() > WORTH_SAYING_MM,
        changed: writing(&typed) != writing(&shown),
        typed,
        exact_mm,
        kept_mm,
        shown,
    })
}

/// A length's writing with case, spaces and the prime marks folded, to tell "88.9mm" from
/// "88.9 mm" as the same thing.
fn writing(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| match c {
            '\u{2033}' => '"',
            '\u{2032}' => '\'',
            other => other,
        })
        .collect::<String>()
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn near(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn a_system_has_a_stable_key_a_name_and_a_bare_unit() {
        assert_eq!(System::default(), System::Metric);
        assert_eq!(System::ALL, [System::Metric, System::Imperial]);
        for s in System::ALL {
            assert_eq!(System::from_key(s.key()), Some(s));
        }
        assert_eq!(System::Metric.key(), "metric");
        assert_eq!(System::Imperial.key(), "imperial");
        assert_eq!(System::from_key("cubits"), None);
        assert_eq!(System::from_key("Metric"), None);
        assert_eq!(System::Metric.bare_unit(), Unit::Mm);
        assert_eq!(System::Imperial.bare_unit(), Unit::In);
        assert_ne!(System::Metric.name(), System::Imperial.name());
        assert!(System::Metric.examples().contains("mm"));
        assert!(System::Imperial.examples().contains("ft"));
        assert_eq!(System::Metric.precision(), "0.01 mm");
        assert_eq!(System::Imperial.precision(), "1/64\"");
    }

    #[test]
    fn any_way_of_writing_a_length_comes_out_in_the_documents_system() {
        let r = read("5 ft 6 in", System::Metric).unwrap();
        assert!(near(r.kept_mm, 1676.4));
        assert_eq!(r.shown, "1676.4 mm");
        assert!(r.changed);
        assert!(!r.rounded);
        let r = read("3 1/2 in", System::Imperial).unwrap();
        assert!(near(r.kept_mm, 88.9));
        assert_eq!(r.shown, "3 1/2\"");
        let r = read("120 cm", System::Imperial).unwrap();
        assert_eq!(r.shown, "47 1/4\"");
        let r = read("1.2 m", System::Metric).unwrap();
        assert_eq!(r.shown, "1200 mm");
        let r = read("5'-6\"", System::Metric).unwrap();
        assert_eq!(r.shown, "1676.4 mm");
        let r = read("2 feet 3 inches", System::Imperial).unwrap();
        assert_eq!(r.shown, "27\"");
    }

    #[test]
    fn a_bare_number_is_in_the_documents_own_unit() {
        assert_eq!(read("1200", System::Metric).unwrap().shown, "1200 mm");
        assert_eq!(read("12", System::Imperial).unwrap().shown, "12\"");
        assert_eq!(read("1.5", System::Metric).unwrap().shown, "1.5 mm");
        assert_eq!(read("1/2", System::Imperial).unwrap().shown, "1/2\"");
        assert!(read("1200", System::Metric).unwrap().changed);
    }

    #[test]
    fn writing_it_already_in_the_documents_form_changes_nothing() {
        let r = read("88.9 mm", System::Metric).unwrap();
        assert!(!r.changed);
        let r = read("  88.9MM ", System::Metric).unwrap();
        assert!(!r.changed, "case and spaces are not a change");
        assert_eq!(r.typed, "88.9MM");
        let r = read("3 1/2\"", System::Imperial).unwrap();
        assert!(!r.changed);
        let r = read("3\u{2033}", System::Imperial).unwrap();
        assert!(!r.changed, "a prime mark is the same as a quote mark");
        assert!(read("3 in", System::Imperial).unwrap().changed);
    }

    #[test]
    fn a_length_is_rounded_to_what_the_document_keeps_and_says_so() {
        // 10 mm is between 25/64" (9.92 mm) and 26/64" (10.32 mm).
        let r = read("10 mm", System::Imperial).unwrap();
        assert_eq!(r.shown, "25/64\"");
        assert!(r.rounded);
        assert!(near(r.exact_mm, 10.0));
        assert!((r.kept_mm - 9.921875).abs() < 1e-9);
        // An exact inch fraction is not rounded at all.
        let r = read("3.175 mm", System::Imperial).unwrap();
        assert_eq!(r.shown, "1/8\"");
        assert!(!r.rounded);
        // Metric keeps hundredths, which moves a length by at most half a hundredth: never
        // enough to remark on.
        let r = read("12.3456 mm", System::Metric).unwrap();
        assert_eq!(r.shown, "12.35 mm");
        assert!(!r.rounded);
        let r = read("12.3449 in", System::Metric).unwrap();
        assert_eq!(r.shown, "313.56 mm");
        assert!(!r.rounded);
        let r = read("0.0049 mm", System::Metric).unwrap();
        assert_eq!(r.shown, "0 mm");
        assert!(!r.rounded);
    }

    #[test]
    fn kept_lengths_are_at_the_documents_precision() {
        assert!(near(keep(88.904, System::Metric), 88.9));
        assert!(near(keep(88.906, System::Metric), 88.91));
        assert!(near(keep(10.0, System::Imperial), 9.921875));
        assert!(near(keep(25.4, System::Imperial), 25.4));
        assert!(near(keep(0.0, System::Imperial), 0.0));
    }

    #[test]
    fn showing_a_length_trims_what_adds_nothing() {
        assert_eq!(show(1000.0, System::Metric), "1000 mm");
        assert_eq!(show(100.0, System::Metric), "100 mm");
        assert_eq!(show(88.9, System::Metric), "88.9 mm");
        assert_eq!(show(0.5, System::Metric), "0.5 mm");
        assert_eq!(show(0.0, System::Metric), "0 mm");
        assert_eq!(show(25.4, System::Imperial), "1\"");
        assert_eq!(show(304.8, System::Imperial), "12\"");
        assert_eq!(show(0.0, System::Imperial), "0\"");
        assert_eq!(show(38.1, System::Imperial), "1 1/2\"");
    }

    #[test]
    fn what_cannot_be_read_says_why() {
        assert_eq!(read("", System::Metric), Err(UnitError::Empty));
        assert_eq!(read("   ", System::Imperial), Err(UnitError::Empty));
        assert_eq!(
            read("a lot", System::Metric),
            Err(UnitError::NotANumber("a lot".into()))
        );
        assert_eq!(read("-3", System::Metric), Err(UnitError::Negative));
        assert_eq!(read("3/0", System::Imperial), Err(UnitError::DivideByZero));
    }

    #[test]
    fn what_is_shown_reads_back_the_same_so_confirming_twice_changes_nothing() {
        for n in 0..=640u32 {
            let mm = f64::from(n) / 64.0 * 25.4;
            let r = read(&show(mm, System::Imperial), System::Imperial).unwrap();
            assert!(near(r.kept_mm, keep(mm, System::Imperial)), "{n}/64");
            assert!(!r.changed, "{n}/64 -> {}", r.shown);
        }
        for n in 0..=2000u32 {
            let mm = f64::from(n) * 0.37;
            let first = read(&format!("{mm}"), System::Metric).unwrap();
            let again = read(&first.shown, System::Metric).unwrap();
            assert_eq!(again.shown, first.shown, "{mm}");
            assert!(near(again.kept_mm, first.kept_mm), "{mm}");
            assert!(!again.changed, "{mm}");
        }
    }

    #[test]
    fn changing_systems_and_back_loses_at_most_the_precision() {
        let metric = read("1500 mm", System::Metric).unwrap();
        let imperial = read(&metric.shown, System::Imperial).unwrap();
        assert_eq!(imperial.shown, "59 1/16\"");
        let back = read(&imperial.shown, System::Metric).unwrap();
        assert!((back.kept_mm - 1500.0).abs() < 0.2, "{}", back.kept_mm);
    }
}
