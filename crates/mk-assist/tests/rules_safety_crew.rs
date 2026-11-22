//! Tests for mk_assist::rules_safety written by Deepak (the Crew, campaign mk-core-23) from the
//! contract alone, without seeing the module.

//! Integration tests for `mk_assist::rules_safety` (contract v1).

// Crew-written tests: a helper that builds a whole `Finding` takes one argument per field, and
// worked rows print constants to the contract's digits; neither matters in a test.
#![allow(clippy::too_many_arguments, clippy::approx_constant)]

use mk_assist::findings::{Finding, Kind, Location, Severity, Source};
use mk_assist::rules_safety::{
    evaluate, family, needs_capability, skill_word, Furniture, Material, SafetyFacts, Skill, Step,
    Wall, WallPart, RULES, RULE_ANTI_TIP, RULE_LASER_NEVER, RULE_MAINS, RULE_VENTILATION,
    RULE_WALL_ANCHOR, TALL_FURNITURE_MM,
};

fn facts_of(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
    pairs
        .iter()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect()
}

fn finding(
    kind: Kind,
    severity: Severity,
    rule: &str,
    location: Location,
    what: &str,
    why: &str,
    fix: Option<&str>,
    facts: &[(&str, &str)],
) -> Finding {
    Finding {
        kind,
        severity,
        source: Source::Rule(rule.to_string()),
        location,
        what: what.to_string(),
        why: why.to_string(),
        fix: fix.map(|text| text.to_string()),
        facts: facts_of(facts),
    }
}

fn base() -> SafetyFacts {
    SafetyFacts {
        steps: Vec::new(),
        materials: vec![
            Material {
                id: "pvc".to_string(),
                laser_never: true,
            },
            Material {
                id: "ply".to_string(),
                laser_never: false,
            },
        ],
        ventilated: Some(true),
        wiring: false,
        skill: None,
        furniture: Vec::new(),
        wall_parts: Vec::new(),
    }
}

fn step(id: u32, needs: &[&str], material: Option<&str>) -> Step {
    Step {
        id,
        needs: needs.iter().map(|text| text.to_string()).collect(),
        material: material.map(|text| text.to_string()),
    }
}

fn furniture(
    id: &str,
    child: bool,
    height_mm: f64,
    drawers: bool,
    anchored: Option<bool>,
) -> Furniture {
    Furniture {
        id: id.to_string(),
        child,
        height_mm,
        drawers,
        anchored,
    }
}

fn wall_part(id: &str, load_kg: f64, anchor_rated_kg: Option<f64>, wall: Wall) -> WallPart {
    WallPart {
        id: id.to_string(),
        load_kg,
        anchor_rated_kg,
        wall,
    }
}

// ---- P1 ----

#[test]
fn rule_p1_family_is_text_before_first_colon() {
    assert_eq!(family("cut.laser:thin"), "cut.laser");
    assert_eq!(family("finish.paint"), "finish.paint");
    assert_eq!(family(""), "");
    assert_eq!(family(":thin"), "");
    assert_eq!(family("a:b:c"), "a");
}

// ---- P2 ----

#[test]
fn rule_p2_needs_capability_matches_family() {
    let needs = vec!["cut.laser:thin".to_string(), "finish.paint".to_string()];
    assert!(needs_capability(&needs, "cut.laser"));
    assert!(needs_capability(&needs, "finish.paint"));
    assert!(!needs_capability(&needs, "cut.saw"));
    assert!(!needs_capability(&[], "cut.laser"));
}

// ---- P3 ----

#[test]
fn rule_p3_skill_word() {
    assert_eq!(skill_word(Skill::Beginner), "beginner");
    assert_eq!(skill_word(Skill::Comfortable), "comfortable");
    assert_eq!(skill_word(Skill::Experienced), "experienced");
    assert_eq!(skill_word(Skill::Pro), "pro");
}

// ---- constants and rule order ----

