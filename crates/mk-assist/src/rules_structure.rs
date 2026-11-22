//! Sancho's structure rule pack (doc 07 section 7.5): loaded with Solid.
//!
//! Written by Deepak (the Crew, campaign mk-core-21, 2026-11-22) from the contract
//! crew/contracts/mk_rules_structure.md (ATLAS-api-harness), which Claude finished from Gordon's
//! brief (the sag is the beam formula, not a lookup).

// `!(a > b)` is deliberate here: a NaN compares false both ways, and a rule must then say nothing.
#![allow(clippy::neg_cmp_op_on_partial_ord)]

use crate::findings::{Finding, Kind, Location, Severity, Source};

/// The sag rule's id.
pub const RULE_SAG: &str = "structure.sag";
/// The screw-through rule's id.
pub const RULE_SCREW_THROUGH: &str = "structure.screw_through";
/// The screw-short rule's id.
pub const RULE_SCREW_SHORT: &str = "structure.screw_short";
/// The edge-distance rule's id.
pub const RULE_EDGE_DISTANCE: &str = "structure.edge_distance";
/// The unsupported-part rule's id.
pub const RULE_UNSUPPORTED: &str = "structure.unsupported";
/// The thin-wall rule's id.
pub const RULE_THIN_WALL: &str = "structure.thin_wall";
/// The slot-severs rule's id.
pub const RULE_SLOT_SEVERS: &str = "structure.slot_severs";

/// The rule ids in the order `evaluate` runs them.
pub const RULES: [&str; 7] = [
    RULE_SAG,
    RULE_SCREW_THROUGH,
    RULE_SCREW_SHORT,
    RULE_EDGE_DISTANCE,
    RULE_UNSUPPORTED,
    RULE_THIN_WALL,
    RULE_SLOT_SEVERS,
];

/// Gravity in m/s^2: one kilogram of load weighs 9.81 N.
pub const G: f64 = 9.81;

/// One part as the app measured it. `material` is a taxonomy id ("wood.softwood.pine").
#[derive(Debug, Clone, PartialEq)]
pub struct Part {
    pub id: String,
    pub material: String,
    pub thickness_mm: f64,
}

/// A shelf: the part `part` resting on supports at both ends, loaded evenly along its span.
#[derive(Debug, Clone, PartialEq)]
pub struct Shelf {
    pub part: String,
    pub span_mm: f64,
    pub depth_mm: f64,
    pub load_kg_per_m: f64,
}

/// The two kinds of fastener the structure pack knows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FastenerKind {
    Screw,
    Dowel,
}

/// A screw or dowel. `joins` lists the parts it passes through, head side first, the part it
/// ends in last. `edge_distance_mm` is measured to the nearest edge of the part `edge_of`.
#[derive(Debug, Clone, PartialEq)]
pub struct Fastener {
    pub id: String,
    pub kind: FastenerKind,
    pub length_mm: f64,
    pub diameter_mm: f64,
    pub joins: Vec<String>,
    pub edge_of: String,
    pub edge_distance_mm: f64,
}

/// Two parts that touch (glued, resting or fixed: any contact holds).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Contact {
    pub a: String,
    pub b: String,
}

/// A slot cut into `part`, and how much material it leaves across the part.
#[derive(Debug, Clone, PartialEq)]
pub struct Slot {
    pub part: String,
    pub width_mm: f64,
    pub remaining_mm: f64,
}

/// The design as the app measured it.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Design {
    pub parts: Vec<Part>,
    pub shelves: Vec<Shelf>,
    pub fasteners: Vec<Fastener>,
    pub contacts: Vec<Contact>,
    pub slots: Vec<Slot>,
}

/// One material's numbers (the app's taxonomy supplies sourced values).
#[derive(Debug, Clone, PartialEq)]
pub struct MaterialData {
    pub material: String,
    /// Modulus of elasticity along the span, MPa (N/mm^2).
    pub stiffness_mpa: f64,
    /// Thinner than this is fragile.
    pub min_wall_mm: f64,
    /// A screw must go at least this far into the part it ends in.
    pub min_embed_mm: f64,
    /// A screw must sit at least this many of its diameters from the edge.
    pub screw_edge_diameters: f64,
    /// The same for a dowel.
    pub dowel_edge_diameters: f64,
}

