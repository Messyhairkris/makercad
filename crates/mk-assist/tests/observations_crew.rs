//! Tests for mk_assist::observations written by Kimi (the Crew, campaign mk-core-32) from the
//! contract alone, without seeing the module.

//! Integration tests for Sancho's deterministic early observations.

// Crew-written tests: a helper that builds a whole `Finding` takes one argument per
// field, and worked rows print constants to the contract's digits; neither matters in a
// test.
#![allow(clippy::too_many_arguments, clippy::approx_constant)]

use mk_assist::observations::{
    evaluate, family, mentions, qualifier, usable, Access, Kind, Observation, ObservationFacts,
    Scale, Tool, NEEDED_CUTS, RULES, RULE_HOUSEHOLD_KIT, RULE_JOINERY, RULE_KNOCK_DOWN,
    RULE_MATERIALS,
};

fn tool(kind: &str, capability: &str, access: Access) -> Tool {
    Tool {
        kind: kind.to_owned(),
        capability: capability.to_owned(),
        access,
    }
}

fn owned_tool(kind: &str, capability: &str) -> Tool {
    tool(kind, capability, Access::Owned)
}

fn jig() -> Tool {
    owned_tool("jigsaw", "cut.rip:rough")
}

fn circ() -> Tool {
    owned_tool("circular saw", "cut.crosscut:rough")
}

fn saw() -> Tool {
    owned_tool("table saw", "cut.rip")
}

fn mitre() -> Tool {
    owned_tool("mitre saw", "cut.crosscut")
}

fn lathe() -> Tool {
    tool("lathe", "cut.rip", Access::Wishlist)
}

fn track() -> Tool {
    tool("track saw", "cut.rip", Access::Shared)
}

fn facts(
    goal: &str,
    scale: Option<Scale>,
    tools: Vec<Tool>,
    materials: &[&str],
) -> ObservationFacts {
    ObservationFacts {
        goal: goal.to_owned(),
        scale,
        tools,
        materials: materials
            .iter()
            .map(|material| (*material).to_owned())
            .collect(),
    }
}

fn o1(kind: &str, capability: &str, word: &str) -> Observation {
    Observation {
        rule: RULE_JOINERY,
        kind: Kind::Advice,
        text: format!(
            concat!(
                "You listed a {} but no tool for {}; ",
                "I'll favour joinery a {} can do."
            ),
            kind, capability, kind
        ),
        why: format!(
            "The goal names a {}, which needs {}, and the {} you listed only does it rough.",
            word, capability, kind
        ),
    }
}

fn o2() -> Observation {
    Observation {
        rule: RULE_KNOCK_DOWN,
        kind: Kind::Question,
        text: "Should I prefer knock-down joints (cam locks) so it comes apart for moving?"
            .to_owned(),
        why: concat!(
            "The goal mentions moving or coming apart, ",
            "so knock-down joints are worth asking about."
        )
        .to_owned(),
    }
}

fn o3() -> Observation {
    Observation {
        rule: RULE_HOUSEHOLD_KIT,
        kind: Kind::Tip,
        text: concat!(
            "No tools listed: the basic household kit is the starting point ",
            "(tape, utility knife, screwdrivers, hammer, scissors)."
        )
        .to_owned(),
        why: "You have not listed any tools yet, so start from the basic household kit.".to_owned(),
    }
}

fn o4() -> Observation {
    Observation {
        rule: RULE_MATERIALS,
        kind: Kind::Question,
        text: "I can't check material use yet - tell me what you have?".to_owned(),
        why: "No materials are listed, so material use is unknown.".to_owned(),
    }
}

#[test]
fn a1_access_marks_only_tools_usable_today() {
    assert!(usable(Access::Owned));
    assert!(usable(Access::Shared));
    assert!(usable(Access::Makerspace));
    assert!(usable(Access::Rentable));
    assert!(!usable(Access::Wishlist));
}

