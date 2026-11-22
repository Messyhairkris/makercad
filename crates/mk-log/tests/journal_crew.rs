//! Tests for mk_log::journal written by Kimi (the Crew, campaign mk-core-8) from the contract
//! alone, without seeing the module.

//! Integration tests for the journal's append rules and computed views.

use mk_log::journal::{
    kind_of, Body, DecisionOption, DecisionSource, Journal, JournalError, MediaRef, MediaTarget,
    PromptMoment, Record, RecordId, RecordKind, Timestamp,
};

fn t(milliseconds: u64) -> Timestamp {
    Timestamp(milliseconds)
}

fn r(id: u64) -> RecordId {
    RecordId(id)
}

fn intent(goal: &str) -> Body {
    intent_why(goal, "")
}

fn intent_why(goal: &str, why: &str) -> Body {
    Body::Intent {
        goal: goal.to_string(),
        criteria: vec![],
        why: why.to_string(),
    }
}

fn decision(reason: &str) -> Body {
    decision_with_options(reason, vec![])
}

fn decision_with_options(reason: &str, options: Vec<DecisionOption>) -> Body {
    Body::Decision {
        question: "which wood?".to_string(),
        options,
        choice: Some("pine".to_string()),
        reason: reason.to_string(),
        source: DecisionSource::WhyButton,
    }
}

fn step(order: u32, text: &str) -> Body {
    Body::Step {
        order,
        text: text.to_string(),
    }
}

fn session(did: &[&str]) -> Body {
    session_with_media(did, vec![])
}

fn session_with_media(did: &[&str], media: Vec<MediaRef>) -> Body {
    Body::Session {
        did: did.iter().map(|item| (*item).to_string()).collect(),
        worked: vec![],
        didnt: vec![],
        next: vec![],
        media,
        mood: None,
    }
}

fn media_ref(clip: Option<(u64, u64)>) -> MediaRef {
    MediaRef {
        path: "media.mp4".to_string(),
        hash: None,
        external: false,
        clip,
    }
}

fn media(clip: Option<(u64, u64)>) -> Body {
    Body::Media {
        media: media_ref(clip),
        caption: None,
        shows: MediaTarget::Unassigned,
    }
}

fn finding_override(rule: &str, reason: &str) -> Body {
    Body::FindingOverride {
        rule: rule.to_string(),
        reason: reason.to_string(),
    }
}

fn outcome(final_photos: Vec<MediaRef>) -> Body {
    Body::Outcome {
        final_photos,
        would_change: vec![],
        build_again: None,
        tips: vec![],
    }
}

fn open_page(prompt: PromptMoment) -> Body {
    Body::OpenPage { prompt }
}

fn record(id: u64, at: u64, supersedes: Option<RecordId>, body: Body) -> Record {
    Record {
        id: r(id),
        at: t(at),
        supersedes,
        body,
    }
}

fn ids(records: Vec<&Record>) -> Vec<RecordId> {
    records.into_iter().map(|record| record.id).collect()
}

fn record_ids(records: &[Record]) -> Vec<RecordId> {
    records.iter().map(|record| record.id).collect()
}

fn current_ids(journal: &Journal) -> Vec<RecordId> {
    ids(journal.current())
}

fn history_ids(journal: &Journal, id: RecordId) -> Vec<RecordId> {
    ids(journal.history_of(id))
}