/// The data the rules read.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Tables {
    pub materials: Vec<MaterialData>,
    /// The sag that shows, in mm per metre of span.
    pub sag_per_m_mm: f64,
    /// Where the numbers come from, for "Why?".
    pub source: String,
}

/// N1: a number is usable when it is finite and greater than zero.
fn usable(x: f64) -> bool {
    x.is_finite() && x > 0.0
}

/// Finds a part by exact id; the first match in input order wins.
fn find_part<'a>(design: &'a Design, id: &str) -> Option<&'a Part> {
    design.parts.iter().find(|p| p.id == id)
}

/// Finds a material by exact id; the first match in input order wins.
fn find_material<'a>(tables: &'a Tables, id: &str) -> Option<&'a MaterialData> {
    tables.materials.iter().find(|m| m.material == id)
}

/// B1: the sag of a simply supported, evenly loaded beam, in mm.
pub fn sag_mm(
    span_mm: f64,
    depth_mm: f64,
    thickness_mm: f64,
    stiffness_mpa: f64,
    load_kg_per_m: f64,
) -> Option<f64> {
    if !usable(span_mm) || !usable(depth_mm) || !usable(thickness_mm) || !usable(stiffness_mpa) {
        return None;
    }
    if !load_kg_per_m.is_finite() || load_kg_per_m < 0.0 {
        return None;
    }
    let w = load_kg_per_m * G / 1000.0;
    let i = depth_mm * thickness_mm * thickness_mm * thickness_mm / 12.0;
    let s2 = span_mm * span_mm;
    let span4 = s2 * s2;
    Some(5.0 * w * span4 / (384.0 * stiffness_mpa * i))
}

/// S1: a shelf that sags more than the table allows.
fn sag_finding(shelf: &Shelf, design: &Design, tables: &Tables) -> Option<Finding> {
    let part = find_part(design, &shelf.part)?;
    let material = find_material(tables, &part.material)?;
    let sag = sag_mm(
        shelf.span_mm,
        shelf.depth_mm,
        part.thickness_mm,
        material.stiffness_mpa,
        shelf.load_kg_per_m,
    )?;
    let allowed = tables.sag_per_m_mm * shelf.span_mm / 1000.0;
    if !usable(allowed) {
        return None;
    }
    if !(sag > allowed) {
        return None;
    }
    Some(Finding {
        kind: Kind::NotSuitable,
        severity: Severity::Warning,
        source: Source::Rule(RULE_SAG.to_string()),
        location: Location::Entity(shelf.part.clone()),
        what: format!("{} sags about {:.1} mm", shelf.part, sag),
        why: format!(
            "Over {} mm, {} mm of {} carrying {} kg per metre sags about {:.1} mm; \
             more than {:.1} mm shows.",
            shelf.span_mm, part.thickness_mm, part.material, shelf.load_kg_per_m, sag, allowed
        ),
        fix: Some("part.thickness".to_string()),
        facts: vec![
            ("part".to_string(), shelf.part.clone()),
            ("span_mm".to_string(), format!("{}", shelf.span_mm)),
            ("thickness_mm".to_string(), format!("{}", part.thickness_mm)),
            ("material".to_string(), part.material.clone()),
            (
                "load_kg_per_m".to_string(),
                format!("{}", shelf.load_kg_per_m),
            ),
            ("sag_mm".to_string(), format!("{:.1}", sag)),
            ("allowed_mm".to_string(), format!("{:.1}", allowed)),
            ("source".to_string(), tables.source.clone()),
        ],
    })
}

