//! Contract tests for mk_log::journal, written by Qwen as a PA (the Crew) from
//! crew/contracts/mk_journal.md; the hidden tests of campaign mk-core-8, plus one test Claude added
//! after mutation testing.

use mk_log::journal::{
    kind_of, Body, DecisionOption, DecisionSource, Deviation, Journal, JournalError, MediaRef,
    MediaTarget, PromptMoment, Record, RecordId, RecordKind, Timestamp,
};

// ---- small builders ----------------------------------------------------------

fn t(ms: u64) -> Timestamp {
    Timestamp(ms)
}

fn r(id: u64) -> RecordId {
    RecordId(id)
}

fn intent(goal: &str, why: &str) -> Body {
    Body::Intent {
        goal: goal.to_string(),
        criteria: Vec::new(),
        why: why.to_string(),
    }
}

fn open_page(prompt: PromptMoment) -> Body {
    Body::OpenPage { prompt }
}

fn step(order: u32, text: &str) -> Body {
    Body::Step {
        order,
        text: text.to_string(),
    }
}

fn session(did: Vec<&str>) -> Body {
    Body::Session {
        did: did.iter().map(|s| s.to_string()).collect(),
        worked: Vec::new(),
        didnt: Vec::new(),
        next: Vec::new(),
        media: Vec::new(),
        mood: None,
    }
}

fn decision(question: &str, reason: &str) -> Body {
    Body::Decision {
        question: question.to_string(),
        options: Vec::new(),
        choice: None,
        reason: reason.to_string(),
        source: DecisionSource::WhyButton,
    }
}

fn media(clip: Option<(u64, u64)>) -> MediaRef {
    MediaRef {
        path: "p".to_string(),
        hash: None,
        external: false,
        clip,
    }
}

fn ids(recs: &[&Record]) -> Vec<u64> {
    recs.iter().map(|rec| rec.id.0).collect()
}

// ---- A1: unknown supersede ---------------------------------------------------

#[test]
fn a1_unknown_supersede() {
    let mut j = Journal::new();
    assert_eq!(
        j.append(t(10), Some(r(999)), intent("g", "")),
        Err(JournalError::UnknownSupersede)
    );
}

// ---- A2: already superseded --------------------------------------------------

#[test]
fn a2_already_superseded() {
    let mut j = Journal::new();
    assert_eq!(j.append(t(10), None, intent("g1", "")), Ok(r(1)));
    assert_eq!(j.append(t(20), Some(r(1)), intent("g2", "")), Ok(r(2)));
    // record 1 is already superseded by 2; a fork is refused.
    assert_eq!(
        j.append(t(30), Some(r(1)), intent("g3", "")),
        Err(JournalError::AlreadySuperseded)
    );
}

// ---- A3: kind mismatch, and an OpenPage takes any kind -----------------------

#[test]
fn a3_kind_mismatch() {
    let mut j = Journal::new();
    assert_eq!(j.append(t(10), None, intent("g", "")), Ok(r(1)));
    // supersede an Intent with a Step: the kinds differ.
    assert_eq!(
        j.append(t(20), Some(r(1)), step(1, "cut")),
        Err(JournalError::KindMismatch)
    );
}

#[test]
fn a3_open_page_takes_any_kind() {
    let mut j = Journal::new();
    assert_eq!(
        j.append(t(10), None, open_page(PromptMoment::FirstMaterial)),
        Ok(r(1))
    );
    // an OpenPage may be answered by a record of any kind.
    assert_eq!(j.append(t(20), Some(r(1)), step(1, "cut")), Ok(r(2)));
}

// ---- A4: time only moves forward ---------------------------------------------

#[test]
fn a4_earlier_than_last() {
    let mut j = Journal::new();
    assert_eq!(j.append(t(10), None, intent("g", "")), Ok(r(1)));
    assert_eq!(
        j.append(t(5), None, intent("g2", "")),
        Err(JournalError::EarlierThanLast)
    );
}

#[test]
fn a4_equal_time_is_fine() {
    let mut j = Journal::new();
    assert_eq!(j.append(t(10), None, intent("g", "")), Ok(r(1)));
    assert_eq!(j.append(t(10), None, intent("g2", "")), Ok(r(2)));
}

// ---- A5: field checks, in order ----------------------------------------------

#[test]
fn a5_empty_goal() {
    let mut j = Journal::new();
    assert_eq!(
        j.append(t(10), None, intent("", "")),
        Err(JournalError::EmptyGoal)
    );
}

