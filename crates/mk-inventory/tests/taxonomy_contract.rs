//! Contract tests for mk_inventory::taxonomy, written by Gordon as a PA (the Crew) from
//! crew/contracts/mk_taxonomy.md. They were the hidden tests of campaign mk-core-12.

use mk_inventory::taxonomy::{
    best_grade, is_a, kit, kits, material, material_by_words, materials, parse_supply, supplies_of,
    tool, tool_by_words, tools, tools_for, words_of,
};
use mk_inventory::taxonomy_data::{Access, Form, Grade, Noise, CAPABILITIES};

use std::collections::HashSet;

// W1: words_of keeps ASCII letters (lower-cased) and digits; everything else separates.

#[test]
fn w1_lowercases_letters_and_drops_punctuation() {
    assert_eq!(words_of("Pocket-Hole JIG!"), ["pocket", "hole", "jig"]);
}

#[test]
fn w1_keeps_digits_and_splits_on_other_characters() {
    assert_eq!(words_of("3/4\" ply"), ["3", "4", "ply"]);
    assert_eq!(words_of("co2  LASER"), ["co2", "laser"]);
    assert_eq!(words_of("ply—wood"), ["ply", "wood"]); // an em dash is not ASCII
}

#[test]
fn w1_empty_and_punctuation_only_give_no_words() {
    assert!(words_of("").is_empty());
    assert!(words_of("  ").is_empty());
    assert!(words_of("!?.,;:/\\'\"()[]").is_empty());
}

// W2: a phrase word also matches with "s" or "es" on the end, and nothing fuzzier.

#[test]
fn w2_plural_s_matches() {
    assert_eq!(
        tool_by_words("two bar clamps").map(|k| k.id),
        Some("clamps")
    );
    assert_eq!(
        material_by_words("bike inner tubes").map(|m| m.id),
        Some("rubber.sheet")
    );
}

#[test]
fn w2_plural_es_matches() {
    assert_eq!(
        material_by_words("old cereal boxes").map(|m| m.id),
        Some("paper.cartonboard")
    );
}

#[test]
fn w2_near_miss_planer_does_not_match_plane() {
    assert_eq!(tool_by_words("planer").map(|k| k.id), Some("planer"));
    assert_eq!(
        tool_by_words("hand plane").map(|k| k.id),
        Some("hand_plane")
    );
    assert!(tool_by_words("planner").is_none());
}

// W3: the longest matching phrase wins; ties go to the kind earlier in the table.

#[test]
fn w3_longest_matching_phrase_wins() {
    assert_eq!(
        tool_by_words("laser printer").map(|k| k.id),
        Some("paper_printer")
    );
}

#[test]
fn w3_tie_goes_to_the_earlier_table_row() {
    // "mill" (#53) and "lathe" (#52) both match one word; lathe comes first.
    assert_eq!(tool_by_words("mill and lathe").map(|k| k.id), Some("lathe"));
}

#[test]
fn w3_no_match_or_no_words_is_none() {
    assert!(tool_by_words("duct tape").is_none());
    assert!(tool_by_words("").is_none());
    assert!(material_by_words("two IKEA Kallax units").is_none());
    assert!(material_by_words("").is_none());
}

// S1: parse_supply takes the text as given, case-sensitively, with no trimming.

#[test]
fn s1_bare_capability_is_normal_grade() {
    let supply = parse_supply("cut.straight").expect("a bare capability should parse");
    assert_eq!(supply.capability, "cut.straight");
    assert_eq!(supply.grade, Grade::Normal);
}

#[test]
fn s1_rough_and_fine_grades() {
    let rough = parse_supply("cut.straight:rough").expect("rough should parse");
    assert!(rough.capability == "cut.straight" && rough.grade == Grade::Rough);
    let fine = parse_supply("drill.hole:fine").expect("fine should parse");
    assert!(fine.capability == "drill.hole" && fine.grade == Grade::Fine);
}

#[test]
fn s1_every_bad_input_is_none() {
    for text in [
        "cut.straight:Rough",
        "cut.straight:",
        "cut.straigh",
        " cut.curve",
        "",
        ":rough",
        "cut.straight:normal",
        "cut.straight: fine",
        "CUT.CURVE",
    ] {
        assert!(parse_supply(text).is_none(), "should be None: {text:?}");
    }
}

