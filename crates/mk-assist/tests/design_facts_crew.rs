//! Tests for mk_assist::design_facts written by Deepak (the Crew, campaign mk-core-34) from the
//! contract alone, without seeing the module.

//! Integration tests for `design_facts` (contract A1-A10 and the worked-example rows).

// Crew-written tests: a helper that builds a whole `Finding` takes one argument per
// field, and worked rows print constants to the contract's digits; neither matters in a
// test.
#![allow(clippy::too_many_arguments, clippy::approx_constant)]

use mk_assist::design_facts::{
    contacts, design, parts_of, shelves, size, thickness_mm, touching, BoxPart,
    CONTACT_TOLERANCE_MM, FLOOR_ID, MIN_SHELF_SPAN_MM,
};
use mk_assist::rules_structure::{Contact, Fastener, FastenerKind, Part, Shelf, Slot};

const PINE: &str = "wood.softwood.pine";

fn bp(id: &str, min: [f64; 3], max: [f64; 3]) -> BoxPart {
    BoxPart {
        id: id.to_string(),
        material: PINE.to_string(),
        min,
        max,
    }
}

fn mp(id: &str, t: f64) -> Part {
    Part {
        id: id.to_string(),
        material: PINE.to_string(),
        thickness_mm: t,
    }
}

fn ct(a: &str, b: &str) -> Contact {
    Contact {
        a: a.to_string(),
        b: b.to_string(),
    }
}

/// Row 9's board and legs; `span` is the board's x length.
fn board_and_legs(span: f64) -> Vec<BoxPart> {
    vec![
        bp("board", [0.0, 0.0, 700.0], [span, 300.0, 718.0]),
        bp("leg_left", [0.0, 0.0, 0.0], [18.0, 300.0, 700.0]),
        bp("leg_right", [span - 18.0, 0.0, 0.0], [span, 300.0, 700.0]),
    ]
}

// ---- A1 ----

#[test]
fn a1_empty_id_is_unusable() {
    // Row 1.
    let p = bp("", [0.0, 0.0, 0.0], [10.0, 20.0, 30.0]);
    assert_eq!(size(&p), None);
    assert_eq!(thickness_mm(&p), None);
    assert_eq!(parts_of(&[p]), vec![]);
}

#[test]
fn a1_flat_box_is_unusable() {
    // Row 2: max > min fails on z.
    let p = bp("flat", [0.0, 0.0, 0.0], [10.0, 10.0, 0.0]);
    assert_eq!(size(&p), None);
    assert_eq!(thickness_mm(&p), None);
    assert_eq!(parts_of(&[p]), vec![]);
}

#[test]
fn a1_non_finite_or_reversed_box_is_unusable() {
    assert_eq!(
        size(&bp("n", [0.0, 0.0, 0.0], [f64::NAN, 10.0, 10.0])),
        None
    );
    assert_eq!(
        size(&bp("i", [0.0, 0.0, 0.0], [f64::INFINITY, 10.0, 10.0])),
        None
    );
    assert_eq!(
        size(&bp("m", [0.0, f64::NAN, 0.0], [10.0, 10.0, 10.0])),
        None
    );
    assert_eq!(size(&bp("r", [0.0, 0.0, 0.0], [-10.0, 10.0, 10.0])), None);
}

#[test]
fn a1_duplicate_id_keeps_the_earlier_part() {
    // Row 3.
    let first = bp("a", [0.0, 0.0, 0.0], [10.0, 10.0, 10.0]);
    let second = bp("a", [100.0, 100.0, 100.0], [200.0, 200.0, 200.0]);
    assert_eq!(parts_of(&[first, second]), vec![mp("a", 10.0)]);
}

#[test]
fn a1_empty_input_is_silent() {
    assert_eq!(parts_of(&[]), vec![]);
    assert_eq!(contacts(&[]), vec![]);
    assert_eq!(shelves(&[], 5.0), vec![]);
    let got = design(&[], None, 5.0, vec![], vec![]);
    assert_eq!(got.parts, vec![]);
    assert_eq!(got.contacts, vec![]);
    assert_eq!(got.shelves, vec![]);
}

