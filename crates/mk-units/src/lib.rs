//! Lengths for makers: millimetres inside; inches, feet, fractions and lumber sizes outside.

/// A unit a length can be typed in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    /// Millimetres.
    Mm,
    /// Centimetres.
    Cm,
    /// Metres.
    M,
    /// Inches.
    In,
    /// Feet.
    Ft,
}

/// Why a typed length could not be read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnitError {
    /// Nothing but spaces.
    Empty,
    /// The text is not a length; carries the trimmed input text.
    NotANumber(String),
    /// A fraction with 0 below the line, like "3/0".
    DivideByZero,
    /// A length below zero, like "-5".
    Negative,
}

/// Millimetres in one `unit`: Mm 1, Cm 10, M 1000, In 25.4, Ft 304.8.
pub fn mm_per(unit: Unit) -> f64 {
    match unit {
        Unit::Mm => 1.0,
        Unit::Cm => 10.0,
        Unit::M => 1000.0,
        Unit::In => 25.4,
        Unit::Ft => 304.8,
    }
}

/// Reads a length a person typed and returns it in millimetres.
/// A number with no unit is in `default_unit`.
pub fn parse_length(text: &str, default_unit: Unit) -> Result<f64, UnitError> {
    // rule 1: trim, and nothing left is Empty.
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(UnitError::Empty);
    }
    // rule 2: lower-case and fold the prime marks.
    let normalized: String = trimmed
        .to_lowercase()
        .chars()
        .map(|c| match c {
            '\u{2033}' => '"',
            '\u{2032}' => '\'',
            other => other,
        })
        .collect();
    // rule 3: a leading minus is Negative.
    if normalized.starts_with('-') {
        return Err(UnitError::Negative);
    }
    let chars: Vec<char> = normalized.chars().collect();
    // rule 4 (v2): a run of more than 15 digits is not a number; checked before any
    // fraction is divided, so it wins over rule 8.
    if digit_run_too_long(&chars) {
        return Err(UnitError::NotANumber(trimmed.to_string()));
    }
    let mut pos = 0usize;
    let first = match parse_quantity(&chars, &mut pos)? {
        Some(q) => q,
        None => return Err(UnitError::NotANumber(trimmed.to_string())),
    };
    while pos < chars.len() && chars[pos] == ' ' {
        pos += 1;
    }
    if pos >= chars.len() {
        // rule 6: one quantity.
        let (value, unit) = first;
        return Ok(value * mm_per(unit.unwrap_or(default_unit)));
    }
    // rule 7 (v2): one '-' may join a feet quantity to the inches quantity.
    if first.1 == Some(Unit::Ft) && chars[pos] == '-' {
        pos += 1;
        while pos < chars.len() && chars[pos] == ' ' {
            pos += 1;
        }
    }
    let second = match parse_quantity(&chars, &mut pos)? {
        Some(q) => q,
        None => return Err(UnitError::NotANumber(trimmed.to_string())),
    };
    while pos < chars.len() && chars[pos] == ' ' {
        pos += 1;
    }
    if pos < chars.len() {
        return Err(UnitError::NotANumber(trimmed.to_string()));
    }
    // rule 7: feet then inches; the second unit is inches when absent.
    let (v1, u1) = first;
    let (v2, u2) = second;
    if u1 != Some(Unit::Ft) {
        return Err(UnitError::NotANumber(trimmed.to_string()));
    }
    let second_unit = match u2 {
        None => Unit::In,
        Some(Unit::In) => Unit::In,
        Some(_) => return Err(UnitError::NotANumber(trimmed.to_string())),
    };
    Ok(v1 * mm_per(Unit::Ft) + v2 * mm_per(second_unit))
}

/// Rule 4 (v2): true when a run of digits longer than 15 appears anywhere except after a
/// `.`, where the digits are a fractional part and have no limit.
fn digit_run_too_long(chars: &[char]) -> bool {
    let mut i = 0usize;
    while i < chars.len() {
        if !chars[i].is_ascii_digit() {
            i += 1;
            continue;
        }
        let start = i;
        while i < chars.len() && chars[i].is_ascii_digit() {
            i += 1;
        }
        let after_dot = start > 0 && chars[start - 1] == '.';
        if !after_dot && i - start > 15 {
            return true;
        }
    }
    false
}

