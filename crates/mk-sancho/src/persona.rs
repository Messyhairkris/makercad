//! The persona file (doc 07 section 7.3.1): read, written, and a model checked against it.
//!
//! Written by Deepak (the Crew, campaign mk-core-15, 2026-11-22) from the contract
//! crew/contracts/mk_persona.md (ATLAS-api-harness), which Claude finished from Gordon's brief.

use std::collections::HashMap;

/// His height: metres, or "user" (the app fills in the user's own).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Height {
    Metres(f64),
    User,
}

/// One persona file, read.
#[derive(Debug, Clone, PartialEq)]
pub struct Persona {
    pub name: String,
    pub model: String,
    pub height: Height,
    pub portrait: Option<String>,
    pub lod_modest: Option<String>,
    pub lod_potato: Option<String>,
    pub head: Option<String>,
    pub point_arm: Option<[String; 3]>,
    /// (clip key, clip name in the model), in CLIP_KEYS order, each key at most once.
    pub clips: Vec<(String, String)>,
}

/// Why a persona file was refused. Line numbers count from 1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PersonaError {
    BadLine(usize),
    UnknownTable(usize, String),
    DuplicateTable(usize, String),
    UnknownKey(usize, String),
    DuplicateKey(usize, String),
    BadValue(usize, String),
    BadHeight(usize),
    MissingKey(String),
}

/// What the app measured in an imported model.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelFacts {
    pub bone_names: Vec<String>,
    pub max_influences: u32,
    pub triangles: u32,
    pub clip_names: Vec<String>,
    pub rest_height_m: f64,
}

/// One honest problem with an imported model.
#[derive(Debug, Clone, PartialEq)]
pub enum Problem {
    TooManyBones { found: usize, limit: usize },
    TooManyInfluences { found: u32, limit: u32 },
    OverBudget { triangles: u32, budget: u32 },
    MissingBone { bone: String },
    MissingClip { key: String, clip: String },
    OddHeight { rest_height_m: f64 },
}

/// The nine clip keys every persona maps, then the four optional ones.
pub const CLIP_KEYS: [&str; 13] = [
    "idle",
    "wave",
    "point",
    "think",
    "worried",
    "celebrate",
    "walk_to",
    "sit",
    "carry_board",
    "reach_high",
    "crouch_look",
    "measure",
    "shrug",
];
pub const REQUIRED_CLIPS: usize = 9;
pub const MAX_BONES: usize = 64;
pub const MAX_INFLUENCES: u32 = 4;
pub const MIN_REST_HEIGHT_M: f64 = 0.5;
pub const MAX_REST_HEIGHT_M: f64 = 2.5;

/// The Mixamo bones Sancho needs (base names): look-at chain, pointing arm, root.
pub const REQUIRED_BONES: [&str; 7] = [
    "Hips",
    "Spine",
    "Neck",
    "Head",
    "RightArm",
    "RightForeArm",
    "RightHand",
];

/// Rigify deform bones and their Mixamo base names.
pub const RIGIFY_TO_MIXAMO: [(&str, &str); 22] = [
    ("DEF-spine", "Hips"),
    ("DEF-spine.001", "Spine"),
    ("DEF-spine.002", "Spine1"),
    ("DEF-spine.003", "Spine2"),
    ("DEF-spine.004", "Neck"),
    ("DEF-spine.006", "Head"),
    ("DEF-shoulder.L", "LeftShoulder"),
    ("DEF-upper_arm.L", "LeftArm"),
    ("DEF-forearm.L", "LeftForeArm"),
    ("DEF-hand.L", "LeftHand"),
    ("DEF-shoulder.R", "RightShoulder"),
    ("DEF-upper_arm.R", "RightArm"),
    ("DEF-forearm.R", "RightForeArm"),
    ("DEF-hand.R", "RightHand"),
    ("DEF-thigh.L", "LeftUpLeg"),
    ("DEF-shin.L", "LeftLeg"),
    ("DEF-foot.L", "LeftFoot"),
    ("DEF-toe.L", "LeftToeBase"),
    ("DEF-thigh.R", "RightUpLeg"),
    ("DEF-shin.R", "RightLeg"),
    ("DEF-foot.R", "RightFoot"),
    ("DEF-toe.R", "RightToeBase"),
];

/// A value read from the right of '=' (rule R4).
enum Val {
    Str(String),
    Arr(Vec<String>),
    Tbl(Vec<(String, String)>),
    Num(f64),
}

