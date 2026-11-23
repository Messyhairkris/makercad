//! Contract tests for mk_assist::design_facts, written by Gordon as a PA (the Crew) from
//! crew/contracts/mk_design_facts.md. They were the hidden tests of campaign mk-core-34.

// Crew-written tests: a helper that builds a whole `Finding` takes one argument per
// field, and worked rows print constants to the contract's digits; neither matters in a
// test.
#![allow(
    clippy::too_many_arguments,
    clippy::approx_constant,
    // Two tests pin the named constants' starting values.
    clippy::assertions_on_constants
)]

use mk_assist::design_facts::{
    contacts, design, parts_of, shelves, size, thickness_mm, touching, BoxPart,
    CONTACT_TOLERANCE_MM, FLOOR_ID, MIN_SHELF_SPAN_MM,
};
use mk_assist::rules_structure::{
    evaluate, sag_mm, Fastener, FastenerKind, MaterialData, Part, Slot, Tables,
};

const WOOD: &str = "wood.softwood.pine";

fn bp(id: &str, material: &str, min: [f64; 3], max: [f64; 3]) -> BoxPart {
    BoxPart {
        id: id.to_string(),
        material: material.to_string(),
        min,
        max,
    }
}

fn wood(id: &str, min: [f64; 3], max: [f64; 3]) -> BoxPart {
    bp(id, WOOD, min, max)
}

fn part_ids(list: &[Part]) -> Vec<String> {
    list.iter().map(|part| part.id.clone()).collect()
}

// A1 - a part is usable when its id is not empty and its box is finite and positive.
#[test]
fn a1_empty_id_is_unusable() {
    let part = bp("", WOOD, [0.0, 0.0, 0.0], [10.0, 20.0, 30.0]);
    assert!(size(&part).is_none());
    assert!(thickness_mm(&part).is_none());
    assert!(parts_of(&[part]).is_empty());
}

#[test]
fn a1_non_finite_and_zero_size_are_unusable() {
    let nan = bp("nan", WOOD, [f64::NAN, 0.0, 0.0], [10.0, 10.0, 10.0]);
    let inf = bp("inf", WOOD, [0.0, 0.0, 0.0], [f64::INFINITY, 10.0, 10.0]);
    let flat = bp("flat", WOOD, [0.0, 0.0, 0.0], [10.0, 10.0, 0.0]);
    let crossed = bp("crossed", WOOD, [0.0, 0.0, 0.0], [10.0, -5.0, 10.0]);
    assert!(size(&nan).is_none());
    assert!(size(&inf).is_none());
    assert!(size(&flat).is_none());
    assert!(size(&crossed).is_none());
    assert!(parts_of(&[nan, inf, flat, crossed]).is_empty());
}

#[test]
fn a1_later_duplicate_id_is_ignored() {
    let parts = vec![
        wood("a", [0.0, 0.0, 0.0], [10.0, 10.0, 10.0]),
        wood("a", [100.0, 100.0, 100.0], [200.0, 200.0, 200.0]),
    ];
    assert!(part_ids(&parts_of(&parts)) == vec!["a".to_string()]);
    assert!(parts_of(&parts)[0].thickness_mm == 10.0);
    assert!(contacts(&parts).is_empty());
}

// A2 - the side lengths, and the smallest side as the thickness.
#[test]
fn a2_size_and_thickness() {
    let part = wood("a", [0.0, 0.0, 0.0], [10.0, 20.0, 30.0]);
    let sides = size(&part);
    assert!(sides.is_some());
    let sides = sides.unwrap_or([f64::NAN; 3]);
    assert!(sides[0] == 10.0 && sides[1] == 20.0 && sides[2] == 30.0);
    assert!(thickness_mm(&part).unwrap_or(f64::NAN) == 10.0);
    let cube = wood("cube", [0.0, 0.0, 0.0], [5.0, 5.0, 5.0]);
    assert!(thickness_mm(&cube).unwrap_or(f64::NAN) == 5.0);
}

// A3 - the tolerance is the named constant, 1 mm in v1.
#[test]
fn a3_tolerance_constants() {
    assert!(CONTACT_TOLERANCE_MM == 1.0);
    assert!(MIN_SHELF_SPAN_MM == 300.0);
    assert!(FLOOR_ID == "floor");
}

