//! Tests for mk_log::interview written by Gordon (the Crew, campaign mk-core-16) from the
//! contract alone, without seeing the module.

//! Integration tests for the session-end interview (src/interview.rs), written from the contract.

use mk_log::interview::{
    draft, should_offer, Answer, Cause, Done, Interview, InterviewError, Mood, Question, Trigger,
    CAUSES, DRAFT_MAX, LONG_SESSION_MINUTES, QUESTIONS,
};
use mk_log::journal::{Body, MediaRef, PromptMoment};

/// The simplest MediaRef: a local photo, no hash, no clip.
fn media(path: &str) -> MediaRef {
    MediaRef {
        path: path.to_string(),
        hash: None,
        external: false,
        clip: None,
    }
}

/// The command log of the contract's worked examples.
fn example_done() -> Vec<Done> {
    vec![
        Done::Added {
            count: 4,
            what: "drawers".to_string(),
        },
        Done::Changed {
            what: "the top".to_string(),
            from: "pine".to_string(),
            to: "plywood".to_string(),
        },
        Done::Generated {
            what: "cut diagrams".to_string(),
        },
    ]
}

/// The draft the worked examples pre-fill "what did you do" with.
const EXAMPLE_DRAFT: &str =
    "You added 4 drawers, changed the top from pine to plywood, and generated cut diagrams.";

/// Six phrases: exactly DRAFT_MAX things, so nothing is folded into "N more things".
const SIX_DRAFT: &str =
    "You generated a, generated b, generated c, generated d, generated e, and generated f.";

/// Seven phrases: the first DRAFT_MAX - 1 kept, the other 2 folded.
const SEVEN_DRAFT: &str =
    "You generated a, generated b, generated c, generated d, generated e, and 2 more things.";

/// The worked example's eight Generated things.
const EIGHT_DRAFT: &str =
    "You generated a, generated b, generated c, generated d, generated e, and 3 more things.";

/// Twenty phrases: the first five kept, the other 15 folded.
const TWENTY_DRAFT: &str =
    "You generated t1, generated t2, generated t3, generated t4, generated t5, and 15 more things.";

/// An interview with no draft and nothing answered.
fn fresh() -> Interview {
    Interview::new(Trigger::WrapUp, &[])
}

/// One Generated phrase per name, in order.
fn generated(names: &[&str]) -> Vec<Done> {
    names
        .iter()
        .map(|name| Done::Generated {
            what: name.to_string(),
        })
        .collect()
}

#[test]
fn rule_o1_wrap_up_is_always_offered() {
    assert_eq!(should_offer(true, true, 0, true), Some(Trigger::WrapUp));
    assert_eq!(should_offer(false, true, 0, false), Some(Trigger::WrapUp));
    assert_eq!(should_offer(false, true, 500, true), Some(Trigger::WrapUp));
}

#[test]
fn rule_o1_already_offered_gives_none() {
    assert_eq!(should_offer(true, false, 200, true), None);
    assert_eq!(should_offer(false, false, 500, true), None);
    assert_eq!(should_offer(false, false, 0, true), None);
}

#[test]
fn rule_o1_closing_beats_long_session() {
    assert_eq!(should_offer(true, false, 10, false), Some(Trigger::Closing));
    assert_eq!(
        should_offer(true, false, 500, false),
        Some(Trigger::Closing)
    );
}

#[test]
fn rule_o1_long_session_boundary() {
    assert_eq!(should_offer(false, false, 119, false), None);
    assert_eq!(
        should_offer(false, false, 120, false),
        Some(Trigger::LongSession)
    );
    assert_eq!(
        should_offer(false, false, 121, false),
        Some(Trigger::LongSession)
    );
}

#[test]
fn rule_o1_quiet_session_is_not_offered() {
    assert_eq!(should_offer(false, false, 0, false), None);
    assert_eq!(should_offer(false, false, 119, false), None);
}

#[test]
fn rule_c1_cause_labels() {
    assert_eq!(
        CAUSES,
        [
            Cause::MeasurementOff,
            Cause::MaterialUnexpected,
            Cause::ToolLimitation,
            Cause::DesignChanged,
            Cause::OutOfTime,
            Cause::Other,
        ]
    );
    assert_eq!(Cause::MeasurementOff.label(), "measurement was off");
    assert_eq!(
        Cause::MaterialUnexpected.label(),
        "material behaved unexpectedly"
    );
    assert_eq!(Cause::ToolLimitation.label(), "tool limitation");
    assert_eq!(Cause::DesignChanged.label(), "design changed");
    assert_eq!(Cause::OutOfTime.label(), "ran out of time");
    assert_eq!(Cause::Other.label(), "other");
}

