//! Tests for mk_assist::golden written by Kimi (the Crew, campaign mk-core-33) from the
//! contract alone, without seeing the module.

//! Integration tests for the golden dresser fixture and its seeded-fault checker.

// Crew-written tests: a helper that builds a whole `Finding` takes one argument per
// field, and worked rows print constants to the contract's digits; neither matters in a
// test.
#![allow(
    clippy::too_many_arguments,
    clippy::approx_constant,
    clippy::cloned_ref_to_slice_refs,
    clippy::useless_vec
)]

use mk_assist::findings::{Finding, Location, Source};
use mk_assist::golden::{
    check, evaluate_all, faulted, location_text, near_misses, seeds, sound, summary, with_faults,
    Golden, Report, Seed, BACK_PANEL_MM, BACK_SCREW_EDGE_MM, BACK_SCREW_LENGTH_MM,
    CARCASE_PANEL_MM, DEPTH_MM, DOWEL_DIAMETER_MM, DOWEL_EDGE_DIAMETERS, DOWEL_EDGE_MM,
    DOWEL_LENGTH_MM, DRAWER_BOX_MM, DRAWER_FRONT_HEIGHT_MM, FAULT_CHEST_HEIGHT_MM,
    FAULT_SAG_LOAD_KG_PER_M, FAULT_SCREW_EDGE_MM, FAULT_SCREW_LENGTH_MM, HEIGHT_MM, MIN_EMBED_MM,
    MIN_WALL_MM, NEAR_LOAD_KG_PER_M, NEAR_SCREW_EDGE_MM, NEAR_SCREW_LENGTH_MM,
    NEAR_SLOT_REMAINING_MM, NEAR_STEP_TOLERANCE_MM, NEAR_WALL_LOAD_KG, PINE_STIFFNESS_MPA,
    PLAN_END_MINUTE, PLAN_START_MINUTE, PLYWOOD_MIN_EMBED_MM, PLYWOOD_STIFFNESS_MPA,
    QUIET_END_MINUTE, QUIET_START_MINUTE, SAG_PER_M_MM, SCREW_DIAMETER_MM, SCREW_EDGE_DIAMETERS,
    SCREW_EDGE_MM, SCREW_LENGTH_MM, SHELF_SPAN_MM, SHELF_THICKNESS_MM, SLOT_REMAINING_MM,
    SLOT_WIDTH_MM, SOUND_LOAD_KG_PER_M, STEP_TOLERANCE_MM, TOOL_ACCURACY_MM, WALL_LOAD_KG,
    WALL_RATING_KG, WIDTH_MM,
};
use mk_assist::rules_core::{Part as CorePart, ProjectFacts, Prompt, ScaleClass};
use mk_assist::rules_safety::{
    Furniture, Material as SafetyMaterial, SafetyFacts, Skill, Step as SafetyStep, Wall, WallPart,
};
use mk_assist::rules_structure::{
    Contact, Design, Fastener, FastenerKind, MaterialData, Part as StructurePart, Shelf,
    Slot as StructureSlot, Tables,
};
use mk_assist::rules_tools::{Access, Noise, Slot as ToolSlot, Step as ToolStep, Tool, ToolFacts};

fn seed(rule: &str, entity: &str) -> Seed {
    Seed {
        rule: rule.to_string(),
        entity: entity.to_string(),
    }
}

fn expected_seeds() -> Vec<Seed> {
    vec![
        seed("structure.sag", "carcase_top"),
        seed("structure.screw_through", "screw_1"),
        seed("structure.edge_distance", "screw_2"),
        seed("tools.missing", "4"),
        seed("safety.anti_tip", "chest"),
    ]
}

fn faulted_finding_pairs() -> Vec<(String, String)> {
    vec![
        ("tools.missing".to_string(), "4".to_string()),
        ("structure.sag".to_string(), "carcase_top".to_string()),
        ("structure.screw_through".to_string(), "screw_1".to_string()),
        ("structure.edge_distance".to_string(), "screw_2".to_string()),
        ("safety.anti_tip".to_string(), "chest".to_string()),
    ]
}

