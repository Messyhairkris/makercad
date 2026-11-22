//! Contract tests for mk_assist::memory, written by Gordon as a PA (the Crew) from
//! crew/contracts/mk_memory.md. They were the hidden tests of campaign mk-core-20.

use mk_assist::memory::{Day, Fact, Memory, Session, DEMOTE_AFTER, FAVOURITES};

fn session(day: i64, materials: &[&str], completed: Option<&str>) -> Session {
    Session {
        day: Day(day),
        materials: materials.iter().map(|name| name.to_string()).collect(),
        completed: completed.map(|name| name.to_string()),
    }
}

// M1 opting out

#[test]
fn m1_new_memory_is_not_opted_out() {
    let memory = Memory::new();
    assert!(!memory.opted_out());
    assert!(memory.last_day().is_none());
}

#[test]
fn m1_opted_out_clears_everything() {
    let mut memory = Memory::new();
    assert!(memory.record(&session(100, &["oak"], Some("Box"))));
    memory.add_override("fit.clearance");
    memory.set_opted_out(true);
    assert!(memory.opted_out());
    assert!(memory.last_day().is_none());
    assert!(memory.completed_projects() == 0);
    assert!(memory.material_count("oak") == 0);
    assert!(memory.favourites().is_empty());
    assert!(memory.overrides("fit.clearance") == 0);
    assert!(memory.demoted().is_empty());
    assert!(memory.greeting(Day(120), Some("x")).is_empty());
}

#[test]
fn m1_record_while_opted_out_is_ignored() {
    let mut memory = Memory::new();
    memory.set_opted_out(true);
    assert!(!memory.record(&session(100, &["oak"], Some("Box"))));
    assert!(memory.last_day().is_none());
    assert!(memory.days_since_last(Day(101)).is_none());
    assert!(memory.streak(Day(100)) == 0);
    assert!(memory.completed_projects() == 0);
    assert!(memory.material_count("oak") == 0);
}

#[test]
fn m1_overrides_while_opted_out_change_nothing() {
    let mut memory = Memory::new();
    memory.set_opted_out(true);
    assert!(memory.add_override("a") == 0);
    memory.set_overrides("a", 5);
    assert!(memory.overrides("a") == 0);
    assert!(memory.demoted().is_empty());
}

#[test]
fn m1_opt_back_in_starts_empty() {
    let mut memory = Memory::new();
    memory.record(&session(100, &["oak"], None));
    memory.add_override("r");
    memory.set_opted_out(true);
    memory.set_opted_out(false);
    assert!(!memory.opted_out());
    assert!(memory.last_day().is_none());
    assert!(memory.material_count("oak") == 0);
    assert!(memory.overrides("r") == 0);
    assert!(memory.greeting(Day(100), None).is_empty());
    assert!(memory.record(&session(100, &["oak"], None)));
    assert!(memory.last_day() == Some(Day(100)));
}

// M2 record

#[test]
fn m2_record_returns_true_and_keeps_the_day() {
    let mut memory = Memory::new();
    assert!(memory.record(&session(100, &[], None)));
    assert!(memory.last_day() == Some(Day(100)));
}

#[test]
fn m2_record_counts_each_material_once_per_session() {
    let mut memory = Memory::new();
    assert!(memory.record(&session(100, &["oak", "oak", "pine"], None)));
    assert!(memory.material_count("oak") == 1);
    assert!(memory.material_count("pine") == 1);
    assert!(memory.record(&session(101, &["oak"], None)));
    assert!(memory.material_count("oak") == 2);
}

#[test]
fn m2_record_skips_blank_materials() {
    let mut memory = Memory::new();
    memory.record(&session(1, &["", "   ", "\t", "\n"], None));
    assert!(memory.favourites().is_empty());
    assert!(memory.material_count("") == 0);
    assert!(memory.material_count("   ") == 0);
    assert!(memory.material_count("oak") == 0);
}

#[test]
fn m2_record_adds_completed_once() {
    let mut memory = Memory::new();
    assert!(memory.record(&session(1, &[], Some("Sharpener box"))));
    assert!(memory.completed_projects() == 1);
    assert!(memory.record(&session(2, &[], Some("Sharpener box"))));
    assert!(memory.completed_projects() == 1);
    assert!(memory.record(&session(3, &[], Some("  "))));
    assert!(memory.record(&session(4, &[], Some(""))));
    assert!(memory.completed_projects() == 1);
}

