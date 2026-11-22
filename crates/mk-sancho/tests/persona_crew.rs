//! Tests for mk_sancho::persona written by Deepak (the Crew, campaign mk-core-15) from the
//! contract alone, without seeing the module.

//! Integration tests for `mk_sancho::persona` (contract v1, rules R1-R8, B1-B3, M1-M6).

use mk_sancho::persona::{
    base_bone, check_model, mixamo_base, read_persona, rigify_to_mixamo, write_persona, Height,
    ModelFacts, Persona, PersonaError, Problem, CLIP_KEYS, MAX_BONES, MAX_INFLUENCES,
    MAX_REST_HEIGHT_M, MIN_REST_HEIGHT_M, REQUIRED_BONES, REQUIRED_CLIPS, RIGIFY_TO_MIXAMO,
};

/// The doc's own example file (doc 07 section 7.3.1), comments included.
const EXAMPLE: &str = r#"# Sancho, the persona file (doc 07 section 7.3.1)
[persona]
name = "Sancho"
model = "sancho.glb"
height_m = 1.68
portrait = "sancho_portrait.png"
lod = { modest = "sancho_lod8k.glb", potato = "sancho_vat.glb" }

[bones]                    # only needed if names aren't Mixamo-standard
head = "mixamorig:Head"
point_arm = ["mixamorig:RightArm", "mixamorig:RightForeArm", "mixamorig:RightHand"]

[clips]
idle = "Idle"
wave = "Waving"
point = "Pointing"
think = "Thinking"
worried = "Nervously Look Around"
celebrate = "Victory"
walk_to = "Walking"
sit = "Sitting Idle"
carry_board = "Carry Heavy Object"
"#;

/// What write_persona must produce for the example persona (R8).
const EXAMPLE_WRITTEN: &str = r#"[persona]
name = "Sancho"
model = "sancho.glb"
height_m = 1.68
portrait = "sancho_portrait.png"
lod = { modest = "sancho_lod8k.glb", potato = "sancho_vat.glb" }

[bones]
head = "mixamorig:Head"
point_arm = ["mixamorig:RightArm", "mixamorig:RightForeArm", "mixamorig:RightHand"]

[clips]
idle = "Idle"
wave = "Waving"
point = "Pointing"
think = "Thinking"
worried = "Nervously Look Around"
celebrate = "Victory"
walk_to = "Walking"
sit = "Sitting Idle"
carry_board = "Carry Heavy Object"
"#;

fn nine_clips() -> Vec<(String, String)> {
    vec![
        ("idle".to_string(), "Idle".to_string()),
        ("wave".to_string(), "Waving".to_string()),
        ("point".to_string(), "Pointing".to_string()),
        ("think".to_string(), "Thinking".to_string()),
        ("worried".to_string(), "Nervously Look Around".to_string()),
        ("celebrate".to_string(), "Victory".to_string()),
        ("walk_to".to_string(), "Walking".to_string()),
        ("sit".to_string(), "Sitting Idle".to_string()),
        ("carry_board".to_string(), "Carry Heavy Object".to_string()),
    ]
}

fn example_persona() -> Persona {
    Persona {
        name: "Sancho".to_string(),
        model: "sancho.glb".to_string(),
        height: Height::Metres(1.68),
        portrait: Some("sancho_portrait.png".to_string()),
        lod_modest: Some("sancho_lod8k.glb".to_string()),
        lod_potato: Some("sancho_vat.glb".to_string()),
        head: Some("mixamorig:Head".to_string()),
        point_arm: Some([
            "mixamorig:RightArm".to_string(),
            "mixamorig:RightForeArm".to_string(),
            "mixamorig:RightHand".to_string(),
        ]),
        clips: nine_clips(),
    }
}

fn example_facts() -> ModelFacts {
    ModelFacts {
        bone_names: REQUIRED_BONES
            .iter()
            .map(|b| format!("mixamorig:{}", b))
            .collect(),
        max_influences: 4,
        triangles: 30000,
        clip_names: nine_clips().into_iter().map(|(_, c)| c).collect(),
        rest_height_m: 1.75,
    }
}

// ---------------------------------------------------------------- R1 lines

#[test]
fn r1_crlf_line_ends_read_the_same() {
    let crlf = EXAMPLE.replace('\n', "\r\n");
    assert_eq!(read_persona(&crlf), Ok(example_persona()));
}

