//! The golden dresser (doc 14 P2): one fixture, a sound build and five seeded faults.
//!
//! Written by Deepak (the Crew, campaign mk-core-33, 2026-11-22) from the contract
//! crew/contracts/mk_golden.md (ATLAS-api-harness), which Gordon wrote himself as a PA (the contract, the reference and the hidden tests).
use crate::findings::{Finding, Location};

pub const WIDTH_MM: f64 = 900.0; // carcase outside width (v1 starting value, to tune)
pub const HEIGHT_MM: f64 = 1100.0; // carcase outside height (v1 starting value, to tune)
pub const DEPTH_MM: f64 = 450.0; // carcase depth (v1 starting value, to tune)
pub const CARCASE_PANEL_MM: f64 = 18.0; // carcase panel thickness (v1 starting value, to tune)
pub const SHELF_THICKNESS_MM: f64 = 12.0; // carcase top thickness (v1 starting value, to tune)
pub const BACK_PANEL_MM: f64 = 6.0; // back panel thickness (v1 starting value, to tune)
pub const DRAWER_BOX_MM: f64 = 12.0; // drawer box thickness (v1 starting value, to tune)
pub const DRAWER_FRONT_HEIGHT_MM: f64 = 250.0; // drawer front height (v1 starting value, to tune)
pub const SHELF_SPAN_MM: f64 = 864.0; // shelf span between the sides (v1 starting value, to tune)
pub const SOUND_LOAD_KG_PER_M: f64 = 8.0; // sound shelf load (v1 starting value, to tune)
pub const SAG_PER_M_MM: f64 = 2.0; // sag that shows, mm per metre of span (v1 starting value, to tune)
pub const PINE_STIFFNESS_MPA: f64 = 10000.0; // pine modulus of elasticity (v1 starting value, to tune)
pub const PLYWOOD_STIFFNESS_MPA: f64 = 6000.0; // plywood modulus of elasticity (v1 starting value, to tune)
pub const MIN_WALL_MM: f64 = 3.0; // thinnest stock either material allows (v1 starting value, to tune)
pub const MIN_EMBED_MM: f64 = 12.0; // pine minimum screw embedment (v1 starting value, to tune)
pub const PLYWOOD_MIN_EMBED_MM: f64 = 10.0; // plywood minimum screw embedment (v1 starting value, to tune)
pub const SCREW_EDGE_DIAMETERS: f64 = 4.0; // screw edge distance in diameters (v1 starting value, to tune)
pub const DOWEL_EDGE_DIAMETERS: f64 = 3.0; // dowel edge distance in diameters (v1 starting value, to tune)
pub const SCREW_LENGTH_MM: f64 = 32.0; // sound carcase screw length (v1 starting value, to tune)
pub const SCREW_DIAMETER_MM: f64 = 4.0; // sound screw diameter (v1 starting value, to tune)
pub const SCREW_EDGE_MM: f64 = 25.0; // sound carcase screw edge distance (v1 starting value, to tune)
pub const BACK_SCREW_LENGTH_MM: f64 = 20.0; // back-panel screw length (v1 starting value, to tune)
pub const BACK_SCREW_EDGE_MM: f64 = 20.0; // back-panel screw edge distance (v1 starting value, to tune)
pub const DOWEL_LENGTH_MM: f64 = 30.0; // dowel length (v1 starting value, to tune)
pub const DOWEL_DIAMETER_MM: f64 = 8.0; // dowel diameter (v1 starting value, to tune)
pub const DOWEL_EDGE_MM: f64 = 30.0; // dowel edge distance (v1 starting value, to tune)
pub const TOOL_ACCURACY_MM: f64 = 0.5; // table saw and router accuracy (v1 starting value, to tune)
pub const STEP_TOLERANCE_MM: f64 = 1.0; // plan step tolerance (v1 starting value, to tune)
pub const PLAN_START_MINUTE: u16 = 540; // plan slot start, 09:00 (v1 starting value, to tune)
pub const PLAN_END_MINUTE: u16 = 600; // plan slot end, 10:00 (v1 starting value, to tune)
pub const QUIET_START_MINUTE: u16 = 1320; // quiet hours start, 22:00 (v1 starting value, to tune)
pub const QUIET_END_MINUTE: u16 = 1440; // quiet hours end, 24:00 (v1 starting value, to tune)
pub const FAULT_CHEST_HEIGHT_MM: f64 = 1300.0; // faulted chest height (v1 starting value, to tune)
pub const WALL_LOAD_KG: f64 = 12.0; // sound wall part load (v1 starting value, to tune)
pub const WALL_RATING_KG: f64 = 20.0; // sound wall anchor rating (v1 starting value, to tune)
pub const SLOT_WIDTH_MM: f64 = 12.0; // dado width (v1 starting value, to tune)
pub const SLOT_REMAINING_MM: f64 = 6.0; // dado remaining material (v1 starting value, to tune)
pub const FAULT_SAG_LOAD_KG_PER_M: f64 = 20.0; // faulted shelf load (v1 starting value, to tune)
pub const FAULT_SCREW_LENGTH_MM: f64 = 40.0; // faulted screw length (v1 starting value, to tune)
pub const FAULT_SCREW_EDGE_MM: f64 = 8.0; // faulted screw edge distance (v1 starting value, to tune)
pub const NEAR_LOAD_KG_PER_M: f64 = 14.0; // near-miss shelf load (v1 starting value, to tune)
pub const NEAR_SCREW_LENGTH_MM: f64 = 35.0; // near-miss screw length (v1 starting value, to tune)
pub const NEAR_SCREW_EDGE_MM: f64 = 16.5; // near-miss screw edge distance (v1 starting value, to tune)
pub const NEAR_STEP_TOLERANCE_MM: f64 = 0.6; // near-miss step tolerance (v1 starting value, to tune)
pub const NEAR_WALL_LOAD_KG: f64 = 20.0; // near-miss wall part load (v1 starting value, to tune)
pub const NEAR_SLOT_REMAINING_MM: f64 = 0.5; // near-miss dado remaining material (v1 starting value, to tune)

