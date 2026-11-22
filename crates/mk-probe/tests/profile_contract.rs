//! Contract tests for mk_probe::profile, written by Qwen as a PA (the Crew) from
//! crew/contracts/mk_profile.md (ATLAS-api-harness). They were the hidden tests of campaign mk-core-6.

use mk_probe::profile::{
    choose_backend, classify, defaults, from_facts, group_digits, parse, potato, to_toml, Attempt,
    Backend, Class, Facts, Lod, Parsed, Physics, Problem, Profile, SanchoTier, BACKEND_ORDER, KEYS,
};

fn attempt(backend: Backend, initialised: bool) -> Attempt {
    Attempt {
        backend,
        initialised,
    }
}

fn facts(
    attempts: Vec<Attempt>,
    p95: f64,
    vat: f64,
    cores: u32,
    ram: u64,
    vram: Option<u64>,
) -> Facts {
    Facts {
        attempts,
        frame_p95_ms: p95,
        sancho_vat_ms: vat,
        cores,
        ram_total_mb: ram,
        vram_mb: vram,
    }
}

// ---- constants -----------------------------------------------------------

#[test]
fn backend_order_is_dx12_then_gl() {
    assert_eq!(BACKEND_ORDER, [Backend::Dx12, Backend::Gl]);
}

#[test]
fn keys_are_in_doc_order() {
    assert_eq!(
        KEYS,
        [
            "class",
            "backend",
            "tri_budget",
            "draw_call_budget",
            "lod_aggressiveness",
            "sancho",
            "splat_render",
            "physics",
            "mem_budget_mb",
            "background_jobs",
            "path_tracer",
        ]
    );
}

// ---- defaults table, field by field --------------------------------------

#[test]
fn defaults_potato_field_by_field() {
    let p = defaults(Class::Potato);
    assert_eq!(p.class, Class::Potato);
    assert_eq!(p.backend, Backend::Dx12);
    assert_eq!(p.tri_budget, 200_000);
    assert_eq!(p.draw_call_budget, 60);
    assert_eq!(p.lod_aggressiveness, Lod::High);
    assert_eq!(p.sancho, SanchoTier::BakedVat);
    assert!(!p.splat_render);
    assert_eq!(p.physics, Physics::Kinematic);
    assert_eq!(p.mem_budget_mb, 2_500);
    assert_eq!(p.background_jobs, 2);
    assert!(!p.path_tracer);
}

#[test]
fn defaults_modest_field_by_field() {
    let p = defaults(Class::Modest);
    assert_eq!(p.class, Class::Modest);
    assert_eq!(p.backend, Backend::Dx12);
    assert_eq!(p.tri_budget, 1_000_000);
    assert_eq!(p.draw_call_budget, 200);
    assert_eq!(p.lod_aggressiveness, Lod::Medium);
    assert_eq!(p.sancho, SanchoTier::Lod8k);
    assert!(p.splat_render);
    assert_eq!(p.physics, Physics::Dynamic);
    assert_eq!(p.mem_budget_mb, 6_000);
    assert_eq!(p.background_jobs, 3);
    assert!(!p.path_tracer);
}

#[test]
fn defaults_capable_field_by_field() {
    let p = defaults(Class::Capable);
    assert_eq!(p.class, Class::Capable);
    assert_eq!(p.backend, Backend::Dx12);
    assert_eq!(p.tri_budget, 4_000_000);
    assert_eq!(p.draw_call_budget, 800);
    assert_eq!(p.lod_aggressiveness, Lod::Low);
    assert_eq!(p.sancho, SanchoTier::Full);
    assert!(p.splat_render);
    assert_eq!(p.physics, Physics::Dynamic);
    assert_eq!(p.mem_budget_mb, 16_000);
    assert_eq!(p.background_jobs, 4);
    assert!(p.path_tracer);
}

#[test]
fn potato_is_defaults_of_potato() {
    assert_eq!(potato(), defaults(Class::Potato));
}

// ---- choose_backend (rule B1) --------------------------------------------