#[test]
fn worked_example_every_row_and_then_column_in_order() {
    let mut journal = Journal::new();
    assert!(journal.records().is_empty());

    // Row 1.
    assert_eq!(journal.append(t(10), None, intent("dresser")), Ok(r(1)));
    assert_eq!(current_ids(&journal), vec![r(1)]);

    // Row 2.
    assert_eq!(
        journal.append(t(20), None, open_page(PromptMoment::FirstMaterial)),
        Ok(r(2))
    );
    assert_eq!(current_ids(&journal), vec![r(1), r(2)]);
    assert_eq!(ids(journal.open_pages()), vec![r(2)]);

    // Row 3.
    assert_eq!(
        journal.append(
            t(30),
            Some(r(1)),
            intent_why("dresser, 4 drawers", "more storage"),
        ),
        Ok(r(3))
    );
    assert_eq!(current_ids(&journal), vec![r(2), r(3)]);
    assert_eq!(history_ids(&journal, r(1)), vec![r(1), r(3)]);

    // Row 4: the two successful records after this refusal must start at id 4.
    let before = journal.records().to_vec();
    assert_eq!(
        journal.append(t(31), Some(r(1)), intent("x")),
        Err(JournalError::AlreadySuperseded)
    );
    assert_eq!(journal.records(), before.as_slice());

    // Row 5.
    let before = journal.records().to_vec();
    assert_eq!(
        journal.append(t(32), Some(r(3)), step(1, "cut")),
        Err(JournalError::KindMismatch)
    );
    assert_eq!(journal.records(), before.as_slice());

    // Row 6.
    assert_eq!(
        journal.append(t(33), Some(r(2)), decision("had it")),
        Ok(r(4))
    );
    assert!(journal.open_pages().is_empty());
    assert_eq!(current_ids(&journal), vec![r(3), r(4)]);
    assert_eq!(history_ids(&journal, r(4)), vec![r(2), r(4)]);

    // Row 7.
    let before = journal.records().to_vec();
    assert_eq!(
        journal.append(t(5), None, step(1, "cut")),
        Err(JournalError::EarlierThanLast)
    );
    assert_eq!(journal.records(), before.as_slice());

    // Row 8.
    assert_eq!(journal.append(t(40), None, step(1, "cut")), Ok(r(5)));

    // Row 9.
    let before = journal.records().to_vec();
    assert_eq!(
        journal.append(t(41), None, step(1, "sand")),
        Err(JournalError::DuplicateStepOrder)
    );
    assert_eq!(journal.records(), before.as_slice());

    // Row 10.
    assert_eq!(
        journal.append(t(41), Some(r(5)), step(1, "cut to length")),
        Ok(r(6))
    );
    assert_eq!(current_ids(&journal), vec![r(3), r(4), r(6)]);

    // Row 11.
    assert_eq!(journal.append(t(50), None, session(&["cut"])), Ok(r(7)));
    assert_eq!(
        journal.append(t(51), Some(r(7)), session(&["cut", "glued"])),
        Ok(r(8))
    );
    assert_eq!(journal.last_session().map(|record| record.id), Some(r(8)));

    // Row 12.
    let before = journal.records().to_vec();
    assert_eq!(
        journal.append(t(60), None, media(Some((5, 5)))),
        Err(JournalError::BadClip)
    );
    assert_eq!(journal.records(), before.as_slice());

    // Row 13.
    let before = journal.records().to_vec();
    assert_eq!(
        journal.append(t(60), Some(r(99)), intent("y")),
        Err(JournalError::UnknownSupersede)
    );
    assert_eq!(journal.records(), before.as_slice());

    // Row 14.
    let before = journal.records().to_vec();
    assert_eq!(
        journal.append(t(60), None, decision("  ")),
        Err(JournalError::EmptyReason)
    );
    assert_eq!(journal.records(), before.as_slice());
    assert_eq!(
        record_ids(journal.records()),
        vec![r(1), r(2), r(3), r(4), r(5), r(6), r(7), r(8)]
    );
}

#[test]
fn a1_unknown_supersede_is_the_first_error() {
    let mut journal = Journal::new();
    assert_eq!(journal.append(t(10), None, intent("goal")), Ok(r(1)));
    let before = journal.records().to_vec();

    // The unknown id takes precedence over both the earlier time and empty goal.
    assert_eq!(
        journal.append(t(0), Some(r(99)), intent("")),
        Err(JournalError::UnknownSupersede)
    );
    assert_eq!(journal.records(), before.as_slice());
}