// A4 - touching grows both boxes by half the tolerance on every side.
#[test]
fn a4_gap_within_tolerance_touches() {
    let a = wood("a", [0.0, 0.0, 0.0], [10.0, 10.0, 10.0]);
    let exact = wood("exact", [10.0, 0.0, 0.0], [20.0, 10.0, 10.0]);
    let at_tolerance = wood("tol", [11.0, 0.0, 0.0], [20.0, 10.0, 10.0]);
    let beyond = wood("beyond", [11.1, 0.0, 0.0], [20.0, 10.0, 10.0]);
    assert!(touching(&a, &exact));
    assert!(touching(&a, &at_tolerance));
    assert!(!touching(&a, &beyond));
}

#[test]
fn a4_overlap_and_self_and_unusable() {
    let a = wood("a", [0.0, 0.0, 0.0], [10.0, 10.0, 10.0]);
    let overlapping = wood("b", [9.0, 0.0, 0.0], [20.0, 10.0, 10.0]);
    assert!(touching(&a, &overlapping));
    assert!(!touching(&a, &a));
    let broken = bp("broken", WOOD, [10.0, 0.0, 0.0], [f64::NAN, 10.0, 10.0]);
    assert!(!touching(&a, &broken));
    assert!(!touching(&broken, &a));
}

#[test]
fn a4_separated_on_one_axis_does_not_touch() {
    let a = wood("a", [0.0, 0.0, 0.0], [10.0, 10.0, 10.0]);
    let above = wood("above", [0.0, 0.0, 50.0], [10.0, 10.0, 60.0]);
    assert!(!touching(&a, &above));
}

// A5 - parts_of keeps usable parts in input order, with id, material and thickness.
#[test]
fn a5_parts_of_order_and_fields() {
    let parts = vec![
        wood("a", [0.0, 0.0, 0.0], [10.0, 20.0, 30.0]),
        wood("a", [0.0, 0.0, 0.0], [40.0, 40.0, 40.0]),
        bp("", WOOD, [0.0, 0.0, 0.0], [10.0, 10.0, 10.0]),
        wood("b", [100.0, 0.0, 0.0], [130.0, 20.0, 40.0]),
    ];
    let got = parts_of(&parts);
    assert!(got.len() == 2);
    assert!(got[0].id == "a" && got[1].id == "b");
    assert!(got[0].material == WOOD);
    assert!(got[0].thickness_mm == 10.0);
    assert!(got[1].thickness_mm == 20.0);
}

// A6 - every touching pair once, earlier part as `a`, ordered by (a index, b index).
#[test]
fn a6_contacts_pairs_and_order() {
    let parts = vec![
        wood("a", [0.0, 0.0, 0.0], [10.0, 10.0, 10.0]),
        wood("b", [10.0, 0.0, 0.0], [20.0, 10.0, 10.0]),
        wood("c", [20.0, 0.0, 0.0], [30.0, 10.0, 10.0]),
    ];
    let got = contacts(&parts);
    assert!(got.len() == 2);
    assert!(got[0].a == "a" && got[0].b == "b");
    assert!(got[1].a == "b" && got[1].b == "c");
}

#[test]
fn a6_shared_part_contacts_ordered_by_a_then_b() {
    let parts = vec![
        wood("a", [0.0, 0.0, 0.0], [10.0, 10.0, 10.0]),
        wood("b", [10.0, 0.0, 0.0], [20.0, 10.0, 10.0]),
        wood("c", [-10.0, 0.0, 0.0], [0.0, 10.0, 10.0]),
    ];
    let got = contacts(&parts);
    assert!(got.len() == 2);
    assert!(got[0].a == "a" && got[0].b == "b");
    assert!(got[1].a == "a" && got[1].b == "c");
}

#[test]
fn a6_no_self_contact_and_apart_is_empty() {
    let parts = vec![
        wood("a", [0.0, 0.0, 0.0], [10.0, 10.0, 10.0]),
        wood("far", [500.0, 0.0, 0.0], [510.0, 10.0, 10.0]),
    ];
    assert!(contacts(&parts).is_empty());
    assert!(contacts(&parts[0..1]).is_empty());
    assert!(contacts(&[]).is_empty());
}

#[test]
fn a6_unusable_and_duplicate_parts_contact_nothing() {
    let parts = vec![
        wood("a", [0.0, 0.0, 0.0], [10.0, 10.0, 10.0]),
        bp("", WOOD, [0.0, 0.0, 0.0], [10.0, 10.0, 10.0]),
        wood("a", [0.0, 0.0, 0.0], [10.0, 10.0, 10.0]),
    ];
    assert!(contacts(&parts).is_empty());
}