/// The facts of one chest: one pack's view per field.
#[derive(Debug, Clone)]
pub struct Golden {
    pub core: crate::rules_core::ProjectFacts,
    pub tools: crate::rules_tools::ToolFacts,
    pub design: crate::rules_structure::Design,
    pub tables: crate::rules_structure::Tables,
    pub safety: crate::rules_safety::SafetyFacts,
}

/// One seeded fault: a rule id and the plain text of the finding's location.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Seed {
    pub rule: String,
    pub entity: String,
}

/// What `check` found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub matched: Vec<Seed>,
    pub missing: Vec<Seed>,
    pub false_flags: Vec<(String, String)>,
}

impl Report {
    /// True exactly when nothing is missing and nothing was flagged that was not seeded.
    pub fn is_perfect(&self) -> bool {
        self.missing.is_empty() && self.false_flags.is_empty()
    }
}

/// A1: the plain text of a location; a seed's `entity` is exactly this.
pub fn location_text(location: &Location) -> String {
    match location {
        Location::Project => "project".to_string(),
        Location::Entity(id) => id.clone(),
        Location::Step(n) => format!("{}", n),
        Location::Field(f) => f.clone(),
    }
}

/// The rule id of a finding: the string inside `Source::Rule`, or "mind".
fn rule_id(finding: &Finding) -> String {
    match &finding.source {
        crate::findings::Source::Rule(id) => id.clone(),
        crate::findings::Source::Mind => "mind".to_string(),
    }
}

fn s(text: &str) -> String {
    text.to_string()
}

fn core_part(name: &str, size_mm: [f64; 3]) -> crate::rules_core::Part {
    crate::rules_core::Part {
        name: s(name),
        size_mm,
    }
}

fn tool_slot(day: u8, start_minute: u16, end_minute: u16) -> crate::rules_tools::Slot {
    crate::rules_tools::Slot {
        day,
        start_minute,
        end_minute,
    }
}

