//! The hardware profile: what the capability probe measured, turned into budgets.
//!
//! Written by Deepak (the Crew, campaign mk-core-6, 2026-11-21) from the contract
//! crew/contracts/mk_profile.md (ATLAS-api-harness), which Claude finished from Gordon's brief.

/// The machine class (doc 12 section 12.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    Potato,
    Modest,
    Capable,
}

/// The graphics backend. `Cpu` is the software fallback, never tried as a probe attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    Dx12,
    Gl,
    Cpu,
}

/// How hard level-of-detail reduces far geometry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lod {
    Low,
    Medium,
    High,
}

/// How Sancho is drawn (doc 07 section 7.3.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SanchoTier {
    Full,
    Lod8k,
    BakedVat,
    Portrait,
}

/// The physics the Motion environment offers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Physics {
    Kinematic,
    Dynamic,
}

/// The hardware profile: exactly the keys of doc 12 section 12.2.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Profile {
    pub class: Class,
    pub backend: Backend,
    pub tri_budget: u32,
    pub draw_call_budget: u32,
    pub lod_aggressiveness: Lod,
    pub sancho: SanchoTier,
    pub splat_render: bool,
    pub physics: Physics,
    pub mem_budget_mb: u32,
    pub background_jobs: u32,
    pub path_tracer: bool,
}

/// One backend tried in a child process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Attempt {
    pub backend: Backend,
    pub initialised: bool,
}

/// What the probe measured.
#[derive(Debug, Clone, PartialEq)]
pub struct Facts {
    /// Every backend attempt, in the order they ran.
    pub attempts: Vec<Attempt>,
    /// The 95th-percentile frame time of the 2 s, 200k-triangle viewport benchmark.
    pub frame_p95_ms: f64,
    /// What the baked (VAT) Sancho adds to a frame.
    pub sancho_vat_ms: f64,
    /// Logical cores.
    pub cores: u32,
    /// Total RAM as the OS reports it.
    pub ram_total_mb: u64,
    /// Dedicated video memory, when the adapter reports it.
    pub vram_mb: Option<u64>,
}

/// Something `parse` met that it could not use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Problem {
    /// A line that is neither blank, a comment, a table header nor `key = value` (1-based).
    BadLine(usize),
    UnknownKey(String),
    BadValue(String),
    Duplicate(String),
    MissingKey(String),
}

/// What `parse` read: always a whole profile, plus what was wrong with the text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parsed {
    pub profile: Profile,
    pub problems: Vec<Problem>,
}

/// The backends the probe tries on Windows, first to last. Never Vulkan.
pub const BACKEND_ORDER: [Backend; 2] = [Backend::Dx12, Backend::Gl];

/// The profile's keys in doc 12 section 12.2 order.
pub const KEYS: [&str; 11] = [
    "class",
    "backend",
    "tri_budget",
    "draw_call_budget",
    "lod_aggressiveness",
    "sancho",
    "splat_render",
    "physics",
    "mem_budget_mb",
    "background_jobs",
    "path_tracer",
];

/// The class's row of the defaults table, with backend `Dx12`.
pub fn defaults(class: Class) -> Profile {
    match class {
        Class::Potato => Profile {
            class: Class::Potato,
            backend: Backend::Dx12,
            tri_budget: 200_000,
            draw_call_budget: 60,
            lod_aggressiveness: Lod::High,
            sancho: SanchoTier::BakedVat,
            splat_render: false,
            physics: Physics::Kinematic,
            mem_budget_mb: 2_500,
            background_jobs: 2,
            path_tracer: false,
        },
        Class::Modest => Profile {
            class: Class::Modest,
            backend: Backend::Dx12,
            tri_budget: 1_000_000,
            draw_call_budget: 200,
            lod_aggressiveness: Lod::Medium,
            sancho: SanchoTier::Lod8k,
            splat_render: true,
            physics: Physics::Dynamic,
            mem_budget_mb: 6_000,
            background_jobs: 3,
            path_tracer: false,
        },
        Class::Capable => Profile {
            class: Class::Capable,
            backend: Backend::Dx12,
            tri_budget: 4_000_000,
            draw_call_budget: 800,
            lod_aggressiveness: Lod::Low,
            sancho: SanchoTier::Full,
            splat_render: true,
            physics: Physics::Dynamic,
            mem_budget_mb: 16_000,
            background_jobs: 4,
            path_tracer: true,
        },
    }
}