// A7 and A8 - a shelf spans two supports, is the thinnest side up, and is loaded evenly.
fn shelf_rig(span: f64, depth: f64, thickness: f64, legs: bool) -> Vec<BoxPart> {
    let mut parts = vec![wood(
        "board",
        [0.0, 0.0, 700.0],
        [span, depth, 700.0 + thickness],
    )];
    if legs {
        parts.push(wood("leg_l", [0.0, 0.0, 0.0], [18.0, depth, 700.0]));
        parts.push(wood("leg_r", [span - 18.0, 0.0, 0.0], [span, depth, 700.0]));
    }
    parts
}

#[test]
fn a7_shelf_span_depth_and_load() {
    let parts = shelf_rig(400.0, 300.0, 18.0, true);
    let got = shelves(&parts, 5.0);
    assert!(got.len() == 1);
    assert!(got[0].part == "board");
    assert!(got[0].span_mm == 400.0);
    assert!(got[0].depth_mm == 300.0);
    assert!(got[0].load_kg_per_m == 5.0);
}

#[test]
fn a7_shelf_needs_both_ends_held() {
    let mut parts = shelf_rig(400.0, 300.0, 18.0, false);
    assert!(shelves(&parts, 5.0).is_empty());
    parts.push(wood("leg_l", [0.0, 0.0, 0.0], [18.0, 300.0, 700.0]));
    assert!(shelves(&parts, 5.0).is_empty());
    parts.push(wood("leg_r", [382.0, 0.0, 0.0], [400.0, 300.0, 700.0]));
    assert!(shelves(&parts, 5.0).len() == 1);
}

#[test]
fn a7_back_panel_holds_neither_end() {
    let parts = vec![
        wood("board", [0.0, 0.0, 0.0], [400.0, 300.0, 18.0]),
        wood("back", [0.0, 0.0, -500.0], [400.0, 18.0, 0.0]),
    ];
    assert!(shelves(&parts, 5.0).is_empty());
}

#[test]
fn a7_load_must_be_finite_and_not_negative() {
    let parts = shelf_rig(400.0, 300.0, 18.0, true);
    assert!(shelves(&parts, -1.0).is_empty());
    assert!(shelves(&parts, f64::NAN).is_empty());
    assert!(shelves(&parts, f64::INFINITY).is_empty());
    assert!(shelves(&parts, 0.0).len() == 1);
    assert!(shelves(&parts, 12.5)[0].load_kg_per_m == 12.5);
}

#[test]
fn a7_span_must_reach_the_minimum() {
    assert!(shelves(&shelf_rig(299.0, 300.0, 18.0, true), 5.0).is_empty());
    let at_minimum = shelves(&shelf_rig(300.0, 300.0, 18.0, true), 5.0);
    assert!(at_minimum.len() == 1);
    assert!(at_minimum[0].span_mm == 300.0);
}

#[test]
fn a7_vertical_must_be_the_smallest_side() {
    let plank = wood("plank", [0.0, 0.0, 0.0], [600.0, 20.0, 400.0]);
    let leg_l = wood("leg_l", [0.0, 0.0, 0.0], [18.0, 600.0, 20.0]);
    let leg_r = wood("leg_r", [582.0, 0.0, 0.0], [600.0, 600.0, 20.0]);
    assert!(shelves(&[plank, leg_l, leg_r], 5.0).is_empty());
}

#[test]
fn a7_cube_is_not_a_shelf() {
    let cube = wood("cube", [0.0, 0.0, 0.0], [500.0, 500.0, 500.0]);
    let leg_l = wood("leg_l", [0.0, 0.0, 0.0], [18.0, 500.0, 500.0]);
    let leg_r = wood("leg_r", [482.0, 0.0, 0.0], [500.0, 500.0, 500.0]);
    assert!(shelves(&[cube, leg_l, leg_r], 5.0).is_empty());
}

#[test]
fn a7_equal_horizontal_sides_span_x() {
    let parts = vec![
        wood("board", [0.0, 0.0, 400.0], [400.0, 400.0, 418.0]),
        wood("leg_l", [0.0, 0.0, 0.0], [18.0, 400.0, 400.0]),
        wood("leg_r", [382.0, 0.0, 0.0], [400.0, 400.0, 400.0]),
    ];
    let got = shelves(&parts, 5.0);
    assert!(got.len() == 1);
    assert!(got[0].span_mm == 400.0 && got[0].depth_mm == 400.0);
}

