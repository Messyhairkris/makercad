//! Sancho's safety rule pack (doc 07 section 7.5): loaded always.
//!
//! Written by Kimi (the Crew, campaign mk-core-23, 2026-11-22) from the contract
//! crew/contracts/mk_rules_safety.md (ATLAS-api-harness), which Gordon wrote himself as a PA (the contract, the reference and the hidden tests).

use crate::findings::{Finding, Kind, Location, Severity, Source};

/// Materials that must never be laser-cut.
pub const RULE_LASER_NEVER: &str = "safety.laser_never";
/// Finishing in an unventilated space.
pub const RULE_VENTILATION: &str = "safety.ventilation";
/// Mains-voltage wiring.
pub const RULE_MAINS: &str = "safety.mains";
/// Child-furniture anti-tip checks.
pub const RULE_ANTI_TIP: &str = "safety.anti_tip";
/// Wall-mounted parts and their anchors.
pub const RULE_WALL_ANCHOR: &str = "safety.wall_anchor";

/// The rule ids in the order they run; findings follow this order.
pub const RULES: [&str; 5] = [
    RULE_LASER_NEVER,
    RULE_VENTILATION,
    RULE_MAINS,
    RULE_ANTI_TIP,
    RULE_WALL_ANCHOR,
];

/// A piece of furniture taller than this (with drawers) can tip over. (v1 starting value, to tune)
pub const TALL_FURNITURE_MM: f64 = 1200.0;

/// The maker's skill (doc 04 section 4.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Skill {
    Beginner,
    Comfortable,
    Experienced,
    Pro,
}

/// The wall a wall-mounted part is fixed to; `Unknown` means nobody looked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Wall {
    Masonry,
    Stud,
    Unknown,
}

/// One step of the plan; the safety pack reads only these three fields.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Step {
    pub id: u32,
    /// Capabilities, as the tool taxonomy writes them (doc 04 section 4.3.1).
    pub needs: Vec<String>,
    /// The id of the material the step works on, matched against `SafetyFacts::materials`.
    pub material: Option<String>,
}

/// One material's safety properties (doc 04 section 4.3.2).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Material {
    pub id: String,
    /// True when the material must never be laser-cut (PVC and vinyl are never laser-safe).
    pub laser_never: bool,
}

/// One piece of furniture (child-furniture anti-tip).
#[derive(Debug, Clone, PartialEq)]
pub struct Furniture {
    pub id: String,
    pub child: bool,
    pub height_mm: f64,
    pub drawers: bool,
    /// Some(true) anchored to a wall, Some(false) not anchored, None unknown.
    pub anchored: Option<bool>,
}

/// One wall-mounted part and its anchor.
#[derive(Debug, Clone, PartialEq)]
pub struct WallPart {
    pub id: String,
    pub load_kg: f64,
    /// The anchor's rated load in kg; None unknown.
    pub anchor_rated_kg: Option<f64>,
    pub wall: Wall,
}

/// Everything the safety pack reads.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SafetyFacts {
    pub steps: Vec<Step>,
    pub materials: Vec<Material>,
    /// Some(true) the space is ventilated, Some(false) it is not, None unknown.
    pub ventilated: Option<bool>,
    /// The design has mains-voltage wiring.
    pub wiring: bool,
    /// The maker's skill; None unknown.
    pub skill: Option<Skill>,
    pub furniture: Vec<Furniture>,
    pub wall_parts: Vec<WallPart>,
}

/// P1: return the text before the first colon, or the whole supply when there is none.
pub fn family(supply: &str) -> &str {
    match supply.find(':') {
        Some(index) => &supply[..index],
        None => supply,
    }
}

/// P2: whether any needed capability belongs to the requested family.
pub fn needs_capability(needs: &[String], capability: &str) -> bool {
    needs.iter().any(|need| family(need) == capability)
}