#[test]
fn m2_record_keeps_a_day_seen_twice() {
    let mut memory = Memory::new();
    memory.record(&session(50, &["oak"], None));
    memory.record(&session(50, &["pine"], None));
    assert!(memory.last_day() == Some(Day(50)));
    assert!(memory.streak(Day(50)) == 1);
    assert!(memory.streak(Day(51)) == 1);
}

// M3 last day and days since

#[test]
fn m3_last_day_is_the_latest_seen() {
    let mut memory = Memory::new();
    assert!(memory.last_day().is_none());
    memory.record(&session(100, &[], None));
    memory.record(&session(120, &[], None));
    memory.record(&session(90, &[], None));
    assert!(memory.last_day() == Some(Day(120)));
}

#[test]
fn m3_days_since_last_is_none_without_a_session() {
    let memory = Memory::new();
    assert!(memory.days_since_last(Day(10)).is_none());
}

#[test]
fn m3_days_since_last_counts_and_floors_at_zero() {
    let mut memory = Memory::new();
    memory.record(&session(110, &[], None));
    assert!(memory.days_since_last(Day(119)) == Some(9));
    assert!(memory.days_since_last(Day(110)) == Some(0));
    assert!(memory.days_since_last(Day(105)) == Some(0));
    assert!(memory.days_since_last(Day(i64::MIN)) == Some(0));
}

#[test]
fn m3_days_since_last_saturates_at_u32_max() {
    let mut memory = Memory::new();
    memory.record(&session(0, &[], None));
    assert!(memory.days_since_last(Day(i64::MAX)) == Some(u32::MAX));
    memory.record(&session(i64::MAX, &[], None));
    assert!(memory.days_since_last(Day(i64::MAX)) == Some(0));
}

// M4 streak

#[test]
fn m4_streak_ends_today() {
    let mut memory = Memory::new();
    for day in 100..=102 {
        memory.record(&session(day, &[], None));
    }
    assert!(memory.streak(Day(102)) == 3);
}

#[test]
fn m4_streak_ends_yesterday_when_today_has_no_session() {
    let mut memory = Memory::new();
    for day in 100..=102 {
        memory.record(&session(day, &[], None));
    }
    assert!(memory.streak(Day(103)) == 3);
}

#[test]
fn m4_streak_is_zero_without_a_recent_session() {
    let mut memory = Memory::new();
    for day in 100..=102 {
        memory.record(&session(day, &[], None));
    }
    assert!(memory.streak(Day(104)) == 0);
    assert!(memory.streak(Day(1000)) == 0);
    assert!(memory.streak(Day(99)) == 0);
}

#[test]
fn m4_streak_breaks_at_a_gap() {
    let mut memory = Memory::new();
    memory.record(&session(1, &[], None));
    memory.record(&session(2, &[], None));
    memory.record(&session(4, &[], None));
    memory.record(&session(5, &[], None));
    assert!(memory.streak(Day(5)) == 2);
    assert!(memory.streak(Day(6)) == 2);
    assert!(memory.streak(Day(4)) == 1);
}

#[test]
fn m4_streak_ignores_session_order() {
    let mut memory = Memory::new();
    memory.record(&session(105, &[], None));
    memory.record(&session(103, &[], None));
    memory.record(&session(104, &[], None));
    assert!(memory.streak(Day(105)) == 3);
    assert!(memory.streak(Day(106)) == 3);
}

#[test]
fn m4_streak_at_the_day_minimum_does_not_panic() {
    let mut memory = Memory::new();
    memory.record(&session(i64::MIN, &[], None));
    assert!(memory.streak(Day(i64::MIN)) == 1);
    assert!(memory.streak(Day(i64::MIN + 1)) == 1);
}

// M5 counts and favourites

#[test]
fn m5_completed_projects_counts_distinct_names() {
    let mut memory = Memory::new();
    memory.record(&session(1, &[], Some("A")));
    memory.record(&session(2, &[], Some("B")));
    memory.record(&session(3, &[], Some("A")));
    assert!(memory.completed_projects() == 2);
}