#[test]
fn r1_blank_lines_and_comment_lines_are_skipped() {
    let text = "\n\n# a comment\n   \n[persona]\n# inside\nname = \"S\"\nmodel = \"m.glb\"\n\
                height_m = 1.7\n\n[clips]\nidle = \"I\"\nwave = \"W\"\npoint = \"P\"\n\
                think = \"T\"\nworried = \"Wo\"\ncelebrate = \"C\"\nwalk_to = \"Wa\"\n\
                sit = \"Si\"\ncarry_board = \"CB\"\n";
    let p = read_persona(text).expect("valid file");
    assert_eq!(p.name, "S");
    assert_eq!(p.height, Height::Metres(1.7));
    assert_eq!(p.clips.len(), 9);
}

#[test]
fn r1_lines_are_trimmed() {
    let text = "  [persona]  \n  name = \"S\"  \n\tmodel = \"m.glb\"\t\n height_m = 1.7 \n\
                [clips]\nidle = \"I\"\nwave = \"W\"\npoint = \"P\"\nthink = \"T\"\n\
                worried = \"Wo\"\ncelebrate = \"C\"\nwalk_to = \"Wa\"\nsit = \"Si\"\n\
                carry_board = \"CB\"\n";
    let p = read_persona(text).expect("valid file");
    assert_eq!(p.name, "S");
    assert_eq!(p.model, "m.glb");
}

// ------------------------------------------------------------- R2 headers

#[test]
fn r2_header_may_carry_a_comment() {
    let text = "[persona]\nname = \"S\"\nmodel = \"m.glb\"\nheight_m = 1.7\n\
                [bones]   # only needed if names aren't Mixamo-standard\n\
                head = \"mixamorig:Head\"\n\
                [clips]\nidle = \"I\"\nwave = \"W\"\npoint = \"P\"\nthink = \"T\"\n\
                worried = \"Wo\"\ncelebrate = \"C\"\nwalk_to = \"Wa\"\nsit = \"Si\"\n\
                carry_board = \"CB\"\n";
    let p = read_persona(text).expect("valid file");
    assert_eq!(p.head, Some("mixamorig:Head".to_string()));
}

#[test]
fn r2_unclosed_header_is_bad_line() {
    assert_eq!(read_persona("[persona\n"), Err(PersonaError::BadLine(1)));
}

#[test]
fn r2_header_with_trailing_junk_is_bad_line() {
    assert_eq!(read_persona("[persona] x\n"), Err(PersonaError::BadLine(1)));
}

#[test]
fn r2_unknown_table() {
    assert_eq!(
        read_persona("[voice]\n"),
        Err(PersonaError::UnknownTable(1, "voice".to_string()))
    );
}

#[test]
fn r2_duplicate_table() {
    let text = "[persona]\nname = \"S\"\nmodel = \"m.glb\"\nheight_m = 1.7\n\
                [clips]\nidle = \"I\"\n[clips]\n";
    assert_eq!(
        read_persona(text),
        Err(PersonaError::DuplicateTable(7, "clips".to_string()))
    );
}

// ---------------------------------------------------------------- R3 keys

#[test]
fn r3_key_line_before_any_header_is_bad_line() {
    assert_eq!(
        read_persona("name = \"Sancho\"\n"),
        Err(PersonaError::BadLine(1))
    );
}

#[test]
fn r3_key_with_bad_characters_is_bad_line() {
    let text = "[persona]\nna me = \"S\"\n";
    assert_eq!(read_persona(text), Err(PersonaError::BadLine(2)));
}

#[test]
fn r3_empty_key_is_bad_line() {
    let text = "[persona]\n= \"S\"\n";
    assert_eq!(read_persona(text), Err(PersonaError::BadLine(2)));
}

#[test]
fn r3_unknown_key_names_table_and_key() {
    let text = "[persona]\nname = \"S\"\nmodel = \"m.glb\"\nheight_m = 1.7\ncolour = \"red\"\n";
    assert_eq!(
        read_persona(text),
        Err(PersonaError::UnknownKey(5, "persona.colour".to_string()))
    );
}

#[test]
fn r3_duplicate_key_names_table_and_key() {
    let text = "[persona]\nname = \"S\"\nmodel = \"m.glb\"\nheight_m = 1.7\n\
                [clips]\nidle = \"I\"\nwave = \"W\"\nwave = \"Waving\"\n";
    assert_eq!(
        read_persona(text),
        Err(PersonaError::DuplicateKey(8, "clips.wave".to_string()))
    );
}

