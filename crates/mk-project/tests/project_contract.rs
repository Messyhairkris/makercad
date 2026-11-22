//! Contract tests for mk_project::project, written by Gordon as a PA (the Crew) from
//! crew/contracts/mk_project.md. They were the hidden tests of campaign mk-core-9.

use mk_project::project::{
    is_real_date, Budget, BudgetLevel, Date, Deadline, FormatStatus, HashEntry, Intent, IntentEdit,
    Manifest, Milestone, ProjectError, Skill, Timestamp, Weekday, WorkWindow, FORMAT_VERSION,
};

fn date(year: u16, month: u8, day: u8) -> Date {
    Date { year, month, day }
}

fn dresser() -> Intent {
    Intent::new("Dresser", "A four-drawer dresser").expect("a valid intent")
}

fn entries(pairs: &[(&str, &str)]) -> Vec<HashEntry> {
    pairs
        .iter()
        .map(|(name, hash)| HashEntry {
            name: name.to_string(),
            hash: hash.to_string(),
        })
        .collect()
}

fn days() -> Vec<Weekday> {
    vec![
        Weekday::Mon,
        Weekday::Tue,
        Weekday::Wed,
        Weekday::Thu,
        Weekday::Fri,
    ]
}

fn window(days: Vec<Weekday>, start_minute: u16, end_minute: u16) -> WorkWindow {
    WorkWindow {
        days,
        start_minute,
        end_minute,
    }
}

fn milestone(label: &str, due: Option<Date>) -> Milestone {
    Milestone {
        label: label.to_string(),
        due,
    }
}

fn with_budget(cap_minor: u64) -> Intent {
    let mut intent = dresser();
    intent.budget = Some(Budget {
        cap_minor,
        currency: "EUR".to_string(),
        spend_only_if_needed: false,
    });
    intent
}

// ---- D1: real dates ----------------------------------------------------------

#[test]
fn d1_2024_february_29_is_real() {
    assert!(is_real_date(date(2024, 2, 29)));
}

#[test]
fn d1_2023_february_29_is_not_real() {
    assert!(!is_real_date(date(2023, 2, 29)));
}

#[test]
fn d1_1900_february_29_is_not_real() {
    assert!(!is_real_date(date(1900, 2, 29)));
}

#[test]
fn d1_2000_february_29_is_real() {
    assert!(is_real_date(date(2000, 2, 29)));
}

#[test]
fn d1_april_31_is_not_real() {
    assert!(!is_real_date(date(2026, 4, 31)));
}

#[test]
fn d1_month_13_is_not_real() {
    assert!(!is_real_date(date(2026, 13, 1)));
}

#[test]
fn d1_day_0_is_not_real() {
    assert!(!is_real_date(date(2026, 1, 0)));
}

#[test]
fn d1_every_month_has_its_length() {
    let lengths = [
        (1u8, 31u8),
        (2, 28),
        (3, 31),
        (4, 30),
        (5, 31),
        (6, 30),
        (7, 31),
        (8, 31),
        (9, 30),
        (10, 31),
        (11, 30),
        (12, 31),
    ];
    for (month, last) in lengths {
        assert!(is_real_date(date(2023, month, last)), "2023-{month}-{last}");
        assert!(
            !is_real_date(date(2023, month, last + 1)),
            "2023-{month}-{}",
            last + 1
        );
    }
}

#[test]
fn d1_february_length_depends_on_leap_year() {
    assert!(is_real_date(date(2024, 2, 28)));
    assert!(is_real_date(date(2024, 2, 29)));
    assert!(!is_real_date(date(2024, 2, 30)));
    assert!(!is_real_date(date(2100, 2, 29)));
}

#[test]
fn d1_year_zero_is_allowed() {
    assert!(is_real_date(date(0, 1, 1)));
    assert!(is_real_date(date(0, 2, 29)));
}

