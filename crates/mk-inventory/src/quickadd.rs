//! Quick-add: one line a maker types about stock they have, read into an item.

use crate::units::{nominal_lumber, parse_length, Unit};

/// What kind of stock a line describes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Dimensional lumber: a nominal size like 2x4.
    Board,
    /// Sheet goods: plywood, MDF and the like.
    Sheet,
    /// Anything else: salvage, fabric, filament, found objects.
    Other,
}

/// One line, understood.
#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    /// How many.
    pub qty: u32,
    /// What kind of stock.
    pub kind: Kind,
    /// The material, lower-case (see the rules); "" when not said.
    pub material: String,
    /// Board: the dressed section from `nominal_lumber`, thinner side first.
    pub section_mm: Option<(f64, f64)>,
    /// Board: the length of each piece.
    pub length_mm: Option<f64>,
    /// Sheet: the thickness.
    pub thickness_mm: Option<f64>,
    /// Sheet: (shorter side, longer side) of each sheet.
    pub sheet_mm: Option<(f64, f64)>,
    /// A condition word, when one was said.
    pub condition: Option<String>,
    /// What was assumed or could not be read, for the maker to confirm, in rule order.
    pub notes: Vec<String>,
    /// The input, trimmed (not lower-cased).
    pub text: String,
}

/// Reads one quick-add line.
pub fn quick_add(text: &str) -> Item {
    let trimmed = text.trim();
    let mut item = Item {
        qty: 1,
        kind: Kind::Other,
        material: String::new(),
        section_mm: None,
        length_mm: None,
        thickness_mm: None,
        sheet_mm: None,
        condition: None,
        notes: Vec::new(),
        text: trimmed.to_string(),
    };
    // rule 1: nothing typed is nothing to read.
    if trimmed.is_empty() {
        item.qty = 0;
        return item;
    }
    let mut words = normalise(trimmed);
    // rule 3: the quantity comes from the first word only.
    item.qty = take_quantity(&mut words);
    // rule 4: the first condition word, and only the first.
    if let Some(pos) = words.iter().position(|w| is_condition(w)) {
        item.condition = Some(words.remove(pos));
    }
    // rule 5: a sheet word wins over a lumber size.
    if let Some(pos) = words.iter().position(|w| w == "sheet" || w == "sheets") {
        item.kind = Kind::Sheet;
        read_sheet(&mut words, pos, &mut item);
        return item;
    }
    // rule 6: a lumber size makes it a board.
    if let Some(pos) = words.iter().position(|w| nominal_lumber(w).is_some()) {
        item.kind = Kind::Board;
        read_board(&mut words, pos, &mut item);
        return item;
    }
    // rule 7: everything else keeps its words as the material.
    item.material = words.join(" ");
    item
}

/// Rule 2: lower-case, fold the symbols, split on whitespace.
fn normalise(text: &str) -> Vec<String> {
    let mut folded = String::new();
    for c in text.to_lowercase().chars() {
        match c {
            '\u{00d7}' => folded.push('x'),
            '\u{2033}' => folded.push('"'),
            '\u{2032}' => folded.push('\''),
            ',' => folded.push(' '),
            '\u{00bd}' => folded.push_str(" 1/2"),
            '\u{00bc}' => folded.push_str(" 1/4"),
            '\u{00be}' => folded.push_str(" 3/4"),
            other => folded.push(other),
        }
    }
    folded.split_whitespace().map(|w| w.to_string()).collect()
}

/// Rule 3: reads the quantity off the front of `words`, consuming what it used.
fn take_quantity(words: &mut Vec<String>) -> u32 {
    let first = match words.first() {
        Some(w) => w.clone(),
        None => return 1,
    };
    if is_digits(&first) {
        // rule 3: a number before a measure word is part of the material, not a count.
        if words.get(1).is_some_and(|w| is_measure_word(w)) {
            return 1;
        }
        let n: u32 = first.parse().unwrap_or(1);
        words.remove(0);
        // rule 3: "15 x pine 2x4" drops the x after a digit quantity.
        if words.first().is_some_and(|w| w == "x") {
            words.remove(0);
        }
        return n;
    }
    if first == "a" && words.get(1).is_some_and(|w| w == "dozen") {
        words.remove(0);
        words.remove(0);
        return 12;
    }
    if let Some(n) = number_word(&first) {
        words.remove(0);
        return n;
    }
    1
}