fn tool_step(
    id: u32,
    needs: &[&str],
    tolerance_mm: Option<f64>,
    planned: Option<crate::rules_tools::Slot>,
) -> crate::rules_tools::Step {
    crate::rules_tools::Step {
        id,
        needs: needs.iter().map(|n| s(n)).collect(),
        tolerance_mm,
        planned,
    }
}

fn tool(
    id: &str,
    supplies: &[&str],
    access: crate::rules_tools::Access,
    accuracy_mm: Option<f64>,
    noise: crate::rules_tools::Noise,
) -> crate::rules_tools::Tool {
    crate::rules_tools::Tool {
        id: s(id),
        supplies: supplies.iter().map(|n| s(n)).collect(),
        access,
        accuracy_mm,
        noise,
    }
}

fn design_part(id: &str, material: &str, thickness_mm: f64) -> crate::rules_structure::Part {
    crate::rules_structure::Part {
        id: s(id),
        material: s(material),
        thickness_mm,
    }
}

fn fastener(
    id: &str,
    kind: crate::rules_structure::FastenerKind,
    length_mm: f64,
    diameter_mm: f64,
    joins: &[&str],
    edge_of: &str,
    edge_distance_mm: f64,
) -> crate::rules_structure::Fastener {
    crate::rules_structure::Fastener {
        id: s(id),
        kind,
        length_mm,
        diameter_mm,
        joins: joins.iter().map(|n| s(n)).collect(),
        edge_of: s(edge_of),
        edge_distance_mm,
    }
}

fn contact(a: &str, b: &str) -> crate::rules_structure::Contact {
    crate::rules_structure::Contact { a: s(a), b: s(b) }
}

fn material_data(
    material: &str,
    stiffness_mpa: f64,
    min_embed_mm: f64,
) -> crate::rules_structure::MaterialData {
    crate::rules_structure::MaterialData {
        material: s(material),
        stiffness_mpa,
        min_wall_mm: MIN_WALL_MM,
        min_embed_mm,
        screw_edge_diameters: SCREW_EDGE_DIAMETERS,
        dowel_edge_diameters: DOWEL_EDGE_DIAMETERS,
    }
}

fn safety_step(id: u32, needs: &[&str], material: Option<&str>) -> crate::rules_safety::Step {
    crate::rules_safety::Step {
        id,
        needs: needs.iter().map(|n| s(n)).collect(),
        material: material.map(s),
    }
}