#[test]
fn rule_c2_mood_keys() {
    assert_eq!(Mood::Worried.key(), "worried");
    assert_eq!(Mood::Fine.key(), "fine");
    assert_eq!(Mood::Excited.key(), "excited");
}

#[test]
fn rule_d1_each_done_becomes_a_phrase() {
    assert_eq!(
        draft(&[Done::Added {
            count: 4,
            what: "drawers".to_string(),
        }])
        .as_deref(),
        Some("You added 4 drawers.")
    );
    assert_eq!(
        draft(&[Done::Added {
            count: 12,
            what: "drawer".to_string(),
        }])
        .as_deref(),
        Some("You added 12 drawer.")
    );
    assert_eq!(
        draft(&[Done::Removed {
            count: 1,
            what: "shelf".to_string(),
        }])
        .as_deref(),
        Some("You removed 1 shelf.")
    );
    assert_eq!(
        draft(&[Done::Changed {
            what: "the top".to_string(),
            from: "pine".to_string(),
            to: "plywood".to_string(),
        }])
        .as_deref(),
        Some("You changed the top from pine to plywood.")
    );
    assert_eq!(
        draft(&[Done::Generated {
            what: "cut diagrams".to_string(),
        }])
        .as_deref(),
        Some("You generated cut diagrams.")
    );
}

#[test]
fn rule_d1_draft_of_nothing_is_none() {
    assert_eq!(draft(&[]), None);
}

#[test]
fn rule_d1_draft_of_one_two_three_things() {
    let a = Done::Generated {
        what: "a".to_string(),
    };
    let b = Done::Generated {
        what: "b".to_string(),
    };
    let c = Done::Generated {
        what: "c".to_string(),
    };
    assert_eq!(
        draft(std::slice::from_ref(&a)).as_deref(),
        Some("You generated a.")
    );
    assert_eq!(
        draft(&[a.clone(), b.clone()]).as_deref(),
        Some("You generated a and generated b.")
    );
    assert_eq!(
        draft(&[a, b, c]).as_deref(),
        Some("You generated a, generated b, and generated c.")
    );
}

#[test]
fn rule_d1_draft_of_six_things() {
    assert_eq!(
        draft(&generated(&["a", "b", "c", "d", "e", "f"])).as_deref(),
        Some(SIX_DRAFT)
    );
}

#[test]
fn rule_d1_draft_of_seven_things() {
    assert_eq!(
        draft(&generated(&["a", "b", "c", "d", "e", "f", "g"])).as_deref(),
        Some(SEVEN_DRAFT)
    );
}

#[test]
fn rule_d1_draft_of_twenty_things() {
    let names: Vec<String> = (1..=20).map(|n| format!("t{n}")).collect();
    let refs: Vec<&str> = names.iter().map(String::as_str).collect();
    assert_eq!(draft(&generated(&refs)).as_deref(), Some(TWENTY_DRAFT));
}

#[test]
fn rule_constants_match_the_contract() {
    assert_eq!(
        QUESTIONS,
        [
            Question::Did,
            Question::Worked,
            Question::Didnt,
            Question::Media,
            Question::Next,
            Question::Mood,
        ]
    );
    assert_eq!(LONG_SESSION_MINUTES, 120);
    assert_eq!(DRAFT_MAX, 6);
}

#[test]
fn rule_i1_new_stores_trigger_and_draft() {
    let done = example_done();
    let i = Interview::new(Trigger::LongSession, &done);
    assert_eq!(i.trigger(), Trigger::LongSession);
    assert_eq!(i.draft(), Some(EXAMPLE_DRAFT));
    assert_eq!(i.current(), Some(Question::Did));
    for question in QUESTIONS {
        assert_eq!(i.answer_of(question), &Answer::Unanswered);
    }
}

#[test]
fn rule_i1_new_with_no_done_has_no_draft() {
    let i = Interview::new(Trigger::Closing, &[]);
    assert_eq!(i.trigger(), Trigger::Closing);
    assert_eq!(i.draft(), None);
    assert_eq!(i.current(), Some(Question::Did));
}

