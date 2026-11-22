//! The project interpreter, v1: the rule table of doc 05 section 5.4.
//!
//! Written by Deepak (the Crew, campaign mk-core-11, 2026-11-21) from the contract
//! crew/contracts/mk_interpret.md (ATLAS-api-harness), which Claude finished from Deepak's brief.

/// The scale classes of doc 05 section 5.5.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScaleClass {
    Room,
    Furniture,
    Body,
    Bench,
    Macro,
}

/// What the maker gave: the goal text, style words, mood-board notes, materials (as typed),
/// and attachment file names.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Input {
    pub goal: String,
    pub style_words: Vec<String>,
    pub notes: Vec<String>,
    pub materials: Vec<String>,
    pub attachments: Vec<String>,
}

/// Why an output was chosen: the rule (1-6), what matched (the rule's word or extension, as the
/// rule writes it), and the confidence in percent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Why {
    pub rule: u8,
    pub matched: String,
    pub confidence: u8,
}

impl Why {
    /// Under 60%: shown as a question, not a decision (doc 05 section 5.4).
    pub fn is_question(&self) -> bool {
        self.confidence < 60
    }
}

/// One output and its reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub value: String,
    pub why: Why,
}

/// Everything the rules concluded.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Interpretation {
    pub scale: Option<(ScaleClass, Why)>,
    pub envs: Vec<Item>,
    pub tools: Vec<Item>,
    pub templates: Vec<Item>,
    pub questions: Vec<Item>,
}

/// One row of the rule table of doc 05 section 5.4.
struct Rule {
    number: u8,
    words: &'static [&'static str],
    scale: Option<ScaleClass>,
    envs: &'static [&'static str],
    tools: &'static [&'static str],
    templates: &'static [&'static str],
    questions: &'static [&'static str],
    by_extension: bool,
}

/// The table, in table order: the first match wins (rules O1 and O2).
static RULES: [Rule; 6] = [
    Rule {
        number: 1,
        words: &[
            "dresser", "cabinet", "shelf", "desk", "bookcase", "table", "bed",
        ],
        scale: Some(ScaleClass::Room),
        envs: &["solid", "sketch", "stock"],
        tools: &[
            "box_joint",
            "drawer_wizard",
            "shelf_pins",
            "panel_from_stock",
        ],
        templates: &[],
        questions: &["space"],
        by_extension: false,
    },
    Rule {
        number: 2,
        words: &[
            "box",
            "package",
            "packaging",
            "carton",
            "sleeve",
            "mailer",
            "dieline",
        ],
        scale: None,
        envs: &["fold", "sketch", "drawings"],
        tools: &[],
        templates: &[
            "FEFCO 0201",
            "ECMA A20.20 reverse tuck end",
            "tuck-top mailer",
            "sleeve",
        ],
        questions: &["What goes in the box?"],
        by_extension: false,
    },
    Rule {
        number: 3,
        words: &[
            "backpack", "bag", "pouch", "sew", "fabric", "pattern", "quilt", "cover",
        ],
        scale: Some(ScaleClass::Body),
        envs: &["softgoods", "solid"],
        tools: &[],
        templates: &[],
        questions: &[],
        by_extension: false,
    },
    Rule {
        number: 4,
        words: &[
            "fold away",
            "murphy",
            "foldable",
            "collapsible",
            "boat",
            "tiny house",
            "convertible",
        ],
        scale: None,
        envs: &["motion", "solid", "stock", "plan"],
        tools: &["fold_states", "hinge", "gas_strut", "clearance_sweep"],
        templates: &[],
        questions: &[],
        by_extension: false,
    },
    Rule {
        number: 5,
        words: &[
            "replacement",
            "broken",
            "battery cover",
            "clip",
            "knob",
            "bracket",
        ],
        scale: Some(ScaleClass::Bench),
        envs: &["solid", "sketch"],
        tools: &["measure_from_photo", "calipers_prompt", "print3d_check"],
        templates: &[],
        questions: &[],
        by_extension: false,
    },
    Rule {
        number: 6,
        words: &["ply", "splat", "spz", "e57", "las", "usdz"],
        scale: None,
        envs: &["space"],
        tools: &[],
        templates: &[],
        questions: &[],
        by_extension: true,
    },
];