/// P3: the skill word used in finding facts and text.
pub fn skill_word(skill: Skill) -> &'static str {
    match skill {
        Skill::Beginner => "beginner",
        Skill::Comfortable => "comfortable",
        Skill::Experienced => "experienced",
        Skill::Pro => "pro",
    }
}

/// E1: run all five safety rules in `RULES` order without changing the supplied facts.
pub fn evaluate(facts: &SafetyFacts) -> Vec<Finding> {
    let mut findings = Vec::new();
    evaluate_laser_never(facts, &mut findings);
    evaluate_ventilation(facts, &mut findings);
    evaluate_mains(facts, &mut findings);
    evaluate_anti_tip(facts, &mut findings);
    evaluate_wall_anchor(facts, &mut findings);
    findings
}

fn rule_source(rule: &str) -> Source {
    Source::Rule(rule.to_string())
}

fn evaluate_laser_never(facts: &SafetyFacts, findings: &mut Vec<Finding>) {
    for step in &facts.steps {
        if !needs_capability(&step.needs, "cut.laser") {
            continue;
        }
        let material_id = match step.material.as_deref() {
            Some(material_id) => material_id,
            None => continue,
        };
        // L1: only the first material with this id decides; an unknown id is silence.
        let material = match facts
            .materials
            .iter()
            .find(|material| material.id.as_str() == material_id)
        {
            Some(material) => material,
            None => continue,
        };
        if !material.laser_never {
            continue;
        }
        findings.push(Finding {
            kind: Kind::Unsafe,
            severity: Severity::Critical,
            source: rule_source(RULE_LASER_NEVER),
            location: Location::Step(step.id),
            what: format!("{} must never be laser-cut", material_id),
            why: format!(
                concat!(
                    "Step {} cuts {} with a laser. ",
                    "That releases chlorine gas, which harms you and the laser."
                ),
                step.id, material_id
            ),
            fix: Some("step.change_material".to_string()),
            facts: vec![
                ("step".to_string(), step.id.to_string()),
                ("capability".to_string(), "cut.laser".to_string()),
                ("material".to_string(), material_id.to_string()),
            ],
        });
    }
}

fn evaluate_ventilation(facts: &SafetyFacts, findings: &mut Vec<Finding>) {
    if facts.ventilated != Some(false) {
        return;
    }
    for step in &facts.steps {
        // V1: the first finishing capability in needs order supplies {c}.
        let capability = step
            .needs
            .iter()
            .find(|need| matches!(family(need), "finish.paint" | "finish.oil"));
        let capability = match capability {
            Some(capability) => capability,
            None => continue,
        };
        findings.push(Finding {
            kind: Kind::Unsafe,
            severity: Severity::Warning,
            source: rule_source(RULE_VENTILATION),
            location: Location::Step(step.id),
            what: format!("Finishing with {} in an unventilated space", capability),
            why: format!(
                concat!(
                    "Step {} applies {} and the space has no ventilation, so the fumes build up. ",
                    "Work outside or ventilate the space."
                ),
                step.id, capability
            ),
            fix: Some("space.ventilate".to_string()),
            facts: vec![
                ("step".to_string(), step.id.to_string()),
                ("capability".to_string(), capability.to_string()),
            ],
        });
    }
}

fn evaluate_mains(facts: &SafetyFacts, findings: &mut Vec<Finding>) {
    if !facts.wiring {
        return;
    }
    let skill = match facts.skill {
        Some(skill) => skill,
        None => return,
    };
    let finding = if skill == Skill::Beginner {
        Finding {
            kind: Kind::Unsafe,
            severity: Severity::Critical,
            source: rule_source(RULE_MAINS),
            location: Location::Project,
            what: "Mains wiring at beginner skill".to_string(),
            why: concat!(
                "This design has mains-voltage wiring and your skill is beginner. ",
                "A mains mistake can kill; have a licensed electrician do or check this work."
            )
            .to_string(),
            fix: Some("plan.add_electrician".to_string()),
            facts: vec![
                ("wiring".to_string(), "mains".to_string()),
                ("skill".to_string(), skill_word(skill).to_string()),
            ],
        }
    } else {
        Finding {
            kind: Kind::Tip,
            severity: Severity::Info,
            source: rule_source(RULE_MAINS),
            location: Location::Project,
            what: "Mains wiring needs an electrician's check".to_string(),
            why: format!(
                concat!(
                    "This design has mains-voltage wiring and your skill is {}. ",
                    "Check your local rules and have the work inspected."
                ),
                skill_word(skill)
            ),
            fix: None,
            facts: vec![
                ("wiring".to_string(), "mains".to_string()),
                ("skill".to_string(), skill_word(skill).to_string()),
            ],
        }
    };
    findings.push(finding);
}