#[test]
fn a2_an_edit_history_cannot_fork() {
    let mut journal = Journal::new();
    assert_eq!(journal.append(t(10), None, intent("one")), Ok(r(1)));
    assert_eq!(journal.append(t(20), Some(r(1)), intent("two")), Ok(r(2)));
    let before = journal.records().to_vec();

    // A2 also precedes the kind and timestamp failures this body would cause.
    assert_eq!(
        journal.append(t(0), Some(r(1)), step(1, "fork")),
        Err(JournalError::AlreadySuperseded)
    );
    assert_eq!(journal.records(), before.as_slice());
}

#[test]
fn a3_superseding_an_ordinary_record_requires_the_same_kind() {
    let mut journal = Journal::new();
    assert_eq!(journal.append(t(10), None, intent("goal")), Ok(r(1)));
    let before = journal.records().to_vec();

    // A3 precedes A4, so the earlier time does not hide the mismatch.
    assert_eq!(
        journal.append(t(0), Some(r(1)), step(1, "cut")),
        Err(JournalError::KindMismatch)
    );
    assert_eq!(journal.records(), before.as_slice());
}

#[test]
fn a3_open_page_can_be_answered_by_a_decision() {
    let mut journal = Journal::new();
    assert_eq!(
        journal.append(t(10), None, open_page(PromptMoment::FirstMaterial)),
        Ok(r(1))
    );
    assert_eq!(
        journal.append(t(11), Some(r(1)), decision("had it")),
        Ok(r(2))
    );
    assert_eq!(current_ids(&journal), vec![r(2)]);
    assert_eq!(history_ids(&journal, r(1)), vec![r(1), r(2)]);
    assert_eq!(history_ids(&journal, r(2)), vec![r(1), r(2)]);
}

#[test]
fn a3_open_page_can_be_answered_by_a_session() {
    let mut journal = Journal::new();
    assert_eq!(
        journal.append(t(10), None, open_page(PromptMoment::SessionEnd)),
        Ok(r(1))
    );
    assert_eq!(
        journal.append(t(11), Some(r(1)), session(&["finished"])),
        Ok(r(2))
    );
    assert_eq!(current_ids(&journal), vec![r(2)]);
    assert_eq!(history_ids(&journal, r(2)), vec![r(1), r(2)]);
    assert_eq!(journal.last_session().map(|record| record.id), Some(r(2)));
}

#[test]
fn a4_earlier_than_the_last_record_is_refused_before_field_checks() {
    let mut journal = Journal::new();
    assert_eq!(journal.append(t(10), None, intent("goal")), Ok(r(1)));
    let before = journal.records().to_vec();

    assert_eq!(
        journal.append(t(9), None, intent(" \t")),
        Err(JournalError::EarlierThanLast)
    );
    assert_eq!(journal.records(), before.as_slice());
}

#[test]
fn a4_a_time_equal_to_the_last_record_is_accepted() {
    let mut journal = Journal::new();
    assert_eq!(journal.append(t(10), None, intent("goal")), Ok(r(1)));
    assert_eq!(journal.append(t(10), None, step(1, "cut")), Ok(r(2)));
    assert_eq!(current_ids(&journal), vec![r(1), r(2)]);
}

#[test]
fn a5_intent_goal_cannot_be_empty_or_whitespace() {
    let mut journal = Journal::new();
    for goal in ["", " \t\n"] {
        assert_eq!(
            journal.append(t(1), None, intent(goal)),
            Err(JournalError::EmptyGoal)
        );
    }
    assert!(journal.records().is_empty());
}

#[test]
fn a5_decision_and_finding_override_reasons_cannot_be_whitespace() {
    let mut journal = Journal::new();
    let bodies = vec![decision(" \t"), finding_override("rule-1", "\n  ")];
    for body in bodies {
        assert_eq!(
            journal.append(t(1), None, body),
            Err(JournalError::EmptyReason)
        );
    }
    assert!(journal.records().is_empty());
}