#[test]
fn rule_a1_every_answer_kind_against_every_question() {
    let kinds: [Answer; 6] = [
        Answer::Unanswered,
        Answer::Skipped,
        Answer::Text("something".to_string()),
        Answer::Causes(vec![Cause::Other], String::new()),
        Answer::Media(vec![media("a.png")]),
        Answer::Mood(Mood::Fine),
    ];
    for question in QUESTIONS {
        for kind in &kinds {
            let mut i = fresh();
            let got = i.answer(question, kind.clone());
            let fits = match kind {
                Answer::Skipped => true,
                Answer::Text(_) => {
                    matches!(question, Question::Did | Question::Worked | Question::Next)
                }
                Answer::Causes(..) => question == Question::Didnt,
                Answer::Media(_) => question == Question::Media,
                Answer::Mood(_) => question == Question::Mood,
                Answer::Unanswered => false,
            };
            if fits {
                assert_eq!(got, Ok(()), "{question:?} should accept {kind:?}");
            } else {
                assert_eq!(
                    got,
                    Err(InterviewError::WrongKind(question)),
                    "{question:?} should refuse {kind:?}"
                );
            }
        }
    }
}

#[test]
fn rule_a1_refused_answer_changes_nothing() {
    let mut i = fresh();
    assert_eq!(
        i.answer(Question::Did, Answer::Text("cut the sides".to_string())),
        Ok(())
    );
    assert_eq!(
        i.answer(
            Question::Did,
            Answer::Causes(vec![Cause::Other], String::new())
        ),
        Err(InterviewError::WrongKind(Question::Did))
    );
    assert_eq!(
        i.answer(Question::Media, Answer::Text("a.png".to_string())),
        Err(InterviewError::WrongKind(Question::Media))
    );
    assert_eq!(
        i.answer(Question::Mood, Answer::Unanswered),
        Err(InterviewError::WrongKind(Question::Mood))
    );
    assert_eq!(
        i.answer_of(Question::Did),
        &Answer::Text("cut the sides".to_string())
    );
    assert_eq!(i.answer_of(Question::Media), &Answer::Unanswered);
    assert_eq!(i.answer_of(Question::Mood), &Answer::Unanswered);
    assert_eq!(i.current(), Some(Question::Worked));
}

#[test]
fn rule_a1_blank_text_is_stored_as_skipped() {
    let mut i = fresh();
    assert_eq!(i.answer(Question::Did, Answer::Text(String::new())), Ok(()));
    assert_eq!(i.answer_of(Question::Did), &Answer::Skipped);
    assert_eq!(
        i.answer(Question::Worked, Answer::Text("   ".to_string())),
        Ok(())
    );
    assert_eq!(i.answer_of(Question::Worked), &Answer::Skipped);
    assert_eq!(
        i.answer(Question::Next, Answer::Text("\n\t ".to_string())),
        Ok(())
    );
    assert_eq!(i.answer_of(Question::Next), &Answer::Skipped);
}

#[test]
fn rule_a1_text_is_stored_exactly_as_given() {
    let mut i = fresh();
    let padded = Answer::Text("  sanded the edges  ".to_string());
    assert_eq!(i.answer(Question::Did, padded.clone()), Ok(()));
    assert_eq!(i.answer_of(Question::Did), &padded);
    let crlf = Answer::Text("a\r\nb".to_string());
    assert_eq!(i.answer(Question::Worked, crlf.clone()), Ok(()));
    assert_eq!(i.answer_of(Question::Worked), &crlf);
}

#[test]
fn rule_a1_causes_drop_repeated_chips() {
    let mut i = fresh();
    assert_eq!(
        i.answer(
            Question::Didnt,
            Answer::Causes(
                vec![Cause::OutOfTime, Cause::MeasurementOff, Cause::OutOfTime],
                "the glue was slow\n".to_string(),
            ),
        ),
        Ok(())
    );
    assert_eq!(
        i.answer_of(Question::Didnt),
        &Answer::Causes(
            vec![Cause::OutOfTime, Cause::MeasurementOff],
            "the glue was slow\n".to_string(),
        )
    );
}

