//! The lookups over the tool and material taxonomy v1: by id, by a maker's words, and by the
//! capability a step needs.
//!
//! Written by Hoss (the Crew, campaign mk-core-12, 2026-11-22) from the contract
//! crew/contracts/mk_taxonomy.md (ATLAS-api-harness), which Claude finished from Gordon's and
//! Deepak's brief.

use crate::taxonomy_data::{
    Grade, Kit, MaterialKind, ToolKind, CAPABILITIES, KITS, MATERIALS, TOOLS,
};

/// One capability at one grade. `capability` is always the entry of CAPABILITIES.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Supply {
    pub capability: &'static str,
    pub grade: Grade,
}

/// Returns the full list of tool kinds, in table order.
pub fn tools() -> &'static [ToolKind] {
    &TOOLS
}

/// Returns the full list of material kinds, in table order.
pub fn materials() -> &'static [MaterialKind] {
    &MATERIALS
}

/// Returns the full list of kits, in table order.
pub fn kits() -> &'static [Kit] {
    &KITS
}

/// Looks up a tool kind by its exact id.
pub fn tool(id: &str) -> Option<&'static ToolKind> {
    TOOLS.iter().find(|t| t.id == id)
}

/// Looks up a material kind by its exact id.
pub fn material(id: &str) -> Option<&'static MaterialKind> {
    MATERIALS.iter().find(|m| m.id == id)
}

/// Looks up a kit by its exact id.
pub fn kit(id: &str) -> Option<&'static Kit> {
    KITS.iter().find(|k| k.id == id)
}

/// Parses a supply description string into a `Supply`.
///
/// * case-sensitive, no trimming
/// * `capability[:grade]` where grade is `"rough"` or `"fine"`
/// * missing grade defaults to `Normal`
/// * capability must be an entry of `CAPABILITIES`
pub fn parse_supply(text: &str) -> Option<Supply> {
    let (cap_str, grade_opt) = match text.split_once(':') {
        Some((c, g)) => (c, Some(g)),
        None => (text, None),
    };
    // capability must exist in the static list
    let capability = CAPABILITIES.iter().find(|&&c| c == cap_str).copied()?;
    let grade = match grade_opt {
        Some("rough") => Grade::Rough,
        Some("fine") => Grade::Fine,
        Some(_) => return None,
        None => Grade::Normal,
    };
    Some(Supply { capability, grade })
}

/// Returns all supplies a tool provides, parsed into `Supply`s.
pub fn supplies_of(tool: &ToolKind) -> Vec<Supply> {
    tool.supplies
        .iter()
        .filter_map(|s| parse_supply(s))
        .collect()
}

/// Returns all tools that can provide the given capability (any grade), in table order.
pub fn tools_for(capability: &str) -> Vec<&'static ToolKind> {
    // unknown capability -> empty list
    if !CAPABILITIES.contains(&capability) {
        return Vec::new();
    }
    TOOLS
        .iter()
        .filter(|t| {
            t.supplies.iter().any(|s| {
                let cap = s.split(':').next().unwrap_or("");
                cap == capability
            })
        })
        .collect()
}

/// Returns the highest grade among the supplied tools for a capability.
///
/// Unknown tool ids are ignored.
pub fn best_grade(tool_ids: &[&str], capability: &str) -> Option<Grade> {
    let mut best: Option<Grade> = None;
    for &id in tool_ids {
        if let Some(tool) = tool(id) {
            for &sup_str in tool.supplies.iter() {
                let (cap, grade_opt) = match sup_str.split_once(':') {
                    Some((c, g)) => (c, Some(g)),
                    None => (sup_str, None),
                };
                if cap != capability {
                    continue;
                }
                let grade = match grade_opt {
                    Some("rough") => Grade::Rough,
                    Some("fine") => Grade::Fine,
                    Some(_) => continue,
                    None => Grade::Normal,
                };
                best = Some(match best {
                    Some(b) if b > grade => b,
                    _ => grade,
                });
            }
        }
    }
    best
}

/// Splits a string into words: lower-cased ASCII letters, ASCII digits, other chars are separators.
pub fn words_of(text: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut cur = String::new();
    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() {
            cur.push(ch.to_ascii_lowercase());
        } else if !cur.is_empty() {
            words.push(std::mem::take(&mut cur));
        }
    }
    if !cur.is_empty() {
        words.push(cur);
    }
    words
}

/// Finds a tool kind whose phrase matches the given text, using the word-matching rules.
pub fn tool_by_words(text: &str) -> Option<&'static ToolKind> {
    let input = words_of(text);
    if input.is_empty() {
        return None;
    }
    let mut best_tool: Option<&ToolKind> = None;
    let mut best_len = 0usize;

    for tool in TOOLS.iter() {
        for &phrase in tool.words.iter() {
            let phrase_words: Vec<&str> = phrase.split(' ').collect();
            if phrase_words.is_empty() {
                continue;
            }
            if phrase_words.len() < best_len {
                continue;
            }
            if matches_phrase(&input, &phrase_words) {
                if phrase_words.len() > best_len {
                    best_len = phrase_words.len();
                    best_tool = Some(tool);
                } else if best_tool.is_none() {
                    best_tool = Some(tool);
                }
            }
        }
    }
    best_tool
}

/// Finds a material kind whose phrase matches the given text, using the word-matching rules.
pub fn material_by_words(text: &str) -> Option<&'static MaterialKind> {
    let input = words_of(text);
    if input.is_empty() {
        return None;
    }
    let mut best_mat: Option<&MaterialKind> = None;
    let mut best_len = 0usize;

    for mat in MATERIALS.iter() {
        for &phrase in mat.words.iter() {
            let phrase_words: Vec<&str> = phrase.split(' ').collect();
            if phrase_words.is_empty() {
                continue;
            }
            if phrase_words.len() < best_len {
                continue;
            }
            if matches_phrase(&input, &phrase_words) {
                if phrase_words.len() > best_len {
                    best_len = phrase_words.len();
                    best_mat = Some(mat);
                } else if best_mat.is_none() {
                    best_mat = Some(mat);
                }
            }
        }
    }
    best_mat
}

/// Returns true when `id` is exactly `family` or starts with `family.` (family must be non-empty).
pub fn is_a(id: &str, family: &str) -> bool {
    if family.is_empty() {
        return false;
    }
    id == family || id.starts_with(&format!("{}.", family))
}

/// Helper: does any position in `input` match the whole `phrase_words` according to rule W2?
fn matches_phrase(input: &[String], phrase_words: &[&str]) -> bool {
    if phrase_words.is_empty() || input.len() < phrase_words.len() {
        return false;
    }
    for start in 0..=input.len() - phrase_words.len() {
        let mut ok = true;
        for (j, &pw) in phrase_words.iter().enumerate() {
            let iw = &input[start + j];
            if iw != pw && iw != &format!("{}s", pw) && iw != &format!("{}es", pw) {
                ok = false;
                break;
            }
        }
        if ok {
            return true;
        }
    }
    false
}