// -------------------------------------------------------------- R4 values

#[test]
fn r4_string_with_quote_inside_is_bad_value() {
    let text = "[persona]\nname = \"San\\\"cho\"\n";
    assert_eq!(
        read_persona(text),
        Err(PersonaError::BadValue(2, "persona.name".to_string()))
    );
}

#[test]
fn r4_junk_after_value_is_bad_value() {
    let text = "[persona]\nname = \"Sancho\" extra\n";
    assert_eq!(
        read_persona(text),
        Err(PersonaError::BadValue(2, "persona.name".to_string()))
    );
}

#[test]
fn r4_point_arm_needs_exactly_three_strings() {
    let text = "[persona]\nname = \"S\"\nmodel = \"m.glb\"\nheight_m = 1.7\n\
                [bones]\npoint_arm = [\"a\", \"b\"]\n";
    assert_eq!(
        read_persona(text),
        Err(PersonaError::BadValue(6, "bones.point_arm".to_string()))
    );
}

#[test]
fn r4_wrong_shape_for_model_is_bad_value() {
    let text = "[persona]\nname = \"S\"\nmodel = 3\n";
    assert_eq!(
        read_persona(text),
        Err(PersonaError::BadValue(3, "persona.model".to_string()))
    );
}

#[test]
fn r4_lod_unknown_inner_key() {
    let text = "[persona]\nname = \"S\"\nmodel = \"m.glb\"\nheight_m = 1.7\n\
                lod = { modest = \"a\", ultra = \"b\" }\n";
    assert_eq!(
        read_persona(text),
        Err(PersonaError::UnknownKey(5, "persona.lod.ultra".to_string()))
    );
}

#[test]
fn r4_comment_after_a_value_ends_the_line() {
    let text = "[persona]\nname = \"S\"   # the companion\nmodel = \"m.glb\" # his model\n\
                height_m = 1.7 # metres\n\
                [clips]\nidle = \"I\" # first\nwave = \"W\"\npoint = \"P\"\nthink = \"T\"\n\
                worried = \"Wo\"\ncelebrate = \"C\"\nwalk_to = \"Wa\"\nsit = \"Si\"\n\
                carry_board = \"CB\"\n";
    let p = read_persona(text).expect("valid file");
    assert_eq!(p.name, "S");
    assert_eq!(p.model, "m.glb");
    assert_eq!(p.height, Height::Metres(1.7));
}

// ---------------------------------------------------------------- R5 keys

#[test]
fn r5_height_user_string() {
    let text = EXAMPLE.replace("height_m = 1.68", "height_m = \"user\"");
    let p = read_persona(&text).expect("valid file");
    assert_eq!(p.height, Height::User);
}

#[test]
fn r5_height_zero_is_bad_height() {
    let text = EXAMPLE.replace("height_m = 1.68", "height_m = 0");
    assert_eq!(read_persona(&text), Err(PersonaError::BadHeight(5)));
}

#[test]
fn r5_height_negative_is_bad_height() {
    let text = EXAMPLE.replace("height_m = 1.68", "height_m = -1.5");
    assert_eq!(read_persona(&text), Err(PersonaError::BadHeight(5)));
}

#[test]
fn r5_height_word_is_bad_height() {
    let text = EXAMPLE.replace("height_m = 1.68", "height_m = \"tall\"");
    assert_eq!(read_persona(&text), Err(PersonaError::BadHeight(5)));
}

#[test]
fn r5_height_malformed_number_is_bad_height() {
    // "1." has no digit after the '.', ".5" has none before it: neither is a number (R4).
    let a = EXAMPLE.replace("height_m = 1.68", "height_m = 1.");
    assert_eq!(read_persona(&a), Err(PersonaError::BadHeight(5)));
    let b = EXAMPLE.replace("height_m = 1.68", "height_m = .5");
    assert_eq!(read_persona(&b), Err(PersonaError::BadHeight(5)));
}

#[test]
fn r5_empty_name_is_bad_value() {
    let text = "[persona]\nname = \"\"\n";
    assert_eq!(
        read_persona(text),
        Err(PersonaError::BadValue(2, "persona.name".to_string()))
    );
}

#[test]
fn r5_empty_clip_is_bad_value() {
    let text = "[persona]\nname = \"S\"\nmodel = \"m.glb\"\nheight_m = 1.7\n\
                [clips]\nidle = \"\"\n";
    assert_eq!(
        read_persona(text),
        Err(PersonaError::BadValue(6, "clips.idle".to_string()))
    );
}