#[test]
fn a7_quarter_boundaries_count_as_ends() {
    let parts = vec![
        wood("board", [0.0, 0.0, 0.0], [400.0, 300.0, 18.0]),
        wood("leg_l", [91.0, 0.0, -100.0], [109.0, 300.0, 0.0]),
        wood("leg_r", [291.0, 0.0, -100.0], [309.0, 300.0, 0.0]),
    ];
    assert!(shelves(&parts, 5.0).len() == 1);
}

#[test]
fn a7_middle_centre_holds_no_end() {
    let parts = vec![
        wood("board", [0.0, 0.0, 0.0], [400.0, 300.0, 18.0]),
        wood("mid", [191.0, 0.0, -100.0], [209.0, 300.0, 0.0]),
    ];
    assert!(shelves(&parts, 5.0).is_empty());
}

// A8 - the shelf's fields are the part id, the span, the depth and the load as given.
#[test]
fn a8_shelf_fields_are_copied_exactly() {
    let parts = vec![
        wood("board", [0.0, 0.0, 500.0], [500.0, 250.0, 512.0]),
        wood("leg_l", [0.0, 0.0, 0.0], [18.0, 250.0, 500.0]),
        wood("leg_r", [482.0, 0.0, 0.0], [500.0, 250.0, 500.0]),
    ];
    let got = shelves(&parts, 3.25);
    assert!(got.len() == 1);
    assert!(got[0].part == "board");
    assert!(got[0].span_mm == 500.0);
    assert!(got[0].depth_mm == 250.0);
    assert!(got[0].load_kg_per_m == 3.25);
    let through_design = design(&parts, None, 3.25, Vec::new(), Vec::new());
    assert!(through_design.shelves == got);
}

// A9 - design is parts_of, contacts, shelves, and the fasteners and slots unchanged.
#[test]
fn a9_design_passes_fasteners_and_slots_through() {
    let parts = vec![
        wood("a", [0.0, 0.0, 0.0], [100.0, 100.0, 18.0]),
        wood("b", [100.0, 0.0, 0.0], [200.0, 100.0, 18.0]),
    ];
    let fastener = Fastener {
        id: "s1".to_string(),
        kind: FastenerKind::Screw,
        length_mm: 40.0,
        diameter_mm: 4.0,
        joins: vec!["a".to_string(), "b".to_string()],
        edge_of: "a".to_string(),
        edge_distance_mm: 20.0,
    };
    let slot = Slot {
        part: "a".to_string(),
        width_mm: 6.0,
        remaining_mm: 4.0,
    };
    let got = design(
        &parts,
        None,
        5.0,
        vec![fastener.clone()],
        vec![slot.clone()],
    );
    assert!(got.fasteners.len() == 1 && got.fasteners[0] == fastener);
    assert!(got.slots.len() == 1 && got.slots[0] == slot);
    assert!(got.parts.len() == 2);
    assert!(got.contacts.len() == 1 && got.contacts[0].a == "a");
    assert!(got.shelves.is_empty());
}

#[test]
fn a9_design_on_empty_input_is_empty() {
    let got = design(&[], None, 5.0, Vec::new(), Vec::new());
    assert!(got.parts.is_empty());
    assert!(got.contacts.is_empty());
    assert!(got.shelves.is_empty());
}

// A10 - the floor is a synthetic part, and only parts within the tolerance stand on it.
#[test]
fn a10_floor_part_is_appended_and_not_measurable() {
    let parts = vec![wood("a", [0.0, 0.0, 0.0], [100.0, 100.0, 18.0])];
    let got = design(&parts, Some(0.0), 5.0, Vec::new(), Vec::new());
    assert!(got.parts.len() == 2);
    assert!(got.parts[1].id == "floor");
    assert!(got.parts[1].material == "floor");
    assert!(got.parts[1].thickness_mm.is_nan());
    assert!(got.contacts.len() == 1);
    assert!(got.contacts[0].a == "a" && got.contacts[0].b == "floor");
}