/// S2 or S3 for one fastener: never both, and a dowel gets neither.
fn screw_finding(fastener: &Fastener, design: &Design, tables: &Tables) -> Option<Finding> {
    if fastener.kind != FastenerKind::Screw {
        return None;
    }
    if fastener.joins.is_empty() {
        return None;
    }
    if !usable(fastener.length_mm) {
        return None;
    }
    // A joined part that is not found, or a thickness that is not usable, means no S2 and no S3.
    let mut thicknesses: Vec<f64> = Vec::with_capacity(fastener.joins.len());
    let mut stock = 0.0;
    for id in &fastener.joins {
        let part = find_part(design, id)?;
        if !usable(part.thickness_mm) {
            return None;
        }
        thicknesses.push(part.thickness_mm);
        stock += part.thickness_mm;
    }
    if fastener.length_mm > stock {
        return Some(Finding {
            kind: Kind::Forbidden,
            severity: Severity::Critical,
            source: Source::Rule(RULE_SCREW_THROUGH.to_string()),
            location: Location::Entity(fastener.id.clone()),
            what: format!("{} pokes out", fastener.id),
            why: format!(
                "{} is {} mm long and the parts it goes through are {} mm together, \
                 so its tip comes out the far side.",
                fastener.id, fastener.length_mm, stock
            ),
            fix: Some("fastener.length".to_string()),
            facts: vec![
                ("fastener".to_string(), fastener.id.clone()),
                ("length_mm".to_string(), format!("{}", fastener.length_mm)),
                ("stock_mm".to_string(), format!("{}", stock)),
            ],
        });
    }
    if fastener.joins.len() < 2 {
        return None;
    }
    let last_id = fastener.joins.last()?;
    let last = find_part(design, last_id)?;
    let material = find_material(tables, &last.material)?;
    if !usable(material.min_embed_mm) {
        return None;
    }
    let embed_sum: f64 = thicknesses.iter().take(thicknesses.len() - 1).sum();
    let embed = (fastener.length_mm - embed_sum).max(0.0);
    if !(embed < material.min_embed_mm) {
        return None;
    }
    Some(Finding {
        kind: Kind::NotSuitable,
        severity: Severity::Warning,
        source: Source::Rule(RULE_SCREW_SHORT.to_string()),
        location: Location::Entity(fastener.id.clone()),
        what: format!("{} barely bites", fastener.id),
        why: format!(
            "{} goes {} mm into {}; {} needs at least {} mm to hold.",
            fastener.id, embed, last.id, last.material, material.min_embed_mm
        ),
        fix: Some("fastener.length".to_string()),
        facts: vec![
            ("fastener".to_string(), fastener.id.clone()),
            ("embed_mm".to_string(), format!("{}", embed)),
            (
                "min_embed_mm".to_string(),
                format!("{}", material.min_embed_mm),
            ),
        ],
    })
}

/// S4: a fastener sitting too close to the edge of the part it ends in.
fn edge_finding(fastener: &Fastener, design: &Design, tables: &Tables) -> Option<Finding> {
    let part = find_part(design, &fastener.edge_of)?;
    let material = find_material(tables, &part.material)?;
    let diameters = match fastener.kind {
        FastenerKind::Screw => material.screw_edge_diameters,
        FastenerKind::Dowel => material.dowel_edge_diameters,
    };
    if !usable(diameters) {
        return None;
    }
    if !fastener.diameter_mm.is_finite() || fastener.diameter_mm <= 0.0 {
        return None;
    }
    if !fastener.edge_distance_mm.is_finite() || fastener.edge_distance_mm < 0.0 {
        return None;
    }
    let need = diameters * fastener.diameter_mm;
    if !(fastener.edge_distance_mm < need) {
        return None;
    }
    Some(Finding {
        kind: Kind::NotSuitable,
        severity: Severity::Warning,
        source: Source::Rule(RULE_EDGE_DISTANCE.to_string()),
        location: Location::Entity(fastener.id.clone()),
        what: format!("{} is too close to the edge", fastener.id),
        why: format!(
            "{} sits {} mm from the edge of {}; {} needs {} mm ({} times its {} mm diameter) \
             or it splits.",
            fastener.id,
            fastener.edge_distance_mm,
            part.id,
            part.material,
            need,
            diameters,
            fastener.diameter_mm
        ),
        fix: Some("fastener.move".to_string()),
        facts: vec![
            ("fastener".to_string(), fastener.id.clone()),
            (
                "edge_distance_mm".to_string(),
                format!("{}", fastener.edge_distance_mm),
            ),
            ("need_mm".to_string(), format!("{}", need)),
        ],
    })
}