#[test]
fn choose_backend_none_is_cpu() {
    assert_eq!(choose_backend(&[]), Backend::Cpu);
}

#[test]
fn choose_backend_dx12_ok() {
    assert_eq!(
        choose_backend(&[attempt(Backend::Dx12, true)]),
        Backend::Dx12
    );
}

#[test]
fn choose_backend_backend_order_decides_not_attempt_order() {
    let attempts = vec![attempt(Backend::Gl, true), attempt(Backend::Dx12, true)];
    assert_eq!(choose_backend(&attempts), Backend::Dx12);
}

#[test]
fn choose_backend_only_the_first_attempt_of_a_backend_counts() {
    let attempts = vec![
        attempt(Backend::Dx12, false),
        attempt(Backend::Dx12, true),
        attempt(Backend::Gl, true),
    ];
    assert_eq!(choose_backend(&attempts), Backend::Gl);
}

#[test]
fn choose_backend_cpu_ok_is_still_cpu() {
    assert_eq!(choose_backend(&[attempt(Backend::Cpu, true)]), Backend::Cpu);
}

// ---- classify (rules K1–K4) ----------------------------------------------

#[test]
fn classify_k1_cpu_is_always_potato() {
    let f = facts(vec![], 1.0, 1.0, 16, 65_536, Some(16_384));
    assert_eq!(classify(&f, Backend::Cpu), Class::Potato);
}

#[test]
fn classify_k2_all_capable_bounds_met() {
    let f = facts(vec![], 10.0, 1.0, 8, 28_672, Some(6_144));
    assert_eq!(classify(&f, Backend::Dx12), Class::Capable);
}

#[test]
fn classify_k2_fails_just_over_10_ms_falls_to_k3() {
    let f = facts(vec![], 10.1, 1.0, 8, 28_672, Some(6_144));
    assert_eq!(classify(&f, Backend::Dx12), Class::Modest);
}

#[test]
fn classify_k3_all_modest_bounds_met() {
    let f = facts(vec![], 20.0, 1.0, 4, 12_288, None);
    assert_eq!(classify(&f, Backend::Dx12), Class::Modest);
}

#[test]
fn classify_k4_nothing_met_is_potato() {
    let f = facts(vec![], 20.1, 1.0, 3, 12_287, None);
    assert_eq!(classify(&f, Backend::Dx12), Class::Potato);
}

#[test]
fn classify_a_frame_time_within_l_is_0_to_l_inclusive() {
    // NaN, infinite and negative times are never within anything.
    let nan = facts(vec![], f64::NAN, 1.0, 8, 28_672, Some(6_144));
    assert_eq!(classify(&nan, Backend::Dx12), Class::Potato);
    let inf = facts(vec![], f64::INFINITY, 1.0, 8, 28_672, Some(6_144));
    assert_eq!(classify(&inf, Backend::Dx12), Class::Potato);
    let neg = facts(vec![], -0.1, 1.0, 8, 28_672, Some(6_144));
    assert_eq!(classify(&neg, Backend::Dx12), Class::Potato);
    // 0.0 is within any positive bound.
    let zero = facts(vec![], 0.0, 1.0, 8, 28_672, Some(6_144));
    assert_eq!(classify(&zero, Backend::Dx12), Class::Capable);
}

// ---- from_facts worked examples (rules F1–F4) ----------------------------

#[test]
fn from_facts_none_is_potato() {
    assert_eq!(from_facts(None), potato());
}

#[test]
fn from_facts_this_machine_is_potato() {
    let f = facts(
        vec![attempt(Backend::Dx12, true)],
        31.0,
        1.5,
        4,
        7_900,
        None,
    );
    assert_eq!(from_facts(Some(&f)), potato());
}

#[test]
fn from_facts_vat_over_2_ms_is_portrait() {
    let f = facts(
        vec![attempt(Backend::Dx12, true)],
        31.0,
        2.5,
        4,
        7_900,
        None,
    );
    let mut want = potato();
    want.sancho = SanchoTier::Portrait;
    assert_eq!(from_facts(Some(&f)), want);
}

