//! Tests for mk_assist::interpret written by Kimi (the Crew, campaign mk-core-11) from the contract
//! alone, without seeing the module.

//! Integration tests for the deterministic project interpreter contract.

use mk_assist::interpret::{interpret, mentions, Input, Interpretation, Item, ScaleClass, Why};

fn why(rule: u8, matched: &str, confidence: u8) -> Why {
    Why {
        rule,
        matched: matched.to_string(),
        confidence,
    }
}

fn item(value: &str, rule: u8, matched: &str, confidence: u8) -> Item {
    Item {
        value: value.to_string(),
        why: why(rule, matched, confidence),
    }
}

fn goal(text: &str) -> Input {
    Input {
        goal: text.to_string(),
        ..Input::default()
    }
}

fn note(text: &str) -> Input {
    Input {
        notes: vec![text.to_string()],
        ..Input::default()
    }
}

#[test]
fn w1_worked_example_boxes_plural_and_case() {
    assert!(mentions("BOXES and bags", "box"));
}

#[test]
fn w1_worked_example_boxy_near_miss() {
    assert!(!mentions("boxy", "box"));
}

#[test]
fn w1_worked_example_empty_phrase() {
    assert!(!mentions("a", ""));
}

#[test]
fn w1_punctuation_hyphens_plurals_and_multiword_phrases() {
    assert!(mentions("Bags, please!", "bag"));
    assert!(mentions("packages!", "package"));
    assert!(mentions("a battery-cover", "battery cover"));
    assert!(mentions("Tiny houses on wheels", "tiny house"));
    assert!(mentions("fold-away bed", "fold away"));
}

#[test]
fn w1_embedded_near_miss_and_nonconsecutive_phrase() {
    assert!(!mentions("embedded", "bed"));
    assert!(!mentions("tiny little house", "tiny house"));
}

#[test]
fn w2_matches_style_words_and_materials_separately() {
    let by_style = interpret(&Input {
        style_words: vec!["box".to_string()],
        ..Input::default()
    });
    assert_eq!(by_style.envs.first(), Some(&item("fold", 2, "box", 50)));

    let by_material = interpret(&Input {
        materials: vec!["backpack".to_string()],
        ..Input::default()
    });
    assert_eq!(
        by_material.scale,
        Some((ScaleClass::Body, why(3, "backpack", 50)))
    );
}

#[test]
fn w2_phrase_never_spans_input_fields() {
    let input = Input {
        goal: "tiny".to_string(),
        style_words: vec!["fold".to_string()],
        notes: vec!["house".to_string()],
        materials: vec!["away".to_string()],
        attachments: Vec::new(),
    };
    assert_eq!(interpret(&input), Interpretation::default());
}

#[test]
fn w3_r1_first_word_in_goal_and_note() {
    let by_goal = interpret(&goal("dresser"));
    assert_eq!(
        by_goal.scale,
        Some((ScaleClass::Room, why(1, "dresser", 90)))
    );

    let by_note = interpret(&note("perhaps a dresser"));
    let note_scale = by_note.scale.expect("R1 should set Room scale");
    assert_eq!(note_scale.1, why(1, "dresser", 50));
    assert!(note_scale.1.is_question());
}

#[test]
fn w3_r2_first_word_in_goal_and_note() {
    let by_goal = interpret(&goal("box"));
    assert_eq!(by_goal.envs.first(), Some(&item("fold", 2, "box", 90)));

    let by_note = interpret(&note("a box"));
    let note_item = by_note.envs.first().expect("R2 should add fold");
    assert_eq!(note_item, &item("fold", 2, "box", 50));
    assert!(note_item.why.is_question());
}

#[test]
fn w3_r3_first_word_in_goal_and_note() {
    let by_goal = interpret(&goal("backpack"));
    assert_eq!(
        by_goal.scale,
        Some((ScaleClass::Body, why(3, "backpack", 90)))
    );

    let by_note = interpret(&note("backpack"));
    let note_scale = by_note.scale.expect("R3 should set Body scale");
    assert_eq!(note_scale.1, why(3, "backpack", 50));
    assert!(note_scale.1.is_question());
}

#[test]
fn w3_r4_first_word_in_goal_and_note() {
    let by_goal = interpret(&goal("fold away"));
    assert_eq!(
        by_goal.envs.first(),
        Some(&item("motion", 4, "fold away", 90))
    );

    let by_note = interpret(&note("fold away"));
    let note_item = by_note.envs.first().expect("R4 should add motion");
    assert_eq!(note_item, &item("motion", 4, "fold away", 50));
    assert!(note_item.why.is_question());
}

