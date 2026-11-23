//! When a tool can be used (decision D31): every day, or only on some days of the week, with a
//! note for what days cannot say ("when my brother visits").
//!
//! A tool a person only has at the weekend (a multimeter at a friend's, a makerspace's open
//! days) is still a tool they have; what changes is when a step that needs it can happen. The
//! days are kept as a set so a plan can ask "how long until I can use it?" ([`Availability::wait_days`]);
//! the note is only ever shown to the person.
//!
//! Pure data: no files, no clock. Whoever asks says which day it is today.

/// A day of the week, Monday first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Day {
    Mon,
    Tue,
    Wed,
    Thu,
    Fri,
    Sat,
    Sun,
}

impl Day {
    /// Every day, Monday first.
    pub const ALL: [Day; 7] = [
        Day::Mon,
        Day::Tue,
        Day::Wed,
        Day::Thu,
        Day::Fri,
        Day::Sat,
        Day::Sun,
    ];

    /// "Mon".
    pub fn short(self) -> &'static str {
        ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"][self as usize]
    }

    /// "Monday".
    pub fn name(self) -> &'static str {
        [
            "Monday",
            "Tuesday",
            "Wednesday",
            "Thursday",
            "Friday",
            "Saturday",
            "Sunday",
        ][self as usize]
    }

    /// "mon": how a project file spells the day.
    pub fn key(self) -> &'static str {
        ["mon", "tue", "wed", "thu", "fri", "sat", "sun"][self as usize]
    }

    /// The day a key ("sat") or a full name ("Saturday") names, in any case.
    pub fn from_key(key: &str) -> Option<Day> {
        let word = key.trim().to_lowercase();
        Day::ALL
            .into_iter()
            .find(|d| d.key() == word || d.name().to_lowercase() == word)
    }

    /// The day `n` days later (the week wraps).
    pub fn after(self, n: u8) -> Day {
        Day::ALL[(self as usize + usize::from(n)) % 7]
    }

    fn bit(self) -> u8 {
        1 << (self as u8)
    }
}

const EVERY_DAY: u8 = 0b111_1111;
const WEEKDAYS: u8 = 0b001_1111;
const WEEKEND: u8 = 0b110_0000;

/// When a tool can be used: at least one day of the week, and a note.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Availability {
    /// Bit 0 is Monday, bit 6 is Sunday. Never zero, never above 127.
    days: u8,
    note: String,
}

impl Default for Availability {
    fn default() -> Self {
        Availability::always()
    }
}

impl Availability {
    /// Every day, nothing to say.
    pub fn always() -> Self {
        Availability {
            days: EVERY_DAY,
            note: String::new(),
        }
    }

    /// Saturday and Sunday.
    pub fn weekends() -> Self {
        Availability {
            days: WEEKEND,
            note: String::new(),
        }
    }

    /// Monday to Friday.
    pub fn weekdays() -> Self {
        Availability {
            days: WEEKDAYS,
            note: String::new(),
        }
    }

    /// Exactly these days; `None` when there are none (a tool with no day at all is a tool the
    /// person does not have: that is the wishlist).
    pub fn on(days: &[Day]) -> Option<Self> {
        Availability::from_mask(days.iter().fold(0, |mask, d| mask | d.bit()))
    }

    /// The days in a mask (bit 0 Monday ... bit 6 Sunday); `None` for no days, or for a bit
    /// beyond Sunday.
    pub fn from_mask(mask: u8) -> Option<Self> {
        (mask != 0 && mask <= EVERY_DAY).then(|| Availability {
            days: mask,
            note: String::new(),
        })
    }

    /// The days as a mask (bit 0 Monday ... bit 6 Sunday), for keeping.
    pub fn mask(&self) -> u8 {
        self.days
    }