/// A2: the sound chest, which every pack leaves alone.
pub fn sound() -> Golden {
    let core = crate::rules_core::ProjectFacts {
        goal: Some(s("A three-drawer chest for the spare room")),
        scale: crate::rules_core::ScaleClass::Furniture,
        prompts: vec![crate::rules_core::Prompt {
            id: s("drawer_runners"),
            skipped: true,
            relevant_now: false,
        }],
        parts: vec![
            core_part("carcase", [WIDTH_MM, DEPTH_MM, HEIGHT_MM]),
            core_part("shelf", [SHELF_SPAN_MM, DEPTH_MM, SHELF_THICKNESS_MM]),
            core_part(
                "drawer_front",
                [SHELF_SPAN_MM, DRAWER_FRONT_HEIGHT_MM, CARCASE_PANEL_MM],
            ),
        ],
    };

    let tools = crate::rules_tools::ToolFacts {
        steps: vec![
            tool_step(
                1,
                &["cut.straight"],
                Some(STEP_TOLERANCE_MM),
                Some(tool_slot(0, PLAN_START_MINUTE, PLAN_END_MINUTE)),
            ),
            tool_step(
                2,
                &["join.screw"],
                None,
                Some(tool_slot(1, PLAN_START_MINUTE, PLAN_END_MINUTE)),
            ),
            tool_step(
                3,
                &["finish.sand"],
                None,
                Some(tool_slot(2, PLAN_START_MINUTE, PLAN_END_MINUTE)),
            ),
            tool_step(
                4,
                &["cut.rebate"],
                Some(STEP_TOLERANCE_MM),
                Some(tool_slot(3, PLAN_START_MINUTE, PLAN_END_MINUTE)),
            ),
        ],
        tools: vec![
            tool(
                "table_saw",
                &["cut.straight"],
                crate::rules_tools::Access::Owned,
                Some(TOOL_ACCURACY_MM),
                crate::rules_tools::Noise::Loud,
            ),
            tool(
                "router",
                &["cut.rebate"],
                crate::rules_tools::Access::Owned,
                Some(TOOL_ACCURACY_MM),
                crate::rules_tools::Noise::Loud,
            ),
            tool(
                "drill",
                &["join.screw"],
                crate::rules_tools::Access::Owned,
                Some(1.0),
                crate::rules_tools::Noise::Moderate,
            ),
            tool(
                "sander",
                &["finish.sand"],
                crate::rules_tools::Access::Owned,
                Some(1.0),
                crate::rules_tools::Noise::Moderate,
            ),
        ],
        quiet_hours: vec![tool_slot(0, QUIET_START_MINUTE, QUIET_END_MINUTE)],
    };

    let pine = "wood.softwood.pine";
    let plywood = "wood.plywood.birch";
    let design = crate::rules_structure::Design {
        parts: vec![
            design_part("carcase_top", pine, SHELF_THICKNESS_MM),
            design_part("carcase_bottom", pine, CARCASE_PANEL_MM),
            design_part("side_left", pine, CARCASE_PANEL_MM),
            design_part("side_right", pine, CARCASE_PANEL_MM),
            design_part("back_panel", plywood, BACK_PANEL_MM),
            design_part("drawer_front_1", pine, CARCASE_PANEL_MM),
            design_part("drawer_front_2", pine, CARCASE_PANEL_MM),
            design_part("drawer_front_3", pine, CARCASE_PANEL_MM),
            design_part("drawer_box_1", plywood, DRAWER_BOX_MM),
            design_part("drawer_box_2", plywood, DRAWER_BOX_MM),
            design_part("drawer_box_3", plywood, DRAWER_BOX_MM),
        ],
        shelves: vec![crate::rules_structure::Shelf {
            part: s("carcase_top"),
            span_mm: SHELF_SPAN_MM,
            depth_mm: DEPTH_MM,
            load_kg_per_m: SOUND_LOAD_KG_PER_M,
        }],
        fasteners: vec![
            fastener(
                "screw_1",
                crate::rules_structure::FastenerKind::Screw,
                SCREW_LENGTH_MM,
                SCREW_DIAMETER_MM,
                &["side_left", "carcase_bottom"],
                "side_left",
                SCREW_EDGE_MM,
            ),
            fastener(
                "screw_2",
                crate::rules_structure::FastenerKind::Screw,
                SCREW_LENGTH_MM,
                SCREW_DIAMETER_MM,
                &["side_right", "carcase_bottom"],
                "side_right",
                SCREW_EDGE_MM,
            ),
            fastener(
                "screw_3",
                crate::rules_structure::FastenerKind::Screw,
                BACK_SCREW_LENGTH_MM,
                SCREW_DIAMETER_MM,
                &["back_panel", "side_right"],
                "back_panel",
                BACK_SCREW_EDGE_MM,
            ),
            fastener(
                "dowel_1",
                crate::rules_structure::FastenerKind::Dowel,
                DOWEL_LENGTH_MM,
                DOWEL_DIAMETER_MM,
                &["side_left", "carcase_top"],
                "side_left",
                DOWEL_EDGE_MM,
            ),
        ],
        contacts: vec![
            contact("carcase_top", "side_left"),
            contact("carcase_top", "side_right"),
            contact("carcase_top", "back_panel"),
            contact("carcase_bottom", "side_left"),
            contact("carcase_bottom", "side_right"),
            contact("carcase_bottom", "back_panel"),
            contact("side_left", "back_panel"),
            contact("side_right", "back_panel"),
            contact("drawer_front_1", "drawer_box_1"),
            contact("drawer_front_2", "drawer_box_2"),
            contact("drawer_front_3", "drawer_box_3"),
            contact("drawer_box_1", "side_left"),
            contact("drawer_box_2", "side_left"),
            contact("drawer_box_3", "side_left"),
        ],
        slots: vec![crate::rules_structure::Slot {
            part: s("carcase_bottom"),
            width_mm: SLOT_WIDTH_MM,
            remaining_mm: SLOT_REMAINING_MM,
        }],
    };

    let tables = crate::rules_structure::Tables {
        materials: vec![
            material_data(pine, PINE_STIFFNESS_MPA, MIN_EMBED_MM),
            material_data(plywood, PLYWOOD_STIFFNESS_MPA, PLYWOOD_MIN_EMBED_MM),
        ],
        sag_per_m_mm: SAG_PER_M_MM,
        source: s("Pine and birch plywood handbook ranges; not yet sourced in the app"),
    };

    let safety = crate::rules_safety::SafetyFacts {
        steps: vec![
            safety_step(1, &["cut.straight"], Some(pine)),
            safety_step(2, &["join.screw"], Some(pine)),
            safety_step(3, &["finish.sand"], None),
            safety_step(4, &["cut.rebate"], Some(plywood)),
        ],
        materials: vec![
            crate::rules_safety::Material {
                id: s(pine),
                laser_never: false,
            },
            crate::rules_safety::Material {
                id: s(plywood),
                laser_never: false,
            },
            crate::rules_safety::Material {
                id: s("plastic.pvc"),
                laser_never: true,
            },
        ],
        ventilated: Some(true),
        wiring: false,
        skill: Some(crate::rules_safety::Skill::Comfortable),
        furniture: vec![crate::rules_safety::Furniture {
            id: s("chest"),
            child: false,
            height_mm: HEIGHT_MM,
            drawers: true,
            anchored: Some(true),
        }],
        wall_parts: vec![crate::rules_safety::WallPart {
            id: s("shelf_above"),
            load_kg: WALL_LOAD_KG,
            anchor_rated_kg: Some(WALL_RATING_KG),
            wall: crate::rules_safety::Wall::Masonry,
        }],
    };

    Golden {
        core,
        tools,
        design,
        tables,
        safety,
    }
}

