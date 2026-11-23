//! Contract tests for mk_assist::observations, written by Gordon as a PA (the Crew) from
//! crew/contracts/mk_observations.md. They were the hidden tests of campaign mk-core-32.

// Crew-written tests: a helper that builds a whole `Finding` takes one argument per
// field, and worked rows print constants to the contract's digits; neither matters in a
// test.
#![allow(clippy::too_many_arguments, clippy::approx_constant)]

use mk_assist::observations::{
    evaluate, family, mentions, qualifier, usable, Access, Kind, Observation, ObservationFacts,
    Scale, Tool, NEEDED_CUTS, RULES, RULE_HOUSEHOLD_KIT, RULE_JOINERY, RULE_KNOCK_DOWN,
    RULE_MATERIALS,
};

fn t(kind: &str, capability: &str, access: Access) -> Tool {
    Tool {
        kind: kind.to_string(),
        capability: capability.to_string(),
        access,
    }
}

fn facts(
    goal: &str,
    scale: Option<Scale>,
    tools: Vec<Tool>,
    materials: Vec<&str>,
) -> ObservationFacts {
    ObservationFacts {
        goal: goal.to_string(),
        scale,
        tools,
        materials: materials.into_iter().map(|m| m.to_string()).collect(),
    }
}

fn obs(rule: &'static str, kind: Kind, text: &str, why: &str) -> Observation {
    Observation {
        rule,
        kind,
        text: text.to_string(),
        why: why.to_string(),
    }
}

fn o1(kind: &str, cap: &str, word: &str) -> Observation {
    obs(
        RULE_JOINERY,
        Kind::Advice,
        &format!(
            "You listed a {} but no tool for {}; I'll favour joinery a {} can do.",
            kind, cap, kind
        ),
        &format!(
            "The goal names a {}, which needs {}, and the {} you listed only does it rough.",
            word, cap, kind
        ),
    )
}

fn o2() -> Observation {
    obs(
        RULE_KNOCK_DOWN,
        Kind::Question,
        "Should I prefer knock-down joints (cam locks) so it comes apart for moving?",
        "The goal mentions moving or coming apart, so knock-down joints are worth asking about.",
    )
}

fn o3() -> Observation {
    obs(
        RULE_HOUSEHOLD_KIT,
        Kind::Tip,
        "No tools listed: the basic household kit is the starting point (tape, utility knife, screwdrivers, hammer, scissors).",
        "You have not listed any tools yet, so start from the basic household kit.",
    )
}

fn o4() -> Observation {
    obs(
        RULE_MATERIALS,
        Kind::Question,
        "I can't check material use yet - tell me what you have?",
        "No materials are listed, so material use is unknown.",
    )
}

fn jig() -> Tool {
    t("jigsaw", "cut.rip:rough", Access::Owned)
}

fn circ() -> Tool {
    t("circular saw", "cut.crosscut:rough", Access::Owned)
}

fn saw() -> Tool {
    t("table saw", "cut.rip", Access::Owned)
}

fn mitre() -> Tool {
    t("mitre saw", "cut.crosscut", Access::Owned)
}

fn lathe() -> Tool {
    t("lathe", "cut.rip", Access::Wishlist)
}

#[test]
fn family_splits_at_first_colon() {
    assert!(family("cut.straight:rough") == "cut.straight");
    assert!(family("cut.rip") == "cut.rip");
    assert!(family("a:b:c") == "a");
    assert!(family(":only").is_empty());
    assert!(family("").is_empty());
    assert!(family("é:r") == "é");
}

#[test]
fn qualifier_is_text_after_first_colon() {
    assert!(qualifier("cut.straight:rough") == "rough");
    assert!(qualifier("cut.rip").is_empty());
    assert!(qualifier("a:b:c") == "b:c");
    assert!(qualifier(":only") == "only");
    assert!(qualifier("é:r") == "r");
}

#[test]
fn usable_follows_access() {
    for access in [
        Access::Owned,
        Access::Shared,
        Access::Makerspace,
        Access::Rentable,
    ] {
        assert!(usable(access));
    }
    assert!(!usable(Access::Wishlist));
}