#[test]
fn w3_r5_first_word_in_goal_and_note() {
    let by_goal = interpret(&goal("replacement"));
    assert_eq!(
        by_goal.scale,
        Some((ScaleClass::Bench, why(5, "replacement", 90)))
    );

    let by_note = interpret(&note("replacement"));
    let note_scale = by_note.scale.expect("R5 should set Bench scale");
    assert_eq!(note_scale.1, why(5, "replacement", 50));
    assert!(note_scale.1.is_question());
}

#[test]
fn r1_worked_example_dresser() {
    let expected = Interpretation {
        scale: Some((ScaleClass::Room, why(1, "dresser", 90))),
        envs: vec![
            item("solid", 1, "dresser", 90),
            item("sketch", 1, "dresser", 90),
            item("stock", 1, "dresser", 90),
        ],
        tools: vec![
            item("box_joint", 1, "dresser", 90),
            item("drawer_wizard", 1, "dresser", 90),
            item("shelf_pins", 1, "dresser", 90),
            item("panel_from_stock", 1, "dresser", 90),
        ],
        templates: Vec::new(),
        questions: vec![item("space", 1, "dresser", 90)],
    };
    assert_eq!(interpret(&goal("A small dresser for the hall")), expected);
}

#[test]
fn r2_worked_example_gift_boxes() {
    let expected = Interpretation {
        scale: None,
        envs: vec![
            item("fold", 2, "box", 90),
            item("sketch", 2, "box", 90),
            item("drawings", 2, "box", 90),
        ],
        tools: Vec::new(),
        templates: vec![
            item("FEFCO 0201", 2, "box", 90),
            item("ECMA A20.20 reverse tuck end", 2, "box", 90),
            item("tuck-top mailer", 2, "box", 90),
            item("sleeve", 2, "box", 90),
        ],
        questions: vec![item("What goes in the box?", 2, "box", 90)],
    };
    assert_eq!(interpret(&goal("Gift boxes for the market")), expected);
}

#[test]
fn r4_worked_example_tiny_houses() {
    let expected = Interpretation {
        scale: None,
        envs: vec![
            item("motion", 4, "tiny house", 90),
            item("solid", 4, "tiny house", 90),
            item("stock", 4, "tiny house", 90),
            item("plan", 4, "tiny house", 90),
        ],
        tools: vec![
            item("fold_states", 4, "tiny house", 90),
            item("hinge", 4, "tiny house", 90),
            item("gas_strut", 4, "tiny house", 90),
            item("clearance_sweep", 4, "tiny house", 90),
        ],
        templates: Vec::new(),
        questions: Vec::new(),
    };
    assert_eq!(interpret(&goal("Tiny houses on wheels")), expected);
}

#[test]
fn r1_and_r4_worked_example_fold_away_bed() {
    let expected = Interpretation {
        scale: Some((ScaleClass::Room, why(1, "bed", 90))),
        envs: vec![
            item("solid", 1, "bed", 90),
            item("sketch", 1, "bed", 90),
            item("stock", 1, "bed", 90),
            item("motion", 4, "fold away", 90),
            item("plan", 4, "fold away", 90),
        ],
        tools: vec![
            item("box_joint", 1, "bed", 90),
            item("drawer_wizard", 1, "bed", 90),
            item("shelf_pins", 1, "bed", 90),
            item("panel_from_stock", 1, "bed", 90),
            item("fold_states", 4, "fold away", 90),
            item("hinge", 4, "fold away", 90),
            item("gas_strut", 4, "fold away", 90),
            item("clearance_sweep", 4, "fold away", 90),
        ],
        templates: Vec::new(),
        questions: vec![item("space", 1, "bed", 90)],
    };
    assert_eq!(interpret(&goal("fold-away bed")), expected);
}

#[test]
fn r1_and_r4_worked_example_desk_and_note() {
    let input = Input {
        goal: "A desk".to_string(),
        notes: vec!["maybe a fold away top".to_string()],
        ..Input::default()
    };
    let expected = Interpretation {
        scale: Some((ScaleClass::Room, why(1, "desk", 90))),
        envs: vec![
            item("solid", 1, "desk", 90),
            item("sketch", 1, "desk", 90),
            item("stock", 1, "desk", 90),
            item("motion", 4, "fold away", 50),
            item("plan", 4, "fold away", 50),
        ],
        tools: vec![
            item("box_joint", 1, "desk", 90),
            item("drawer_wizard", 1, "desk", 90),
            item("shelf_pins", 1, "desk", 90),
            item("panel_from_stock", 1, "desk", 90),
            item("fold_states", 4, "fold away", 50),
            item("hinge", 4, "fold away", 50),
            item("gas_strut", 4, "fold away", 50),
            item("clearance_sweep", 4, "fold away", 50),
        ],
        templates: Vec::new(),
        questions: vec![item("space", 1, "desk", 90)],
    };
    let actual = interpret(&input);
    assert!(actual.tools[4].why.is_question());
    assert_eq!(actual, expected);
}