#[test]
fn from_facts_vat_exactly_2_ms_keeps_baked_vat() {
    let f = facts(
        vec![attempt(Backend::Dx12, true)],
        31.0,
        2.0,
        4,
        7_900,
        None,
    );
    assert_eq!(from_facts(Some(&f)), potato());
}

#[test]
fn from_facts_vat_nan_is_portrait() {
    let f = facts(
        vec![attempt(Backend::Dx12, true)],
        31.0,
        f64::NAN,
        4,
        7_900,
        None,
    );
    let mut want = potato();
    want.sancho = SanchoTier::Portrait;
    assert_eq!(from_facts(Some(&f)), want);
}

#[test]
fn from_facts_gl_fallback() {
    let f = facts(
        vec![attempt(Backend::Dx12, false), attempt(Backend::Gl, true)],
        31.0,
        1.0,
        4,
        7_900,
        None,
    );
    let mut want = potato();
    want.backend = Backend::Gl;
    assert_eq!(from_facts(Some(&f)), want);
}

#[test]
fn from_facts_all_failed_is_cpu() {
    let f = facts(
        vec![attempt(Backend::Dx12, false), attempt(Backend::Gl, false)],
        5.0,
        1.0,
        16,
        65_536,
        Some(16_384),
    );
    let mut want = potato();
    want.backend = Backend::Cpu;
    assert_eq!(from_facts(Some(&f)), want);
}

#[test]
fn from_facts_no_attempts_is_cpu() {
    let f = facts(vec![], 5.0, 1.0, 16, 65_536, Some(16_384));
    let mut want = potato();
    want.backend = Backend::Cpu;
    assert_eq!(from_facts(Some(&f)), want);
}

#[test]
fn from_facts_modest() {
    let f = facts(
        vec![attempt(Backend::Dx12, true)],
        15.0,
        0.5,
        6,
        15_800,
        None,
    );
    assert_eq!(from_facts(Some(&f)), defaults(Class::Modest));
}

#[test]
fn from_facts_modest_every_bound_inclusive() {
    let f = facts(
        vec![attempt(Backend::Dx12, true)],
        20.0,
        0.5,
        4,
        12_288,
        None,
    );
    assert_eq!(from_facts(Some(&f)), defaults(Class::Modest));
}

#[test]
fn from_facts_too_slow_for_modest_is_potato() {
    let f = facts(
        vec![attempt(Backend::Dx12, true)],
        20.5,
        0.5,
        6,
        15_800,
        None,
    );
    assert_eq!(from_facts(Some(&f)), potato());
}

#[test]
fn from_facts_f4_is_potato_only() {
    let f = facts(
        vec![attempt(Backend::Dx12, true)],
        15.0,
        3.0,
        6,
        15_800,
        None,
    );
    assert_eq!(from_facts(Some(&f)), defaults(Class::Modest));
}

#[test]
fn from_facts_capable_with_gl() {
    let f = facts(
        vec![attempt(Backend::Gl, true)],
        8.0,
        0.3,
        8,
        32_000,
        Some(8_192),
    );
    let mut want = defaults(Class::Capable);
    want.backend = Backend::Gl;
    assert_eq!(from_facts(Some(&f)), want);
}

#[test]
fn from_facts_no_vram_reported_is_modest() {
    let f = facts(
        vec![attempt(Backend::Dx12, true)],
        8.0,
        0.3,
        8,
        32_000,
        None,
    );
    assert_eq!(from_facts(Some(&f)), defaults(Class::Modest));
}

#[test]
fn from_facts_vram_below_threshold_is_modest() {
    let f = facts(
        vec![attempt(Backend::Dx12, true)],
        8.0,
        0.3,
        8,
        32_000,
        Some(6_143),
    );
    assert_eq!(from_facts(Some(&f)), defaults(Class::Modest));
}

#[test]
fn from_facts_too_slow_for_capable_is_modest() {
    let f = facts(
        vec![attempt(Backend::Dx12, true)],
        10.5,
        0.3,
        12,
        65_536,
        Some(12_288),
    );
    assert_eq!(from_facts(Some(&f)), defaults(Class::Modest));
}

