//! Tests for mk_assist::voice written by Kimi (the Crew, campaign mk-core-25) from the
//! contract alone, without seeing the module.

//! Integration tests for Sancho's deterministic Voice greeting layer.

// Crew-written tests: a helper that builds a whole `Finding` takes one argument per
// field, and worked rows print constants to the contract's digits; neither matters in a
// test.
#![allow(clippy::too_many_arguments, clippy::approx_constant)]

use mk_assist::memory::Fact;
use mk_assist::voice::{days_phrase, greeting, plural, MAX_LINES};

fn lines(text: &[&str]) -> Vec<String> {
    text.iter().map(|line| (*line).to_string()).collect()
}

#[test]
fn p1_plural_worked_examples_h1_through_h4() {
    assert_eq!(plural(0, "day", "days"), "0 days");
    assert_eq!(plural(1, "day", "days"), "1 day");
    assert_eq!(plural(2, "day", "days"), "2 days");
    assert_eq!(plural(1, "project", "projects"), "1 project");
}

#[test]
fn d1_days_phrase_worked_examples_h5_through_h7() {
    assert_eq!(days_phrase(0), "today");
    assert_eq!(days_phrase(1), "yesterday");
    assert_eq!(days_phrase(9), "9 days");
}

#[test]
fn g1_empty_input_worked_example_1() {
    assert_eq!(greeting(&[]), Vec::<String>::new());
}

#[test]
fn g2_first_occurrence_wins_worked_example_18() {
    let facts = [Fact::Streak(2), Fact::Streak(5)];
    assert_eq!(greeting(&facts), lines(&["That's 2 days in a row."]));

    let facts = [
        Fact::Favourite("oak".to_string()),
        Fact::Favourite("walnut".to_string()),
    ];
    assert_eq!(greeting(&facts), lines(&["Still loving the oak."]));
}

#[test]
fn g2_first_present_occurrence_skips_absent_duplicates() {
    let facts = [Fact::Streak(0), Fact::Streak(5)];
    assert_eq!(greeting(&facts), lines(&["That's 5 days in a row."]));

    let facts = [
        Fact::Favourite(" \t\n".to_string()),
        Fact::Favourite("oak".to_string()),
    ];
    assert_eq!(greeting(&facts), lines(&["Still loving the oak."]));
}

#[test]
fn g3_zero_streak_and_completed_are_absent_worked_examples_8_and_11() {
    assert_eq!(greeting(&[Fact::Streak(0)]), Vec::<String>::new());
    assert_eq!(
        greeting(&[Fact::CompletedProjects(0)]),
        Vec::<String>::new()
    );
}

#[test]
fn g3_zero_does_not_block_fallback_worked_example_21() {
    let facts = [Fact::Streak(0), Fact::Favourite("oak".to_string())];
    assert_eq!(greeting(&facts), lines(&["Still loving the oak."]));

    let facts = [
        Fact::CompletedProjects(0),
        Fact::Favourite("walnut".to_string()),
    ];
    assert_eq!(greeting(&facts), lines(&["Still loving the walnut."]));
}

#[test]
fn g4_blank_waiting_is_absent_worked_example_20() {
    let facts = [
        Fact::Favourite("oak".to_string()),
        Fact::Waiting("   ".to_string()),
    ];
    assert_eq!(greeting(&facts), lines(&["Still loving the oak."]));
    assert_eq!(
        greeting(&[Fact::Favourite("\t\n ".to_string())]),
        Vec::<String>::new()
    );
}

#[test]
fn g4_surviving_text_is_used_exactly_without_trimming() {
    let facts = [Fact::Favourite("  Oak  ".to_string())];
    assert_eq!(greeting(&facts), lines(&["Still loving the   Oak  ."]));

    let facts = [Fact::Waiting("\tglue-up\n".to_string())];
    assert_eq!(greeting(&facts), lines(&["Waiting on \tglue-up\n."]));
}

#[test]
fn g5_days_lead_worked_examples_2_and_3() {
    assert_eq!(
        greeting(&[Fact::DaysSinceLast(9)]),
        lines(&["Welcome back - it's been 9 days."])
    );
    assert_eq!(
        greeting(&[Fact::DaysSinceLast(1)]),
        lines(&["Welcome back - it's been yesterday."])
    );
}

#[test]
fn g5_zero_days_is_present_worked_example_4() {
    assert_eq!(
        greeting(&[Fact::DaysSinceLast(0)]),
        lines(&["Welcome back."])
    );
}

#[test]
fn g6_streak_lead_worked_example_7() {
    assert_eq!(
        greeting(&[Fact::Streak(1)]),
        lines(&["That's 1 day in a row."])
    );
}

#[test]
fn g7_completed_lead_worked_examples_9_and_10() {
    assert_eq!(
        greeting(&[Fact::CompletedProjects(1)]),
        lines(&["You've finished 1 project."])
    );
    assert_eq!(
        greeting(&[Fact::CompletedProjects(3)]),
        lines(&["You've finished 3 projects."])
    );
}