#[test]
fn m5_material_count_reads_counts() {
    let mut memory = Memory::new();
    memory.record(&session(1, &["oak", "pine"], None));
    memory.record(&session(2, &["oak"], None));
    assert!(memory.material_count("oak") == 2);
    assert!(memory.material_count("pine") == 1);
    assert!(memory.material_count("birch") == 0);
}

#[test]
fn m5_favourites_order_by_count_then_name() {
    let mut memory = Memory::new();
    memory.record(&session(1, &["oak", "pine", "birch"], None));
    memory.record(&session(2, &["oak", "pine"], None));
    memory.record(&session(3, &["oak"], None));
    assert!(
        memory.favourites() == vec!["oak".to_string(), "pine".to_string(), "birch".to_string()]
    );
}

#[test]
fn m5_favourites_ties_break_by_name_and_keep_three() {
    let mut memory = Memory::new();
    memory.record(&session(1, &["z", "b", "m", "a"], None));
    assert!(memory.favourites() == vec!["a".to_string(), "b".to_string(), "m".to_string()]);
}

#[test]
fn m5_favourites_is_empty_without_materials() {
    let memory = Memory::new();
    assert!(memory.favourites().is_empty());
}

#[test]
fn m5_favourites_never_exceed_the_limit() {
    let mut memory = Memory::new();
    memory.record(&session(1, &["a", "b", "c", "d", "e"], None));
    assert!(memory.favourites().len() == FAVOURITES);
    memory.record(&session(2, &["a"], None));
    assert!(memory.favourites().len() == FAVOURITES);
}

// M6 overrides

#[test]
fn m6_overrides_read_zero_when_unknown() {
    let memory = Memory::new();
    assert!(memory.overrides("nope") == 0);
    assert!(memory.demoted().is_empty());
}

#[test]
fn m6_add_override_counts_up_and_saturates() {
    let mut memory = Memory::new();
    assert!(memory.add_override("fit.clearance") == 1);
    assert!(memory.add_override("fit.clearance") == 2);
    assert!(memory.overrides("fit.clearance") == 2);
    memory.set_overrides("fit.clearance", u32::MAX);
    assert!(memory.add_override("fit.clearance") == u32::MAX);
}

#[test]
fn m6_set_overrides_replaces_and_zero_removes() {
    let mut memory = Memory::new();
    memory.add_override("r");
    memory.set_overrides("r", 7);
    assert!(memory.overrides("r") == 7);
    memory.set_overrides("r", 0);
    assert!(memory.overrides("r") == 0);
    assert!(memory.demoted().is_empty());
}

#[test]
fn m6_demoted_lists_rules_at_the_threshold_by_name() {
    let mut memory = Memory::new();
    memory.set_overrides("tools.noise", 1);
    for _ in 0..DEMOTE_AFTER {
        memory.add_override("fit.clearance");
    }
    memory.set_overrides("stock.grain", 5);
    assert!(memory.demoted() == vec!["fit.clearance".to_string(), "stock.grain".to_string()]);
}

// M7 greeting

#[test]
fn m7_greeting_facts_in_order() {
    let mut memory = Memory::new();
    memory.record(&session(100, &["oak", "pine"], Some("Box")));
    memory.record(&session(101, &["oak", "birch"], None));
    assert!(
        memory.greeting(Day(102), None)
            == vec![
                Fact::DaysSinceLast(1),
                Fact::Streak(2),
                Fact::CompletedProjects(1),
                Fact::Favourite("oak".to_string()),
            ]
    );
}

#[test]
fn m7_greeting_omits_facts_that_say_nothing() {
    let mut memory = Memory::new();
    assert!(memory.greeting(Day(1), None).is_empty());
    memory.record(&session(1, &[], None));
    assert!(memory.greeting(Day(1), None).is_empty());
}

#[test]
fn m7_greeting_trims_waiting_and_skips_blank() {
    let mut memory = Memory::new();
    memory.record(&session(10, &[], None));
    assert!(
        memory.greeting(Day(10), Some("  The drawer fronts  "))
            == vec![Fact::Waiting("The drawer fronts".to_string())]
    );
    assert!(memory.greeting(Day(10), Some("   ")).is_empty());
    assert!(memory.greeting(Day(10), Some("")).is_empty());
    assert!(memory.greeting(Day(10), None).is_empty());
}