#[test]
fn a2_family_and_qualifier_split_at_the_first_colon() {
    assert_eq!(family("cut.straight:rough"), "cut.straight");
    assert_eq!(qualifier("cut.straight:rough"), "rough");
    assert_eq!(family("cut.rip"), "cut.rip");
    assert_eq!(qualifier("cut.rip"), "");
    assert_eq!(family("cut.rip:rough:extra"), "cut.rip");
    assert_eq!(qualifier("cut.rip:rough:extra"), "rough:extra");
    assert_eq!(family(""), "");
    assert_eq!(qualifier(""), "");
    assert_eq!(family(":rough"), "");
    assert_eq!(qualifier(":rough"), "rough");
}

#[test]
fn a2_only_the_whole_lowercase_rough_qualifier_is_not_exact() {
    for capability in [
        "cut.rip",
        "cut.rip:fine",
        "cut.rip:Rough",
        "cut.rip:rough:extra",
    ] {
        let input = facts(
            "a dresser",
            Some(Scale::Room),
            vec![owned_tool("odd saw", capability)],
            &["15 pine 2x4 3ft"],
        );
        assert_eq!(evaluate(&input), vec![]);
    }
}

#[test]
fn a3_words_are_consecutive_case_insensitive_whole_runs_with_simple_plurals() {
    assert!(mentions("shelves", "shelf"));
    assert!(mentions("cats", "cat"));
    assert!(mentions("boxes", "box"));
    assert!(mentions("babies", "baby"));
    assert!(mentions("wolves", "wolf"));
    assert!(mentions("knives", "knife"));
    assert!(!mentions("FLAT-packed", "flat pack"));
    assert!(mentions("It is PORTABLE", "portable"));
    assert!(!mentions("comes apart", "come apart"));
    assert!(!mentions("come now apart", "come apart"));
    assert!(!mentions("remove", "move"));
    assert!(!mentions("a bookshelf", "shelf"));
    assert!(!mentions("shelving", "shelf"));
    assert!(!mentions("anything", "---"));
    assert!(!mentions("", "shelf"));
    assert!(!mentions("", ""));
}

#[test]
fn a4_scale_gate_allows_o1_only_for_room_or_missing_scale() {
    for scale in [Scale::Bench, Scale::Body] {
        let input = facts(
            "a dresser",
            Some(scale),
            vec![jig(), circ()],
            &["15 pine 2x4 3ft"],
        );
        assert_eq!(evaluate(&input), vec![]);
    }
}

#[test]
fn e1_evaluate_uses_declared_rule_and_needed_cut_order() {
    assert_eq!(
        RULES,
        [
            RULE_JOINERY,
            RULE_KNOCK_DOWN,
            RULE_HOUSEHOLD_KIT,
            RULE_MATERIALS
        ]
    );
    assert_eq!(NEEDED_CUTS, ["cut.rip", "cut.crosscut"]);

    let input = facts(
        "a dresser that must come apart",
        Some(Scale::Room),
        vec![jig(), circ()],
        &[],
    );
    assert_eq!(
        evaluate(&input),
        vec![
            o1("jigsaw", "cut.rip", "dresser"),
            o1("circular saw", "cut.crosscut", "dresser"),
            o2(),
            o4(),
        ]
    );
}

#[test]
fn o1_any_exact_usable_tool_suppresses_a_rough_observation() {
    let input = facts(
        "a dresser",
        Some(Scale::Room),
        vec![jig(), saw()],
        &["15 pine 2x4 3ft"],
    );
    assert_eq!(evaluate(&input), vec![]);
}

