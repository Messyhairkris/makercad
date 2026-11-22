//! Contract tests for mk_sancho::persona, written by Gordon as a PA (the Crew) from
//! crew/contracts/mk_persona.md. They were the hidden tests of campaign mk-core-15.

use mk_sancho::persona::{
    base_bone, check_model, mixamo_base, read_persona, rigify_to_mixamo, write_persona, Height,
    ModelFacts, Persona, PersonaError, Problem, CLIP_KEYS, MAX_BONES, MAX_INFLUENCES,
    MAX_REST_HEIGHT_M, MIN_REST_HEIGHT_M, REQUIRED_BONES, REQUIRED_CLIPS, RIGIFY_TO_MIXAMO,
};

/// Doc 07 section 7.3.1's example persona, comments included. The arrow in the
/// [clips] comment is written as "->" so this file stays ASCII.
const EXAMPLE: &str = "\
[persona]
name = \"Sancho\"
model = \"sancho.glb\"
height_m = 1.68
portrait = \"sancho_portrait.png\"
lod = { modest = \"sancho_lod8k.glb\", potato = \"sancho_vat.glb\" }

[bones]                    # only needed if names aren't Mixamo-standard
head = \"mixamorig:Head\"
point_arm = [\"mixamorig:RightArm\", \"mixamorig:RightForeArm\", \"mixamorig:RightHand\"]

[clips]
# his events -> the clip in the model that plays each
idle = \"Idle\"
wave = \"Waving\"
point = \"Pointing\"
think = \"Thinking\"
worried = \"Nervously Look Around\"
celebrate = \"Victory\"
walk_to = \"Walking\"
sit = \"Sitting Idle\"
carry_board = \"Carry Heavy Object\"
";

/// The smallest complete persona: no [bones] table, no optional keys.
const MINIMAL: &str = "\
[persona]
name = \"S\"
model = \"s.glb\"
height_m = 1.7

[clips]
idle = \"Idle\"
wave = \"Wave\"
point = \"Point\"
think = \"Think\"
worried = \"Worried\"
celebrate = \"Celebrate\"
walk_to = \"Walk\"
sit = \"Sit\"
carry_board = \"Carry\"
";

const EXAMPLE_CLIPS: [(&str, &str); 9] = [
    ("idle", "Idle"),
    ("wave", "Waving"),
    ("point", "Pointing"),
    ("think", "Thinking"),
    ("worried", "Nervously Look Around"),
    ("celebrate", "Victory"),
    ("walk_to", "Walking"),
    ("sit", "Sitting Idle"),
    ("carry_board", "Carry Heavy Object"),
];

fn example_clips() -> Vec<(String, String)> {
    EXAMPLE_CLIPS
        .iter()
        .map(|(key, clip)| (key.to_string(), clip.to_string()))
        .collect()
}

fn example_clip_names() -> Vec<String> {
    EXAMPLE_CLIPS
        .iter()
        .map(|(_, clip)| clip.to_string())
        .collect()
}

/// The persona the doc's example file reads as.
fn example() -> Persona {
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
        clips: example_clips(),
    }
}

/// Facts for a model that is fine: the seven required bones, 4 influences,
/// 30k triangles, all nine clips, 1.75 m tall.
fn good_facts() -> ModelFacts {
    ModelFacts {
        bone_names: REQUIRED_BONES
            .iter()
            .map(|bone| format!("mixamorig:{}", bone))
            .collect(),
        max_influences: MAX_INFLUENCES,
        triangles: 30_000,
        clip_names: example_clip_names(),
        rest_height_m: 1.75,
    }
}

#[test]
fn r1_blank_lines_and_comments_are_skipped() {
    let text = format!(
        "# h\u{e9}llo, Sancho\n\n# another\n{}\n# trailing\n",
        MINIMAL
    );
    assert!(read_persona(&text) == read_persona(MINIMAL));
}

#[test]
fn r1_crlf_line_ends_are_accepted() {
    let text = MINIMAL.replace('\n', "\r\n");
    assert!(read_persona(&text) == read_persona(MINIMAL));
    assert!(read_persona(EXAMPLE.replace('\n', "\r\n").as_str()) == Ok(example()));
}

#[test]
fn r1_lines_are_trimmed() {
    let text = MINIMAL.replace("name = \"S\"", "   name   =   \"S\"   ");
    assert!(read_persona(&text) == read_persona(MINIMAL));
}

#[test]
fn r2_header_may_carry_a_comment() {
    assert!(
        read_persona(&MINIMAL.replace("[clips]", "[clips]   # the nine, then the rest"))
            == read_persona(MINIMAL)
    );
    assert!(
        read_persona(&MINIMAL.replace("[persona]", "[persona]  # he is Sancho"))
            == read_persona(MINIMAL)
    );
    assert!(
        read_persona(&MINIMAL.replace("[persona]", "[persona]  # h\u{e9}llo"))
            == read_persona(MINIMAL)
    );
    // The doc's own example carries one after [bones].
    assert!(read_persona(EXAMPLE) == Ok(example()));
}

#[test]
fn r2_header_must_be_exactly_bracketed() {
    assert!(read_persona(&MINIMAL.replace("[clips]", "[clips")) == Err(PersonaError::BadLine(6)));
    assert!(read_persona(&MINIMAL.replace("[clips]", "clips]")) == Err(PersonaError::BadLine(6)));
    assert!(
        read_persona(&MINIMAL.replace("[clips]", "[clips] extra")) == Err(PersonaError::BadLine(6))
    );
}

#[test]
fn r2_unknown_table() {
    assert!(
        read_persona(&MINIMAL.replace("[clips]", "[voice]"))
            == Err(PersonaError::UnknownTable(6, "voice".to_string()))
    );
}

#[test]
fn r2_duplicate_table() {
    let twice = format!("{}[clips]\n", MINIMAL);
    assert!(read_persona(&twice) == Err(PersonaError::DuplicateTable(16, "clips".to_string())));
}

#[test]
fn r3_key_before_any_header_is_bad_line() {
    let text = format!("name = \"Sancho\"\n{}", MINIMAL);
    assert!(read_persona(&text) == Err(PersonaError::BadLine(1)));
}

#[test]
fn r3_key_must_be_an_identifier() {
    assert!(
        read_persona(&MINIMAL.replace("name = \"S\"", "na me = \"S\""))
            == Err(PersonaError::BadLine(2))
    );
    assert!(
        read_persona(&MINIMAL.replace("name = \"S\"", "na-me = \"S\""))
            == Err(PersonaError::BadLine(2))
    );
    assert!(
        read_persona(&MINIMAL.replace("name = \"S\"", "\u{e9} = \"S\""))
            == Err(PersonaError::BadLine(2))
    );
}

#[test]
fn r3_key_line_needs_an_equals_sign() {
    assert!(
        read_persona(&MINIMAL.replace("name = \"S\"", "name \"S\""))
            == Err(PersonaError::BadLine(2))
    );
}

#[test]
fn r3_unknown_key_in_a_table() {
    assert!(
        read_persona(&MINIMAL.replace("name = \"S\"", "colour = \"red\""))
            == Err(PersonaError::UnknownKey(2, "persona.colour".to_string()))
    );
    assert!(
        read_persona(&MINIMAL.replace("name = \"S\"", "head = \"H\""))
            == Err(PersonaError::UnknownKey(2, "persona.head".to_string()))
    );
    let extra_clip = MINIMAL.replace(
        "carry_board = \"Carry\"\n",
        "carry_board = \"Carry\"\nshout = \"Shout\"\n",
    );
    assert!(
        read_persona(&extra_clip) == Err(PersonaError::UnknownKey(16, "clips.shout".to_string()))
    );
}

#[test]
fn r3_duplicate_key_in_a_table() {
    let twice = MINIMAL.replace("name = \"S\"\n", "name = \"S\"\nname = \"S\"\n");
    assert!(read_persona(&twice) == Err(PersonaError::DuplicateKey(3, "persona.name".to_string())));
    let waved = EXAMPLE.replace("wave = \"Waving\"", "wave = \"Waving\"\nwave = \"Waving\"");
    assert!(read_persona(&waved) == Err(PersonaError::DuplicateKey(16, "clips.wave".to_string())));
}

#[test]
fn r4_string_has_no_escapes() {
    // The file then holds: name = "San\"cho"
    assert!(
        read_persona(&MINIMAL.replace("name = \"S\"", "name = \"San\\\"cho\""))
            == Err(PersonaError::BadValue(2, "persona.name".to_string()))
    );
    // The file then holds: name = "San\cho"
    assert!(
        read_persona(&MINIMAL.replace("name = \"S\"", "name = \"San\\cho\""))
            == Err(PersonaError::BadValue(2, "persona.name".to_string()))
    );
    assert!(
        read_persona(&MINIMAL.replace("name = \"S\"", "name = \"Sancho"))
            == Err(PersonaError::BadValue(2, "persona.name".to_string()))
    );
}

#[test]
fn r4_value_must_end_the_line() {
    assert!(
        read_persona(&MINIMAL.replace("name = \"S\"", "name = \"Sancho\" extra"))
            == Err(PersonaError::BadValue(2, "persona.name".to_string()))
    );
    assert!(
        read_persona(&MINIMAL.replace("model = \"s.glb\"", "model = 3"))
            == Err(PersonaError::BadValue(3, "persona.model".to_string()))
    );
}

#[test]
fn r4_array_needs_one_item_and_no_trailing_comma() {
    let empty = MINIMAL.replace("[clips]", "[bones]\npoint_arm = []\n\n[clips]");
    assert!(read_persona(&empty) == Err(PersonaError::BadValue(7, "bones.point_arm".to_string())));
    let trailing = MINIMAL.replace(
        "[clips]",
        "[bones]\npoint_arm = [\"a\", \"b\", \"c\",]\n\n[clips]",
    );
    assert!(
        read_persona(&trailing) == Err(PersonaError::BadValue(7, "bones.point_arm".to_string()))
    );
    let two = MINIMAL.replace("[clips]", "[bones]\npoint_arm = [\"a\", \"b\"]\n\n[clips]");
    assert!(read_persona(&two) == Err(PersonaError::BadValue(7, "bones.point_arm".to_string())));
}

#[test]
fn r4_inline_table_needs_key_value_pairs() {
    let not_a_table = MINIMAL.replace("height_m = 1.7", "height_m = 1.7\nlod = \"x\"");
    assert!(
        read_persona(&not_a_table) == Err(PersonaError::BadValue(5, "persona.lod".to_string()))
    );
    let not_a_string = MINIMAL.replace("height_m = 1.7", "height_m = 1.7\nlod = { modest = 3 }");
    assert!(
        read_persona(&not_a_string) == Err(PersonaError::BadValue(5, "persona.lod".to_string()))
    );
}

#[test]
fn r4_number_has_digits_on_both_sides_of_the_point() {
    assert!(
        read_persona(&MINIMAL.replace("height_m = 1.7", "height_m = 1."))
            == Err(PersonaError::BadHeight(4))
    );
    assert!(
        read_persona(&MINIMAL.replace("height_m = 1.7", "height_m = .5"))
            == Err(PersonaError::BadHeight(4))
    );
    assert!(
        read_persona(&MINIMAL.replace("height_m = 1.7", "height_m = -"))
            == Err(PersonaError::BadHeight(4))
    );
}

#[test]
fn r4_comment_after_a_value() {
    // A '#' inside the quotes is part of the string, not a comment.
    let hashed = MINIMAL.replace(
        "name = \"S\"",
        "name = \"S#1\"   # the hash is in the string",
    );
    assert!(read_persona(&hashed).map(|persona| persona.name) == Ok("S#1".to_string()));
    assert!(
        read_persona(&MINIMAL.replace("height_m = 1.7", "height_m = 1.7   # metres"))
            == read_persona(MINIMAL)
    );
}

#[test]
fn r5_name_must_not_be_empty() {
    assert!(
        read_persona(&EXAMPLE.replace("name = \"Sancho\"", "name = \"\""))
            == Err(PersonaError::BadValue(2, "persona.name".to_string()))
    );
    assert!(
        read_persona(&EXAMPLE.replace("name = \"Sancho\"", "name = \"   \""))
            == Err(PersonaError::BadValue(2, "persona.name".to_string()))
    );
}

#[test]
fn r5_model_must_be_a_non_empty_string() {
    assert!(
        read_persona(&EXAMPLE.replace("model = \"sancho.glb\"", "model = 3"))
            == Err(PersonaError::BadValue(3, "persona.model".to_string()))
    );
    assert!(
        read_persona(&EXAMPLE.replace("model = \"sancho.glb\"", "model = \"\""))
            == Err(PersonaError::BadValue(3, "persona.model".to_string()))
    );
}

#[test]
fn r5_clip_must_not_be_empty() {
    assert!(
        read_persona(&EXAMPLE.replace("idle = \"Idle\"", "idle = \"\""))
            == Err(PersonaError::BadValue(14, "clips.idle".to_string()))
    );
}

#[test]
fn r5_height_metres() {
    assert!(read_persona(EXAMPLE).map(|persona| persona.height) == Ok(Height::Metres(1.68)));
    let two = MINIMAL.replace("height_m = 1.7", "height_m = 2");
    assert!(read_persona(&two).map(|persona| persona.height) == Ok(Height::Metres(2.0)));
}

#[test]
fn r5_height_user() {
    let text = EXAMPLE.replace("height_m = 1.68", "height_m = \"user\"");
    assert!(read_persona(&text).map(|persona| persona.height) == Ok(Height::User));
}

#[test]
fn r5_height_zero_is_bad() {
    assert!(
        read_persona(&EXAMPLE.replace("height_m = 1.68", "height_m = 0"))
            == Err(PersonaError::BadHeight(4))
    );
}

#[test]
fn r5_height_negative_is_bad() {
    assert!(
        read_persona(&EXAMPLE.replace("height_m = 1.68", "height_m = -1.5"))
            == Err(PersonaError::BadHeight(4))
    );
}

#[test]
fn r5_height_word_is_bad() {
    assert!(
        read_persona(&EXAMPLE.replace("height_m = 1.68", "height_m = \"tall\""))
            == Err(PersonaError::BadHeight(4))
    );
}

#[test]
fn r5_lod_keys_are_modest_and_potato() {
    let text = MINIMAL.replace(
        "height_m = 1.7",
        "height_m = 1.7\nlod = { modest = \"a\", ultra = \"b\" }",
    );
    assert!(
        read_persona(&text) == Err(PersonaError::UnknownKey(5, "persona.lod.ultra".to_string()))
    );
}

#[test]
fn r5_lod_holds_only_the_keys_given() {
    let both = MINIMAL.replace(
        "height_m = 1.7",
        "height_m = 1.7\nlod = { modest = \"a\", potato = \"b\" }",
    );
    assert!(
        read_persona(&both).map(|persona| (persona.lod_modest, persona.lod_potato))
            == Ok((Some("a".to_string()), Some("b".to_string())))
    );
    let one = MINIMAL.replace("height_m = 1.7", "height_m = 1.7\nlod = { potato = \"b\" }");
    assert!(
        read_persona(&one).map(|persona| (persona.lod_modest, persona.lod_potato))
            == Ok((None, Some("b".to_string())))
    );
}

#[test]
fn r5_point_arm_needs_exactly_three_strings() {
    let short = MINIMAL.replace("[clips]", "[bones]\npoint_arm = [\"a\", \"b\"]\n\n[clips]");
    assert!(read_persona(&short) == Err(PersonaError::BadValue(7, "bones.point_arm".to_string())));
    let not_array = MINIMAL.replace("[clips]", "[bones]\npoint_arm = \"a\"\n\n[clips]");
    assert!(
        read_persona(&not_array) == Err(PersonaError::BadValue(7, "bones.point_arm".to_string()))
    );
}

#[test]
fn r5_portrait_is_a_string() {
    let text = MINIMAL.replace("height_m = 1.7", "height_m = 1.7\nportrait = \"p.png\"");
    assert!(read_persona(&text).map(|persona| persona.portrait) == Ok(Some("p.png".to_string())));
    let wrong = MINIMAL.replace("height_m = 1.7", "height_m = 1.7\nportrait = 3");
    assert!(read_persona(&wrong) == Err(PersonaError::BadValue(5, "persona.portrait".to_string())));
}

#[test]
fn r6_missing_name_is_reported_first() {
    let text = MINIMAL.replace("name = \"S\"\n", "");
    assert!(read_persona(&text) == Err(PersonaError::MissingKey("persona.name".to_string())));
}

#[test]
fn r6_missing_model() {
    let text = MINIMAL.replace("model = \"s.glb\"\n", "");
    assert!(read_persona(&text) == Err(PersonaError::MissingKey("persona.model".to_string())));
}

#[test]
fn r6_missing_height() {
    let text = MINIMAL.replace("height_m = 1.7\n", "");
    assert!(read_persona(&text) == Err(PersonaError::MissingKey("persona.height_m".to_string())));
}

#[test]
fn r6_missing_clips_table_shows_as_idle() {
    let text = "[persona]\nname = \"S\"\nmodel = \"s.glb\"\nheight_m = 1.7\n";
    assert!(read_persona(text) == Err(PersonaError::MissingKey("clips.idle".to_string())));
}

#[test]
fn r6_missing_sit() {
    let text = EXAMPLE.replace("sit = \"Sitting Idle\"\n", "");
    assert!(read_persona(&text) == Err(PersonaError::MissingKey("clips.sit".to_string())));
}

#[test]
fn r6_optional_clips_are_not_required() {
    assert!(read_persona(MINIMAL).is_ok());
}

#[test]
fn r6_bones_table_may_be_absent() {
    let without = EXAMPLE.replace(
        "[bones]                    # only needed if names aren't Mixamo-standard\nhead = \"mixamorig:Head\"\npoint_arm = [\"mixamorig:RightArm\", \"mixamorig:RightForeArm\", \"mixamorig:RightHand\"]\n\n",
        "",
    );
    assert!(
        read_persona(&without).map(|persona| (persona.head, persona.point_arm)) == Ok((None, None))
    );
}

#[test]
fn r7_clips_are_kept_in_clip_keys_order() {
    let reversed = "\
[persona]
name = \"S\"
model = \"s.glb\"
height_m = 1.7

[clips]
carry_board = \"Carry\"
sit = \"Sit\"
walk_to = \"Walk\"
celebrate = \"Celebrate\"
worried = \"Worried\"
think = \"Think\"
point = \"Point\"
wave = \"Wave\"
idle = \"Idle\"
";
    assert!(read_persona(reversed) == read_persona(MINIMAL));
    let persona = read_persona(reversed).expect("the reversed example reads");
    let keys: Vec<&str> = persona.clips.iter().map(|(key, _)| key.as_str()).collect();
    assert!(keys == CLIP_KEYS[..REQUIRED_CLIPS].to_vec());
}

#[test]
fn r7_clip_lookup() {
    let persona = read_persona(EXAMPLE).expect("the example reads");
    assert!(persona.clip("idle") == Some("Idle"));
    assert!(persona.clip("carry_board") == Some("Carry Heavy Object"));
    assert!(persona.clip("shrug").is_none());
    assert!(persona.clip("nonsense").is_none());
}

#[test]
fn r7_optional_clip_is_kept_last() {
    let text = EXAMPLE.replace(
        "carry_board = \"Carry Heavy Object\"\n",
        "carry_board = \"Carry Heavy Object\"\nshrug = \"Shrugging\"\n",
    );
    let persona = read_persona(&text).expect("the ten-clip example reads");
    assert!(persona.clips.len() == 10);
    assert!(persona.clips[9] == ("shrug".to_string(), "Shrugging".to_string()));
    assert!(persona.clip("shrug") == Some("Shrugging"));
}

#[test]
fn r8_writes_the_documented_text() {
    let persona = Persona {
        name: "Sancho".to_string(),
        model: "sancho.glb".to_string(),
        height: Height::User,
        portrait: None,
        lod_modest: Some("a.glb".to_string()),
        lod_potato: None,
        head: None,
        point_arm: None,
        clips: vec![
            ("idle".to_string(), "Idle".to_string()),
            ("wave".to_string(), "Waving".to_string()),
        ],
    };
    let expected = "\
[persona]
name = \"Sancho\"
model = \"sancho.glb\"
height_m = \"user\"
lod = { modest = \"a.glb\" }

[clips]
idle = \"Idle\"
wave = \"Waving\"
";
    assert!(write_persona(&persona) == Ok(expected.to_string()));
}

#[test]
fn r8_writes_metres_bones_and_portrait() {
    let persona = Persona {
        name: "S".to_string(),
        model: "s.glb".to_string(),
        height: Height::Metres(1.905),
        portrait: Some("p.png".to_string()),
        lod_modest: None,
        lod_potato: None,
        head: Some("mixamorig:Head".to_string()),
        point_arm: Some(["a".to_string(), "b".to_string(), "c".to_string()]),
        clips: vec![("idle".to_string(), "Idle".to_string())],
    };
    let expected = "\
[persona]
name = \"S\"
model = \"s.glb\"
height_m = 1.905
portrait = \"p.png\"

[bones]
head = \"mixamorig:Head\"
point_arm = [\"a\", \"b\", \"c\"]

[clips]
idle = \"Idle\"
";
    assert!(write_persona(&persona) == Ok(expected.to_string()));
}

#[test]
fn r8_optional_parts_are_omitted() {
    let persona = Persona {
        name: "S".to_string(),
        model: "s.glb".to_string(),
        height: Height::Metres(1.7),
        portrait: None,
        lod_modest: None,
        lod_potato: None,
        head: None,
        point_arm: None,
        clips: vec![("idle".to_string(), "Idle".to_string())],
    };
    let text = write_persona(&persona).expect("the persona writes");
    assert!(!text.contains("[bones]"));
    assert!(!text.contains("portrait"));
    assert!(!text.contains("lod"));
    assert!(text.ends_with('\n'));
    assert!(!text.contains('\r'));
    assert!(text.matches("\n\n").count() == 1);
}

#[test]
fn r8_refuses_strings_it_could_not_read_back() {
    let mut persona = example();
    persona.name = "a\"b".to_string();
    assert!(write_persona(&persona) == Err(PersonaError::BadValue(0, "persona.name".to_string())));

    let mut persona = example();
    persona.model = "a\\b".to_string();
    assert!(write_persona(&persona) == Err(PersonaError::BadValue(0, "persona.model".to_string())));

    let mut persona = example();
    persona.clips[0].1 = "Idle\n".to_string();
    assert!(write_persona(&persona) == Err(PersonaError::BadValue(0, "clips.idle".to_string())));
}

#[test]
fn roundtrip_example() {
    let first = read_persona(EXAMPLE).expect("the example reads");
    let text = write_persona(&first).expect("the example writes");
    assert!(read_persona(&text) == Ok(first));
}

#[test]
fn roundtrip_user_height_and_no_bones() {
    let persona = Persona {
        name: "Sancho".to_string(),
        model: "sancho.glb".to_string(),
        height: Height::User,
        portrait: Some("p.png".to_string()),
        lod_modest: None,
        lod_potato: None,
        head: None,
        point_arm: None,
        clips: example_clips(),
    };
    let text = write_persona(&persona).expect("it writes");
    assert!(read_persona(&text) == Ok(persona));
}

#[test]
fn roundtrip_one_lod_key() {
    let mut persona = example();
    persona.lod_potato = None;
    let text = write_persona(&persona).expect("it writes");
    assert!(read_persona(&text) == Ok(persona));
}

#[test]
fn roundtrip_height_1_905() {
    let mut persona = example();
    persona.height = Height::Metres(1.905);
    let text = write_persona(&persona).expect("it writes");
    assert!(read_persona(&text) == Ok(persona));
}

#[test]
fn roundtrip_optional_clips() {
    let mut persona = example();
    persona
        .clips
        .push(("reach_high".to_string(), "Reaching".to_string()));
    persona
        .clips
        .push(("crouch_look".to_string(), "Crouching".to_string()));
    persona
        .clips
        .push(("measure".to_string(), "Measuring".to_string()));
    persona
        .clips
        .push(("shrug".to_string(), "Shrugging".to_string()));
    let text = write_persona(&persona).expect("it writes");
    assert!(read_persona(&text) == Ok(persona));
}

#[test]
fn b1_mixamo_base_reads_the_suffix() {
    assert!(mixamo_base("mixamorig:Head") == Some("Head"));
    assert!(mixamo_base("mixamorig12:Hips") == Some("Hips"));
    assert!(mixamo_base("mixamorig007:LeftArm") == Some("LeftArm"));
}

#[test]
fn b1_mixamo_base_rejects_other_names() {
    assert!(mixamo_base("mixamorig:").is_none());
    assert!(mixamo_base("Head").is_none());
    assert!(mixamo_base("MIXAMORIG:Head").is_none());
    assert!(mixamo_base("mixamorigHead").is_none());
    assert!(mixamo_base("").is_none());
}

#[test]
fn b2_rigify_table() {
    assert!(RIGIFY_TO_MIXAMO.len() == 22);
    assert!(rigify_to_mixamo("DEF-spine") == Some("Hips"));
    assert!(rigify_to_mixamo("DEF-spine.004") == Some("Neck"));
    assert!(rigify_to_mixamo("DEF-spine.006") == Some("Head"));
    assert!(rigify_to_mixamo("DEF-upper_arm.R") == Some("RightArm"));
    assert!(rigify_to_mixamo("DEF-forearm.R") == Some("RightForeArm"));
    assert!(rigify_to_mixamo("DEF-hand.R") == Some("RightHand"));
    assert!(rigify_to_mixamo("DEF-toe.L") == Some("LeftToeBase"));
    assert!(rigify_to_mixamo("DEF-thigh.R") == Some("RightUpLeg"));
    assert!(rigify_to_mixamo("DEF-spine.005").is_none());
    assert!(rigify_to_mixamo("DEF-spine.001").is_some());
    assert!(rigify_to_mixamo("def-spine").is_none());
    assert!(rigify_to_mixamo("").is_none());
}

#[test]
fn b2_rigify_table_order() {
    assert!(RIGIFY_TO_MIXAMO[0] == ("DEF-spine", "Hips"));
    assert!(RIGIFY_TO_MIXAMO[21] == ("DEF-toe.R", "RightToeBase"));
    assert!(RIGIFY_TO_MIXAMO
        .iter()
        .all(|(_, mixamo)| mixamo.chars().all(|c| c.is_ascii_alphanumeric())));
}

#[test]
fn b3_base_bone_tries_mixamo_first() {
    assert!(base_bone("mixamorig:Neck") == Some("Neck"));
    assert!(base_bone("DEF-spine.006") == Some("Head"));
    assert!(base_bone("DEF-spine.005").is_none());
    assert!(base_bone("Head").is_none());
    assert!(base_bone("mixamorig:").is_none());
}

#[test]
fn m_clean_model_has_no_problems() {
    assert!(check_model(&example(), &good_facts(), 30_000).is_empty());
}

#[test]
fn m1_too_many_bones() {
    let mut facts = good_facts();
    while facts.bone_names.len() < MAX_BONES {
        facts
            .bone_names
            .push(format!("extra{}", facts.bone_names.len()));
    }
    assert!(check_model(&example(), &facts, 30_000).is_empty());
    facts.bone_names.push("one_more".to_string());
    assert!(
        check_model(&example(), &facts, 30_000)
            == vec![Problem::TooManyBones {
                found: 65,
                limit: MAX_BONES,
            }]
    );
}

#[test]
fn m2_too_many_influences() {
    let mut facts = good_facts();
    facts.max_influences = MAX_INFLUENCES;
    assert!(check_model(&example(), &facts, 30_000).is_empty());
    facts.max_influences = 8;
    assert!(
        check_model(&example(), &facts, 30_000)
            == vec![Problem::TooManyInfluences {
                found: 8,
                limit: MAX_INFLUENCES,
            }]
    );
}

#[test]
fn m3_over_budget() {
    let facts = good_facts();
    assert!(check_model(&example(), &facts, facts.triangles).is_empty());
    assert!(
        check_model(&example(), &facts, 8_000)
            == vec![Problem::OverBudget {
                triangles: 30_000,
                budget: 8_000,
            }]
    );
}

#[test]
fn m4_missing_bone() {
    let mut facts = good_facts();
    facts
        .bone_names
        .retain(|bone| bone != "mixamorig:RightHand");
    assert!(
        check_model(&example(), &facts, 30_000)
            == vec![Problem::MissingBone {
                bone: "RightHand".to_string(),
            }]
    );
}

#[test]
fn m4_reports_missing_bones_in_order() {
    let mut facts = good_facts();
    facts.bone_names.clear();
    let expected: Vec<Problem> = REQUIRED_BONES
        .iter()
        .map(|bone| Problem::MissingBone {
            bone: bone.to_string(),
        })
        .collect();
    assert!(check_model(&example(), &facts, 30_000) == expected);
}

#[test]
fn m4_named_and_rigify_bones_are_accepted() {
    // The persona's own names, found exactly in the model.
    let mut persona = example();
    persona.head = Some("SANCHO_Head".to_string());
    persona.point_arm = Some([
        "SANCHO_A".to_string(),
        "SANCHO_B".to_string(),
        "SANCHO_C".to_string(),
    ]);
    let mut facts = good_facts();
    facts.bone_names = vec![
        "SANCHO_Head".to_string(),
        "SANCHO_A".to_string(),
        "SANCHO_B".to_string(),
        "SANCHO_C".to_string(),
        "mixamorig:Hips".to_string(),
        "mixamorig:Spine".to_string(),
        "mixamorig:Neck".to_string(),
    ];
    assert!(check_model(&persona, &facts, 30_000).is_empty());

    // The persona names another bone, but the model's Mixamo name is enough.
    let mut persona = example();
    persona.head = Some("SANCHO_Head".to_string());
    assert!(check_model(&persona, &good_facts(), 30_000).is_empty());

    // A persona without [bones] and a Rigify-rigged model: still no MissingBone.
    let mut persona = example();
    persona.head = None;
    persona.point_arm = None;
    let mut facts = good_facts();
    facts.bone_names = [
        "DEF-spine",
        "DEF-spine.001",
        "DEF-spine.004",
        "DEF-spine.006",
        "DEF-upper_arm.R",
        "DEF-forearm.R",
        "DEF-hand.R",
    ]
    .iter()
    .map(|name| name.to_string())
    .collect();
    assert!(check_model(&persona, &facts, 30_000).is_empty());
}

#[test]
fn m5_missing_clip() {
    let mut facts = good_facts();
    facts.clip_names.retain(|clip| clip != "Victory");
    assert!(
        check_model(&example(), &facts, 30_000)
            == vec![Problem::MissingClip {
                key: "celebrate".to_string(),
                clip: "Victory".to_string(),
            }]
    );
}

#[test]
fn m5_reports_missing_clips_in_persona_order() {
    let mut facts = good_facts();
    facts.clip_names.clear();
    let keys: Vec<String> = check_model(&example(), &facts, 30_000)
        .iter()
        .filter_map(|problem| match problem {
            Problem::MissingClip { key, .. } => Some(key.clone()),
            _ => None,
        })
        .collect();
    let expected: Vec<String> = example().clips.iter().map(|(key, _)| key.clone()).collect();
    assert!(keys == expected);
}

#[test]
fn m6_odd_height() {
    let mut facts = good_facts();
    facts.rest_height_m = 168.0;
    assert!(
        check_model(&example(), &facts, 30_000)
            == vec![Problem::OddHeight {
                rest_height_m: 168.0
            }]
    );
    facts.rest_height_m = MIN_REST_HEIGHT_M;
    assert!(check_model(&example(), &facts, 30_000).is_empty());
    facts.rest_height_m = MAX_REST_HEIGHT_M;
    assert!(check_model(&example(), &facts, 30_000).is_empty());
    facts.rest_height_m = MIN_REST_HEIGHT_M - 0.01;
    assert!(
        check_model(&example(), &facts, 30_000)
            == vec![Problem::OddHeight {
                rest_height_m: MIN_REST_HEIGHT_M - 0.01,
            }]
    );
}

#[test]
fn m6_not_a_finite_height() {
    let mut facts = good_facts();
    facts.rest_height_m = f64::NAN;
    let problems = check_model(&example(), &facts, 30_000);
    assert!(problems.len() == 1);
    assert!(matches!(problems[0], Problem::OddHeight { .. }));
    facts.rest_height_m = f64::INFINITY;
    assert!(matches!(
        check_model(&example(), &facts, 30_000).as_slice(),
        [Problem::OddHeight { .. }]
    ));
}

#[test]
fn m_all_six_in_order() {
    let mut facts = good_facts();
    let mut bones: Vec<String> = REQUIRED_BONES
        .iter()
        .filter(|bone| **bone != "RightHand")
        .map(|bone| format!("mixamorig:{}", bone))
        .collect();
    while bones.len() < 65 {
        bones.push(format!("extra{}", bones.len()));
    }
    facts.bone_names = bones;
    facts.max_influences = 8;
    facts.triangles = 30_000;
    facts.clip_names.retain(|clip| clip != "Victory");
    facts.rest_height_m = 168.0;
    let expected = vec![
        Problem::TooManyBones {
            found: 65,
            limit: MAX_BONES,
        },
        Problem::TooManyInfluences {
            found: 8,
            limit: MAX_INFLUENCES,
        },
        Problem::OverBudget {
            triangles: 30_000,
            budget: 8_000,
        },
        Problem::MissingBone {
            bone: "RightHand".to_string(),
        },
        Problem::MissingClip {
            key: "celebrate".to_string(),
            clip: "Victory".to_string(),
        },
        Problem::OddHeight {
            rest_height_m: 168.0,
        },
    ];
    assert!(check_model(&example(), &facts, 8_000) == expected);
}

#[test]
fn never_panics_on_empty_text() {
    assert!(read_persona("") == Err(PersonaError::MissingKey("persona.name".to_string())));
    assert!(read_persona("\u{0}").is_err());
}

#[test]
fn never_panics_on_a_lone_quote() {
    assert!(read_persona("\"").is_err());
    assert!(read_persona("name = \"").is_err());
    assert!(read_persona("[persona]\nname = \"").is_err());
}

#[test]
fn never_panics_on_a_lone_bracket() {
    assert!(read_persona("[") == Err(PersonaError::BadLine(1)));
    assert!(read_persona("[]").is_err());
    assert!(read_persona("[[").is_err());
}

#[test]
fn never_panics_on_a_lone_equals() {
    assert!(read_persona("=") == Err(PersonaError::BadLine(1)));
    assert!(read_persona("[persona]\n=") == Err(PersonaError::BadLine(2)));
}

#[test]
fn never_panics_on_ten_thousand_lines() {
    let comments = "# a note\n".repeat(10_000);
    assert!(read_persona(&comments) == Err(PersonaError::MissingKey("persona.name".to_string())));
    let blanks = "\n".repeat(10_000);
    assert!(read_persona(&blanks).is_err());
    let repeated = format!("[persona]\n{}", "name = \"S\"\n".repeat(10_000));
    assert!(
        read_persona(&repeated) == Err(PersonaError::DuplicateKey(3, "persona.name".to_string()))
    );
}