#[test]
fn rule_e1_rules_constant_order() {
    assert_eq!(
        RULES,
        [
            RULE_LASER_NEVER,
            RULE_VENTILATION,
            RULE_MAINS,
            RULE_ANTI_TIP,
            RULE_WALL_ANCHOR
        ]
    );
    assert_eq!(RULE_LASER_NEVER, "safety.laser_never");
    assert_eq!(RULE_VENTILATION, "safety.ventilation");
    assert_eq!(RULE_MAINS, "safety.mains");
    assert_eq!(RULE_ANTI_TIP, "safety.anti_tip");
    assert_eq!(RULE_WALL_ANCHOR, "safety.wall_anchor");
    assert_eq!(TALL_FURNITURE_MM, 1200.0);
}

// ---- P4 numbers ----

#[test]
fn rule_p4_numbers_use_plain_f64_display() {
    let mut tall = base();
    tall.furniture = vec![furniture("dresser", false, 1400.0, true, Some(false))];
    let got = evaluate(&tall);
    assert_eq!(got.len(), 1);
    assert_eq!(
        got[0].facts[2],
        ("height_mm".to_string(), "1400".to_string())
    );

    let mut heavy = base();
    heavy.wall_parts = vec![wall_part("tv", 25.5, Some(25.0), Wall::Stud)];
    let got = evaluate(&heavy);
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].facts[1], ("load_kg".to_string(), "25.5".to_string()));
    assert_eq!(
        got[0].facts[2],
        ("anchor_rated_kg".to_string(), "25".to_string())
    );
}

// ---- L1 ----

#[test]
fn example_01_laser_never_pvc() {
    let mut facts = base();
    facts.steps = vec![step(1, &["cut.laser"], Some("pvc"))];
    let want =
        vec![finding(
        Kind::Unsafe,
        Severity::Critical,
        RULE_LASER_NEVER,
        Location::Step(1),
        "pvc must never be laser-cut",
        "Step 1 cuts pvc with a laser. That releases chlorine gas, which harms you and the laser.",
        Some("step.change_material"),
        &[("step", "1"), ("capability", "cut.laser"), ("material", "pvc")],
    )];
    assert_eq!(evaluate(&facts), want);
}

#[test]
fn example_02_laser_never_ply_is_silence() {
    let mut facts = base();
    facts.steps = vec![step(2, &["cut.laser"], Some("ply"))];
    assert_eq!(evaluate(&facts), Vec::new());
}

#[test]
fn example_03_laser_never_unknown_material_is_silence() {
    let mut facts = base();
    facts.steps = vec![step(2, &["cut.laser"], Some("oak"))];
    assert_eq!(evaluate(&facts), Vec::new());
}

#[test]
fn example_04_laser_never_no_material_is_silence() {
    let mut facts = base();
    facts.steps = vec![step(2, &["cut.laser"], None)];
    assert_eq!(evaluate(&facts), Vec::new());
}

#[test]
fn example_05_laser_never_matches_family() {
    let mut facts = base();
    facts.steps = vec![step(9, &["cut.laser:thin"], Some("pvc"))];
    let want =
        vec![finding(
        Kind::Unsafe,
        Severity::Critical,
        RULE_LASER_NEVER,
        Location::Step(9),
        "pvc must never be laser-cut",
        "Step 9 cuts pvc with a laser. That releases chlorine gas, which harms you and the laser.",
        Some("step.change_material"),
        &[("step", "9"), ("capability", "cut.laser"), ("material", "pvc")],
    )];
    assert_eq!(evaluate(&facts), want);
}

#[test]
fn example_06_laser_never_input_order() {
    let mut facts = base();
    facts.steps = vec![
        step(1, &["cut.laser"], Some("pvc")),
        step(2, &["cut.laser"], Some("pvc")),
    ];
    let want = vec![
        finding(
            Kind::Unsafe,
            Severity::Critical,
            RULE_LASER_NEVER,
            Location::Step(1),
            "pvc must never be laser-cut",
            "Step 1 cuts pvc with a laser. That releases chlorine gas, which harms you and the laser.",
            Some("step.change_material"),
            &[("step", "1"), ("capability", "cut.laser"), ("material", "pvc")],
        ),
        finding(
            Kind::Unsafe,
            Severity::Critical,
            RULE_LASER_NEVER,
            Location::Step(2),
            "pvc must never be laser-cut",
            "Step 2 cuts pvc with a laser. That releases chlorine gas, which harms you and the laser.",
            Some("step.change_material"),
            &[("step", "2"), ("capability", "cut.laser"), ("material", "pvc")],
        ),
    ];
    assert_eq!(evaluate(&facts), want);
}