#[test]
fn a5_whitespace_goal() {
    let mut j = Journal::new();
    assert_eq!(
        j.append(t(10), None, intent("   ", "")),
        Err(JournalError::EmptyGoal)
    );
}

#[test]
fn a5_empty_reason_on_decision() {
    let mut j = Journal::new();
    assert_eq!(
        j.append(t(10), None, decision("q", "")),
        Err(JournalError::EmptyReason)
    );
}

#[test]
fn a5_empty_reason_on_finding_override() {
    let mut j = Journal::new();
    let body = Body::FindingOverride {
        rule: "r".to_string(),
        reason: "".to_string(),
    };
    assert_eq!(j.append(t(10), None, body), Err(JournalError::EmptyReason));
}

#[test]
fn a5_whitespace_reason() {
    let mut j = Journal::new();
    assert_eq!(
        j.append(t(10), None, decision("q", "  ")),
        Err(JournalError::EmptyReason)
    );
}

#[test]
fn a5_empty_rule() {
    let mut j = Journal::new();
    // the reason is fine, so the empty rule is what is refused.
    let body = Body::FindingOverride {
        rule: "".to_string(),
        reason: "ok".to_string(),
    };
    assert_eq!(j.append(t(10), None, body), Err(JournalError::EmptyRule));
}

#[test]
fn a5_whitespace_rule() {
    let mut j = Journal::new();
    let body = Body::FindingOverride {
        rule: " \t ".to_string(),
        reason: "ok".to_string(),
    };
    assert_eq!(j.append(t(10), None, body), Err(JournalError::EmptyRule));
}

#[test]
fn a5_bad_clip_in_media() {
    let mut j = Journal::new();
    let body = Body::Media {
        media: media(Some((5, 5))),
        caption: None,
        shows: MediaTarget::Unassigned,
    };
    assert_eq!(j.append(t(10), None, body), Err(JournalError::BadClip));
}

#[test]
fn a5_bad_clip_in_session_media() {
    let mut j = Journal::new();
    let body = Body::Session {
        did: Vec::new(),
        worked: Vec::new(),
        didnt: Vec::new(),
        next: Vec::new(),
        media: vec![media(Some((9, 2)))],
        mood: None,
    };
    assert_eq!(j.append(t(10), None, body), Err(JournalError::BadClip));
}

#[test]
fn a5_bad_clip_in_outcome_photos() {
    let mut j = Journal::new();
    let body = Body::Outcome {
        final_photos: vec![media(Some((7, 7)))],
        would_change: Vec::new(),
        build_again: None,
        tips: Vec::new(),
    };
    assert_eq!(j.append(t(10), None, body), Err(JournalError::BadClip));
}

#[test]
fn a5_bad_clip_in_option_thumbnail() {
    let mut j = Journal::new();
    let opt = DecisionOption {
        label: "a".to_string(),
        thumbnail: Some(media(Some((4, 1)))),
    };
    let body = Body::Decision {
        question: "q".to_string(),
        options: vec![opt],
        choice: None,
        reason: "ok".to_string(),
        source: DecisionSource::WhyButton,
    };
    assert_eq!(j.append(t(10), None, body), Err(JournalError::BadClip));
}

#[test]
fn a5_good_clip_is_fine() {
    let mut j = Journal::new();
    let body = Body::Media {
        media: media(Some((1, 5))),
        caption: None,
        shows: MediaTarget::Unassigned,
    };
    assert_eq!(j.append(t(10), None, body), Ok(r(1)));
}

#[test]
fn a5_reason_is_checked_before_rule() {
    let mut j = Journal::new();
    // both the reason and the rule are empty; the reason is refused first.
    let body = Body::FindingOverride {
        rule: "".to_string(),
        reason: "".to_string(),
    };
    assert_eq!(j.append(t(10), None, body), Err(JournalError::EmptyReason));
}

#[test]
fn a5_reason_is_checked_before_clip() {
    let mut j = Journal::new();
    // the reason is empty and a thumbnail has a bad clip; the reason is refused first.
    let opt = DecisionOption {
        label: "a".to_string(),
        thumbnail: Some(media(Some((5, 5)))),
    };
    let body = Body::Decision {
        question: "q".to_string(),
        options: vec![opt],
        choice: None,
        reason: "".to_string(),
        source: DecisionSource::WhyButton,
    };
    assert_eq!(j.append(t(10), None, body), Err(JournalError::EmptyReason));
}

// ---- A6: step orders are unique ----------------------------------------------