/// S5: a part that nothing holds.
fn unsupported_finding(part: &Part, design: &Design) -> Option<Finding> {
    if design.parts.len() < 2 {
        return None;
    }
    let id = part.id.as_str();
    for contact in &design.contacts {
        let a = contact.a.as_str();
        let b = contact.b.as_str();
        if (a == id && b != id) || (b == id && a != id) {
            return None;
        }
    }
    for fastener in &design.fasteners {
        let holds = fastener.joins.iter().any(|j| j == id);
        let other = fastener.joins.iter().any(|j| j != id);
        if holds && other {
            return None;
        }
    }
    Some(Finding {
        kind: Kind::Forbidden,
        severity: Severity::Warning,
        source: Source::Rule(RULE_UNSUPPORTED.to_string()),
        location: Location::Entity(part.id.clone()),
        what: format!("{} is held by nothing", part.id),
        why: format!(
            "{} touches no other part and nothing fastens it, so it would fall.",
            part.id
        ),
        fix: Some("part.attach".to_string()),
        facts: vec![("part".to_string(), part.id.clone())],
    })
}

/// S6: a part thinner than its material allows.
fn thin_wall_finding(part: &Part, tables: &Tables) -> Option<Finding> {
    let material = find_material(tables, &part.material)?;
    if !usable(material.min_wall_mm) || !usable(part.thickness_mm) {
        return None;
    }
    if !(part.thickness_mm < material.min_wall_mm) {
        return None;
    }
    Some(Finding {
        kind: Kind::NotSuitable,
        severity: Severity::Warning,
        source: Source::Rule(RULE_THIN_WALL.to_string()),
        location: Location::Entity(part.id.clone()),
        what: format!("{} is thin", part.id),
        why: format!(
            "{} is {} mm thick; {} thinner than {} mm is fragile.",
            part.id, part.thickness_mm, part.material, material.min_wall_mm
        ),
        fix: Some("part.thickness".to_string()),
        facts: vec![
            ("part".to_string(), part.id.clone()),
            ("thickness_mm".to_string(), format!("{}", part.thickness_mm)),
            (
                "min_wall_mm".to_string(),
                format!("{}", material.min_wall_mm),
            ),
        ],
    })
}

/// S7: a slot that leaves no material across its part.
fn slot_finding(slot: &Slot) -> Option<Finding> {
    if !slot.remaining_mm.is_finite() || slot.remaining_mm > 0.0 {
        return None;
    }
    Some(Finding {
        kind: Kind::Forbidden,
        severity: Severity::Critical,
        source: Source::Rule(RULE_SLOT_SEVERS.to_string()),
        location: Location::Entity(slot.part.clone()),
        what: format!("A slot cuts {} in two", slot.part),
        why: format!(
            "The slot in {} leaves no material across it, so the part falls apart.",
            slot.part
        ),
        fix: Some("slot.shorten".to_string()),
        facts: vec![
            ("part".to_string(), slot.part.clone()),
            ("remaining_mm".to_string(), format!("{}", slot.remaining_mm)),
        ],
    })
}

/// E1: runs S1 over shelves, S2/S3 over fasteners, S4 over fasteners, S5 and S6 over parts,
/// then S7 over slots, each in input order.
pub fn evaluate(design: &Design, tables: &Tables) -> Vec<Finding> {
    let mut findings = Vec::new();

    for shelf in &design.shelves {
        if let Some(f) = sag_finding(shelf, design, tables) {
            findings.push(f);
        }
    }
    for fastener in &design.fasteners {
        if let Some(f) = screw_finding(fastener, design, tables) {
            findings.push(f);
        }
    }
    for fastener in &design.fasteners {
        if let Some(f) = edge_finding(fastener, design, tables) {
            findings.push(f);
        }
    }
    for part in &design.parts {
        if let Some(f) = unsupported_finding(part, design) {
            findings.push(f);
        }
    }
    for part in &design.parts {
        if let Some(f) = thin_wall_finding(part, tables) {
            findings.push(f);
        }
    }
    for slot in &design.slots {
        if let Some(f) = slot_finding(slot) {
            findings.push(f);
        }
    }

    findings
}