/// Writes millimetres as inches with a reduced fraction, rounded to the nearest
/// 1/`denom` inch, followed by `"`.
pub fn format_inches(mm: f64, denom: u32) -> String {
    // rule 1: only these denominators; anything else is 16.
    let denom = match denom {
        1 | 2 | 4 | 8 | 16 | 32 | 64 => denom,
        _ => 16,
    };
    // rule 2: f64::round rounds halves away from zero.
    let n = (mm.abs() / 25.4 * denom as f64).round();
    // rule 3: zero prints as 0", never -0".
    if n == 0.0 {
        return "0\"".to_string();
    }
    let n = n as u64;
    let den = denom as u64;
    let whole = n / den;
    let rest = n % den;
    // rule 4: reduce rest/denom; gcd is at least 1, so checked_div never fails.
    let g = gcd(rest, den);
    let rest = rest.checked_div(g).unwrap_or(rest);
    let den = den.checked_div(g).unwrap_or(den);
    // rule 5: the text.
    let body = if rest == 0 {
        format!("{whole}\"")
    } else if whole == 0 {
        format!("{rest}/{den}\"")
    } else {
        format!("{whole} {rest}/{den}\"")
    };
    if mm < 0.0 {
        format!("-{body}")
    } else {
        body
    }
}

/// The real (dressed) cross-section of a US lumber size, in millimetres, thinner side
/// first: "2x4" -> Some((38.1, 88.9)). None for a size not in the table.
pub fn nominal_lumber(name: &str) -> Option<(f64, f64)> {
    let lowered = name.trim().to_lowercase();
    // rule 1: one trailing `s` is allowed.
    let t: &str = lowered.strip_suffix('s').unwrap_or(lowered.as_str());
    let chars: Vec<char> = t.chars().collect();
    let mut i = 0usize;
    let a = read_u32(&chars, &mut i)?;
    while i < chars.len() && chars[i] == ' ' {
        i += 1;
    }
    if i >= chars.len() {
        return None;
    }
    if chars[i] == 'x' || chars[i] == '\u{00d7}' {
        i += 1;
    } else if chars[i] == 'b' && i + 1 < chars.len() && chars[i + 1] == 'y' {
        i += 2;
    } else {
        return None;
    }
    while i < chars.len() && chars[i] == ' ' {
        i += 1;
    }
    let b = read_u32(&chars, &mut i)?;
    while i < chars.len() && chars[i] == ' ' {
        i += 1;
    }
    if i < chars.len() {
        return None;
    }
    // rule 2: either order.
    let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
    // rule 3: only the pairs in the table.
    let known = matches!(
        (lo, hi),
        (1, 2)
            | (1, 3)
            | (1, 4)
            | (1, 6)
            | (1, 8)
            | (1, 10)
            | (1, 12)
            | (2, 2)
            | (2, 3)
            | (2, 4)
            | (2, 6)
            | (2, 8)
            | (2, 10)
            | (2, 12)
            | (4, 4)
            | (4, 6)
            | (6, 6)
    );
    if !known {
        return None;
    }
    Some((dressed(lo)?, dressed(hi)?))
}

/// The dressed thickness of a nominal lumber number, in millimetres.
fn dressed(n: u32) -> Option<f64> {
    match n {
        1 => Some(19.05),
        2 => Some(38.1),
        3 => Some(63.5),
        4 => Some(88.9),
        6 => Some(139.7),
        8 => Some(184.15),
        10 => Some(234.95),
        12 => Some(285.75),
        _ => None,
    }
}

/// Greatest common divisor of two whole numbers; gcd(0, d) is d.
fn gcd(a: u64, b: u64) -> u64 {
    let mut x = a;
    let mut y = b;
    while y != 0 {
        let t = x % y;
        x = y;
        y = t;
    }
    x
}

/// Reads a run of ASCII digits as a whole number.
fn read_u32(chars: &[char], pos: &mut usize) -> Option<u32> {
    let mut i = *pos;
    let mut s = String::new();
    while i < chars.len() && chars[i].is_ascii_digit() {
        s.push(chars[i]);
        i += 1;
    }
    if s.is_empty() {
        return None;
    }
    let v = s.parse::<u32>().ok()?;
    *pos = i;
    Some(v)
}

/// Reads one quantity: a number and an optional unit, with spaces allowed around both.
fn parse_quantity(
    chars: &[char],
    pos: &mut usize,
) -> Result<Option<(f64, Option<Unit>)>, UnitError> {
    let mut p = *pos;
    while p < chars.len() && chars[p] == ' ' {
        p += 1;
    }
    let value = match parse_number(chars, &mut p)? {
        Some(v) => v,
        None => return Ok(None),
    };
    while p < chars.len() && chars[p] == ' ' {
        p += 1;
    }
    let unit = parse_unit(chars, &mut p);
    *pos = p;
    Ok(Some((value, unit)))
}

/// Reads a number: whole, decimal, fraction, or a mixed number (rule 4).
fn parse_number(chars: &[char], pos: &mut usize) -> Result<Option<f64>, UnitError> {
    let (value, int_opt) = match parse_simple(chars, pos)? {
        Some(v) => v,
        None => return Ok(None),
    };
    if let Some(whole) = int_opt {
        let save = *pos;
        let mut p = *pos;
        let mut separated = false;
        while p < chars.len() && chars[p] == ' ' {
            p += 1;
        }
        if p > *pos {
            separated = true;
        } else if p < chars.len() && chars[p] == '-' {
            p += 1;
            separated = true;
        }
        if separated {
            if let Some(frac) = try_fraction(chars, &mut p)? {
                *pos = p;
                return Ok(Some(whole as f64 + frac));
            }
        }
        *pos = save;
    }
    Ok(Some(value))
}

