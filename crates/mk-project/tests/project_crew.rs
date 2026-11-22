//! Tests for mk_project::project written by Deepak (the Crew, campaign mk-core-9) from the contract
//! alone, without seeing the module.

//! Integration tests for `mk_project::project`: the manifest and intent rules
//! (D1, M1-M5, H1-H2, I1-I4, B1), written from the contract alone.

use mk_project::project::{
    is_real_date, Budget, BudgetLevel, Date, Deadline, FormatStatus, HashEntry, Intent, IntentEdit,
    Manifest, Milestone, ProjectError, Skill, Timestamp, Weekday, WorkWindow, FORMAT_VERSION,
};

fn date(year: u16, month: u8, day: u8) -> Date {
    Date { year, month, day }
}

fn entry(name: &str, hash: &str) -> HashEntry {
    HashEntry {
        name: name.to_string(),
        hash: hash.to_string(),
    }
}

fn window(days: Vec<Weekday>, start: u16, end: u16) -> WorkWindow {
    WorkWindow {
        days,
        start_minute: start,
        end_minute: end,
    }
}

fn milestone(label: &str, due: Option<Date>) -> Milestone {
    Milestone {
        label: label.to_string(),
        due,
    }
}

fn deadline(due: Date, windows: Vec<WorkWindow>, milestones: Vec<Milestone>) -> Deadline {
    Deadline {
        due,
        windows,
        milestones,
    }
}

fn budget(cap: u64, currency: &str, only_if_needed: bool) -> Budget {
    Budget {
        cap_minor: cap,
        currency: currency.to_string(),
        spend_only_if_needed: only_if_needed,
    }
}

fn new_intent() -> Intent {
    Intent::new("Dresser", "A four-drawer dresser").unwrap()
}

// ---------------------------------------------------------------- dates (D1)