// S2: supplies_of parses every entry, in the order the tool lists them.

#[test]
fn s2_supplies_of_keeps_the_tool_order() {
    let jigsaw = tool("jigsaw").expect("jigsaw exists");
    let capabilities: Vec<&str> = supplies_of(jigsaw).iter().map(|s| s.capability).collect();
    assert_eq!(capabilities, ["cut.curve", "cut.straight", "cut.sheet"]);
    let grades: Vec<Grade> = supplies_of(jigsaw).iter().map(|s| s.grade).collect();
    assert_eq!(grades, [Grade::Normal, Grade::Rough, Grade::Normal]);
}

#[test]
fn s2_supplies_of_parses_every_entry_of_a_big_tool() {
    let table_saw = tool("table_saw").expect("table_saw exists");
    assert_eq!(supplies_of(table_saw).len(), table_saw.supplies.len());
    assert!(supplies_of(table_saw)
        .iter()
        .any(|s| s.capability == "cut.rip"));
}

// S3: tools_for lists every tool that supplies a capability, in table order.

#[test]
fn s3_tools_for_lists_every_tool_in_table_order() {
    let ids: Vec<&str> = tools_for("cut.curve").iter().map(|k| k.id).collect();
    assert_eq!(
        ids,
        [
            "coping_saw",
            "jigsaw",
            "rotary_tool",
            "band_saw",
            "scroll_saw",
            "laser_cutter",
            "cnc_router",
        ]
    );
}

#[test]
fn s3_single_tool_and_unknown_capability() {
    let ids: Vec<&str> = tools_for("join.sew.heavy").iter().map(|k| k.id).collect();
    assert_eq!(ids, ["heavy_sewing_machine"]);
    assert!(tools_for("join.nail").is_empty());
    assert!(tools_for("").is_empty());
    assert!(tools_for("cut.curve:fine").is_empty()); // a grade is not a capability
}

// S4: best_grade takes the highest grade among the supplies of that capability.

#[test]
fn s4_highest_grade_wins() {
    assert_eq!(best_grade(&["jigsaw"], "cut.straight"), Some(Grade::Rough));
    assert_eq!(
        best_grade(&["jigsaw", "track_saw"], "cut.straight"),
        Some(Grade::Fine)
    );
}

#[test]
fn s4_ignores_unknown_ids() {
    assert_eq!(
        best_grade(&["nope", "screwdrivers"], "join.screw"),
        Some(Grade::Rough)
    );
    assert_eq!(
        best_grade(&["drill_driver", "screwdrivers"], "join.screw"),
        Some(Grade::Normal)
    );
}

#[test]
fn s4_none_when_nothing_supplies_it() {
    assert!(best_grade(&["hammer"], "join.screw").is_none());
    assert!(best_grade(&[], "cut.curve").is_none());
    assert!(best_grade(&["jigsaw"], "join.nail").is_none());
}

// L1: tool, material and kit find by exact id.

#[test]
fn l1_tool_by_exact_id() {
    assert_eq!(tool("table_saw").map(|k| k.id), Some("table_saw"));
    assert_eq!(tool("jigsaw").map(|k| k.name), Some("Jigsaw"));
}

#[test]
fn l1_material_by_exact_id() {
    assert_eq!(material("wood.mdf").map(|m| m.id), Some("wood.mdf"));
    assert_eq!(material("plastic.acrylic").map(|m| m.name), Some("Acrylic"));
}

#[test]
fn l1_kit_by_exact_id() {
    assert_eq!(
        kit("basic_household").map(|k| k.name),
        Some("Basic household")
    );
}

#[test]
fn l1_unknown_ids_are_none() {
    assert!(tool("Jigsaw").is_none());
    assert!(tool("").is_none());
    assert!(material("wood").is_none());
    assert!(material("").is_none());
    assert!(kit("").is_none());
    assert!(kit("woodshop").is_none());
}