#[test]
fn o1_uses_the_first_furniture_word_in_the_goal_and_its_list_spelling() {
    let cabinet_first = facts(
        "a cabinet beside a desk",
        Some(Scale::Room),
        vec![jig()],
        &["15 pine 2x4 3ft"],
    );
    assert_eq!(
        evaluate(&cabinet_first),
        vec![o1("jigsaw", "cut.rip", "cabinet")]
    );

    let plural_shelf = facts(
        "shelves",
        Some(Scale::Room),
        vec![jig()],
        &["15 pine 2x4 3ft"],
    );
    assert_eq!(
        evaluate(&plural_shelf),
        vec![o1("jigsaw", "cut.rip", "shelf")]
    );
}

#[test]
fn o1_empty_tool_kind_is_used_verbatim_without_guessing() {
    let input = facts(
        "a dresser",
        Some(Scale::Room),
        vec![owned_tool("", "cut.rip:rough")],
        &["15 pine 2x4 3ft"],
    );
    assert_eq!(evaluate(&input), vec![o1("", "cut.rip", "dresser")]);
}

#[test]
fn o2_every_listed_word_or_phrase_emits_exactly_one_question() {
    for phrase in [
        "come apart",
        "comes apart",
        "take apart",
        "takes apart",
        "disassemble",
        "dismantle",
        "knock down",
        "knockdown",
        "flat pack",
        "flatpack",
        "flat packed",
        "portable",
        "move",
        "moved",
        "moving",
        "movable",
        "moveable",
        "transport",
    ] {
        let input = facts(
            phrase,
            Some(Scale::Bench),
            vec![saw()],
            &["15 pine 2x4 3ft"],
        );
        assert_eq!(evaluate(&input), vec![o2()]);
    }
}

#[test]
fn o3_a_list_containing_only_wishlist_tools_is_not_empty() {
    let input = facts("a birdhouse", Some(Scale::Bench), vec![lathe()], &[]);
    assert_eq!(evaluate(&input), vec![o4()]);
}

#[test]
fn o4_blank_materials_do_not_count_but_lazy_precision_does() {
    let blank = facts(
        "a birdhouse",
        Some(Scale::Bench),
        vec![jig()],
        &["", "  ", "\t\n"],
    );
    assert_eq!(evaluate(&blank), vec![o4()]);

    let listed = facts(
        "a birdhouse",
        Some(Scale::Bench),
        vec![jig()],
        &["", "some plywood", "  "],
    );
    assert_eq!(evaluate(&listed), vec![]);
}

#[test]
fn e1_nan_infinities_empty_capabilities_and_unknown_ids_never_panic_or_guess() {
    let input = ObservationFacts {
        goal: "NaN inf -inf infinity \0 ???".to_owned(),
        scale: None,
        tools: vec![
            owned_tool("", ""),
            tool("unknown-id", "cut.unknown:rough", Access::Wishlist),
        ],
        materials: vec![" \t\n".to_owned()],
    };
    assert_eq!(evaluate(&input), vec![o4()]);
}

#[test]
fn o1_worked_example_1_rough_rip_only() {
    let input = facts(
        "a dresser",
        Some(Scale::Room),
        vec![jig()],
        &["15 pine 2x4 3ft"],
    );
    assert_eq!(evaluate(&input), vec![o1("jigsaw", "cut.rip", "dresser")]);
}

#[test]
fn o1_worked_example_2_two_rough_substitutes_follow_needed_cut_order() {
    let input = facts(
        "a dresser",
        Some(Scale::Room),
        vec![jig(), circ()],
        &["15 pine 2x4 3ft"],
    );
    assert_eq!(
        evaluate(&input),
        vec![
            o1("jigsaw", "cut.rip", "dresser"),
            o1("circular saw", "cut.crosscut", "dresser"),
        ]
    );
}

#[test]
fn a3_worked_example_3_bookshelf_is_not_the_whole_word_shelf() {
    let input = facts(
        "a bookshelf",
        Some(Scale::Room),
        vec![jig()],
        &["15 pine 2x4 3ft"],
    );
    assert_eq!(evaluate(&input), vec![]);
}