#[test]
fn a1_non_finite_inputs_never_panic() {
    let parts = vec![
        bp("nan", [f64::NAN, 0.0, 0.0], [10.0, 10.0, 10.0]),
        bp("inf", [0.0, 0.0, 0.0], [f64::INFINITY, 10.0, 10.0]),
        bp("ok", [0.0, 0.0, 0.0], [10.0, 10.0, 10.0]),
    ];
    assert_eq!(parts_of(&parts), vec![mp("ok", 10.0)]);
    assert_eq!(contacts(&parts), vec![]);
    assert_eq!(shelves(&parts, f64::NAN), vec![]);
    let got = design(&parts, Some(f64::NAN), f64::INFINITY, vec![], vec![]);
    assert_eq!(got.parts, vec![mp("ok", 10.0)]);
    assert_eq!(got.contacts, vec![]);
}

// ---- A2 ----

#[test]
fn a2_size_and_thickness_are_the_side_lengths() {
    // Row 4.
    let p = bp("a", [0.0, 0.0, 0.0], [10.0, 20.0, 30.0]);
    assert_eq!(size(&p), Some([10.0, 20.0, 30.0]));
    assert_eq!(thickness_mm(&p), Some(10.0));
}

#[test]
fn a2_cube_thickness_is_its_side() {
    // Row 5.
    let p = bp("cube", [0.0, 0.0, 0.0], [5.0, 5.0, 5.0]);
    assert_eq!(size(&p), Some([5.0, 5.0, 5.0]));
    assert_eq!(thickness_mm(&p), Some(5.0));
}

#[test]
fn a2_thickness_is_exact_with_no_rounding() {
    let p = bp("odd", [0.0, 0.0, 0.0], [10.0, 20.0, 3.000_000_1]);
    assert_eq!(thickness_mm(&p), Some(3.000_000_1));
}

// ---- A3 ----

#[test]
fn a3_constants_are_the_contract_values() {
    assert_eq!(CONTACT_TOLERANCE_MM, 1.0);
    assert_eq!(MIN_SHELF_SPAN_MM, 300.0);
    assert_eq!(FLOOR_ID, "floor");
}

// ---- A4 ----

#[test]
fn a4_touching_uses_the_tolerance_gap() {
    // Row 6: gap 0 touches, gap 1.0 touches, gap 1.1 does not.
    let a = bp("a", [0.0, 0.0, 0.0], [10.0, 10.0, 10.0]);
    let b = bp("b", [10.0, 0.0, 0.0], [20.0, 10.0, 10.0]);
    let c = bp("c", [11.0, 0.0, 0.0], [20.0, 10.0, 10.0]);
    let d = bp("d", [11.1, 0.0, 0.0], [20.0, 10.0, 10.0]);
    assert!(touching(&a, &b));
    assert!(touching(&a, &c));
    assert!(!touching(&a, &d));
    assert!(touching(&b, &a));
    assert!(!touching(&d, &a));
}

#[test]
fn a4_a_part_does_not_touch_itself() {
    // Row 7, first half.
    let a = bp("a", [0.0, 0.0, 0.0], [10.0, 10.0, 10.0]);
    assert!(!touching(&a, &a));
}

#[test]
fn a4_unusable_part_touches_nothing() {
    // Row 7, second half.
    let a = bp("a", [0.0, 0.0, 0.0], [10.0, 10.0, 10.0]);
    let broken = bp("broken", [10.0, 0.0, 0.0], [f64::NAN, 10.0, 10.0]);
    assert!(!touching(&a, &broken));
    assert!(!touching(&broken, &a));
}

// ---- A5 ----

#[test]
fn a5_parts_of_copies_id_material_and_thickness() {
    let a = bp("a", [0.0, 0.0, 0.0], [10.0, 20.0, 30.0]);
    let b = bp("b", [100.0, 0.0, 0.0], [110.0, 40.0, 50.0]);
    assert_eq!(parts_of(&[a, b]), vec![mp("a", 10.0), mp("b", 10.0)]);
}

// ---- A6 ----

