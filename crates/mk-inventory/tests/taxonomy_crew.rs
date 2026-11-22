//! Tests for mk_inventory::taxonomy written by Gordon (the Crew, campaign mk-core-12) from the
//! contract alone, without seeing the module.

//! Integration tests for `taxonomy`: the lookups over the tool and material taxonomy.

use mk_inventory::taxonomy::{
    best_grade, is_a, kit, kits, material, material_by_words, materials, parse_supply, supplies_of,
    tool, tool_by_words, tools, tools_for, words_of, Supply,
};
use mk_inventory::taxonomy_data::{Access, Grade, ToolKind, CAPABILITIES, KITS, MATERIALS, TOOLS};

/// The expected `words_of` output as a `Vec<String>`, so whole values compare.
fn ws(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

/// The ids of a tool list, in order, so whole lists compare.
fn tool_ids(list: Vec<&'static ToolKind>) -> Vec<&'static str> {
    list.iter().map(|t| t.id).collect()
}

#[test]
fn counts_match_the_contract() {
    assert_eq!(CAPABILITIES.len(), 41);
    assert_eq!(tools().len(), 67);
    assert_eq!(materials().len(), 90);
    assert_eq!(kits().len(), 8);
}

#[test]
fn tables_are_returned_whole_in_table_order() {
    assert_eq!(tools().len(), TOOLS.len());
    for (i, t) in TOOLS.iter().enumerate() {
        assert_eq!(tools()[i].id, t.id);
    }
    assert_eq!(materials().len(), MATERIALS.len());
    for (i, m) in MATERIALS.iter().enumerate() {
        assert_eq!(materials()[i].id, m.id);
    }
    assert_eq!(kits().len(), KITS.len());
    for (i, k) in KITS.iter().enumerate() {
        assert_eq!(kits()[i].id, k.id);
    }
    assert_eq!(tools()[0].id, "tape_measure");
    assert_eq!(tools()[66].id, "mold_kit");
    assert_eq!(materials()[0].id, "wood.softwood.pine");
    assert_eq!(materials()[89].id, "glass.sheet");
    assert_eq!(kits()[0].id, "basic_household");
    assert_eq!(kits()[7].id, "makerspace");
}

#[test]
fn w1_words_of_lowercases_and_splits_on_punctuation() {
    assert_eq!(words_of("Pocket-Hole JIG!"), ws(&["pocket", "hole", "jig"]));
    assert_eq!(words_of("3/4\" ply"), ws(&["3", "4", "ply"]));
    assert_eq!(words_of("don't"), ws(&["don", "t"]));
    assert_eq!(words_of("a_b"), ws(&["a", "b"]));
}

#[test]
fn w1_words_of_keeps_digits_and_treats_non_ascii_as_separator() {
    assert_eq!(words_of("500D Cordura"), ws(&["500d", "cordura"]));
    assert_eq!(words_of("x2"), ws(&["x2"]));
    // 'e' with an accent is not ASCII, so it separates: "caf" then nothing.
    assert_eq!(words_of("caf\u{e9}"), ws(&["caf"]));
    assert_eq!(words_of("\u{dc}n\u{ef}code"), ws(&["n", "code"]));
    assert_eq!(words_of("\u{65e5}\u{672c}\u{8a9e}"), Vec::<String>::new());
}

#[test]
fn w1_words_of_empty_and_blank_input() {
    assert_eq!(words_of(""), Vec::<String>::new());
    assert_eq!(words_of("  "), Vec::<String>::new());
    assert_eq!(words_of("---"), Vec::<String>::new());
}

#[test]
fn w2_plural_s_matches_a_phrase_word() {
    // "clamps" is "clamp" + "s", so "bar clamp" matches inside "two bar clamps".
    assert_eq!(
        tool_by_words("two bar clamps").map(|t| t.id),
        Some("clamps")
    );
    // "tubes" is "tube" + "s"; "bike inner tube" is the 3-word phrase.
    assert_eq!(
        material_by_words("bike inner tubes").map(|m| m.id),
        Some("rubber.sheet")
    );
}

#[test]
fn w2_plural_es_matches_a_phrase_word() {
    // "boxes" is "box" + "es", so "cereal box" matches inside "old cereal boxes".
    assert_eq!(
        material_by_words("old cereal boxes").map(|m| m.id),
        Some("paper.cartonboard")
    );
    // "brushes" is "brush" + "es", so "paint brush" matches inside "paint brushes".
    assert_eq!(
        tool_by_words("paint brushes").map(|t| t.id),
        Some("brushes")
    );
}

#[test]
fn w2_planer_does_not_match_plane() {
    // "planer" is not "plane" + "s" or "plane" + "es": it is its own phrase on tool #51.
    assert_eq!(tool_by_words("planer").map(|t| t.id), Some("planer"));
    assert_eq!(tool_by_words("plane").map(|t| t.id), Some("hand_plane"));
    assert_eq!(tool_by_words("planes").map(|t| t.id), Some("hand_plane"));
}

#[test]
fn w2_phrase_must_be_contiguous_and_in_order() {
    // "tape measure" is a phrase, but the words are reversed and neither is a phrase alone.
    assert_eq!(tool_by_words("measure tape"), None);
    // "table saw" is a phrase, but "saw" and "table" alone are no phrases.
    assert_eq!(tool_by_words("saw table"), None);
}

#[test]
fn w3_longest_phrase_wins() {
    assert_eq!(
        tool_by_words("laser printer").map(|t| t.id),
        Some("paper_printer")
    );
    assert_eq!(tool_by_words("cnc mill").map(|t| t.id), Some("cnc_router"));
    assert_eq!(
        tool_by_words("an old Kreg pocket-hole jig").map(|t| t.id),
        Some("pocket_hole_jig")
    );
    assert_eq!(
        tool_by_words("industrial sewing machine").map(|t| t.id),
        Some("heavy_sewing_machine")
    );
    assert_eq!(
        material_by_words("Baltic birch plywood").map(|m| m.id),
        Some("wood.plywood.birch")
    );
    assert_eq!(
        material_by_words("melamine-faced particleboard").map(|m| m.id),
        Some("wood.particleboard.melamine")
    );
}

#[test]
fn w3_tie_goes_to_the_earlier_kind() {
    // "mill" (#53) and "lathe" (#52) both match one word; lathe is earlier.
    assert_eq!(tool_by_words("mill and lathe").map(|t| t.id), Some("lathe"));
    assert_eq!(tool_by_words("lathe and mill").map(|t| t.id), Some("lathe"));
    // "iron" (#29) and "hammer" (#19) both match one word; hammer is earlier.
    assert_eq!(
        tool_by_words("iron and hammer").map(|t| t.id),
        Some("hammer")
    );
    // "router" (#34) and "sander" (#35) both match one word; router is earlier.
    assert_eq!(
        tool_by_words("router and sander").map(|t| t.id),
        Some("router")
    );
}

#[test]
fn w3_no_match_is_none() {
    assert_eq!(tool_by_words("duct tape"), None);
    assert_eq!(tool_by_words(""), None);
    // No phrase is the bare word "knife" or "saw".
    assert_eq!(tool_by_words("knife"), None);
    assert_eq!(tool_by_words("saw"), None);
    assert_eq!(material_by_words("two IKEA Kallax units"), None);
    // No material phrase is the bare word "glue".
    assert_eq!(material_by_words("glue"), None);
    assert_eq!(material_by_words(""), None);
}

#[test]
fn worked_example_tool_by_words_rows() {
    assert_eq!(tool_by_words("Ryobi jigsaw").map(|t| t.id), Some("jigsaw"));
    assert_eq!(
        tool_by_words("DRILL PRESS").map(|t| t.id),
        Some("drill_press")
    );
    assert_eq!(
        tool_by_words("drill bits").map(|t| t.id),
        Some("drill_driver")
    );
    assert_eq!(
        tool_by_words("phone with LiDAR").map(|t| t.id),
        Some("phone_lidar")
    );
    assert_eq!(
        tool_by_words("my phone").map(|t| t.id),
        Some("phone_camera")
    );
}

#[test]
fn worked_example_material_by_words_rows() {
    assert_eq!(
        material_by_words("half a sheet of 3/4 ply in the garage").map(|m| m.id),
        Some("wood.plywood.softwood")
    );
    assert_eq!(
        material_by_words("an old tent").map(|m| m.id),
        Some("fabric.nylon.ripstop")
    );
    assert_eq!(
        material_by_words("5 kg of PLA").map(|m| m.id),
        Some("plastic.pla")
    );
    assert_eq!(
        material_by_words("2 yards 500D Cordura").map(|m| m.id),
        Some("fabric.nylon.cordura")
    );
    assert_eq!(
        material_by_words("birch").map(|m| m.id),
        Some("wood.hardwood.birch")
    );
    assert_eq!(
        material_by_words("acrylic paint").map(|m| m.id),
        Some("finish.paint")
    );
    assert_eq!(
        material_by_words("vinyl leather offcuts").map(|m| m.id),
        Some("fabric.faux_leather")
    );
    assert_eq!(
        material_by_words("steel wire").map(|m| m.id),
        Some("metal.wire")
    );
}

#[test]
fn worked_example_words_of_rows() {
    assert_eq!(words_of("3/4\" ply"), ws(&["3", "4", "ply"]));
    assert_eq!(words_of("  "), Vec::<String>::new());
}

#[test]
fn s1_parse_supply_without_colon_is_normal() {
    assert_eq!(
        parse_supply("cut.straight"),
        Some(Supply {
            capability: "cut.straight",
            grade: Grade::Normal
        })
    );
    assert_eq!(
        parse_supply("measure.photo"),
        Some(Supply {
            capability: "measure.photo",
            grade: Grade::Normal
        })
    );
    assert_eq!(
        parse_supply("clamp"),
        Some(Supply {
            capability: "clamp",
            grade: Grade::Normal
        })
    );
}

#[test]
fn s1_parse_supply_reads_rough_and_fine() {
    assert_eq!(
        parse_supply("cut.straight:rough"),
        Some(Supply {
            capability: "cut.straight",
            grade: Grade::Rough
        })
    );
    assert_eq!(
        parse_supply("drill.hole:fine"),
        Some(Supply {
            capability: "drill.hole",
            grade: Grade::Fine
        })
    );
}

#[test]
fn s1_parse_supply_rejects_bad_grade_words() {
    assert_eq!(parse_supply("cut.straight:Rough"), None);
    assert_eq!(parse_supply("cut.straight:"), None);
    assert_eq!(parse_supply("cut.straight:normal"), None);
    assert_eq!(parse_supply("cut.straight:rough:fine"), None);
    assert_eq!(parse_supply("cut.straight:rough "), None);
    assert_eq!(parse_supply(":rough"), None);
}

#[test]
fn s1_parse_supply_rejects_unknown_capability() {
    assert_eq!(parse_supply("cut.straigh"), None);
    assert_eq!(parse_supply(" cut.curve"), None);
    assert_eq!(parse_supply("cut.curve "), None);
    assert_eq!(parse_supply("CUT.CURVE"), None);
    assert_eq!(parse_supply(""), None);
}

#[test]
fn worked_example_parse_supply_rows() {
    assert_eq!(
        parse_supply("cut.straight"),
        Some(Supply {
            capability: "cut.straight",
            grade: Grade::Normal
        })
    );
    assert_eq!(
        parse_supply("cut.straight:rough"),
        Some(Supply {
            capability: "cut.straight",
            grade: Grade::Rough
        })
    );
    assert_eq!(
        parse_supply("drill.hole:fine"),
        Some(Supply {
            capability: "drill.hole",
            grade: Grade::Fine
        })
    );
    assert_eq!(
        parse_supply("measure.photo"),
        Some(Supply {
            capability: "measure.photo",
            grade: Grade::Normal
        })
    );
    assert_eq!(parse_supply("cut.straight:Rough"), None);
    assert_eq!(parse_supply("cut.straight:"), None);
    assert_eq!(parse_supply("cut.straigh"), None);
    assert_eq!(parse_supply(" cut.curve"), None);
    assert_eq!(parse_supply(""), None);
}

#[test]
fn s2_supplies_of_parses_every_entry_in_order() {
    let jigsaw = tool("jigsaw").unwrap();
    assert_eq!(
        supplies_of(jigsaw),
        vec![
            Supply {
                capability: "cut.curve",
                grade: Grade::Normal
            },
            Supply {
                capability: "cut.straight",
                grade: Grade::Rough
            },
            Supply {
                capability: "cut.sheet",
                grade: Grade::Normal
            },
        ]
    );
    let table_saw = tool("table_saw").unwrap();
    assert_eq!(
        supplies_of(table_saw),
        vec![
            Supply {
                capability: "cut.straight",
                grade: Grade::Fine
            },
            Supply {
                capability: "cut.rip",
                grade: Grade::Fine
            },
            Supply {
                capability: "cut.crosscut",
                grade: Grade::Normal
            },
            Supply {
                capability: "cut.sheet",
                grade: Grade::Normal
            },
            Supply {
                capability: "cut.angle",
                grade: Grade::Normal
            },
        ]
    );
}

#[test]
fn s2_supplies_of_tool_without_supplies_is_empty() {
    assert_eq!(supplies_of(tool("hammer").unwrap()), Vec::<Supply>::new());
    assert_eq!(supplies_of(tool("nail_gun").unwrap()), Vec::<Supply>::new());
}

#[test]
fn s3_tools_for_lists_in_table_order() {
    assert_eq!(
        tool_ids(tools_for("cut.straight")),
        vec![
            "hand_saw",
            "circular_saw",
            "track_saw",
            "jigsaw",
            "multi_tool",
            "angle_grinder",
            "table_saw",
            "panel_saw",
            "laser_cutter",
        ]
    );
    assert_eq!(
        tool_ids(tools_for("shape.plane")),
        vec!["chisels", "hand_plane", "planer"]
    );
    assert_eq!(tool_ids(tools_for("clamp")), vec!["clamps"]);
    assert_eq!(tool_ids(tools_for("cut.laser")), vec!["laser_cutter"]);
    assert_eq!(tool_ids(tools_for("measure.photo")), vec!["phone_camera"]);
}

#[test]
fn s3_tools_for_unknown_capability_is_empty() {
    assert_eq!(tool_ids(tools_for("join.nail")), Vec::<&str>::new());
    assert_eq!(tool_ids(tools_for("")), Vec::<&str>::new());
    assert_eq!(tool_ids(tools_for("cut.curve:rough")), Vec::<&str>::new());
}

#[test]
fn worked_example_tools_for_rows() {
    assert_eq!(
        tool_ids(tools_for("cut.curve")),
        vec![
            "coping_saw",
            "jigsaw",
            "rotary_tool",
            "band_saw",
            "scroll_saw",
            "laser_cutter",
            "cnc_router",
        ]
    );
    assert_eq!(
        tool_ids(tools_for("join.sew.heavy")),
        vec!["heavy_sewing_machine"]
    );
    assert_eq!(tool_ids(tools_for("join.nail")), Vec::<&str>::new());
}

#[test]
fn s4_best_grade_returns_the_highest() {
    assert_eq!(
        best_grade(&["circular_saw"], "cut.angle"),
        Some(Grade::Rough)
    );
    assert_eq!(
        best_grade(&["circular_saw", "table_saw"], "cut.angle"),
        Some(Grade::Normal)
    );
    assert_eq!(
        best_grade(&["laser_cutter"], "cut.curve"),
        Some(Grade::Fine)
    );
    assert_eq!(best_grade(&["track_saw"], "cut.sheet"), Some(Grade::Fine));
}

#[test]
fn s4_best_grade_ignores_unknown_ids() {
    assert_eq!(best_grade(&["nope"], "cut.curve"), None);
    assert_eq!(
        best_grade(&["nope", "table_saw"], "cut.rip"),
        Some(Grade::Fine)
    );
    assert_eq!(
        best_grade(&["", "jigsaw"], "cut.curve"),
        Some(Grade::Normal)
    );
}

#[test]
fn s4_best_grade_none_when_absent() {
    assert_eq!(best_grade(&["hammer"], "cut.curve"), None);
    assert_eq!(best_grade(&[], "clamp"), None);
    assert_eq!(best_grade(&["jigsaw"], "cut.laser"), None);
}

#[test]
fn worked_example_best_grade_rows() {
    assert_eq!(best_grade(&["jigsaw"], "cut.straight"), Some(Grade::Rough));
    assert_eq!(
        best_grade(&["jigsaw", "track_saw"], "cut.straight"),
        Some(Grade::Fine)
    );
    assert_eq!(best_grade(&["jigsaw"], "cut.sheet"), Some(Grade::Normal));
    assert_eq!(
        best_grade(&["nope", "screwdrivers"], "join.screw"),
        Some(Grade::Rough)
    );
    assert_eq!(
        best_grade(&["drill_driver", "screwdrivers"], "join.screw"),
        Some(Grade::Normal)
    );
    assert_eq!(best_grade(&["hammer"], "join.screw"), None);
    assert_eq!(best_grade(&[], "cut.curve"), None);
}

#[test]
fn l1_tool_finds_by_exact_id() {
    let t = tool("jigsaw").unwrap();
    assert_eq!(t.id, "jigsaw");
    assert_eq!(t.name, "Jigsaw");
    assert_eq!(tool("Jigsaw"), None);
    assert_eq!(tool("jig"), None);
    assert_eq!(tool(""), None);
}

#[test]
fn l1_material_finds_by_exact_id() {
    let m = material("wood.mdf").unwrap();
    assert_eq!(m.id, "wood.mdf");
    assert_eq!(m.name, "MDF");
    assert_eq!(material("wood"), None);
    assert_eq!(material("wood.mdfx"), None);
    assert_eq!(material(""), None);
}

#[test]
fn l1_kit_finds_by_exact_id() {
    let k = kit("laser").unwrap();
    assert_eq!(k.id, "laser");
    assert_eq!(k.name, "Laser");
    assert_eq!(kit("makerspace").unwrap().access, Access::Makerspace);
    assert_eq!(kit("basic_household").unwrap().access, Access::Owned);
    assert_eq!(kit("Laser"), None);
    assert_eq!(kit(""), None);
}

#[test]
fn l1_every_table_id_is_findable() {
    for t in tools() {
        assert_eq!(tool(t.id).map(|x| x.id), Some(t.id));
    }
    for m in materials() {
        assert_eq!(material(m.id).map(|x| x.id), Some(m.id));
    }
    for k in kits() {
        assert_eq!(kit(k.id).map(|x| x.id), Some(k.id));
    }
}

#[test]
fn f1_is_a_on_segment_boundaries() {
    assert!(is_a("wood.softwood.pine", "wood"));
    assert!(is_a("wood.softwood.pine", "wood.softwood"));
    assert!(is_a("wood.mdf.moisture", "wood.mdf"));
    assert!(is_a("wood.mdf", "wood.mdf"));
    assert!(is_a("wood.softwood.pine", "wood.softwood.pine"));
    assert!(is_a("wood.", "wood"));
    assert!(is_a("wood", "wood"));
}

#[test]
fn f1_is_a_false_cases() {
    assert!(!is_a("wood.softwood.pine", "wood.soft"));
    assert!(!is_a("wood", "wood.mdf"));
    assert!(!is_a("x", ""));
    assert!(!is_a("", ""));
    assert!(!is_a("woodx", "wood"));
    assert!(!is_a("wood.softwood.pine", "wood.softwood.pines"));
    assert!(!is_a("wood.softwood.pine", "wood.softwood.pine.extra"));
    assert!(!is_a("wood.softwood.pine", "pine"));
}

#[test]
fn worked_example_is_a_rows() {
    assert!(is_a("wood.softwood.pine", "wood"));
    assert!(is_a("wood.softwood.pine", "wood.softwood"));
    assert!(is_a("wood.mdf.moisture", "wood.mdf"));
    assert!(is_a("wood.mdf", "wood.mdf"));
    assert!(!is_a("wood.softwood.pine", "wood.soft"));
    assert!(!is_a("wood", "wood.mdf"));
    assert!(!is_a("x", ""));
}

#[test]
fn i1_ids_are_unique_within_each_table() {
    let mut seen = std::collections::HashSet::new();
    for t in tools() {
        assert!(seen.insert(t.id), "duplicate tool id {}", t.id);
    }
    let mut seen = std::collections::HashSet::new();
    for m in materials() {
        assert!(seen.insert(m.id), "duplicate material id {}", m.id);
    }
    let mut seen = std::collections::HashSet::new();
    for k in kits() {
        assert!(seen.insert(k.id), "duplicate kit id {}", k.id);
    }
}

#[test]
fn i2_phrases_are_in_words_of_form_and_unique_per_table() {
    let mut seen = std::collections::HashSet::new();
    for t in tools() {
        for p in t.words {
            assert_eq!(
                words_of(p).join(" "),
                *p,
                "tool phrase not in words_of form: {}",
                p
            );
            assert!(seen.insert(*p), "duplicate tool phrase: {}", p);
        }
    }
    let mut seen = std::collections::HashSet::new();
    for m in materials() {
        for p in m.words {
            assert_eq!(
                words_of(p).join(" "),
                *p,
                "material phrase not in words_of form: {}",
                p
            );
            assert!(seen.insert(*p), "duplicate material phrase: {}", p);
        }
    }
}

#[test]
fn i3_every_supply_parses() {
    for t in tools() {
        for s in t.supplies {
            let parsed = parse_supply(s);
            assert!(
                parsed.is_some(),
                "tool {} supply {:?} does not parse",
                t.id,
                s
            );
            let supply = parsed.unwrap();
            assert!(
                CAPABILITIES.contains(&supply.capability),
                "tool {} supply {:?} names a capability outside CAPABILITIES",
                t.id,
                s
            );
        }
        assert_eq!(supplies_of(t).len(), t.supplies.len());
    }
}

#[test]
fn i4_every_kit_tool_is_a_tool_id() {
    for k in kits() {
        for id in k.tools {
            assert!(tool(id).is_some(), "kit {} names unknown tool {}", k.id, id);
        }
    }
}

#[test]
fn i5_every_material_has_a_form() {
    for m in materials() {
        assert!(!m.forms.is_empty(), "material {} has no form", m.id);
    }
}

#[test]
fn i6_ids_use_only_the_allowed_characters() {
    let plain = |c: char| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_';
    for t in tools() {
        assert!(t.id.chars().all(plain), "bad tool id {}", t.id);
    }
    for k in kits() {
        assert!(k.id.chars().all(plain), "bad kit id {}", k.id);
    }
    for m in materials() {
        assert!(
            m.id.chars().all(|c| plain(c) || c == '.'),
            "bad material id {}",
            m.id
        );
    }
}

#[test]
fn odd_input_never_panics() {
    let long = "word ".repeat(10_000);
    assert_eq!(words_of(&long).len(), 10_000);
    assert_eq!(tool_by_words(&long), None);
    assert_eq!(material_by_words(&long), None);
    assert_eq!(parse_supply(&long), None);
    assert_eq!(tool(&long), None);
    assert_eq!(material(&long), None);
    assert_eq!(kit(&long), None);
    assert!(tools_for(&long).is_empty());
    assert_eq!(best_grade(&[long.as_str()], "cut.curve"), None);
    assert!(!is_a(&long, "wood"));
    assert!(!is_a("wood", &long));

    assert_eq!(tool_by_words("\u{65e5}\u{672c}\u{8a9e}"), None);
    assert_eq!(material_by_words("\u{65e5}\u{672c}\u{8a9e}"), None);
    assert_eq!(parse_supply("\u{65e5}"), None);
    assert_eq!(words_of(""), Vec::<String>::new());
    assert_eq!(tool_by_words(""), None);
    assert_eq!(material_by_words(""), None);
    assert_eq!(parse_supply(""), None);
    assert_eq!(tool(""), None);
    assert_eq!(material(""), None);
    assert_eq!(kit(""), None);
    assert!(tools_for("").is_empty());
    assert_eq!(best_grade(&[], ""), None);
    assert!(!is_a("", ""));
}
