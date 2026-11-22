//! Tests for mk_assist::memory written by Kimi (the Crew, campaign mk-core-20) from the
//! contract alone, without seeing the module.

//! Integration tests for Sancho's relationship memory.

use mk_assist::memory::{Day, Fact, Memory, Session, DEMOTE_AFTER, FAVOURITES};

fn session(day: i64, materials: &[&str], completed: Option<&str>) -> Session {
    Session {
        day: Day(day),
        materials: materials.iter().map(|name| (*name).to_string()).collect(),
        completed: completed.map(str::to_string),
    }
}

fn names(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

#[test]
fn m1_opt_out_clears_everything_and_blocks_changes() {
    let mut memory = Memory::default();
    assert!(!memory.opted_out());
    assert!(memory.record(&session(10, &["oak"], Some("Box"))));
    memory.set_overrides("fit.clearance", DEMOTE_AFTER);

    memory.set_opted_out(true);
    assert!(memory.opted_out());
    assert!(!memory.record(&session(11, &["pine"], Some("Crate"))));
    assert_eq!(memory.add_override("tools.noise"), 0);
    memory.set_overrides("stock.grain", 9);

    assert_eq!(memory.last_day(), None);
    assert_eq!(memory.days_since_last(Day(20)), None);
    assert_eq!(memory.streak(Day(20)), 0);
    assert_eq!(memory.completed_projects(), 0);
    assert_eq!(memory.material_count("oak"), 0);
    assert_eq!(memory.material_count("pine"), 0);
    assert_eq!(memory.overrides("fit.clearance"), 0);
    assert_eq!(memory.overrides("tools.noise"), 0);
    assert_eq!(memory.overrides("stock.grain"), 0);
    assert_eq!(memory.favourites(), Vec::<String>::new());
    assert_eq!(memory.demoted(), Vec::<String>::new());
    assert_eq!(
        memory.greeting(Day(20), Some("waiting")),
        Vec::<Fact>::new()
    );
}

#[test]
fn m1_opting_back_in_starts_empty_and_accepts_new_history() {
    let mut memory = Memory::new();
    assert!(memory.record(&session(4, &["oak"], Some("Old project"))));
    memory.set_overrides("old.rule", 8);

    memory.set_opted_out(true);
    memory.set_opted_out(false);

    assert!(!memory.opted_out());
    assert_eq!(memory.last_day(), None);
    assert_eq!(memory.completed_projects(), 0);
    assert_eq!(memory.material_count("oak"), 0);
    assert_eq!(memory.overrides("old.rule"), 0);
    assert_eq!(memory.favourites(), Vec::<String>::new());
    assert_eq!(memory.demoted(), Vec::<String>::new());

    assert!(memory.record(&session(7, &["pine"], Some("New project"))));
    assert_eq!(memory.last_day(), Some(Day(7)));
    assert_eq!(memory.completed_projects(), 1);
    assert_eq!(memory.favourites(), names(&["pine"]));
    assert_eq!(memory.add_override("new.rule"), 1);
}

#[test]
fn m2_record_deduplicates_days_materials_and_completed_projects() {
    let mut memory = Memory::new();
    assert!(memory.record(&session(10, &["oak", "oak", " ", ""], Some("Box"))));
    assert!(memory.record(&session(10, &["pine"], Some("Box"))));
    assert!(memory.record(&session(11, &["oak"], Some("Box"))));

    assert_eq!(memory.last_day(), Some(Day(11)));
    assert_eq!(memory.material_count("oak"), 2);
    assert_eq!(memory.material_count("pine"), 1);
    assert_eq!(memory.material_count(" "), 0);
    assert_eq!(memory.material_count(""), 0);
    assert_eq!(memory.completed_projects(), 1);
}

#[test]
fn m2_record_keeps_exact_nonblank_names_and_skips_blank_completed_names() {
    let mut memory = Memory::new();
    assert!(memory.record(&session(3, &["oak", "Oak", " oak ", "\t"], Some(" \t "))));
    assert!(memory.record(&session(4, &["Oak"], None)));

    assert_eq!(memory.material_count("oak"), 1);
    assert_eq!(memory.material_count("Oak"), 2);
    assert_eq!(memory.material_count(" oak "), 1);
    assert_eq!(memory.material_count("\t"), 0);
    assert_eq!(memory.completed_projects(), 0);
}

#[test]
fn m3_last_day_uses_latest_session_recorded_out_of_day_order() {
    let mut memory = Memory::new();
    assert_eq!(memory.last_day(), None);
    assert!(memory.record(&session(42, &[], None)));
    assert!(memory.record(&session(7, &[], None)));
    assert!(memory.record(&session(100, &[], None)));
    assert!(memory.record(&session(99, &[], None)));
    assert!(memory.record(&session(7, &[], None)));

    assert_eq!(memory.last_day(), Some(Day(100)));
}

#[test]
fn m3_days_since_last_handles_none_past_and_saturation() {
    let mut memory = Memory::new();
    assert_eq!(memory.days_since_last(Day(100)), None);
    assert!(memory.record(&session(100, &[], None)));

    assert_eq!(memory.days_since_last(Day(99)), Some(0));
    assert_eq!(memory.days_since_last(Day(100)), Some(0));
    assert_eq!(memory.days_since_last(Day(109)), Some(9));

    let mut saturated = Memory::new();
    assert!(saturated.record(&session(i64::MIN, &[], None)));
    // The mathematical difference exceeds both i64 and u32, so it clamps to u32::MAX.
    assert_eq!(saturated.days_since_last(Day(i64::MAX)), Some(u32::MAX));
}

#[test]
fn m4_streak_stops_at_gaps_and_can_end_yesterday() {
    let mut memory = Memory::new();
    for day in [10, 11, 12, 15] {
        assert!(memory.record(&session(day, &[], None)));
    }

    assert_eq!(memory.streak(Day(12)), 3);
    assert_eq!(memory.streak(Day(13)), 3);
    assert_eq!(memory.streak(Day(14)), 0);
    assert_eq!(memory.streak(Day(15)), 1);
    assert_eq!(memory.streak(Day(16)), 1);
    assert_eq!(memory.streak(Day(17)), 0);
}

#[test]
fn m4_streak_handles_sessions_recorded_out_of_day_order() {
    let mut memory = Memory::new();
    for day in [3, 1, 7, 2, 6] {
        assert!(memory.record(&session(day, &[], None)));
    }

    assert_eq!(memory.streak(Day(3)), 3);
    assert_eq!(memory.streak(Day(4)), 3);
    assert_eq!(memory.streak(Day(5)), 0);
    assert_eq!(memory.streak(Day(6)), 1);
    assert_eq!(memory.streak(Day(7)), 2);
    assert_eq!(memory.streak(Day(8)), 2);
}

#[test]
fn m4_streak_handles_earliest_day_without_overflow() {
    let mut memory = Memory::new();
    assert_eq!(memory.streak(Day(i64::MIN)), 0);
    assert!(memory.record(&session(i64::MIN, &[], None)));
    assert_eq!(memory.streak(Day(i64::MIN)), 1);
}

#[test]
fn m5_completed_projects_counts_distinct_nonempty_names() {
    let mut memory = Memory::new();
    assert!(memory.record(&session(1, &[], Some("Box"))));
    assert!(memory.record(&session(2, &[], Some("Box"))));
    assert!(memory.record(&session(3, &[], Some("Crate"))));
    assert!(memory.record(&session(4, &[], Some(" "))));
    assert!(memory.record(&session(5, &[], None)));

    assert_eq!(memory.completed_projects(), 2);
}

#[test]
fn m5_material_count_returns_zero_for_unknown_names() {
    let mut memory = Memory::new();
    assert_eq!(memory.material_count("oak"), 0);
    assert!(memory.record(&session(1, &["oak", "oak"], None)));

    assert_eq!(memory.material_count("oak"), 1);
    assert_eq!(memory.material_count("Oak"), 0);
    assert_eq!(memory.material_count("pine"), 0);
}

#[test]
fn m5_favourites_sorts_by_count_then_name_and_truncates() {
    assert_eq!(FAVOURITES, 3);
    let mut memory = Memory::new();
    assert!(memory.record(&session(
        1,
        &["cedar", "cedar", "ash", "birch", "zinc"],
        None
    )));
    assert!(memory.record(&session(2, &["cedar", "ash", "zinc"], None)));
    assert!(memory.record(&session(3, &["cedar", "zinc"], None)));

    // Counts are cedar 3, zinc 3, ash 2, birch 1; cedar wins the tie by name.
    assert_eq!(memory.favourites(), names(&["cedar", "zinc", "ash"]));
}

#[test]
fn m6_overrides_add_set_and_remove() {
    let mut memory = Memory::new();
    assert_eq!(memory.overrides("fit.clearance"), 0);
    assert_eq!(memory.add_override("fit.clearance"), 1);
    assert_eq!(memory.add_override("fit.clearance"), 2);
    assert_eq!(memory.overrides("fit.clearance"), 2);

    memory.set_overrides("fit.clearance", 7);
    memory.set_overrides("stock.grain", 4);
    assert_eq!(memory.overrides("fit.clearance"), 7);
    assert_eq!(memory.overrides("stock.grain"), 4);

    memory.set_overrides("fit.clearance", 0);
    memory.set_overrides("unknown", 0);
    assert_eq!(memory.overrides("fit.clearance"), 0);
    assert_eq!(memory.overrides("unknown"), 0);
}

#[test]
fn m6_add_override_saturates_at_u32_max() {
    let mut memory = Memory::new();
    memory.set_overrides("fit.clearance", u32::MAX - 1);

    assert_eq!(memory.add_override("fit.clearance"), u32::MAX);
    assert_eq!(memory.add_override("fit.clearance"), u32::MAX);
    assert_eq!(memory.overrides("fit.clearance"), u32::MAX);
}

#[test]
fn m6_demoted_uses_threshold_and_name_order() {
    assert_eq!(DEMOTE_AFTER, 3);
    let mut memory = Memory::new();
    memory.set_overrides("z.rule", DEMOTE_AFTER);
    memory.set_overrides("a.rule", DEMOTE_AFTER + 1);
    memory.set_overrides("m.rule", DEMOTE_AFTER - 1);
    memory.set_overrides("b.rule", DEMOTE_AFTER);

    assert_eq!(memory.demoted(), names(&["a.rule", "b.rule", "z.rule"]));
    memory.set_overrides("a.rule", 0);
    assert_eq!(memory.demoted(), names(&["b.rule", "z.rule"]));
}

#[test]
fn m7_greeting_emits_facts_in_contract_order() {
    let mut memory = Memory::new();
    assert!(memory.record(&session(100, &["oak", "pine"], None)));
    assert!(memory.record(&session(101, &["oak"], None)));
    assert!(memory.record(&session(102, &["birch"], Some("Sharpener box"))));

    // Day 103 gives days-since-last 1 and a three-day streak ending on day 102.
    assert_eq!(
        memory.greeting(Day(103), Some("  The drawer fronts \t")),
        vec![
            Fact::DaysSinceLast(1),
            Fact::Streak(3),
            Fact::CompletedProjects(1),
            Fact::Favourite("oak".to_string()),
            Fact::Waiting("The drawer fronts".to_string()),
        ]
    );
}

#[test]
fn m7_greeting_omits_facts_with_nothing_to_say() {
    let mut memory = Memory::new();
    assert_eq!(memory.greeting(Day(10), Some(" \t ")), Vec::<Fact>::new());

    assert!(memory.record(&session(10, &[], None)));
    assert_eq!(memory.greeting(Day(10), None), Vec::<Fact>::new());
    assert_eq!(
        memory.greeting(Day(11), Some(" ")),
        vec![Fact::DaysSinceLast(1)]
    );
}

#[test]
fn worked_example_in_order_on_one_memory() {
    let mut memory = Memory::new();

    // Row 1.
    assert_eq!(memory.greeting(Day(100), None), Vec::<Fact>::new());

    // Rows 2-4.
    assert!(memory.record(&session(100, &["oak", "oak", "pine"], None)));
    assert!(memory.record(&session(101, &["oak", " "], None)));
    assert!(memory.record(&session(102, &["birch plywood"], Some("Sharpener box"))));

    // Row 5.
    assert_eq!(memory.streak(Day(102)), 3);
    assert_eq!(memory.streak(Day(103)), 3);
    assert_eq!(memory.streak(Day(104)), 0);

    // Row 6.
    assert!(memory.record(&session(110, &["pine"], Some("Sharpener box"))));

    // Rows 7-9.
    assert_eq!(memory.completed_projects(), 1);
    assert_eq!(memory.material_count("oak"), 2);
    assert_eq!(memory.material_count("pine"), 2);
    assert_eq!(
        memory.favourites(),
        names(&["oak", "pine", "birch plywood"])
    );
    assert_eq!(memory.days_since_last(Day(119)), Some(9));
    assert_eq!(memory.days_since_last(Day(105)), Some(0));

    // Row 10.
    assert_eq!(
        memory.greeting(Day(119), Some("  The drawer fronts ")),
        vec![
            Fact::DaysSinceLast(9),
            Fact::CompletedProjects(1),
            Fact::Favourite("oak".to_string()),
            Fact::Waiting("The drawer fronts".to_string()),
        ]
    );

    // Row 11: only day 110 belongs to this streak, so there is no Streak fact.
    assert_eq!(
        memory.greeting(Day(111), None),
        vec![
            Fact::DaysSinceLast(1),
            Fact::CompletedProjects(1),
            Fact::Favourite("oak".to_string()),
        ]
    );

    // Rows 12-14.
    assert_eq!(memory.add_override("fit.clearance"), 1);
    assert_eq!(memory.add_override("fit.clearance"), 2);
    assert_eq!(memory.add_override("fit.clearance"), 3);
    memory.set_overrides("stock.grain", 5);
    memory.set_overrides("tools.noise", 1);
    assert_eq!(memory.demoted(), names(&["fit.clearance", "stock.grain"]));
    memory.set_overrides("stock.grain", 0);
    assert_eq!(memory.overrides("stock.grain"), 0);

    // Rows 15-17.
    memory.set_opted_out(true);
    assert_eq!(memory.greeting(Day(119), Some("x")), Vec::<Fact>::new());
    assert_eq!(memory.last_day(), None);
    assert_eq!(memory.add_override("a"), 0);
    assert!(!memory.record(&session(120, &["oak"], None)));
    memory.set_opted_out(false);
    assert_eq!(memory.last_day(), None);
    assert_eq!(memory.favourites(), Vec::<String>::new());
}