#[test]
fn a10_floor_tolerance_band() {
    let parts = vec![
        wood("sunk_ok", [0.0, 0.0, -0.5], [10.0, 10.0, 9.5]),
        wood("sunk_edge", [100.0, 0.0, -1.0], [110.0, 10.0, 9.0]),
        wood("sunk_deep", [200.0, 0.0, -1.1], [210.0, 10.0, 8.9]),
        wood("float_ok", [300.0, 0.0, 1.0], [310.0, 10.0, 11.0]),
        wood("float_high", [400.0, 0.0, 1.1], [410.0, 10.0, 11.1]),
    ];
    let got = design(&parts, Some(0.0), 5.0, Vec::new(), Vec::new());
    let floor_contacts = got.contacts.iter().filter(|c| c.b == "floor").count();
    assert!(floor_contacts == 3);
    assert!(got.contacts.iter().any(|c| c.a == "sunk_ok"));
    assert!(got.contacts.iter().any(|c| c.a == "sunk_edge"));
    assert!(got.contacts.iter().any(|c| c.a == "float_ok"));
    assert!(!got.contacts.iter().any(|c| c.a == "sunk_deep"));
    assert!(!got.contacts.iter().any(|c| c.a == "float_high"));
}

#[test]
fn a10_floor_contacts_come_after_part_contacts() {
    let parts = vec![
        wood("a", [0.0, 0.0, 0.0], [10.0, 10.0, 10.0]),
        wood("b", [10.0, 0.0, 0.0], [20.0, 10.0, 10.0]),
    ];
    let got = design(&parts, Some(0.0), 5.0, Vec::new(), Vec::new());
    assert!(got.contacts.len() == 3);
    assert!(got.contacts[0].a == "a" && got.contacts[0].b == "b");
    assert!(got.contacts[1].a == "a" && got.contacts[1].b == "floor");
    assert!(got.contacts[2].a == "b" && got.contacts[2].b == "floor");
}

#[test]
fn a10_no_floor_without_a_finite_height() {
    let parts = vec![wood("a", [0.0, 0.0, 0.0], [100.0, 100.0, 18.0])];
    let none = design(&parts, None, 5.0, Vec::new(), Vec::new());
    assert!(none.parts.len() == 1);
    let nan = design(&parts, Some(f64::NAN), 5.0, Vec::new(), Vec::new());
    assert!(nan.parts.len() == 1 && nan.contacts.is_empty());
    let inf = design(&parts, Some(f64::INFINITY), 5.0, Vec::new(), Vec::new());
    assert!(inf.parts.len() == 1 && inf.contacts.is_empty());
}

#[test]
fn a10_measured_part_named_floor_is_ignored() {
    let parts = vec![
        wood("floor", [0.0, 0.0, 0.0], [100.0, 100.0, 18.0]),
        wood("a", [0.0, 0.0, 0.0], [10.0, 10.0, 10.0]),
    ];
    assert!(part_ids(&parts_of(&parts)) == vec!["a".to_string()]);
    let got = design(&parts, Some(0.0), 5.0, Vec::new(), Vec::new());
    assert!(got.parts.len() == 2);
    assert!(got.parts[0].id == "a" && got.parts[1].id == "floor");
    assert!(got.contacts.len() == 1);
    assert!(got.contacts[0].a == "a" && got.contacts[0].b == "floor");
}

// The worked example: the golden dresser, boxes only, standing on a floor at z = 0.
fn dresser_parts(top_thickness_mm: f64) -> Vec<BoxPart> {
    vec![
        wood("side_left", [0.0, 0.0, 0.0], [18.0, 450.0, 800.0]),
        wood("side_right", [482.0, 0.0, 0.0], [500.0, 450.0, 800.0]),
        wood(
            "top",
            [0.0, 0.0, 800.0 - top_thickness_mm],
            [500.0, 450.0, 800.0],
        ),
        wood("bottom", [0.0, 0.0, 0.0], [500.0, 450.0, 18.0]),
        wood(
            "back",
            [0.0, 0.0, 18.0],
            [500.0, 18.0, 800.0 - top_thickness_mm],
        ),
        wood("drawer_front", [200.0, 430.0, 18.0], [300.0, 450.0, 138.0]),
        wood("drawer_side_l", [200.0, 20.0, 18.0], [220.0, 430.0, 138.0]),
        wood("drawer_side_r", [280.0, 20.0, 18.0], [300.0, 430.0, 138.0]),
        wood("drawer_back", [200.0, 20.0, 18.0], [300.0, 40.0, 138.0]),
    ]
}

