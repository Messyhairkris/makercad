//! Contract tests for mk_assist::interpret, written by Gordon as a PA (the Crew) from
//! crew/contracts/mk_interpret.md. They were the hidden tests of campaign mk-core-11.

use mk_assist::interpret::{interpret, mentions, Input, Interpretation, Item, ScaleClass, Why};

// ---- helpers -----------------------------------------------------------------

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

fn in_note(text: &str) -> Input {
    Input {
        notes: vec![text.to_string()],
        ..Input::default()
    }
}

fn with_attachment(name: &str) -> Input {
    Input {
        attachments: vec![name.to_string()],
        ..Input::default()
    }
}

/// Is there an output whose why is exactly (rule, matched, confidence)?
fn has_why(interp: &Interpretation, rule: u8, matched: &str, confidence: u8) -> bool {
    let same =
        |why: &Why| why.rule == rule && why.matched == matched && why.confidence == confidence;
    let scale_same = match &interp.scale {
        Some((_, why)) => same(why),
        None => false,
    };
    scale_same
        || interp
            .envs
            .iter()
            .chain(&interp.tools)
            .chain(&interp.templates)
            .chain(&interp.questions)
            .any(|item| same(&item.why))
}

/// The whole result when only rule 1 fires, with `matched` at `confidence`.
fn only_rule_1(matched: &str, c: u8) -> Interpretation {
    Interpretation {
        scale: Some((ScaleClass::Room, why(1, matched, c))),
        envs: vec![
            item("solid", 1, matched, c),
            item("sketch", 1, matched, c),
            item("stock", 1, matched, c),
        ],
        tools: vec![
            item("box_joint", 1, matched, c),
            item("drawer_wizard", 1, matched, c),
            item("shelf_pins", 1, matched, c),
            item("panel_from_stock", 1, matched, c),
        ],
        templates: Vec::new(),
        questions: vec![item("space", 1, matched, c)],
    }
}

fn only_rule_2(matched: &str, c: u8) -> Interpretation {
    Interpretation {
        scale: None,
        envs: vec![
            item("fold", 2, matched, c),
            item("sketch", 2, matched, c),
            item("drawings", 2, matched, c),
        ],
        tools: Vec::new(),
        templates: vec![
            item("FEFCO 0201", 2, matched, c),
            item("ECMA A20.20 reverse tuck end", 2, matched, c),
            item("tuck-top mailer", 2, matched, c),
            item("sleeve", 2, matched, c),
        ],
        questions: vec![item("What goes in the box?", 2, matched, c)],
    }
}

fn only_rule_3(matched: &str, c: u8) -> Interpretation {
    Interpretation {
        scale: Some((ScaleClass::Body, why(3, matched, c))),
        envs: vec![
            item("softgoods", 3, matched, c),
            item("solid", 3, matched, c),
        ],
        tools: Vec::new(),
        templates: Vec::new(),
        questions: Vec::new(),
    }
}

fn only_rule_4(matched: &str, c: u8) -> Interpretation {
    Interpretation {
        scale: None,
        envs: vec![
            item("motion", 4, matched, c),
            item("solid", 4, matched, c),
            item("stock", 4, matched, c),
            item("plan", 4, matched, c),
        ],
        tools: vec![
            item("fold_states", 4, matched, c),
            item("hinge", 4, matched, c),
            item("gas_strut", 4, matched, c),
            item("clearance_sweep", 4, matched, c),
        ],
        templates: Vec::new(),
        questions: Vec::new(),
    }
}

fn only_rule_5(matched: &str, c: u8) -> Interpretation {
    Interpretation {
        scale: Some((ScaleClass::Bench, why(5, matched, c))),
        envs: vec![item("solid", 5, matched, c), item("sketch", 5, matched, c)],
        tools: vec![
            item("measure_from_photo", 5, matched, c),
            item("calipers_prompt", 5, matched, c),
            item("print3d_check", 5, matched, c),
        ],
        templates: Vec::new(),
        questions: Vec::new(),
    }
}

