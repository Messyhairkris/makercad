//! A length typed into a document (D34): read however it was written, shown in the document's
//! own measuring system while it is typed, and kept in that system once the maker confirms it
//! (Enter, or moving on to something else).
//!
//! The reading is `mk_units::system`'s; this is what a person sees of it. Any length box in the
//! app (the Quest's longest size now; a design's variables when they come) uses [`length_field`],
//! so every one of them behaves the same way.

use egui::{Response, Ui};
use mk_units::system::{self, System};
use mk_units::UnitError;

use crate::quest_screen::{field, muted};
use crate::theme::Palette;

/// What to say under a length box while it is being typed in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Preview {
    /// Nothing typed yet: nothing to say.
    Blank,
    /// It reads: what it comes to in the document's system.
    Reads(String),
    /// It does not read: why, and what would.
    Unreadable(String),
}

/// What `text` comes to in `system`, in words, for under the box.
pub fn preview(text: &str, system: System) -> Preview {
    match system::read(text, system) {
        Ok(r) if !r.changed => Preview::Reads(format!("Kept as {}.", r.shown)),
        Ok(r) if r.rounded => Preview::Reads(format!(
            "That is {}, to the nearest {}. Press Enter to keep it.",
            r.shown,
            system.precision()
        )),
        Ok(r) => Preview::Reads(format!("That is {}. Press Enter to keep it.", r.shown)),
        Err(UnitError::Empty) => Preview::Blank,
        Err(UnitError::NotANumber(_)) => Preview::Unreadable(format!(
            "I can't read that as a length yet (try {}).",
            system.examples()
        )),
        Err(UnitError::DivideByZero) => {
            Preview::Unreadable("A fraction can't have 0 below the line.".into())
        }
        Err(UnitError::Negative) => Preview::Unreadable("A length can't be below zero.".into()),
    }
}

/// Confirms what is typed: the text becomes the length written in `system`, and the length (at
/// the document's precision, in millimetres) is returned. Text that does not read is left alone
/// and gives `None`.
pub fn confirm(text: &mut String, system: System) -> Option<f64> {
    let reading = system::read(text, system).ok()?;
    if reading.shown != *text {
        text.clone_from(&reading.shown);
    }
    Some(reading.kept_mm)
}

/// Confirms `text` when the box has just lost focus (Enter, or the maker moved on).
pub fn confirm_on_leave(response: &Response, text: &mut String, system: System) -> Option<f64> {
    if response.lost_focus() {
        confirm(text, system)
    } else {
        None
    }
}

/// Says what the typing comes to, in the muted voice, under the box.
pub fn say(ui: &mut Ui, p: &Palette, text: &str, system: System) {
    match preview(text, system) {
        Preview::Blank => {}
        Preview::Reads(said) | Preview::Unreadable(said) => muted(ui, p, &said),
    }
}

/// A one-line length box for a document that measures in `system`: type it any way, see what it
/// comes to as you go, and it becomes the document's own writing when confirmed. Returns the
/// confirmed length in millimetres on the frame the maker confirms it.
pub fn length_field(
    ui: &mut Ui,
    p: &Palette,
    text: &mut String,
    system: System,
    hint: &str,
) -> Option<f64> {
    let response = field(ui, text, hint);
    let kept = confirm_on_leave(&response, text, system);
    say(ui, p, text, system);
    kept
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typing_says_what_it_comes_to_in_the_documents_system() {
        assert_eq!(preview("", System::Metric), Preview::Blank);
        assert_eq!(preview("   ", System::Imperial), Preview::Blank);
        assert_eq!(
            preview("5 ft 6 in", System::Metric),
            Preview::Reads("That is 1676.4 mm. Press Enter to keep it.".into())
        );
        // 1200 mm is 47.244": the nearest 64th, 47 1/4", is 0.15 mm longer, and that is said.
        assert_eq!(
            preview("120 cm", System::Imperial),
            Preview::Reads(
                "That is 47 1/4\", to the nearest 1/64\". Press Enter to keep it.".into()
            )
        );
        // An exact inch fraction needs no such note.
        assert_eq!(
            preview("47.25 in", System::Imperial),
            Preview::Reads("That is 47 1/4\". Press Enter to keep it.".into())
        );
        assert_eq!(
            preview("1200", System::Metric),
            Preview::Reads("That is 1200 mm. Press Enter to keep it.".into())
        );
    }

    #[test]
    fn a_length_already_in_the_documents_writing_is_just_kept() {
        assert_eq!(
            preview("88.9 mm", System::Metric),
            Preview::Reads("Kept as 88.9 mm.".into())
        );
        assert_eq!(
            preview("3 1/2\"", System::Imperial),
            Preview::Reads("Kept as 3 1/2\".".into())
        );
    }

    #[test]
    fn rounding_is_said_so_the_maker_is_never_surprised() {
        assert_eq!(
            preview("10 mm", System::Imperial),
            Preview::Reads(
                "That is 25/64\", to the nearest 1/64\". Press Enter to keep it.".into()
            )
        );
    }

    #[test]
    fn what_cannot_be_read_says_what_would_be() {
        assert_eq!(
            preview("a lot", System::Metric),
            Preview::Unreadable(
                "I can't read that as a length yet (try 1200 mm, 120 cm or 1.2 m).".into()
            )
        );
        assert_eq!(
            preview("a lot", System::Imperial),
            Preview::Unreadable(
                "I can't read that as a length yet (try 48 in, 4 ft or 3 1/2\").".into()
            )
        );
        assert_eq!(
            preview("3/0", System::Imperial),
            Preview::Unreadable("A fraction can't have 0 below the line.".into())
        );
        assert_eq!(
            preview("-4", System::Metric),
            Preview::Unreadable("A length can't be below zero.".into())
        );
    }

    #[test]
    fn confirming_turns_the_typing_into_the_documents_writing() {
        let mut text = "5 ft 6 in".to_string();
        let kept = confirm(&mut text, System::Metric).unwrap();
        assert!((kept - 1676.4).abs() < 1e-9);
        assert_eq!(text, "1676.4 mm");
        // Again: nothing more happens.
        let again = confirm(&mut text, System::Metric).unwrap();
        assert!((again - kept).abs() < 1e-9);
        assert_eq!(text, "1676.4 mm");
        // The same typing in an imperial document keeps its own form.
        let mut text = "1 ft 6 in".to_string();
        confirm(&mut text, System::Imperial).unwrap();
        assert_eq!(text, "18\"");
    }

    #[test]
    fn confirming_text_that_does_not_read_leaves_it_alone() {
        let mut text = "a lot".to_string();
        assert_eq!(confirm(&mut text, System::Metric), None);
        assert_eq!(text, "a lot");
        let mut text = String::new();
        assert_eq!(confirm(&mut text, System::Imperial), None);
        assert_eq!(text, "");
    }
}