#[test]
fn r5_strings_are_kept_exactly_as_written() {
    let text = "[persona]\nname = \" Sancho \"\nmodel = \"m.glb\"\nheight_m = 1.7\n\
                [clips]\nidle = \" Idle \"\nwave = \"W\"\npoint = \"P\"\nthink = \"T\"\n\
                worried = \"Wo\"\ncelebrate = \"C\"\nwalk_to = \"Wa\"\nsit = \"Si\"\n\
                carry_board = \"CB\"\n";
    let p = read_persona(text).expect("valid file");
    assert_eq!(p.name, " Sancho ");
    assert_eq!(p.clip("idle"), Some(" Idle "));
}

// -------------------------------------------------------------- R6 missing

#[test]
fn r6_missing_sit_clip() {
    let text = EXAMPLE.replace("sit = \"Sitting Idle\"\n", "");
    assert_eq!(
        read_persona(&text),
        Err(PersonaError::MissingKey("clips.sit".to_string()))
    );
}

#[test]
fn r6_missing_name_comes_first() {
    let text = EXAMPLE.replace("name = \"Sancho\"\n", "");
    assert_eq!(
        read_persona(&text),
        Err(PersonaError::MissingKey("persona.name".to_string()))
    );
}

#[test]
fn r6_missing_clips_table_shows_as_first_clip_key() {
    let text = "[persona]\nname = \"S\"\nmodel = \"m.glb\"\nheight_m = 1.7\n";
    assert_eq!(
        read_persona(text),
        Err(PersonaError::MissingKey("clips.idle".to_string()))
    );
}

#[test]
fn r6_missing_height_comes_before_clips() {
    let text = "[persona]\nname = \"S\"\nmodel = \"m.glb\"\n";
    assert_eq!(
        read_persona(text),
        Err(PersonaError::MissingKey("persona.height_m".to_string()))
    );
}

// ---------------------------------------------------------------- R7 clips

#[test]
fn r7_clips_are_in_clip_keys_order_not_file_order() {
    let text = "[persona]\nname = \"Sancho\"\nmodel = \"sancho.glb\"\nheight_m = 1.68\n\
                portrait = \"sancho_portrait.png\"\n\
                lod = { modest = \"sancho_lod8k.glb\", potato = \"sancho_vat.glb\" }\n\
                [bones]\nhead = \"mixamorig:Head\"\n\
                point_arm = [\"mixamorig:RightArm\", \"mixamorig:RightForeArm\", \
                \"mixamorig:RightHand\"]\n\
                [clips]\ncarry_board = \"Carry Heavy Object\"\nsit = \"Sitting Idle\"\n\
                walk_to = \"Walking\"\ncelebrate = \"Victory\"\n\
                worried = \"Nervously Look Around\"\nthink = \"Thinking\"\npoint = \"Pointing\"\n\
                wave = \"Waving\"\nidle = \"Idle\"\n";
    assert_eq!(read_persona(text), Ok(example_persona()));
}

#[test]
fn r7_optional_clip_shrug_goes_last() {
    let text = format!("{}shrug = \"Shrugging\"\n", EXAMPLE);
    let p = read_persona(&text).expect("valid file");
    assert_eq!(p.clips.len(), 10);
    assert_eq!(p.clips[9], ("shrug".to_string(), "Shrugging".to_string()));
    assert_eq!(p.clip("shrug"), Some("Shrugging"));
}

#[test]
fn r7_clip_lookup_returns_none_for_absent_key() {
    let p = example_persona();
    assert_eq!(p.clip("wave"), Some("Waving"));
    assert_eq!(p.clip("carry_board"), Some("Carry Heavy Object"));
    assert_eq!(p.clip("shrug"), None);
    assert_eq!(p.clip("nonsense"), None);
}

#[test]
fn r7_clip_keys_and_required_count_are_as_documented() {
    assert_eq!(CLIP_KEYS.len(), 13);
    assert_eq!(REQUIRED_CLIPS, 9);
    assert_eq!(
        CLIP_KEYS,
        [
            "idle",
            "wave",
            "point",
            "think",
            "worried",
            "celebrate",
            "walk_to",
            "sit",
            "carry_board",
            "reach_high",
            "crouch_look",
            "measure",
            "shrug"
        ]
    );
}