#[test]
fn a6_duplicate_step_order() {
    let mut j = Journal::new();
    assert_eq!(j.append(t(10), None, step(1, "cut")), Ok(r(1)));
    assert_eq!(
        j.append(t(20), None, step(1, "sand")),
        Err(JournalError::DuplicateStepOrder)
    );
}

#[test]
fn a6_superseding_the_clashing_step_is_fine() {
    let mut j = Journal::new();
    assert_eq!(j.append(t(10), None, step(1, "cut")), Ok(r(1)));
    // a step may reuse an order when it supersedes the step that holds it.
    assert_eq!(
        j.append(t(20), Some(r(1)), step(1, "cut to length")),
        Ok(r(2))
    );
}

// ---- A7 and A8: ids, and a refusal changes nothing ---------------------------

#[test]
fn a7_first_id_is_one() {
    let mut j = Journal::new();
    assert_eq!(j.append(t(10), None, intent("g", "")), Ok(r(1)));
}

#[test]
fn a7_ids_increase_by_one() {
    let mut j = Journal::new();
    assert_eq!(j.append(t(10), None, intent("g1", "")), Ok(r(1)));
    assert_eq!(j.append(t(20), None, intent("g2", "")), Ok(r(2)));
    assert_eq!(j.append(t(30), None, intent("g3", "")), Ok(r(3)));
}

#[test]
fn a8_a_refused_record_changes_nothing() {
    let mut j = Journal::new();
    assert_eq!(j.append(t(10), None, intent("g", "")), Ok(r(1)));
    let before: Vec<Record> = j.records().to_vec();
    // an earlier time is refused; the list and the next id are untouched.
    assert_eq!(
        j.append(t(5), None, intent("g2", "")),
        Err(JournalError::EarlierThanLast)
    );
    assert_eq!(j.records().to_vec(), before);
    assert_eq!(j.append(t(20), None, intent("g2", "")), Ok(r(2)));
}

// ---- the worked example, in order, on one journal ----------------------------

#[test]
fn worked_example() {
    let mut j = Journal::new();

    // 1
    assert_eq!(j.append(t(10), None, intent("dresser", "")), Ok(r(1)));
    assert_eq!(ids(&j.current()), vec![1]);
    // 2
    assert_eq!(
        j.append(t(20), None, open_page(PromptMoment::FirstMaterial)),
        Ok(r(2))
    );
    assert_eq!(ids(&j.current()), vec![1, 2]);
    assert_eq!(ids(&j.open_pages()), vec![2]);
    // 3
    assert_eq!(
        j.append(
            t(30),
            Some(r(1)),
            intent("dresser, 4 drawers", "more storage")
        ),
        Ok(r(3))
    );
    assert_eq!(ids(&j.current()), vec![2, 3]);
    assert_eq!(ids(&j.history_of(r(1))), vec![1, 3]);
    // 4
    assert_eq!(
        j.append(t(31), Some(r(1)), intent("x", "")),
        Err(JournalError::AlreadySuperseded)
    );
    // 5
    assert_eq!(
        j.append(t(32), Some(r(3)), step(1, "cut")),
        Err(JournalError::KindMismatch)
    );
    // 6
    let wood = Body::Decision {
        question: "which wood?".to_string(),
        options: Vec::new(),
        choice: Some("pine".to_string()),
        reason: "had it".to_string(),
        source: DecisionSource::WhyButton,
    };
    assert_eq!(j.append(t(33), Some(r(2)), wood), Ok(r(4)));
    assert_eq!(ids(&j.open_pages()), Vec::<u64>::new());
    assert_eq!(ids(&j.current()), vec![3, 4]);
    assert_eq!(ids(&j.history_of(r(4))), vec![2, 4]);
    // 7
    assert_eq!(
        j.append(t(5), None, step(1, "cut")),
        Err(JournalError::EarlierThanLast)
    );
    // 8
    assert_eq!(j.append(t(40), None, step(1, "cut")), Ok(r(5)));
    // 9
    assert_eq!(
        j.append(t(41), None, step(1, "sand")),
        Err(JournalError::DuplicateStepOrder)
    );
    // 10
    assert_eq!(
        j.append(t(41), Some(r(5)), step(1, "cut to length")),
        Ok(r(6))
    );
    assert_eq!(ids(&j.current()), vec![3, 4, 6]);
    // 11
    assert_eq!(j.append(t(50), None, session(vec!["cut"])), Ok(r(7)));
    assert_eq!(
        j.append(t(51), Some(r(7)), session(vec!["cut", "glued"])),
        Ok(r(8))
    );
    assert_eq!(j.last_session().map(|rec| rec.id), Some(r(8)));
    // 12
    let bad_media = Body::Media {
        media: media(Some((5, 5))),
        caption: None,
        shows: MediaTarget::Unassigned,
    };
    assert_eq!(j.append(t(60), None, bad_media), Err(JournalError::BadClip));
    // 13
    assert_eq!(
        j.append(t(60), Some(r(99)), intent("y", "")),
        Err(JournalError::UnknownSupersede)
    );
    // 14
    assert_eq!(
        j.append(t(60), None, decision("q", "  ")),
        Err(JournalError::EmptyReason)
    );
}