#[test]
fn o1_worked_example_4_exact_rip_and_crosscut_tools_emit_nothing() {
    let input = facts(
        "a desk",
        Some(Scale::Room),
        vec![saw(), mitre()],
        &["half a sheet of 3/4 ply"],
    );
    assert_eq!(evaluate(&input), vec![]);
}

#[test]
fn a4_worked_example_5_bench_scale_blocks_cabinet_joinery_advice() {
    let input = facts(
        "a cabinet",
        Some(Scale::Bench),
        vec![jig(), circ()],
        &["15 pine 2x4 3ft"],
    );
    assert_eq!(evaluate(&input), vec![]);
}

#[test]
fn a1_worked_example_6_wishlist_lathe_is_not_usable() {
    let input = facts(
        "a shelf",
        Some(Scale::Room),
        vec![lathe()],
        &["15 pine 2x4 3ft"],
    );
    assert_eq!(evaluate(&input), vec![]);
}

#[test]
fn o2_worked_example_7_come_apart_for_moving_asks_one_question() {
    let input = facts(
        "a chair that must come apart for moving",
        Some(Scale::Room),
        vec![saw(), mitre()],
        &["15 pine 2x4 3ft"],
    );
    assert_eq!(evaluate(&input), vec![o2()]);
}

#[test]
fn o2_worked_example_8_flat_packed_is_two_matching_runs() {
    let input = facts(
        "a flat-packed desk",
        Some(Scale::Room),
        vec![saw(), mitre()],
        &["half a sheet of 3/4 ply"],
    );
    assert_eq!(evaluate(&input), vec![o2()]);
}

#[test]
fn e1_worked_example_9_portable_bench_emits_o2_o3_o4() {
    let input = facts("a portable bench", Some(Scale::Bench), vec![], &[]);
    assert_eq!(evaluate(&input), vec![o2(), o3(), o4()]);
}

#[test]
fn e1_worked_example_10_empty_tools_and_materials_emit_o3_o4() {
    let input = facts("a birdhouse", Some(Scale::Bench), vec![], &[]);
    assert_eq!(evaluate(&input), vec![o3(), o4()]);
}

#[test]
fn o4_worked_example_11_listed_tools_leave_only_materials_question() {
    let input = facts("a birdhouse", Some(Scale::Bench), vec![jig(), circ()], &[]);
    assert_eq!(evaluate(&input), vec![o4()]);
}

#[test]
fn e1_worked_example_12_birdhouse_with_tool_and_material_emits_nothing() {
    let input = facts(
        "a birdhouse",
        Some(Scale::Bench),
        vec![jig()],
        &["15 pine 2x4 3ft"],
    );
    assert_eq!(evaluate(&input), vec![]);
}

#[test]
fn e1_worked_example_13_joinery_knock_down_and_materials_share_facts() {
    let input = facts(
        "a dresser that must come apart",
        Some(Scale::Room),
        vec![jig(), circ()],
        &[],
    );
    assert_eq!(
        evaluate(&input),
        vec![
            o1("jigsaw", "cut.rip", "dresser"),
            o1("circular saw", "cut.crosscut", "dresser"),
            o2(),
            o4(),
        ]
    );
}

#[test]
fn e1_worked_example_14_completely_empty_facts_emit_only_o3_o4() {
    let input = facts("", None, vec![], &[]);
    assert_eq!(evaluate(&input), vec![o3(), o4()]);
}

#[test]
fn o2_worked_example_15_movable_desk_asks_one_question() {
    let input = facts(
        "a movable desk",
        Some(Scale::Room),
        vec![saw(), mitre()],
        &["15 pine 2x4 3ft"],
    );
    assert_eq!(evaluate(&input), vec![o2()]);
}

#[test]
fn a1_worked_example_16_shared_track_saw_supplies_exact_rip() {
    let input = facts("a shelf", None, vec![track()], &["15 pine 2x4 3ft"]);
    assert_eq!(evaluate(&input), vec![]);
}