// ---- V1 ----

#[test]
fn example_07_ventilation_paint() {
    let mut facts = base();
    facts.ventilated = Some(false);
    facts.steps = vec![step(3, &["finish.paint"], None)];
    let want = vec![finding(
        Kind::Unsafe,
        Severity::Warning,
        RULE_VENTILATION,
        Location::Step(3),
        "Finishing with finish.paint in an unventilated space",
        "Step 3 applies finish.paint and the space has no ventilation, so the fumes build up. \
Work outside or ventilate the space.",
        Some("space.ventilate"),
        &[("step", "3"), ("capability", "finish.paint")],
    )];
    assert_eq!(evaluate(&facts), want);
}

#[test]
fn example_08_ventilation_oil() {
    let mut facts = base();
    facts.ventilated = Some(false);
    facts.steps = vec![step(4, &["finish.oil"], None)];
    let want = vec![finding(
        Kind::Unsafe,
        Severity::Warning,
        RULE_VENTILATION,
        Location::Step(4),
        "Finishing with finish.oil in an unventilated space",
        "Step 4 applies finish.oil and the space has no ventilation, so the fumes build up. \
Work outside or ventilate the space.",
        Some("space.ventilate"),
        &[("step", "4"), ("capability", "finish.oil")],
    )];
    assert_eq!(evaluate(&facts), want);
}

#[test]
fn example_09_ventilated_true_is_silence() {
    let mut facts = base();
    facts.ventilated = Some(true);
    facts.steps = vec![step(3, &["finish.paint"], None)];
    assert_eq!(evaluate(&facts), Vec::new());
}

#[test]
fn example_10_ventilated_unknown_is_silence() {
    let mut facts = base();
    facts.ventilated = None;
    facts.steps = vec![step(3, &["finish.paint"], None)];
    assert_eq!(evaluate(&facts), Vec::new());
}

#[test]
fn example_11_ventilation_first_finish_wins() {
    let mut facts = base();
    facts.ventilated = Some(false);
    facts.steps = vec![step(5, &["finish.oil", "finish.paint"], None)];
    let want = vec![finding(
        Kind::Unsafe,
        Severity::Warning,
        RULE_VENTILATION,
        Location::Step(5),
        "Finishing with finish.oil in an unventilated space",
        "Step 5 applies finish.oil and the space has no ventilation, so the fumes build up. \
Work outside or ventilate the space.",
        Some("space.ventilate"),
        &[("step", "5"), ("capability", "finish.oil")],
    )];
    assert_eq!(evaluate(&facts), want);
}

// ---- M1 ----

#[test]
fn example_12_mains_beginner() {
    let mut facts = base();
    facts.wiring = true;
    facts.skill = Some(Skill::Beginner);
    let want = vec![finding(
        Kind::Unsafe,
        Severity::Critical,
        RULE_MAINS,
        Location::Project,
        "Mains wiring at beginner skill",
        "This design has mains-voltage wiring and your skill is beginner. A mains mistake can \
kill; have a licensed electrician do or check this work.",
        Some("plan.add_electrician"),
        &[("wiring", "mains"), ("skill", "beginner")],
    )];
    assert_eq!(evaluate(&facts), want);
}

#[test]
fn example_13_mains_comfortable() {
    let mut facts = base();
    facts.wiring = true;
    facts.skill = Some(Skill::Comfortable);
    let want = vec![finding(
        Kind::Tip,
        Severity::Info,
        RULE_MAINS,
        Location::Project,
        "Mains wiring needs an electrician's check",
        "This design has mains-voltage wiring and your skill is comfortable. Check your local \
rules and have the work inspected.",
        None,
        &[("wiring", "mains"), ("skill", "comfortable")],
    )];
    assert_eq!(evaluate(&facts), want);
}

#[test]
fn example_14_no_wiring_is_silence() {
    let mut facts = base();
    facts.wiring = false;
    facts.skill = Some(Skill::Beginner);
    assert_eq!(evaluate(&facts), Vec::new());
}