#[test]
fn a5_finding_override_rule_cannot_be_empty_or_whitespace() {
    let mut journal = Journal::new();
    for rule in ["", "  \t\n"] {
        assert_eq!(
            journal.append(t(1), None, finding_override(rule, "reason")),
            Err(JournalError::EmptyRule)
        );
    }
    assert!(journal.records().is_empty());
}

#[test]
fn a5_bad_clip_is_checked_in_media_body() {
    let mut journal = Journal::new();
    assert_eq!(
        journal.append(t(1), None, media(Some((5, 5)))),
        Err(JournalError::BadClip)
    );
    assert!(journal.records().is_empty());
}

#[test]
fn a5_bad_clip_is_checked_in_session_media() {
    let mut journal = Journal::new();
    let body = session_with_media(&["filmed"], vec![media_ref(Some((8, 3)))]);
    assert_eq!(journal.append(t(1), None, body), Err(JournalError::BadClip));
    assert!(journal.records().is_empty());
}

#[test]
fn a5_bad_clip_is_checked_in_outcome_final_photos() {
    let mut journal = Journal::new();
    let body = outcome(vec![media_ref(Some((0, 0)))]);
    assert_eq!(journal.append(t(1), None, body), Err(JournalError::BadClip));
    assert!(journal.records().is_empty());
}

#[test]
fn a5_bad_clip_is_checked_in_decision_option_thumbnail() {
    let mut journal = Journal::new();
    let options = vec![DecisionOption {
        label: "option".to_string(),
        thumbnail: Some(media_ref(Some((7, 7)))),
    }];
    assert_eq!(
        journal.append(t(1), None, decision_with_options("reason", options)),
        Err(JournalError::BadClip)
    );
    assert!(journal.records().is_empty());
}

#[test]
fn a5_field_checks_run_in_the_contract_order() {
    let mut journal = Journal::new();

    // EmptyReason precedes a bad thumbnail elsewhere in the same decision.
    let options = vec![DecisionOption {
        label: "option".to_string(),
        thumbnail: Some(media_ref(Some((4, 4)))),
    }];
    assert_eq!(
        journal.append(t(1), None, decision_with_options(" ", options)),
        Err(JournalError::EmptyReason)
    );

    // EmptyReason also precedes EmptyRule for a finding override.
    assert_eq!(
        journal.append(t(1), None, finding_override("", "")),
        Err(JournalError::EmptyReason)
    );
    assert_eq!(
        journal.append(t(1), None, finding_override("", "reason")),
        Err(JournalError::EmptyRule)
    );
    assert!(journal.records().is_empty());
}

#[test]
fn a5_clip_with_in_point_before_out_point_is_accepted() {
    let mut journal = Journal::new();
    assert_eq!(journal.append(t(1), None, media(Some((4, 5)))), Ok(r(1)));
    assert_eq!(current_ids(&journal), vec![r(1)]);
}

#[test]
fn a6_step_order_cannot_clash_with_another_current_step() {
    let mut journal = Journal::new();
    assert_eq!(journal.append(t(10), None, step(7, "cut")), Ok(r(1)));
    let before = journal.records().to_vec();
    assert_eq!(
        journal.append(t(11), None, step(7, "sand")),
        Err(JournalError::DuplicateStepOrder)
    );
    assert_eq!(journal.records(), before.as_slice());
}

#[test]
fn a6_new_step_may_reuse_the_order_of_the_step_it_supersedes() {
    let mut journal = Journal::new();
    assert_eq!(journal.append(t(10), None, step(1, "cut")), Ok(r(1)));
    assert_eq!(
        journal.append(t(11), Some(r(1)), step(1, "cut to length")),
        Ok(r(2))
    );
    assert_eq!(current_ids(&journal), vec![r(2)]);
    assert_eq!(history_ids(&journal, r(1)), vec![r(1), r(2)]);
}