#[test]
fn a6_contacts_lists_each_touching_pair_once_in_order() {
    // Row 8.
    let a = bp("a", [0.0, 0.0, 0.0], [10.0, 10.0, 10.0]);
    let b = bp("b", [10.0, 0.0, 0.0], [20.0, 10.0, 10.0]);
    let c = bp("c", [20.0, 0.0, 0.0], [30.0, 10.0, 10.0]);
    assert_eq!(contacts(&[a, b, c]), vec![ct("a", "b"), ct("b", "c")]);
}

#[test]
fn a6_contacts_ordered_by_a_index_then_b_index() {
    let a = bp("a", [0.0, 0.0, 0.0], [10.0, 10.0, 10.0]);
    let b = bp("b", [10.0, 0.0, 0.0], [20.0, 10.0, 10.0]);
    let c = bp("c", [0.0, 10.0, 0.0], [10.0, 20.0, 10.0]);
    assert_eq!(
        contacts(&[a, b, c]),
        vec![ct("a", "b"), ct("a", "c"), ct("b", "c")]
    );
}

#[test]
fn a6_unusable_part_contacts_nothing() {
    let a = bp("a", [0.0, 0.0, 0.0], [10.0, 10.0, 10.0]);
    let broken = bp("broken", [10.0, 0.0, 0.0], [20.0, 10.0, f64::NAN]);
    let c = bp("c", [20.0, 0.0, 0.0], [30.0, 10.0, 10.0]);
    assert_eq!(contacts(&[a, broken, c]), vec![]);
}

// ---- A7 / A8 ----

#[test]
fn a7_and_a8_a_board_on_two_legs_is_one_shelf() {
    // Row 9: span axis x, leg centres 9 and 391 sit in the quarters.
    assert_eq!(
        shelves(&board_and_legs(400.0), 5.0),
        vec![Shelf {
            part: "board".to_string(),
            span_mm: 400.0,
            depth_mm: 300.0,
            load_kg_per_m: 5.0,
        }]
    );
}

#[test]
fn a7_one_end_held_is_not_a_shelf() {
    // Row 10.
    let parts = vec![
        bp("board", [0.0, 0.0, 700.0], [400.0, 300.0, 718.0]),
        bp("leg_left", [0.0, 0.0, 0.0], [18.0, 300.0, 700.0]),
    ];
    assert_eq!(shelves(&parts, 5.0), vec![]);
}

#[test]
fn a7_a_support_in_the_middle_holds_no_end() {
    // Row 11: the back's centre x = 200 is in neither quarter of 0..400.
    let parts = vec![
        bp("board", [0.0, 0.0, 0.0], [400.0, 300.0, 18.0]),
        bp("back", [0.0, 0.0, -500.0], [400.0, 18.0, 0.0]),
    ];
    assert_eq!(shelves(&parts, 5.0), vec![]);
}

#[test]
fn a7_load_gate_rejects_negative_and_non_finite() {
    // Row 12.
    let parts = board_and_legs(400.0);
    assert_eq!(shelves(&parts, -1.0), vec![]);
    assert_eq!(shelves(&parts, f64::NAN), vec![]);
    assert_eq!(shelves(&parts, f64::INFINITY), vec![]);
    assert_eq!(shelves(&parts, f64::NEG_INFINITY), vec![]);
    let zero = vec![Shelf {
        part: "board".to_string(),
        span_mm: 400.0,
        depth_mm: 300.0,
        load_kg_per_m: 0.0,
    }];
    assert_eq!(shelves(&parts, 0.0), zero);
    assert_eq!(shelves(&parts, -0.0), zero);
}

#[test]
fn a7_span_must_reach_the_minimum() {
    // Row 13: 299 is short, 300 is exactly enough.
    assert_eq!(shelves(&board_and_legs(299.0), 5.0), vec![]);
    assert_eq!(
        shelves(&board_and_legs(300.0), 5.0),
        vec![Shelf {
            part: "board".to_string(),
            span_mm: 300.0,
            depth_mm: 300.0,
            load_kg_per_m: 5.0,
        }]
    );
}

#[test]
fn a7_smallest_side_must_be_vertical() {
    // Row 14: the plank's smallest side is y = 20, not z.
    let parts = vec![
        bp("plank", [0.0, 0.0, 0.0], [600.0, 20.0, 400.0]),
        bp("leg_left", [0.0, 0.0, 0.0], [18.0, 600.0, 20.0]),
        bp("leg_right", [582.0, 0.0, 0.0], [600.0, 600.0, 20.0]),
    ];
    assert_eq!(shelves(&parts, 5.0), vec![]);
}