// ---- V2: history_of ----------------------------------------------------------

#[test]
fn history_of_every_record_of_a_chain() {
    let mut j = Journal::new();
    assert_eq!(j.append(t(10), None, intent("g1", "")), Ok(r(1)));
    assert_eq!(j.append(t(20), Some(r(1)), intent("g2", "")), Ok(r(2)));
    assert_eq!(j.append(t(30), Some(r(2)), intent("g3", "")), Ok(r(3)));
    // every record of the chain sees the whole chain, oldest first.
    assert_eq!(ids(&j.history_of(r(1))), vec![1, 2, 3]);
    assert_eq!(ids(&j.history_of(r(2))), vec![1, 2, 3]);
    assert_eq!(ids(&j.history_of(r(3))), vec![1, 2, 3]);
}

#[test]
fn history_of_an_open_page_answered_by_another_kind() {
    let mut j = Journal::new();
    assert_eq!(
        j.append(t(10), None, open_page(PromptMoment::FirstMaterial)),
        Ok(r(1))
    );
    assert_eq!(j.append(t(20), Some(r(1)), decision("q", "r")), Ok(r(2)));
    // the OpenPage and the record that answers it are in one chain.
    assert_eq!(ids(&j.history_of(r(1))), vec![1, 2]);
    assert_eq!(ids(&j.history_of(r(2))), vec![1, 2]);
}

#[test]
fn history_of_an_unknown_id_is_empty() {
    let mut j = Journal::new();
    assert_eq!(j.append(t(10), None, intent("g", "")), Ok(r(1)));
    assert_eq!(ids(&j.history_of(r(999))), Vec::<u64>::new());
}

// ---- from_records ------------------------------------------------------------

fn worked_example_records() -> Vec<Record> {
    vec![
        Record {
            id: r(1),
            at: t(10),
            supersedes: None,
            body: intent("dresser", ""),
        },
        Record {
            id: r(2),
            at: t(20),
            supersedes: None,
            body: open_page(PromptMoment::FirstMaterial),
        },
        Record {
            id: r(3),
            at: t(30),
            supersedes: Some(r(1)),
            body: intent("dresser, 4 drawers", "more storage"),
        },
        Record {
            id: r(4),
            at: t(33),
            supersedes: Some(r(2)),
            body: Body::Decision {
                question: "which wood?".to_string(),
                options: Vec::new(),
                choice: Some("pine".to_string()),
                reason: "had it".to_string(),
                source: DecisionSource::WhyButton,
            },
        },
        Record {
            id: r(5),
            at: t(40),
            supersedes: None,
            body: step(1, "cut"),
        },
        Record {
            id: r(6),
            at: t(41),
            supersedes: Some(r(5)),
            body: step(1, "cut to length"),
        },
        Record {
            id: r(7),
            at: t(50),
            supersedes: None,
            body: session(vec!["cut"]),
        },
        Record {
            id: r(8),
            at: t(51),
            supersedes: Some(r(7)),
            body: session(vec!["cut", "glued"]),
        },
    ]
}

#[test]
fn from_records_on_the_worked_example() {
    let recs = worked_example_records();
    let j = Journal::from_records(recs.clone()).expect("the worked example is valid");
    assert_eq!(j.records().to_vec(), recs);
}

#[test]
fn from_records_a_wrong_id_is_bad_id_at_that_index() {
    let mut recs = worked_example_records();
    recs[2].id = r(9); // the third record must have id 3, not 9.
    assert_eq!(Journal::from_records(recs), Err((2, JournalError::BadId)));
}

#[test]
fn from_records_a_bad_record_in_the_middle() {
    let mut recs = worked_example_records();
    // the second record supersedes an id that does not exist.
    recs[1].supersedes = Some(r(99));
    assert_eq!(
        Journal::from_records(recs),
        Err((1, JournalError::UnknownSupersede))
    );
}