#[test]
fn rule_a1_causes_with_no_chips_and_no_note_is_skipped() {
    let mut i = fresh();
    assert_eq!(
        i.answer(Question::Didnt, Answer::Causes(vec![], String::new())),
        Ok(())
    );
    assert_eq!(i.answer_of(Question::Didnt), &Answer::Skipped);
    assert_eq!(
        i.answer(Question::Didnt, Answer::Causes(vec![], "   ".to_string())),
        Ok(())
    );
    assert_eq!(i.answer_of(Question::Didnt), &Answer::Skipped);
    assert_eq!(
        i.answer(
            Question::Didnt,
            Answer::Causes(vec![], "a note".to_string())
        ),
        Ok(())
    );
    assert_eq!(
        i.answer_of(Question::Didnt),
        &Answer::Causes(vec![], "a note".to_string())
    );
}

#[test]
fn rule_a1_media_with_no_refs_is_skipped() {
    let mut i = fresh();
    assert_eq!(i.answer(Question::Media, Answer::Media(vec![])), Ok(()));
    assert_eq!(i.answer_of(Question::Media), &Answer::Skipped);
}

#[test]
fn rule_a1_fitting_answer_replaces_the_earlier_one() {
    let mut i = fresh();
    assert_eq!(
        i.answer(Question::Did, Answer::Text("first".to_string())),
        Ok(())
    );
    assert_eq!(
        i.answer(Question::Did, Answer::Text("second".to_string())),
        Ok(())
    );
    assert_eq!(
        i.answer_of(Question::Did),
        &Answer::Text("second".to_string())
    );
    assert_eq!(
        i.answer(
            Question::Didnt,
            Answer::Causes(vec![Cause::Other], "x".to_string())
        ),
        Ok(())
    );
    assert_eq!(
        i.answer(
            Question::Didnt,
            Answer::Causes(vec![Cause::OutOfTime], "y".to_string())
        ),
        Ok(())
    );
    assert_eq!(
        i.answer_of(Question::Didnt),
        &Answer::Causes(vec![Cause::OutOfTime], "y".to_string())
    );
    assert_eq!(
        i.answer(Question::Media, Answer::Media(vec![media("a.png")])),
        Ok(())
    );
    assert_eq!(
        i.answer(Question::Media, Answer::Media(vec![media("b.png")])),
        Ok(())
    );
    assert_eq!(
        i.answer_of(Question::Media),
        &Answer::Media(vec![media("b.png")])
    );
}

#[test]
fn rule_a2_current_is_the_first_unanswered() {
    let mut i = fresh();
    assert_eq!(i.current(), Some(Question::Did));
    assert_eq!(i.answer(Question::Mood, Answer::Mood(Mood::Fine)), Ok(()));
    assert_eq!(i.current(), Some(Question::Did));
    assert_eq!(i.answer(Question::Did, Answer::Skipped), Ok(()));
    assert_eq!(i.current(), Some(Question::Worked));
    assert_eq!(i.answer(Question::Worked, Answer::Skipped), Ok(()));
    assert_eq!(i.current(), Some(Question::Didnt));
    assert_eq!(i.answer(Question::Didnt, Answer::Skipped), Ok(()));
    assert_eq!(i.current(), Some(Question::Media));
    assert_eq!(i.answer(Question::Media, Answer::Skipped), Ok(()));
    assert_eq!(i.current(), Some(Question::Next));
    assert_eq!(i.answer(Question::Next, Answer::Skipped), Ok(()));
    assert_eq!(i.current(), None);
}

#[test]
fn rule_a3_keep_going_is_the_next_text_trimmed() {
    let mut i = fresh();
    assert_eq!(i.keep_going(), None);
    assert_eq!(
        i.answer(
            Question::Next,
            Answer::Text("  Sand the drawer fronts\nBuy pulls  ".to_string()),
        ),
        Ok(())
    );
    assert_eq!(
        i.keep_going(),
        Some("Sand the drawer fronts\nBuy pulls".to_string())
    );
}

#[test]
fn rule_a3_keep_going_is_none_without_a_next_text() {
    let mut i = fresh();
    assert_eq!(i.keep_going(), None);
    assert_eq!(i.answer(Question::Next, Answer::Skipped), Ok(()));
    assert_eq!(i.keep_going(), None);
    let mut j = fresh();
    assert_eq!(
        j.answer(Question::Next, Answer::Text("   ".to_string())),
        Ok(())
    );
    assert_eq!(j.keep_going(), None);
}