#[test]
fn a7_a8_assign_ids_append_in_order_and_preserve_earlier_records() {
    let expected = vec![
        record(1, 10, None, intent("goal")),
        record(2, 20, None, open_page(PromptMoment::FirstMaterial)),
        record(3, 30, None, step(1, "cut")),
    ];
    let mut journal = Journal::new();

    for expected_record in &expected {
        assert_eq!(
            journal.append(
                expected_record.at,
                expected_record.supersedes,
                expected_record.body.clone(),
            ),
            Ok(expected_record.id)
        );
    }
    assert_eq!(journal.records(), expected.as_slice());
}

#[test]
fn v1_current_contains_exactly_unsuperseded_records_in_append_order() {
    let mut journal = Journal::new();
    assert_eq!(journal.append(t(1), None, intent("one")), Ok(r(1)));
    assert_eq!(
        journal.append(t(2), None, open_page(PromptMoment::FirstMaterial)),
        Ok(r(2))
    );
    assert_eq!(journal.append(t(3), Some(r(1)), intent("two")), Ok(r(3)));
    assert_eq!(journal.append(t(4), None, step(1, "cut")), Ok(r(4)));
    assert_eq!(
        journal.append(t(5), Some(r(2)), session(&["answered"])),
        Ok(r(5))
    );

    assert_eq!(current_ids(&journal), vec![r(3), r(4), r(5)]);
}

#[test]
fn v2_history_of_every_record_in_a_three_record_chain_and_unknown_id() {
    let mut journal = Journal::new();
    assert_eq!(journal.append(t(1), None, intent("one")), Ok(r(1)));
    assert_eq!(journal.append(t(2), Some(r(1)), intent("two")), Ok(r(2)));
    assert_eq!(journal.append(t(3), Some(r(2)), intent("three")), Ok(r(3)));

    let chain = vec![r(1), r(2), r(3)];
    assert_eq!(history_ids(&journal, r(1)), chain);
    assert_eq!(history_ids(&journal, r(2)), chain);
    assert_eq!(history_ids(&journal, r(3)), chain);
    assert!(journal.history_of(r(99)).is_empty());
    assert!(journal.history_of(r(0)).is_empty());
}

#[test]
fn v3_last_session_is_the_correction_not_the_superseded_session() {
    let mut journal = Journal::new();
    assert_eq!(journal.append(t(10), None, session(&["cut"])), Ok(r(1)));
    let corrected_body = session(&["cut", "glued"]);
    assert_eq!(
        journal.append(t(20), Some(r(1)), corrected_body.clone()),
        Ok(r(2))
    );
    assert_eq!(journal.append(t(30), None, intent("later")), Ok(r(3)));

    let expected = record(2, 20, Some(r(1)), corrected_body);
    assert_eq!(journal.last_session(), Some(&expected));
}

#[test]
fn v4_answering_an_open_page_removes_only_that_page() {
    let mut journal = Journal::new();
    assert_eq!(
        journal.append(t(1), None, open_page(PromptMoment::FirstMaterial)),
        Ok(r(1))
    );
    assert_eq!(
        journal.append(t(2), None, open_page(PromptMoment::SessionEnd)),
        Ok(r(2))
    );
    assert_eq!(ids(journal.open_pages()), vec![r(1), r(2)]);

    assert_eq!(
        journal.append(t(3), Some(r(1)), decision("answered")),
        Ok(r(3))
    );
    assert_eq!(ids(journal.open_pages()), vec![r(2)]);
    assert_eq!(current_ids(&journal), vec![r(2), r(3)]);
}

#[test]
fn refused_append_preserves_records_and_leaves_the_next_id_unchanged() {
    let mut journal = Journal::new();
    assert_eq!(journal.append(t(10), None, intent("valid")), Ok(r(1)));
    let before = journal.records().to_vec();

    assert_eq!(
        journal.append(t(20), None, intent("")),
        Err(JournalError::EmptyGoal)
    );
    assert_eq!(journal.records(), before.as_slice());
    assert_eq!(journal.append(t(20), None, intent("next")), Ok(r(2)));
}