#[test]
fn rule_d1_every_month_length() {
    let months: [(u8, u8); 12] = [
        (1, 31),
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
    for (month, last) in months {
        assert!(
            is_real_date(date(2023, month, 1)),
            "2023-{month}-1 should be real"
        );
        assert!(
            is_real_date(date(2023, month, last)),
            "2023-{month}-{last} should be real"
        );
        assert!(
            !is_real_date(date(2023, month, last + 1)),
            "2023-{month}-{} should not be real",
            last + 1
        );
    }
}

#[test]
fn rule_d1_leap_years_1900_2000_2023_2024() {
    // 1900 is divisible by 100 but not by 400: not a leap year.
    assert!(!is_real_date(date(1900, 2, 29)));
    assert!(is_real_date(date(1900, 2, 28)));
    // 2000 is divisible by 400: a leap year.
    assert!(is_real_date(date(2000, 2, 29)));
    assert!(!is_real_date(date(2000, 2, 30)));
    // 2023 is not divisible by 4.
    assert!(!is_real_date(date(2023, 2, 29)));
    assert!(is_real_date(date(2023, 2, 28)));
    // 2024 is divisible by 4.
    assert!(is_real_date(date(2024, 2, 29)));
    assert!(!is_real_date(date(2024, 2, 30)));
    // 2100 is divisible by 100 but not by 400.
    assert!(!is_real_date(date(2100, 2, 29)));
}

#[test]
fn rule_d1_worked_examples() {
    assert!(is_real_date(Date {
        year: 2024,
        month: 2,
        day: 29
    }));
    assert!(!is_real_date(Date {
        year: 2023,
        month: 2,
        day: 29
    }));
    assert!(!is_real_date(Date {
        year: 1900,
        month: 2,
        day: 29
    }));
    assert!(is_real_date(Date {
        year: 2000,
        month: 2,
        day: 29
    }));
    assert!(!is_real_date(Date {
        year: 2026,
        month: 4,
        day: 31
    }));
    assert!(!is_real_date(Date {
        year: 2026,
        month: 13,
        day: 1
    }));
    assert!(!is_real_date(Date {
        year: 2026,
        month: 1,
        day: 0
    }));
}

#[test]
fn rule_d1_any_year_is_allowed_including_zero() {
    assert!(is_real_date(date(0, 1, 1)));
    // 0 is divisible by 400, so year 0 is a leap year.
    assert!(is_real_date(date(0, 2, 29)));
    assert!(!is_real_date(date(0, 2, 30)));
    assert!(is_real_date(date(u16::MAX, 12, 31)));
    assert!(!is_real_date(date(u16::MAX, 12, 32)));
    assert!(!is_real_date(date(2026, 0, 1)));
    assert!(!is_real_date(date(2026, 12, 0)));
}

// ------------------------------------------------------------- manifest (M1-M5)

#[test]
fn rule_m1_new_manifest_defaults() {
    let m = Manifest::new("0.1.0");
    assert_eq!(m.format_version, FORMAT_VERSION);
    assert_eq!(m.format_version, 1);
    assert_eq!(m.app_version, "0.1.0");
    assert_eq!(m.environments, Vec::<String>::new());
    assert_eq!(m.hashes().to_vec(), Vec::<HashEntry>::new());
}

#[test]
fn rule_m2_format_status_current_older_newer() {
    let mut m = Manifest::new("0.1.0");
    assert_eq!(m.format_status(), FormatStatus::Current);
    m.format_version = 0;
    assert_eq!(m.format_status(), FormatStatus::Older(0));
    m.format_version = 2;
    assert_eq!(m.format_status(), FormatStatus::Newer(2));
    m.format_version = 99;
    assert_eq!(m.format_status(), FormatStatus::Newer(99));
}

#[test]
fn rule_m3_newer_format_is_refused() {
    let mut m = Manifest::new("0.1.0");
    m.format_version = 2;
    assert_eq!(m.validate(&[]), Err(ProjectError::NewerFormat(2)));
    m.format_version = 7;
    assert_eq!(m.validate(&["design"]), Err(ProjectError::NewerFormat(7)));
}

#[test]
fn rule_m3_blank_app_version_is_refused() {
    assert_eq!(
        Manifest::new("  ").validate(&[]),
        Err(ProjectError::EmptyAppVersion)
    );
    assert_eq!(
        Manifest::new("").validate(&[]),
        Err(ProjectError::EmptyAppVersion)
    );
    assert_eq!(
        Manifest::new("\t\n").validate(&[]),
        Err(ProjectError::EmptyAppVersion)
    );
    assert_eq!(Manifest::new("0.1.0").validate(&[]), Ok(()));
}

#[test]
fn rule_m3_unknown_environment_is_refused() {
    let mut m = Manifest::new("0.1.0");
    m.environments = vec!["design".to_string(), "stock".to_string()];
    assert_eq!(
        m.validate(&["design"]),
        Err(ProjectError::UnknownEnvironment("stock".to_string()))
    );
    // exact and case-sensitive: "Design" is not "design".
    assert_eq!(
        m.validate(&["Design", "stock"]),
        Err(ProjectError::UnknownEnvironment("design".to_string()))
    );
    assert_eq!(m.validate(&["design", "stock"]), Ok(()));
    // the first unknown environment in order is the one reported.
    m.environments = vec!["a".to_string(), "b".to_string()];
    assert_eq!(
        m.validate(&[]),
        Err(ProjectError::UnknownEnvironment("a".to_string()))
    );
}

#[test]
fn rule_m3_check_order_newer_format_before_blank_app_version() {
    let mut m = Manifest::new("   ");
    m.format_version = 3;
    m.environments = vec!["nope".to_string()];
    assert_eq!(m.validate(&[]), Err(ProjectError::NewerFormat(3)));
}

#[test]
fn rule_m3_check_order_blank_app_version_before_unknown_environment() {
    let mut m = Manifest::new("");
    m.environments = vec!["nope".to_string()];
    assert_eq!(m.validate(&[]), Err(ProjectError::EmptyAppVersion));
}

#[test]
fn rule_m4_set_hash_refuses_bad_name() {
    let mut m = Manifest::new("0.1.0");
    assert_eq!(
        m.set_hash("../x", "aa"),
        Err(ProjectError::BadHashName("../x".to_string()))
    );
    assert_eq!(
        m.set_hash("", "aa"),
        Err(ProjectError::BadHashName("".to_string()))
    );
    assert_eq!(
        m.set_hash("designs/../a.sc", "aa"),
        Err(ProjectError::BadHashName("designs/../a.sc".to_string()))
    );
    // the name is checked before the hash: both bad reports the name.
    assert_eq!(
        m.set_hash("", " "),
        Err(ProjectError::BadHashName("".to_string()))
    );
    assert_eq!(m.set_hash("designs/a.sc", "aa"), Ok(()));
    assert_eq!(m.hashes().to_vec(), vec![entry("designs/a.sc", "aa")]);
}

#[test]
fn rule_m4_set_hash_refuses_blank_hash() {
    let mut m = Manifest::new("0.1.0");
    assert_eq!(
        m.set_hash("media/p.jpg", " "),
        Err(ProjectError::EmptyHash("media/p.jpg".to_string()))
    );
    assert_eq!(
        m.set_hash("media/p.jpg", ""),
        Err(ProjectError::EmptyHash("media/p.jpg".to_string()))
    );
    assert_eq!(m.hashes().to_vec(), Vec::<HashEntry>::new());
}

#[test]
fn rule_m5_older_format_is_valid() {
    let mut m = Manifest::new("0.1.0");
    m.format_version = 0;
    m.environments = vec!["design".to_string()];
    m.set_hash("designs/a.sc", "aa").unwrap();
    assert_eq!(m.validate(&["design"]), Ok(()));
    assert_eq!(m.format_status(), FormatStatus::Older(0));
}

// ----------------------------------------------------------- hash index (H1-H2)

#[test]
fn rule_h1_set_hash_sorted_after_many_inserts_and_a_replacement() {
    let mut m = Manifest::new("0.1.0");
    for (name, hash) in [
        ("designs/m.sc", "05"),
        ("designs/b.sc", "02"),
        ("designs/z.sc", "06"),
        ("designs/a.sc", "01"),
        ("designs/y.sc", "05b"),
        ("designs/c.sc", "03"),
    ] {
        assert_eq!(m.set_hash(name, hash), Ok(()));
    }
    // a replacement keeps one entry per name and leaves the order sorted.
    assert_eq!(m.set_hash("designs/b.sc", "02b"), Ok(()));
    assert_eq!(
        m.hashes().to_vec(),
        vec![
            entry("designs/a.sc", "01"),
            entry("designs/b.sc", "02b"),
            entry("designs/c.sc", "03"),
            entry("designs/m.sc", "05"),
            entry("designs/y.sc", "05b"),
            entry("designs/z.sc", "06"),
        ]
    );
}

#[test]
fn rule_h2_hashes_are_sorted_by_byte_order() {
    let mut m = Manifest::new("0.1.0");
    m.set_hash("designs/a.sc", "01").unwrap();
    m.set_hash("designs/B.sc", "02").unwrap();
    m.set_hash("designs/_.sc", "03").unwrap();
    // byte order: 'B' (0x42) < '_' (0x5F) < 'a' (0x61).
    assert_eq!(
        m.hashes().to_vec(),
        vec![
            entry("designs/B.sc", "02"),
            entry("designs/_.sc", "03"),
            entry("designs/a.sc", "01"),
        ]
    );
}

#[test]
fn rule_h1_refused_set_hash_changes_nothing() {
    let mut m = Manifest::new("0.1.0");
    m.set_hash("designs/a.sc", "01").unwrap();
    let before = m.hashes().to_vec();
    assert_eq!(
        m.set_hash("designs/../a.sc", "99"),
        Err(ProjectError::BadHashName("designs/../a.sc".to_string()))
    );
    assert_eq!(m.hashes().to_vec(), before);
    assert_eq!(
        m.set_hash("designs/a.sc", "  "),
        Err(ProjectError::EmptyHash("designs/a.sc".to_string()))
    );
    assert_eq!(m.hashes().to_vec(), before);
    assert_eq!(m.hash_of("designs/a.sc"), Some("01"));
}

#[test]
fn rule_h2_hash_of_returns_the_recorded_hash() {
    let mut m = Manifest::new("0.1.0");
    m.set_hash("designs/a.sc", "aa").unwrap();
    m.set_hash("designs/b.sc", "bb").unwrap();
    assert_eq!(m.hash_of("designs/a.sc"), Some("aa"));
    assert_eq!(m.hash_of("designs/b.sc"), Some("bb"));
    assert_eq!(m.hash_of("designs/c.sc"), None);
    m.set_hash("designs/a.sc", "cc").unwrap();
    assert_eq!(m.hash_of("designs/a.sc"), Some("cc"));
}

// -------------------------------------------------------------- intent (I1-I4)

#[test]
fn rule_i1_new_intent_defaults() {
    let i = Intent::new("Dresser", "A four-drawer dresser").unwrap();
    assert_eq!(i.name, "Dresser");
    assert_eq!(i.goal, "A four-drawer dresser");
    assert_eq!(i.success, Vec::<String>::new());
    assert_eq!(i.constraints, Vec::<String>::new());
    assert_eq!(i.skill, Skill::Beginner);
    assert_eq!(i.style_words, Vec::<String>::new());
    assert_eq!(i.deadline, None);
    assert_eq!(i.budget, None);
    assert_eq!(i.history, Vec::<IntentEdit>::new());
    // name and goal are kept exactly as given, not trimmed.
    let j = Intent::new("  Dresser  ", "  a goal  ").unwrap();
    assert_eq!(j.name, "  Dresser  ");
    assert_eq!(j.goal, "  a goal  ");
}

#[test]
fn rule_i1_blank_name_before_blank_goal() {
    assert_eq!(Intent::new(" ", ""), Err(ProjectError::EmptyName));
    assert_eq!(Intent::new("", "  "), Err(ProjectError::EmptyName));
    assert_eq!(Intent::new("\t\n", "goal"), Err(ProjectError::EmptyName));
}

#[test]
fn rule_i1_blank_goal_is_refused() {
    assert_eq!(Intent::new("Dresser", "  "), Err(ProjectError::EmptyGoal));
    assert_eq!(Intent::new("Dresser", ""), Err(ProjectError::EmptyGoal));
    assert_eq!(Intent::new("Dresser", "\t"), Err(ProjectError::EmptyGoal));
}

#[test]
fn rule_i2_bad_due_date_is_refused() {
    let mut i = new_intent();
    i.deadline = Some(deadline(date(2026, 2, 30), vec![], vec![]));
    assert_eq!(i.validate(), Err(ProjectError::BadDate(date(2026, 2, 30))));
    i.deadline = Some(deadline(date(2026, 2, 28), vec![], vec![]));
    assert_eq!(i.validate(), Ok(()));
}

#[test]
fn rule_i2_bad_milestone_date_is_refused() {
    let mut i = new_intent();
    i.deadline = Some(deadline(
        date(2026, 12, 31),
        vec![],
        vec![
            milestone("frame", Some(date(2026, 6, 31))),
            milestone("drawers", Some(date(2026, 4, 31))),
        ],
    ));
    // June has 30 days, so the first milestone is the first bad date.
    assert_eq!(i.validate(), Err(ProjectError::BadDate(date(2026, 6, 31))));
    // a milestone with no due date is fine.
    i.deadline = Some(deadline(
        date(2026, 12, 31),
        vec![],
        vec![
            milestone("frame", None),
            milestone("drawers", Some(date(2026, 4, 30))),
        ],
    ));
    assert_eq!(i.validate(), Ok(()));
}

#[test]
fn rule_i2_bad_work_window_is_refused() {
    let due = date(2026, 12, 31);
    let mut i = new_intent();
    // no days.
    i.deadline = Some(deadline(due, vec![window(vec![], 1140, 1260)], vec![]));
    assert_eq!(i.validate(), Err(ProjectError::BadWorkWindow(0)));
    // start == end.
    i.deadline = Some(deadline(
        due,
        vec![window(vec![Weekday::Mon], 600, 600)],
        vec![],
    ));
    assert_eq!(i.validate(), Err(ProjectError::BadWorkWindow(0)));
    // start > end.
    i.deadline = Some(deadline(
        due,
        vec![window(vec![Weekday::Sat], 1260, 1200)],
        vec![],
    ));
    assert_eq!(i.validate(), Err(ProjectError::BadWorkWindow(0)));
    // end past midnight.
    i.deadline = Some(deadline(
        due,
        vec![window(vec![Weekday::Mon], 0, 1441)],
        vec![],
    ));
    assert_eq!(i.validate(), Err(ProjectError::BadWorkWindow(0)));
    // the index is the window's position in the list.
    let weekdays = vec![
        Weekday::Mon,
        Weekday::Tue,
        Weekday::Wed,
        Weekday::Thu,
        Weekday::Fri,
    ];
    i.deadline = Some(deadline(
        due,
        vec![
            window(weekdays, 1140, 1260),
            window(vec![Weekday::Sat], 1260, 1200),
        ],
        vec![],
    ));
    assert_eq!(i.validate(), Err(ProjectError::BadWorkWindow(1)));
}

#[test]
fn rule_i2_window_ending_exactly_at_1440_is_valid() {
    let mut i = new_intent();
    i.deadline = Some(deadline(
        date(2026, 12, 31),
        vec![
            window(vec![Weekday::Mon], 0, 1440),
            window(vec![Weekday::Sun], 1439, 1440),
        ],
        vec![],
    ));
    assert_eq!(i.validate(), Ok(()));
}

#[test]
fn rule_i2_bad_budget_is_refused() {
    let mut i = new_intent();
    i.budget = Some(budget(0, "GBP", true));
    assert_eq!(i.validate(), Err(ProjectError::BadBudget));
    i.budget = Some(budget(1, "GBP", true));
    assert_eq!(i.validate(), Ok(()));
}

#[test]
fn rule_i2_empty_currency_is_refused() {
    let mut i = new_intent();
    i.budget = Some(budget(10_000, "  ", false));
    assert_eq!(i.validate(), Err(ProjectError::EmptyCurrency));
    i.budget = Some(budget(10_000, "", false));
    assert_eq!(i.validate(), Err(ProjectError::EmptyCurrency));
    i.budget = Some(budget(10_000, "GBP", false));
    assert_eq!(i.validate(), Ok(()));
}

#[test]
fn rule_i2_check_order() {
    let bad_due = date(2026, 2, 30);
    let good_due = date(2026, 12, 31);
    let bad_window = window(vec![], 0, 60);
    let good_window = window(vec![Weekday::Mon], 600, 660);

    // a blank name beats a bad due date.
    let mut i = new_intent();
    i.name = "  ".to_string();
    i.deadline = Some(deadline(bad_due, vec![], vec![]));
    assert_eq!(i.validate(), Err(ProjectError::EmptyName));

    // a blank goal beats a bad due date.
    let mut i = new_intent();
    i.goal = "".to_string();
    i.deadline = Some(deadline(bad_due, vec![], vec![]));
    assert_eq!(i.validate(), Err(ProjectError::EmptyGoal));

    // the due date beats a bad window.
    let mut i = new_intent();
    i.deadline = Some(deadline(bad_due, vec![bad_window.clone()], vec![]));
    assert_eq!(i.validate(), Err(ProjectError::BadDate(bad_due)));

    // a milestone date beats a bad window.
    let mut i = new_intent();
    i.deadline = Some(deadline(
        good_due,
        vec![bad_window.clone()],
        vec![milestone("frame", Some(bad_due))],
    ));
    assert_eq!(i.validate(), Err(ProjectError::BadDate(bad_due)));

    // a bad window beats a bad budget.
    let mut i = new_intent();
    i.deadline = Some(deadline(good_due, vec![good_window, bad_window], vec![]));
    i.budget = Some(budget(0, "GBP", true));
    assert_eq!(i.validate(), Err(ProjectError::BadWorkWindow(1)));

    // a zero cap beats a blank currency.
    let mut i = new_intent();
    i.budget = Some(budget(0, "  ", false));
    assert_eq!(i.validate(), Err(ProjectError::BadBudget));
}

#[test]
fn rule_i3_past_deadline_is_valid() {
    let mut i = new_intent();
    i.deadline = Some(deadline(
        date(2000, 1, 1),
        vec![window(vec![Weekday::Mon], 600, 660)],
        vec![milestone("frame", Some(date(1999, 12, 1)))],
    ));
    assert_eq!(i.validate(), Ok(()));
}

#[test]
fn rule_i4_edit_goal_changes_goal_and_records_history() {
    let mut i = new_intent();
    assert_eq!(
        i.edit_goal(Timestamp(5), "A five-drawer dresser", "more socks"),
        Ok(())
    );
    assert_eq!(i.goal, "A five-drawer dresser");
    assert_eq!(
        i.history,
        vec![IntentEdit {
            at: Timestamp(5),
            text: "A five-drawer dresser".to_string(),
            why: "more socks".to_string(),
        }]
    );
    // a second edit appends and keeps the first.
    assert_eq!(
        i.edit_goal(Timestamp(9), "A six-drawer dresser", "even more socks"),
        Ok(())
    );
    assert_eq!(i.goal, "A six-drawer dresser");
    assert_eq!(
        i.history,
        vec![
            IntentEdit {
                at: Timestamp(5),
                text: "A five-drawer dresser".to_string(),
                why: "more socks".to_string(),
            },
            IntentEdit {
                at: Timestamp(9),
                text: "A six-drawer dresser".to_string(),
                why: "even more socks".to_string(),
            },
        ]
    );
    // text is kept exactly as given, not trimmed.
    assert_eq!(i.edit_goal(Timestamp(10), "  padded  ", "why"), Ok(()));
    assert_eq!(i.goal, "  padded  ");
}

#[test]
fn rule_i4_refused_edit_changes_nothing() {
    let mut i = new_intent();
    i.edit_goal(Timestamp(5), "A five-drawer dresser", "more socks")
        .unwrap();
    let before = i.clone();
    assert_eq!(
        i.edit_goal(Timestamp(6), "  ", "why"),
        Err(ProjectError::EmptyGoal)
    );
    assert_eq!(i, before);
    assert_eq!(
        i.edit_goal(Timestamp(6), "x", " "),
        Err(ProjectError::EmptyWhy)
    );
    assert_eq!(i, before);
    // the text is checked before the reason.
    assert_eq!(
        i.edit_goal(Timestamp(6), "", ""),
        Err(ProjectError::EmptyGoal)
    );
    assert_eq!(i, before);
}

// --------------------------------------------------------------- budget (B1)

#[test]
fn rule_b1_budget_level_boundaries() {
    let mut i = new_intent();
    i.budget = Some(budget(10_000, "GBP", true));
    assert_eq!(i.budget_level(0), Some(BudgetLevel::Ok));
    assert_eq!(i.budget_level(7_999), Some(BudgetLevel::Ok)); // 79.99%
    assert_eq!(i.budget_level(8_000), Some(BudgetLevel::Warn)); // exactly 80%
    assert_eq!(i.budget_level(9_999), Some(BudgetLevel::Warn));
    assert_eq!(i.budget_level(10_000), Some(BudgetLevel::AtCap));
    assert_eq!(i.budget_level(12_000), Some(BudgetLevel::AtCap));
    // 80% of 5 is 4 exactly.
    i.budget = Some(budget(5, "GBP", false));
    assert_eq!(i.budget_level(3), Some(BudgetLevel::Ok));
    assert_eq!(i.budget_level(4), Some(BudgetLevel::Warn));
    assert_eq!(i.budget_level(5), Some(BudgetLevel::AtCap));
}

#[test]
fn rule_b1_budget_level_near_u64_max() {
    let mut i = new_intent();
    i.budget = Some(budget(u64::MAX, "GBP", false));
    assert_eq!(i.budget_level(u64::MAX - 1), Some(BudgetLevel::Warn));
    assert_eq!(i.budget_level(u64::MAX), Some(BudgetLevel::AtCap));
    assert_eq!(i.budget_level(0), Some(BudgetLevel::Ok));
    // a cap of 1: only 0 is below 80%.
    i.budget = Some(budget(1, "GBP", false));
    assert_eq!(i.budget_level(0), Some(BudgetLevel::Ok));
    assert_eq!(i.budget_level(1), Some(BudgetLevel::AtCap));
}

#[test]
fn rule_b1_no_budget_is_none() {
    let i = new_intent();
    assert_eq!(i.budget, None);
    assert_eq!(i.budget_level(0), None);
    assert_eq!(i.budget_level(u64::MAX), None);
}

// ------------------------------------------------------------- odd text

#[test]
fn odd_text_never_panics() {
    let long = "a".repeat(10_000);
    let i = Intent::new("dresser (oak)", "a four-drawer dresser, 60cm wide").unwrap();
    assert_eq!(i.name, "dresser (oak)");
    assert_eq!(i.goal, "a four-drawer dresser, 60cm wide");
    assert_eq!(i.validate(), Ok(()));
    let mut m = Manifest::new("0.1.0");
    let name = format!("designs/{long}.sc");
    assert_eq!(m.set_hash(&name, "aa"), Ok(()));
    assert_eq!(m.hash_of(&name), Some("aa"));
    // a control character in the name is refused, not a panic.
    assert_eq!(
        m.set_hash("designs/a\u{1}.sc", "aa"),
        Err(ProjectError::BadHashName("designs/a\u{1}.sc".to_string()))
    );
}