/// `defaults(Class::Potato)`: doc 12 section 12.2's example, exactly.
pub fn potato() -> Profile {
    defaults(Class::Potato)
}

/// The backend to use (rule B1).
pub fn choose_backend(attempts: &[Attempt]) -> Backend {
    for backend in BACKEND_ORDER {
        // B1: only the first attempt of each backend counts.
        let first = attempts.iter().find(|a| a.backend == backend);
        if matches!(first, Some(a) if a.initialised) {
            return backend;
        }
    }
    Backend::Cpu
}

/// A frame time is within `l` only when it is a real number from 0.0 to `l` inclusive.
fn within(x: f64, l: f64) -> bool {
    (0.0..=l).contains(&x)
}

/// The machine class for these facts on this backend (rules K1-K4).
pub fn classify(facts: &Facts, backend: Backend) -> Class {
    if backend == Backend::Cpu {
        return Class::Potato;
    }
    let vram_ok = matches!(facts.vram_mb, Some(v) if v >= 6_144);
    if facts.cores >= 8
        && facts.ram_total_mb >= 28_672
        && vram_ok
        && within(facts.frame_p95_ms, 10.0)
    {
        return Class::Capable;
    }
    if facts.cores >= 4 && facts.ram_total_mb >= 12_288 && within(facts.frame_p95_ms, 20.0) {
        return Class::Modest;
    }
    Class::Potato
}

/// The profile for measured facts; `None` (failed or missing probe) gives `potato()` (rules F1-F4).
pub fn from_facts(facts: Option<&Facts>) -> Profile {
    let facts = match facts {
        Some(f) => f,
        None => return potato(),
    };
    let backend = choose_backend(&facts.attempts);
    let class = classify(facts, backend);
    let mut profile = defaults(class);
    profile.backend = backend;
    // F4: only a potato cares how much the baked Sancho costs.
    if class == Class::Potato && !within(facts.sancho_vat_ms, 2.0) {
        profile.sancho = SanchoTier::Portrait;
    }
    profile
}

/// `n` in decimal with `_` between groups of three digits from the right (rule T2).
pub fn group_digits(n: u32) -> String {
    let digits = n.to_string();
    let len = digits.len();
    let mut out = String::with_capacity(len + len / 3);
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (len - i).is_multiple_of(3) {
            out.push('_');
        }
        out.push(c);
    }
    out
}

fn class_word(c: Class) -> &'static str {
    match c {
        Class::Potato => "potato",
        Class::Modest => "modest",
        Class::Capable => "capable",
    }
}

fn backend_word(b: Backend) -> &'static str {
    match b {
        Backend::Dx12 => "dx12",
        Backend::Gl => "gl",
        Backend::Cpu => "cpu",
    }
}

fn lod_word(l: Lod) -> &'static str {
    match l {
        Lod::Low => "low",
        Lod::Medium => "medium",
        Lod::High => "high",
    }
}

fn sancho_word(s: SanchoTier) -> &'static str {
    match s {
        SanchoTier::Full => "full",
        SanchoTier::Lod8k => "lod8k",
        SanchoTier::BakedVat => "baked_vat",
        SanchoTier::Portrait => "portrait",
    }
}

fn physics_word(p: Physics) -> &'static str {
    match p {
        Physics::Kinematic => "kinematic",
        Physics::Dynamic => "dynamic",
    }
}