#[test]
fn from_records_accepts_good_records_and_preserves_them_unchanged() {
    let records = vec![
        record(1, 10, None, intent("dresser")),
        record(2, 20, None, open_page(PromptMoment::FirstMaterial)),
        record(
            3,
            30,
            Some(r(1)),
            intent_why("dresser, 4 drawers", "more storage"),
        ),
        record(4, 33, Some(r(2)), decision("had it")),
        record(5, 40, None, step(1, "cut")),
        record(6, 41, Some(r(5)), step(1, "cut to length")),
        record(7, 50, None, session(&["cut"])),
        record(8, 51, Some(r(7)), session(&["cut", "glued"])),
    ];

    let journal = Journal::from_records(records.clone()).expect("all records satisfy A1-A7");
    assert_eq!(journal.records(), records.as_slice());
    assert_eq!(current_ids(&journal), vec![r(3), r(4), r(6), r(8)]);
    assert_eq!(history_ids(&journal, r(1)), vec![r(1), r(3)]);
    assert_eq!(history_ids(&journal, r(4)), vec![r(2), r(4)]);
    assert_eq!(journal.last_session().map(|record| record.id), Some(r(8)));
}

#[test]
fn from_records_reports_a_wrong_id_before_other_record_errors() {
    let records = vec![
        record(1, 10, None, intent("good")),
        // Index 1 has the wrong id and an unknown supersede target; BadId wins.
        record(3, 20, Some(r(99)), intent("")),
    ];

    assert_eq!(
        Journal::from_records(records),
        Err((1, JournalError::BadId))
    );
}

#[test]
fn from_records_reports_a_bad_record_in_the_middle_with_its_index() {
    let records = vec![
        record(1, 10, None, intent("good")),
        record(2, 20, Some(r(99)), intent("bad")),
        record(3, 30, None, open_page(PromptMoment::SessionEnd)),
    ];

    assert_eq!(
        Journal::from_records(records),
        Err((1, JournalError::UnknownSupersede))
    );
}

#[test]
fn long_journal_a7_a8_v1_v2_remain_consistent_for_thousands_of_records() {
    const COUNT: u64 = 2_500;

    let mut journal = Journal::new();
    for index in 0..COUNT {
        let supersedes = if index == 0 { None } else { Some(r(index)) };
        let body = intent(&format!("goal {index}"));
        assert_eq!(journal.append(t(index), supersedes, body), Ok(r(index + 1)));
    }

    let expected_ids: Vec<RecordId> = (1..=COUNT).map(r).collect();
    assert_eq!(record_ids(journal.records()), expected_ids);
    assert_eq!(history_ids(&journal, r(1)), expected_ids);
    assert_eq!(history_ids(&journal, r(COUNT)), expected_ids);
    assert_eq!(current_ids(&journal), vec![r(COUNT)]);
    assert!(journal.last_session().is_none());

    assert_eq!(
        journal.append(t(COUNT), None, session(&["finished"])),
        Ok(r(COUNT + 1))
    );
    assert_eq!(
        journal.last_session().map(|record| record.id),
        Some(r(COUNT + 1))
    );
}

#[test]
fn kind_of_matches_every_body_variant() {
    let cases = vec![
        (intent("goal"), RecordKind::Intent),
        (decision("reason"), RecordKind::Decision),
        (step(1, "cut"), RecordKind::Step),
        (session(&["cut"]), RecordKind::Session),
        (media(None), RecordKind::Media),
        (
            finding_override("rule", "reason"),
            RecordKind::FindingOverride,
        ),
        (
            Body::AsBuilt {
                deviations: vec![],
                callouts: vec![],
            },
            RecordKind::AsBuilt,
        ),
        (outcome(vec![]), RecordKind::Outcome),
        (open_page(PromptMoment::Conclusion), RecordKind::OpenPage),
    ];

    for (body, kind) in cases {
        assert_eq!(kind_of(&body), kind);
    }
}