fn finding_pairs(golden: &Golden) -> Vec<(String, String)> {
    evaluate_all(golden)
        .iter()
        .map(|finding| {
            let rule = match &finding.source {
                Source::Rule(rule) => rule.clone(),
                Source::Mind => "mind".to_string(),
            };
            (rule, location_text(&finding.location))
        })
        .collect()
}

fn assert_golden_eq(actual: &Golden, expected: &Golden) {
    assert_eq!(actual.core, expected.core);
    assert_eq!(actual.tools, expected.tools);
    assert_eq!(actual.design, expected.design);
    assert_eq!(actual.tables, expected.tables);
    assert_eq!(actual.safety, expected.safety);
}

fn core_part(name: &str, size_mm: [f64; 3]) -> CorePart {
    CorePart {
        name: name.to_string(),
        size_mm,
    }
}

fn structure_part(id: &str, material: &str, thickness_mm: f64) -> StructurePart {
    StructurePart {
        id: id.to_string(),
        material: material.to_string(),
        thickness_mm,
    }
}

fn contact(a: &str, b: &str) -> Contact {
    Contact {
        a: a.to_string(),
        b: b.to_string(),
    }
}

#[test]
fn api_constants_match_the_contract_exactly() {
    assert_eq!(
        [
            WIDTH_MM,
            HEIGHT_MM,
            DEPTH_MM,
            CARCASE_PANEL_MM,
            SHELF_THICKNESS_MM,
            BACK_PANEL_MM,
            DRAWER_BOX_MM,
            DRAWER_FRONT_HEIGHT_MM,
            SHELF_SPAN_MM,
            SOUND_LOAD_KG_PER_M,
            SAG_PER_M_MM,
            PINE_STIFFNESS_MPA,
            PLYWOOD_STIFFNESS_MPA,
            MIN_WALL_MM,
            MIN_EMBED_MM,
            PLYWOOD_MIN_EMBED_MM,
            SCREW_EDGE_DIAMETERS,
            DOWEL_EDGE_DIAMETERS,
            SCREW_LENGTH_MM,
            SCREW_DIAMETER_MM,
            SCREW_EDGE_MM,
            BACK_SCREW_LENGTH_MM,
            BACK_SCREW_EDGE_MM,
            DOWEL_LENGTH_MM,
            DOWEL_DIAMETER_MM,
            DOWEL_EDGE_MM,
            TOOL_ACCURACY_MM,
            STEP_TOLERANCE_MM,
        ],
        [
            900.0, 1100.0, 450.0, 18.0, 12.0, 6.0, 12.0, 250.0, 864.0, 8.0, 2.0, 10000.0, 6000.0,
            3.0, 12.0, 10.0, 4.0, 3.0, 32.0, 4.0, 25.0, 20.0, 20.0, 30.0, 8.0, 30.0, 0.5, 1.0,
        ]
    );
    assert_eq!(
        [
            PLAN_START_MINUTE,
            PLAN_END_MINUTE,
            QUIET_START_MINUTE,
            QUIET_END_MINUTE,
        ],
        [540, 600, 1320, 1440]
    );
    assert_eq!(
        [
            FAULT_CHEST_HEIGHT_MM,
            WALL_LOAD_KG,
            WALL_RATING_KG,
            SLOT_WIDTH_MM,
            SLOT_REMAINING_MM,
            FAULT_SAG_LOAD_KG_PER_M,
            FAULT_SCREW_LENGTH_MM,
            FAULT_SCREW_EDGE_MM,
            NEAR_LOAD_KG_PER_M,
            NEAR_SCREW_LENGTH_MM,
            NEAR_SCREW_EDGE_MM,
            NEAR_STEP_TOLERANCE_MM,
            NEAR_WALL_LOAD_KG,
            NEAR_SLOT_REMAINING_MM,
        ],
        [1300.0, 12.0, 20.0, 12.0, 6.0, 20.0, 40.0, 8.0, 14.0, 35.0, 16.5, 0.6, 20.0, 0.5,]
    );
}

