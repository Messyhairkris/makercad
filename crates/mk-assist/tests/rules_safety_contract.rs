//! Contract tests for mk_assist::rules_safety, written by Gordon as a PA (the Crew) from
//! crew/contracts/mk_rules_safety.md. They were the hidden tests of campaign mk-core-23.

use mk_assist::findings::{Finding, Kind, Location, Severity, Source};
use mk_assist::rules_safety::{
    evaluate, family, needs_capability, skill_word, Furniture, Material, SafetyFacts, Skill, Step,
    Wall, WallPart, RULES, RULE_ANTI_TIP, RULE_LASER_NEVER, RULE_MAINS, RULE_VENTILATION,
    RULE_WALL_ANCHOR, TALL_FURNITURE_MM,
};

// Hidden tests for `mk_assist::rules_safety` (contract: crew/contracts/mk_rules_safety.md).
// Each worked-example row of that contract is one `example_NN_...` test below.

/// One expected finding; the facts are `(key, value)` pairs in the rule's own order.
macro_rules! find {
    ($kind:expr, $severity:expr, $rule:expr, $location:expr, $what:expr, $why:expr, $fix:expr, $facts:expr) => {
        Finding {
            kind: $kind,
            severity: $severity,
            source: Source::Rule($rule.to_string()),
            location: $location,
            what: $what.to_string(),
            why: $why.to_string(),
            fix: $fix.map(|f: &str| f.to_string()),
            facts: $facts
                .iter()
                .map(|kv| (kv.0.to_string(), kv.1.to_string()))
                .collect(),
        }
    };
}

fn step(id: u32, needs: &[&str], material: Option<&str>) -> Step {
    Step {
        id,
        needs: needs.iter().map(|s| s.to_string()).collect(),
        material: material.map(|m| m.to_string()),
    }
}

fn mat(id: &str, laser_never: bool) -> Material {
    Material {
        id: id.to_string(),
        laser_never,
    }
}