fn dup(ln: usize, k: &str) -> PersonaError {
    PersonaError::DuplicateKey(ln, k.to_string())
}

fn skip_ws(c: &[char], i: &mut usize) {
    while *i < c.len() && c[*i].is_whitespace() {
        *i += 1;
    }
}

fn parse_str_at(c: &[char], i: &mut usize) -> Result<String, ()> {
    if *i >= c.len() || c[*i] != '"' {
        return Err(());
    }
    *i += 1;
    let mut out = String::new();
    while *i < c.len() {
        let ch = c[*i];
        if ch == '"' {
            *i += 1;
            return Ok(out);
        }
        if ch == '\\' || ch.is_control() {
            return Err(());
        }
        out.push(ch);
        *i += 1;
    }
    Err(())
}

fn parse_arr_at(c: &[char], i: &mut usize) -> Result<Vec<String>, ()> {
    if *i >= c.len() || c[*i] != '[' {
        return Err(());
    }
    *i += 1;
    let mut out = Vec::new();
    skip_ws(c, i);
    if *i < c.len() && c[*i] == ']' {
        return Err(());
    }
    loop {
        skip_ws(c, i);
        out.push(parse_str_at(c, i)?);
        skip_ws(c, i);
        if *i >= c.len() {
            return Err(());
        }
        if c[*i] == ',' {
            *i += 1;
            continue;
        }
        if c[*i] == ']' {
            *i += 1;
            return Ok(out);
        }
        return Err(());
    }
}

fn parse_tbl_at(c: &[char], i: &mut usize) -> Result<Vec<(String, String)>, ()> {
    if *i >= c.len() || c[*i] != '{' {
        return Err(());
    }
    *i += 1;
    let mut out = Vec::new();
    skip_ws(c, i);
    if *i < c.len() && c[*i] == '}' {
        *i += 1;
        return Ok(out);
    }
    loop {
        skip_ws(c, i);
        let ks = *i;
        while *i < c.len() && (c[*i].is_ascii_alphanumeric() || c[*i] == '_') {
            *i += 1;
        }
        if *i == ks {
            return Err(());
        }
        let key: String = c[ks..*i].iter().collect();
        skip_ws(c, i);
        if *i >= c.len() || c[*i] != '=' {
            return Err(());
        }
        *i += 1;
        skip_ws(c, i);
        let v = parse_str_at(c, i)?;
        out.push((key, v));
        skip_ws(c, i);
        if *i >= c.len() {
            return Err(());
        }
        if c[*i] == ',' {
            *i += 1;
            continue;
        }
        if c[*i] == '}' {
            *i += 1;
            return Ok(out);
        }
        return Err(());
    }
}

fn parse_number(s: &str) -> Option<f64> {
    let b = s.as_bytes();
    let mut i = 0usize;
    if i < b.len() && b[i] == b'-' {
        i += 1;
    }
    let start = i;
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
    }
    if i == start {
        return None;
    }
    if i < b.len() && b[i] == b'.' {
        i += 1;
        let s2 = i;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        if i == s2 {
            return None;
        }
    }
    if i != b.len() {
        return None;
    }
    s.parse::<f64>().ok()
}

fn parse_num_at(c: &[char], i: &mut usize) -> Result<f64, ()> {
    let start = *i;
    while *i < c.len() && !c[*i].is_whitespace() {
        *i += 1;
    }
    let s: String = c[start..*i].iter().collect();
    parse_number(&s).ok_or(())
}

fn parse_val_at(c: &[char], i: &mut usize) -> Result<Val, ()> {
    if *i >= c.len() {
        return Err(());
    }
    match c[*i] {
        '"' => Ok(Val::Str(parse_str_at(c, i)?)),
        '[' => Ok(Val::Arr(parse_arr_at(c, i)?)),
        '{' => Ok(Val::Tbl(parse_tbl_at(c, i)?)),
        _ => Ok(Val::Num(parse_num_at(c, i)?)),
    }
}

fn parse_val(s: &str) -> Result<Val, ()> {
    let c: Vec<char> = s.chars().collect();
    let mut i = 0usize;
    let v = parse_val_at(&c, &mut i)?;
    skip_ws(&c, &mut i);
    if i != c.len() {
        return Err(());
    }
    Ok(v)
}

fn str_val(s: &str) -> Result<String, ()> {
    match parse_val(s) {
        Ok(Val::Str(x)) => Ok(x),
        _ => Err(()),
    }
}