#[test]
fn m7_greeting_uses_only_the_first_favourite() {
    let mut memory = Memory::new();
    memory.record(&session(1, &["oak", "pine", "birch"], None));
    assert!(memory.greeting(Day(1), None) == vec![Fact::Favourite("birch".to_string())]);
}

// The worked example, in the contract's order.

#[test]
fn worked_example_in_order() {
    let mut memory = Memory::new();
    assert!(memory.greeting(Day(100), None).is_empty());
    assert!(memory.record(&session(100, &["oak", "oak", "pine"], None)));
    assert!(memory.record(&session(101, &["oak", " "], None)));
    assert!(memory.record(&session(102, &["birch plywood"], Some("Sharpener box"))));
    assert!(memory.streak(Day(102)) == 3);
    assert!(memory.streak(Day(103)) == 3);
    assert!(memory.streak(Day(104)) == 0);
    assert!(memory.record(&session(110, &["pine"], Some("Sharpener box"))));
    assert!(memory.completed_projects() == 1);
    assert!(memory.material_count("oak") == 2);
    assert!(memory.material_count("pine") == 2);
    assert!(
        memory.favourites()
            == vec![
                "oak".to_string(),
                "pine".to_string(),
                "birch plywood".to_string()
            ]
    );
    assert!(memory.days_since_last(Day(119)) == Some(9));
    assert!(memory.days_since_last(Day(105)) == Some(0));
    assert!(
        memory.greeting(Day(119), Some("  The drawer fronts "))
            == vec![
                Fact::DaysSinceLast(9),
                Fact::CompletedProjects(1),
                Fact::Favourite("oak".to_string()),
                Fact::Waiting("The drawer fronts".to_string()),
            ]
    );
    assert!(
        memory.greeting(Day(111), None)
            == vec![
                Fact::DaysSinceLast(1),
                Fact::CompletedProjects(1),
                Fact::Favourite("oak".to_string()),
            ]
    );
    assert!(memory.add_override("fit.clearance") == 1);
    assert!(memory.add_override("fit.clearance") == 2);
    assert!(memory.add_override("fit.clearance") == 3);
    memory.set_overrides("stock.grain", 5);
    memory.set_overrides("tools.noise", 1);
    assert!(memory.demoted() == vec!["fit.clearance".to_string(), "stock.grain".to_string()]);
    memory.set_overrides("stock.grain", 0);
    assert!(memory.overrides("stock.grain") == 0);
    memory.set_opted_out(true);
    assert!(memory.greeting(Day(119), Some("x")).is_empty());
    assert!(memory.last_day().is_none());
    assert!(memory.add_override("a") == 0);
    assert!(!memory.record(&session(120, &["oak"], None)));
    memory.set_opted_out(false);
    assert!(memory.last_day().is_none());
    assert!(memory.favourites().is_empty());
}

// No call panics, whatever it is handed.

#[test]
fn no_call_panics_on_odd_input() {
    let mut memory = Memory::new();
    let empty = Session {
        day: Day(i64::MIN),
        materials: Vec::new(),
        completed: None,
    };
    assert!(memory.record(&empty));
    memory.record(&session(i64::MAX, &["", "  ", "\u{0}"], Some("")));
    let _ = memory.last_day();
    let _ = memory.days_since_last(Day(i64::MIN));
    let _ = memory.days_since_last(Day(i64::MAX));
    let _ = memory.streak(Day(i64::MIN));
    let _ = memory.streak(Day(i64::MAX));
    let _ = memory.completed_projects();
    let _ = memory.material_count("");
    let _ = memory.material_count("\u{0}");
    let _ = memory.favourites();
    let _ = memory.overrides("");
    let _ = memory.add_override("");
    memory.set_overrides("", 0);
    let _ = memory.demoted();
    let _ = memory.greeting(Day(i64::MIN), Some(""));
    let _ = memory.greeting(Day(i64::MAX), Some("\u{0}"));

    let mut opted = Memory::new();
    opted.set_opted_out(true);
    let _ = opted.greeting(Day(0), Some("x"));
    opted.set_opted_out(false);
    let _ = opted.streak(Day(i64::MIN));
}