#[test]
fn l1_every_table_id_finds_its_own_kind() {
    for kind in tools() {
        assert_eq!(tool(kind.id).map(|k| k.id), Some(kind.id));
    }
    for kind in materials() {
        assert_eq!(material(kind.id).map(|m| m.id), Some(kind.id));
    }
    for kind in kits() {
        assert_eq!(kit(kind.id).map(|k| k.id), Some(kind.id));
    }
}

// F1: is_a is the family itself, or a child of it after a dot.

#[test]
fn f1_family_is_the_id_or_a_dot_child() {
    assert!(is_a("wood.softwood.pine", "wood"));
    assert!(is_a("wood.softwood.pine", "wood.softwood"));
    assert!(is_a("wood.mdf.moisture", "wood.mdf"));
    assert!(is_a("wood.mdf", "wood.mdf"));
}

#[test]
fn f1_family_needs_a_dot_boundary() {
    assert!(!is_a("wood.softwood.pine", "wood.soft"));
    assert!(!is_a("wood", "wood.mdf"));
    assert!(!is_a("x", ""));
    assert!(!is_a("wood.mdf.moisture", "wood.md"));
    assert!(!is_a("", ""));
}

#[test]
fn f1_holds_for_every_material_id_and_its_parent() {
    for kind in materials() {
        assert!(is_a(kind.id, kind.id), "{} is not its own family", kind.id);
        if let Some((family, _)) = kind.id.rsplit_once('.') {
            assert!(is_a(kind.id, family), "{} is not under {family}", kind.id);
        }
    }
}

// Accessors return the whole tables, in table order.

#[test]
fn counts_and_table_order_match_the_contract() {
    assert_eq!(CAPABILITIES.len(), 41);
    assert_eq!(tools().len(), 67);
    assert_eq!(materials().len(), 90);
    assert_eq!(kits().len(), 8);
    assert_eq!(tools().first().map(|k| k.id), Some("tape_measure"));
    assert_eq!(tools().last().map(|k| k.id), Some("mold_kit"));
    assert_eq!(
        materials().first().map(|m| m.id),
        Some("wood.softwood.pine")
    );
    assert_eq!(materials().last().map(|m| m.id), Some("glass.sheet"));
    assert_eq!(kits().first().map(|k| k.id), Some("basic_household"));
    assert_eq!(kits().last().map(|k| k.id), Some("makerspace"));
}

#[test]
fn sample_rows_carry_their_fields() {
    assert!(matches!(
        tool("jigsaw").map(|k| &k.noise),
        Some(Noise::Loud)
    ));
    assert!(matches!(
        tool("tape_measure").map(|k| &k.noise),
        Some(Noise::Quiet)
    ));
    let pine = material("wood.softwood.pine").expect("pine exists");
    assert_eq!(pine.forms.len(), 2);
    assert!(matches!(pine.forms.first(), Some(Form::Board)));
    assert!(matches!(pine.forms.get(1), Some(Form::Block)));
    let plywood = material("wood.plywood.softwood").expect("plywood exists");
    assert!(matches!(plywood.forms.first(), Some(Form::Sheet)));
    assert!(matches!(
        material("plastic.polycarbonate").map(|m| m.laser_never),
        Some(true)
    ));
    assert!(matches!(
        material("wood.softwood.pine").map(|m| m.laser_never),
        Some(false)
    ));
    assert!(matches!(
        kit("makerspace").map(|k| &k.access),
        Some(Access::Makerspace)
    ));
    assert!(matches!(
        kit("basic_household").map(|k| &k.access),
        Some(Access::Owned)
    ));
}

// Worked examples: tools.

#[test]
fn ex_tool_ryobi_jigsaw() {
    assert_eq!(tool_by_words("Ryobi jigsaw").map(|k| k.id), Some("jigsaw"));
}

#[test]
fn ex_tool_laser_printer() {
    assert_eq!(
        tool_by_words("laser printer").map(|k| k.id),
        Some("paper_printer")
    );
}

#[test]
fn ex_tool_kreg_pocket_hole_jig() {
    assert_eq!(
        tool_by_words("an old Kreg pocket-hole jig").map(|k| k.id),
        Some("pocket_hole_jig")
    );
}