/// A3: the five seeded faults, in order.
pub fn seeds() -> Vec<Seed> {
    vec![
        Seed {
            rule: s("structure.sag"),
            entity: s("carcase_top"),
        },
        Seed {
            rule: s("structure.screw_through"),
            entity: s("screw_1"),
        },
        Seed {
            rule: s("structure.edge_distance"),
            entity: s("screw_2"),
        },
        Seed {
            rule: s("tools.missing"),
            entity: s("4"),
        },
        Seed {
            rule: s("safety.anti_tip"),
            entity: s("chest"),
        },
    ]
}

/// A4: the sound chest with the named faults applied; each is idempotent, >= 5 is ignored.
pub fn with_faults(indices: &[usize]) -> Golden {
    let mut golden = sound();
    for &index in indices {
        match index {
            0 => {
                for shelf in golden.design.shelves.iter_mut() {
                    if shelf.part == "carcase_top" {
                        shelf.load_kg_per_m = FAULT_SAG_LOAD_KG_PER_M;
                    }
                }
            }
            1 => {
                for fastener in golden.design.fasteners.iter_mut() {
                    if fastener.id == "screw_1" {
                        fastener.length_mm = FAULT_SCREW_LENGTH_MM;
                    }
                }
            }
            2 => {
                for fastener in golden.design.fasteners.iter_mut() {
                    if fastener.id == "screw_2" {
                        fastener.edge_distance_mm = FAULT_SCREW_EDGE_MM;
                    }
                }
            }
            3 => {
                golden.tools.tools.retain(|tool| tool.id != "router");
            }
            4 => {
                for furniture in golden.safety.furniture.iter_mut() {
                    if furniture.id == "chest" {
                        furniture.height_mm = FAULT_CHEST_HEIGHT_MM;
                        furniture.anchored = Some(false);
                    }
                }
            }
            _ => {}
        }
    }
    golden
}

/// A4: all five faults at once.
pub fn faulted() -> Golden {
    with_faults(&[0, 1, 2, 3, 4])
}