#[test]
fn rule_ids_and_needed_cuts_are_exact() {
    assert!(
        RULES
            == [
                RULE_JOINERY,
                RULE_KNOCK_DOWN,
                RULE_HOUSEHOLD_KIT,
                RULE_MATERIALS
            ]
    );
    assert!(NEEDED_CUTS == ["cut.rip", "cut.crosscut"]);
    assert!(RULE_JOINERY == "obs.joinery");
    assert!(RULE_KNOCK_DOWN == "obs.knock_down");
    assert!(RULE_HOUSEHOLD_KIT == "obs.household_kit");
    assert!(RULE_MATERIALS == "obs.materials");
}

#[test]
fn mentions_matches_whole_words_only() {
    assert!(mentions("please move it", "move"));
    assert!(mentions("MOVE it", "move"));
    assert!(!mentions("remove the shelf", "move"));
    assert!(!mentions("moving", "move"));
    assert!(!mentions("moved", "move"));
    assert!(!mentions("", "move"));
}

#[test]
fn mentions_allows_simple_plurals() {
    assert!(mentions("three shelves", "shelf"));
    assert!(mentions("two desks", "desk"));
    assert!(mentions("boxes", "box"));
    assert!(mentions("cabinets", "cabinet"));
    assert!(mentions("dressers", "dresser"));
    assert!(mentions("ponies", "pony"));
    assert!(mentions("shelfs", "shelf"));
    assert!(mentions("shelfes", "shelf"));
    assert!(mentions("knives", "knife"));
}

#[test]
fn mentions_matches_phrases_and_hyphenated_words() {
    assert!(mentions("it must come apart", "come apart"));
    assert!(mentions("flat-packed desk", "flat packed"));
    assert!(mentions("flat packed desk", "flat-packed"));
    assert!(mentions("two flat packs", "flat pack"));
    assert!(!mentions("it comes apart", "come apart"));
    assert!(mentions("it comes apart", "comes apart"));
}

#[test]
fn mentions_empty_needle_matches_nothing() {
    assert!(!mentions("anything at all", ""));
    assert!(!mentions("anything at all", " - "));
    assert!(!mentions("", ""));
}

#[test]
fn o1_joinery_rough_rip_names_kind_and_missing_cut() {
    let f = facts(
        "a dresser",
        Some(Scale::Room),
        vec![jig()],
        vec!["15 pine 2x4 3ft"],
    );
    assert!(evaluate(&f) == vec![o1("jigsaw", "cut.rip", "dresser")]);
}

#[test]
fn o1_joinery_two_rough_tools_give_rip_then_crosscut() {
    let f = facts(
        "a dresser",
        Some(Scale::Room),
        vec![jig(), circ()],
        vec!["15 pine 2x4 3ft"],
    );
    assert!(
        evaluate(&f)
            == vec![
                o1("jigsaw", "cut.rip", "dresser"),
                o1("circular saw", "cut.crosscut", "dresser")
            ]
    );
}

#[test]
fn o1_joinery_quiet_when_exact_tools_exist() {
    let f = facts(
        "a desk",
        Some(Scale::Room),
        vec![saw(), mitre()],
        vec!["half a sheet of 3/4 ply"],
    );
    assert!(evaluate(&f).is_empty());
}

#[test]
fn o1_joinery_quiet_on_bench_and_body_scale() {
    assert!(evaluate(&facts(
        "a cabinet",
        Some(Scale::Bench),
        vec![jig(), circ()],
        vec!["pine"],
    ))
    .is_empty());
    assert!(evaluate(&facts(
        "a cabinet",
        Some(Scale::Body),
        vec![jig(), circ()],
        vec!["pine"],
    ))
    .is_empty());
    assert!(
        evaluate(&facts(
            "a cabinet",
            Some(Scale::Room),
            vec![jig(), circ()],
            vec!["pine"],
        ))
        .len()
            == 2
    );
}

#[test]
fn o1_joinery_quiet_on_whole_words_only() {
    let f = facts("a bookshelf", Some(Scale::Room), vec![jig()], vec!["pine"]);
    assert!(evaluate(&f).is_empty());
}

#[test]
fn o1_joinery_ignores_wishlist_tools() {
    assert!(evaluate(&facts(
        "a shelf",
        Some(Scale::Room),
        vec![lathe()],
        vec!["pine"],
    ))
    .is_empty());
    assert!(evaluate(&facts(
        "a shelf",
        Some(Scale::Room),
        vec![t("jigsaw", "cut.rip:rough", Access::Wishlist)],
        vec!["pine"],
    ))
    .is_empty());
}