fn quoted(word: &str) -> String {
    let mut s = String::with_capacity(word.len() + 2);
    s.push('"');
    s.push_str(word);
    s.push('"');
    s
}

fn bool_word(b: bool) -> String {
    if b {
        "true".to_string()
    } else {
        "false".to_string()
    }
}

/// The TOML text of one key's value (rules T2-T3).
fn value_text(profile: &Profile, key: &str) -> String {
    match key {
        "class" => quoted(class_word(profile.class)),
        "backend" => quoted(backend_word(profile.backend)),
        "tri_budget" => group_digits(profile.tri_budget),
        "draw_call_budget" => group_digits(profile.draw_call_budget),
        "lod_aggressiveness" => quoted(lod_word(profile.lod_aggressiveness)),
        "sancho" => quoted(sancho_word(profile.sancho)),
        "splat_render" => bool_word(profile.splat_render),
        "physics" => quoted(physics_word(profile.physics)),
        "mem_budget_mb" => group_digits(profile.mem_budget_mb),
        "background_jobs" => group_digits(profile.background_jobs),
        "path_tracer" => bool_word(profile.path_tracer),
        _ => String::new(),
    }
}

/// The profile as TOML text (rules T1-T3).
pub fn to_toml(profile: &Profile) -> String {
    let mut out = String::from("[profile]\n");
    for key in KEYS {
        out.push_str(key);
        out.push_str(" = ");
        out.push_str(&value_text(profile, key));
        out.push('\n');
    }
    out
}

/// The text before the first `#` that is not inside a pair of double quotes (rule P4).
fn cut_comment(s: &str) -> &str {
    let mut in_quotes = false;
    for (i, c) in s.char_indices() {
        match c {
            '"' => in_quotes = !in_quotes,
            '#' if !in_quotes => return &s[..i],
            _ => {}
        }
    }
    s
}

/// The word inside `"word"`, or `None` when the value is not exactly that shape (rule P8).
fn quoted_word(value: &str) -> Option<&str> {
    if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
        Some(&value[1..value.len() - 1])
    } else {
        None
    }
}

/// An integer value: digits, single `_` only between digits, no leading zero, fits `u32` (P8).
fn parse_int(value: &str) -> Option<u32> {
    let bytes = value.as_bytes();
    if bytes.is_empty() {
        return None;
    }
    if !bytes.iter().all(|b| b.is_ascii_digit() || *b == b'_') {
        return None;
    }
    if bytes[0] == b'_' || bytes[bytes.len() - 1] == b'_' {
        return None;
    }
    if bytes
        .windows(3)
        .any(|w| w[1] == b'_' && (w[0] == b'_' || w[2] == b'_'))
    {
        return None;
    }
    if bytes[0] == b'0' && bytes.len() > 1 {
        return None;
    }
    let mut acc: u64 = 0;
    for b in bytes {
        if *b == b'_' {
            continue;
        }
        acc = acc * 10 + u64::from(*b - b'0');
        if acc > u64::from(u32::MAX) {
            return None;
        }
    }
    Some(acc as u32)
}