/// A5: the four packs in order, each list kept as its pack gave it.
pub fn evaluate_all(golden: &Golden) -> Vec<Finding> {
    let mut out = Vec::new();
    out.extend(crate::rules_core::evaluate(&golden.core));
    out.extend(crate::rules_tools::evaluate(&golden.tools));
    out.extend(crate::rules_structure::evaluate(
        &golden.design,
        &golden.tables,
    ));
    out.extend(crate::rules_safety::evaluate(&golden.safety));
    out
}

/// A6: seeds against findings, one to one, in `expected` order.
pub fn check(golden: &Golden, expected: &[Seed]) -> Report {
    let findings = evaluate_all(golden);
    let mut used = vec![false; findings.len()];
    let mut matched: Vec<Seed> = Vec::new();
    let mut missing: Vec<Seed> = Vec::new();
    for seed in expected {
        let mut hit = false;
        for (i, finding) in findings.iter().enumerate() {
            if used[i] {
                continue;
            }
            if rule_id(finding) == seed.rule && location_text(&finding.location) == seed.entity {
                used[i] = true;
                matched.push(seed.clone());
                hit = true;
                break;
            }
        }
        if !hit {
            missing.push(seed.clone());
        }
    }
    let mut false_flags: Vec<(String, String)> = Vec::new();
    for (i, finding) in findings.iter().enumerate() {
        if !used[i] {
            false_flags.push((rule_id(finding), location_text(&finding.location)));
        }
    }
    Report {
        matched,
        missing,
        false_flags,
    }
}

/// A7: the one-line report, with "flag" pluralised only when n != 1.
pub fn summary(report: &Report) -> String {
    let matched = report.matched.len();
    let total = report.matched.len() + report.missing.len();
    let n = report.false_flags.len();
    let plural = if n != 1 { "s" } else { "" };
    format!(
        "{} of {} seeded faults flagged, {} false flag{}",
        matched, total, n, plural
    )
}

/// A8: eight chests, each one number moved toward a limit and still inside it.
pub fn near_misses() -> Vec<(String, Golden)> {
    let mut out: Vec<(String, Golden)> = Vec::new();

    let mut g = sound();
    for shelf in g.design.shelves.iter_mut() {
        if shelf.part == "carcase_top" {
            shelf.load_kg_per_m = NEAR_LOAD_KG_PER_M;
        }
    }
    out.push((s("shelf_load_at_ninety_percent"), g));

    let mut g = sound();
    for fastener in g.design.fasteners.iter_mut() {
        if fastener.id == "screw_1" {
            fastener.length_mm = NEAR_SCREW_LENGTH_MM;
        }
    }
    out.push((s("screw_one_mm_short_of_poking_through"), g));

    let mut g = sound();
    for fastener in g.design.fasteners.iter_mut() {
        if fastener.id == "screw_2" {
            fastener.edge_distance_mm = NEAR_SCREW_EDGE_MM;
        }
    }
    out.push((s("screw_just_beyond_edge_distance"), g));

    let mut g = sound();
    for step in g.tools.steps.iter_mut() {
        if step.id == 1 {
            step.tolerance_mm = Some(NEAR_STEP_TOLERANCE_MM);
        }
    }
    out.push((s("step_tool_accurate_enough"), g));

    let mut g = sound();
    for furniture in g.safety.furniture.iter_mut() {
        if furniture.id == "chest" {
            furniture.anchored = Some(false);
        }
    }
    out.push((s("chest_not_tall"), g));

    let mut g = sound();
    for furniture in g.safety.furniture.iter_mut() {
        if furniture.id == "chest" {
            furniture.height_mm = FAULT_CHEST_HEIGHT_MM;
        }
    }
    out.push((s("tall_chest_anchored"), g));

    let mut g = sound();
    for wall_part in g.safety.wall_parts.iter_mut() {
        if wall_part.id == "shelf_above" {
            wall_part.load_kg = NEAR_WALL_LOAD_KG;
        }
    }
    out.push((s("wall_load_at_rating"), g));

    let mut g = sound();
    for slot in g.design.slots.iter_mut() {
        if slot.part == "carcase_bottom" {
            slot.remaining_mm = NEAR_SLOT_REMAINING_MM;
        }
    }
    out.push((s("slot_just_above_severing"), g));

    out
}