#[test]
fn a7_equal_horizontal_sides_span_along_x() {
    // Row 15.
    let parts = vec![
        bp("board", [0.0, 0.0, 400.0], [400.0, 400.0, 418.0]),
        bp("leg_left", [0.0, 0.0, 0.0], [18.0, 400.0, 400.0]),
        bp("leg_right", [382.0, 0.0, 0.0], [400.0, 400.0, 400.0]),
    ];
    assert_eq!(
        shelves(&parts, 5.0),
        vec![Shelf {
            part: "board".to_string(),
            span_mm: 400.0,
            depth_mm: 400.0,
            load_kg_per_m: 5.0,
        }]
    );
}

#[test]
fn a7_a_cube_is_never_a_shelf() {
    let parts = vec![
        bp("cube", [0.0, 0.0, 0.0], [400.0, 400.0, 400.0]),
        bp("leg_left", [0.0, 0.0, -400.0], [18.0, 400.0, 0.0]),
        bp("leg_right", [382.0, 0.0, -400.0], [400.0, 400.0, 0.0]),
    ];
    assert_eq!(shelves(&parts, 5.0), vec![]);
}

#[test]
fn a7_vertical_side_must_be_strictly_the_smallest() {
    // z = 18 ties with y = 18, so z is not strictly the smallest side.
    let parts = vec![
        bp("board", [0.0, 0.0, 700.0], [400.0, 18.0, 718.0]),
        bp("leg_left", [0.0, 0.0, 0.0], [18.0, 18.0, 700.0]),
        bp("leg_right", [382.0, 0.0, 0.0], [400.0, 18.0, 700.0]),
    ];
    assert_eq!(shelves(&parts, 5.0), vec![]);
}

#[test]
fn a8_load_is_copied_unchanged() {
    // Row 16.
    assert_eq!(
        shelves(&board_and_legs(400.0), 12.5),
        vec![Shelf {
            part: "board".to_string(),
            span_mm: 400.0,
            depth_mm: 300.0,
            load_kg_per_m: 12.5,
        }]
    );
}

#[test]
fn a8_shelves_appear_in_input_order() {
    let parts = vec![
        bp("low", [0.0, 0.0, 300.0], [400.0, 300.0, 318.0]),
        bp("low_left", [0.0, 0.0, 0.0], [18.0, 300.0, 300.0]),
        bp("low_right", [382.0, 0.0, 0.0], [400.0, 300.0, 300.0]),
        bp("high", [0.0, 0.0, 700.0], [400.0, 300.0, 718.0]),
        bp("high_left", [0.0, 0.0, 318.0], [18.0, 300.0, 700.0]),
        bp("high_right", [382.0, 0.0, 318.0], [400.0, 300.0, 700.0]),
    ];
    assert_eq!(
        shelves(&parts, 5.0),
        vec![
            Shelf {
                part: "low".to_string(),
                span_mm: 400.0,
                depth_mm: 300.0,
                load_kg_per_m: 5.0,
            },
            Shelf {
                part: "high".to_string(),
                span_mm: 400.0,
                depth_mm: 300.0,
                load_kg_per_m: 5.0,
            },
        ]
    );
}

// ---- A9 ----

#[test]
fn a9_design_gathers_facts_and_passes_fasteners_and_slots_through() {
    // Row 21.
    let parts = board_and_legs(400.0);
    let fastener = Fastener {
        id: "s1".to_string(),
        kind: FastenerKind::Screw,
        length_mm: 50.0,
        diameter_mm: 4.0,
        joins: vec!["leg_left".to_string(), "board".to_string()],
        edge_of: "board".to_string(),
        edge_distance_mm: 20.0,
    };
    let slot = Slot {
        part: "board".to_string(),
        width_mm: 6.0,
        remaining_mm: 12.0,
    };
    let got = design(
        &parts,
        None,
        5.0,
        vec![fastener.clone()],
        vec![slot.clone()],
    );
    assert_eq!(got.parts, parts_of(&parts));
    assert_eq!(got.contacts, contacts(&parts));
    assert_eq!(got.shelves, shelves(&parts, 5.0));
    assert_eq!(got.fasteners, vec![fastener]);
    assert_eq!(got.slots, vec![slot]);
}

