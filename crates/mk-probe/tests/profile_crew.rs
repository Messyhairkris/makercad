//! Tests for mk_probe::profile written by Gordon (the Crew, campaign mk-core-6) from the contract
//! alone, without seeing the module.

//! Integration tests for `profile.rs`, written from the contract alone.

use mk_probe::profile::{
    choose_backend, classify, defaults, from_facts, group_digits, parse, potato, to_toml, Attempt,
    Backend, Class, Facts, Lod, Parsed, Physics, Problem, Profile, SanchoTier, BACKEND_ORDER, KEYS,
};

fn ok(backend: Backend) -> Attempt {
    Attempt {
        backend,
        initialised: true,
    }
}

fn failed(backend: Backend) -> Attempt {
    Attempt {
        backend,
        initialised: false,
    }
}

fn facts(
    attempts: Vec<Attempt>,
    frame_p95_ms: f64,
    sancho_vat_ms: f64,
    cores: u32,
    ram_total_mb: u64,
    vram_mb: Option<u64>,
) -> Facts {
    Facts {
        attempts,
        frame_p95_ms,
        sancho_vat_ms,
        cores,
        ram_total_mb,
        vram_mb,
    }
}

fn missing(keys: &[&str]) -> Vec<Problem> {
    keys.iter()
        .map(|k| Problem::MissingKey((*k).to_string()))
        .collect()
}

fn with_backend(mut p: Profile, backend: Backend) -> Profile {
    p.backend = backend;
    p
}

// KEYS order: class, backend, tri_budget, draw_call_budget, lod_aggressiveness, sancho,
// splat_render, physics, mem_budget_mb, background_jobs, path_tracer.
const ALL_BUT_CLASS: [&str; 10] = [
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
];

const ALL_BUT_BACKEND: [&str; 10] = [
    "class",
    "tri_budget",
    "draw_call_budget",
    "lod_aggressiveness",
    "sancho",
    "splat_render",
    "physics",
    "mem_budget_mb",
    "background_jobs",
    "path_tracer",
];

const ALL_BUT_TRI: [&str; 10] = [
    "class",
    "backend",
    "draw_call_budget",
    "lod_aggressiveness",
    "sancho",
    "splat_render",
    "physics",
    "mem_budget_mb",
    "background_jobs",
    "path_tracer",
];

const ALL_BUT_SPLAT: [&str; 10] = [
    "class",
    "backend",
    "tri_budget",
    "draw_call_budget",
    "lod_aggressiveness",
    "sancho",
    "physics",
    "mem_budget_mb",
    "background_jobs",
    "path_tracer",
];

#[test]
fn defaults_table_potato_column_field_by_field() {
    let want = Profile {
        class: Class::Potato,
        backend: Backend::Dx12,
        tri_budget: 200_000,
        draw_call_budget: 60,
        lod_aggressiveness: Lod::High,
        sancho: SanchoTier::BakedVat,
        splat_render: false,
        physics: Physics::Kinematic,
        mem_budget_mb: 2_500,
        background_jobs: 2,
        path_tracer: false,
    };
    assert_eq!(defaults(Class::Potato), want);
}

#[test]
fn defaults_table_modest_column_field_by_field() {
    let want = Profile {
        class: Class::Modest,
        backend: Backend::Dx12,
        tri_budget: 1_000_000,
        draw_call_budget: 200,
        lod_aggressiveness: Lod::Medium,
        sancho: SanchoTier::Lod8k,
        splat_render: true,
        physics: Physics::Dynamic,
        mem_budget_mb: 6_000,
        background_jobs: 3,
        path_tracer: false,
    };
    assert_eq!(defaults(Class::Modest), want);
}

#[test]
fn defaults_table_capable_column_field_by_field() {
    let want = Profile {
        class: Class::Capable,
        backend: Backend::Dx12,
        tri_budget: 4_000_000,
        draw_call_budget: 800,
        lod_aggressiveness: Lod::Low,
        sancho: SanchoTier::Full,
        splat_render: true,
        physics: Physics::Dynamic,
        mem_budget_mb: 16_000,
        background_jobs: 4,
        path_tracer: true,
    };
    assert_eq!(defaults(Class::Capable), want);
}

#[test]
fn potato_is_defaults_of_potato_class() {
    assert_eq!(potato(), defaults(Class::Potato));
}