// ---- a long journal never panics ---------------------------------------------

#[test]
fn a_journal_of_thousands_of_records_never_panics() {
    let mut j = Journal::new();
    let n: u64 = 3000;
    for i in 1..=n {
        assert_eq!(
            j.append(Timestamp(i * 10), None, step(i as u32, "s")),
            Ok(RecordId(i))
        );
    }
    assert_eq!(j.records().len(), n as usize);
    assert_eq!(j.current().len(), n as usize);
    assert_eq!(j.history_of(RecordId(n)).len(), 1);
}

// ---- the views, on their own -------------------------------------------------

#[test]
fn kind_of_names_every_body() {
    assert_eq!(kind_of(&intent("g", "")), RecordKind::Intent);
    assert_eq!(
        kind_of(&open_page(PromptMoment::FirstMaterial)),
        RecordKind::OpenPage
    );
    assert_eq!(kind_of(&step(1, "cut")), RecordKind::Step);
    assert_eq!(kind_of(&session(vec!["x"])), RecordKind::Session);
    let media = Body::Media {
        media: media(None),
        caption: None,
        shows: MediaTarget::Unassigned,
    };
    assert_eq!(kind_of(&media), RecordKind::Media);
    let finding = Body::FindingOverride {
        rule: "r".to_string(),
        reason: "r".to_string(),
    };
    assert_eq!(kind_of(&finding), RecordKind::FindingOverride);
    let as_built = Body::AsBuilt {
        deviations: vec![Deviation {
            part: "p".to_string(),
            expected: "e".to_string(),
            actual: "a".to_string(),
        }],
        callouts: Vec::new(),
    };
    assert_eq!(kind_of(&as_built), RecordKind::AsBuilt);
    let outcome = Body::Outcome {
        final_photos: Vec::new(),
        would_change: Vec::new(),
        build_again: None,
        tips: Vec::new(),
    };
    assert_eq!(kind_of(&outcome), RecordKind::Outcome);
    assert_eq!(kind_of(&decision("q", "r")), RecordKind::Decision);
}

#[test]
fn current_drops_superseded_records() {
    let mut j = Journal::new();
    assert_eq!(j.append(t(10), None, intent("g1", "")), Ok(r(1)));
    assert_eq!(j.append(t(20), None, step(1, "cut")), Ok(r(2)));
    assert_eq!(j.append(t(30), Some(r(1)), intent("g2", "")), Ok(r(3)));
    // record 1 is superseded by 3, so it is no longer current.
    assert_eq!(ids(&j.current()), vec![2, 3]);
}

#[test]
fn last_session_is_none_when_there_is_no_session() {
    let mut j = Journal::new();
    assert_eq!(j.append(t(10), None, intent("g", "")), Ok(r(1)));
    assert_eq!(j.last_session(), None);
}

#[test]
fn last_session_is_the_corrected_one() {
    let mut j = Journal::new();
    assert_eq!(j.append(t(10), None, session(vec!["cut"])), Ok(r(1)));
    assert_eq!(
        j.append(t(20), Some(r(1)), session(vec!["cut", "glued"])),
        Ok(r(2))
    );
    // the corrected interview wins over the one it corrected.
    assert_eq!(j.last_session().map(|rec| rec.id), Some(r(2)));
}

#[test]
fn open_pages_drop_when_answered() {
    let mut j = Journal::new();
    assert_eq!(
        j.append(t(10), None, open_page(PromptMoment::FirstMaterial)),
        Ok(r(1))
    );
    assert_eq!(ids(&j.open_pages()), vec![1]);
    // answering the page supersedes it, so it is no longer current.
    assert_eq!(j.append(t(20), Some(r(1)), decision("q", "r")), Ok(r(2)));
    assert_eq!(ids(&j.open_pages()), Vec::<u64>::new());
}

// Added by Claude after mutation testing (mk-core-8): both test sets missed a decision option
// with no thumbnail, so `thumbnail: None` being refused as BadClip went unnoticed.
#[test]
fn a5_option_without_thumbnail_is_accepted() {
    let mut j = Journal::new();
    let body = Body::Decision {
        question: "which wood?".to_string(),
        options: vec![DecisionOption {
            label: "pine".to_string(),
            thumbnail: None,
        }],
        choice: Some("pine".to_string()),
        reason: "had it".to_string(),
        source: DecisionSource::WhyButton,
    };
    assert_eq!(j.append(Timestamp(1), None, body), Ok(RecordId(1)));
}