#[test]
fn a9_unknown_fastener_and_slot_ids_pass_through_unchanged() {
    let fastener = Fastener {
        id: "ghost".to_string(),
        kind: FastenerKind::Dowel,
        length_mm: 30.0,
        diameter_mm: 8.0,
        joins: vec!["nowhere".to_string()],
        edge_of: "nowhere".to_string(),
        edge_distance_mm: 5.0,
    };
    let slot = Slot {
        part: "nowhere".to_string(),
        width_mm: 3.0,
        remaining_mm: 4.0,
    };
    let got = design(&[], None, 5.0, vec![fastener.clone()], vec![slot.clone()]);
    assert_eq!(got.fasteners, vec![fastener]);
    assert_eq!(got.slots, vec![slot]);
}

// ---- A10 ----

#[test]
fn a10_floor_contact_within_the_tolerance_only() {
    // Row 17: z-mins -0.5, -1.0, -1.1, +1.0, +1.1 against floor 0.0.
    let zs = [-0.5, -1.0, -1.1, 1.0, 1.1];
    let mut parts = Vec::new();
    for (i, z) in zs.iter().enumerate() {
        let x = i as f64 * 200.0;
        parts.push(bp(
            &format!("p{}", i),
            [x, 0.0, *z],
            [x + 100.0, 100.0, *z + 18.0],
        ));
    }
    let got = design(&parts, Some(0.0), 5.0, vec![], vec![]);
    assert_eq!(
        got.contacts,
        vec![ct("p0", FLOOR_ID), ct("p1", FLOOR_ID), ct("p3", FLOOR_ID)]
    );
}

#[test]
fn a10_floor_part_and_contact_are_added() {
    // Row 18.
    let parts = vec![bp("a", [0.0, 0.0, 0.0], [100.0, 100.0, 18.0])];
    let got = design(&parts, Some(0.0), 5.0, vec![], vec![]);
    assert_eq!(got.parts.len(), 2);
    assert_eq!(got.parts[0], mp("a", 18.0));
    assert_eq!(got.parts[1].id, FLOOR_ID);
    assert_eq!(got.parts[1].material, "floor");
    assert!(got.parts[1].thickness_mm.is_nan());
    assert_eq!(got.contacts, vec![ct("a", FLOOR_ID)]);
}

#[test]
fn a10_no_floor_when_z_is_none_or_not_finite() {
    // Row 19.
    let parts = vec![bp("a", [0.0, 0.0, 0.0], [100.0, 100.0, 18.0])];
    for z in [
        None,
        Some(f64::NAN),
        Some(f64::INFINITY),
        Some(f64::NEG_INFINITY),
    ] {
        let got = design(&parts, z, 5.0, vec![], vec![]);
        assert_eq!(got.parts, vec![mp("a", 18.0)]);
        assert_eq!(got.contacts, vec![]);
    }
}

#[test]
fn a10_a_measured_part_named_floor_is_ignored() {
    // Row 20.
    let parts = vec![
        bp(FLOOR_ID, [0.0, 0.0, 0.0], [100.0, 100.0, 18.0]),
        bp("a", [0.0, 0.0, 0.0], [10.0, 10.0, 10.0]),
    ];
    assert_eq!(parts_of(&parts), vec![mp("a", 10.0)]);
    let got = design(&parts, Some(0.0), 5.0, vec![], vec![]);
    assert_eq!(got.contacts, vec![ct("a", FLOOR_ID)]);
}

#[test]
fn a10_floor_contacts_come_after_part_contacts() {
    let parts = vec![
        bp("a", [0.0, 0.0, 0.0], [10.0, 10.0, 10.0]),
        bp("b", [10.0, 0.0, 0.0], [20.0, 10.0, 10.0]),
    ];
    let got = design(&parts, Some(0.0), 5.0, vec![], vec![]);
    assert_eq!(
        got.contacts,
        vec![ct("a", "b"), ct("a", FLOOR_ID), ct("b", FLOOR_ID)]
    );
}