#[test]
fn rule_t1_t3_to_toml_potato_byte_for_byte() {
    let want = concat!(
        "[profile]\n",
        "class = \"potato\"\n",
        "backend = \"dx12\"\n",
        "tri_budget = 200_000\n",
        "draw_call_budget = 60\n",
        "lod_aggressiveness = \"high\"\n",
        "sancho = \"baked_vat\"\n",
        "splat_render = false\n",
        "physics = \"kinematic\"\n",
        "mem_budget_mb = 2_500\n",
        "background_jobs = 2\n",
        "path_tracer = false\n",
    );
    assert_eq!(to_toml(&potato()), want);
}

#[test]
fn rule_t2_group_digits_boundaries() {
    assert_eq!(group_digits(0), "0");
    assert_eq!(group_digits(60), "60");
    assert_eq!(group_digits(999), "999");
    assert_eq!(group_digits(1_000), "1_000");
    assert_eq!(group_digits(2_500), "2_500");
    assert_eq!(group_digits(200_000), "200_000");
    assert_eq!(group_digits(1_000_000), "1_000_000");
    assert_eq!(group_digits(4_294_967_295), "4_294_967_295");
}

#[test]
fn constants_backend_order_and_keys() {
    assert_eq!(BACKEND_ORDER, [Backend::Dx12, Backend::Gl]);
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

#[test]
fn rule_b1_choose_backend_worked_examples() {
    assert_eq!(choose_backend(&[]), Backend::Cpu);
    assert_eq!(choose_backend(&[ok(Backend::Dx12)]), Backend::Dx12);
    // BACKEND_ORDER decides, not attempt order.
    assert_eq!(
        choose_backend(&[ok(Backend::Gl), ok(Backend::Dx12)]),
        Backend::Dx12
    );
    // Only the first dx12 attempt counts, so gl wins.
    assert_eq!(
        choose_backend(&[failed(Backend::Dx12), ok(Backend::Dx12), ok(Backend::Gl)]),
        Backend::Gl
    );
    // A Cpu attempt is never a probe attempt.
    assert_eq!(choose_backend(&[ok(Backend::Cpu)]), Backend::Cpu);
}

#[test]
fn rule_b1_first_attempt_of_each_backend_only() {
    // The only dx12 attempt failed, so dx12 is out and there is no gl attempt.
    assert_eq!(choose_backend(&[failed(Backend::Dx12)]), Backend::Cpu);
    // The first gl attempt failed; the later good one does not count.
    assert_eq!(
        choose_backend(&[failed(Backend::Gl), ok(Backend::Gl)]),
        Backend::Cpu
    );
    // Both first attempts failed, so the later good dx12 does not count either.
    assert_eq!(
        choose_backend(&[
            failed(Backend::Gl),
            failed(Backend::Dx12),
            ok(Backend::Dx12)
        ]),
        Backend::Cpu
    );
    // A good first attempt wins even when a later attempt of that backend failed.
    assert_eq!(
        choose_backend(&[ok(Backend::Gl), failed(Backend::Gl)]),
        Backend::Gl
    );
}

#[test]
fn rule_k1_cpu_backend_is_potato() {
    let f = facts(vec![], 1.0, 0.1, 16, 65_536, Some(16_384));
    assert_eq!(classify(&f, Backend::Cpu), Class::Potato);
}

#[test]
fn rule_k2_capable_bounds_inclusive() {
    // Every bound exactly on its threshold.
    let edge = facts(vec![], 10.0, 0.1, 8, 28_672, Some(6_144));
    assert_eq!(classify(&edge, Backend::Dx12), Class::Capable);
    let fast = facts(vec![], 0.0, 0.1, 8, 28_672, Some(6_144));
    assert_eq!(classify(&fast, Backend::Dx12), Class::Capable);
    // One step below each bound falls to K3 (modest), never capable.
    let cores = facts(vec![], 10.0, 0.1, 7, 28_672, Some(6_144));
    assert_eq!(classify(&cores, Backend::Dx12), Class::Modest);
    let ram = facts(vec![], 10.0, 0.1, 8, 28_671, Some(6_144));
    assert_eq!(classify(&ram, Backend::Dx12), Class::Modest);
    let vram = facts(vec![], 10.0, 0.1, 8, 28_672, Some(6_143));
    assert_eq!(classify(&vram, Backend::Dx12), Class::Modest);
    let none = facts(vec![], 10.0, 0.1, 8, 28_672, None);
    assert_eq!(classify(&none, Backend::Dx12), Class::Modest);
    let slow = facts(vec![], 10.000_1, 0.1, 8, 28_672, Some(6_144));
    assert_eq!(classify(&slow, Backend::Dx12), Class::Modest);
}

#[test]
fn rule_k3_modest_bounds_inclusive() {
    let edge = facts(vec![], 20.0, 0.1, 4, 12_288, None);
    assert_eq!(classify(&edge, Backend::Dx12), Class::Modest);
    let fast = facts(vec![], 0.0, 0.1, 4, 12_288, None);
    assert_eq!(classify(&fast, Backend::Dx12), Class::Modest);
    let cores = facts(vec![], 20.0, 0.1, 3, 12_288, None);
    assert_eq!(classify(&cores, Backend::Dx12), Class::Potato);
    let ram = facts(vec![], 20.0, 0.1, 4, 12_287, None);
    assert_eq!(classify(&ram, Backend::Dx12), Class::Potato);
    let slow = facts(vec![], 20.5, 0.1, 4, 12_288, None);
    assert_eq!(classify(&slow, Backend::Dx12), Class::Potato);
}

#[test]
fn rule_k4_otherwise_potato() {
    let weak = facts(vec![], 100.0, 0.1, 1, 100, None);
    assert_eq!(classify(&weak, Backend::Dx12), Class::Potato);
    let weak_gl = facts(vec![], 100.0, 0.1, 1, 100, None);
    assert_eq!(classify(&weak_gl, Backend::Gl), Class::Potato);
}

#[test]
fn rule_k2_k3_nan_negative_infinite_never_within() {
    let capable_specs = |t: f64| facts(vec![], t, 0.1, 16, 65_536, Some(16_384));
    assert_eq!(
        classify(&capable_specs(f64::NAN), Backend::Dx12),
        Class::Potato
    );
    assert_eq!(classify(&capable_specs(-1.0), Backend::Dx12), Class::Potato);
    assert_eq!(
        classify(&capable_specs(f64::INFINITY), Backend::Dx12),
        Class::Potato
    );
    assert_eq!(
        classify(&capable_specs(f64::NEG_INFINITY), Backend::Dx12),
        Class::Potato
    );
    let modest_specs = |t: f64| facts(vec![], t, 0.1, 6, 15_800, None);
    assert_eq!(
        classify(&modest_specs(f64::NAN), Backend::Dx12),
        Class::Potato
    );
    assert_eq!(classify(&modest_specs(-0.5), Backend::Dx12), Class::Potato);
}

#[test]
fn rule_f1_none_gives_potato() {
    assert_eq!(from_facts(None), potato());
}

#[test]
fn rule_f4_sancho_portrait_only_for_potato_over_two_ms() {
    let portrait = {
        let mut p = potato();
        p.sancho = SanchoTier::Portrait;
        p
    };
    let potato_facts = |vat: f64| facts(vec![ok(Backend::Dx12)], 31.0, vat, 4, 7_900, None);
    assert_eq!(from_facts(Some(&potato_facts(2.5))), portrait);
    assert_eq!(from_facts(Some(&potato_facts(f64::NAN))), portrait);
    assert_eq!(from_facts(Some(&potato_facts(f64::INFINITY))), portrait);
    assert_eq!(from_facts(Some(&potato_facts(-1.0))), portrait);
    // Exactly 2.0 keeps BakedVat.
    assert_eq!(from_facts(Some(&potato_facts(2.0))), potato());
    assert_eq!(from_facts(Some(&potato_facts(1.5))), potato());
    // F4 is for potato only: a modest machine keeps its Lod8k.
    let modest = facts(vec![ok(Backend::Dx12)], 15.0, 3.0, 6, 15_800, None);
    assert_eq!(from_facts(Some(&modest)), defaults(Class::Modest));
}

#[test]
fn worked_examples_from_facts_table() {
    let portrait = {
        let mut p = potato();
        p.sancho = SanchoTier::Portrait;
        p
    };
    // Row 1.
    assert_eq!(from_facts(None), potato());
    // Row 2: this machine.
    let this_machine = facts(vec![ok(Backend::Dx12)], 31.0, 1.5, 4, 7_900, None);
    assert_eq!(from_facts(Some(&this_machine)), potato());
    // Row 3.
    let slow_vat = facts(vec![ok(Backend::Dx12)], 31.0, 2.5, 4, 7_900, None);
    assert_eq!(from_facts(Some(&slow_vat)), portrait);
    // Row 4: exactly 2.0 keeps BakedVat.
    let edge_vat = facts(vec![ok(Backend::Dx12)], 31.0, 2.0, 4, 7_900, None);
    assert_eq!(from_facts(Some(&edge_vat)), potato());
    // Row 5.
    let nan_vat = facts(vec![ok(Backend::Dx12)], 31.0, f64::NAN, 4, 7_900, None);
    assert_eq!(from_facts(Some(&nan_vat)), portrait);
    // Row 6.
    let gl_only = facts(
        vec![failed(Backend::Dx12), ok(Backend::Gl)],
        31.0,
        1.0,
        4,
        7_900,
        None,
    );
    assert_eq!(
        from_facts(Some(&gl_only)),
        with_backend(potato(), Backend::Gl)
    );
    // Row 7: K1, VAT 1.0 keeps BakedVat.
    let all_failed = facts(
        vec![failed(Backend::Dx12), failed(Backend::Gl)],
        5.0,
        1.0,
        16,
        65_536,
        Some(16_384),
    );
    assert_eq!(
        from_facts(Some(&all_failed)),
        with_backend(potato(), Backend::Cpu)
    );
    // Row 8.
    let no_attempts = facts(vec![], 5.0, 1.0, 16, 65_536, Some(16_384));
    assert_eq!(
        from_facts(Some(&no_attempts)),
        with_backend(potato(), Backend::Cpu)
    );
    // Row 9.
    let modest = facts(vec![ok(Backend::Dx12)], 15.0, 0.5, 6, 15_800, None);
    assert_eq!(from_facts(Some(&modest)), defaults(Class::Modest));
    // Row 10: every bound inclusive.
    let modest_edge = facts(vec![ok(Backend::Dx12)], 20.0, 0.5, 4, 12_288, None);
    assert_eq!(from_facts(Some(&modest_edge)), defaults(Class::Modest));
    // Row 11: too slow for modest.
    let too_slow = facts(vec![ok(Backend::Dx12)], 20.5, 0.5, 6, 15_800, None);
    assert_eq!(from_facts(Some(&too_slow)), potato());
    // Row 12: F4 is for potato only.
    let modest_slow_vat = facts(vec![ok(Backend::Dx12)], 15.0, 3.0, 6, 15_800, None);
    assert_eq!(from_facts(Some(&modest_slow_vat)), defaults(Class::Modest));
    // Row 13.
    let capable_gl = facts(vec![ok(Backend::Gl)], 8.0, 0.3, 8, 32_000, Some(8_192));
    assert_eq!(
        from_facts(Some(&capable_gl)),
        with_backend(defaults(Class::Capable), Backend::Gl)
    );
    // Row 14: no VRAM reported.
    let no_vram = facts(vec![ok(Backend::Dx12)], 8.0, 0.3, 8, 32_000, None);
    assert_eq!(from_facts(Some(&no_vram)), defaults(Class::Modest));
    // Row 15.
    let small_vram = facts(vec![ok(Backend::Dx12)], 8.0, 0.3, 8, 32_000, Some(6_143));
    assert_eq!(from_facts(Some(&small_vram)), defaults(Class::Modest));
    // Row 16: too slow for capable.
    let slow_capable = facts(vec![ok(Backend::Dx12)], 10.5, 0.3, 12, 65_536, Some(12_288));
    assert_eq!(from_facts(Some(&slow_capable)), defaults(Class::Modest));
    // Row 17: a negative time is not a time.
    let negative = facts(vec![ok(Backend::Dx12)], -1.0, 0.3, 12, 65_536, Some(12_288));
    assert_eq!(from_facts(Some(&negative)), potato());
    // Row 18: 7 cores.
    let seven_cores = facts(vec![ok(Backend::Dx12)], 8.0, 0.3, 7, 65_536, Some(12_288));
    assert_eq!(from_facts(Some(&seven_cores)), defaults(Class::Modest));
}

#[test]
fn rule_p1_crlf_line_ends() {
    let text = "[profile]\r\nclass = \"capable\"\r\n";
    let mut want = potato();
    want.class = Class::Capable;
    assert_eq!(
        parse(text),
        Parsed {
            profile: want,
            problems: missing(&ALL_BUT_CLASS)
        }
    );
    // A whole file with CRLF line ends reads back unchanged.
    let full = to_toml(&defaults(Class::Capable)).replace('\n', "\r\n");
    assert_eq!(
        parse(&full),
        Parsed {
            profile: defaults(Class::Capable),
            problems: vec![]
        }
    );
}

#[test]
fn rule_p2_blank_and_comment_lines_skipped() {
    let text = "\n# a comment\n[profile]\n\n   \nclass = \"capable\"\n# another\n";
    let mut want = potato();
    want.class = Class::Capable;
    assert_eq!(
        parse(text),
        Parsed {
            profile: want,
            problems: missing(&ALL_BUT_CLASS)
        }
    );
}

#[test]
fn rule_p3_table_header_and_bad_header() {
    // An unterminated header is a BadLine(1), then every key is missing.
    assert_eq!(
        parse("[profile"),
        Parsed {
            profile: potato(),
            problems: {
                let mut p = vec![Problem::BadLine(1)];
                p.extend(missing(&KEYS));
                p
            },
        }
    );
    // The table name is trimmed.
    let mut want = potato();
    want.class = Class::Capable;
    assert_eq!(
        parse("[ profile ]\nclass = \"capable\"\n"),
        Parsed {
            profile: want,
            problems: missing(&ALL_BUT_CLASS)
        }
    );
    // A key line in another table is skipped (P5), so class stays potato.
    assert_eq!(
        parse("[profile]\n[other]\nclass = \"capable\"\n"),
        Parsed {
            profile: potato(),
            problems: missing(&KEYS)
        }
    );
}

#[test]
fn rule_p4_bad_line_and_empty_key() {
    assert_eq!(
        parse("[profile]\nhello\n"),
        Parsed {
            profile: potato(),
            problems: {
                let mut p = vec![Problem::BadLine(2)];
                p.extend(missing(&KEYS));
                p
            },
        }
    );
    // P4 comes first in every table, inside `profile` or not.
    assert_eq!(
        parse("[other]\nhello\n"),
        Parsed {
            profile: potato(),
            problems: {
                let mut p = vec![Problem::BadLine(2)];
                p.extend(missing(&KEYS));
                p
            },
        }
    );
    // An empty key is a BadLine too.
    assert_eq!(
        parse("[profile]\n= 5\n"),
        Parsed {
            profile: potato(),
            problems: {
                let mut p = vec![Problem::BadLine(2)];
                p.extend(missing(&KEYS));
                p
            },
        }
    );
}

#[test]
fn rule_p5_keys_outside_profile_table_skipped() {
    let text = "class = \"capable\"\n[profile]\nbackend = \"gl\"\n";
    let mut want = potato();
    want.backend = Backend::Gl;
    assert_eq!(
        parse(text),
        Parsed {
            profile: want,
            problems: missing(&ALL_BUT_BACKEND)
        }
    );
}

#[test]
fn rule_p6_unknown_key() {
    let text = "[profile]\ncolour = \"red\"\n";
    let mut want = vec![Problem::UnknownKey("colour".to_string())];
    want.extend(missing(&KEYS));
    assert_eq!(
        parse(text),
        Parsed {
            profile: potato(),
            problems: want
        }
    );
}

#[test]
fn rule_p7_duplicate_key_earlier_line_stands() {
    let text = "[profile]\nclass = \"modest\"\nclass = \"capable\"\n";
    let mut want = potato();
    want.class = Class::Modest;
    let mut problems = vec![Problem::Duplicate("class".to_string())];
    problems.extend(missing(&ALL_BUT_CLASS));
    assert_eq!(
        parse(text),
        Parsed {
            profile: want,
            problems
        }
    );

    // A bad first value still counts as named, so the second line is a Duplicate.
    let text = "[profile]\nclass = \"nope\"\nclass = \"capable\"\n";
    let mut problems = vec![
        Problem::BadValue("class".to_string()),
        Problem::Duplicate("class".to_string()),
    ];
    problems.extend(missing(&ALL_BUT_CLASS));
    assert_eq!(
        parse(text),
        Parsed {
            profile: potato(),
            problems
        }
    );
}

#[test]
fn rule_p8_bad_integers() {
    let bad = [
        "2__500",
        "_2",
        "2_",
        "007",
        "-1",
        "4294967296",
        "1.5",
        "\"2\"",
        "",
    ];
    for value in bad {
        let text = format!("[profile]\ntri_budget = {}\n", value);
        let mut want = vec![Problem::BadValue("tri_budget".to_string())];
        want.extend(missing(&ALL_BUT_TRI));
        assert_eq!(
            parse(&text),
            Parsed {
                profile: potato(),
                problems: want
            }
        );
    }
}

#[test]
fn rule_p8_good_integers() {
    let good = [("2_500", 2_500u32), ("0", 0), ("4294967295", 4_294_967_295)];
    for (value, n) in good {
        let text = format!("[profile]\ntri_budget = {}\n", value);
        let mut want = potato();
        want.tri_budget = n;
        assert_eq!(
            parse(&text),
            Parsed {
                profile: want,
                problems: missing(&ALL_BUT_TRI)
            }
        );
    }
}

#[test]
fn rule_p8_bad_booleans() {
    let bad = ["True", "1", "\"true\""];
    for value in bad {
        let text = format!("[profile]\nsplat_render = {}\n", value);
        let mut want = vec![Problem::BadValue("splat_render".to_string())];
        want.extend(missing(&ALL_BUT_SPLAT));
        assert_eq!(
            parse(&text),
            Parsed {
                profile: potato(),
                problems: want
            }
        );
    }
    // The good ones are stored.
    let mut want = potato();
    want.splat_render = true;
    assert_eq!(
        parse("[profile]\nsplat_render = true\n"),
        Parsed {
            profile: want,
            problems: missing(&ALL_BUT_SPLAT)
        }
    );
    assert_eq!(
        parse("[profile]\nsplat_render = false\n"),
        Parsed {
            profile: potato(),
            problems: missing(&ALL_BUT_SPLAT)
        }
    );
}

#[test]
fn rule_p8_bad_words() {
    let bad = ["\"Potato\"", "potato", "\"potato \"", "\"potato"];
    for value in bad {
        let text = format!("[profile]\nclass = {}\n", value);
        let mut want = vec![Problem::BadValue("class".to_string())];
        want.extend(missing(&ALL_BUT_CLASS));
        assert_eq!(
            parse(&text),
            Parsed {
                profile: potato(),
                problems: want
            }
        );
    }
    // Other word fields are checked against their own word lists.
    let text = "[profile]\nsancho = \"baked\"\n";
    let mut want = vec![Problem::BadValue("sancho".to_string())];
    want.extend(missing(&[
        "class",
        "backend",
        "tri_budget",
        "draw_call_budget",
        "lod_aggressiveness",
        "splat_render",
        "physics",
        "mem_budget_mb",
        "background_jobs",
        "path_tracer",
    ]));
    assert_eq!(
        parse(text),
        Parsed {
            profile: potato(),
            problems: want
        }
    );
    let text = "[profile]\nphysics = \"Kinematic\"\n";
    let mut want = vec![Problem::BadValue("physics".to_string())];
    want.extend(missing(&[
        "class",
        "backend",
        "tri_budget",
        "draw_call_budget",
        "lod_aggressiveness",
        "sancho",
        "splat_render",
        "mem_budget_mb",
        "background_jobs",
        "path_tracer",
    ]));
    assert_eq!(
        parse(text),
        Parsed {
            profile: potato(),
            problems: want
        }
    );
}

#[test]
fn rule_p4_comment_after_value_and_hash_inside_quotes() {
    // A comment after a value is cut off, so tri_budget is 1000 and nothing is wrong.
    let text = concat!(
        "[profile]\n",
        "class = \"potato\"\n",
        "backend = \"dx12\"\n",
        "tri_budget = 1_000 # a comment\n",
        "draw_call_budget = 60\n",
        "lod_aggressiveness = \"high\"\n",
        "sancho = \"baked_vat\"\n",
        "splat_render = false\n",
        "physics = \"kinematic\"\n",
        "mem_budget_mb = 2_500\n",
        "background_jobs = 2\n",
        "path_tracer = false\n",
    );
    let mut want = potato();
    want.tri_budget = 1_000;
    assert_eq!(
        parse(text),
        Parsed {
            profile: want,
            problems: vec![]
        }
    );

    // A `#` inside quotes is part of the value, so the word is wrong.
    let text = "[profile]\nclass = \"pot#ato\"\n";
    let mut want = vec![Problem::BadValue("class".to_string())];
    want.extend(missing(&ALL_BUT_CLASS));
    assert_eq!(
        parse(text),
        Parsed {
            profile: potato(),
            problems: want
        }
    );
}

#[test]
fn rule_p9_missing_keys_in_order() {
    assert_eq!(
        parse(""),
        Parsed {
            profile: potato(),
            problems: missing(&KEYS)
        }
    );
    let mut want = potato();
    want.class = Class::Capable;
    assert_eq!(
        parse("[profile]\nclass = \"capable\"\n"),
        Parsed {
            profile: want,
            problems: missing(&ALL_BUT_CLASS)
        }
    );
}

#[test]
fn round_trip_parse_to_toml_every_class_backend_lod_sancho_physics() {
    let classes = [Class::Potato, Class::Modest, Class::Capable];
    let backends = [Backend::Dx12, Backend::Gl, Backend::Cpu];
    let lods = [Lod::Low, Lod::Medium, Lod::High];
    let sanchos = [
        SanchoTier::Full,
        SanchoTier::Lod8k,
        SanchoTier::BakedVat,
        SanchoTier::Portrait,
    ];
    let physics = [Physics::Kinematic, Physics::Dynamic];
    for &class in &classes {
        for &backend in &backends {
            for &lod in &lods {
                for &sancho in &sanchos {
                    for &phys in &physics {
                        let mut p = defaults(class);
                        p.backend = backend;
                        p.lod_aggressiveness = lod;
                        p.sancho = sancho;
                        p.physics = phys;
                        p.tri_budget = 0;
                        p.draw_call_budget = 4_294_967_295;
                        p.mem_budget_mb = 1_000;
                        p.background_jobs = 999;
                        p.splat_render = true;
                        p.path_tracer = true;
                        let text = to_toml(&p);
                        assert_eq!(
                            parse(&text),
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

#[test]
fn odd_texts_never_panic() {
    let texts: [&str; 26] = [
        "",
        "\n",
        "\n\n\n",
        "   ",
        "\r\n",
        "[",
        "]",
        "[]",
        "[profile",
        "[profile]",
        "[profile]\n[profile]",
        "[[profile]]",
        "=",
        " = ",
        "class",
        "class =",
        "class = ",
        "[profile]\n=",
        "[profile]\n= 5",
        "[profile]\nclass = \"\u{1F600}\"",
        "[profile]\ntri_budget = 999999999999999999999",
        "[profile]\ntri_budget = 1_2_3",
        "[profile]\nclass = \"potato\" \"gl\"",
        "\u{1F600}\u{1F600}",
        "[profile]\nclass = \"potato\"#x",
        "[profile]\nclass = \"potato\" # x",
    ];
    for text in texts {
        // The contract says parse never fails; reaching the end is the assertion.
        let parsed = parse(text);
        let _ = parsed.profile.class;
    }
}

#[test]
fn odd_facts_never_panic() {
    let odd = [
        facts(vec![], f64::NAN, f64::NAN, 0, 0, None),
        facts(
            vec![],
            f64::INFINITY,
            f64::INFINITY,
            u32::MAX,
            u64::MAX,
            Some(u64::MAX),
        ),
        facts(vec![], f64::NEG_INFINITY, -1.0, 1, 1, Some(0)),
        facts(vec![ok(Backend::Cpu)], -0.0, 0.0, 8, 28_672, Some(6_144)),
        facts(
            vec![failed(Backend::Dx12), failed(Backend::Gl)],
            0.0,
            2.0,
            4,
            12_288,
            None,
        ),
        facts(
            vec![ok(Backend::Gl), ok(Backend::Dx12)],
            1e300,
            -1e300,
            3,
            0,
            Some(1),
        ),
    ];
    for f in &odd {
        let p = from_facts(Some(f));
        assert!(matches!(
            p.class,
            Class::Potato | Class::Modest | Class::Capable
        ));
        let c = classify(f, Backend::Dx12);
        assert!(matches!(c, Class::Potato | Class::Modest | Class::Capable));
        let b = choose_backend(&f.attempts);
        assert!(matches!(b, Backend::Dx12 | Backend::Gl | Backend::Cpu));
    }
    assert_eq!(from_facts(None), potato());
}