#[test]
fn rule_f1_session_body_splits_texts_into_trimmed_lines() {
    let mut i = Interview::new(Trigger::Closing, &[]);
    assert_eq!(
        i.answer(
            Question::Did,
            Answer::Text("cut the sides\n\n  drilled  \n".to_string())
        ),
        Ok(())
    );
    assert_eq!(
        i.answer(
            Question::Worked,
            Answer::Text("the dado jig\r\nclamped it".to_string())
        ),
        Ok(())
    );
    assert_eq!(
        i.answer(
            Question::Didnt,
            Answer::Causes(vec![Cause::ToolLimitation], "the saw drifted\n".to_string()),
        ),
        Ok(())
    );
    assert_eq!(
        i.answer(Question::Next, Answer::Text("sand\nbuy pulls".to_string())),
        Ok(())
    );
    assert_eq!(
        i.answer(
            Question::Media,
            Answer::Media(vec![media("a.png"), media("b.mp4")])
        ),
        Ok(())
    );
    assert_eq!(
        i.answer(Question::Mood, Answer::Mood(Mood::Excited)),
        Ok(())
    );
    assert_eq!(
        i.finish(),
        vec![Body::Session {
            did: vec!["cut the sides".to_string(), "drilled".to_string()],
            worked: vec!["the dado jig".to_string(), "clamped it".to_string()],
            didnt: vec!["tool limitation".to_string(), "the saw drifted".to_string()],
            next: vec!["sand".to_string(), "buy pulls".to_string()],
            media: vec![media("a.png"), media("b.mp4")],
            mood: Some("excited".to_string()),
        }]
    );
}

#[test]
fn rule_f1_didnt_is_chips_then_note_lines() {
    let mut i = fresh();
    assert_eq!(
        i.answer(
            Question::Didnt,
            Answer::Causes(
                vec![Cause::OutOfTime, Cause::MeasurementOff],
                "the glue was slow\n\n  clamped late  ".to_string(),
            ),
        ),
        Ok(())
    );
    assert_eq!(
        i.finish(),
        vec![
            Body::Session {
                did: vec![],
                worked: vec![],
                didnt: vec![
                    "ran out of time".to_string(),
                    "measurement was off".to_string(),
                    "the glue was slow".to_string(),
                    "clamped late".to_string(),
                ],
                next: vec![],
                media: vec![],
                mood: None,
            },
            Body::OpenPage {
                prompt: PromptMoment::SessionEnd,
            },
        ]
    );
}

#[test]
fn rule_f1_media_and_mood() {
    let mut i = fresh();
    assert_eq!(
        i.answer(
            Question::Media,
            Answer::Media(vec![media("a.png"), media("b.mp4")])
        ),
        Ok(())
    );
    assert_eq!(i.answer(Question::Mood, Answer::Mood(Mood::Fine)), Ok(()));
    assert_eq!(
        i.finish(),
        vec![
            Body::Session {
                did: vec![],
                worked: vec![],
                didnt: vec![],
                next: vec![],
                media: vec![media("a.png"), media("b.mp4")],
                mood: Some("fine".to_string()),
            },
            Body::OpenPage {
                prompt: PromptMoment::SessionEnd,
            },
        ]
    );
}

#[test]
fn rule_f1_open_page_when_a_prompt_is_left_open() {
    let mut i = fresh();
    assert_eq!(
        i.answer(Question::Did, Answer::Text("cut the sides".to_string())),
        Ok(())
    );
    assert_eq!(i.answer(Question::Worked, Answer::Skipped), Ok(()));
    assert_eq!(i.answer(Question::Didnt, Answer::Skipped), Ok(()));
    assert_eq!(
        i.answer(Question::Next, Answer::Text("sand".to_string())),
        Ok(())
    );
    assert_eq!(i.answer(Question::Media, Answer::Skipped), Ok(()));
    assert_eq!(i.answer(Question::Mood, Answer::Skipped), Ok(()));
    assert_eq!(
        i.finish(),
        vec![
            Body::Session {
                did: vec!["cut the sides".to_string()],
                worked: vec![],
                didnt: vec![],
                next: vec!["sand".to_string()],
                media: vec![],
                mood: None,
            },
            Body::OpenPage {
                prompt: PromptMoment::SessionEnd,
            },
        ]
    );
}