#[test]
fn r5_and_r6_worked_example_replacement_knob() {
    let input = Input {
        goal: "Replacement knob".to_string(),
        attachments: vec!["room.PLY".to_string()],
        ..Input::default()
    };
    let expected = Interpretation {
        scale: Some((ScaleClass::Bench, why(5, "replacement", 90))),
        envs: vec![
            item("solid", 5, "replacement", 90),
            item("sketch", 5, "replacement", 90),
            item("space", 6, "ply", 90),
        ],
        tools: vec![
            item("measure_from_photo", 5, "replacement", 90),
            item("calipers_prompt", 5, "replacement", 90),
            item("print3d_check", 5, "replacement", 90),
        ],
        templates: Vec::new(),
        questions: Vec::new(),
    };
    assert_eq!(interpret(&input), expected);
}

#[test]
fn o1_and_o2_worked_example_tote_bag_and_cabinet() {
    let input = Input {
        goal: "A tote bag".to_string(),
        style_words: vec!["cabinet".to_string()],
        ..Input::default()
    };
    let expected = Interpretation {
        scale: Some((ScaleClass::Room, why(1, "cabinet", 50))),
        envs: vec![
            item("solid", 3, "bag", 90),
            item("sketch", 1, "cabinet", 50),
            item("stock", 1, "cabinet", 50),
            item("softgoods", 3, "bag", 90),
        ],
        tools: vec![
            item("box_joint", 1, "cabinet", 50),
            item("drawer_wizard", 1, "cabinet", 50),
            item("shelf_pins", 1, "cabinet", 50),
            item("panel_from_stock", 1, "cabinet", 50),
        ],
        templates: Vec::new(),
        questions: vec![item("space", 1, "cabinet", 50)],
    };
    assert_eq!(interpret(&input), expected);
}

#[test]
fn r6_every_uppercase_extension_has_ninety_percent_confidence() {
    for extension in ["ply", "splat", "spz", "e57", "las", "usdz"] {
        let input = Input {
            attachments: vec![format!("scan.{}", extension.to_ascii_uppercase())],
            ..Input::default()
        };
        let expected = Interpretation {
            scale: None,
            envs: vec![item("space", 6, extension, 90)],
            tools: Vec::new(),
            templates: Vec::new(),
            questions: Vec::new(),
        };
        assert!(!expected.envs[0].why.is_question());
        assert_eq!(interpret(&input), expected, "extension {extension}");
    }
}

#[test]
fn r6_first_listed_extension_wins_regardless_of_attachment_order() {
    let input = Input {
        attachments: vec!["scan.USDZ".to_string(), "scan.PLY".to_string()],
        ..Input::default()
    };
    let expected = Interpretation {
        scale: None,
        envs: vec![item("space", 6, "ply", 90)],
        tools: Vec::new(),
        templates: Vec::new(),
        questions: Vec::new(),
    };
    assert_eq!(interpret(&input), expected);
}

#[test]
fn o1_first_fired_rule_with_a_scale_wins() {
    let output = interpret(&goal("box backpack replacement"));
    assert_eq!(
        output.scale,
        Some((ScaleClass::Body, why(3, "backpack", 90)))
    );
}

#[test]
fn o2_equal_confidence_keeps_earlier_rule_and_first_position() {
    let output = interpret(&goal("table bag"));
    assert_eq!(
        output.envs,
        vec![
            item("solid", 1, "table", 90),
            item("sketch", 1, "table", 90),
            item("stock", 1, "table", 90),
            item("softgoods", 3, "bag", 90),
        ]
    );
}

#[test]
fn o4_worked_example_embedded_lights_is_default() {
    assert_eq!(
        interpret(&goal("Embedded lights")),
        Interpretation::default()
    );
}

#[test]
fn o4_empty_input_is_default() {
    assert_eq!(interpret(&Input::default()), Interpretation::default());
}

#[test]
fn o4_emoji_input_never_panics_and_is_default() {
    let input = Input {
        goal: "\u{1f4e6} \u{1f9f0} \u{2728} \u{1fa9a}".to_string(),
        notes: vec!["\u{1f3e0}\u{1f4d0}".to_string()],
        attachments: vec!["README".to_string()],
        ..Input::default()
    };
    assert_eq!(interpret(&input), Interpretation::default());
}

#[test]
fn o4_very_long_input_never_panics_and_is_default() {
    let input = Input {
        goal: "a ".repeat(20_000),
        notes: vec!["\u{2728}".repeat(10_000)],
        ..Input::default()
    };
    assert_eq!(interpret(&input), Interpretation::default());
}