/// Reads a persona TOML-like text.
pub fn read_persona(text: &str) -> Result<Persona, PersonaError> {
    #[derive(Default)]
    struct Builder {
        name: Option<String>,
        model: Option<String>,
        height: Option<Height>,
        portrait: Option<String>,
        lod_seen: bool,
        lod_modest: Option<String>,
        lod_potato: Option<String>,
        head: Option<String>,
        point_arm: Option<[String; 3]>,
        clips: HashMap<String, String>,
    }

    let mut b = Builder::default();
    let mut seen_tables: HashMap<String, usize> = HashMap::new();
    let mut cur: Option<u8> = None;

    for (idx, raw) in text.split('\n').enumerate() {
        let ln = idx + 1;
        let line = raw.strip_suffix('\r').unwrap_or(raw).trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') {
            let h = match line.find('#') {
                Some(p) => &line[..p],
                None => line,
            };
            let h = h.trim();
            if h.len() < 2 || !h.starts_with('[') || !h.ends_with(']') {
                return Err(PersonaError::BadLine(ln));
            }
            let name = &h[1..h.len() - 1];
            let t = match name {
                "persona" => 0u8,
                "bones" => 1,
                "clips" => 2,
                _ => return Err(PersonaError::UnknownTable(ln, name.to_string())),
            };
            if seen_tables.contains_key(name) {
                return Err(PersonaError::DuplicateTable(ln, name.to_string()));
            }
            seen_tables.insert(name.to_string(), ln);
            cur = Some(t);
            continue;
        }
        let tbl = match cur {
            Some(t) => t,
            None => return Err(PersonaError::BadLine(ln)),
        };
        let mut in_str = false;
        let mut cut = line.len();
        for (bi, ch) in line.char_indices() {
            if ch == '"' {
                in_str = !in_str;
            } else if ch == '#' && !in_str {
                cut = bi;
                break;
            }
        }
        let line = line[..cut].trim_end();
        let eq = match line.find('=') {
            Some(p) => p,
            None => return Err(PersonaError::BadLine(ln)),
        };
        let key = line[..eq].trim();
        let val = line[eq + 1..].trim();
        if key.is_empty() || !key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return Err(PersonaError::BadLine(ln));
        }
        let bv = |k: &str| PersonaError::BadValue(ln, k.to_string());

        match tbl {
            0 => match key {
                "name" => {
                    if b.name.is_some() {
                        return Err(dup(ln, "persona.name"));
                    }
                    let s = str_val(val).map_err(|_| bv("persona.name"))?;
                    if s.trim().is_empty() {
                        return Err(bv("persona.name"));
                    }
                    b.name = Some(s);
                }
                "model" => {
                    if b.model.is_some() {
                        return Err(dup(ln, "persona.model"));
                    }
                    let s = str_val(val).map_err(|_| bv("persona.model"))?;
                    if s.trim().is_empty() {
                        return Err(bv("persona.model"));
                    }
                    b.model = Some(s);
                }
                "height_m" => {
                    if b.height.is_some() {
                        return Err(dup(ln, "persona.height_m"));
                    }
                    let h = match parse_val(val) {
                        Ok(Val::Str(s)) if s == "user" => Height::User,
                        Ok(Val::Num(n)) if n > 0.0 && n.is_finite() => Height::Metres(n),
                        _ => return Err(PersonaError::BadHeight(ln)),
                    };
                    b.height = Some(h);
                }
                "portrait" => {
                    if b.portrait.is_some() {
                        return Err(dup(ln, "persona.portrait"));
                    }
                    let s = str_val(val).map_err(|_| bv("persona.portrait"))?;
                    b.portrait = Some(s);
                }
                "lod" => {
                    if b.lod_seen {
                        return Err(dup(ln, "persona.lod"));
                    }
                    b.lod_seen = true;
                    let pairs = match parse_val(val) {
                        Ok(Val::Tbl(p)) => p,
                        _ => return Err(bv("persona.lod")),
                    };
                    for (k, v) in pairs {
                        match k.as_str() {
                            "modest" => {
                                if b.lod_modest.is_some() {
                                    return Err(dup(ln, "persona.lod.modest"));
                                }
                                b.lod_modest = Some(v);
                            }
                            "potato" => {
                                if b.lod_potato.is_some() {
                                    return Err(dup(ln, "persona.lod.potato"));
                                }
                                b.lod_potato = Some(v);
                            }
                            other => {
                                return Err(PersonaError::UnknownKey(
                                    ln,
                                    format!("persona.lod.{}", other),
                                ))
                            }
                        }
                    }
                }
                other => return Err(PersonaError::UnknownKey(ln, format!("persona.{}", other))),
            },
            1 => match key {
                "head" => {
                    if b.head.is_some() {
                        return Err(dup(ln, "bones.head"));
                    }
                    let s = str_val(val).map_err(|_| bv("bones.head"))?;
                    b.head = Some(s);
                }
                "point_arm" => {
                    if b.point_arm.is_some() {
                        return Err(dup(ln, "bones.point_arm"));
                    }
                    let arr = match parse_val(val) {
                        Ok(Val::Arr(a)) => a,
                        _ => return Err(bv("bones.point_arm")),
                    };
                    if arr.len() != 3 {
                        return Err(bv("bones.point_arm"));
                    }
                    let mut it = arr.into_iter();
                    let a0 = it.next().unwrap_or_default();
                    let a1 = it.next().unwrap_or_default();
                    let a2 = it.next().unwrap_or_default();
                    b.point_arm = Some([a0, a1, a2]);
                }
                other => return Err(PersonaError::UnknownKey(ln, format!("bones.{}", other))),
            },
            _ => {
                if !CLIP_KEYS.contains(&key) {
                    return Err(PersonaError::UnknownKey(ln, format!("clips.{}", key)));
                }
                if b.clips.contains_key(key) {
                    return Err(dup(ln, &format!("clips.{}", key)));
                }
                let s = str_val(val).map_err(|_| bv(&format!("clips.{}", key)))?;
                if s.trim().is_empty() {
                    return Err(bv(&format!("clips.{}", key)));
                }
                b.clips.insert(key.to_string(), s);
            }
        }
    }

    let name = b
        .name
        .ok_or_else(|| PersonaError::MissingKey("persona.name".to_string()))?;
    let model = b
        .model
        .ok_or_else(|| PersonaError::MissingKey("persona.model".to_string()))?;
    let height = b
        .height
        .ok_or_else(|| PersonaError::MissingKey("persona.height_m".to_string()))?;
    for &k in CLIP_KEYS.iter().take(REQUIRED_CLIPS) {
        if !b.clips.contains_key(k) {
            return Err(PersonaError::MissingKey(format!("clips.{}", k)));
        }
    }
    let mut clips = Vec::new();
    for &k in CLIP_KEYS.iter() {
        if let Some(v) = b.clips.get(k) {
            clips.push((k.to_string(), v.clone()));
        }
    }
    Ok(Persona {
        name,
        model,
        height,
        portrait: b.portrait,
        lod_modest: b.lod_modest,
        lod_potato: b.lod_potato,
        head: b.head,
        point_arm: b.point_arm,
        clips,
    })
}

