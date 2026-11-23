//! Contract tests for mk_assist::voice, written by Gordon as a PA (the Crew) from
//! crew/contracts/mk_voice.md. They were the hidden tests of campaign mk-core-25.

// Crew-written tests: a helper that builds a whole `Finding` takes one argument per
// field, and worked rows print constants to the contract's digits; neither matters in a
// test.
#![allow(clippy::too_many_arguments, clippy::approx_constant)]

use mk_assist::memory::Fact;
use mk_assist::voice::{days_phrase, greeting, plural, MAX_LINES};

fn s(items: &[&str]) -> Vec<String> {
    items.iter().map(|item| item.to_string()).collect()
}

// P1: plural(n, one, many).

#[test]
fn p1_plural_one_is_singular() {
    assert_eq!(plural(1, "day", "days"), "1 day"); // H2
}

#[test]
fn p1_plural_zero_is_plural() {
    assert_eq!(plural(0, "day", "days"), "0 days"); // H1
}

#[test]
fn p1_plural_two_is_plural() {
    assert_eq!(plural(2, "day", "days"), "2 days"); // H3
}

#[test]
fn p1_plural_project_one() {
    assert_eq!(plural(1, "project", "projects"), "1 project"); // H4
}

// D1: days_phrase(days).

#[test]
fn d1_days_phrase_zero_is_today() {
    assert_eq!(days_phrase(0), "today"); // H5
}

#[test]
fn d1_days_phrase_one_is_yesterday() {
    assert_eq!(days_phrase(1), "yesterday"); // H6
}

#[test]
fn d1_days_phrase_many() {
    assert_eq!(days_phrase(9), "9 days"); // H7
}

// G1: no facts, no lines.

#[test]
fn g1_empty_facts_give_no_lines() {
    let none: [Fact; 0] = [];
    assert_eq!(greeting(&none), s(&[])); // example 1
}

// G2: the first occurrence of each kind wins.

#[test]
fn g2_first_occurrence_of_a_kind_wins() {
    assert_eq!(
        greeting(&[Fact::Streak(2), Fact::Streak(5)]),
        s(&["That's 2 days in a row."]) // example 18
    );
}

#[test]
fn g2_first_waiting_wins() {
    assert_eq!(
        greeting(&[
            Fact::Waiting("the glue-up".to_string()),
            Fact::Waiting("the finish".to_string()),
        ]),
        s(&["Waiting on the glue-up."])
    );
}

// G3: zero counts are absent.

#[test]
fn g3_zero_streak_is_absent() {
    assert_eq!(greeting(&[Fact::Streak(0)]), s(&[])); // example 8
}

#[test]
fn g3_zero_streak_leaves_favourite() {
    assert_eq!(
        greeting(&[Fact::Streak(0), Fact::Favourite("oak".to_string()),]),
        s(&["Still loving the oak."]) // example 21
    );
}

#[test]
fn g3_zero_completed_is_absent() {
    assert_eq!(greeting(&[Fact::CompletedProjects(0)]), s(&[])); // example 11
}

// G4: blank favourite or waiting text is absent.

#[test]
fn g4_blank_waiting_is_absent() {
    assert_eq!(
        greeting(&[
            Fact::Favourite("oak".to_string()),
            Fact::Waiting("   ".to_string()),
        ]),
        s(&["Still loving the oak."]) // example 20
    );
}

#[test]
fn g4_blank_favourite_is_absent() {
    assert_eq!(greeting(&[Fact::Favourite(String::new())]), s(&[]));
}

// G5: days is the lead.

#[test]
fn g5_days_lead_many() {
    assert_eq!(
        greeting(&[Fact::DaysSinceLast(9)]),
        s(&["Welcome back - it's been 9 days."]) // example 2
    );
}

#[test]
fn g5_days_lead_one_is_yesterday() {
    assert_eq!(
        greeting(&[Fact::DaysSinceLast(1)]),
        s(&["Welcome back - it's been yesterday."]) // example 3
    );
}

#[test]
fn g5_days_lead_zero_is_plain_welcome() {
    assert_eq!(
        greeting(&[Fact::DaysSinceLast(0)]),
        s(&["Welcome back."]) // example 4
    );
}

// G6: streak is the lead when there is no days fact.

#[test]
fn g6_streak_lead_is_singular_for_one() {
    assert_eq!(
        greeting(&[Fact::Streak(1)]),
        s(&["That's 1 day in a row."]) // example 7
    );
}

// G7: completed projects is the lead when there is no days or streak fact.

#[test]
fn g7_completed_lead_is_singular_for_one() {
    assert_eq!(
        greeting(&[Fact::CompletedProjects(1)]),
        s(&["You've finished 1 project."]) // example 9
    );
}

#[test]
fn g7_completed_lead_is_plural_for_three() {
    assert_eq!(
        greeting(&[Fact::CompletedProjects(3)]),
        s(&["You've finished 3 projects."]) // example 10
    );
}

// G8: favourite is the lead only when nothing stronger is present.

#[test]
fn g8_favourite_lead() {
    assert_eq!(
        greeting(&[Fact::Favourite("oak".to_string())]),
        s(&["Still loving the oak."]) // example 12
    );
}

// G9/G10: waiting alone is line 1 and the last line.