#[test]
fn d1_month_zero_is_not_real() {
    assert!(!is_real_date(date(2026, 0, 1)));
}

// ---- M: the manifest ---------------------------------------------------------

#[test]
fn m1_new_manifest_defaults() {
    let manifest = Manifest::new("0.1.0");
    assert_eq!(manifest.format_version, FORMAT_VERSION);
    assert_eq!(manifest.app_version, "0.1.0");
    assert!(manifest.environments.is_empty());
    assert!(manifest.hashes().is_empty());
}

#[test]
fn m2_format_status_current_older_newer() {
    let mut manifest = Manifest::new("0.1.0");
    assert_eq!(manifest.format_status(), FormatStatus::Current);
    manifest.format_version = 0;
    assert_eq!(manifest.format_status(), FormatStatus::Older(0));
    manifest.format_version = 2;
    assert_eq!(manifest.format_status(), FormatStatus::Newer(2));
}

#[test]
fn m3_newer_format_is_refused_before_blank_version_and_unknown_environment() {
    let mut manifest = Manifest::new("  ");
    manifest.format_version = 2;
    manifest.environments = vec!["nope".to_string()];
    assert_eq!(manifest.validate(&[]), Err(ProjectError::NewerFormat(2)));
}

#[test]
fn m3_blank_app_version_is_refused_before_unknown_environment() {
    let mut manifest = Manifest::new("  ");
    manifest.format_version = 0;
    manifest.environments = vec!["nope".to_string()];
    assert_eq!(manifest.validate(&[]), Err(ProjectError::EmptyAppVersion));
}

#[test]
fn m3_blank_app_version_is_refused() {
    assert_eq!(
        Manifest::new("  ").validate(&[]),
        Err(ProjectError::EmptyAppVersion)
    );
    assert_eq!(
        Manifest::new("\t\n").validate(&[]),
        Err(ProjectError::EmptyAppVersion)
    );
}

#[test]
fn m3_unknown_environment_is_refused() {
    let mut manifest = Manifest::new("0.1.0");
    manifest.environments = vec!["design".to_string(), "stock".to_string()];
    assert_eq!(
        manifest.validate(&["design"]),
        Err(ProjectError::UnknownEnvironment("stock".to_string()))
    );
    assert_eq!(manifest.validate(&["design", "stock"]), Ok(()));
}

#[test]
fn m3_environment_match_is_case_sensitive() {
    let mut manifest = Manifest::new("0.1.0");
    manifest.environments = vec!["Design".to_string()];
    assert_eq!(
        manifest.validate(&["design"]),
        Err(ProjectError::UnknownEnvironment("Design".to_string()))
    );
}

#[test]
fn m3_first_unknown_environment_in_order_is_reported() {
    let mut manifest = Manifest::new("0.1.0");
    manifest.environments = vec!["design".to_string(), "b".to_string(), "c".to_string()];
    assert_eq!(
        manifest.validate(&["design"]),
        Err(ProjectError::UnknownEnvironment("b".to_string()))
    );
}

#[test]
fn m4_valid_hash_entries_validate_ok() {
    let mut manifest = Manifest::new("0.1.0");
    manifest.set_hash("a.sc", "1").expect("valid entry");
    manifest.set_hash("b.sc", "2").expect("valid entry");
    assert_eq!(manifest.validate(&[]), Ok(()));
}

#[test]
fn m5_older_format_validates_ok() {
    let mut manifest = Manifest::new("0.1.0");
    manifest.format_version = 0;
    manifest.environments = vec!["design".to_string()];
    assert_eq!(manifest.validate(&["design"]), Ok(()));
}

// ---- H: the hash index -------------------------------------------------------