#[test]
fn rule_f1_photos_and_mood_never_leave_a_page_open() {
    let mut i = fresh();
    assert_eq!(
        i.answer(Question::Did, Answer::Text("cut".to_string())),
        Ok(())
    );
    assert_eq!(
        i.answer(Question::Worked, Answer::Text("the jig".to_string())),
        Ok(())
    );
    assert_eq!(
        i.answer(
            Question::Didnt,
            Answer::Causes(vec![Cause::Other], String::new())
        ),
        Ok(())
    );
    assert_eq!(
        i.answer(Question::Next, Answer::Text("sand".to_string())),
        Ok(())
    );
    assert_eq!(i.answer(Question::Media, Answer::Skipped), Ok(()));
    assert_eq!(i.answer(Question::Mood, Answer::Skipped), Ok(()));
    assert_eq!(
        i.finish(),
        vec![Body::Session {
            did: vec!["cut".to_string()],
            worked: vec!["the jig".to_string()],
            didnt: vec!["other".to_string()],
            next: vec!["sand".to_string()],
            media: vec![],
            mood: None,
        }]
    );

    // The same, with photos and mood never touched at all.
    let mut j = fresh();
    assert_eq!(
        j.answer(Question::Did, Answer::Text("cut".to_string())),
        Ok(())
    );
    assert_eq!(
        j.answer(Question::Worked, Answer::Text("the jig".to_string())),
        Ok(())
    );
    assert_eq!(
        j.answer(
            Question::Didnt,
            Answer::Causes(vec![Cause::Other], String::new())
        ),
        Ok(())
    );
    assert_eq!(
        j.answer(Question::Next, Answer::Text("sand".to_string())),
        Ok(())
    );
    assert_eq!(
        j.finish(),
        vec![Body::Session {
            did: vec!["cut".to_string()],
            worked: vec!["the jig".to_string()],
            didnt: vec!["other".to_string()],
            next: vec!["sand".to_string()],
            media: vec![],
            mood: None,
        }]
    );
}

#[test]
fn rule_f1_all_skipped_gives_only_the_open_page() {
    let mut i = fresh();
    for question in QUESTIONS {
        assert_eq!(i.answer(question, Answer::Skipped), Ok(()));
    }
    assert_eq!(
        i.finish(),
        vec![Body::OpenPage {
            prompt: PromptMoment::SessionEnd,
        }]
    );
}

#[test]
fn rule_f1_all_unanswered_gives_only_the_open_page() {
    let i = fresh();
    assert_eq!(
        i.finish(),
        vec![Body::OpenPage {
            prompt: PromptMoment::SessionEnd,
        }]
    );
}

#[test]
fn rule_f1_finish_twice_gives_the_same_bodies() {
    let mut i = Interview::new(Trigger::WrapUp, &example_done());
    assert_eq!(
        i.answer(Question::Did, Answer::Text(EXAMPLE_DRAFT.to_string())),
        Ok(())
    );
    assert_eq!(
        i.answer(
            Question::Didnt,
            Answer::Causes(vec![Cause::OutOfTime], "slow glue".to_string()),
        ),
        Ok(())
    );
    assert_eq!(
        i.answer(Question::Next, Answer::Text("sand".to_string())),
        Ok(())
    );
    let first = i.finish();
    let second = i.finish();
    assert_eq!(first, second);
    assert_eq!(
        i.answer_of(Question::Did),
        &Answer::Text(EXAMPLE_DRAFT.to_string())
    );
    assert_eq!(i.current(), Some(Question::Worked));
    assert_eq!(i.keep_going(), Some("sand".to_string()));
}

#[test]
fn worked_example_should_offer_table() {
    assert_eq!(should_offer(true, false, 10, false), Some(Trigger::Closing));
    assert_eq!(
        should_offer(false, false, 120, false),
        Some(Trigger::LongSession)
    );
    assert_eq!(should_offer(false, false, 119, false), None);
    assert_eq!(should_offer(true, false, 200, true), None);
    assert_eq!(should_offer(true, true, 0, true), Some(Trigger::WrapUp));
}

#[test]
fn worked_example_draft_table() {
    let done = example_done();
    assert_eq!(draft(&done).as_deref(), Some(EXAMPLE_DRAFT));
    assert_eq!(
        draft(&done[..2]).as_deref(),
        Some("You added 4 drawers and changed the top from pine to plywood.")
    );
    assert_eq!(
        draft(&done[2..]).as_deref(),
        Some("You generated cut diagrams.")
    );
    assert_eq!(draft(&[]), None);
    assert_eq!(
        draft(&generated(&["a", "b", "c", "d", "e", "f", "g", "h"])).as_deref(),
        Some(EIGHT_DRAFT)
    );
}