#[test]
fn example_15_unknown_skill_is_silence() {
    let mut facts = base();
    facts.wiring = true;
    facts.skill = None;
    assert_eq!(evaluate(&facts), Vec::new());
}

// ---- A1 ----

#[test]
fn example_16_anti_tip_tall_dresser() {
    let mut facts = base();
    facts.furniture = vec![furniture("dresser", false, 1400.0, true, Some(false))];
    let want = vec![finding(
        Kind::Unsafe,
        Severity::Warning,
        RULE_ANTI_TIP,
        Location::Entity("dresser".to_string()),
        "dresser can tip over",
        "dresser is not anchored to a wall, and a child can bring it down by climbing it or \
pulling a drawer. Anchor it to a stud.",
        Some("part.add_anchor"),
        &[
            ("part", "dresser"),
            ("child", "false"),
            ("height_mm", "1400"),
            ("drawers", "true"),
        ],
    )];
    assert_eq!(evaluate(&facts), want);
}

#[test]
fn example_17_anti_tip_short_cabinet_is_silence() {
    let mut facts = base();
    facts.furniture = vec![furniture("cabinet", false, 900.0, true, Some(false))];
    assert_eq!(evaluate(&facts), Vec::new());
}

#[test]
fn example_18_anti_tip_child_crib() {
    let mut facts = base();
    facts.furniture = vec![furniture("crib", true, 700.0, false, Some(false))];
    let want = vec![finding(
        Kind::Unsafe,
        Severity::Warning,
        RULE_ANTI_TIP,
        Location::Entity("crib".to_string()),
        "crib can tip over",
        "crib is not anchored to a wall, and a child can bring it down by climbing it or \
pulling a drawer. Anchor it to a stud.",
        Some("part.add_anchor"),
        &[
            ("part", "crib"),
            ("child", "true"),
            ("height_mm", "700"),
            ("drawers", "false"),
        ],
    )];
    assert_eq!(evaluate(&facts), want);
}

#[test]
fn example_19_anti_tip_anchored_is_silence() {
    let mut facts = base();
    facts.furniture = vec![furniture("dresser", false, 1400.0, true, Some(true))];
    assert_eq!(evaluate(&facts), Vec::new());
}

#[test]
fn example_20_anti_tip_unknown_anchor_is_silence() {
    let mut facts = base();
    facts.furniture = vec![furniture("table", true, 700.0, false, None)];
    assert_eq!(evaluate(&facts), Vec::new());
}

#[test]
fn example_21_anti_tip_equal_height_is_silence() {
    let mut facts = base();
    facts.furniture = vec![furniture("cabinet", false, 1200.0, true, Some(false))];
    assert_eq!(evaluate(&facts), Vec::new());
}

#[test]
fn example_22_anti_tip_non_finite_height_is_silence() {
    let mut facts = base();
    facts.furniture = vec![
        furniture("nan", false, f64::NAN, true, Some(false)),
        furniture("inf", false, f64::INFINITY, true, Some(false)),
    ];
    assert_eq!(evaluate(&facts), Vec::new());
}

// ---- W1 ----

#[test]
fn example_23_wall_anchor_heavier_than_anchor() {
    let mut facts = base();
    facts.wall_parts = vec![wall_part("tv", 30.0, Some(25.0), Wall::Stud)];
    let want = vec![finding(
        Kind::Unsafe,
        Severity::Critical,
        RULE_WALL_ANCHOR,
        Location::Entity("tv".to_string()),
        "tv is heavier than its anchor",
        "tv weighs 30 kg and its anchor is rated 25 kg. Use a stronger fixing or lighten the part.",
        Some("part.change_anchor"),
        &[("part", "tv"), ("load_kg", "30"), ("anchor_rated_kg", "25")],
    )];
    assert_eq!(evaluate(&facts), want);
}

#[test]
fn example_24_wall_anchor_light_part_is_silence() {
    let mut facts = base();
    facts.wall_parts = vec![wall_part("shelf", 10.0, Some(25.0), Wall::Masonry)];
    assert_eq!(evaluate(&facts), Vec::new());
}

#[test]
fn example_25_wall_anchor_equal_is_not_excess() {
    let mut facts = base();
    facts.wall_parts = vec![wall_part("shelf", 25.0, Some(25.0), Wall::Stud)];
    assert_eq!(evaluate(&facts), Vec::new());
}

