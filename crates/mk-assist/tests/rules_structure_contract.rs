//! Contract tests for mk_assist::rules_structure, written by Gordon as a PA (the Crew) from
//! crew/contracts/mk_rules_structure.md. They were the hidden tests of campaign mk-core-21.

//! Hidden tests for Sancho's structure rule pack (doc 07 section 7.5).
//!
//! The numbers are the contract's worked examples, which are also the app's example numbers:
//! sag_per_m_mm 1.67, source "test table", three materials. Nothing here touches files,
//! randomness or time.

use mk_assist::findings::{Finding, Kind, Location, Severity, Source};
use mk_assist::rules_structure::{
    evaluate, sag_mm, Contact, Design, Fastener, FastenerKind, MaterialData, Part, Shelf, Slot,
    Tables, G, RULES, RULE_EDGE_DISTANCE, RULE_SAG, RULE_SCREW_SHORT, RULE_SCREW_THROUGH,
    RULE_SLOT_SEVERS, RULE_THIN_WALL, RULE_UNSUPPORTED,
};

fn material(
    id: &str,
    stiffness_mpa: f64,
    min_wall_mm: f64,
    min_embed_mm: f64,
    screw_edge_diameters: f64,
    dowel_edge_diameters: f64,
) -> MaterialData {
    MaterialData {
        material: id.to_string(),
        stiffness_mpa,
        min_wall_mm,
        min_embed_mm,
        screw_edge_diameters,
        dowel_edge_diameters,
    }
}

fn tables() -> Tables {
    Tables {
        materials: vec![
            material("wood.softwood.pine", 8000.0, 6.0, 15.0, 3.0, 2.0),
            material("wood.sheet.plywood", 6000.0, 3.0, 12.0, 2.5, 2.0),
            material("wood.sheet.particleboard", 2500.0, 10.0, 20.0, 5.0, 3.0),
        ],
        sag_per_m_mm: 1.67,
        source: "test table".to_string(),
    }
}