/// Rule 5: a sheet line, with the sheet word at `pos`.
fn read_sheet(words: &mut Vec<String>, pos: usize, item: &mut Item) {
    words.remove(pos);
    let mut size: Option<(f64, f64)> = None;
    // rule 5: half/quarter must be the word right before the sheet word.
    if pos > 0 {
        match words.get(pos - 1).map(|w| w.as_str()) {
            Some("half") => {
                size = Some((1219.2, 1219.2));
                item.notes.push("a half sheet is 4' x 4'".to_string());
                words.remove(pos - 1);
            }
            Some("quarter") => {
                size = Some((609.6, 1219.2));
                item.notes.push("a quarter sheet is 2' x 4'".to_string());
                words.remove(pos - 1);
            }
            _ => {}
        }
    }
    if size.is_none() {
        if let Some(i) = words.iter().position(|w| sheet_size_word(w).is_some()) {
            size = sheet_size_word(&words[i]);
            words.remove(i);
        }
    }
    if size.is_none() {
        size = Some((1219.2, 2438.4));
        item.notes.push("a full sheet is 4' x 8'".to_string());
    }
    item.sheet_mm = size;
    // rule 5: the material word, with a species in front when one was said.
    let mut material = String::new();
    if let Some(i) = words.iter().position(|w| sheet_material(w).is_some()) {
        let base = sheet_material(&words[i]).unwrap_or("");
        words.remove(i);
        material = base.to_string();
        if let Some(j) = words.iter().position(|w| is_species(w)) {
            let species = words.remove(j);
            material = format!("{species} {material}");
        }
    }
    item.material = material;
    words.retain(|w| w != "of" && w != "x");
    let rest = words.join(" ");
    if rest.is_empty() {
        item.thickness_mm = None;
    } else {
        match parse_length(&rest, Unit::In) {
            Ok(mm) => item.thickness_mm = Some(mm),
            Err(_) => {
                item.thickness_mm = None;
                item.notes
                    .push(format!("could not read the thickness: {rest}"));
            }
        }
    }
    // rule 5: last, the plywood caliper note.
    if item.material.ends_with("plywood") && item.thickness_mm.is_some() {
        item.notes
            .push("plywood is often thinner than its name: caliper it".to_string());
    }
}

/// Rule 6: a board line, with the lumber size at `pos`.
fn read_board(words: &mut Vec<String>, pos: usize, item: &mut Item) {
    let size_word = words.remove(pos);
    item.section_mm = nominal_lumber(&size_word);
    if let Some(i) = words.iter().position(|w| is_species(w)) {
        item.material = words.remove(i);
    }
    words.retain(|w| {
        !matches!(
            w.as_str(),
            "of" | "x" | "long" | "each" | "piece" | "pieces" | "pcs" | "at" | "@"
        )
    });
    let rest = words.join(" ");
    if rest.is_empty() {
        item.length_mm = None;
        return;
    }
    match parse_length(&rest, Unit::In) {
        Ok(mm) => {
            item.length_mm = Some(mm);
            // rule 6: a bare number is inches, and the maker is told so.
            let has_unit = rest
                .chars()
                .any(|c| c.is_alphabetic() || c == '\'' || c == '"');
            if !has_unit {
                item.notes
                    .push("a length without a unit is read as inches".to_string());
            }
        }
        Err(_) => {
            item.length_mm = None;
            item.notes
                .push(format!("could not read the length: {rest}"));
        }
    }
}

/// Rule 3: 1 to 4 ASCII digits.
fn is_digits(word: &str) -> bool {
    !word.is_empty() && word.len() <= 4 && word.chars().all(|c| c.is_ascii_digit())
}

/// Rule 3: the measure words that keep a number out of the quantity.
fn is_measure_word(word: &str) -> bool {
    matches!(
        word,
        "mm" | "cm"
            | "m"
            | "in"
            | "inch"
            | "inches"
            | "ft"
            | "foot"
            | "feet"
            | "yd"
            | "yard"
            | "yards"
            | "kg"
            | "g"
            | "lb"
            | "lbs"
            | "oz"
            | "l"
    )
}

/// Rule 3: a spelled-out quantity.
fn number_word(word: &str) -> Option<u32> {
    match word {
        "a" | "an" | "one" => Some(1),
        "two" => Some(2),
        "three" => Some(3),
        "four" => Some(4),
        "five" => Some(5),
        "six" => Some(6),
        "seven" => Some(7),
        "eight" => Some(8),
        "nine" => Some(9),
        "ten" => Some(10),
        "eleven" => Some(11),
        "twelve" => Some(12),
        "dozen" => Some(12),
        _ => None,
    }
}

/// Rule 4: the condition words.
fn is_condition(word: &str) -> bool {
    matches!(
        word,
        "new" | "used" | "old" | "scrap" | "salvaged" | "reclaimed" | "leftover"
    )
}

/// Rule 6: the species words, also used in front of a sheet material.
fn is_species(word: &str) -> bool {
    matches!(
        word,
        "pine"
            | "spruce"
            | "fir"
            | "spf"
            | "cedar"
            | "redwood"
            | "oak"
            | "poplar"
            | "maple"
            | "walnut"
            | "cherry"
            | "ash"
            | "birch"
    )
}

/// Rule 5: the sheet material table.
fn sheet_material(word: &str) -> Option<&'static str> {
    match word {
        "ply" | "plywood" => Some("plywood"),
        "mdf" => Some("mdf"),
        "osb" => Some("osb"),
        "hardboard" => Some("hardboard"),
        "melamine" => Some("melamine"),
        "particleboard" => Some("particleboard"),
        "acrylic" | "plexiglass" | "perspex" => Some("acrylic"),
        _ => None,
    }
}

/// Rule 5: a word `AxB` with whole numbers 1 to 12, as feet, shorter side first.
fn sheet_size_word(word: &str) -> Option<(f64, f64)> {
    let (a, b) = word.split_once('x')?;
    if !is_whole(a) || !is_whole(b) {
        return None;
    }
    let a: u32 = a.parse().ok()?;
    let b: u32 = b.parse().ok()?;
    if !(1..=12).contains(&a) || !(1..=12).contains(&b) {
        return None;
    }
    let (short, long) = if a <= b { (a, b) } else { (b, a) };
    Some((short as f64 * 304.8, long as f64 * 304.8))
}

/// Rule 5: a run of ASCII digits, nothing else.
fn is_whole(word: &str) -> bool {
    !word.is_empty() && word.chars().all(|c| c.is_ascii_digit())
}