#[test]
fn ex_tool_drill_press_uppercase() {
    assert_eq!(
        tool_by_words("DRILL PRESS").map(|k| k.id),
        Some("drill_press")
    );
}

#[test]
fn ex_tool_drill_bits() {
    assert_eq!(
        tool_by_words("drill bits").map(|k| k.id),
        Some("drill_driver")
    );
}

#[test]
fn ex_tool_cnc_mill() {
    assert_eq!(tool_by_words("cnc mill").map(|k| k.id), Some("cnc_router"));
}

#[test]
fn ex_tool_mill_and_lathe() {
    assert_eq!(tool_by_words("mill and lathe").map(|k| k.id), Some("lathe"));
}

#[test]
fn ex_tool_two_bar_clamps() {
    assert_eq!(
        tool_by_words("two bar clamps").map(|k| k.id),
        Some("clamps")
    );
}

#[test]
fn ex_tool_phone_with_lidar() {
    assert_eq!(
        tool_by_words("phone with LiDAR").map(|k| k.id),
        Some("phone_lidar")
    );
}

#[test]
fn ex_tool_my_phone() {
    assert_eq!(
        tool_by_words("my phone").map(|k| k.id),
        Some("phone_camera")
    );
}

#[test]
fn ex_tool_industrial_sewing_machine() {
    assert_eq!(
        tool_by_words("industrial sewing machine").map(|k| k.id),
        Some("heavy_sewing_machine")
    );
}

#[test]
fn ex_tool_duct_tape_is_none() {
    assert!(tool_by_words("duct tape").is_none());
}

#[test]
fn ex_tool_empty_is_none() {
    assert!(tool_by_words("").is_none());
}

// Worked examples: materials.

#[test]
fn ex_material_half_a_sheet_of_ply() {
    assert_eq!(
        material_by_words("half a sheet of 3/4 ply in the garage").map(|m| m.id),
        Some("wood.plywood.softwood")
    );
}

#[test]
fn ex_material_old_cereal_boxes() {
    assert_eq!(
        material_by_words("old cereal boxes").map(|m| m.id),
        Some("paper.cartonboard")
    );
}

#[test]
fn ex_material_an_old_tent() {
    assert_eq!(
        material_by_words("an old tent").map(|m| m.id),
        Some("fabric.nylon.ripstop")
    );
}

#[test]
fn ex_material_bike_inner_tubes() {
    assert_eq!(
        material_by_words("bike inner tubes").map(|m| m.id),
        Some("rubber.sheet")
    );
}

#[test]
fn ex_material_five_kg_of_pla() {
    assert_eq!(
        material_by_words("5 kg of PLA").map(|m| m.id),
        Some("plastic.pla")
    );
}

#[test]
fn ex_material_500d_cordura() {
    assert_eq!(
        material_by_words("2 yards 500D Cordura").map(|m| m.id),
        Some("fabric.nylon.cordura")
    );
}

#[test]
fn ex_material_baltic_birch_plywood() {
    assert_eq!(
        material_by_words("Baltic birch plywood").map(|m| m.id),
        Some("wood.plywood.birch")
    );
}

#[test]
fn ex_material_birch() {
    assert_eq!(
        material_by_words("birch").map(|m| m.id),
        Some("wood.hardwood.birch")
    );
}

#[test]
fn ex_material_acrylic_paint() {
    assert_eq!(
        material_by_words("acrylic paint").map(|m| m.id),
        Some("finish.paint")
    );
}

#[test]
fn ex_material_vinyl_leather() {
    assert_eq!(
        material_by_words("vinyl leather offcuts").map(|m| m.id),
        Some("fabric.faux_leather")
    );
}

#[test]
fn ex_material_melamine_faced_particleboard() {
    assert_eq!(
        material_by_words("melamine-faced particleboard").map(|m| m.id),
        Some("wood.particleboard.melamine")
    );
}

#[test]
fn ex_material_steel_wire() {
    assert_eq!(
        material_by_words("steel wire").map(|m| m.id),
        Some("metal.wire")
    );
}

#[test]
fn ex_material_ikea_is_none() {
    assert!(material_by_words("two IKEA Kallax units").is_none());
}

// Worked examples: parse_supply and best_grade.