#[test]
fn h1_set_hash_sorts_and_replaces() {
    let mut manifest = Manifest::new("0.1.0");
    assert_eq!(manifest.set_hash("designs/b.sc", "aa"), Ok(()));
    assert_eq!(manifest.set_hash("designs/a.sc", "bb"), Ok(()));
    assert_eq!(manifest.set_hash("designs/b.sc", "cc"), Ok(()));
    let want = entries(&[("designs/a.sc", "bb"), ("designs/b.sc", "cc")]);
    assert_eq!(manifest.hashes(), want.as_slice());
}

#[test]
fn h1_refused_name_changes_nothing() {
    let mut manifest = Manifest::new("0.1.0");
    manifest.set_hash("media/p.jpg", "aa").expect("valid entry");
    let before = manifest.hashes().to_vec();
    assert_eq!(
        manifest.set_hash("../x", "aa"),
        Err(ProjectError::BadHashName("../x".to_string()))
    );
    assert_eq!(manifest.hashes(), before.as_slice());
}

#[test]
fn h1_refused_hash_changes_nothing() {
    let mut manifest = Manifest::new("0.1.0");
    let before = manifest.hashes().to_vec();
    assert_eq!(
        manifest.set_hash("media/p.jpg", " "),
        Err(ProjectError::EmptyHash("media/p.jpg".to_string()))
    );
    assert_eq!(manifest.hashes(), before.as_slice());
}

#[test]
fn h1_refused_replacement_keeps_the_old_hash() {
    let mut manifest = Manifest::new("0.1.0");
    manifest.set_hash("media/p.jpg", "aa").expect("valid entry");
    assert_eq!(
        manifest.set_hash("media/p.jpg", "  "),
        Err(ProjectError::EmptyHash("media/p.jpg".to_string()))
    );
    assert_eq!(manifest.hash_of("media/p.jpg"), Some("aa"));
}

#[test]
fn h1_name_is_checked_before_hash() {
    let mut manifest = Manifest::new("0.1.0");
    assert_eq!(
        manifest.set_hash("../x", "  "),
        Err(ProjectError::BadHashName("../x".to_string()))
    );
}

#[test]
fn h1_layout_refused_names_are_bad_hash_names() {
    let mut manifest = Manifest::new("0.1.0");
    let names = [
        "", "a\\b", "/x", "a:b", "a<b", "a>b", "a\"b", "a|b", "a?b", "a*b", "a//b", "a/./b",
        "a/../b", "a/b.", "a/b ", "CON", "NUL", "COM1",
    ];
    for name in names {
        assert_eq!(
            manifest.set_hash(name, "aa"),
            Err(ProjectError::BadHashName(name.to_string())),
            "{name}"
        );
    }
    assert!(manifest.hashes().is_empty());
}

#[test]
fn h2_hashes_stay_sorted_after_many_inserts() {
    let mut manifest = Manifest::new("0.1.0");
    manifest.set_hash("z.sc", "h2").expect("valid entry");
    manifest.set_hash("a.sc", "h3").expect("valid entry");
    manifest.set_hash("m.sc", "h5").expect("valid entry");
    manifest.set_hash("b.sc", "h1").expect("valid entry");
    manifest.set_hash("y.sc", "h6").expect("valid entry");
    manifest.set_hash("c.sc", "h4").expect("valid entry");
    let want = entries(&[
        ("a.sc", "h3"),
        ("b.sc", "h1"),
        ("c.sc", "h4"),
        ("m.sc", "h5"),
        ("y.sc", "h6"),
        ("z.sc", "h2"),
    ]);
    assert_eq!(manifest.hashes(), want.as_slice());
}

#[test]
fn h2_hashes_stay_sorted_after_many_replacements() {
    let mut manifest = Manifest::new("0.1.0");
    manifest.set_hash("c.sc", "1").expect("valid entry");
    manifest.set_hash("a.sc", "2").expect("valid entry");
    manifest.set_hash("b.sc", "3").expect("valid entry");
    manifest.set_hash("c.sc", "4").expect("valid entry");
    manifest.set_hash("a.sc", "5").expect("valid entry");
    let want = entries(&[("a.sc", "5"), ("b.sc", "3"), ("c.sc", "4")]);
    assert_eq!(manifest.hashes(), want.as_slice());
}