#[test]
fn example_26_wall_anchor_unknown_wall() {
    let mut facts = base();
    facts.wall_parts = vec![wall_part("mirror", 5.0, Some(50.0), Wall::Unknown)];
    let want = vec![finding(
        Kind::Unsafe,
        Severity::Warning,
        RULE_WALL_ANCHOR,
        Location::Entity("mirror".to_string()),
        "mirror is on a wall of unknown type",
        "mirror is fixed to a wall whose type is unknown, so its anchor cannot be checked. \
Find the studs or the substrate first.",
        Some("part.confirm_wall"),
        &[("part", "mirror"), ("wall", "unknown")],
    )];
    assert_eq!(evaluate(&facts), want);
}

#[test]
fn example_27_wall_anchor_case_one_wins() {
    let mut facts = base();
    facts.wall_parts = vec![wall_part("shelf", 40.0, Some(25.0), Wall::Unknown)];
    let want = vec![finding(
        Kind::Unsafe,
        Severity::Critical,
        RULE_WALL_ANCHOR,
        Location::Entity("shelf".to_string()),
        "shelf is heavier than its anchor",
        "shelf weighs 40 kg and its anchor is rated 25 kg. Use a stronger fixing or lighten the part.",
        Some("part.change_anchor"),
        &[("part", "shelf"), ("load_kg", "40"), ("anchor_rated_kg", "25")],
    )];
    assert_eq!(evaluate(&facts), want);
}

#[test]
fn example_28_wall_anchor_non_finite_and_missing_rating_is_silence() {
    let mut facts = base();
    facts.wall_parts = vec![
        wall_part("a", 30.0, None, Wall::Stud),
        wall_part("b", f64::NAN, Some(25.0), Wall::Stud),
        wall_part("c", f64::INFINITY, Some(25.0), Wall::Masonry),
    ];
    assert_eq!(evaluate(&facts), Vec::new());
}

// ---- whole-pack rows ----

#[test]
fn example_29_all_rules_in_order() {
    let mut facts = base();
    facts.steps = vec![
        step(1, &["cut.laser"], Some("pvc")),
        step(2, &["finish.paint"], None),
    ];
    facts.ventilated = Some(false);
    facts.wiring = true;
    facts.skill = Some(Skill::Beginner);
    facts.furniture = vec![furniture("dresser", false, 1400.0, true, Some(false))];
    facts.wall_parts = vec![wall_part("tv", 30.0, Some(25.0), Wall::Stud)];
    let want = vec![
        finding(
            Kind::Unsafe,
            Severity::Critical,
            RULE_LASER_NEVER,
            Location::Step(1),
            "pvc must never be laser-cut",
            "Step 1 cuts pvc with a laser. That releases chlorine gas, which harms you and the laser.",
            Some("step.change_material"),
            &[("step", "1"), ("capability", "cut.laser"), ("material", "pvc")],
        ),
        finding(
            Kind::Unsafe,
            Severity::Warning,
            RULE_VENTILATION,
            Location::Step(2),
            "Finishing with finish.paint in an unventilated space",
            "Step 2 applies finish.paint and the space has no ventilation, so the fumes build up. \
Work outside or ventilate the space.",
            Some("space.ventilate"),
            &[("step", "2"), ("capability", "finish.paint")],
        ),
        finding(
            Kind::Unsafe,
            Severity::Critical,
            RULE_MAINS,
            Location::Project,
            "Mains wiring at beginner skill",
            "This design has mains-voltage wiring and your skill is beginner. A mains mistake can \
kill; have a licensed electrician do or check this work.",
            Some("plan.add_electrician"),
            &[("wiring", "mains"), ("skill", "beginner")],
        ),
        finding(
            Kind::Unsafe,
            Severity::Warning,
            RULE_ANTI_TIP,
            Location::Entity("dresser".to_string()),
            "dresser can tip over",
            "dresser is not anchored to a wall, and a child can bring it down by climbing it or \
pulling a drawer. Anchor it to a stud.",
            Some("part.add_anchor"),
            &[
                ("part", "dresser"),
                ("child", "false"),
                ("height_mm", "1400"),
                ("drawers", "true"),
            ],
        ),
        finding(
            Kind::Unsafe,
            Severity::Critical,
            RULE_WALL_ANCHOR,
            Location::Entity("tv".to_string()),
            "tv is heavier than its anchor",
            "tv weighs 30 kg and its anchor is rated 25 kg. Use a stronger fixing or lighten the part.",
            Some("part.change_anchor"),
            &[("part", "tv"), ("load_kg", "30"), ("anchor_rated_kg", "25")],
        ),
    ];
    assert_eq!(evaluate(&facts), want);
}