#[test]
fn o1_joinery_quiet_without_a_rough_substitute() {
    let f = facts(
        "a desk",
        Some(Scale::Room),
        vec![t("drill", "drill.hole", Access::Owned)],
        vec!["pine"],
    );
    assert!(evaluate(&f).is_empty());
}

#[test]
fn o1_joinery_shared_tools_count_as_usable() {
    let exact = facts(
        "a shelf",
        Some(Scale::Room),
        vec![t("track saw", "cut.rip", Access::Shared)],
        vec!["pine"],
    );
    assert!(evaluate(&exact).is_empty());
    let rough = facts(
        "a shelf",
        Some(Scale::Room),
        vec![t("track saw", "cut.rip:rough", Access::Shared)],
        vec!["pine"],
    );
    assert!(evaluate(&rough) == vec![o1("track saw", "cut.rip", "shelf")]);
}

#[test]
fn o1_joinery_qualifier_is_case_sensitive_and_whole() {
    let upper = facts(
        "a shelf",
        Some(Scale::Room),
        vec![t("jigsaw", "cut.rip:Rough", Access::Owned)],
        vec!["pine"],
    );
    assert!(evaluate(&upper).is_empty());
    let extra = facts(
        "a shelf",
        Some(Scale::Room),
        vec![t("jigsaw", "cut.rip:rough:extra", Access::Owned)],
        vec!["pine"],
    );
    assert!(evaluate(&extra).is_empty());
}

#[test]
fn o2_knock_down_comes_apart_is_a_question() {
    let f = facts(
        "a chair that must come apart for moving",
        Some(Scale::Room),
        vec![saw(), mitre()],
        vec!["pine"],
    );
    assert!(evaluate(&f) == vec![o2()]);
}

#[test]
fn o2_knock_down_flat_packed() {
    let f = facts(
        "a flat-packed desk",
        Some(Scale::Room),
        vec![saw(), mitre()],
        vec!["half a sheet of 3/4 ply"],
    );
    assert!(evaluate(&f) == vec![o2()]);
}

#[test]
fn o2_knock_down_portable_and_movable() {
    assert!(
        evaluate(&facts(
            "a portable bench",
            Some(Scale::Bench),
            vec![],
            vec![]
        )) == vec![o2(), o3(), o4()]
    );
    assert!(
        evaluate(&facts(
            "a movable desk",
            Some(Scale::Room),
            vec![saw(), mitre()],
            vec!["pine"],
        )) == vec![o2()]
    );
}

#[test]
fn o2_knock_down_quiet_without_moving_words() {
    let f = facts("a birdhouse", Some(Scale::Bench), vec![jig()], vec!["pine"]);
    assert!(evaluate(&f).is_empty());
}

#[test]
fn o3_household_kit_no_tools_is_a_tip() {
    let f = facts("a birdhouse", Some(Scale::Bench), vec![], vec!["pine"]);
    assert!(evaluate(&f) == vec![o3()]);
}

#[test]
fn o3_household_kit_quiet_when_only_a_wishlist_tool_is_listed() {
    let f = facts(
        "a birdhouse",
        Some(Scale::Bench),
        vec![lathe()],
        vec!["pine"],
    );
    assert!(evaluate(&f).is_empty());
}

#[test]
fn o4_materials_no_materials_is_a_question() {
    let f = facts("a birdhouse", Some(Scale::Bench), vec![jig()], vec![]);
    assert!(evaluate(&f) == vec![o4()]);
}

#[test]
fn o4_materials_quiet_when_materials_are_listed() {
    let f = facts(
        "a birdhouse",
        Some(Scale::Bench),
        vec![jig()],
        vec!["some plywood"],
    );
    assert!(evaluate(&f).is_empty());
}

#[test]
fn o4_materials_blank_entries_do_not_count() {
    let f = facts(
        "a birdhouse",
        Some(Scale::Bench),
        vec![jig()],
        vec!["", "   "],
    );
    assert!(evaluate(&f) == vec![o4()]);
}

#[test]
fn evaluate_emits_in_rule_order() {
    let f = facts(
        "a dresser that must come apart",
        Some(Scale::Room),
        vec![jig(), circ()],
        vec![],
    );
    assert!(
        evaluate(&f)
            == vec![
                o1("jigsaw", "cut.rip", "dresser"),
                o1("circular saw", "cut.crosscut", "dresser"),
                o2(),
                o4(),
            ]
    );
}