#[test]
fn g8_favourite_lead_worked_example_12() {
    assert_eq!(
        greeting(&[Fact::Favourite("oak".to_string())]),
        lines(&["Still loving the oak."])
    );
}

#[test]
fn g9_no_lead_allows_only_waiting_worked_example_13() {
    assert_eq!(
        greeting(&[Fact::Waiting("the glue-up".to_string())]),
        lines(&["Waiting on the glue-up."])
    );
}

#[test]
fn g10_waiting_is_last_worked_examples_5_and_14() {
    let facts = [
        Fact::DaysSinceLast(9),
        Fact::Waiting("the drawer fronts".to_string()),
    ];
    assert_eq!(
        greeting(&facts),
        lines(&[
            "Welcome back - it's been 9 days.",
            "Waiting on the drawer fronts.",
        ])
    );

    let facts = [Fact::Streak(4), Fact::Waiting("the finish".to_string())];
    assert_eq!(
        greeting(&facts),
        lines(&["That's 4 days in a row.", "Waiting on the finish."])
    );
}

#[test]
fn g10_waiting_last_independent_of_input_order_worked_example_19() {
    let facts = [
        Fact::Waiting("the glue-up".to_string()),
        Fact::DaysSinceLast(1),
    ];
    assert_eq!(
        greeting(&facts),
        lines(&[
            "Welcome back - it's been yesterday.",
            "Waiting on the glue-up.",
        ])
    );
}

#[test]
fn g11_days_tail_priority_worked_examples_6_and_15() {
    let facts = [Fact::DaysSinceLast(9), Fact::Streak(3)];
    assert_eq!(
        greeting(&facts),
        lines(&[
            "Welcome back - it's been 9 days.",
            "That's 3 days in a row.",
        ])
    );

    let facts = [
        Fact::DaysSinceLast(2),
        Fact::CompletedProjects(3),
        Fact::Favourite("oak".to_string()),
    ];
    assert_eq!(
        greeting(&facts),
        lines(&[
            "Welcome back - it's been 2 days.",
            "You've finished 3 projects.",
        ])
    );
}

#[test]
fn g11_completed_then_favourite_tail_worked_example_16() {
    let facts = [
        Fact::CompletedProjects(2),
        Fact::Favourite("oak".to_string()),
    ];
    assert_eq!(
        greeting(&facts),
        lines(&["You've finished 2 projects.", "Still loving the oak."])
    );
}

#[test]
fn g11_streak_tail_priority_and_favourite_fallback() {
    let facts = [
        Fact::Streak(2),
        Fact::CompletedProjects(3),
        Fact::Favourite("oak".to_string()),
    ];
    assert_eq!(
        greeting(&facts),
        lines(&["That's 2 days in a row.", "You've finished 3 projects."])
    );

    let facts = [Fact::DaysSinceLast(2), Fact::Favourite("oak".to_string())];
    assert_eq!(
        greeting(&facts),
        lines(&["Welcome back - it's been 2 days.", "Still loving the oak."])
    );
}

#[test]
fn g12_all_five_facts_worked_example_17_and_max_lines() {
    assert_eq!(MAX_LINES, 2);
    let facts = [
        Fact::DaysSinceLast(9),
        Fact::Streak(3),
        Fact::CompletedProjects(2),
        Fact::Favourite("oak".to_string()),
        Fact::Waiting("the drawer fronts".to_string()),
    ];
    let got = greeting(&facts);
    assert_eq!(
        got,
        lines(&[
            "Welcome back - it's been 9 days.",
            "Waiting on the drawer fronts.",
        ])
    );
    assert!(got.len() <= MAX_LINES);
}

#[test]
fn g12_priority_is_independent_of_input_order() {
    let facts = [
        Fact::Waiting("the drawer fronts".to_string()),
        Fact::Favourite("oak".to_string()),
        Fact::CompletedProjects(2),
        Fact::Streak(3),
        Fact::DaysSinceLast(9),
    ];
    assert_eq!(
        greeting(&facts),
        lines(&[
            "Welcome back - it's been 9 days.",
            "Waiting on the drawer fronts.",
        ])
    );
}

#[test]
fn contract_never_panics_on_u32_max_and_returns_exact_text() {
    assert_eq!(plural(u32::MAX, "day", "days"), "4294967295 days");
    assert_eq!(days_phrase(u32::MAX), "4294967295 days");
    assert_eq!(
        greeting(&[Fact::DaysSinceLast(u32::MAX)]),
        lines(&["Welcome back - it's been 4294967295 days."])
    );
    assert_eq!(
        greeting(&[Fact::Streak(u32::MAX)]),
        lines(&["That's 4294967295 days in a row."])
    );
    assert_eq!(
        greeting(&[Fact::CompletedProjects(u32::MAX)]),
        lines(&["You've finished 4294967295 projects."])
    );
}