/// Writes a persona back to text (rule R8).
pub fn write_persona(persona: &Persona) -> Result<String, PersonaError> {
    fn ok(s: &str) -> bool {
        !s.chars().any(|c| c == '"' || c == '\\' || c.is_control())
    }
    fn bad(k: &str) -> PersonaError {
        PersonaError::BadValue(0, k.to_string())
    }
    if !ok(&persona.name) {
        return Err(bad("persona.name"));
    }
    if !ok(&persona.model) {
        return Err(bad("persona.model"));
    }
    if let Some(p) = &persona.portrait {
        if !ok(p) {
            return Err(bad("persona.portrait"));
        }
    }
    if let Some(m) = &persona.lod_modest {
        if !ok(m) {
            return Err(bad("persona.lod.modest"));
        }
    }
    if let Some(p) = &persona.lod_potato {
        if !ok(p) {
            return Err(bad("persona.lod.potato"));
        }
    }
    if let Some(h) = &persona.head {
        if !ok(h) {
            return Err(bad("bones.head"));
        }
    }
    if let Some(a) = &persona.point_arm {
        for s in a.iter() {
            if !ok(s) {
                return Err(bad("bones.point_arm"));
            }
        }
    }
    for (k, c) in &persona.clips {
        if !ok(c) {
            return Err(bad(&format!("clips.{}", k)));
        }
    }

    let mut out = String::new();
    out.push_str("[persona]\n");
    out.push_str(&format!("name = \"{}\"\n", persona.name));
    out.push_str(&format!("model = \"{}\"\n", persona.model));
    match persona.height {
        Height::Metres(v) => out.push_str(&format!("height_m = {}\n", v)),
        Height::User => out.push_str("height_m = \"user\"\n"),
    }
    if let Some(p) = &persona.portrait {
        out.push_str(&format!("portrait = \"{}\"\n", p));
    }
    if persona.lod_modest.is_some() || persona.lod_potato.is_some() {
        out.push_str("lod = {");
        let mut first = true;
        if let Some(m) = &persona.lod_modest {
            out.push_str(&format!(" modest = \"{}\"", m));
            first = false;
        }
        if let Some(p) = &persona.lod_potato {
            if !first {
                out.push(',');
            }
            out.push_str(&format!(" potato = \"{}\"", p));
        }
        out.push_str(" }\n");
    }
    if persona.head.is_some() || persona.point_arm.is_some() {
        out.push_str("\n[bones]\n");
        if let Some(h) = &persona.head {
            out.push_str(&format!("head = \"{}\"\n", h));
        }
        if let Some(a) = &persona.point_arm {
            out.push_str(&format!(
                "point_arm = [\"{}\", \"{}\", \"{}\"]\n",
                a[0], a[1], a[2]
            ));
        }
    }
    out.push_str("\n[clips]\n");
    for (k, c) in &persona.clips {
        out.push_str(&format!("{} = \"{}\"\n", k, c));
    }
    Ok(out)
}