/// Store one good value; `false` means the value was bad and nothing changed (rule P8).
fn apply_value(profile: &mut Profile, key: &str, value: &str) -> bool {
    match key {
        "class" => match quoted_word(value) {
            Some("potato") => {
                profile.class = Class::Potato;
                true
            }
            Some("modest") => {
                profile.class = Class::Modest;
                true
            }
            Some("capable") => {
                profile.class = Class::Capable;
                true
            }
            _ => false,
        },
        "backend" => match quoted_word(value) {
            Some("dx12") => {
                profile.backend = Backend::Dx12;
                true
            }
            Some("gl") => {
                profile.backend = Backend::Gl;
                true
            }
            Some("cpu") => {
                profile.backend = Backend::Cpu;
                true
            }
            _ => false,
        },
        "tri_budget" => match parse_int(value) {
            Some(v) => {
                profile.tri_budget = v;
                true
            }
            None => false,
        },
        "draw_call_budget" => match parse_int(value) {
            Some(v) => {
                profile.draw_call_budget = v;
                true
            }
            None => false,
        },
        "lod_aggressiveness" => match quoted_word(value) {
            Some("low") => {
                profile.lod_aggressiveness = Lod::Low;
                true
            }
            Some("medium") => {
                profile.lod_aggressiveness = Lod::Medium;
                true
            }
            Some("high") => {
                profile.lod_aggressiveness = Lod::High;
                true
            }
            _ => false,
        },
        "sancho" => match quoted_word(value) {
            Some("full") => {
                profile.sancho = SanchoTier::Full;
                true
            }
            Some("lod8k") => {
                profile.sancho = SanchoTier::Lod8k;
                true
            }
            Some("baked_vat") => {
                profile.sancho = SanchoTier::BakedVat;
                true
            }
            Some("portrait") => {
                profile.sancho = SanchoTier::Portrait;
                true
            }
            _ => false,
        },
        "splat_render" => match value {
            "true" => {
                profile.splat_render = true;
                true
            }
            "false" => {
                profile.splat_render = false;
                true
            }
            _ => false,
        },
        "physics" => match quoted_word(value) {
            Some("kinematic") => {
                profile.physics = Physics::Kinematic;
                true
            }
            Some("dynamic") => {
                profile.physics = Physics::Dynamic;
                true
            }
            _ => false,
        },
        "mem_budget_mb" => match parse_int(value) {
            Some(v) => {
                profile.mem_budget_mb = v;
                true
            }
            None => false,
        },
        "background_jobs" => match parse_int(value) {
            Some(v) => {
                profile.background_jobs = v;
                true
            }
            None => false,
        },
        "path_tracer" => match value {
            "true" => {
                profile.path_tracer = true;
                true
            }
            "false" => {
                profile.path_tracer = false;
                true
            }
            _ => false,
        },
        _ => false,
    }
}

/// Reads profile TOML text. Never fails (rules P1-P9).
pub fn parse(text: &str) -> Parsed {
    let mut profile = potato();
    let mut problems: Vec<Problem> = Vec::new();
    let mut named = [false; KEYS.len()];
    let mut table: &str = "";

    for (idx, raw) in text.split('\n').enumerate() {
        let n = idx + 1;
        let line = raw.strip_suffix('\r').unwrap_or(raw);
        let line = line.trim();
        // P2
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        // P3
        if line.starts_with('[') {
            if line.ends_with(']') {
                table = line[1..line.len() - 1].trim();
            } else {
                problems.push(Problem::BadLine(n));
            }
            continue;
        }
        // P4: every other line needs a non-empty key before the first `=`.
        let eq = match line.find('=') {
            Some(i) => i,
            None => {
                problems.push(Problem::BadLine(n));
                continue;
            }
        };
        let key = line[..eq].trim();
        if key.is_empty() {
            problems.push(Problem::BadLine(n));
            continue;
        }
        let value = cut_comment(&line[eq + 1..]).trim();
        // P5: key lines count only inside [profile].
        if table != "profile" {
            continue;
        }
        // P6
        let pos = match KEYS.iter().position(|k| *k == key) {
            Some(p) => p,
            None => {
                problems.push(Problem::UnknownKey(key.to_string()));
                continue;
            }
        };
        // P7: the earlier line stands, good value or bad.
        if named[pos] {
            problems.push(Problem::Duplicate(key.to_string()));
            continue;
        }
        named[pos] = true;
        // P8
        if !apply_value(&mut profile, key, value) {
            problems.push(Problem::BadValue(key.to_string()));
        }
    }
    // P9
    for (i, key) in KEYS.iter().enumerate() {
        if !named[i] {
            problems.push(Problem::MissingKey((*key).to_string()));
        }
    }
    Parsed { profile, problems }
}