// ---------------------------------------------------------------- R8 write

#[test]
fn r8_write_the_example_persona() {
    assert_eq!(
        write_persona(&example_persona()),
        Ok(EXAMPLE_WRITTEN.to_string())
    );
}

#[test]
fn r8_write_user_height_without_bones_or_lods() {
    let p = Persona {
        name: "Sancho".to_string(),
        model: "sancho.glb".to_string(),
        height: Height::User,
        portrait: None,
        lod_modest: None,
        lod_potato: None,
        head: None,
        point_arm: None,
        clips: nine_clips(),
    };
    let want = r#"[persona]
name = "Sancho"
model = "sancho.glb"
height_m = "user"

[clips]
idle = "Idle"
wave = "Waving"
point = "Pointing"
think = "Thinking"
worried = "Nervously Look Around"
celebrate = "Victory"
walk_to = "Walking"
sit = "Sitting Idle"
carry_board = "Carry Heavy Object"
"#;
    assert_eq!(write_persona(&p), Ok(want.to_string()));
}

#[test]
fn r8_write_only_the_lod_key_that_is_some() {
    let mut p = example_persona();
    p.lod_potato = None;
    let text = write_persona(&p).expect("writable");
    assert!(text.contains("lod = { modest = \"sancho_lod8k.glb\" }\n"));
    assert!(!text.contains("potato"));
}

#[test]
fn r8_write_refuses_a_string_it_could_not_read_back() {
    let mut p = example_persona();
    p.name = "a\"b".to_string();
    assert_eq!(
        write_persona(&p),
        Err(PersonaError::BadValue(0, "persona.name".to_string()))
    );
    let mut q = example_persona();
    q.model = "a\\b".to_string();
    assert_eq!(
        write_persona(&q),
        Err(PersonaError::BadValue(0, "persona.model".to_string()))
    );
}

#[test]
fn r8_round_trip_on_several_personas() {
    let mut user = example_persona();
    user.height = Height::User;
    let mut bare = example_persona();
    bare.head = None;
    bare.point_arm = None;
    bare.portrait = None;
    bare.lod_modest = None;
    bare.lod_potato = None;
    let mut extra = example_persona();
    extra
        .clips
        .push(("shrug".to_string(), "Shrugging".to_string()));
    extra.lod_potato = None;
    for p in [example_persona(), user, bare, extra] {
        let text = write_persona(&p).expect("writable");
        assert_eq!(read_persona(&text), Ok(p));
    }
}

#[test]
fn r8_round_trip_of_the_example_file() {
    let p = read_persona(EXAMPLE).expect("valid file");
    let text = write_persona(&p).expect("writable");
    assert_eq!(read_persona(&text), Ok(example_persona()));
}

// ------------------------------------------------------------ B1-B3 bones

#[test]
fn b1_mixamo_base() {
    assert_eq!(mixamo_base("mixamorig:Head"), Some("Head"));
    assert_eq!(mixamo_base("mixamorig12:Hips"), Some("Hips"));
    assert_eq!(mixamo_base("mixamorig:"), None);
    assert_eq!(mixamo_base("Head"), None);
    assert_eq!(mixamo_base("MIXAMORIG:Head"), None);
    assert_eq!(mixamo_base("mixamorig"), None);
}

#[test]
fn b2_rigify_table_has_all_twenty_two_entries_in_order() {
    let want: [(&str, &str); 22] = [
        ("DEF-spine", "Hips"),
        ("DEF-spine.001", "Spine"),
        ("DEF-spine.002", "Spine1"),
        ("DEF-spine.003", "Spine2"),
        ("DEF-spine.004", "Neck"),
        ("DEF-spine.006", "Head"),
        ("DEF-shoulder.L", "LeftShoulder"),
        ("DEF-upper_arm.L", "LeftArm"),
        ("DEF-forearm.L", "LeftForeArm"),
        ("DEF-hand.L", "LeftHand"),
        ("DEF-shoulder.R", "RightShoulder"),
        ("DEF-upper_arm.R", "RightArm"),
        ("DEF-forearm.R", "RightForeArm"),
        ("DEF-hand.R", "RightHand"),
        ("DEF-thigh.L", "LeftUpLeg"),
        ("DEF-shin.L", "LeftLeg"),
        ("DEF-foot.L", "LeftFoot"),
        ("DEF-toe.L", "LeftToeBase"),
        ("DEF-thigh.R", "RightUpLeg"),
        ("DEF-shin.R", "RightLeg"),
        ("DEF-foot.R", "RightFoot"),
        ("DEF-toe.R", "RightToeBase"),
    ];
    assert_eq!(RIGIFY_TO_MIXAMO, want);
    for (rigify, mixamo) in want {
        assert_eq!(rigify_to_mixamo(rigify), Some(mixamo));
    }
    assert_eq!(rigify_to_mixamo("DEF-spine.005"), None);
    assert_eq!(rigify_to_mixamo("DEF-finger_index.01.L"), None);
    assert_eq!(rigify_to_mixamo("mixamorig:Head"), None);
}