fn only_rule_6(ext: &str) -> Interpretation {
    Interpretation {
        scale: None,
        envs: vec![item("space", 6, ext, 90)],
        tools: Vec::new(),
        templates: Vec::new(),
        questions: Vec::new(),
    }
}

const RULE_1_WORDS: [&str; 7] = [
    "dresser", "cabinet", "shelf", "desk", "bookcase", "table", "bed",
];
const RULE_2_WORDS: [&str; 7] = [
    "box",
    "package",
    "packaging",
    "carton",
    "sleeve",
    "mailer",
    "dieline",
];
const RULE_3_WORDS: [&str; 8] = [
    "backpack", "bag", "pouch", "sew", "fabric", "pattern", "quilt", "cover",
];
const RULE_4_WORDS: [&str; 7] = [
    "fold away",
    "murphy",
    "foldable",
    "collapsible",
    "boat",
    "tiny house",
    "convertible",
];
const RULE_5_WORDS: [&str; 6] = [
    "replacement",
    "broken",
    "battery cover",
    "clip",
    "knob",
    "bracket",
];
const RULE_6_EXTENSIONS: [&str; 6] = ["ply", "splat", "spz", "e57", "las", "usdz"];

// ---- the worked-example table, one whole Interpretation per row -----------------

#[test]
fn worked_example_row_1_dresser() {
    let got = interpret(&goal("A small dresser for the hall"));
    let want = Interpretation {
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
    assert_eq!(got, want);
}

#[test]
fn worked_example_row_2_gift_boxes() {
    let got = interpret(&goal("Gift boxes for the market"));
    let want = Interpretation {
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
    assert_eq!(got, want);
}

#[test]
fn worked_example_row_3_embedded_lights() {
    let got = interpret(&goal("Embedded lights"));
    assert_eq!(got, Interpretation::default());
}

#[test]
fn worked_example_row_4_desk_with_note() {
    let got = interpret(&Input {
        goal: "A desk".to_string(),
        notes: vec!["maybe a fold away top".to_string()],
        ..Input::default()
    });
    let want = Interpretation {
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
    assert_eq!(got, want);
}

#[test]
fn worked_example_row_5_replacement_knob() {
    let got = interpret(&Input {
        goal: "Replacement knob".to_string(),
        attachments: vec!["room.PLY".to_string()],
        ..Input::default()
    });
    let want = Interpretation {
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
    assert_eq!(got, want);
}

#[test]
fn worked_example_row_6_tote_bag() {
    let got = interpret(&Input {
        goal: "A tote bag".to_string(),
        style_words: vec!["cabinet".to_string()],
        ..Input::default()
    });
    let want = Interpretation {
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
    assert_eq!(got, want);
}

#[test]
fn worked_example_row_7_tiny_houses() {
    let got = interpret(&goal("Tiny houses on wheels"));
    let want = Interpretation {
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
    assert_eq!(got, want);
}

#[test]
fn worked_example_row_8_fold_away_bed() {
    let got = interpret(&goal("fold-away bed"));
    let want = Interpretation {
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
    assert_eq!(got, want);
}

#[test]
fn worked_example_row_9_mentions_boxes() {
    assert!(mentions("BOXES and bags", "box"));
}

#[test]
fn worked_example_row_10_mentions_boxy() {
    assert!(!mentions("boxy", "box"));
}

#[test]
fn worked_example_row_11_mentions_empty_phrase() {
    assert!(!mentions("a", ""));
}

// ---- one whole Interpretation per rule, firing alone ---------------------------

#[test]
fn rule_1_alone() {
    assert_eq!(interpret(&goal("A shelf")), only_rule_1("shelf", 90));
}

#[test]
fn rule_2_alone() {
    assert_eq!(interpret(&goal("A carton")), only_rule_2("carton", 90));
}

#[test]
fn rule_3_alone() {
    assert_eq!(interpret(&goal("A pouch")), only_rule_3("pouch", 90));
}

#[test]
fn rule_4_alone() {
    assert_eq!(interpret(&goal("A murphy")), only_rule_4("murphy", 90));
}

#[test]
fn rule_5_alone() {
    assert_eq!(interpret(&goal("A knob")), only_rule_5("knob", 90));
}

#[test]
fn rule_6_alone() {
    assert_eq!(interpret(&with_attachment("scan.E57")), only_rule_6("e57"));
}

// ---- every word of every rule, alone in the goal (90) and in a note (50) --------

#[test]
fn rule_1_words_in_goal() {
    for word in RULE_1_WORDS {
        let got = interpret(&goal(word));
        assert!(
            has_why(&got, 1, word, 90),
            "rule 1 word \"{word}\" in the goal"
        );
    }
}

#[test]
fn rule_1_words_in_a_note() {
    for word in RULE_1_WORDS {
        let got = interpret(&in_note(word));
        assert!(
            has_why(&got, 1, word, 50),
            "rule 1 word \"{word}\" in a note"
        );
    }
}

#[test]
fn rule_2_words_in_goal() {
    for word in RULE_2_WORDS {
        let got = interpret(&goal(word));
        assert!(
            has_why(&got, 2, word, 90),
            "rule 2 word \"{word}\" in the goal"
        );
    }
}

#[test]
fn rule_2_words_in_a_note() {
    for word in RULE_2_WORDS {
        let got = interpret(&in_note(word));
        assert!(
            has_why(&got, 2, word, 50),
            "rule 2 word \"{word}\" in a note"
        );
    }
}

#[test]
fn rule_3_words_in_goal() {
    for word in RULE_3_WORDS {
        let got = interpret(&goal(word));
        assert!(
            has_why(&got, 3, word, 90),
            "rule 3 word \"{word}\" in the goal"
        );
    }
}

#[test]
fn rule_3_words_in_a_note() {
    for word in RULE_3_WORDS {
        let got = interpret(&in_note(word));
        assert!(
            has_why(&got, 3, word, 50),
            "rule 3 word \"{word}\" in a note"
        );
    }
}

#[test]
fn rule_4_words_in_goal() {
    for word in RULE_4_WORDS {
        let got = interpret(&goal(word));
        assert!(
            has_why(&got, 4, word, 90),
            "rule 4 word \"{word}\" in the goal"
        );
    }
}

#[test]
fn rule_4_words_in_a_note() {
    for word in RULE_4_WORDS {
        let got = interpret(&in_note(word));
        assert!(
            has_why(&got, 4, word, 50),
            "rule 4 word \"{word}\" in a note"
        );
    }
}

#[test]
fn rule_5_words_in_goal() {
    for word in RULE_5_WORDS {
        let got = interpret(&goal(word));
        assert!(
            has_why(&got, 5, word, 90),
            "rule 5 word \"{word}\" in the goal"
        );
    }
}

#[test]
fn rule_5_words_in_a_note() {
    for word in RULE_5_WORDS {
        let got = interpret(&in_note(word));
        assert!(
            has_why(&got, 5, word, 50),
            "rule 5 word \"{word}\" in a note"
        );
    }
}

// ---- rule 6: attachment extensions --------------------------------------------

#[test]
fn rule_6_every_extension_upper_and_lower() {
    for ext in RULE_6_EXTENSIONS {
        let lower = interpret(&with_attachment(&format!("model.{ext}")));
        assert_eq!(lower, only_rule_6(ext), "lowercase extension {ext}");
        let upper = interpret(&with_attachment(&format!("model.{}", ext.to_uppercase())));
        assert_eq!(upper, only_rule_6(ext), "uppercase extension {ext}");
    }
}

#[test]
fn rule_6_first_extension_in_list_order_wins() {
    let got = interpret(&Input {
        attachments: vec!["scan.usdz".to_string(), "desk.ply".to_string()],
        ..Input::default()
    });
    assert_eq!(got, only_rule_6("ply"));
}

#[test]
fn rule_6_extension_is_after_the_last_dot() {
    assert_eq!(
        interpret(&with_attachment("archive.tar.PLY")),
        only_rule_6("ply")
    );
    assert_eq!(
        interpret(&with_attachment("scan.usdz.tar")),
        Interpretation::default()
    );
}

#[test]
fn rule_6_name_without_a_dot_has_no_extension() {
    assert_eq!(
        interpret(&with_attachment("ply")),
        Interpretation::default()
    );
    assert_eq!(
        interpret(&with_attachment("room.")),
        Interpretation::default()
    );
}

#[test]
fn attachments_do_not_fire_word_rules() {
    // Rule W2 searches the goal, style words, notes and materials, never file names.
    assert_eq!(
        interpret(&with_attachment("dresser.png")),
        Interpretation::default()
    );
    assert_eq!(
        interpret(&with_attachment("dresser.ply")),
        only_rule_6("ply")
    );
}

// ---- O1: the scale ------------------------------------------------------------

#[test]
fn out_1_scale_is_the_first_fired_rule_with_a_scale() {
    // Rule 1 at 50 beats rule 3 at 90: table order decides the scale, not confidence.
    let got = interpret(&Input {
        goal: "A tote bag".to_string(),
        style_words: vec!["cabinet".to_string()],
        ..Input::default()
    });
    assert_eq!(got.scale, Some((ScaleClass::Room, why(1, "cabinet", 50))));

    // Rule 3 (Body) comes before rule 5 (Bench).
    let got = interpret(&goal("A broken bag"));
    assert_eq!(got.scale, Some((ScaleClass::Body, why(3, "bag", 90))));
}

#[test]
fn out_1_scale_stays_none_for_rules_without_one() {
    assert_eq!(interpret(&goal("A murphy")).scale, None);
    assert_eq!(interpret(&goal("A carton")).scale, None);
    assert_eq!(interpret(&with_attachment("a.las")).scale, None);
    assert_eq!(interpret(&Input::default()).scale, None);
}

// ---- O2: envs, tools, templates, questions -------------------------------------

#[test]
fn out_2_higher_confidence_wins_and_position_stays() {
    // Rule 1 fires first (solid at 50), rule 3 later (solid at 90): solid keeps position 0
    // and takes rule 3's why.
    let got = interpret(&Input {
        goal: "A bag".to_string(),
        style_words: vec!["shelf".to_string()],
        ..Input::default()
    });
    assert_eq!(
        got.envs,
        vec![
            item("solid", 3, "bag", 90),
            item("sketch", 1, "shelf", 50),
            item("stock", 1, "shelf", 50),
            item("softgoods", 3, "bag", 90),
        ]
    );
    assert!(got.envs[0].value == "solid");
}

#[test]
fn out_2_tie_goes_to_the_earlier_rule() {
    // Both rules fire from the goal at 90 and share solid and stock: rule 1 wins the why.
    let got = interpret(&goal("fold away desk"));
    assert_eq!(
        got.envs,
        vec![
            item("solid", 1, "desk", 90),
            item("sketch", 1, "desk", 90),
            item("stock", 1, "desk", 90),
            item("motion", 4, "fold away", 90),
            item("plan", 4, "fold away", 90),
        ]
    );
    assert!(got.envs[0].why.rule == 1);
    assert!(got.envs[2].why.rule == 1);
}

#[test]
fn out_2_later_lower_confidence_does_not_replace() {
    let got = interpret(&Input {
        goal: "A desk".to_string(),
        notes: vec!["a bag".to_string()],
        ..Input::default()
    });
    assert_eq!(
        got.envs,
        vec![
            item("solid", 1, "desk", 90),
            item("sketch", 1, "desk", 90),
            item("stock", 1, "desk", 90),
            item("softgoods", 3, "bag", 50),
        ]
    );
    assert!(got.envs[0].why.rule == 1);
}

#[test]
fn out_2_each_value_appears_once() {
    let got = interpret(&goal("A desk that will fold away"));
    let values: Vec<&str> = got.envs.iter().map(|item| item.value.as_str()).collect();
    assert_eq!(values, vec!["solid", "sketch", "stock", "motion", "plan"]);
    assert_eq!(
        got.envs.iter().filter(|item| item.value == "solid").count(),
        1
    );
    assert_eq!(got.tools.len(), 8);
}

// ---- O3: every why carries its rule, word and confidence ------------------------

#[test]
fn out_3_every_why_has_its_rule_word_and_confidence() {
    let got = interpret(&Input {
        goal: "a fold-away desk".to_string(),
        notes: vec!["a bag".to_string()],
        materials: vec!["ply".to_string()],
        attachments: vec!["x.ply".to_string()],
        style_words: Vec::new(),
    });
    let mut whys = Vec::new();
    if let Some((_, why)) = &got.scale {
        whys.push(why.clone());
    }
    for item in got
        .envs
        .iter()
        .chain(&got.tools)
        .chain(&got.templates)
        .chain(&got.questions)
    {
        whys.push(item.why.clone());
    }
    assert!(!whys.is_empty());
    for why in whys {
        assert!((1..=6).contains(&why.rule));
        assert!(!why.matched.is_empty());
        assert!(why.confidence == 90 || why.confidence == 50);
    }
}

// ---- Why::is_question ----------------------------------------------------------

#[test]
fn why_is_a_question_under_sixty() {
    assert!(why(1, "box", 0).is_question());
    assert!(why(1, "box", 50).is_question());
    assert!(why(1, "box", 59).is_question());
    assert!(!why(1, "box", 60).is_question());
    assert!(!why(1, "box", 90).is_question());
    assert!(!why(1, "box", 100).is_question());
}

#[test]
fn note_only_outputs_are_questions() {
    let got = interpret(&in_note("fold away"));
    assert_eq!(got.tools.len(), 4);
    assert!(got.envs.iter().all(|item| item.why.is_question()));
    assert!(got.tools.iter().all(|item| item.why.is_question()));
}

// ---- W2 and W3: where a word is found and what that is worth --------------------

#[test]
fn out_w3_style_words_fire_at_fifty() {
    let got = interpret(&Input {
        style_words: vec!["bookcase".to_string()],
        ..Input::default()
    });
    assert_eq!(got.scale, Some((ScaleClass::Room, why(1, "bookcase", 50))));
}

#[test]
fn out_w3_materials_fire_at_fifty() {
    let got = interpret(&Input {
        materials: vec!["a quilt".to_string()],
        ..Input::default()
    });
    assert_eq!(got.scale, Some((ScaleClass::Body, why(3, "quilt", 50))));

    // A material name is not an attachment name: R6 reads attachments only.
    let plywood = interpret(&Input {
        materials: vec!["plywood".to_string()],
        ..Input::default()
    });
    assert_eq!(plywood, Interpretation::default());
}

#[test]
fn w2_a_phrase_never_spans_two_fields() {
    let split = Input {
        style_words: vec!["tiny".to_string()],
        notes: vec!["house".to_string()],
        ..Input::default()
    };
    assert_eq!(interpret(&split), Interpretation::default());
}

#[test]
fn a_rules_matched_is_its_first_word_that_fires() {
    // "cabinet" is second in rule 1's list and fires at 50; "desk" fires at 90 but is later.
    let got = interpret(&Input {
        goal: "a desk".to_string(),
        style_words: vec!["cabinet".to_string()],
        ..Input::default()
    });
    assert_eq!(got.scale, Some((ScaleClass::Room, why(1, "cabinet", 50))));
    assert_eq!(got.questions, vec![item("space", 1, "cabinet", 50)]);
}

// ---- W1: mentions --------------------------------------------------------------

#[test]
fn w1_ignores_case() {
    assert!(mentions("A SMALL DRESSER", "small dresser"));
    assert!(mentions("Box", "bOx"));
    assert!(mentions("a TiNy HoUsE", "tiny house"));
}

#[test]
fn w1_punctuation_becomes_space() {
    assert!(mentions("A small dresser, for the hall.", "dresser for"));
    assert!(mentions("box;bag", "box bag"));
    assert!(mentions("(box)", "box"));
    assert!(mentions("box\u{2014}bag", "box bag"));
}

#[test]
fn w1_hyphen_becomes_space() {
    assert!(mentions("fold-away bed", "fold away"));
    assert!(mentions("fold away", "fold-away"));
    assert!(!mentions("foldaway", "fold away"));
}

#[test]
fn w1_plurals_with_s_and_es() {
    assert!(mentions("boxes", "box"));
    assert!(mentions("bags", "bag"));
    assert!(mentions("sleeves", "sleeve"));
    assert!(mentions("two tiny houses", "tiny house"));
    assert!(mentions("batches", "batch"));
    assert!(!mentions("box", "boxes"));
}

#[test]
fn w1_only_the_last_word_may_be_plural() {
    assert!(!mentions("boxes bag", "box bag"));
    assert!(!mentions("bags box", "bag box"));
    assert!(mentions("tiny boxes", "tiny box"));
}

#[test]
fn w1_phrases_are_consecutive_words() {
    assert!(mentions("a tiny house on wheels", "tiny house"));
    assert!(!mentions("a tiny red house", "tiny house"));
    assert!(!mentions("house tiny", "tiny house"));
    assert!(mentions("battery cover", "battery cover"));
    assert!(!mentions("battery cover", "cover battery"));
}

#[test]
fn w1_near_misses() {
    assert!(!mentions("boxy", "box"));
    assert!(!mentions("boxed", "box"));
    assert!(!mentions("embedded", "bed"));
    assert!(!mentions("bag", "bags"));
    assert!(!mentions("tabled", "table"));
    assert!(!mentions("folds away", "fold away"));
    assert!(mentions("a bed", "bed"));
}

#[test]
fn w1_empty_phrase_and_empty_text() {
    assert!(!mentions("", ""));
    assert!(!mentions("", "box"));
    assert!(!mentions("box", ""));
    assert!(!mentions("box", "!!!"));
    assert!(!mentions("a", ""));
}

#[test]
fn w1_digits_and_non_ascii() {
    assert!(mentions("3D printed bracket", "3d"));
    assert!(mentions("mk2 desk", "mk2"));
    assert!(mentions("caf\u{e9} box", "box"));
    assert!(!mentions("caf\u{e9}", "cafe"));
    // A non-ASCII letter is not an ASCII letter or digit: rule W1 makes it a space.
    assert!(mentions("caf\u{e9}", "caf"));
}

#[test]
fn w1_emoji_is_a_separator() {
    assert!(!mentions("\u{1f6e0}\u{fe0f}", "\u{1f6e0}"));
    assert!(mentions("\u{1f6e0}\u{fe0f} a box", "box"));
    assert!(!mentions("box", "\u{1f6e0}"));
}

#[test]
fn w1_long_text_does_not_panic() {
    let mut text = "widget ".repeat(5_000);
    text.push_str("box");
    assert!(mentions(&text, "box"));
    assert!(!mentions(&text, "boxy"));
}

// ---- odd input that must never panic -------------------------------------------

#[test]
fn odd_input_empty() {
    assert_eq!(interpret(&Input::default()), Interpretation::default());
}

#[test]
fn odd_input_emoji_and_punctuation() {
    let got = interpret(&Input {
        goal: "\u{1f6e0}\u{fe0f}\u{2728}\u{1fa9a}".to_string(),
        style_words: vec!["\u{2014}".to_string()],
        notes: vec!["\u{2026}".to_string()],
        materials: vec![String::new()],
        attachments: vec![String::new()],
    });
    assert_eq!(got, Interpretation::default());
}

#[test]
fn odd_input_ten_thousand_words() {
    let mut text = "widget ".repeat(10_000);
    text.push_str("dresser");
    assert_eq!(interpret(&goal(&text)), only_rule_1("dresser", 90));

    let filler = "widget ".repeat(10_000);
    assert_eq!(interpret(&goal(&filler)), Interpretation::default());
}