#[test]
fn from_facts_negative_time_is_potato() {
    let f = facts(
        vec![attempt(Backend::Dx12, true)],
        -1.0,
        0.3,
        12,
        65_536,
        Some(12_288),
    );
    assert_eq!(from_facts(Some(&f)), potato());
}

#[test]
fn from_facts_seven_cores_is_modest() {
    let f = facts(
        vec![attempt(Backend::Dx12, true)],
        8.0,
        0.3,
        7,
        65_536,
        Some(12_288),
    );
    assert_eq!(from_facts(Some(&f)), defaults(Class::Modest));
}

// ---- group_digits (rule T2) ----------------------------------------------

#[test]
fn group_digits_all() {
    assert_eq!(group_digits(0), "0");
    assert_eq!(group_digits(7), "7");
    assert_eq!(group_digits(60), "60");
    assert_eq!(group_digits(999), "999");
    assert_eq!(group_digits(1000), "1_000");
    assert_eq!(group_digits(2500), "2_500");
    assert_eq!(group_digits(200000), "200_000");
    assert_eq!(group_digits(1000000), "1_000_000");
    assert_eq!(group_digits(4294967295), "4_294_967_295");
}

// ---- to_toml (rules T1–T3) ------------------------------------------------

#[test]
fn to_toml_potato_byte_for_byte() {
    let want = "[profile]\n\
class = \"potato\"\n\
backend = \"dx12\"\n\
tri_budget = 200_000\n\
draw_call_budget = 60\n\
lod_aggressiveness = \"high\"\n\
sancho = \"baked_vat\"\n\
splat_render = false\n\
physics = \"kinematic\"\n\
mem_budget_mb = 2_500\n\
background_jobs = 2\n\
path_tracer = false\n";
    assert_eq!(to_toml(&potato()), want);
}

// ---- round trip parse(&to_toml(&p)) ---------------------------------------