fn dresser_tables() -> Tables {
    Tables {
        materials: vec![MaterialData {
            material: WOOD.to_string(),
            stiffness_mpa: 10000.0,
            min_wall_mm: 5.0,
            min_embed_mm: 8.0,
            screw_edge_diameters: 3.0,
            dowel_edge_diameters: 2.0,
        }],
        sag_per_m_mm: 2.0,
        source: "hidden test tables".to_string(),
    }
}

#[test]
fn dresser_only_the_top_and_the_bottom_are_shelves() {
    let parts = dresser_parts(18.0);
    let got = design(&parts, Some(0.0), 5.0, Vec::new(), Vec::new());
    let shelf_ids: Vec<String> = got.shelves.iter().map(|s| s.part.clone()).collect();
    assert!(shelf_ids.len() == 2);
    assert!(shelf_ids[0] == "top" && shelf_ids[1] == "bottom");
    assert!(got.shelves[0].span_mm == 500.0);
    assert!(got.shelves[0].depth_mm == 450.0);
}

#[test]
fn dresser_every_part_is_held() {
    let parts = dresser_parts(18.0);
    let got = design(&parts, Some(0.0), 5.0, Vec::new(), Vec::new());
    for part in &got.parts {
        let held = got
            .contacts
            .iter()
            .any(|c| (c.a == part.id && c.b != part.id) || (c.b == part.id && c.a != part.id));
        assert!(held);
    }
}

#[test]
fn dresser_gives_no_findings() {
    let parts = dresser_parts(18.0);
    let got = design(&parts, Some(0.0), 5.0, Vec::new(), Vec::new());
    let findings = evaluate(&got, &dresser_tables());
    assert!(findings.is_empty());
}

#[test]
fn dresser_with_a_thin_top_gives_exactly_the_sag_finding() {
    let parts = dresser_parts(6.0);
    let got = design(&parts, Some(0.0), 20.0, Vec::new(), Vec::new());
    let top_is_a_shelf = got.shelves.iter().any(|s| s.part == "top");
    assert!(top_is_a_shelf);
    let sag = sag_mm(500.0, 450.0, 6.0, 10000.0, 20.0).unwrap_or(f64::NAN);
    assert!(sag > 1.5 && sag < 2.5);
    let findings = evaluate(&got, &dresser_tables());
    assert!(findings.len() == 1);
    assert!(format!("{:?}", findings[0]).contains("structure.sag"));
}

// No panic on garbage: fewer facts, never a crash.
#[test]
fn never_panics_on_bad_input() {
    let bad = vec![
        bp("", WOOD, [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]),
        bp("nan", WOOD, [f64::NAN, 0.0, 0.0], [1.0, 1.0, 1.0]),
        bp("inf", WOOD, [0.0, 0.0, 0.0], [f64::INFINITY, 1.0, 1.0]),
        bp("flat", WOOD, [0.0, 0.0, 0.0], [0.0, 1.0, 1.0]),
        bp("crossed", WOOD, [1.0, 1.0, 1.0], [0.0, 0.0, 0.0]),
        bp("dup", WOOD, [0.0, 0.0, 0.0], [10.0, 10.0, 10.0]),
        bp("dup", WOOD, [0.0, 0.0, 0.0], [20.0, 20.0, 20.0]),
    ];
    assert!(parts_of(&bad).len() == 1);
    assert!(contacts(&bad).is_empty());
    assert!(shelves(&bad, f64::NAN).is_empty());
    assert!(shelves(&bad, -0.0).is_empty());
    assert!(shelves(&bad, 0.0).is_empty());
    assert!(size(&bad[1]).is_none());
    assert!(!touching(&bad[1], &bad[2]));
    let got = design(&bad, Some(f64::INFINITY), f64::NAN, Vec::new(), Vec::new());
    assert!(got.parts.len() == 1);
    assert!(got.shelves.is_empty());
    assert!(got.contacts.is_empty());
}

#[test]
fn never_panics_on_huge_and_tiny_numbers() {
    let parts = vec![
        wood("big", [0.0, 0.0, 0.0], [1.0e300, 1.0e300, 1.0e300]),
        wood("small", [0.0, 0.0, 0.0], [1.0e-300, 1.0e-300, 1.0e-300]),
    ];
    let got = design(&parts, Some(0.0), f64::MAX, Vec::new(), Vec::new());
    assert!(got.parts.len() == 3);
    assert!(touching(&parts[0], &parts[1]));
}