/// Rule W1's word list: ASCII-lowercase, every character that is not an ASCII letter or digit
/// becomes a separator, and empty words are dropped.
fn words_of(text: &str) -> Vec<String> {
    text.to_ascii_lowercase()
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(String::from)
        .collect()
}

/// Whether `phrase` occurs in `text` as whole words, ignoring case (rule W1).
pub fn mentions(text: &str, phrase: &str) -> bool {
    let wanted = words_of(phrase);
    if wanted.is_empty() {
        return false;
    }
    let have = words_of(text);
    if have.len() < wanted.len() {
        return false;
    }
    let last = &wanted[wanted.len() - 1];
    let last_s = format!("{last}s");
    let last_es = format!("{last}es");
    let prefix_len = wanted.len() - 1;
    have.windows(wanted.len()).any(|window| {
        let prefix_ok = window[..prefix_len]
            .iter()
            .zip(wanted[..prefix_len].iter())
            .all(|(found, word)| found == word);
        if !prefix_ok {
            return false;
        }
        let tail = &window[prefix_len];
        tail == last || tail == &last_s || tail == &last_es
    })
}

/// The text after an attachment's last `.`, lowercased; `None` when the name has no `.`.
fn extension_of(name: &str) -> Option<String> {
    let dot = name.rfind('.')?;
    Some(name[dot + 1..].to_ascii_lowercase())
}

/// Rule W3: 90 when the word is in the goal, 50 when only in style words, notes or materials.
fn word_confidence(input: &Input, word: &str) -> Option<u8> {
    if mentions(&input.goal, word) {
        return Some(90);
    }
    let others = input
        .style_words
        .iter()
        .chain(input.notes.iter())
        .chain(input.materials.iter());
    for text in others {
        if mentions(text, word) {
            return Some(50);
        }
    }
    None
}

/// Whether some attachment carries `extension` (rule R6).
fn attachment_has_extension(input: &Input, extension: &str) -> bool {
    input
        .attachments
        .iter()
        .filter_map(|name| extension_of(name))
        .any(|found| found == extension)
}

/// The why of a fired rule, or `None` when it does not fire. `matched` is the first of its words
/// in list order that fires (rule R6: the first extension in list order that some attachment has).
fn rule_why(rule: &Rule, input: &Input) -> Option<Why> {
    if rule.by_extension {
        for extension in rule.words {
            if attachment_has_extension(input, extension) {
                return Some(Why {
                    rule: rule.number,
                    matched: (*extension).to_string(),
                    confidence: 90,
                });
            }
        }
        None
    } else {
        for word in rule.words {
            if let Some(confidence) = word_confidence(input, word) {
                return Some(Why {
                    rule: rule.number,
                    matched: (*word).to_string(),
                    confidence,
                });
            }
        }
        None
    }
}

/// Rule O2: fired rules in table order, values in table order, each value once at its first
/// position, keeping the why with the highest confidence (a tie keeps the earlier rule).
fn collect(
    fired: &[(&'static Rule, Why)],
    pick: fn(&Rule) -> &'static [&'static str],
) -> Vec<Item> {
    let mut items: Vec<Item> = Vec::new();
    for (rule, why) in fired {
        for value in pick(rule) {
            let candidate = Why {
                rule: why.rule,
                matched: why.matched.clone(),
                confidence: why.confidence,
            };
            match items.iter_mut().find(|item| item.value == *value) {
                Some(existing) => {
                    if candidate.confidence > existing.why.confidence {
                        existing.why = candidate;
                    }
                }
                None => items.push(Item {
                    value: (*value).to_string(),
                    why: candidate,
                }),
            }
        }
    }
    items
}

/// Runs the rule table on the input (rules R1-R6, O1-O4).
pub fn interpret(input: &Input) -> Interpretation {
    let mut fired: Vec<(&'static Rule, Why)> = Vec::new();
    for rule in &RULES {
        if let Some(why) = rule_why(rule, input) {
            fired.push((rule, why));
        }
    }
    if fired.is_empty() {
        return Interpretation::default();
    }
    let scale = fired
        .iter()
        .find_map(|(rule, why)| rule.scale.map(|class| (class, why.clone())));
    Interpretation {
        scale,
        envs: collect(&fired, |rule| rule.envs),
        tools: collect(&fired, |rule| rule.tools),
        templates: collect(&fired, |rule| rule.templates),
        questions: collect(&fired, |rule| rule.questions),
    }
}
