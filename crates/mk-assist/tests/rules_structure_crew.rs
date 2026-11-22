//! Tests for mk_assist::rules_structure written by Kimi (the Crew, campaign mk-core-21) from the
//! contract alone, without seeing the module.

//! Integration tests for Sancho's structure rule pack, derived from its contract.

use mk_assist::findings::{Finding, Kind, Location, Severity, Source};
use mk_assist::rules_structure::{
    evaluate, sag_mm, Contact, Design, Fastener, FastenerKind, MaterialData, Part, Shelf, Slot,
    Tables, RULES, RULE_EDGE_DISTANCE, RULE_SAG, RULE_SCREW_SHORT, RULE_SCREW_THROUGH,
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

fn part(id: &str, material_id: &str, thickness_mm: f64) -> Part {
    Part {
        id: id.to_string(),
        material: material_id.to_string(),
        thickness_mm,
    }
}

fn contact(a: &str, b: &str) -> Contact {
    Contact {
        a: a.to_string(),
        b: b.to_string(),
    }
}

fn base_design() -> Design {
    Design {
        parts: vec![
            part("side_l", "wood.softwood.pine", 18.0),
            part("side_r", "wood.softwood.pine", 18.0),
            part("shelf_a", "wood.softwood.pine", 18.0),
            part("top", "wood.sheet.particleboard", 16.0),
        ],
        shelves: vec![],
        fasteners: vec![],
        contacts: vec![
            contact("side_l", "top"),
            contact("side_r", "top"),
            contact("side_l", "shelf_a"),
            contact("side_r", "shelf_a"),
        ],
        slots: vec![],
    }
}

fn shelf(part_id: &str, span_mm: f64, depth_mm: f64, load_kg_per_m: f64) -> Shelf {
    Shelf {
        part: part_id.to_string(),
        span_mm,
        depth_mm,
        load_kg_per_m,
    }
}

#[allow(clippy::too_many_arguments)]
fn fastener(
    id: &str,
    kind: FastenerKind,
    length_mm: f64,
    diameter_mm: f64,
    joins: &[&str],
    edge_of: &str,
    edge_distance_mm: f64,
) -> Fastener {
    Fastener {
        id: id.to_string(),
        kind,
        length_mm,
        diameter_mm,
        joins: joins.iter().map(|id| (*id).to_string()).collect(),
        edge_of: edge_of.to_string(),
        edge_distance_mm,
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
    fastener(
        id,
        FastenerKind::Screw,
        length_mm,
        diameter_mm,
        joins,
        edge_of,
        edge_distance_mm,
    )
}

fn slot(part_id: &str, width_mm: f64, remaining_mm: f64) -> Slot {
    Slot {
        part: part_id.to_string(),
        width_mm,
        remaining_mm,
    }
}

#[allow(clippy::too_many_arguments)]
fn expected_finding(
    rule: &str,
    kind: Kind,
    severity: Severity,
    entity: &str,
    what: &str,
    why: &str,
    fix: &str,
    facts: &[(&str, &str)],
) -> Finding {
    Finding {
        kind,
        severity,
        source: Source::Rule(rule.to_string()),
        location: Location::Entity(entity.to_string()),
        what: what.to_string(),
        why: why.to_string(),
        fix: Some(fix.to_string()),
        facts: facts
            .iter()
            .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
            .collect(),
    }
}

fn expected_s1() -> Finding {
    expected_finding(
        RULE_SAG,
        Kind::NotSuitable,
        Severity::Warning,
        "shelf_a",
        "shelf_a sags about 1.6 mm",
        concat!(
            "Over 800 mm, 18 mm of wood.softwood.pine carrying 30 kg per metre ",
            "sags about 1.6 mm; more than 1.3 mm shows."
        ),
        "part.thickness",
        &[
            ("part", "shelf_a"),
            ("span_mm", "800"),
            ("thickness_mm", "18"),
            ("material", "wood.softwood.pine"),
            ("load_kg_per_m", "30"),
            ("sag_mm", "1.6"),
            ("allowed_mm", "1.3"),
            ("source", "test table"),
        ],
    )
}

fn expected_s2_f1() -> Finding {
    expected_finding(
        RULE_SCREW_THROUGH,
        Kind::Forbidden,
        Severity::Critical,
        "f1",
        "f1 pokes out",
        concat!(
            "f1 is 40 mm long and the parts it goes through are 36 mm together, ",
            "so its tip comes out the far side."
        ),
        "fastener.length",
        &[("fastener", "f1"), ("length_mm", "40"), ("stock_mm", "36")],
    )
}

fn expected_s3_f2() -> Finding {
    expected_finding(
        RULE_SCREW_SHORT,
        Kind::NotSuitable,
        Severity::Warning,
        "f2",
        "f2 barely bites",
        concat!(
            "f2 goes 12 mm into shelf_a; wood.softwood.pine needs at least 15 mm ",
            "to hold."
        ),
        "fastener.length",
        &[
            ("fastener", "f2"),
            ("embed_mm", "12"),
            ("min_embed_mm", "15"),
        ],
    )
}

fn expected_s4_f3() -> Finding {
    expected_finding(
        RULE_EDGE_DISTANCE,
        Kind::NotSuitable,
        Severity::Warning,
        "f3",
        "f3 is too close to the edge",
        concat!(
            "f3 sits 8 mm from the edge of side_r; wood.softwood.pine needs 12 mm ",
            "(3 times its 4 mm diameter) or it splits."
        ),
        "fastener.move",
        &[
            ("fastener", "f3"),
            ("edge_distance_mm", "8"),
            ("need_mm", "12"),
        ],
    )
}

fn expected_s5_knob() -> Finding {
    expected_finding(
        RULE_UNSUPPORTED,
        Kind::Forbidden,
        Severity::Warning,
        "knob",
        "knob is held by nothing",
        "knob touches no other part and nothing fastens it, so it would fall.",
        "part.attach",
        &[("part", "knob")],
    )
}

fn expected_s6_panel() -> Finding {
    expected_finding(
        RULE_THIN_WALL,
        Kind::NotSuitable,
        Severity::Warning,
        "panel",
        "panel is thin",
        "panel is 2.5 mm thick; wood.sheet.plywood thinner than 3 mm is fragile.",
        "part.thickness",
        &[
            ("part", "panel"),
            ("thickness_mm", "2.5"),
            ("min_wall_mm", "3"),
        ],
    )
}

fn expected_s7_top() -> Finding {
    expected_finding(
        RULE_SLOT_SEVERS,
        Kind::Forbidden,
        Severity::Critical,
        "top",
        "A slot cuts top in two",
        "The slot in top leaves no material across it, so the part falls apart.",
        "slot.shorten",
        &[("part", "top"), ("remaining_mm", "0")],
    )
}

#[test]
fn rule_b1_sag_mm_matches_the_worked_beam_formula() {
    let got =
        sag_mm(800.0, 250.0, 18.0, 8000.0, 30.0).expect("all worked-example inputs are usable");
    // I = 250 * 18^3 / 12 = 121500; the formula reduces to 218 / 135 mm.
    assert!((got - 1.614_814_814_814_814_8).abs() < 1e-9);
}

#[test]
fn rule_b1_sag_mm_allows_a_zero_load() {
    assert_eq!(sag_mm(800.0, 250.0, 18.0, 8000.0, 0.0), Some(0.0));
}

#[test]
fn rule_b1_sag_mm_rejects_unusable_numbers() {
    assert_eq!(sag_mm(0.0, 250.0, 18.0, 8000.0, 30.0), None);
    assert_eq!(sag_mm(800.0, f64::NAN, 18.0, 8000.0, 30.0), None);
    assert_eq!(sag_mm(800.0, 250.0, -18.0, 8000.0, 30.0), None);
    assert_eq!(sag_mm(800.0, 250.0, 18.0, f64::INFINITY, 30.0), None);
    assert_eq!(sag_mm(800.0, 250.0, 18.0, 8000.0, -1.0), None);
    assert_eq!(sag_mm(800.0, 250.0, 18.0, 8000.0, f64::NAN), None);
}

#[test]
fn worked_example_base_design_has_no_findings() {
    assert_eq!(evaluate(&base_design(), &tables()), vec![]);
}

#[test]
fn rule_s1_worked_example_flags_the_sagging_shelf() {
    let mut design = base_design();
    design.shelves.push(shelf("shelf_a", 800.0, 250.0, 30.0));
    assert_eq!(evaluate(&design, &tables()), vec![expected_s1()]);
}

#[test]
fn rule_s1_worked_example_short_span_is_quiet() {
    let mut design = base_design();
    design.shelves.push(shelf("shelf_a", 600.0, 250.0, 30.0));
    assert_eq!(evaluate(&design, &tables()), vec![]);
}

#[test]
fn rule_s1_worked_example_zero_load_is_quiet() {
    let mut design = base_design();
    design.shelves.push(shelf("shelf_a", 800.0, 250.0, 0.0));
    assert_eq!(evaluate(&design, &tables()), vec![]);
}

#[test]
fn rule_s1_worked_example_unknown_shelf_part_is_quiet() {
    let mut design = base_design();
    design.shelves.push(shelf("ghost", 800.0, 250.0, 30.0));
    assert_eq!(evaluate(&design, &tables()), vec![]);
}

#[test]
fn rule_s2_worked_example_flags_a_screw_through_the_stock() {
    let mut design = base_design();
    design.fasteners.push(screw(
        "f1",
        40.0,
        4.0,
        &["side_l", "shelf_a"],
        "shelf_a",
        20.0,
    ));
    assert_eq!(evaluate(&design, &tables()), vec![expected_s2_f1()]);
}

#[test]
fn rule_s2_worked_example_length_equal_to_stock_is_quiet() {
    let mut design = base_design();
    design.fasteners.push(screw(
        "f",
        36.0,
        4.0,
        &["side_l", "shelf_a"],
        "shelf_a",
        20.0,
    ));
    assert_eq!(evaluate(&design, &tables()), vec![]);
}

#[test]
fn rule_s3_worked_example_flags_a_screw_that_barely_bites() {
    let mut design = base_design();
    design.fasteners.push(screw(
        "f2",
        30.0,
        4.0,
        &["side_r", "shelf_a"],
        "shelf_a",
        20.0,
    ));
    assert_eq!(evaluate(&design, &tables()), vec![expected_s3_f2()]);
}

#[test]
fn rule_s2_s3_never_both_match_one_fastener() {
    let mut design = base_design();
    design.parts.push(part("end", "wood.softwood.pine", 10.0));
    design.contacts.push(contact("side_l", "end"));
    // Stock is 28 and embed is 30 - 18 = 12, so S2 and S3's tests both hold.
    // E1 nevertheless permits only S2.
    design
        .fasteners
        .push(screw("both", 30.0, 4.0, &["side_l", "end"], "side_l", 20.0));
    let expected = expected_finding(
        RULE_SCREW_THROUGH,
        Kind::Forbidden,
        Severity::Critical,
        "both",
        "both pokes out",
        concat!(
            "both is 30 mm long and the parts it goes through are 28 mm together, ",
            "so its tip comes out the far side."
        ),
        "fastener.length",
        &[
            ("fastener", "both"),
            ("length_mm", "30"),
            ("stock_mm", "28"),
        ],
    );
    assert_eq!(evaluate(&design, &tables()), vec![expected]);
}

#[test]
fn rule_s3_embed_is_floored_at_zero() {
    let mut design = base_design();
    design.fasteners.push(screw(
        "short",
        10.0,
        4.0,
        &["side_l", "shelf_a"],
        "shelf_a",
        20.0,
    ));
    let expected = expected_finding(
        RULE_SCREW_SHORT,
        Kind::NotSuitable,
        Severity::Warning,
        "short",
        "short barely bites",
        concat!(
            "short goes 0 mm into shelf_a; wood.softwood.pine needs at least 15 mm ",
            "to hold."
        ),
        "fastener.length",
        &[
            ("fastener", "short"),
            ("embed_mm", "0"),
            ("min_embed_mm", "15"),
        ],
    );
    assert_eq!(evaluate(&design, &tables()), vec![expected]);
}

#[test]
fn rule_s2_s3_worked_example_dowel_is_quiet() {
    let mut design = base_design();
    design.fasteners.push(fastener(
        "d1",
        FastenerKind::Dowel,
        40.0,
        8.0,
        &["side_l", "shelf_a"],
        "shelf_a",
        20.0,
    ));
    assert_eq!(evaluate(&design, &tables()), vec![]);
}

#[test]
fn rule_s2_s3_even_a_short_dowel_gets_no_screw_finding() {
    let mut design = base_design();
    design.fasteners.push(fastener(
        "short_dowel",
        FastenerKind::Dowel,
        10.0,
        8.0,
        &["side_l", "shelf_a"],
        "shelf_a",
        20.0,
    ));
    assert_eq!(evaluate(&design, &tables()), vec![]);
}

#[test]
fn rule_s4_worked_example_flags_insufficient_edge_distance() {
    let mut design = base_design();
    design.fasteners.push(screw(
        "f3",
        35.0,
        4.0,
        &["side_r", "shelf_a"],
        "side_r",
        8.0,
    ));
    assert_eq!(evaluate(&design, &tables()), vec![expected_s4_f3()]);
}

#[test]
fn rule_e1_worked_example_s2_precedes_s4_for_the_same_screw() {
    let mut design = base_design();
    design.fasteners.push(screw(
        "f4",
        40.0,
        4.0,
        &["side_l", "shelf_a"],
        "shelf_a",
        8.0,
    ));
    let s2 = expected_finding(
        RULE_SCREW_THROUGH,
        Kind::Forbidden,
        Severity::Critical,
        "f4",
        "f4 pokes out",
        concat!(
            "f4 is 40 mm long and the parts it goes through are 36 mm together, ",
            "so its tip comes out the far side."
        ),
        "fastener.length",
        &[("fastener", "f4"), ("length_mm", "40"), ("stock_mm", "36")],
    );
    let s4 = expected_finding(
        RULE_EDGE_DISTANCE,
        Kind::NotSuitable,
        Severity::Warning,
        "f4",
        "f4 is too close to the edge",
        concat!(
            "f4 sits 8 mm from the edge of shelf_a; wood.softwood.pine needs 12 mm ",
            "(3 times its 4 mm diameter) or it splits."
        ),
        "fastener.move",
        &[
            ("fastener", "f4"),
            ("edge_distance_mm", "8"),
            ("need_mm", "12"),
        ],
    );
    assert_eq!(evaluate(&design, &tables()), vec![s2, s4]);
}

#[test]
fn rule_s4_dowel_uses_the_dowel_edge_diameters() {
    let mut design = base_design();
    design.fasteners.push(fastener(
        "dowel_edge",
        FastenerKind::Dowel,
        40.0,
        8.0,
        &["side_l", "shelf_a"],
        "shelf_a",
        15.0,
    ));
    let expected = expected_finding(
        RULE_EDGE_DISTANCE,
        Kind::NotSuitable,
        Severity::Warning,
        "dowel_edge",
        "dowel_edge is too close to the edge",
        concat!(
            "dowel_edge sits 15 mm from the edge of shelf_a; wood.softwood.pine needs ",
            "16 mm (2 times its 8 mm diameter) or it splits."
        ),
        "fastener.move",
        &[
            ("fastener", "dowel_edge"),
            ("edge_distance_mm", "15"),
            ("need_mm", "16"),
        ],
    );
    assert_eq!(evaluate(&design, &tables()), vec![expected]);
}

#[test]
fn rule_s4_invalid_diameters_and_edge_distances_are_quiet() {
    let mut design = base_design();
    design.fasteners.push(screw(
        "zero_d",
        36.0,
        0.0,
        &["side_l", "shelf_a"],
        "shelf_a",
        0.0,
    ));
    design.fasteners.push(screw(
        "negative_edge",
        36.0,
        4.0,
        &["side_l", "shelf_a"],
        "shelf_a",
        -1.0,
    ));
    design.fasteners.push(screw(
        "infinite_edge",
        36.0,
        4.0,
        &["side_l", "shelf_a"],
        "shelf_a",
        f64::INFINITY,
    ));
    design.fasteners.push(screw(
        "infinite_diameter",
        36.0,
        f64::INFINITY,
        &["side_l", "shelf_a"],
        "shelf_a",
        0.0,
    ));
    assert_eq!(evaluate(&design, &tables()), vec![]);
}

#[test]
fn rule_s5_worked_example_flags_an_unsupported_part() {
    let mut design = base_design();
    design.parts.push(part("knob", "wood.softwood.pine", 30.0));
    assert_eq!(evaluate(&design, &tables()), vec![expected_s5_knob()]);
}

#[test]
fn rule_s5_worked_example_one_part_design_is_quiet() {
    let design = Design {
        parts: vec![part("knob", "wood.softwood.pine", 30.0)],
        ..Design::default()
    };
    assert_eq!(evaluate(&design, &tables()), vec![]);
}

#[test]
fn rule_s5_self_contact_and_a_one_part_fastener_hold_nothing() {
    let mut design = base_design();
    design.parts.push(part("knob", "wood.softwood.pine", 30.0));
    design.contacts.push(contact("knob", "knob"));
    design
        .fasteners
        .push(screw("self_fastener", 30.0, 4.0, &["knob"], "side_l", 20.0));
    assert_eq!(evaluate(&design, &tables()), vec![expected_s5_knob()]);
}

#[test]
fn rule_s5_a_fastener_listing_a_different_part_holds_both_parts() {
    let design = Design {
        parts: vec![
            part("a", "wood.softwood.pine", 18.0),
            part("b", "wood.softwood.pine", 18.0),
        ],
        fasteners: vec![screw("joint", 36.0, 4.0, &["a", "b"], "a", 20.0)],
        ..Design::default()
    };
    assert_eq!(evaluate(&design, &tables()), vec![]);
}

#[test]
fn rule_s6_worked_example_flags_a_thin_wall() {
    let mut design = base_design();
    design.parts.push(part("panel", "wood.sheet.plywood", 2.5));
    design.contacts.push(contact("side_l", "panel"));
    assert_eq!(evaluate(&design, &tables()), vec![expected_s6_panel()]);
}

#[test]
fn rule_s6_thickness_equal_to_the_minimum_is_quiet() {
    let mut design = base_design();
    design.parts.push(part("exact", "wood.sheet.plywood", 3.0));
    design.contacts.push(contact("side_l", "exact"));
    assert_eq!(evaluate(&design, &tables()), vec![]);
}

#[test]
fn rule_s7_worked_example_flags_a_slot_that_severs_a_part() {
    let mut design = base_design();
    design.slots.push(slot("top", 6.0, 0.0));
    assert_eq!(evaluate(&design, &tables()), vec![expected_s7_top()]);
}

#[test]
fn rule_s7_worked_example_positive_remaining_material_is_quiet() {
    let mut design = base_design();
    design.slots.push(slot("top", 6.0, 4.0));
    assert_eq!(evaluate(&design, &tables()), vec![]);
}

#[test]
fn rule_s7_negative_remaining_material_severs_the_part() {
    let mut design = base_design();
    design.slots.push(slot("top", 6.0, -2.0));
    let expected = expected_finding(
        RULE_SLOT_SEVERS,
        Kind::Forbidden,
        Severity::Critical,
        "top",
        "A slot cuts top in two",
        "The slot in top leaves no material across it, so the part falls apart.",
        "slot.shorten",
        &[("part", "top"), ("remaining_mm", "-2")],
    );
    assert_eq!(evaluate(&design, &tables()), vec![expected]);
}

#[test]
fn rule_e1_all_in_one_worked_example_has_the_contract_order() {
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
    design.slots.push(slot("top", 6.0, 0.0));

    assert_eq!(
        evaluate(&design, &tables()),
        vec![
            expected_s1(),
            expected_s2_f1(),
            expected_s3_f2(),
            expected_s4_f3(),
            expected_s5_knob(),
            expected_s6_panel(),
            expected_s7_top(),
        ]
    );
}

#[test]
fn rule_n1_unknown_ids_give_no_finding() {
    let mut design = base_design();
    design.shelves.push(shelf("ghost", 800.0, 250.0, 30.0));
    design.parts.push(part("mystery", "ghost.material", 2.0));
    design.contacts.push(contact("side_l", "mystery"));
    design.shelves.push(shelf("mystery", 800.0, 250.0, 30.0));
    design.fasteners.push(screw(
        "unknown_join",
        40.0,
        4.0,
        &["ghost", "shelf_a"],
        "shelf_a",
        20.0,
    ));
    design.fasteners.push(screw(
        "unknown_edge_part",
        36.0,
        4.0,
        &["side_l", "shelf_a"],
        "ghost",
        0.0,
    ));
    design.fasteners.push(screw(
        "unknown_edge_material",
        36.0,
        4.0,
        &["side_l", "shelf_a"],
        "mystery",
        0.0,
    ));
    assert_eq!(evaluate(&design, &tables()), vec![]);
}

#[test]
fn rule_n1_nan_numbers_give_no_finding() {
    let mut design = base_design();
    design.shelves.push(shelf("shelf_a", f64::NAN, 250.0, 30.0));
    design.fasteners.push(screw(
        "nan_length",
        f64::NAN,
        4.0,
        &["side_l", "shelf_a"],
        "shelf_a",
        20.0,
    ));
    design.fasteners.push(screw(
        "nan_dimensions",
        36.0,
        f64::NAN,
        &["side_l", "shelf_a"],
        "shelf_a",
        f64::NAN,
    ));
    design
        .parts
        .push(part("nan_wall", "wood.softwood.pine", f64::NAN));
    design.contacts.push(contact("side_l", "nan_wall"));
    design.slots.push(slot("top", 6.0, f64::NAN));
    assert_eq!(evaluate(&design, &tables()), vec![]);
}

#[test]
fn rule_e1_rules_constant_lists_the_evaluation_order() {
    assert_eq!(
        RULES,
        [
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