/// Reads a plain whole number, decimal or fraction; the whole value is kept for a mixed
/// number only when no decimal point was seen.
fn parse_simple(chars: &[char], pos: &mut usize) -> Result<Option<(f64, Option<u64>)>, UnitError> {
    let mut i = *pos;
    let mut int_digits = String::new();
    while i < chars.len() && chars[i].is_ascii_digit() {
        int_digits.push(chars[i]);
        i += 1;
    }
    let mut frac_digits = String::new();
    let mut has_dot = false;
    if i < chars.len() && chars[i] == '.' {
        let mut j = i + 1;
        let mut fd = String::new();
        while j < chars.len() && chars[j].is_ascii_digit() {
            fd.push(chars[j]);
            j += 1;
        }
        // rule 4: digits must follow the dot, so "12." is not a number.
        if !fd.is_empty() {
            has_dot = true;
            frac_digits = fd;
            i = j;
        }
    }
    if int_digits.is_empty() && !has_dot {
        return Ok(None);
    }
    if !has_dot && i < chars.len() && chars[i] == '/' {
        let mut j = i + 1;
        let mut dd = String::new();
        while j < chars.len() && chars[j].is_ascii_digit() {
            dd.push(chars[j]);
            j += 1;
        }
        if !dd.is_empty() {
            let num: u64 = int_digits.parse().unwrap_or(0);
            let den: u64 = dd.parse().unwrap_or(0);
            // rule 8: a zero below the line.
            if den == 0 {
                return Err(UnitError::DivideByZero);
            }
            *pos = j;
            return Ok(Some((num as f64 / den as f64, None)));
        }
    }
    let int_opt: Option<u64> = if int_digits.is_empty() {
        None
    } else {
        int_digits.parse::<u64>().ok()
    };
    let value = if has_dot {
        let head = if int_digits.is_empty() {
            "0"
        } else {
            int_digits.as_str()
        };
        let s = format!("{head}.{frac_digits}");
        s.parse::<f64>().unwrap_or(0.0)
    } else {
        int_opt.map(|v| v as f64).unwrap_or(0.0)
    };
    *pos = i;
    Ok(Some((value, if has_dot { None } else { int_opt })))
}

/// Reads `digits/digits` at `pos` as a fraction, leaving `pos` alone when there is none.
fn try_fraction(chars: &[char], pos: &mut usize) -> Result<Option<f64>, UnitError> {
    let mut i = *pos;
    let mut nd = String::new();
    while i < chars.len() && chars[i].is_ascii_digit() {
        nd.push(chars[i]);
        i += 1;
    }
    if nd.is_empty() {
        return Ok(None);
    }
    if i >= chars.len() || chars[i] != '/' {
        return Ok(None);
    }
    i += 1;
    let mut dd = String::new();
    while i < chars.len() && chars[i].is_ascii_digit() {
        dd.push(chars[i]);
        i += 1;
    }
    if dd.is_empty() {
        return Ok(None);
    }
    let num: u64 = nd.parse().unwrap_or(0);
    let den: u64 = dd.parse().unwrap_or(0);
    if den == 0 {
        return Err(UnitError::DivideByZero);
    }
    *pos = i;
    Ok(Some(num as f64 / den as f64))
}

/// Reads a unit word or a quote mark at `pos`; an unknown word leaves `pos` alone.
fn parse_unit(chars: &[char], pos: &mut usize) -> Option<Unit> {
    if *pos >= chars.len() {
        return None;
    }
    let c = chars[*pos];
    if c == '"' {
        *pos += 1;
        return Some(Unit::In);
    }
    if c == '\'' {
        *pos += 1;
        return Some(Unit::Ft);
    }
    if !c.is_ascii_alphabetic() {
        return None;
    }
    let mut i = *pos;
    let mut word = String::new();
    while i < chars.len() && chars[i].is_ascii_alphabetic() {
        word.push(chars[i]);
        i += 1;
    }
    let unit = match word.as_str() {
        "mm" | "millimeter" | "millimeters" | "millimetre" | "millimetres" => Some(Unit::Mm),
        "cm" | "centimeter" | "centimeters" | "centimetre" | "centimetres" => Some(Unit::Cm),
        "m" | "meter" | "meters" | "metre" | "metres" => Some(Unit::M),
        "in" | "inch" | "inches" => Some(Unit::In),
        "ft" | "foot" | "feet" => Some(Unit::Ft),
        _ => None,
    };
    if unit.is_some() {
        *pos = i;
    }
    unit
}