#[test]
fn a1_location_text_preserves_ids_fields_and_unpadded_steps() {
    assert_eq!(location_text(&Location::Project), "project");
    assert_eq!(
        location_text(&Location::Entity("chest/\u{0}".to_string())),
        "chest/\u{0}"
    );
    assert_eq!(location_text(&Location::Step(0)), "0");
    assert_eq!(location_text(&Location::Step(7)), "7");
    assert_eq!(location_text(&Location::Step(u32::MAX)), "4294967295");
    assert_eq!(
        location_text(&Location::Field("prompt.a".to_string())),
        "prompt.a"
    );
    assert_eq!(location_text(&Location::Entity(String::new())), "");
    assert_eq!(location_text(&Location::Field(String::new())), "");
}

#[test]
fn a2_sound_core_facts_are_exact() {
    let expected = ProjectFacts {
        goal: Some("A three-drawer chest for the spare room".to_string()),
        scale: ScaleClass::Furniture,
        prompts: vec![Prompt {
            id: "drawer_runners".to_string(),
            skipped: true,
            relevant_now: false,
        }],
        parts: vec![
            core_part("carcase", [900.0, 450.0, 1100.0]),
            core_part("shelf", [864.0, 450.0, 12.0]),
            core_part("drawer_front", [864.0, 250.0, 18.0]),
        ],
    };
    assert_eq!(sound().core, expected);
}

#[test]
fn a2_sound_tool_facts_are_exact() {
    let slot = |day| ToolSlot {
        day,
        start_minute: 540,
        end_minute: 600,
    };
    let expected = ToolFacts {
        steps: vec![
            ToolStep {
                id: 1,
                needs: vec!["cut.straight".to_string()],
                tolerance_mm: Some(1.0),
                planned: Some(slot(0)),
            },
            ToolStep {
                id: 2,
                needs: vec!["join.screw".to_string()],
                tolerance_mm: None,
                planned: Some(slot(1)),
            },
            ToolStep {
                id: 3,
                needs: vec!["finish.sand".to_string()],
                tolerance_mm: None,
                planned: Some(slot(2)),
            },
            ToolStep {
                id: 4,
                needs: vec!["cut.rebate".to_string()],
                tolerance_mm: Some(1.0),
                planned: Some(slot(3)),
            },
        ],
        tools: vec![
            Tool {
                id: "table_saw".to_string(),
                supplies: vec!["cut.straight".to_string()],
                access: Access::Owned,
                accuracy_mm: Some(0.5),
                noise: Noise::Loud,
            },
            Tool {
                id: "router".to_string(),
                supplies: vec!["cut.rebate".to_string()],
                access: Access::Owned,
                accuracy_mm: Some(0.5),
                noise: Noise::Loud,
            },
            Tool {
                id: "drill".to_string(),
                supplies: vec!["join.screw".to_string()],
                access: Access::Owned,
                accuracy_mm: Some(1.0),
                noise: Noise::Moderate,
            },
            Tool {
                id: "sander".to_string(),
                supplies: vec!["finish.sand".to_string()],
                access: Access::Owned,
                accuracy_mm: Some(1.0),
                noise: Noise::Moderate,
            },
        ],
        quiet_hours: vec![ToolSlot {
            day: 0,
            start_minute: 1320,
            end_minute: 1440,
        }],
    };
    assert_eq!(sound().tools, expected);
}