fn furn(id: &str, child: bool, height_mm: f64, drawers: bool, anchored: Option<bool>) -> Furniture {
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

/// pvc is never laser-safe and ply is safe; ventilated, no wiring, no furniture, no wall parts.
fn step_facts(steps: Vec<Step>) -> SafetyFacts {
    SafetyFacts {
        steps,
        materials: vec![mat("pvc", true), mat("ply", false)],
        ventilated: Some(true),
        wiring: false,
        skill: None,
        furniture: Vec::new(),
        wall_parts: Vec::new(),
    }
}

// ---- P1-P3: the helpers -------------------------------------------------------------

#[test]
fn family_strips_the_suffix() {
    assert!(family("cut.laser:thin") == "cut.laser");
    assert!(family("finish.paint") == "finish.paint");
    assert!(family("").is_empty());
}

#[test]
fn family_takes_the_text_before_the_first_colon() {
    assert!(family("a:b:c") == "a");
    assert!(family(":odd").is_empty());
}

#[test]
fn needs_capability_matches_families() {
    let needs = vec!["cut.laser:thin".to_string(), "shape.sand".to_string()];
    assert!(needs_capability(&needs, "cut.laser"));
    assert!(!needs_capability(&needs, "cut.straight"));
}

#[test]
fn needs_capability_of_nothing_is_false() {
    assert!(!needs_capability(&[], "cut.laser"));
}

#[test]
fn skill_words_are_the_docs_words() {
    assert!(skill_word(Skill::Beginner) == "beginner");
    assert!(skill_word(Skill::Comfortable) == "comfortable");
    assert!(skill_word(Skill::Experienced) == "experienced");
    assert!(skill_word(Skill::Pro) == "pro");
}

#[test]
fn rules_run_in_the_documented_order() {
    assert!(
        RULES
            == [
                RULE_LASER_NEVER,
                RULE_VENTILATION,
                RULE_MAINS,
                RULE_ANTI_TIP,
                RULE_WALL_ANCHOR
            ]
    );
}

#[test]
fn tall_furniture_constant_is_1200_mm() {
    assert!((TALL_FURNITURE_MM - 1200.0).abs() < f64::EPSILON);
}

// ---- L1 safety.laser_never ----------------------------------------------------------

#[test]
fn example_01_safety_laser_never_laser_on_pvc_is_unsafe_critical() {
    let facts = step_facts(vec![step(1, &["cut.laser"], Some("pvc"))]);
    assert_eq!(
        evaluate(&facts),
        vec![find!(
            Kind::Unsafe,
            Severity::Critical,
            RULE_LASER_NEVER,
            Location::Step(1),
            "pvc must never be laser-cut",
            "Step 1 cuts pvc with a laser. That releases chlorine gas, which harms you and the laser.",
            Some("step.change_material"),
            [("step", "1"), ("capability", "cut.laser"), ("material", "pvc")]
        )]
    );
}

#[test]
fn example_02_safety_laser_never_laser_on_a_safe_material_is_silent() {
    let facts = step_facts(vec![step(2, &["cut.laser"], Some("ply"))]);
    assert!(evaluate(&facts).is_empty());
}

#[test]
fn example_03_safety_laser_never_laser_on_an_unknown_material_is_silent() {
    let facts = step_facts(vec![step(2, &["cut.laser"], Some("oak"))]);
    assert!(evaluate(&facts).is_empty());
}

#[test]
fn example_04_safety_laser_never_laser_without_a_material_is_silent() {
    let facts = step_facts(vec![step(2, &["cut.laser"], None)]);
    assert!(evaluate(&facts).is_empty());
}

#[test]
fn example_05_safety_laser_never_laser_capability_family_still_counts() {
    let facts = step_facts(vec![step(9, &["cut.laser:thin"], Some("pvc"))]);
    assert_eq!(
        evaluate(&facts),
        vec![find!(
            Kind::Unsafe,
            Severity::Critical,
            RULE_LASER_NEVER,
            Location::Step(9),
            "pvc must never be laser-cut",
            "Step 9 cuts pvc with a laser. That releases chlorine gas, which harms you and the laser.",
            Some("step.change_material"),
            [("step", "9"), ("capability", "cut.laser"), ("material", "pvc")]
        )]
    );
}

#[test]
fn example_06_safety_laser_never_laser_gives_one_finding_per_step() {
    let facts = step_facts(vec![
        step(1, &["cut.laser"], Some("pvc")),
        step(2, &["cut.laser"], Some("pvc")),
    ]);
    let out = evaluate(&facts);
    assert!(out.len() == 2);
    assert!(out[0].location == Location::Step(1));
    assert!(out[1].location == Location::Step(2));
    assert!(out[0].facts[0] == ("step".to_string(), "1".to_string()));
    assert!(out[1].facts[0] == ("step".to_string(), "2".to_string()));
}

// ---- V1 safety.ventilation ----------------------------------------------------------

#[test]
fn example_07_safety_ventilation_paint_in_an_unventilated_space() {
    let mut facts = step_facts(vec![step(3, &["finish.paint"], None)]);
    facts.ventilated = Some(false);
    assert_eq!(
        evaluate(&facts),
        vec![find!(
            Kind::Unsafe,
            Severity::Warning,
            RULE_VENTILATION,
            Location::Step(3),
            "Finishing with finish.paint in an unventilated space",
            "Step 3 applies finish.paint and the space has no ventilation, so the fumes build up. Work outside or ventilate the space.",
            Some("space.ventilate"),
            [("step", "3"), ("capability", "finish.paint")]
        )]
    );
}

#[test]
fn example_08_safety_ventilation_oil_in_an_unventilated_space() {
    let mut facts = step_facts(vec![step(4, &["finish.oil"], None)]);
    facts.ventilated = Some(false);
    assert_eq!(
        evaluate(&facts),
        vec![find!(
            Kind::Unsafe,
            Severity::Warning,
            RULE_VENTILATION,
            Location::Step(4),
            "Finishing with finish.oil in an unventilated space",
            "Step 4 applies finish.oil and the space has no ventilation, so the fumes build up. Work outside or ventilate the space.",
            Some("space.ventilate"),
            [("step", "4"), ("capability", "finish.oil")]
        )]
    );
}

#[test]
fn example_09_safety_ventilation_a_ventilated_space_is_silent() {
    let facts = step_facts(vec![step(3, &["finish.paint"], None)]);
    assert!(facts.ventilated == Some(true));
    assert!(evaluate(&facts).is_empty());
}

#[test]
fn example_10_safety_ventilation_an_unknown_space_is_silent() {
    let mut facts = step_facts(vec![step(3, &["finish.paint"], None)]);
    facts.ventilated = None;
    assert!(evaluate(&facts).is_empty());
}

#[test]
fn example_11_safety_ventilation_the_first_finish_in_needs_order_is_named() {
    let mut facts = step_facts(vec![step(5, &["finish.oil", "finish.paint"], None)]);
    facts.ventilated = Some(false);
    let out = evaluate(&facts);
    assert!(out.len() == 1);
    assert!(out[0].what == "Finishing with finish.oil in an unventilated space");
    assert!(out[0].facts[1] == ("capability".to_string(), "finish.oil".to_string()));
}

// ---- M1 safety.mains ----------------------------------------------------------------

#[test]
fn example_12_safety_mains_mains_at_beginner_skill_needs_an_electrician() {
    let facts = SafetyFacts {
        wiring: true,
        skill: Some(Skill::Beginner),
        ..SafetyFacts::default()
    };
    assert_eq!(
        evaluate(&facts),
        vec![find!(
            Kind::Unsafe,
            Severity::Critical,
            RULE_MAINS,
            Location::Project,
            "Mains wiring at beginner skill",
            "This design has mains-voltage wiring and your skill is beginner. A mains mistake can kill; have a licensed electrician do or check this work.",
            Some("plan.add_electrician"),
            [("wiring", "mains"), ("skill", "beginner")]
        )]
    );
}

#[test]
fn example_13_safety_mains_mains_at_comfortable_skill_is_a_tip() {
    let facts = SafetyFacts {
        wiring: true,
        skill: Some(Skill::Comfortable),
        ..SafetyFacts::default()
    };
    assert_eq!(
        evaluate(&facts),
        vec![find!(
            Kind::Tip,
            Severity::Info,
            RULE_MAINS,
            Location::Project,
            "Mains wiring needs an electrician's check",
            "This design has mains-voltage wiring and your skill is comfortable. Check your local rules and have the work inspected.",
            None,
            [("wiring", "mains"), ("skill", "comfortable")]
        )]
    );
}

#[test]
fn example_14_safety_mains_mains_never_panics_without_wiring() {
    let facts = SafetyFacts {
        wiring: false,
        skill: Some(Skill::Beginner),
        ..SafetyFacts::default()
    };
    assert!(evaluate(&facts).is_empty());
}

#[test]
fn example_15_safety_mains_mains_without_a_known_skill_is_silent() {
    let facts = SafetyFacts {
        wiring: true,
        skill: None,
        ..SafetyFacts::default()
    };
    assert!(evaluate(&facts).is_empty());
}

// ---- A1 safety.anti_tip -------------------------------------------------------------

#[test]
fn example_16_safety_anti_tip_a_tall_dresser_is_not_anchored() {
    let facts = SafetyFacts {
        furniture: vec![furn("dresser", false, 1400.0, true, Some(false))],
        ..SafetyFacts::default()
    };
    assert_eq!(
        evaluate(&facts),
        vec![find!(
            Kind::Unsafe,
            Severity::Warning,
            RULE_ANTI_TIP,
            Location::Entity("dresser".to_string()),
            "dresser can tip over",
            "dresser is not anchored to a wall, and a child can bring it down by climbing it or pulling a drawer. Anchor it to a stud.",
            Some("part.add_anchor"),
            [
                ("part", "dresser"),
                ("child", "false"),
                ("height_mm", "1400"),
                ("drawers", "true")
            ]
        )]
    );
}

#[test]
fn example_17_safety_anti_tip_a_short_cabinet_is_silent() {
    let facts = SafetyFacts {
        furniture: vec![furn("cabinet", false, 900.0, true, Some(false))],
        ..SafetyFacts::default()
    };
    assert!(evaluate(&facts).is_empty());
}

#[test]
fn example_18_safety_anti_tip_a_childs_piece_is_flagged_however_short() {
    let facts = SafetyFacts {
        furniture: vec![furn("crib", true, 700.0, false, Some(false))],
        ..SafetyFacts::default()
    };
    assert_eq!(
        evaluate(&facts),
        vec![find!(
            Kind::Unsafe,
            Severity::Warning,
            RULE_ANTI_TIP,
            Location::Entity("crib".to_string()),
            "crib can tip over",
            "crib is not anchored to a wall, and a child can bring it down by climbing it or pulling a drawer. Anchor it to a stud.",
            Some("part.add_anchor"),
            [
                ("part", "crib"),
                ("child", "true"),
                ("height_mm", "700"),
                ("drawers", "false")
            ]
        )]
    );
}

#[test]
fn example_19_safety_anti_tip_an_anchored_dresser_is_silent() {
    let facts = SafetyFacts {
        furniture: vec![furn("dresser", false, 1400.0, true, Some(true))],
        ..SafetyFacts::default()
    };
    assert!(evaluate(&facts).is_empty());
}

#[test]
fn example_20_safety_anti_tip_an_unknown_anchor_is_silent() {
    let facts = SafetyFacts {
        furniture: vec![furn("table", true, 700.0, false, None)],
        ..SafetyFacts::default()
    };
    assert!(evaluate(&facts).is_empty());
}

#[test]
fn example_21_safety_anti_tip_the_constant_itself_is_not_taller() {
    let facts = SafetyFacts {
        furniture: vec![furn("cabinet", false, TALL_FURNITURE_MM, true, Some(false))],
        ..SafetyFacts::default()
    };
    assert!(evaluate(&facts).is_empty());
}

#[test]
fn example_22_safety_anti_tip_a_nan_height_is_not_taller() {
    let facts = SafetyFacts {
        furniture: vec![
            furn("nan", false, f64::NAN, true, Some(false)),
            furn("inf", false, f64::INFINITY, true, Some(false)),
        ],
        ..SafetyFacts::default()
    };
    assert!(evaluate(&facts).is_empty());
}

// ---- W1 safety.wall_anchor ----------------------------------------------------------

#[test]
fn example_23_safety_wall_anchor_a_load_over_the_rating_is_critical() {
    let facts = SafetyFacts {
        wall_parts: vec![wall_part("tv", 30.0, Some(25.0), Wall::Stud)],
        ..SafetyFacts::default()
    };
    assert_eq!(
        evaluate(&facts),
        vec![find!(
            Kind::Unsafe,
            Severity::Critical,
            RULE_WALL_ANCHOR,
            Location::Entity("tv".to_string()),
            "tv is heavier than its anchor",
            "tv weighs 30 kg and its anchor is rated 25 kg. Use a stronger fixing or lighten the part.",
            Some("part.change_anchor"),
            [
                ("part", "tv"),
                ("load_kg", "30"),
                ("anchor_rated_kg", "25")
            ]
        )]
    );
}

#[test]
fn example_24_safety_wall_anchor_a_load_within_the_rating_is_silent() {
    let facts = SafetyFacts {
        wall_parts: vec![wall_part("shelf", 10.0, Some(25.0), Wall::Masonry)],
        ..SafetyFacts::default()
    };
    assert!(evaluate(&facts).is_empty());
}

#[test]
fn example_25_safety_wall_anchor_a_load_equal_to_the_rating_is_fine() {
    let facts = SafetyFacts {
        wall_parts: vec![wall_part("shelf", 25.0, Some(25.0), Wall::Stud)],
        ..SafetyFacts::default()
    };
    assert!(evaluate(&facts).is_empty());
}

#[test]
fn example_26_safety_wall_anchor_an_unknown_wall_is_a_warning() {
    let facts = SafetyFacts {
        wall_parts: vec![wall_part("mirror", 5.0, Some(50.0), Wall::Unknown)],
        ..SafetyFacts::default()
    };
    assert_eq!(
        evaluate(&facts),
        vec![find!(
            Kind::Unsafe,
            Severity::Warning,
            RULE_WALL_ANCHOR,
            Location::Entity("mirror".to_string()),
            "mirror is on a wall of unknown type",
            "mirror is fixed to a wall whose type is unknown, so its anchor cannot be checked. Find the studs or the substrate first.",
            Some("part.confirm_wall"),
            [("part", "mirror"), ("wall", "unknown")]
        )]
    );
}

#[test]
fn example_27_safety_wall_anchor_overload_wins_over_an_unknown_wall() {
    let facts = SafetyFacts {
        wall_parts: vec![wall_part("shelf", 40.0, Some(25.0), Wall::Unknown)],
        ..SafetyFacts::default()
    };
    let out = evaluate(&facts);
    assert!(out.len() == 1);
    assert!(out[0].severity == Severity::Critical);
    assert!(out[0].what == "shelf is heavier than its anchor");
}

#[test]
fn example_28_safety_wall_anchor_a_missing_rating_on_a_known_wall_is_silent() {
    let facts = SafetyFacts {
        wall_parts: vec![
            wall_part("hook", 30.0, None, Wall::Stud),
            wall_part("nan", f64::NAN, Some(25.0), Wall::Masonry),
            wall_part("inf", f64::INFINITY, Some(25.0), Wall::Masonry),
        ],
        ..SafetyFacts::default()
    };
    assert!(evaluate(&facts).is_empty());
}

// ---- E1 evaluate: order, silence and no panics --------------------------------------

#[test]
fn example_29_every_rule_in_the_documented_order() {
    let facts = SafetyFacts {
        steps: vec![
            step(1, &["cut.laser"], Some("pvc")),
            step(2, &["finish.paint"], None),
        ],
        materials: vec![mat("pvc", true)],
        ventilated: Some(false),
        wiring: true,
        skill: Some(Skill::Beginner),
        furniture: vec![furn("dresser", false, 1400.0, true, Some(false))],
        wall_parts: vec![wall_part("tv", 30.0, Some(25.0), Wall::Stud)],
    };
    let out = evaluate(&facts);
    assert!(out.len() == 5);
    assert!(out[0].source == Source::Rule(RULE_LASER_NEVER.to_string()));
    assert!(out[0].location == Location::Step(1));
    assert!(out[1].source == Source::Rule(RULE_VENTILATION.to_string()));
    assert!(out[1].location == Location::Step(2));
    assert!(out[2].source == Source::Rule(RULE_MAINS.to_string()));
    assert!(out[2].location == Location::Project);
    assert!(out[3].source == Source::Rule(RULE_ANTI_TIP.to_string()));
    assert!(out[3].location == Location::Entity("dresser".to_string()));
    assert!(out[4].source == Source::Rule(RULE_WALL_ANCHOR.to_string()));
    assert!(out[4].location == Location::Entity("tv".to_string()));
}

#[test]
fn example_30_empty_facts_give_no_findings() {
    assert!(evaluate(&SafetyFacts::default()).is_empty());
}

#[test]
fn example_31_one_step_with_two_rules_gives_both() {
    let mut facts = step_facts(vec![step(6, &["cut.laser", "finish.oil"], Some("pvc"))]);
    facts.ventilated = Some(false);
    let out = evaluate(&facts);
    assert!(out.len() == 2);
    assert!(out[0].source == Source::Rule(RULE_LASER_NEVER.to_string()));
    assert!(out[1].source == Source::Rule(RULE_VENTILATION.to_string()));
    assert!(out[1].facts[1] == ("capability".to_string(), "finish.oil".to_string()));
}

#[test]
fn example_32_repeated_capabilities_give_one_finding() {
    let mut facts = step_facts(vec![step(8, &["finish.oil", "finish.oil"], None)]);
    facts.ventilated = Some(false);
    assert!(evaluate(&facts).len() == 1);
}

#[test]
fn example_33_nonsense_never_panics() {
    let facts = SafetyFacts {
        steps: vec![
            step(0, &[], None),
            step(u32::MAX, &["cut.laser"], Some("ghost")),
            step(7, &["finish.paint"], Some("pvc")),
        ],
        materials: Vec::new(),
        ventilated: Some(false),
        wiring: false,
        skill: None,
        furniture: vec![
            furn("", false, f64::NEG_INFINITY, true, Some(false)),
            furn("", true, f64::NAN, true, None),
        ],
        wall_parts: vec![
            wall_part("", f64::NAN, Some(f64::NAN), Wall::Unknown),
            wall_part("", f64::NEG_INFINITY, Some(f64::INFINITY), Wall::Masonry),
        ],
    };
    let out = evaluate(&facts);
    assert!(out.len() == 2);
    assert!(out[0].source == Source::Rule(RULE_VENTILATION.to_string()));
    assert!(out[1].source == Source::Rule(RULE_WALL_ANCHOR.to_string()));
}