#[test]
fn example_30_default_facts_give_nothing() {
    assert_eq!(evaluate(&SafetyFacts::default()), Vec::new());
}

#[test]
fn example_31_one_step_two_rules() {
    let mut facts = base();
    facts.ventilated = Some(false);
    facts.steps = vec![step(6, &["cut.laser", "finish.oil"], Some("pvc"))];
    let want = vec![
        finding(
            Kind::Unsafe,
            Severity::Critical,
            RULE_LASER_NEVER,
            Location::Step(6),
            "pvc must never be laser-cut",
            "Step 6 cuts pvc with a laser. That releases chlorine gas, which harms you and the laser.",
            Some("step.change_material"),
            &[("step", "6"), ("capability", "cut.laser"), ("material", "pvc")],
        ),
        finding(
            Kind::Unsafe,
            Severity::Warning,
            RULE_VENTILATION,
            Location::Step(6),
            "Finishing with finish.oil in an unventilated space",
            "Step 6 applies finish.oil and the space has no ventilation, so the fumes build up. \
Work outside or ventilate the space.",
            Some("space.ventilate"),
            &[("step", "6"), ("capability", "finish.oil")],
        ),
    ];
    assert_eq!(evaluate(&facts), want);
}

#[test]
fn example_32_repeated_capability_gives_one_finding() {
    let mut facts = base();
    facts.ventilated = Some(false);
    facts.steps = vec![step(8, &["finish.oil", "finish.oil"], None)];
    let want = vec![finding(
        Kind::Unsafe,
        Severity::Warning,
        RULE_VENTILATION,
        Location::Step(8),
        "Finishing with finish.oil in an unventilated space",
        "Step 8 applies finish.oil and the space has no ventilation, so the fumes build up. \
Work outside or ventilate the space.",
        Some("space.ventilate"),
        &[("step", "8"), ("capability", "finish.oil")],
    )];
    assert_eq!(evaluate(&facts), want);
}

#[test]
fn example_33_absurd_inputs_never_panic() {
    let mut facts = base();
    facts.materials = Vec::new();
    facts.ventilated = Some(false);
    facts.steps = vec![
        step(u32::MAX, &[], Some("ghost")),
        step(7, &["finish.paint"], None),
    ];
    facts.furniture = vec![
        furniture("f1", false, f64::NEG_INFINITY, true, Some(false)),
        furniture("f2", false, f64::NAN, true, Some(false)),
    ];
    facts.wall_parts = vec![
        wall_part("", f64::NAN, Some(25.0), Wall::Unknown),
        wall_part("a", f64::NEG_INFINITY, Some(25.0), Wall::Stud),
        wall_part("b", f64::INFINITY, Some(25.0), Wall::Stud),
    ];
    let want = vec![
        finding(
            Kind::Unsafe,
            Severity::Warning,
            RULE_VENTILATION,
            Location::Step(7),
            "Finishing with finish.paint in an unventilated space",
            "Step 7 applies finish.paint and the space has no ventilation, so the fumes build up. \
Work outside or ventilate the space.",
            Some("space.ventilate"),
            &[("step", "7"), ("capability", "finish.paint")],
        ),
        finding(
            Kind::Unsafe,
            Severity::Warning,
            RULE_WALL_ANCHOR,
            Location::Entity(String::new()),
            " is on a wall of unknown type",
            " is fixed to a wall whose type is unknown, so its anchor cannot be checked. \
Find the studs or the substrate first.",
            Some("part.confirm_wall"),
            &[("part", ""), ("wall", "unknown")],
        ),
    ];
    assert_eq!(evaluate(&facts), want);
}