#[test]
fn h2_hashes_one_entry_per_name() {
    let mut manifest = Manifest::new("0.1.0");
    manifest.set_hash("a.sc", "1").expect("valid entry");
    manifest.set_hash("a.sc", "2").expect("valid entry");
    manifest.set_hash("a.sc", "3").expect("valid entry");
    let want = entries(&[("a.sc", "3")]);
    assert_eq!(manifest.hashes(), want.as_slice());
}

#[test]
fn h2_hash_of_returns_the_recorded_hash() {
    let mut manifest = Manifest::new("0.1.0");
    manifest.set_hash("a.sc", "1").expect("valid entry");
    assert_eq!(manifest.hash_of("a.sc"), Some("1"));
    manifest.set_hash("a.sc", "2").expect("valid entry");
    assert_eq!(manifest.hash_of("a.sc"), Some("2"));
}

#[test]
fn h2_hash_of_missing_name_is_none() {
    let manifest = Manifest::new("0.1.0");
    assert_eq!(manifest.hash_of("a.sc"), None);
}

// ---- I: the intent -----------------------------------------------------------

#[test]
fn i1_blank_name_is_reported_before_blank_goal() {
    assert_eq!(Intent::new(" ", ""), Err(ProjectError::EmptyName));
    assert_eq!(Intent::new("", ""), Err(ProjectError::EmptyName));
}

#[test]
fn i1_blank_goal_is_reported() {
    assert_eq!(Intent::new("Dresser", "  "), Err(ProjectError::EmptyGoal));
}

#[test]
fn i1_new_intent_defaults() {
    let intent = dresser();
    assert_eq!(
        intent,
        Intent {
            name: "Dresser".to_string(),
            goal: "A four-drawer dresser".to_string(),
            success: Vec::new(),
            constraints: Vec::new(),
            skill: Skill::Beginner,
            style_words: Vec::new(),
            deadline: None,
            budget: None,
            history: Vec::new(),
        }
    );
    assert_eq!(intent.validate(), Ok(()));
}

#[test]
fn i1_name_and_goal_are_kept_exactly() {
    let intent = Intent::new("  Dresser  ", "  a goal  ").expect("valid intent");
    assert_eq!(intent.name, "  Dresser  ");
    assert_eq!(intent.goal, "  a goal  ");
}

#[test]
fn i2_blank_name_is_reported_before_a_bad_due() {
    let mut intent = dresser();
    intent.name = "  ".to_string();
    intent.deadline = Some(Deadline {
        due: date(2026, 2, 30),
        windows: Vec::new(),
        milestones: Vec::new(),
    });
    assert_eq!(intent.validate(), Err(ProjectError::EmptyName));
}

#[test]
fn i2_blank_goal_is_reported_before_a_bad_due() {
    let mut intent = dresser();
    intent.goal = String::new();
    intent.deadline = Some(Deadline {
        due: date(2026, 2, 30),
        windows: Vec::new(),
        milestones: Vec::new(),
    });
    assert_eq!(intent.validate(), Err(ProjectError::EmptyGoal));
}

#[test]
fn i2_bad_due_is_reported() {
    let mut intent = dresser();
    intent.deadline = Some(Deadline {
        due: date(2026, 2, 30),
        windows: Vec::new(),
        milestones: Vec::new(),
    });
    assert_eq!(
        intent.validate(),
        Err(ProjectError::BadDate(date(2026, 2, 30)))
    );
}

#[test]
fn i2_bad_milestone_due_is_reported() {
    let mut intent = dresser();
    intent.deadline = Some(Deadline {
        due: date(2026, 12, 1),
        windows: Vec::new(),
        milestones: vec![
            milestone("frame", Some(date(2026, 6, 1))),
            milestone("drawers", Some(date(2026, 2, 30))),
        ],
    });
    assert_eq!(
        intent.validate(),
        Err(ProjectError::BadDate(date(2026, 2, 30)))
    );
}