#[test]
fn evaluate_empty_goal_gives_only_kit_and_materials() {
    let f = facts("", None, vec![], vec![]);
    assert!(evaluate(&f) == vec![o3(), o4()]);
}

#[test]
fn worked_examples_all_rows() {
    let cases: Vec<(ObservationFacts, Vec<Observation>)> = vec![
        (
            facts(
                "a dresser",
                Some(Scale::Room),
                vec![jig()],
                vec!["15 pine 2x4 3ft"],
            ),
            vec![o1("jigsaw", "cut.rip", "dresser")],
        ),
        (
            facts(
                "a dresser",
                Some(Scale::Room),
                vec![jig(), circ()],
                vec!["15 pine 2x4 3ft"],
            ),
            vec![
                o1("jigsaw", "cut.rip", "dresser"),
                o1("circular saw", "cut.crosscut", "dresser"),
            ],
        ),
        (
            facts("a bookshelf", Some(Scale::Room), vec![jig()], vec!["pine"]),
            vec![],
        ),
        (
            facts(
                "a desk",
                Some(Scale::Room),
                vec![saw(), mitre()],
                vec!["ply"],
            ),
            vec![],
        ),
        (
            facts(
                "a cabinet",
                Some(Scale::Bench),
                vec![jig(), circ()],
                vec!["pine"],
            ),
            vec![],
        ),
        (
            facts("a shelf", Some(Scale::Room), vec![lathe()], vec!["pine"]),
            vec![],
        ),
        (
            facts(
                "a chair that must come apart for moving",
                Some(Scale::Room),
                vec![saw(), mitre()],
                vec!["pine"],
            ),
            vec![o2()],
        ),
        (
            facts(
                "a flat-packed desk",
                Some(Scale::Room),
                vec![saw(), mitre()],
                vec!["ply"],
            ),
            vec![o2()],
        ),
        (
            facts("a portable bench", Some(Scale::Bench), vec![], vec![]),
            vec![o2(), o3(), o4()],
        ),
        (
            facts("a birdhouse", Some(Scale::Bench), vec![], vec![]),
            vec![o3(), o4()],
        ),
        (
            facts(
                "a birdhouse",
                Some(Scale::Bench),
                vec![jig(), circ()],
                vec![],
            ),
            vec![o4()],
        ),
        (
            facts("a birdhouse", Some(Scale::Bench), vec![jig()], vec!["pine"]),
            vec![],
        ),
        (
            facts(
                "a dresser that must come apart",
                Some(Scale::Room),
                vec![jig(), circ()],
                vec![],
            ),
            vec![
                o1("jigsaw", "cut.rip", "dresser"),
                o1("circular saw", "cut.crosscut", "dresser"),
                o2(),
                o4(),
            ],
        ),
        (facts("", None, vec![], vec![]), vec![o3(), o4()]),
        (
            facts(
                "a movable desk",
                Some(Scale::Room),
                vec![saw(), mitre()],
                vec!["pine"],
            ),
            vec![o2()],
        ),
        (
            facts(
                "a shelf",
                None,
                vec![t("track saw", "cut.rip", Access::Shared)],
                vec!["pine"],
            ),
            vec![],
        ),
    ];
    for (i, (f, want)) in cases.iter().enumerate() {
        assert!(
            evaluate(f).as_slice() == want.as_slice(),
            "worked example row {}",
            i + 1
        );
    }
}

#[test]
fn no_panic_on_odd_input() {
    let odd = facts(
        "",
        None,
        vec![
            t("", "", Access::Owned),
            t("jigsaw", ":", Access::Owned),
            t("x", "cut.rip:rough:extra", Access::Owned),
            t("y", "cut.rip:Rough", Access::Owned),
            t("z", "cut.rip", Access::Wishlist),
        ],
        vec![""],
    );
    let _ = evaluate(&odd);

    let many = facts(
        "shelf shelf shelf",
        Some(Scale::Room),
        vec![t("jigsaw", "cut.rip:rough", Access::Owned)],
        vec!["\u{0}"],
    );
    assert!(evaluate(&many).len() == 1);

    let unicode = facts("ℵ☃ – ﬀ", None, vec![], vec!["\u{0}"]);
    let _ = evaluate(&unicode);

    assert!(evaluate(&ObservationFacts::default()).len() == 2);
}