#[test]
fn round_trip_every_class_backend_sancho_physics_lod() {
    let classes = [Class::Potato, Class::Modest, Class::Capable];
    let backends = [Backend::Dx12, Backend::Gl, Backend::Cpu];
    let lods = [Lod::Low, Lod::Medium, Lod::High];
    let sancho_tiers = [
        SanchoTier::Full,
        SanchoTier::Lod8k,
        SanchoTier::BakedVat,
        SanchoTier::Portrait,
    ];
    let physics = [Physics::Kinematic, Physics::Dynamic];
    for &class in &classes {
        for &backend in &backends {
            for &lod in &lods {
                for &sancho in &sancho_tiers {
                    for &phys in &physics {
                        for splat in [false, true] {
                            for path in [false, true] {
                                let p = Profile {
                                    class,
                                    backend,
                                    tri_budget: 200_000,
                                    draw_call_budget: 60,
                                    lod_aggressiveness: lod,
                                    sancho,
                                    splat_render: splat,
                                    physics: phys,
                                    mem_budget_mb: 2_500,
                                    background_jobs: 2,
                                    path_tracer: path,
                                };
                                let parsed = parse(&to_toml(&p));
                                assert_eq!(
                                    parsed,
                                    Parsed {
                                        profile: p,
                                        problems: vec![]
                                    }
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn round_trip_integer_extremes() {
    let tris = [0u32, 1, 200_000, 4_294_967_295];
    let draw_calls = [0u32, 60, 800];
    let mems = [0u32, 2_500, 16_000];
    let jobs = [0u32, 2, 4];
    for &tri in &tris {
        for &dc in &draw_calls {
            for &mem in &mems {
                for &bg in &jobs {
                    let p = Profile {
                        class: Class::Modest,
                        backend: Backend::Gl,
                        tri_budget: tri,
                        draw_call_budget: dc,
                        lod_aggressiveness: Lod::Medium,
                        sancho: SanchoTier::Lod8k,
                        splat_render: true,
                        physics: Physics::Dynamic,
                        mem_budget_mb: mem,
                        background_jobs: bg,
                        path_tracer: true,
                    };
                    let parsed = parse(&to_toml(&p));
                    assert_eq!(
                        parsed,
                        Parsed {
                            profile: p,
                            problems: vec![]
                        }
                    );
                }
            }
        }
    }
}

// ---- parse worked examples and rules P1–P9 --------------------------------

#[test]
fn parse_empty_is_potato_with_all_eleven_missing() {
    let parsed = parse("");
    let problems: Vec<Problem> = KEYS
        .iter()
        .map(|k| Problem::MissingKey(k.to_string()))
        .collect();
    assert_eq!(
        parsed,
        Parsed {
            profile: potato(),
            problems
        }
    );
}

#[test]
fn parse_single_key_in_profile() {
    let parsed = parse("[profile]\nclass = \"capable\"");
    let mut want = potato();
    want.class = Class::Capable;
    let problems: Vec<Problem> = KEYS
        .iter()
        .filter(|k| **k != "class")
        .map(|k| Problem::MissingKey(k.to_string()))
        .collect();
    assert_eq!(
        parsed,
        Parsed {
            profile: want,
            problems
        }
    );
}

#[test]
fn parse_key_before_any_table_is_skipped() {
    let parsed = parse("class = \"capable\"");
    let problems: Vec<Problem> = KEYS
        .iter()
        .map(|k| Problem::MissingKey(k.to_string()))
        .collect();
    assert_eq!(
        parsed,
        Parsed {
            profile: potato(),
            problems
        }
    );
}

#[test]
fn parse_comment_after_a_value() {
    let text = "[profile]\n\
class = \"potato\"\n\
backend = \"dx12\"\n\
tri_budget = 1_000 # a comment\n\
draw_call_budget = 60\n\
lod_aggressiveness = \"high\"\n\
sancho = \"baked_vat\"\n\
splat_render = false\n\
physics = \"kinematic\"\n\
mem_budget_mb = 2_500\n\
background_jobs = 2\n\
path_tracer = false\n";
    let mut want = potato();
    want.tri_budget = 1000;
    let parsed = parse(text);
    assert_eq!(
        parsed,
        Parsed {
            profile: want,
            problems: vec![]
        }
    );
}

#[test]
fn parse_bad_value_wrong_case() {
    let parsed = parse("[profile]\nclass = \"Potato\"");
    let mut problems = vec![Problem::BadValue("class".into())];
    for k in KEYS.iter().filter(|k| **k != "class") {
        problems.push(Problem::MissingKey(k.to_string()));
    }
    assert_eq!(
        parsed,
        Parsed {
            profile: potato(),
            problems
        }
    );
}

#[test]
fn parse_unknown_key() {
    let parsed = parse("[profile]\ncolour = \"red\"");
    let mut problems = vec![Problem::UnknownKey("colour".into())];
    for k in KEYS.iter() {
        problems.push(Problem::MissingKey(k.to_string()));
    }
    assert_eq!(
        parsed,
        Parsed {
            profile: potato(),
            problems
        }
    );
}

#[test]
fn parse_duplicate_key_the_earlier_line_stands() {
    let parsed = parse("[profile]\nclass = \"modest\"\nclass = \"capable\"");
    let mut want = potato();
    want.class = Class::Modest;
    let mut problems = vec![Problem::Duplicate("class".into())];
    for k in KEYS.iter().filter(|k| **k != "class") {
        problems.push(Problem::MissingKey(k.to_string()));
    }
    assert_eq!(
        parsed,
        Parsed {
            profile: want,
            problems
        }
    );
}

#[test]
fn parse_bad_line_without_equals() {
    let parsed = parse("[profile]\nhello");
    let mut problems = vec![Problem::BadLine(2)];
    for k in KEYS.iter() {
        problems.push(Problem::MissingKey(k.to_string()));
    }
    assert_eq!(
        parsed,
        Parsed {
            profile: potato(),
            problems
        }
    );
}

#[test]
fn parse_bad_line_open_bracket() {
    let parsed = parse("[profile");
    let mut problems = vec![Problem::BadLine(1)];
    for k in KEYS.iter() {
        problems.push(Problem::MissingKey(k.to_string()));
    }
    assert_eq!(
        parsed,
        Parsed {
            profile: potato(),
            problems
        }
    );
}

#[test]
fn parse_other_table_then_profile() {
    let parsed = parse("[other]\nclass = \"capable\"\n[profile]\nclass = \"modest\"");
    let mut want = potato();
    want.class = Class::Modest;
    let problems: Vec<Problem> = KEYS
        .iter()
        .filter(|k| **k != "class")
        .map(|k| Problem::MissingKey(k.to_string()))
        .collect();
    assert_eq!(
        parsed,
        Parsed {
            profile: want,
            problems
        }
    );
}

#[test]
fn parse_integers_good() {
    for (text, want) in [
        ("2_500", 2500u32),
        ("0", 0u32),
        ("4294967295", 4294967295u32),
    ] {
        let parsed = parse(&format!("[profile]\ntri_budget = {}", text));
        let mut want_profile = potato();
        want_profile.tri_budget = want;
        let problems: Vec<Problem> = KEYS
            .iter()
            .filter(|k| **k != "tri_budget")
            .map(|k| Problem::MissingKey(k.to_string()))
            .collect();
        assert_eq!(
            parsed,
            Parsed {
                profile: want_profile,
                problems
            }
        );
    }
}

#[test]
fn parse_integers_bad() {
    for text in [
        "2__500",
        "_2",
        "2_",
        "007",
        "-1",
        "4294967296",
        "1.5",
        "\"2\"",
        "",
    ] {
        let parsed = parse(&format!("[profile]\ntri_budget = {}", text));
        let mut problems = vec![Problem::BadValue("tri_budget".into())];
        for k in KEYS.iter().filter(|k| **k != "tri_budget") {
            problems.push(Problem::MissingKey(k.to_string()));
        }
        assert_eq!(
            parsed,
            Parsed {
                profile: potato(),
                problems
            }
        );
    }
}

#[test]
fn parse_booleans_good_and_bad() {
    for (text, want) in [("true", true), ("false", false)] {
        let parsed = parse(&format!("[profile]\nsplat_render = {}", text));
        let mut want_profile = potato();
        want_profile.splat_render = want;
        let problems: Vec<Problem> = KEYS
            .iter()
            .filter(|k| **k != "splat_render")
            .map(|k| Problem::MissingKey(k.to_string()))
            .collect();
        assert_eq!(
            parsed,
            Parsed {
                profile: want_profile,
                problems
            }
        );
    }
    for text in ["True", "1", "\"true\""] {
        let parsed = parse(&format!("[profile]\nsplat_render = {}", text));
        let mut problems = vec![Problem::BadValue("splat_render".into())];
        for k in KEYS.iter().filter(|k| **k != "splat_render") {
            problems.push(Problem::MissingKey(k.to_string()));
        }
        assert_eq!(
            parsed,
            Parsed {
                profile: potato(),
                problems
            }
        );
    }
}

#[test]
fn parse_hash_inside_quotes_is_part_of_the_value() {
    let parsed = parse("[profile]\nclass = \"pot#ato\"");
    let mut problems = vec![Problem::BadValue("class".into())];
    for k in KEYS.iter().filter(|k| **k != "class") {
        problems.push(Problem::MissingKey(k.to_string()));
    }
    assert_eq!(
        parsed,
        Parsed {
            profile: potato(),
            problems
        }
    );
}

#[test]
fn parse_crlf_line_endings() {
    let text = "[profile]\r\nclass = \"modest\"\r\nbackend = \"gl\"\r\n";
    let parsed = parse(text);
    let mut want = potato();
    want.class = Class::Modest;
    want.backend = Backend::Gl;
    let problems: Vec<Problem> = KEYS
        .iter()
        .filter(|k| **k != "class" && **k != "backend")
        .map(|k| Problem::MissingKey(k.to_string()))
        .collect();
    assert_eq!(
        parsed,
        Parsed {
            profile: want,
            problems
        }
    );
}

#[test]
fn parse_table_header_with_spaces() {
    let parsed = parse("[ profile ]\nclass = \"modest\"");
    let mut want = potato();
    want.class = Class::Modest;
    let problems: Vec<Problem> = KEYS
        .iter()
        .filter(|k| **k != "class")
        .map(|k| Problem::MissingKey(k.to_string()))
        .collect();
    assert_eq!(
        parsed,
        Parsed {
            profile: want,
            problems
        }
    );
}

#[test]
fn parse_second_profile_table() {
    let parsed = parse("[profile]\nclass = \"modest\"\n[profile]\nbackend = \"gl\"");
    let mut want = potato();
    want.class = Class::Modest;
    want.backend = Backend::Gl;
    let problems: Vec<Problem> = KEYS
        .iter()
        .filter(|k| **k != "class" && **k != "backend")
        .map(|k| Problem::MissingKey(k.to_string()))
        .collect();
    assert_eq!(
        parsed,
        Parsed {
            profile: want,
            problems
        }
    );
}

// ---- odd inputs that must never panic -------------------------------------

#[test]
fn odd_texts_never_panic() {
    let texts = [
        "",
        "[",
        "=",
        "\"",
        "[profile]\nclass = \"café\"",
        "[profile]\nclass = \"🍟\"",
        "[profile]\ntri_budget = 99999999999999999999",
        "[profile]\ntri_budget = 18446744073709551616",
        "\r\n\r\n\r\n",
        "class = \"potato\" = \"modest\"",
        "[profile]\n= 5",
        "[profile]\ntri_budget =",
        "[profile]\ntri_budget = _",
        "[profile]\ntri_budget = 0_0",
        "[profile]\ntri_budget = 00",
        "[profile]\nsplat_render = TRUE",
        "[profile]\npath_tracer = 0",
        "[profile]\nbackend = \"vulkan\"",
        "[profile]\nphysics = \"quantum\"",
        "[ profile ]\nclass = \"modest\"",
        "[other]\nhello",
        "[profile]\nclass",
        "[profile]\ntri_budget = 1_000_000_000",
    ];
    for text in texts {
        let parsed = parse(text);
        // The result is always a whole, valid profile.
        assert!(matches!(
            parsed.profile.class,
            Class::Potato | Class::Modest | Class::Capable
        ));
        assert!(matches!(
            parsed.profile.backend,
            Backend::Dx12 | Backend::Gl | Backend::Cpu
        ));
    }
    // A few of these have exact, checkable results.
    assert_eq!(parse("=").problems[0], Problem::BadLine(1));
    assert_eq!(parse("[").problems[0], Problem::BadLine(1));
    assert_eq!(parse("\"").problems[0], Problem::BadLine(1));
}

#[test]
fn odd_facts_never_panic() {
    let list = vec![
        facts(vec![], f64::NAN, f64::NAN, 0, 0, None),
        facts(
            vec![],
            f64::INFINITY,
            f64::INFINITY,
            u32::MAX,
            u64::MAX,
            Some(u64::MAX),
        ),
        facts(
            vec![],
            f64::NEG_INFINITY,
            f64::NEG_INFINITY,
            u32::MAX,
            u64::MAX,
            Some(u64::MAX),
        ),
        facts(vec![attempt(Backend::Cpu, true)], 1.0, 1.0, 1, 1, Some(1)),
        facts(
            vec![attempt(Backend::Dx12, true)],
            0.0,
            0.0,
            8,
            28_672,
            Some(6_144),
        ),
        facts(
            vec![attempt(Backend::Dx12, false)],
            f64::NAN,
            f64::INFINITY,
            u32::MAX,
            u64::MAX,
            None,
        ),
    ];
    for f in &list {
        let profile = from_facts(Some(f));
        assert!(matches!(
            profile.class,
            Class::Potato | Class::Modest | Class::Capable
        ));
        assert!(matches!(
            profile.backend,
            Backend::Dx12 | Backend::Gl | Backend::Cpu
        ));
    }
    assert_eq!(from_facts(None), potato());
}