#[test]
fn g9_g10_waiting_alone_is_the_only_line() {
    assert_eq!(
        greeting(&[Fact::Waiting("the glue-up".to_string())]),
        s(&["Waiting on the glue-up."]) // example 13
    );
}

#[test]
fn g10_waiting_is_last_after_days() {
    assert_eq!(
        greeting(&[
            Fact::DaysSinceLast(9),
            Fact::Waiting("the drawer fronts".to_string()),
        ]),
        s(&[
            "Welcome back - it's been 9 days.",
            "Waiting on the drawer fronts.",
        ]) // examples 5 and 17
    );
}

#[test]
fn g10_waiting_is_last_after_streak() {
    assert_eq!(
        greeting(&[Fact::Streak(4), Fact::Waiting("the finish".to_string()),]),
        s(&["That's 4 days in a row.", "Waiting on the finish."]) // example 14
    );
}

// G11: the tail when there is no waiting.

#[test]
fn g11_days_then_streak() {
    assert_eq!(
        greeting(&[Fact::DaysSinceLast(9), Fact::Streak(3)]),
        s(&[
            "Welcome back - it's been 9 days.",
            "That's 3 days in a row.",
        ]) // example 6
    );
}

#[test]
fn g11_days_then_completed_drops_favourite() {
    assert_eq!(
        greeting(&[
            Fact::DaysSinceLast(2),
            Fact::CompletedProjects(3),
            Fact::Favourite("oak".to_string()),
        ]),
        s(&[
            "Welcome back - it's been 2 days.",
            "You've finished 3 projects.",
        ]) // example 15
    );
}

#[test]
fn g11_days_then_favourite_when_nothing_stronger() {
    assert_eq!(
        greeting(&[Fact::DaysSinceLast(2), Fact::Favourite("oak".to_string()),]),
        s(&["Welcome back - it's been 2 days.", "Still loving the oak."])
    );
}

#[test]
fn g11_streak_then_completed() {
    assert_eq!(
        greeting(&[Fact::Streak(3), Fact::CompletedProjects(2)]),
        s(&["That's 3 days in a row.", "You've finished 2 projects."])
    );
}

#[test]
fn g11_streak_then_favourite() {
    assert_eq!(
        greeting(&[Fact::Streak(3), Fact::Favourite("oak".to_string())]),
        s(&["That's 3 days in a row.", "Still loving the oak."])
    );
}

#[test]
fn g11_completed_then_favourite() {
    assert_eq!(
        greeting(&[
            Fact::CompletedProjects(2),
            Fact::Favourite("oak".to_string()),
        ]),
        s(&["You've finished 2 projects.", "Still loving the oak."]) // example 16
    );
}

#[test]
fn g8_completed_outranks_favourite() {
    assert_eq!(
        greeting(&[
            Fact::Favourite("oak".to_string()),
            Fact::CompletedProjects(2),
        ]),
        s(&["You've finished 2 projects.", "Still loving the oak."])
    );
}

// G12: assembly, the line cap, and priority independent of input order.

#[test]
fn g12_all_five_facts_give_two_lines() {
    assert_eq!(
        greeting(&[
            Fact::DaysSinceLast(9),
            Fact::Streak(3),
            Fact::CompletedProjects(2),
            Fact::Favourite("oak".to_string()),
            Fact::Waiting("the drawer fronts".to_string()),
        ]),
        s(&[
            "Welcome back - it's been 9 days.",
            "Waiting on the drawer fronts.",
        ]) // example 17
    );
}

#[test]
fn g12_never_more_than_max_lines() {
    assert_eq!(MAX_LINES, 2);
    let facts = [
        Fact::DaysSinceLast(9),
        Fact::Streak(3),
        Fact::CompletedProjects(2),
        Fact::Favourite("oak".to_string()),
        Fact::Waiting("the drawer fronts".to_string()),
    ];
    assert!(greeting(&facts).len() <= MAX_LINES);
}

#[test]
fn g12_kind_priority_ignores_input_order() {
    assert_eq!(
        greeting(&[
            Fact::Waiting("the glue-up".to_string()),
            Fact::DaysSinceLast(1),
        ]),
        s(&[
            "Welcome back - it's been yesterday.",
            "Waiting on the glue-up.",
        ]) // example 19
    );
}

// No input may panic; the biggest u32 values are worded, not guessed.

#[test]
fn g5_days_at_max_does_not_panic() {
    assert_eq!(
        greeting(&[Fact::DaysSinceLast(u32::MAX)]),
        s(&["Welcome back - it's been 4294967295 days."])
    );
}

#[test]
fn d1_days_phrase_at_max_does_not_panic() {
    assert_eq!(days_phrase(u32::MAX), "4294967295 days");
}

#[test]
fn g12_all_kinds_at_max_do_not_panic() {
    let facts = [
        Fact::DaysSinceLast(u32::MAX),
        Fact::Streak(u32::MAX),
        Fact::CompletedProjects(u32::MAX),
        Fact::Favourite("oak".to_string()),
        Fact::Waiting("the drawer fronts".to_string()),
    ];
    let lines = greeting(&facts);
    assert!(lines.len() <= MAX_LINES);
    assert_eq!(lines[0], "Welcome back - it's been 4294967295 days.");
}