#[test]
fn i2_milestones_are_checked_in_order() {
    let mut intent = dresser();
    intent.deadline = Some(Deadline {
        due: date(2026, 12, 1),
        windows: Vec::new(),
        milestones: vec![
            milestone("first", Some(date(2026, 2, 30))),
            milestone("second", Some(date(2026, 4, 31))),
        ],
    });
    assert_eq!(
        intent.validate(),
        Err(ProjectError::BadDate(date(2026, 2, 30)))
    );
}

#[test]
fn i2_milestone_without_a_date_is_skipped() {
    let mut intent = dresser();
    intent.deadline = Some(Deadline {
        due: date(2026, 12, 1),
        windows: Vec::new(),
        milestones: vec![milestone("someday", None)],
    });
    assert_eq!(intent.validate(), Ok(()));
}

#[test]
fn i2_bad_second_window_is_reported_by_index() {
    let mut intent = dresser();
    intent.deadline = Some(Deadline {
        due: date(2026, 12, 1),
        windows: vec![
            window(days(), 1140, 1260),
            window(vec![Weekday::Sat], 1260, 1200),
        ],
        milestones: Vec::new(),
    });
    assert_eq!(intent.validate(), Err(ProjectError::BadWorkWindow(1)));
}

#[test]
fn i2_window_with_no_days_is_bad() {
    let mut intent = dresser();
    intent.deadline = Some(Deadline {
        due: date(2026, 12, 1),
        windows: vec![window(Vec::new(), 1140, 1260)],
        milestones: Vec::new(),
    });
    assert_eq!(intent.validate(), Err(ProjectError::BadWorkWindow(0)));
}

#[test]
fn i2_window_zero_to_1440_is_valid() {
    let mut intent = dresser();
    intent.deadline = Some(Deadline {
        due: date(2026, 12, 1),
        windows: vec![window(days(), 0, 1440)],
        milestones: Vec::new(),
    });
    assert_eq!(intent.validate(), Ok(()));
}

#[test]
fn i2_window_ending_after_1440_is_bad() {
    let mut intent = dresser();
    intent.deadline = Some(Deadline {
        due: date(2026, 12, 1),
        windows: vec![window(days(), 0, 1441)],
        milestones: Vec::new(),
    });
    assert_eq!(intent.validate(), Err(ProjectError::BadWorkWindow(0)));
}

#[test]
fn i2_window_start_must_be_before_end() {
    let mut intent = dresser();
    intent.deadline = Some(Deadline {
        due: date(2026, 12, 1),
        windows: vec![window(days(), 1200, 1200), window(days(), 1200, 1199)],
        milestones: Vec::new(),
    });
    assert_eq!(intent.validate(), Err(ProjectError::BadWorkWindow(0)));
}

#[test]
fn i2_window_index_is_its_position() {
    let mut intent = dresser();
    intent.deadline = Some(Deadline {
        due: date(2026, 12, 1),
        windows: vec![
            window(days(), 0, 60),
            window(days(), 60, 120),
            window(Vec::new(), 0, 60),
        ],
        milestones: Vec::new(),
    });
    assert_eq!(intent.validate(), Err(ProjectError::BadWorkWindow(2)));
}

#[test]
fn i2_budget_cap_zero_is_bad() {
    let mut intent = dresser();
    intent.budget = Some(Budget {
        cap_minor: 0,
        currency: "EUR".to_string(),
        spend_only_if_needed: false,
    });
    assert_eq!(intent.validate(), Err(ProjectError::BadBudget));
}

#[test]
fn i2_blank_currency_is_bad() {
    let mut intent = dresser();
    intent.budget = Some(Budget {
        cap_minor: 1,
        currency: "  ".to_string(),
        spend_only_if_needed: false,
    });
    assert_eq!(intent.validate(), Err(ProjectError::EmptyCurrency));
}