#[test]
fn b3_base_bone_tries_mixamo_then_rigify() {
    assert_eq!(base_bone("DEF-spine.006"), Some("Head"));
    assert_eq!(base_bone("DEF-spine.005"), None);
    assert_eq!(base_bone("mixamorig:Neck"), Some("Neck"));
    assert_eq!(base_bone("mixamorig7:RightHand"), Some("RightHand"));
    assert_eq!(base_bone("Head"), None);
}

// ------------------------------------------------------- M1-M6 check_model

#[test]
fn m_clean_model_has_no_problems() {
    assert_eq!(
        check_model(&example_persona(), &example_facts(), 30000),
        Vec::<Problem>::new()
    );
}

#[test]
fn m1_too_many_bones() {
    let mut facts = example_facts();
    while facts.bone_names.len() < 65 {
        facts
            .bone_names
            .push(format!("extra{}", facts.bone_names.len()));
    }
    assert_eq!(
        check_model(&example_persona(), &facts, 30000),
        vec![Problem::TooManyBones {
            found: 65,
            limit: MAX_BONES
        }]
    );
}

#[test]
fn m1_sixty_four_bones_is_fine() {
    let mut facts = example_facts();
    while facts.bone_names.len() < 64 {
        facts
            .bone_names
            .push(format!("extra{}", facts.bone_names.len()));
    }
    assert_eq!(
        check_model(&example_persona(), &facts, 30000),
        Vec::<Problem>::new()
    );
}

#[test]
fn m2_too_many_influences() {
    let mut facts = example_facts();
    facts.max_influences = 8;
    assert_eq!(
        check_model(&example_persona(), &facts, 30000),
        vec![Problem::TooManyInfluences {
            found: 8,
            limit: MAX_INFLUENCES
        }]
    );
}

#[test]
fn m3_over_budget() {
    assert_eq!(
        check_model(&example_persona(), &example_facts(), 8000),
        vec![Problem::OverBudget {
            triangles: 30000,
            budget: 8000
        }]
    );
}

#[test]
fn m4_missing_bone_is_reported_by_base_name() {
    let mut facts = example_facts();
    facts.bone_names.retain(|b| b != "mixamorig:RightHand");
    assert_eq!(
        check_model(&example_persona(), &facts, 30000),
        vec![Problem::MissingBone {
            bone: "RightHand".to_string()
        }]
    );
}

#[test]
fn m4_persona_named_bone_must_match_exactly() {
    // The persona names "mixamorig:Head"; a model whose bone is plain "Head" does not satisfy it
    // (base_bone("Head") is None), while the other six mixamorig names still match.
    let mut facts = example_facts();
    facts.bone_names.retain(|b| b != "mixamorig:Head");
    facts.bone_names.push("Head".to_string());
    assert_eq!(
        check_model(&example_persona(), &facts, 30000),
        vec![Problem::MissingBone {
            bone: "Head".to_string()
        }]
    );
}

#[test]
fn m4_rigify_names_satisfy_a_persona_without_bones() {
    let mut persona = example_persona();
    persona.head = None;
    persona.point_arm = None;
    let facts = ModelFacts {
        bone_names: vec![
            "DEF-spine".to_string(),
            "DEF-spine.001".to_string(),
            "DEF-spine.004".to_string(),
            "DEF-spine.006".to_string(),
            "DEF-upper_arm.R".to_string(),
            "DEF-forearm.R".to_string(),
            "DEF-hand.R".to_string(),
        ],
        max_influences: 4,
        triangles: 30000,
        clip_names: nine_clips().into_iter().map(|(_, c)| c).collect(),
        rest_height_m: 1.75,
    };
    assert_eq!(check_model(&persona, &facts, 30000), Vec::<Problem>::new());
}