/// Returns the Mixamo base name of a `mixamorig` bone name (rule B1).
pub fn mixamo_base(name: &str) -> Option<&str> {
    let after = name.strip_prefix("mixamorig")?;
    let mut digits = 0usize;
    for c in after.chars() {
        if c.is_ascii_digit() {
            digits += 1;
        } else {
            break;
        }
    }
    let rest = &after[digits..];
    if rest.starts_with(':') && rest.len() > 1 {
        Some(&rest[1..])
    } else {
        None
    }
}

/// Returns the Mixamo base name for a Rigify bone, if present in the table (rule B2).
pub fn rigify_to_mixamo(name: &str) -> Option<&'static str> {
    for &(rig, mix) in RIGIFY_TO_MIXAMO.iter() {
        if rig == name {
            return Some(mix);
        }
    }
    None
}

/// Returns the base bone name, trying Mixamo then Rigify (rule B3).
pub fn base_bone(name: &str) -> Option<&str> {
    mixamo_base(name).or_else(|| rigify_to_mixamo(name))
}

/// Checks a model against a persona, returning every problem found (M1-M6).
pub fn check_model(persona: &Persona, facts: &ModelFacts, triangle_budget: u32) -> Vec<Problem> {
    let mut out = Vec::new();

    if facts.bone_names.len() > MAX_BONES {
        out.push(Problem::TooManyBones {
            found: facts.bone_names.len(),
            limit: MAX_BONES,
        });
    }
    if facts.max_influences > MAX_INFLUENCES {
        out.push(Problem::TooManyInfluences {
            found: facts.max_influences,
            limit: MAX_INFLUENCES,
        });
    }
    if facts.triangles > triangle_budget {
        out.push(Problem::OverBudget {
            triangles: facts.triangles,
            budget: triangle_budget,
        });
    }

    for (i, &bone) in REQUIRED_BONES.iter().enumerate() {
        let named: Option<&String> = match i {
            3 => persona.head.as_ref(),
            4 => persona.point_arm.as_ref().map(|a| &a[0]),
            5 => persona.point_arm.as_ref().map(|a| &a[1]),
            6 => persona.point_arm.as_ref().map(|a| &a[2]),
            _ => None,
        };
        let mut found = false;
        if let Some(n) = named {
            if facts.bone_names.iter().any(|b| b == n) {
                found = true;
            }
        }
        if !found {
            found = facts.bone_names.iter().any(|b| base_bone(b) == Some(bone));
        }
        if !found {
            out.push(Problem::MissingBone {
                bone: bone.to_string(),
            });
        }
    }

    for (k, c) in &persona.clips {
        if !facts.clip_names.iter().any(|n| n == c) {
            out.push(Problem::MissingClip {
                key: k.clone(),
                clip: c.clone(),
            });
        }
    }

    if !facts.rest_height_m.is_finite()
        || facts.rest_height_m < MIN_REST_HEIGHT_M
        || facts.rest_height_m > MAX_REST_HEIGHT_M
    {
        out.push(Problem::OddHeight {
            rest_height_m: facts.rest_height_m,
        });
    }

    out
}

impl Persona {
    /// Returns the clip name for a given key, if present.
    pub fn clip(&self, key: &str) -> Option<&str> {
        for (k, v) in &self.clips {
            if k == key {
                return Some(v);
            }
        }
        None
    }
}