#[test]
fn a2_sound_structural_design_is_exact() {
    let expected = Design {
        parts: vec![
            structure_part("carcase_top", "wood.softwood.pine", 12.0),
            structure_part("carcase_bottom", "wood.softwood.pine", 18.0),
            structure_part("side_left", "wood.softwood.pine", 18.0),
            structure_part("side_right", "wood.softwood.pine", 18.0),
            structure_part("back_panel", "wood.plywood.birch", 6.0),
            structure_part("drawer_front_1", "wood.softwood.pine", 18.0),
            structure_part("drawer_front_2", "wood.softwood.pine", 18.0),
            structure_part("drawer_front_3", "wood.softwood.pine", 18.0),
            structure_part("drawer_box_1", "wood.plywood.birch", 12.0),
            structure_part("drawer_box_2", "wood.plywood.birch", 12.0),
            structure_part("drawer_box_3", "wood.plywood.birch", 12.0),
        ],
        shelves: vec![Shelf {
            part: "carcase_top".to_string(),
            span_mm: 864.0,
            depth_mm: 450.0,
            load_kg_per_m: 8.0,
        }],
        fasteners: vec![
            Fastener {
                id: "screw_1".to_string(),
                kind: FastenerKind::Screw,
                length_mm: 32.0,
                diameter_mm: 4.0,
                joins: vec!["side_left".to_string(), "carcase_bottom".to_string()],
                edge_of: "side_left".to_string(),
                edge_distance_mm: 25.0,
            },
            Fastener {
                id: "screw_2".to_string(),
                kind: FastenerKind::Screw,
                length_mm: 32.0,
                diameter_mm: 4.0,
                joins: vec!["side_right".to_string(), "carcase_bottom".to_string()],
                edge_of: "side_right".to_string(),
                edge_distance_mm: 25.0,
            },
            Fastener {
                id: "screw_3".to_string(),
                kind: FastenerKind::Screw,
                length_mm: 20.0,
                diameter_mm: 4.0,
                joins: vec!["back_panel".to_string(), "side_right".to_string()],
                edge_of: "back_panel".to_string(),
                edge_distance_mm: 20.0,
            },
            Fastener {
                id: "dowel_1".to_string(),
                kind: FastenerKind::Dowel,
                length_mm: 30.0,
                diameter_mm: 8.0,
                joins: vec!["side_left".to_string(), "carcase_top".to_string()],
                edge_of: "side_left".to_string(),
                edge_distance_mm: 30.0,
            },
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
        slots: vec![StructureSlot {
            part: "carcase_bottom".to_string(),
            width_mm: 12.0,
            remaining_mm: 6.0,
        }],
    };
    assert_eq!(sound().design, expected);
}

#[test]
fn a2_sound_material_tables_are_exact() {
    let expected = Tables {
        materials: vec![
            MaterialData {
                material: "wood.softwood.pine".to_string(),
                stiffness_mpa: 10000.0,
                min_wall_mm: 3.0,
                min_embed_mm: 12.0,
                screw_edge_diameters: 4.0,
                dowel_edge_diameters: 3.0,
            },
            MaterialData {
                material: "wood.plywood.birch".to_string(),
                stiffness_mpa: 6000.0,
                min_wall_mm: 3.0,
                min_embed_mm: 10.0,
                screw_edge_diameters: 4.0,
                dowel_edge_diameters: 3.0,
            },
        ],
        sag_per_m_mm: 2.0,
        source: "Pine and birch plywood handbook ranges; not yet sourced in the app".to_string(),
    };
    assert_eq!(sound().tables, expected);
}

#[test]
fn a2_sound_safety_facts_are_exact() {
    let expected = SafetyFacts {
        steps: vec![
            SafetyStep {
                id: 1,
                needs: vec!["cut.straight".to_string()],
                material: Some("wood.softwood.pine".to_string()),
            },
            SafetyStep {
                id: 2,
                needs: vec!["join.screw".to_string()],
                material: Some("wood.softwood.pine".to_string()),
            },
            SafetyStep {
                id: 3,
                needs: vec!["finish.sand".to_string()],
                material: None,
            },
            SafetyStep {
                id: 4,
                needs: vec!["cut.rebate".to_string()],
                material: Some("wood.plywood.birch".to_string()),
            },
        ],
        materials: vec![
            SafetyMaterial {
                id: "wood.softwood.pine".to_string(),
                laser_never: false,
            },
            SafetyMaterial {
                id: "wood.plywood.birch".to_string(),
                laser_never: false,
            },
            SafetyMaterial {
                id: "plastic.pvc".to_string(),
                laser_never: true,
            },
        ],
        ventilated: Some(true),
        wiring: false,
        skill: Some(Skill::Comfortable),
        furniture: vec![Furniture {
            id: "chest".to_string(),
            child: false,
            height_mm: 1100.0,
            drawers: true,
            anchored: Some(true),
        }],
        wall_parts: vec![WallPart {
            id: "shelf_above".to_string(),
            load_kg: 12.0,
            anchor_rated_kg: Some(20.0),
            wall: Wall::Masonry,
        }],
    };
    assert_eq!(sound().safety, expected);
    assert_eq!(evaluate_all(&sound()), Vec::<Finding>::new());
}

#[test]
fn a3_seeds_are_exactly_five_in_contract_order() {
    assert_eq!(seeds(), expected_seeds());
}

#[test]
fn a4_each_fault_changes_only_its_named_facts_and_is_idempotent() {
    let mut sag = sound();
    sag.design.shelves[0].load_kg_per_m = 20.0;
    assert_golden_eq(&with_faults(&[0]), &sag);

    let mut through = sound();
    through.design.fasteners[0].length_mm = 40.0;
    assert_golden_eq(&with_faults(&[1]), &through);

    let mut edge = sound();
    edge.design.fasteners[1].edge_distance_mm = 8.0;
    assert_golden_eq(&with_faults(&[2]), &edge);

    let mut missing = sound();
    missing.tools.tools.retain(|tool| tool.id != "router");
    assert_golden_eq(&with_faults(&[3]), &missing);

    let mut tip = sound();
    tip.safety.furniture[0].height_mm = 1300.0;
    tip.safety.furniture[0].anchored = Some(false);
    assert_golden_eq(&with_faults(&[4]), &tip);

    for index in 0..5 {
        assert_golden_eq(&with_faults(&[index, index, index]), &with_faults(&[index]));
    }
    assert_golden_eq(&with_faults(&[5, 99, usize::MAX]), &sound());
    assert_golden_eq(&with_faults(&[]), &sound());
    assert_golden_eq(&faulted(), &with_faults(&[0, 1, 2, 3, 4]));
}

#[test]
fn a5_evaluate_all_concatenates_pack_results_in_pack_order() {
    let golden = faulted();
    let mut expected = mk_assist::rules_core::evaluate(&golden.core);
    expected.extend(mk_assist::rules_tools::evaluate(&golden.tools));
    expected.extend(mk_assist::rules_structure::evaluate(
        &golden.design,
        &golden.tables,
    ));
    expected.extend(mk_assist::rules_safety::evaluate(&golden.safety));
    assert_eq!(evaluate_all(&golden), expected);
}

#[test]
fn a6_check_uses_expected_order_and_leaves_unused_findings_in_eval_order() {
    let all = expected_seeds();
    let expected = vec![all[4].clone(), all[0].clone()];
    let report = check(&faulted(), &expected);
    assert_eq!(
        report,
        Report {
            matched: expected,
            missing: Vec::new(),
            false_flags: vec![
                ("tools.missing".to_string(), "4".to_string()),
                ("structure.screw_through".to_string(), "screw_1".to_string(),),
                ("structure.edge_distance".to_string(), "screw_2".to_string(),),
            ],
        }
    );
}

#[test]
fn a7_report_perfection_and_summary_grammar_follow_the_counts() {
    let perfect = Report {
        matched: expected_seeds(),
        missing: Vec::new(),
        false_flags: Vec::new(),
    };
    assert!(perfect.is_perfect());
    assert!(Report {
        matched: Vec::new(),
        missing: Vec::new(),
        false_flags: Vec::new(),
    }
    .is_perfect());

    let missing = Report {
        matched: Vec::new(),
        missing: vec![seed("a.rule", "a")],
        false_flags: Vec::new(),
    };
    assert!(!missing.is_perfect());
    let false_flag = Report {
        matched: Vec::new(),
        missing: Vec::new(),
        false_flags: vec![("a.rule".to_string(), "a".to_string())],
    };
    assert!(!false_flag.is_perfect());

    let mixed = Report {
        matched: vec![seed("a.rule", "a"), seed("b.rule", "b")],
        missing: vec![seed("c.rule", "c")],
        false_flags: vec![
            ("d.rule".to_string(), "d".to_string()),
            ("e.rule".to_string(), "e".to_string()),
        ],
    };
    assert_eq!(
        summary(&mixed),
        "2 of 3 seeded faults flagged, 2 false flags"
    );
}

#[test]
fn a8_near_misses_have_exact_names_and_exactly_one_change_each() {
    let mut shelf = sound();
    shelf.design.shelves[0].load_kg_per_m = 14.0;
    let mut through = sound();
    through.design.fasteners[0].length_mm = 35.0;
    let mut edge = sound();
    edge.design.fasteners[1].edge_distance_mm = 16.5;
    let mut accuracy = sound();
    accuracy.tools.steps[0].tolerance_mm = Some(0.6);
    let mut short_unanchored = sound();
    short_unanchored.safety.furniture[0].anchored = Some(false);
    let mut tall_anchored = sound();
    tall_anchored.safety.furniture[0].height_mm = 1300.0;
    let mut wall = sound();
    wall.safety.wall_parts[0].load_kg = 20.0;
    let mut slot = sound();
    slot.design.slots[0].remaining_mm = 0.5;

    let expected = vec![
        ("shelf_load_at_ninety_percent".to_string(), shelf),
        ("screw_one_mm_short_of_poking_through".to_string(), through),
        ("screw_just_beyond_edge_distance".to_string(), edge),
        ("step_tool_accurate_enough".to_string(), accuracy),
        ("chest_not_tall".to_string(), short_unanchored),
        ("tall_chest_anchored".to_string(), tall_anchored),
        ("wall_load_at_rating".to_string(), wall),
        ("slot_just_above_severing".to_string(), slot),
    ];
    let actual = near_misses();
    let actual_names: Vec<String> = actual.iter().map(|(name, _)| name.clone()).collect();
    let expected_names: Vec<String> = expected.iter().map(|(name, _)| name.clone()).collect();
    assert_eq!(actual_names, expected_names);

    for ((actual_name, actual_golden), (_, expected_golden)) in actual.iter().zip(expected.iter()) {
        assert_golden_eq(actual_golden, expected_golden);
        assert_eq!(
            evaluate_all(actual_golden),
            Vec::<Finding>::new(),
            "near miss {actual_name} produced a finding"
        );
        assert_eq!(
            check(actual_golden, &expected_seeds()),
            Report {
                matched: Vec::new(),
                missing: expected_seeds(),
                false_flags: Vec::new(),
            }
        );
    }
}

#[test]
fn a9_nan_infinities_empty_lists_and_unknown_ids_never_panic() {
    let mut empty = sound();
    empty.core.prompts.clear();
    empty.core.parts.clear();
    empty.tools.steps.clear();
    empty.tools.tools.clear();
    empty.tools.quiet_hours.clear();
    empty.design = Design::default();
    empty.tables = Tables::default();
    empty.safety = SafetyFacts::default();
    assert_eq!(evaluate_all(&empty), Vec::<Finding>::new());
    assert_eq!(
        check(&empty, &expected_seeds()),
        Report {
            matched: Vec::new(),
            missing: expected_seeds(),
            false_flags: Vec::new(),
        }
    );

    let mut unknown = sound();
    unknown.design.shelves[0].part = "unknown.shelf".to_string();
    unknown.design.fasteners[0].joins =
        vec!["unknown.head".to_string(), "unknown.tail".to_string()];
    unknown.design.fasteners[0].edge_of = "unknown.edge".to_string();
    unknown.design.slots[0].part = "unknown.slot".to_string();
    unknown.safety.steps[0].material = Some("unknown.material".to_string());
    assert_eq!(evaluate_all(&unknown), Vec::<Finding>::new());

    let mut nan = sound();
    nan.core.parts[0].size_mm = [f64::NAN, 450.0, 1100.0];
    nan.tools.steps[0].tolerance_mm = Some(f64::NAN);
    nan.tools.tools[0].accuracy_mm = Some(f64::NAN);
    nan.design.parts[0].thickness_mm = f64::NAN;
    nan.design.shelves[0].load_kg_per_m = f64::NAN;
    nan.design.fasteners[0].length_mm = f64::NAN;
    nan.design.slots[0].remaining_mm = f64::NAN;
    nan.tables.materials[0].stiffness_mpa = f64::NAN;
    nan.tables.sag_per_m_mm = f64::NAN;
    nan.safety.furniture[0].height_mm = f64::NAN;
    nan.safety.wall_parts[0].load_kg = f64::NAN;

    let mut infinite = sound();
    infinite.core.parts[0].size_mm = [f64::INFINITY, 450.0, 1100.0];
    infinite.tools.steps[0].tolerance_mm = Some(f64::INFINITY);
    infinite.tools.tools[0].accuracy_mm = Some(f64::NEG_INFINITY);
    infinite.design.parts[0].thickness_mm = f64::INFINITY;
    infinite.design.shelves[0].load_kg_per_m = f64::INFINITY;
    infinite.design.fasteners[0].length_mm = f64::NEG_INFINITY;
    infinite.design.slots[0].remaining_mm = f64::INFINITY;
    infinite.tables.materials[0].stiffness_mpa = f64::INFINITY;
    infinite.tables.sag_per_m_mm = f64::NEG_INFINITY;
    infinite.safety.furniture[0].height_mm = f64::INFINITY;
    infinite.safety.wall_parts[0].load_kg = f64::INFINITY;

    for golden in [nan, infinite] {
        let _findings = evaluate_all(&golden);
        let report = check(&golden, &expected_seeds());
        let _summary = summary(&report);
        let _all_ignored = with_faults(&[usize::MAX, 99]);
    }
}

#[test]
fn worked_example_01_sound_evaluates_to_no_findings() {
    assert_eq!(evaluate_all(&sound()), Vec::<Finding>::new());
}

#[test]
fn worked_example_02_seeds_match_the_table() {
    assert_eq!(seeds(), expected_seeds());
}

#[test]
fn worked_example_03_faulted_findings_have_exact_rule_order() {
    assert_eq!(finding_pairs(&faulted()), faulted_finding_pairs());
}

#[test]
fn worked_example_04_faulted_check_is_a_perfect_five() {
    let report = check(&faulted(), &seeds());
    assert_eq!(
        report,
        Report {
            matched: expected_seeds(),
            missing: Vec::new(),
            false_flags: Vec::new(),
        }
    );
    assert!(report.is_perfect());
}

#[test]
fn worked_example_05_sound_check_has_five_missing_and_no_false_flags() {
    let report = check(&sound(), &seeds());
    assert_eq!(
        report,
        Report {
            matched: Vec::new(),
            missing: expected_seeds(),
            false_flags: Vec::new(),
        }
    );
    assert!(!report.is_perfect());
}

#[test]
fn worked_example_06_perfect_summary_is_exact() {
    assert_eq!(
        summary(&check(&faulted(), &seeds())),
        "5 of 5 seeded faults flagged, 0 false flags"
    );
}

#[test]
fn worked_example_07_sound_summary_is_exact() {
    assert_eq!(
        summary(&check(&sound(), &seeds())),
        "0 of 5 seeded faults flagged, 0 false flags"
    );
}

#[test]
fn worked_example_08_fault_zero_only_flags_sag_at_carcase_top() {
    assert_eq!(
        finding_pairs(&with_faults(&[0])),
        vec![("structure.sag".to_string(), "carcase_top".to_string())]
    );
}

#[test]
fn worked_example_09_fault_one_only_flags_screw_through_at_screw_one() {
    assert_eq!(
        finding_pairs(&with_faults(&[1])),
        vec![("structure.screw_through".to_string(), "screw_1".to_string())]
    );
}

#[test]
fn worked_example_10_fault_two_only_flags_edge_distance_at_screw_two() {
    assert_eq!(
        finding_pairs(&with_faults(&[2])),
        vec![("structure.edge_distance".to_string(), "screw_2".to_string())]
    );
}

#[test]
fn worked_example_11_fault_three_only_flags_missing_tool_at_step_four() {
    assert_eq!(
        finding_pairs(&with_faults(&[3])),
        vec![("tools.missing".to_string(), "4".to_string())]
    );
}

#[test]
fn worked_example_12_fault_four_only_flags_anti_tip_at_chest() {
    assert_eq!(
        finding_pairs(&with_faults(&[4])),
        vec![("safety.anti_tip".to_string(), "chest".to_string())]
    );
}

#[test]
fn worked_example_13_repeated_and_out_of_range_fault_indices_are_ignored() {
    assert_golden_eq(&with_faults(&[0, 0, 0, 99]), &with_faults(&[0]));
    assert_eq!(
        finding_pairs(&with_faults(&[0, 0, 0, 99])),
        vec![("structure.sag".to_string(), "carcase_top".to_string())]
    );
}

#[test]
fn worked_example_14_no_fault_indices_is_the_sound_chest() {
    assert_golden_eq(&with_faults(&[]), &sound());
    assert_eq!(evaluate_all(&with_faults(&[])), Vec::<Finding>::new());
}

#[test]
fn worked_example_15_reverse_fault_application_keeps_evaluation_order() {
    assert_golden_eq(&with_faults(&[4, 3, 2, 1, 0]), &faulted());
    assert_eq!(
        finding_pairs(&with_faults(&[4, 3, 2, 1, 0])),
        faulted_finding_pairs()
    );
}

#[test]
fn worked_example_16_all_location_variants_have_exact_text() {
    assert_eq!(
        vec![
            location_text(&Location::Project),
            location_text(&Location::Entity("chest".to_string())),
            location_text(&Location::Step(7)),
            location_text(&Location::Field("prompt.a".to_string())),
        ],
        vec!["project", "chest", "7", "prompt.a"]
    );
}

#[test]
fn worked_example_17_all_eight_named_near_misses_are_silent() {
    let near = near_misses();
    let names: Vec<String> = near.iter().map(|(name, _)| name.clone()).collect();
    assert_eq!(
        names,
        vec![
            "shelf_load_at_ninety_percent",
            "screw_one_mm_short_of_poking_through",
            "screw_just_beyond_edge_distance",
            "step_tool_accurate_enough",
            "chest_not_tall",
            "tall_chest_anchored",
            "wall_load_at_rating",
            "slot_just_above_severing",
        ]
    );
    for (_, golden) in near {
        assert_eq!(evaluate_all(&golden), Vec::<Finding>::new());
    }
}

#[test]
fn worked_example_18_one_false_flag_is_singular() {
    let report = Report {
        matched: Vec::new(),
        missing: Vec::new(),
        false_flags: vec![("tools.missing".to_string(), "4".to_string())],
    };
    assert_eq!(
        summary(&report),
        "0 of 0 seeded faults flagged, 1 false flag"
    );
}

#[test]
fn worked_example_19_empty_expected_list_makes_all_five_findings_false_flags() {
    let report = check(&faulted(), &[]);
    assert_eq!(
        report,
        Report {
            matched: Vec::new(),
            missing: Vec::new(),
            false_flags: faulted_finding_pairs(),
        }
    );
    assert!(!report.is_perfect());
}

#[test]
fn worked_example_20_duplicate_seed_consumes_the_finding_only_once() {
    let duplicated = expected_seeds()[0].clone();
    let report = check(
        &with_faults(&[0]),
        &[duplicated.clone(), duplicated.clone()],
    );
    assert_eq!(
        report,
        Report {
            matched: vec![duplicated.clone()],
            missing: vec![duplicated],
            false_flags: Vec::new(),
        }
    );
}

#[test]
fn worked_example_21_unknown_seed_is_missing_without_false_flags() {
    let unknown = seed("nope.rule", "nothing");
    let report = check(&sound(), &[unknown.clone()]);
    assert_eq!(
        report,
        Report {
            matched: Vec::new(),
            missing: vec![unknown],
            false_flags: Vec::new(),
        }
    );
}