#[test]
fn m4_every_missing_bone_in_required_order() {
    let mut persona = example_persona();
    persona.head = None;
    persona.point_arm = None;
    let facts = ModelFacts {
        bone_names: Vec::new(),
        max_influences: 4,
        triangles: 30000,
        clip_names: nine_clips().into_iter().map(|(_, c)| c).collect(),
        rest_height_m: 1.75,
    };
    let want: Vec<Problem> = REQUIRED_BONES
        .iter()
        .map(|b| Problem::MissingBone {
            bone: b.to_string(),
        })
        .collect();
    assert_eq!(check_model(&persona, &facts, 30000), want);
}

#[test]
fn m5_missing_clip() {
    let mut facts = example_facts();
    facts.clip_names.retain(|c| c != "Victory");
    assert_eq!(
        check_model(&example_persona(), &facts, 30000),
        vec![Problem::MissingClip {
            key: "celebrate".to_string(),
            clip: "Victory".to_string()
        }]
    );
}

#[test]
fn m6_odd_height_in_centimetres() {
    let mut facts = example_facts();
    facts.rest_height_m = 168.0;
    assert_eq!(
        check_model(&example_persona(), &facts, 30000),
        vec![Problem::OddHeight {
            rest_height_m: 168.0
        }]
    );
}

#[test]
fn m6_height_bounds_are_inclusive() {
    let mut low = example_facts();
    low.rest_height_m = MIN_REST_HEIGHT_M;
    assert_eq!(
        check_model(&example_persona(), &low, 30000),
        Vec::<Problem>::new()
    );
    let mut high = example_facts();
    high.rest_height_m = MAX_REST_HEIGHT_M;
    assert_eq!(
        check_model(&example_persona(), &high, 30000),
        Vec::<Problem>::new()
    );
    let mut under = example_facts();
    under.rest_height_m = 0.49;
    assert_eq!(
        check_model(&example_persona(), &under, 30000),
        vec![Problem::OddHeight {
            rest_height_m: 0.49
        }]
    );
}

#[test]
fn m6_non_finite_height_is_odd() {
    let mut facts = example_facts();
    facts.rest_height_m = f64::NAN;
    let problems = check_model(&example_persona(), &facts, 30000);
    assert_eq!(problems.len(), 1);
    match &problems[0] {
        Problem::OddHeight { rest_height_m } => assert!(rest_height_m.is_nan()),
        other => panic!("expected OddHeight, got {:?}", other),
    }
}

#[test]
fn m_all_six_problems_in_m1_to_m6_order() {
    let mut facts = example_facts();
    facts.bone_names.retain(|b| b != "mixamorig:RightHand");
    while facts.bone_names.len() < 65 {
        facts
            .bone_names
            .push(format!("extra{}", facts.bone_names.len()));
    }
    facts.max_influences = 8;
    facts.clip_names.retain(|c| c != "Victory");
    facts.rest_height_m = 168.0;
    assert_eq!(
        check_model(&example_persona(), &facts, 8000),
        vec![
            Problem::TooManyBones {
                found: 65,
                limit: 64
            },
            Problem::TooManyInfluences { found: 8, limit: 4 },
            Problem::OverBudget {
                triangles: 30000,
                budget: 8000
            },
            Problem::MissingBone {
                bone: "RightHand".to_string()
            },
            Problem::MissingClip {
                key: "celebrate".to_string(),
                clip: "Victory".to_string()
            },
            Problem::OddHeight {
                rest_height_m: 168.0
            },
        ]
    );
}

// ------------------------------------------------------------ no panics

#[test]
fn no_input_panics() {
    assert_eq!(
        read_persona(""),
        Err(PersonaError::MissingKey("persona.name".to_string()))
    );
    assert!(read_persona("\"").is_err());
    assert!(read_persona("[").is_err());
    assert!(read_persona("=").is_err());
    assert!(read_persona("[persona]\nname = \n").is_err());
    assert!(read_persona("[persona]\nname = \"a\n").is_err());
    assert!(read_persona("[persona]\npoint_arm = []\n").is_err());
    assert!(read_persona("[persona]\nlod = {}\n").is_err());
    let many = "# comment\n".repeat(10_000);
    assert!(read_persona(&many).is_err());
    let keys = "[persona]\n".to_string() + &"name = \"x\"\n".repeat(10_000);
    assert!(read_persona(&keys).is_err());
}