    /// The days a file names ("sat", "Sunday"); `None` when any is not a day, or there are none.
    pub fn from_keys<'a>(keys: impl IntoIterator<Item = &'a str>, note: &str) -> Option<Self> {
        let mut days = Vec::new();
        for key in keys {
            days.push(Day::from_key(key)?);
        }
        Some(Availability::on(&days)?.with_note(note))
    }

    /// The days' keys, Monday first ("mon", "tue", ...), for a file.
    pub fn keys(&self) -> Vec<&'static str> {
        self.days().into_iter().map(Day::key).collect()
    }

    /// The same days with this note (one line: line breaks become spaces; blanks at the ends
    /// go).
    pub fn with_note(mut self, note: &str) -> Self {
        self.set_note(note);
        self
    }

    /// Changes the note (as [`with_note`](Self::with_note) reads it).
    pub fn set_note(&mut self, note: &str) {
        self.note = note.split_whitespace().collect::<Vec<_>>().join(" ");
    }

    /// The note, "" when there is none.
    pub fn note(&self) -> &str {
        &self.note
    }

    /// Whether the tool can be used on this day.
    pub fn covers(&self, day: Day) -> bool {
        self.days & day.bit() != 0
    }

    /// The days it can be used, Monday first.
    pub fn days(&self) -> Vec<Day> {
        Day::ALL.into_iter().filter(|d| self.covers(*d)).collect()
    }

    /// Adds the day when it is missing, takes it away when it is there. Taking away the last
    /// day is refused (false, nothing changed); otherwise true.
    pub fn toggle(&mut self, day: Day) -> bool {
        let next = self.days ^ day.bit();
        if next == 0 {
            return false;
        }
        self.days = next;
        true
    }

    /// Every day of the week.
    pub fn is_every_day(&self) -> bool {
        self.days == EVERY_DAY
    }

    /// Every day and no note: there is nothing to say about when.
    pub fn is_plain(&self) -> bool {
        self.is_every_day() && self.note.is_empty()
    }

    /// How many days from `today` until the tool can be used: 0 when today is one of its days.
    pub fn wait_days(&self, today: Day) -> u8 {
        // Always found within a week: there is never a tool with no day. (So the fallback is
        // unreachable, and changing it is the one mutation the tests cannot and need not catch.)
        (0..7u8).find(|n| self.covers(today.after(*n))).unwrap_or(0)
    }

    /// In a person's words: "any day", "weekends", "weekdays", "Saturdays", "Tue and Thu",
    /// "Mon, Wed and Fri", "every day except Sun"; with the note in brackets after it.
    pub fn words(&self) -> String {
        let days = self.days();
        let base = match self.days {
            EVERY_DAY => "any day".to_string(),
            WEEKEND => "weekends".to_string(),
            WEEKDAYS => "weekdays".to_string(),
            _ if days.len() == 1 => format!("{}s", days[0].name()),
            _ if days.len() == 6 => {
                let missing = Day::ALL.into_iter().find(|d| !self.covers(*d));
                format!("every day except {}", missing.map_or("", Day::short))
            }
            _ => {
                let names: Vec<&str> = days.iter().map(|d| d.short()).collect();
                match names.split_last() {
                    Some((last, rest)) => format!("{} and {last}", rest.join(", ")),
                    None => String::new(),
                }
            }
        };
        if self.note.is_empty() {
            base
        } else {
            format!("{base} ({})", self.note)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_week_is_monday_first_and_wraps() {
        assert_eq!(Day::ALL[0], Day::Mon);
        assert_eq!(Day::ALL[6], Day::Sun);
        assert_eq!(Day::Sat.after(0), Day::Sat);
        assert_eq!(Day::Sat.after(2), Day::Mon);
        assert_eq!(Day::Sun.after(7), Day::Sun);
        assert_eq!(Day::Wed.short(), "Wed");
        assert_eq!(Day::Wed.name(), "Wednesday");
        assert_eq!(Day::Wed.key(), "wed");
    }

    #[test]
    fn days_are_read_from_keys_and_names_in_any_case() {
        assert_eq!(Day::from_key("sat"), Some(Day::Sat));
        assert_eq!(Day::from_key(" Sunday "), Some(Day::Sun));
        assert_eq!(Day::from_key("FRI"), Some(Day::Fri));
        assert_eq!(Day::from_key("someday"), None);
        assert_eq!(Day::from_key(""), None);
        for d in Day::ALL {
            assert_eq!(Day::from_key(d.key()), Some(d));
            assert_eq!(Day::from_key(d.name()), Some(d));
        }
    }

    #[test]
    fn a_plain_availability_is_every_day_with_nothing_to_say() {
        let a = Availability::always();
        assert!(a.is_plain() && a.is_every_day());
        assert_eq!(a, Availability::default());
        assert_eq!(a.words(), "any day");
        assert_eq!(a.days().len(), 7);
        assert_eq!(a.mask(), 127);
        for d in Day::ALL {
            assert!(a.covers(d));
            assert_eq!(a.wait_days(d), 0);
        }
    }

    #[test]
    fn weekends_and_weekdays_cover_the_right_days() {
        let w = Availability::weekends();
        assert_eq!(w.days(), vec![Day::Sat, Day::Sun]);
        assert!(!w.covers(Day::Fri) && w.covers(Day::Sat));
        assert_eq!(w.words(), "weekends");
        assert!(!w.is_plain() && !w.is_every_day());
        let d = Availability::weekdays();
        assert_eq!(
            d.days(),
            vec![Day::Mon, Day::Tue, Day::Wed, Day::Thu, Day::Fri]
        );
        assert_eq!(d.words(), "weekdays");
        assert_eq!(Availability::on(&[Day::Sun, Day::Sat]), Some(w));
    }

    #[test]
    fn a_tool_with_no_day_is_refused() {
        assert_eq!(Availability::on(&[]), None);
        assert_eq!(Availability::from_mask(0), None);
        assert_eq!(Availability::from_mask(128), None);
        assert_eq!(Availability::from_mask(255), None);
        assert!(Availability::from_mask(127).is_some_and(|a| a.is_plain()));
        assert!(Availability::from_mask(1).is_some_and(|a| a.days() == vec![Day::Mon]));
        let mut one = Availability::on(&[Day::Sat]).unwrap();
        assert!(!one.toggle(Day::Sat), "the last day cannot be taken away");
        assert_eq!(one.days(), vec![Day::Sat]);
    }

    #[test]
    fn toggling_adds_and_takes_away_a_day() {
        let mut a = Availability::weekends();
        assert!(a.toggle(Day::Fri));
        assert_eq!(a.days(), vec![Day::Fri, Day::Sat, Day::Sun]);
        assert!(a.toggle(Day::Sun));
        assert_eq!(a.days(), vec![Day::Fri, Day::Sat]);
        assert_eq!(a.mask(), 0b010_0000 | 0b001_0000);
    }

    #[test]
    fn words_say_the_days_the_way_a_person_would() {
        let w = |days: &[Day]| Availability::on(days).unwrap().words();
        assert_eq!(w(&[Day::Sat]), "Saturdays");
        assert_eq!(w(&[Day::Tue, Day::Thu]), "Tue and Thu");
        assert_eq!(w(&[Day::Mon, Day::Wed, Day::Fri]), "Mon, Wed and Fri");
        assert_eq!(
            w(&[Day::Mon, Day::Tue, Day::Wed, Day::Thu, Day::Sat]),
            "Mon, Tue, Wed, Thu and Sat"
        );
        assert_eq!(
            w(&[Day::Mon, Day::Tue, Day::Wed, Day::Thu, Day::Fri, Day::Sat]),
            "every day except Sun"
        );
        assert_eq!(
            w(&[Day::Tue, Day::Wed, Day::Thu, Day::Fri, Day::Sat, Day::Sun]),
            "every day except Mon"
        );
    }

    #[test]
    fn the_note_goes_in_brackets_and_stays_on_one_line() {
        let a = Availability::weekends().with_note("  when my brother\n visits ");
        assert_eq!(a.note(), "when my brother visits");
        assert_eq!(a.words(), "weekends (when my brother visits)");
        assert!(!a.is_plain());
        let any = Availability::always().with_note("at the makerspace");
        assert_eq!(any.words(), "any day (at the makerspace)");
        assert!(any.is_every_day() && !any.is_plain());
        let mut b = any.clone();
        b.set_note("   ");
        assert_eq!(b, Availability::always());
    }

    #[test]
    fn the_wait_counts_days_to_the_next_one_it_can_be_used() {
        let w = Availability::weekends();
        assert_eq!(w.wait_days(Day::Mon), 5);
        assert_eq!(w.wait_days(Day::Fri), 1);
        assert_eq!(w.wait_days(Day::Sat), 0);
        assert_eq!(w.wait_days(Day::Sun), 0);
        let tue = Availability::on(&[Day::Tue]).unwrap();
        assert_eq!(tue.wait_days(Day::Wed), 6, "a whole week less a day");
        assert_eq!(tue.wait_days(Day::Tue), 0);
        assert_eq!(tue.wait_days(Day::Mon), 1);
    }

    #[test]
    fn files_say_days_by_key_and_read_back_the_same() {
        let a = Availability::weekends().with_note("at Sam's");
        assert_eq!(a.keys(), vec!["sat", "sun"]);
        let back = Availability::from_keys(a.keys(), a.note()).unwrap();
        assert_eq!(back, a);
        assert_eq!(
            Availability::from_keys(["Saturday", "sun"], ""),
            Some(Availability::weekends())
        );
        assert_eq!(Availability::from_keys(["sat", "funday"], ""), None);
        assert_eq!(Availability::from_keys([], ""), None);
    }
}