#[test]
fn worked_example_interview_table() {
    let done = example_done();
    let mut i = Interview::new(Trigger::WrapUp, &done);

    // Row 1.
    assert_eq!(i.current(), Some(Question::Did));
    // Row 2.
    assert_eq!(
        i.answer(Question::Did, Answer::Text(EXAMPLE_DRAFT.to_string())),
        Ok(())
    );
    assert_eq!(i.current(), Some(Question::Worked));
    // Row 3: a Causes answer does not fit Worked, and nothing changes.
    assert_eq!(
        i.answer(
            Question::Worked,
            Answer::Causes(vec![Cause::Other], String::new())
        ),
        Err(InterviewError::WrongKind(Question::Worked))
    );
    assert_eq!(i.answer_of(Question::Worked), &Answer::Unanswered);
    assert_eq!(i.current(), Some(Question::Worked));
    // Row 4.
    assert_eq!(
        i.answer(Question::Worked, Answer::Text("  ".to_string())),
        Ok(())
    );
    assert_eq!(i.answer_of(Question::Worked), &Answer::Skipped);
    // Row 5.
    assert_eq!(
        i.answer(
            Question::Didnt,
            Answer::Causes(
                vec![Cause::OutOfTime, Cause::MeasurementOff, Cause::OutOfTime],
                "the glue was slow\n".to_string(),
            ),
        ),
        Ok(())
    );
    assert_eq!(
        i.answer_of(Question::Didnt),
        &Answer::Causes(
            vec![Cause::OutOfTime, Cause::MeasurementOff],
            "the glue was slow\n".to_string(),
        )
    );
    // Row 6.
    assert_eq!(i.answer(Question::Media, Answer::Media(vec![])), Ok(()));
    assert_eq!(i.answer_of(Question::Media), &Answer::Skipped);
    // Row 7.
    assert_eq!(
        i.answer(
            Question::Next,
            Answer::Text("Sand the drawer fronts\nBuy pulls".to_string()),
        ),
        Ok(())
    );
    assert_eq!(
        i.keep_going(),
        Some("Sand the drawer fronts\nBuy pulls".to_string())
    );
    // Row 8.
    assert_eq!(i.current(), Some(Question::Mood));
    // Row 9.
    assert_eq!(
        i.answer(Question::Mood, Answer::Unanswered),
        Err(InterviewError::WrongKind(Question::Mood))
    );
    // Row 10.
    assert_eq!(
        i.finish(),
        vec![
            Body::Session {
                did: vec![EXAMPLE_DRAFT.to_string()],
                worked: vec![],
                didnt: vec![
                    "ran out of time".to_string(),
                    "measurement was off".to_string(),
                    "the glue was slow".to_string(),
                ],
                next: vec![
                    "Sand the drawer fronts".to_string(),
                    "Buy pulls".to_string()
                ],
                media: vec![],
                mood: None,
            },
            Body::OpenPage {
                prompt: PromptMoment::SessionEnd,
            },
        ]
    );
    // Row 11.
    assert_eq!(
        i.answer(Question::Worked, Answer::Text("The dado jig".to_string())),
        Ok(())
    );
    assert_eq!(
        i.answer(Question::Mood, Answer::Mood(Mood::Excited)),
        Ok(())
    );
    assert_eq!(
        i.finish(),
        vec![Body::Session {
            did: vec![EXAMPLE_DRAFT.to_string()],
            worked: vec!["The dado jig".to_string()],
            didnt: vec![
                "ran out of time".to_string(),
                "measurement was off".to_string(),
                "the glue was slow".to_string(),
            ],
            next: vec![
                "Sand the drawer fronts".to_string(),
                "Buy pulls".to_string()
            ],
            media: vec![],
            mood: Some("excited".to_string()),
        }]
    );
    // Row 12.
    let mut all_skipped = Interview::new(Trigger::Closing, &done);
    for question in QUESTIONS {
        assert_eq!(all_skipped.answer(question, Answer::Skipped), Ok(()));
    }
    assert_eq!(
        all_skipped.finish(),
        vec![Body::OpenPage {
            prompt: PromptMoment::SessionEnd,
        }]
    );
    // Row 13.
    let untouched = Interview::new(Trigger::Closing, &done);
    assert_eq!(
        untouched.finish(),
        vec![Body::OpenPage {
            prompt: PromptMoment::SessionEnd,
        }]
    );
}