fn evaluate_anti_tip(facts: &SafetyFacts, findings: &mut Vec<Finding>) {
    for furniture in &facts.furniture {
        if furniture.anchored != Some(false) {
            continue;
        }
        // A1: a non-finite height is not taller than the constant.
        let tall_with_drawers = furniture.height_mm.is_finite()
            && furniture.height_mm > TALL_FURNITURE_MM
            && furniture.drawers;
        if !furniture.child && !tall_with_drawers {
            continue;
        }
        findings.push(Finding {
            kind: Kind::Unsafe,
            severity: Severity::Warning,
            source: rule_source(RULE_ANTI_TIP),
            location: Location::Entity(furniture.id.clone()),
            what: format!("{} can tip over", furniture.id),
            why: format!(
                concat!(
                    "{} is not anchored to a wall, and a child can bring it down by climbing it ",
                    "or pulling a drawer. Anchor it to a stud."
                ),
                furniture.id
            ),
            fix: Some("part.add_anchor".to_string()),
            facts: vec![
                ("part".to_string(), furniture.id.clone()),
                ("child".to_string(), furniture.child.to_string()),
                ("height_mm".to_string(), furniture.height_mm.to_string()),
                ("drawers".to_string(), furniture.drawers.to_string()),
            ],
        });
    }
}

fn evaluate_wall_anchor(facts: &SafetyFacts, findings: &mut Vec<Finding>) {
    for part in &facts.wall_parts {
        // W1 case 1: non-finite loads and ratings are not given.
        let overload = match part.anchor_rated_kg {
            Some(rated)
                if part.load_kg.is_finite() && rated.is_finite() && part.load_kg > rated =>
            {
                Some(rated)
            }
            _ => None,
        };
        if let Some(rated) = overload {
            findings.push(Finding {
                kind: Kind::Unsafe,
                severity: Severity::Critical,
                source: rule_source(RULE_WALL_ANCHOR),
                location: Location::Entity(part.id.clone()),
                what: format!("{} is heavier than its anchor", part.id),
                why: format!(
                    concat!(
                        "{} weighs {} kg and its anchor is rated {} kg. ",
                        "Use a stronger fixing or lighten the part."
                    ),
                    part.id, part.load_kg, rated
                ),
                fix: Some("part.change_anchor".to_string()),
                facts: vec![
                    ("part".to_string(), part.id.clone()),
                    ("load_kg".to_string(), part.load_kg.to_string()),
                    ("anchor_rated_kg".to_string(), rated.to_string()),
                ],
            });
            continue;
        }
        // W1 case 2 applies only when case 1 did not match.
        if part.wall == Wall::Unknown {
            findings.push(Finding {
                kind: Kind::Unsafe,
                severity: Severity::Warning,
                source: rule_source(RULE_WALL_ANCHOR),
                location: Location::Entity(part.id.clone()),
                what: format!("{} is on a wall of unknown type", part.id),
                why: format!(
                    concat!(
                        "{} is fixed to a wall whose type is unknown, so its anchor cannot be ",
                        "checked. Find the studs or the substrate first."
                    ),
                    part.id
                ),
                fix: Some("part.confirm_wall".to_string()),
                facts: vec![
                    ("part".to_string(), part.id.clone()),
                    ("wall".to_string(), "unknown".to_string()),
                ],
            });
        }
    }
}