fn part(id: &str, material: &str, thickness_mm: f64) -> Part {
    Part {
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

fn shelf(part: &str, span_mm: f64, depth_mm: f64, load_kg_per_m: f64) -> Shelf {
    Shelf {
        part: part.to_string(),
        span_mm,
        depth_mm,
        load_kg_per_m,
    }
}

fn screw(
    id: &str,
    length_mm: f64,
    diameter_mm: f64,
    joins: &[&str],
    edge_of: &str,
    edge_distance_mm: f64,
) -> Fastener {
    Fastener {
        id: id.to_string(),
        kind: FastenerKind::Screw,
        length_mm,
        diameter_mm,
        joins: joins.iter().map(|id| id.to_string()).collect(),
        edge_of: edge_of.to_string(),
        edge_distance_mm,
    }
}

fn dowel(
    id: &str,
    length_mm: f64,
    diameter_mm: f64,
    joins: &[&str],
    edge_of: &str,
    edge_distance_mm: f64,
) -> Fastener {
    Fastener {
        kind: FastenerKind::Dowel,
        ..screw(id, length_mm, diameter_mm, joins, edge_of, edge_distance_mm)
    }
}

/// The contract's base design B: four parts, two sides holding the top and the shelf.
fn base_design() -> Design {
    Design {
        parts: vec![
            part("side_l", "wood.softwood.pine", 18.0),
            part("side_r", "wood.softwood.pine", 18.0),
            part("shelf_a", "wood.softwood.pine", 18.0),
            part("top", "wood.sheet.particleboard", 16.0),
        ],
        shelves: Vec::new(),
        fasteners: Vec::new(),
        contacts: vec![
            contact("side_l", "top"),
            contact("side_r", "top"),
            contact("side_l", "shelf_a"),
            contact("side_r", "shelf_a"),
        ],
        slots: Vec::new(),
    }
}

fn rule_of(finding: &Finding) -> &str {
    match &finding.source {
        Source::Rule(rule) => rule,
        Source::Mind => "mind",
    }
}

// ---------------------------------------------------------------------------
// B1: sag_mm

#[test]
fn b1_sag_matches_the_beam_formula() {
    let sag = sag_mm(800.0, 250.0, 18.0, 8000.0, 30.0).expect("usable numbers");
    let load_n_per_mm = 30.0 * G / 1000.0;
    let second_moment = 250.0 * 18.0_f64.powi(3) / 12.0;
    let expected = 5.0 * load_n_per_mm * 800.0_f64.powi(4) / (384.0 * 8000.0 * second_moment);
    assert!((sag - expected).abs() < 1e-9);
    assert!(sag > 1.6 && sag < 1.7);
}

#[test]
fn b1_sag_is_zero_when_the_load_is_zero() {
    let sag = sag_mm(800.0, 250.0, 18.0, 8000.0, 0.0).expect("a zero load is usable");
    assert!(sag == 0.0);
}

#[test]
fn b1_sag_halves_with_a_stiffer_material() {
    let soft = sag_mm(800.0, 250.0, 18.0, 4000.0, 30.0).expect("usable numbers");
    let stiff = sag_mm(800.0, 250.0, 18.0, 8000.0, 30.0).expect("usable numbers");
    assert!((soft - 2.0 * stiff).abs() < 1e-9);
}

#[test]
fn b1_sag_is_none_for_unusable_inputs() {
    assert!(sag_mm(0.0, 250.0, 18.0, 8000.0, 30.0).is_none());
    assert!(sag_mm(-800.0, 250.0, 18.0, 8000.0, 30.0).is_none());
    assert!(sag_mm(800.0, f64::NAN, 18.0, 8000.0, 30.0).is_none());
    assert!(sag_mm(800.0, 250.0, f64::INFINITY, 8000.0, 30.0).is_none());
    assert!(sag_mm(800.0, 250.0, 18.0, 0.0, 30.0).is_none());
    assert!(sag_mm(800.0, 250.0, 18.0, 8000.0, -30.0).is_none());
    assert!(sag_mm(800.0, 250.0, 18.0, 8000.0, f64::NAN).is_none());
    assert!(sag_mm(800.0, 250.0, 18.0, 8000.0, f64::INFINITY).is_none());
}

// ---------------------------------------------------------------------------
// S1: structure.sag

#[test]
fn s1_sag_reports_the_worked_example() {
    let mut design = base_design();
    design.shelves.push(shelf("shelf_a", 800.0, 250.0, 30.0));
    let findings = evaluate(&design, &tables());
    assert_eq!(
        findings,
        vec![Finding {
            kind: Kind::NotSuitable,
            severity: Severity::Warning,
            source: Source::Rule(RULE_SAG.to_string()),
            location: Location::Entity("shelf_a".to_string()),
            what: "shelf_a sags about 1.6 mm".to_string(),
            why: "Over 800 mm, 18 mm of wood.softwood.pine carrying 30 kg per metre sags about 1.6 mm; more than 1.3 mm shows.".to_string(),
            fix: Some("part.thickness".to_string()),
            facts: vec![
                ("part".to_string(), "shelf_a".to_string()),
                ("span_mm".to_string(), "800".to_string()),
                ("thickness_mm".to_string(), "18".to_string()),
                ("material".to_string(), "wood.softwood.pine".to_string()),
                ("load_kg_per_m".to_string(), "30".to_string()),
                ("sag_mm".to_string(), "1.6".to_string()),
                ("allowed_mm".to_string(), "1.3".to_string()),
                ("source".to_string(), "test table".to_string()),
            ],
        }]
    );
}

#[test]
fn s1_sag_is_quiet_when_the_span_is_short() {
    let mut design = base_design();
    design.shelves.push(shelf("shelf_a", 600.0, 250.0, 30.0));
    assert!(evaluate(&design, &tables()).is_empty());
}

#[test]
fn s1_sag_is_quiet_with_no_load() {
    let mut design = base_design();
    design.shelves.push(shelf("shelf_a", 800.0, 250.0, 0.0));
    assert!(evaluate(&design, &tables()).is_empty());
}

#[test]
fn s1_sag_is_quiet_for_an_unknown_part() {
    let mut design = base_design();
    design.shelves.push(shelf("ghost", 800.0, 250.0, 30.0));
    assert!(evaluate(&design, &tables()).is_empty());
}

#[test]
fn s1_sag_is_quiet_when_the_material_is_unknown() {
    let mut design = base_design();
    design.parts.push(part("mystery", "wood.unknown", 18.0));
    design.contacts.push(contact("side_l", "mystery"));
    design.shelves.push(shelf("mystery", 800.0, 250.0, 30.0));
    assert!(evaluate(&design, &tables()).is_empty());
}

#[test]
fn s1_sag_is_quiet_when_the_allowed_sag_is_unusable() {
    let mut design = base_design();
    design.shelves.push(shelf("shelf_a", 800.0, 250.0, 30.0));
    let mut tables = tables();
    tables.sag_per_m_mm = 0.0;
    assert!(evaluate(&design, &tables).is_empty());
}

// ---------------------------------------------------------------------------
// S2: structure.screw_through

#[test]
fn s2_screw_through_reports_the_worked_example() {
    let mut design = base_design();
    design.fasteners.push(screw(
        "f1",
        40.0,
        4.0,
        &["side_l", "shelf_a"],
        "shelf_a",
        20.0,
    ));
    let findings = evaluate(&design, &tables());
    assert_eq!(
        findings,
        vec![Finding {
            kind: Kind::Forbidden,
            severity: Severity::Critical,
            source: Source::Rule(RULE_SCREW_THROUGH.to_string()),
            location: Location::Entity("f1".to_string()),
            what: "f1 pokes out".to_string(),
            why: "f1 is 40 mm long and the parts it goes through are 36 mm together, so its tip comes out the far side.".to_string(),
            fix: Some("fastener.length".to_string()),
            facts: vec![
                ("fastener".to_string(), "f1".to_string()),
                ("length_mm".to_string(), "40".to_string()),
                ("stock_mm".to_string(), "36".to_string()),
            ],
        }]
    );
}

#[test]
fn s2_screw_at_exact_stock_is_quiet() {
    let mut design = base_design();
    design.fasteners.push(screw(
        "f",
        36.0,
        4.0,
        &["side_l", "shelf_a"],
        "shelf_a",
        20.0,
    ));
    assert!(evaluate(&design, &tables()).is_empty());
}

#[test]
fn s2_screw_through_is_quiet_when_a_joined_part_is_unknown() {
    let mut design = base_design();
    design
        .fasteners
        .push(screw("f", 90.0, 4.0, &["side_l", "ghost"], "shelf_a", 20.0));
    assert!(evaluate(&design, &tables()).is_empty());
}

#[test]
fn s2_dowel_never_pokes_out() {
    let mut design = base_design();
    design.fasteners.push(dowel(
        "d1",
        400.0,
        8.0,
        &["side_l", "shelf_a"],
        "shelf_a",
        20.0,
    ));
    let findings = evaluate(&design, &tables());
    assert!(findings
        .iter()
        .all(|finding| rule_of(finding) != RULE_SCREW_THROUGH));
    assert!(findings
        .iter()
        .all(|finding| rule_of(finding) != RULE_SCREW_SHORT));
}

// ---------------------------------------------------------------------------
// S3: structure.screw_short

#[test]
fn s3_screw_short_reports_the_worked_example() {
    let mut design = base_design();
    design.fasteners.push(screw(
        "f2",
        30.0,
        4.0,
        &["side_r", "shelf_a"],
        "shelf_a",
        20.0,
    ));
    let findings = evaluate(&design, &tables());
    assert_eq!(
        findings,
        vec![Finding {
            kind: Kind::NotSuitable,
            severity: Severity::Warning,
            source: Source::Rule(RULE_SCREW_SHORT.to_string()),
            location: Location::Entity("f2".to_string()),
            what: "f2 barely bites".to_string(),
            why: "f2 goes 12 mm into shelf_a; wood.softwood.pine needs at least 15 mm to hold."
                .to_string(),
            fix: Some("fastener.length".to_string()),
            facts: vec![
                ("fastener".to_string(), "f2".to_string()),
                ("embed_mm".to_string(), "12".to_string()),
                ("min_embed_mm".to_string(), "15".to_string()),
            ],
        }]
    );
}

#[test]
fn s3_embed_floors_at_zero() {
    let mut design = base_design();
    design.fasteners.push(screw(
        "shy",
        10.0,
        4.0,
        &["side_l", "shelf_a"],
        "shelf_a",
        20.0,
    ));
    let findings = evaluate(&design, &tables());
    assert_eq!(findings.len(), 1);
    assert!(!findings[0].why.is_empty());
    assert!(findings[0].why.contains("goes 0 mm into shelf_a"));
    assert!(findings[0]
        .facts
        .contains(&("embed_mm".to_string(), "0".to_string())));
}

#[test]
fn s3_screw_long_enough_is_quiet() {
    let mut design = base_design();
    design.fasteners.push(screw(
        "f",
        34.0,
        4.0,
        &["side_r", "shelf_a"],
        "shelf_a",
        20.0,
    ));
    assert!(evaluate(&design, &tables()).is_empty());
}

#[test]
fn s3_one_joined_part_is_never_short() {
    let mut design = base_design();
    design
        .fasteners
        .push(screw("f", 2.0, 4.0, &["shelf_a"], "shelf_a", 20.0));
    assert!(evaluate(&design, &tables()).is_empty());
}

#[test]
fn s3_is_quiet_when_the_last_material_is_unknown() {
    let mut design = base_design();
    design.parts.push(part("mystery", "wood.unknown", 18.0));
    design.contacts.push(contact("side_l", "mystery"));
    design.fasteners.push(screw(
        "f",
        20.0,
        4.0,
        &["side_l", "mystery"],
        "mystery",
        20.0,
    ));
    assert!(evaluate(&design, &tables()).is_empty());
}

#[test]
fn s2_and_s3_are_never_both_reported() {
    let mut design = base_design();
    // 35 > 34 (S2) and it goes only 17 mm into particleboard, which wants 20 (S3).
    design
        .fasteners
        .push(screw("f", 35.0, 4.0, &["side_l", "top"], "top", 20.0));
    let findings = evaluate(&design, &tables());
    assert_eq!(findings.len(), 1);
    assert!(rule_of(&findings[0]) == RULE_SCREW_THROUGH);
}

// ---------------------------------------------------------------------------
// S4: structure.edge_distance

#[test]
fn s4_edge_distance_reports_the_worked_example() {
    let mut design = base_design();
    design.fasteners.push(screw(
        "f3",
        35.0,
        4.0,
        &["side_r", "shelf_a"],
        "side_r",
        8.0,
    ));
    let findings = evaluate(&design, &tables());
    assert_eq!(
        findings,
        vec![Finding {
            kind: Kind::NotSuitable,
            severity: Severity::Warning,
            source: Source::Rule(RULE_EDGE_DISTANCE.to_string()),
            location: Location::Entity("f3".to_string()),
            what: "f3 is too close to the edge".to_string(),
            why: "f3 sits 8 mm from the edge of side_r; wood.softwood.pine needs 12 mm (3 times its 4 mm diameter) or it splits.".to_string(),
            fix: Some("fastener.move".to_string()),
            facts: vec![
                ("fastener".to_string(), "f3".to_string()),
                ("edge_distance_mm".to_string(), "8".to_string()),
                ("need_mm".to_string(), "12".to_string()),
            ],
        }]
    );
}

#[test]
fn s4_dowel_uses_the_dowel_diameters() {
    let mut design = base_design();
    design.fasteners.push(dowel(
        "d1",
        40.0,
        8.0,
        &["side_l", "shelf_a"],
        "shelf_a",
        15.0,
    ));
    let findings = evaluate(&design, &tables());
    assert_eq!(findings.len(), 1);
    assert!(rule_of(&findings[0]) == RULE_EDGE_DISTANCE);
    assert!(findings[0]
        .facts
        .contains(&("need_mm".to_string(), "16".to_string())));
    assert!(findings[0]
        .why
        .contains("needs 16 mm (2 times its 8 mm diameter)"));
}

#[test]
fn s4_screw_at_exact_need_is_quiet() {
    let mut design = base_design();
    design.fasteners.push(screw(
        "f3",
        35.0,
        4.0,
        &["side_r", "shelf_a"],
        "side_r",
        12.0,
    ));
    assert!(evaluate(&design, &tables()).is_empty());
}

#[test]
fn s4_particleboard_needs_five_diameters() {
    let mut design = base_design();
    design
        .fasteners
        .push(screw("f", 35.0, 4.0, &["side_r", "shelf_a"], "top", 19.0));
    let findings = evaluate(&design, &tables());
    assert_eq!(findings.len(), 1);
    assert!(findings[0]
        .facts
        .contains(&("need_mm".to_string(), "20".to_string())));
}

#[test]
fn s4_is_quiet_for_an_unknown_edge_part() {
    let mut design = base_design();
    design
        .fasteners
        .push(screw("f", 36.0, 4.0, &["side_l", "shelf_a"], "ghost", 1.0));
    assert!(evaluate(&design, &tables()).is_empty());
}

#[test]
fn s4_is_quiet_for_unusable_numbers() {
    let mut zero = base_design();
    zero.fasteners.push(screw(
        "f",
        36.0,
        0.0,
        &["side_l", "shelf_a"],
        "shelf_a",
        0.0,
    ));
    assert!(evaluate(&zero, &tables()).is_empty());

    let mut nan = base_design();
    nan.fasteners.push(screw(
        "f",
        36.0,
        4.0,
        &["side_l", "shelf_a"],
        "shelf_a",
        f64::NAN,
    ));
    assert!(evaluate(&nan, &tables()).is_empty());
}

// ---------------------------------------------------------------------------
// S5: structure.unsupported

#[test]
fn s5_unsupported_reports_the_worked_example() {
    let mut design = base_design();
    design.parts.push(part("knob", "wood.softwood.pine", 30.0));
    let findings = evaluate(&design, &tables());
    assert_eq!(
        findings,
        vec![Finding {
            kind: Kind::Forbidden,
            severity: Severity::Warning,
            source: Source::Rule(RULE_UNSUPPORTED.to_string()),
            location: Location::Entity("knob".to_string()),
            what: "knob is held by nothing".to_string(),
            why: "knob touches no other part and nothing fastens it, so it would fall.".to_string(),
            fix: Some("part.attach".to_string()),
            facts: vec![("part".to_string(), "knob".to_string())],
        }]
    );
}

#[test]
fn s5_a_lone_part_design_is_quiet() {
    let design = Design {
        parts: vec![part("knob", "wood.softwood.pine", 30.0)],
        ..Design::default()
    };
    assert!(evaluate(&design, &tables()).is_empty());
}

#[test]
fn s5_a_contact_with_itself_holds_nothing() {
    let design = Design {
        parts: vec![
            part("a", "wood.softwood.pine", 18.0),
            part("b", "wood.softwood.pine", 18.0),
        ],
        contacts: vec![contact("a", "a")],
        ..Design::default()
    };
    let findings = evaluate(&design, &tables());
    assert_eq!(findings.len(), 2);
    assert!(findings
        .iter()
        .all(|finding| rule_of(finding) == RULE_UNSUPPORTED));
}

#[test]
fn s5_a_contact_across_parts_holds_both() {
    let design = Design {
        parts: vec![
            part("a", "wood.softwood.pine", 18.0),
            part("b", "wood.softwood.pine", 18.0),
        ],
        contacts: vec![contact("b", "a")],
        ..Design::default()
    };
    assert!(evaluate(&design, &tables()).is_empty());
}

#[test]
fn s5_a_fastener_between_two_parts_holds_them() {
    let mut design = Design {
        parts: vec![
            part("a", "wood.softwood.pine", 18.0),
            part("b", "wood.softwood.pine", 18.0),
        ],
        ..Design::default()
    };
    design
        .fasteners
        .push(screw("f", 36.0, 4.0, &["a", "b"], "a", 20.0));
    assert!(evaluate(&design, &tables()).is_empty());
}

#[test]
fn s5_a_fastener_through_one_part_holds_nothing() {
    let mut design = Design {
        parts: vec![
            part("a", "wood.softwood.pine", 18.0),
            part("b", "wood.softwood.pine", 18.0),
        ],
        ..Design::default()
    };
    design
        .fasteners
        .push(screw("f", 2.0, 4.0, &["a"], "a", 20.0));
    let findings = evaluate(&design, &tables());
    assert_eq!(findings.len(), 2);
    assert!(findings
        .iter()
        .all(|finding| rule_of(finding) == RULE_UNSUPPORTED));
}

// ---------------------------------------------------------------------------
// S6: structure.thin_wall

#[test]
fn s6_thin_wall_reports_the_worked_example() {
    let mut design = base_design();
    design.parts.push(part("panel", "wood.sheet.plywood", 2.5));
    design.contacts.push(contact("side_l", "panel"));
    let findings = evaluate(&design, &tables());
    assert_eq!(
        findings,
        vec![Finding {
            kind: Kind::NotSuitable,
            severity: Severity::Warning,
            source: Source::Rule(RULE_THIN_WALL.to_string()),
            location: Location::Entity("panel".to_string()),
            what: "panel is thin".to_string(),
            why: "panel is 2.5 mm thick; wood.sheet.plywood thinner than 3 mm is fragile."
                .to_string(),
            fix: Some("part.thickness".to_string()),
            facts: vec![
                ("part".to_string(), "panel".to_string()),
                ("thickness_mm".to_string(), "2.5".to_string()),
                ("min_wall_mm".to_string(), "3".to_string()),
            ],
        }]
    );
}

#[test]
fn s6_exact_min_wall_is_quiet() {
    let mut design = base_design();
    design.parts.push(part("panel", "wood.sheet.plywood", 3.0));
    design.contacts.push(contact("side_l", "panel"));
    assert!(evaluate(&design, &tables()).is_empty());
}

#[test]
fn s6_unknown_material_is_quiet() {
    let mut design = base_design();
    design.parts.push(part("panel", "wood.unknown", 1.0));
    design.contacts.push(contact("side_l", "panel"));
    assert!(evaluate(&design, &tables()).is_empty());
}

#[test]
fn s6_unusable_thickness_is_quiet() {
    let mut design = base_design();
    design.parts.push(part("panel", "wood.sheet.plywood", 0.0));
    design.contacts.push(contact("side_l", "panel"));
    assert!(evaluate(&design, &tables()).is_empty());
}

#[test]
fn s6_checks_the_base_parts_too() {
    let mut design = base_design();
    design.parts[0].thickness_mm = 5.0;
    let findings = evaluate(&design, &tables());
    assert_eq!(findings.len(), 1);
    assert!(rule_of(&findings[0]) == RULE_THIN_WALL);
    assert!(findings[0].location == Location::Entity("side_l".to_string()));
}

// ---------------------------------------------------------------------------
// S7: structure.slot_severs

#[test]
fn s7_slot_severs_reports_the_worked_example() {
    let mut design = base_design();
    design.slots.push(Slot {
        part: "top".to_string(),
        width_mm: 6.0,
        remaining_mm: 0.0,
    });
    let findings = evaluate(&design, &tables());
    assert_eq!(
        findings,
        vec![Finding {
            kind: Kind::Forbidden,
            severity: Severity::Critical,
            source: Source::Rule(RULE_SLOT_SEVERS.to_string()),
            location: Location::Entity("top".to_string()),
            what: "A slot cuts top in two".to_string(),
            why: "The slot in top leaves no material across it, so the part falls apart."
                .to_string(),
            fix: Some("slot.shorten".to_string()),
            facts: vec![
                ("part".to_string(), "top".to_string()),
                ("remaining_mm".to_string(), "0".to_string()),
            ],
        }]
    );
}

#[test]
fn s7_negative_remaining_also_severs() {
    let mut design = base_design();
    design.slots.push(Slot {
        part: "top".to_string(),
        width_mm: 6.0,
        remaining_mm: -2.5,
    });
    let findings = evaluate(&design, &tables());
    assert_eq!(findings.len(), 1);
    assert!(findings[0]
        .facts
        .contains(&("remaining_mm".to_string(), "-2.5".to_string())));
}

#[test]
fn s7_remaining_material_is_quiet() {
    let mut design = base_design();
    design.slots.push(Slot {
        part: "top".to_string(),
        width_mm: 6.0,
        remaining_mm: 4.0,
    });
    assert!(evaluate(&design, &tables()).is_empty());
}

#[test]
fn s7_nan_remaining_is_quiet() {
    let mut design = base_design();
    design.slots.push(Slot {
        part: "top".to_string(),
        width_mm: 6.0,
        remaining_mm: f64::NAN,
    });
    assert!(evaluate(&design, &tables()).is_empty());
}

#[test]
fn s7_a_slot_on_an_unknown_part_still_severs() {
    let mut design = base_design();
    design.slots.push(Slot {
        part: "ghost".to_string(),
        width_mm: 6.0,
        remaining_mm: 0.0,
    });
    let findings = evaluate(&design, &tables());
    assert_eq!(findings.len(), 1);
    assert!(rule_of(&findings[0]) == RULE_SLOT_SEVERS);
    assert!(findings[0].location == Location::Entity("ghost".to_string()));
}

// ---------------------------------------------------------------------------
// E1: evaluate's order and whole-design behaviour

#[test]
fn e1_the_rules_are_listed_in_run_order() {
    assert!(
        RULES
            == [
                RULE_SAG,
                RULE_SCREW_THROUGH,
                RULE_SCREW_SHORT,
                RULE_EDGE_DISTANCE,
                RULE_UNSUPPORTED,
                RULE_THIN_WALL,
                RULE_SLOT_SEVERS,
            ]
    );
}

#[test]
fn e1_gravity_is_standard() {
    // A constant compared with a literal: kept as the contract's own check of its API block.
    #[allow(clippy::assertions_on_constants)]
    {
        assert!(G == 9.81);
    }
}

#[test]
fn e1_the_base_design_has_no_findings() {
    assert!(evaluate(&base_design(), &tables()).is_empty());
}

#[test]
fn e1_an_empty_design_has_no_findings() {
    assert!(evaluate(&Design::default(), &Tables::default()).is_empty());
}

#[test]
fn e1_f4_gets_s2_then_s4() {
    let mut design = base_design();
    design.fasteners.push(screw(
        "f4",
        40.0,
        4.0,
        &["side_l", "shelf_a"],
        "shelf_a",
        8.0,
    ));
    let findings = evaluate(&design, &tables());
    let rules: Vec<&str> = findings.iter().map(rule_of).collect();
    assert!(rules == [RULE_SCREW_THROUGH, RULE_EDGE_DISTANCE]);
    assert!(findings
        .iter()
        .all(|finding| finding.location == Location::Entity("f4".to_string())));
}

#[test]
fn e1_shelves_are_reported_in_input_order() {
    let mut design = base_design();
    design.shelves.push(shelf("shelf_a", 800.0, 250.0, 30.0));
    design.shelves.push(shelf("top", 800.0, 250.0, 30.0));
    let findings = evaluate(&design, &tables());
    assert_eq!(findings.len(), 2);
    assert!(findings[0].location == Location::Entity("shelf_a".to_string()));
    assert!(findings[1].location == Location::Entity("top".to_string()));
}

#[test]
fn e1_the_combined_worked_example_runs_in_rule_order() {
    let mut design = base_design();
    design.shelves.push(shelf("shelf_a", 800.0, 250.0, 30.0));
    design.fasteners.push(screw(
        "f1",
        40.0,
        4.0,
        &["side_l", "shelf_a"],
        "shelf_a",
        20.0,
    ));
    design.fasteners.push(screw(
        "f2",
        30.0,
        4.0,
        &["side_r", "shelf_a"],
        "shelf_a",
        20.0,
    ));
    design.fasteners.push(screw(
        "f3",
        35.0,
        4.0,
        &["side_r", "shelf_a"],
        "side_r",
        8.0,
    ));
    design.parts.push(part("knob", "wood.softwood.pine", 30.0));
    design.parts.push(part("panel", "wood.sheet.plywood", 2.5));
    design.contacts.push(contact("side_l", "panel"));
    design.slots.push(Slot {
        part: "top".to_string(),
        width_mm: 6.0,
        remaining_mm: 0.0,
    });

    let findings = evaluate(&design, &tables());
    let got: Vec<(&str, &Location)> = findings
        .iter()
        .map(|finding| (rule_of(finding), &finding.location))
        .collect();
    let expected_rules = [
        RULE_SAG,
        RULE_SCREW_THROUGH,
        RULE_SCREW_SHORT,
        RULE_EDGE_DISTANCE,
        RULE_UNSUPPORTED,
        RULE_THIN_WALL,
        RULE_SLOT_SEVERS,
    ];
    assert_eq!(got.len(), expected_rules.len());
    for (index, rule) in expected_rules.iter().enumerate() {
        assert!(got[index].0 == *rule);
    }
    assert!(got[0].1 == &Location::Entity("shelf_a".to_string()));
    assert!(got[1].1 == &Location::Entity("f1".to_string()));
    assert!(got[2].1 == &Location::Entity("f2".to_string()));
    assert!(got[3].1 == &Location::Entity("f3".to_string()));
    assert!(got[4].1 == &Location::Entity("knob".to_string()));
    assert!(got[5].1 == &Location::Entity("panel".to_string()));
    assert!(got[6].1 == &Location::Entity("top".to_string()));
}

#[test]
fn e1_hostile_numbers_and_ids_never_panic() {
    let hostile = Design {
        parts: vec![
            part("nan", "wood.softwood.pine", f64::NAN),
            part("inf", "wood.unknown", f64::INFINITY),
            part("zero", "wood.softwood.pine", 0.0),
            part("", "", -1.0),
        ],
        shelves: vec![
            shelf("nan", f64::NAN, f64::INFINITY, f64::NAN),
            shelf("missing", 1e308, 250.0, 1e308),
        ],
        fasteners: vec![
            screw("empty", f64::NAN, 0.0, &[], "", -1.0),
            screw(
                "dup",
                1e308,
                f64::INFINITY,
                &["nan", "nan"],
                "inf",
                f64::INFINITY,
            ),
            dowel("d", 40.0, 8.0, &["zero", "missing"], "zero", f64::NAN),
        ],
        contacts: vec![contact("nan", "nan"), contact("", "nan")],
        slots: vec![
            Slot {
                part: "nan".to_string(),
                width_mm: f64::NAN,
                remaining_mm: f64::NEG_INFINITY,
            },
            Slot {
                part: "".to_string(),
                width_mm: 1.0,
                remaining_mm: 0.0,
            },
        ],
    };
    let findings = evaluate(&hostile, &tables());
    // Finite negatives in a slot are the only certain finding here; the point is no panic.
    assert!(findings
        .iter()
        .any(|finding| rule_of(finding) == RULE_SLOT_SEVERS));
}

#[test]
fn e1_unknown_materials_leave_every_rule_quiet() {
    let mut tables = tables();
    tables.materials.clear();
    let mut design = base_design();
    design.shelves.push(shelf("shelf_a", 800.0, 250.0, 30.0));
    design.fasteners.push(screw(
        "f1",
        40.0,
        4.0,
        &["side_l", "shelf_a"],
        "shelf_a",
        8.0,
    ));
    design.slots.push(Slot {
        part: "top".to_string(),
        width_mm: 6.0,
        remaining_mm: 0.0,
    });
    let findings = evaluate(&design, &tables);
    // S2 and S7 need no material; S1, S3, S4 and S6 do.
    let rules: Vec<&str> = findings.iter().map(rule_of).collect();
    assert!(rules == [RULE_SCREW_THROUGH, RULE_SLOT_SEVERS]);
}