#[test]
fn i2_valid_deadline_and_budget_validate_ok() {
    let mut intent = dresser();
    intent.deadline = Some(Deadline {
        due: date(2026, 12, 1),
        windows: vec![window(days(), 1140, 1260)],
        milestones: vec![milestone("frame", Some(date(2026, 6, 1)))],
    });
    intent.budget = Some(Budget {
        cap_minor: 10_000,
        currency: "EUR".to_string(),
        spend_only_if_needed: true,
    });
    assert_eq!(intent.validate(), Ok(()));
}

#[test]
fn i2_due_is_checked_before_milestones_and_windows() {
    let mut intent = dresser();
    intent.deadline = Some(Deadline {
        due: date(2026, 2, 30),
        windows: vec![window(Vec::new(), 0, 60)],
        milestones: vec![milestone("bad", Some(date(2026, 4, 31)))],
    });
    assert_eq!(
        intent.validate(),
        Err(ProjectError::BadDate(date(2026, 2, 30)))
    );
}

#[test]
fn i2_milestones_are_checked_before_windows() {
    let mut intent = dresser();
    intent.deadline = Some(Deadline {
        due: date(2026, 12, 1),
        windows: vec![window(Vec::new(), 0, 60)],
        milestones: vec![milestone("bad", Some(date(2026, 4, 31)))],
    });
    assert_eq!(
        intent.validate(),
        Err(ProjectError::BadDate(date(2026, 4, 31)))
    );
}

#[test]
fn i2_windows_are_checked_before_the_budget() {
    let mut intent = dresser();
    intent.deadline = Some(Deadline {
        due: date(2026, 12, 1),
        windows: vec![window(Vec::new(), 0, 60)],
        milestones: Vec::new(),
    });
    intent.budget = Some(Budget {
        cap_minor: 0,
        currency: "  ".to_string(),
        spend_only_if_needed: false,
    });
    assert_eq!(intent.validate(), Err(ProjectError::BadWorkWindow(0)));
}

#[test]
fn i3_a_past_deadline_is_valid() {
    let mut intent = dresser();
    intent.deadline = Some(Deadline {
        due: date(2020, 1, 1),
        windows: Vec::new(),
        milestones: Vec::new(),
    });
    assert_eq!(intent.validate(), Ok(()));
}

#[test]
fn i4_edit_goal_changes_the_goal_and_records_the_edit() {
    let mut intent = dresser();
    assert_eq!(
        intent.edit_goal(Timestamp(5), "A five-drawer dresser", "more socks"),
        Ok(())
    );
    assert_eq!(intent.goal, "A five-drawer dresser");
    assert_eq!(
        intent.history,
        vec![IntentEdit {
            at: Timestamp(5),
            text: "A five-drawer dresser".to_string(),
            why: "more socks".to_string(),
        }]
    );
}

#[test]
fn i4_blank_text_changes_nothing() {
    let mut intent = dresser();
    let before = intent.clone();
    assert_eq!(
        intent.edit_goal(Timestamp(6), "  ", "why"),
        Err(ProjectError::EmptyGoal)
    );
    assert_eq!(intent, before);
}

#[test]
fn i4_blank_why_changes_nothing() {
    let mut intent = dresser();
    let before = intent.clone();
    assert_eq!(
        intent.edit_goal(Timestamp(6), "x", " "),
        Err(ProjectError::EmptyWhy)
    );
    assert_eq!(intent, before);
}

#[test]
fn i4_history_keeps_edits_in_order() {
    let mut intent = dresser();
    intent
        .edit_goal(Timestamp(1), "first", "one")
        .expect("valid edit");
    intent
        .edit_goal(Timestamp(2), "second", "two")
        .expect("valid edit");
    assert_eq!(intent.goal, "second");
    assert_eq!(
        intent.history,
        vec![
            IntentEdit {
                at: Timestamp(1),
                text: "first".to_string(),
                why: "one".to_string()
            },
            IntentEdit {
                at: Timestamp(2),
                text: "second".to_string(),
                why: "two".to_string()
            },
        ]
    );
}