#[test]
fn ex_parse_supply_measure_photo() {
    let supply = parse_supply("measure.photo").expect("measure.photo parses");
    assert!(supply.capability == "measure.photo" && supply.grade == Grade::Normal);
}

#[test]
fn ex_best_grade_jigsaw_cut_sheet() {
    assert_eq!(best_grade(&["jigsaw"], "cut.sheet"), Some(Grade::Normal));
}

// Invariants the real tables must satisfy.

#[test]
fn i1_ids_are_unique_within_each_table() {
    let mut seen = HashSet::new();
    for kind in tools() {
        assert!(seen.insert(kind.id), "duplicate tool id: {}", kind.id);
    }
    let mut seen = HashSet::new();
    for kind in materials() {
        assert!(seen.insert(kind.id), "duplicate material id: {}", kind.id);
    }
    let mut seen = HashSet::new();
    for kind in kits() {
        assert!(seen.insert(kind.id), "duplicate kit id: {}", kind.id);
    }
}

#[test]
fn i2_phrases_are_words_of_form_and_unique() {
    let mut seen = HashSet::new();
    for kind in tools() {
        for phrase in kind.words {
            assert_eq!(
                words_of(phrase).join(" "),
                *phrase,
                "tool phrase is not in words_of form: {phrase}"
            );
            assert!(seen.insert(*phrase), "duplicate tool phrase: {phrase}");
        }
    }
    let mut seen = HashSet::new();
    for kind in materials() {
        for phrase in kind.words {
            assert_eq!(
                words_of(phrase).join(" "),
                *phrase,
                "material phrase is not in words_of form: {phrase}"
            );
            assert!(seen.insert(*phrase), "duplicate material phrase: {phrase}");
        }
    }
}

#[test]
fn i3_every_supply_parses() {
    for kind in tools() {
        for entry in kind.supplies {
            assert!(
                parse_supply(entry).is_some(),
                "{} has an unparsable supply: {entry}",
                kind.id
            );
        }
        assert_eq!(
            supplies_of(kind).len(),
            kind.supplies.len(),
            "{} lost a supply",
            kind.id
        );
    }
}

#[test]
fn i4_every_kit_tool_is_a_tool_id() {
    for kind in kits() {
        for id in kind.tools {
            assert!(tool(id).is_some(), "{} names unknown tool {id}", kind.id);
        }
    }
}

#[test]
fn i5_every_material_has_a_form() {
    for kind in materials() {
        assert!(!kind.forms.is_empty(), "{} has no forms", kind.id);
    }
}

/// True when every character of `id` is a lower-case ASCII letter, a digit, '_', and
/// optionally '.'.
fn id_chars_ok(id: &str, allow_dot: bool) -> bool {
    id.chars().all(|c| {
        c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || (allow_dot && c == '.')
    })
}

#[test]
fn i6_ids_use_only_the_allowed_characters() {
    for kind in tools() {
        assert!(id_chars_ok(kind.id, false), "tool id: {}", kind.id);
    }
    for kind in kits() {
        assert!(id_chars_ok(kind.id, false), "kit id: {}", kind.id);
    }
    for kind in materials() {
        assert!(id_chars_ok(kind.id, true), "material id: {}", kind.id);
    }
}

// Odd input never panics and never matches by accident.

#[test]
fn odd_input_never_panics() {
    assert!(tool_by_words("").is_none());
    assert!(material_by_words("").is_none());
    assert_eq!(tool_by_words("🔨 jigsaw 🔩").map(|k| k.id), Some("jigsaw"));
    assert!(words_of("日本語のテキスト").is_empty());
    assert!(material_by_words("🔨🔩🧰").is_none());

    let many = "zzz ".repeat(10_000);
    assert_eq!(words_of(&many).len(), 10_000);
    assert!(tool_by_words(&many).is_none());
    assert!(material_by_words(&many).is_none());

    let long_word = "z".repeat(100_000);
    assert!(tool_by_words(&long_word).is_none());
    assert!(best_grade(&["nope"; 50], "cut.curve").is_none());
    assert!(!is_a(&"?".repeat(1_000), "wood"));
    assert!(!is_a("", ""));
}