#[test]
fn i4_text_and_why_are_kept_exactly() {
    let mut intent = dresser();
    intent
        .edit_goal(Timestamp(7), "  x  ", "  because  ")
        .expect("valid edit");
    assert_eq!(intent.goal, "  x  ");
    assert_eq!(intent.history[0].text, "  x  ");
    assert_eq!(intent.history[0].why, "  because  ");
}

// ---- B1: the budget level ----------------------------------------------------

#[test]
fn b1_no_budget_is_none() {
    assert_eq!(dresser().budget_level(0), None);
}

#[test]
fn b1_budget_level_edges() {
    let intent = with_budget(10_000);
    assert_eq!(intent.budget_level(7_999), Some(BudgetLevel::Ok));
    assert_eq!(intent.budget_level(8_000), Some(BudgetLevel::Warn));
    assert_eq!(intent.budget_level(9_999), Some(BudgetLevel::Warn));
    assert_eq!(intent.budget_level(10_000), Some(BudgetLevel::AtCap));
    assert_eq!(intent.budget_level(12_000), Some(BudgetLevel::AtCap));
}

#[test]
fn b1_exact_eighty_percent_is_warn() {
    assert_eq!(with_budget(5).budget_level(4), Some(BudgetLevel::Warn));
    assert_eq!(with_budget(5).budget_level(3), Some(BudgetLevel::Ok));
}

#[test]
fn b1_near_u64_max_does_not_overflow() {
    let intent = with_budget(u64::MAX);
    assert_eq!(intent.budget_level(u64::MAX - 1), Some(BudgetLevel::Warn));
    assert_eq!(intent.budget_level(u64::MAX), Some(BudgetLevel::AtCap));
    assert_eq!(intent.budget_level(u64::MAX / 2), Some(BudgetLevel::Ok));
}

#[test]
fn b1_cap_one() {
    let intent = with_budget(1);
    assert_eq!(intent.budget_level(0), Some(BudgetLevel::Ok));
    assert_eq!(intent.budget_level(1), Some(BudgetLevel::AtCap));
}

// ---- odd text ----------------------------------------------------------------

#[test]
fn odd_text_emoji_never_panics() {
    let mut intent = Intent::new("🎨 Dresser", "Build a 🪵 dresser").expect("valid intent");
    assert_eq!(intent.validate(), Ok(()));
    intent
        .edit_goal(Timestamp(1), "🪑 with 🔩", "🔧")
        .expect("valid edit");
    assert_eq!(intent.goal, "🪑 with 🔩");
    assert_eq!(intent.history.len(), 1);
}

#[test]
fn odd_text_very_long_names_never_panic() {
    let name = "n".repeat(10_000);
    let goal = "g".repeat(100_000);
    let intent = Intent::new(&name, &goal).expect("valid intent");
    assert_eq!(intent.name, name);
    assert_eq!(intent.goal, goal);
    assert_eq!(intent.validate(), Ok(()));
    let long_hash_name = format!("{}.sc", "a".repeat(5_000));
    let mut manifest = Manifest::new("0.1.0");
    assert_eq!(manifest.set_hash(&long_hash_name, "aa"), Ok(()));
    assert_eq!(manifest.hash_of(&long_hash_name), Some("aa"));
}

#[test]
fn odd_text_emoji_hash_name_never_panics() {
    let mut manifest = Manifest::new("0.1.0");
    assert_eq!(manifest.set_hash("🎨/dresser.sc", "aa"), Ok(()));
    assert_eq!(manifest.hash_of("🎨/dresser.sc"), Some("aa"));
}
