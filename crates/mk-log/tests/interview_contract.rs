//! Contract tests for mk_log::interview, written by Qwen as a PA (the Crew) from
//! crew/contracts/mk_interview.md. They were the hidden tests of campaign mk-core-16.

use mk_log::interview::{
    draft, should_offer, Answer, Cause, Done, Interview, InterviewError, Mood, Question, Trigger,
    CAUSES, DRAFT_MAX, LONG_SESSION_MINUTES, QUESTIONS,
};
use mk_log::journal::{Body, MediaRef, PromptMoment};

fn media_ref(path: &str) -> MediaRef {
    MediaRef {
        path: path.to_string(),
        hash: None,
        external: false,
        clip: None,
    }
}

/// The worked example's command log.
fn done_list() -> Vec<Done> {
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

/// n Generated things, "thing0" .. "thing{n-1}".
fn gen(n: usize) -> Vec<Done> {
    (0..n)
        .map(|k| Done::Generated {
            what: format!("thing{}", k),
        })
        .collect()
}

// ---- O1 should_offer ------------------------------------------------------

#[test]
fn o1_wrap_up_always_offers() {
    assert_eq!(should_offer(false, true, 0, false), Some(Trigger::WrapUp));
    assert_eq!(should_offer(true, true, 0, true), Some(Trigger::WrapUp));
    assert_eq!(should_offer(false, true, 5, true), Some(Trigger::WrapUp));
}

#[test]
fn o1_already_offered_wins_over_closing_and_minutes() {
    assert_eq!(should_offer(true, false, 200, true), None);
    assert_eq!(should_offer(false, false, 999, true), None);
}

#[test]
fn o1_closing_offers_when_not_wrap_up_or_offered() {
    assert_eq!(should_offer(true, false, 10, false), Some(Trigger::Closing));
    assert_eq!(should_offer(true, false, 0, false), Some(Trigger::Closing));
}

#[test]
fn o1_long_session_at_and_above_threshold() {
    assert_eq!(LONG_SESSION_MINUTES, 120);
    assert_eq!(
        should_offer(false, false, LONG_SESSION_MINUTES, false),
        Some(Trigger::LongSession)
    );
    assert_eq!(
        should_offer(false, false, LONG_SESSION_MINUTES + 1, false),
        Some(Trigger::LongSession)
    );
}

#[test]
fn o1_below_threshold_offers_none() {
    assert_eq!(
        should_offer(false, false, LONG_SESSION_MINUTES - 1, false),
        None
    );
    assert_eq!(should_offer(false, false, 0, false), None);
}

// ---- C1 and C2 -------------------------------------------------------------

#[test]
fn c1_cause_labels_in_doc_order() {
    let labels: Vec<&str> = CAUSES.iter().map(|c| c.label()).collect();
    assert_eq!(
        labels,
        vec![
            "measurement was off",
            "material behaved unexpectedly",
            "tool limitation",
            "design changed",
            "ran out of time",
            "other",
        ]
    );
}

#[test]
fn c2_mood_keys() {
    assert_eq!(Mood::Worried.key(), "worried");
    assert_eq!(Mood::Fine.key(), "fine");
    assert_eq!(Mood::Excited.key(), "excited");
}

// ---- D1 draft ---------------------------------------------------------------

#[test]
fn d1_draft_zero_things_is_none() {
    assert_eq!(draft(&[]), None);
}

#[test]
fn d1_draft_one_thing() {
    assert_eq!(
        draft(&[Done::Added {
            count: 1,
            what: "drawer".to_string()
        }]),
        Some("You added 1 drawer.".to_string())
    );
}

#[test]
fn d1_draft_two_things() {
    let done = vec![
        Done::Added {
            count: 2,
            what: "drawers".to_string(),
        },
        Done::Removed {
            count: 3,
            what: "screws".to_string(),
        },
    ];
    assert_eq!(
        draft(&done),
        Some("You added 2 drawers and removed 3 screws.".to_string())
    );
}

#[test]
fn d1_draft_three_things() {
    let done = vec![
        Done::Added {
            count: 1,
            what: "drawer".to_string(),
        },
        Done::Removed {
            count: 2,
            what: "screws".to_string(),
        },
        Done::Changed {
            what: "the top".to_string(),
            from: "pine".to_string(),
            to: "plywood".to_string(),
        },
    ];
    assert_eq!(
        draft(&done),
        Some(
            "You added 1 drawer, removed 2 screws, and changed the top from pine to plywood."
                .to_string()
        )
    );
}

#[test]
fn d1_draft_each_kind_phrases() {
    let done = vec![
        Done::Added {
            count: 4,
            what: "drawers".to_string(),
        },
        Done::Removed {
            count: 1,
            what: "bracket".to_string(),
        },
        Done::Changed {
            what: "the top".to_string(),
            from: "pine".to_string(),
            to: "plywood".to_string(),
        },
        Done::Generated {
            what: "cut diagrams".to_string(),
        },
    ];
    assert_eq!(
        draft(&done),
        Some(
            "You added 4 drawers, removed 1 bracket, changed the top from pine to plywood, and generated cut diagrams."
                .to_string()
        )
    );
}

#[test]
fn d1_draft_six_things_names_all() {
    assert_eq!(DRAFT_MAX, 6);
    assert_eq!(
        draft(&gen(6)),
        Some(
            "You generated thing0, generated thing1, generated thing2, generated thing3, generated thing4, and generated thing5."
                .to_string()
        )
    );
}

#[test]
fn d1_draft_seven_things_says_two_more() {
    assert_eq!(
        draft(&gen(7)),
        Some(
            "You generated thing0, generated thing1, generated thing2, generated thing3, generated thing4, and 2 more things."
                .to_string()
        )
    );
}

#[test]
fn d1_draft_twenty_things_says_fifteen_more() {
    assert_eq!(
        draft(&gen(20)),
        Some(
            "You generated thing0, generated thing1, generated thing2, generated thing3, generated thing4, and 15 more things."
                .to_string()
        )
    );
}

// ---- I1 Interview::new -------------------------------------------------------

#[test]
fn i1_new_stores_trigger_draft_and_unanswered() {
    let done = done_list();
    let i = Interview::new(Trigger::WrapUp, &done);
    assert_eq!(i.trigger(), Trigger::WrapUp);
    assert_eq!(
        i.draft(),
        Some(
            "You added 4 drawers, changed the top from pine to plywood, and generated cut diagrams."
        )
    );
    for q in QUESTIONS {
        assert_eq!(i.answer_of(q), &Answer::Unanswered);
    }
    assert_eq!(i.current(), Some(Question::Did));

    let empty = Interview::new(Trigger::Closing, &[]);
    assert_eq!(empty.trigger(), Trigger::Closing);
    assert_eq!(empty.draft(), None);
    assert_eq!(empty.current(), Some(Question::Did));
}

// ---- A1 answer ----------------------------------------------------------------

#[test]
fn a1_every_answer_kind_against_every_question() {
    let answers = [
        Answer::Unanswered,
        Answer::Skipped,
        Answer::Text("t".to_string()),
        Answer::Causes(vec![Cause::Other], "n".to_string()),
        Answer::Media(vec![media_ref("p")]),
        Answer::Mood(Mood::Fine),
    ];
    for q in QUESTIONS {
        for a in &answers {
            let expected = match a {
                Answer::Unanswered => Err(InterviewError::WrongKind(q)),
                Answer::Skipped => Ok(()),
                Answer::Text(_) => {
                    if matches!(q, Question::Did | Question::Worked | Question::Next) {
                        Ok(())
                    } else {
                        Err(InterviewError::WrongKind(q))
                    }
                }
                Answer::Causes(..) => {
                    if q == Question::Didnt {
                        Ok(())
                    } else {
                        Err(InterviewError::WrongKind(q))
                    }
                }
                Answer::Media(_) => {
                    if q == Question::Media {
                        Ok(())
                    } else {
                        Err(InterviewError::WrongKind(q))
                    }
                }
                Answer::Mood(_) => {
                    if q == Question::Mood {
                        Ok(())
                    } else {
                        Err(InterviewError::WrongKind(q))
                    }
                }
            };
            let mut i = Interview::new(Trigger::Closing, &[]);
            assert_eq!(i.answer(q, a.clone()), expected);
        }
    }
}

#[test]
fn a1_refused_answer_changes_nothing() {
    let mut i = Interview::new(Trigger::WrapUp, &done_list());
    assert_eq!(
        i.answer(Question::Did, Answer::Text("did".to_string())),
        Ok(())
    );
    let before = i.clone();
    assert_eq!(
        i.answer(
            Question::Worked,
            Answer::Causes(vec![Cause::Other], String::new())
        ),
        Err(InterviewError::WrongKind(Question::Worked))
    );
    assert_eq!(
        i.answer(Question::Media, Answer::Mood(Mood::Fine)),
        Err(InterviewError::WrongKind(Question::Media))
    );
    assert_eq!(
        i.answer(Question::Next, Answer::Unanswered),
        Err(InterviewError::WrongKind(Question::Next))
    );
    assert_eq!(i, before);
    assert_eq!(i.current(), Some(Question::Worked));
    assert_eq!(i.keep_going(), None);
}

#[test]
fn a1_text_empty_after_trim_is_skipped() {
    let mut i = Interview::new(Trigger::Closing, &[]);
    assert_eq!(
        i.answer(Question::Did, Answer::Text("   ".to_string())),
        Ok(())
    );
    assert_eq!(i.answer_of(Question::Did), &Answer::Skipped);
    assert_eq!(
        i.answer(Question::Worked, Answer::Text("\r\n \n".to_string())),
        Ok(())
    );
    assert_eq!(i.answer_of(Question::Worked), &Answer::Skipped);
    assert_eq!(
        i.answer(Question::Next, Answer::Text(String::new())),
        Ok(())
    );
    assert_eq!(i.answer_of(Question::Next), &Answer::Skipped);
}

#[test]
fn a1_text_stored_exactly_as_given() {
    let mut i = Interview::new(Trigger::Closing, &[]);
    let t = "  kept  as-is  \n";
    assert_eq!(i.answer(Question::Did, Answer::Text(t.to_string())), Ok(()));
    assert_eq!(i.answer_of(Question::Did), &Answer::Text(t.to_string()));
}

#[test]
fn a1_causes_drops_repeated_chips_keeps_first() {
    let mut i = Interview::new(Trigger::Closing, &[]);
    assert_eq!(
        i.answer(
            Question::Didnt,
            Answer::Causes(
                vec![
                    Cause::OutOfTime,
                    Cause::MeasurementOff,
                    Cause::OutOfTime,
                    Cause::Other,
                    Cause::MeasurementOff,
                ],
                "the glue was slow\n".to_string(),
            ),
        ),
        Ok(())
    );
    assert_eq!(
        i.answer_of(Question::Didnt),
        &Answer::Causes(
            vec![Cause::OutOfTime, Cause::MeasurementOff, Cause::Other],
            "the glue was slow\n".to_string(),
        )
    );
}

#[test]
fn a1_causes_no_chips_and_blank_note_is_skipped() {
    let mut i = Interview::new(Trigger::Closing, &[]);
    assert_eq!(
        i.answer(Question::Didnt, Answer::Causes(vec![], "   ".to_string())),
        Ok(())
    );
    assert_eq!(i.answer_of(Question::Didnt), &Answer::Skipped);
    assert_eq!(
        i.answer(Question::Didnt, Answer::Causes(vec![], String::new())),
        Ok(())
    );
    assert_eq!(i.answer_of(Question::Didnt), &Answer::Skipped);
}

#[test]
fn a1_causes_no_chips_with_note_is_kept() {
    let mut i = Interview::new(Trigger::Closing, &[]);
    assert_eq!(
        i.answer(
            Question::Didnt,
            Answer::Causes(vec![], "the glue was slow".to_string())
        ),
        Ok(())
    );
    assert_eq!(
        i.answer_of(Question::Didnt),
        &Answer::Causes(vec![], "the glue was slow".to_string())
    );
}

#[test]
fn a1_media_without_refs_is_skipped() {
    let mut i = Interview::new(Trigger::Closing, &[]);
    assert_eq!(i.answer(Question::Media, Answer::Media(vec![])), Ok(()));
    assert_eq!(i.answer_of(Question::Media), &Answer::Skipped);
    let refs = vec![media_ref("a.jpg"), media_ref("b.mp4")];
    assert_eq!(
        i.answer(Question::Media, Answer::Media(refs.clone())),
        Ok(())
    );
    assert_eq!(i.answer_of(Question::Media), &Answer::Media(refs));
}

#[test]
fn a1_a_fitting_answer_replaces_the_earlier_one() {
    let mut i = Interview::new(Trigger::Closing, &[]);
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
        i.answer(Question::Mood, Answer::Mood(Mood::Worried)),
        Ok(())
    );
    assert_eq!(
        i.answer(Question::Mood, Answer::Mood(Mood::Excited)),
        Ok(())
    );
    assert_eq!(i.answer_of(Question::Mood), &Answer::Mood(Mood::Excited));
    // a Skipped replaces a real answer too
    assert_eq!(i.answer(Question::Did, Answer::Skipped), Ok(()));
    assert_eq!(i.answer_of(Question::Did), &Answer::Skipped);
}

// ---- A2 current -----------------------------------------------------------------

#[test]
fn a2_current_walks_questions_in_order() {
    let mut i = Interview::new(Trigger::Closing, &[]);
    assert_eq!(i.current(), Some(Question::Did));
    assert_eq!(
        i.answer(Question::Did, Answer::Text("d".to_string())),
        Ok(())
    );
    assert_eq!(i.current(), Some(Question::Worked));
    assert_eq!(i.answer(Question::Worked, Answer::Skipped), Ok(()));
    assert_eq!(i.current(), Some(Question::Didnt));
    assert_eq!(
        i.answer(
            Question::Didnt,
            Answer::Causes(vec![Cause::Other], "n".to_string())
        ),
        Ok(())
    );
    assert_eq!(i.current(), Some(Question::Media));
    assert_eq!(i.answer(Question::Media, Answer::Skipped), Ok(()));
    assert_eq!(i.current(), Some(Question::Next));
    assert_eq!(
        i.answer(Question::Next, Answer::Text("n".to_string())),
        Ok(())
    );
    assert_eq!(i.current(), Some(Question::Mood));
    assert_eq!(i.answer(Question::Mood, Answer::Mood(Mood::Fine)), Ok(()));
    assert_eq!(i.current(), None);
}

#[test]
fn a2_current_is_none_when_all_answered() {
    let mut i = Interview::new(Trigger::Closing, &[]);
    for q in QUESTIONS {
        assert_eq!(i.answer(q, Answer::Skipped), Ok(()));
    }
    assert_eq!(i.current(), None);
}

// ---- A3 keep_going -----------------------------------------------------------------

#[test]
fn a3_keep_going_is_next_text_trimmed() {
    let mut i = Interview::new(Trigger::Closing, &[]);
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
fn a3_keep_going_is_none_when_next_is_not_text() {
    let mut i = Interview::new(Trigger::Closing, &[]);
    assert_eq!(i.answer(Question::Next, Answer::Skipped), Ok(()));
    assert_eq!(i.keep_going(), None);
    let mut j = Interview::new(Trigger::Closing, &[]);
    assert_eq!(
        j.answer(Question::Next, Answer::Text("x".to_string())),
        Ok(())
    );
    assert_eq!(j.keep_going(), Some("x".to_string()));
    assert_eq!(j.answer(Question::Next, Answer::Skipped), Ok(()));
    assert_eq!(j.keep_going(), None);
}

// ---- F1 finish ----------------------------------------------------------------------

#[test]
fn f1_session_body_from_answers() {
    let mut i = Interview::new(Trigger::WrapUp, &done_list());
    assert_eq!(
        i.answer(Question::Did, Answer::Text("built the case".to_string())),
        Ok(())
    );
    assert_eq!(
        i.answer(Question::Worked, Answer::Text("The dado jig".to_string())),
        Ok(())
    );
    assert_eq!(
        i.answer(
            Question::Didnt,
            Answer::Causes(vec![Cause::ToolLimitation], "the bit was dull".to_string()),
        ),
        Ok(())
    );
    let refs = vec![media_ref("a.jpg"), media_ref("b.mp4")];
    assert_eq!(
        i.answer(Question::Media, Answer::Media(refs.clone())),
        Ok(())
    );
    assert_eq!(
        i.answer(Question::Next, Answer::Text("buy pulls".to_string())),
        Ok(())
    );
    assert_eq!(
        i.answer(Question::Mood, Answer::Mood(Mood::Excited)),
        Ok(())
    );
    assert_eq!(
        i.finish(),
        vec![Body::Session {
            did: vec!["built the case".to_string()],
            worked: vec!["The dado jig".to_string()],
            didnt: vec![
                "tool limitation".to_string(),
                "the bit was dull".to_string()
            ],
            next: vec!["buy pulls".to_string()],
            media: refs,
            mood: Some("excited".to_string()),
        }]
    );
}

#[test]
fn f1_text_lines_split_trim_drop_blanks_and_crlf() {
    let mut i = Interview::new(Trigger::Closing, &[]);
    assert_eq!(
        i.answer(
            Question::Did,
            Answer::Text("first\r\n\r\n  second  \n\nthird\r\n".to_string()),
        ),
        Ok(())
    );
    assert_eq!(
        i.answer(
            Question::Worked,
            Answer::Text("  \r\nonly\r\n  ".to_string())
        ),
        Ok(())
    );
    let bodies = i.finish();
    assert_eq!(bodies.len(), 2);
    match &bodies[0] {
        Body::Session { did, worked, .. } => {
            assert_eq!(
                *did,
                vec![
                    "first".to_string(),
                    "second".to_string(),
                    "third".to_string(),
                ]
            );
            assert_eq!(*worked, vec!["only".to_string()]);
        }
        other => panic!("expected a Session body first, got {:?}", other),
    }
}

#[test]
fn f1_didnt_is_chip_labels_then_note_lines() {
    let mut i = Interview::new(Trigger::Closing, &[]);
    assert_eq!(
        i.answer(
            Question::Didnt,
            Answer::Causes(
                vec![Cause::OutOfTime, Cause::Other],
                "the glue was slow\r\n\r\nand the bit was dull\n".to_string(),
            ),
        ),
        Ok(())
    );
    let bodies = i.finish();
    match &bodies[0] {
        Body::Session { didnt, .. } => {
            assert_eq!(
                *didnt,
                vec![
                    "ran out of time".to_string(),
                    "other".to_string(),
                    "the glue was slow".to_string(),
                    "and the bit was dull".to_string(),
                ]
            );
        }
        other => panic!("expected a Session body first, got {:?}", other),
    }
}

#[test]
fn f1_open_page_when_any_of_the_four_is_open() {
    let mut i = Interview::new(Trigger::Closing, &[]);
    assert_eq!(
        i.answer(Question::Next, Answer::Text("buy pulls".to_string())),
        Ok(())
    );
    let bodies = i.finish();
    assert_eq!(bodies.len(), 2);
    assert!(matches!(&bodies[0], Body::Session { .. }));
    assert_eq!(
        bodies[1],
        Body::OpenPage {
            prompt: PromptMoment::SessionEnd
        }
    );
}

#[test]
fn f1_no_open_page_when_the_four_are_answered() {
    let mut i = Interview::new(Trigger::Closing, &[]);
    assert_eq!(
        i.answer(Question::Did, Answer::Text("x".to_string())),
        Ok(())
    );
    assert_eq!(
        i.answer(Question::Worked, Answer::Text("y".to_string())),
        Ok(())
    );
    assert_eq!(
        i.answer(
            Question::Didnt,
            Answer::Causes(vec![Cause::Other], "z".to_string())
        ),
        Ok(())
    );
    assert_eq!(
        i.answer(Question::Next, Answer::Text("w".to_string())),
        Ok(())
    );
    // photos and mood are optional: Skipped Media and a Mood leave no page open
    assert_eq!(i.answer(Question::Media, Answer::Skipped), Ok(()));
    assert_eq!(i.answer(Question::Mood, Answer::Mood(Mood::Fine)), Ok(()));
    assert_eq!(
        i.finish(),
        vec![Body::Session {
            did: vec!["x".to_string()],
            worked: vec!["y".to_string()],
            didnt: vec!["other".to_string(), "z".to_string()],
            next: vec!["w".to_string()],
            media: vec![],
            mood: Some("fine".to_string()),
        }]
    );
}

#[test]
fn f1_all_skipped_gives_only_the_open_page() {
    let mut i = Interview::new(Trigger::Closing, &done_list());
    for q in QUESTIONS {
        assert_eq!(i.answer(q, Answer::Skipped), Ok(()));
    }
    assert_eq!(
        i.finish(),
        vec![Body::OpenPage {
            prompt: PromptMoment::SessionEnd
        }]
    );
}

#[test]
fn f1_all_unanswered_gives_only_the_open_page() {
    let i = Interview::new(Trigger::Closing, &done_list());
    assert_eq!(
        i.finish(),
        vec![Body::OpenPage {
            prompt: PromptMoment::SessionEnd
        }]
    );
}

#[test]
fn f1_finish_twice_gives_the_same_bodies() {
    let mut i = Interview::new(Trigger::WrapUp, &done_list());
    assert_eq!(
        i.answer(Question::Did, Answer::Text("a\nb".to_string())),
        Ok(())
    );
    assert_eq!(
        i.answer(
            Question::Didnt,
            Answer::Causes(vec![Cause::Other], "n".to_string())
        ),
        Ok(())
    );
    assert_eq!(
        i.answer(Question::Mood, Answer::Mood(Mood::Worried)),
        Ok(())
    );
    assert_eq!(i.finish(), i.finish());
    // and the interview is unchanged
    assert_eq!(i.current(), Some(Question::Worked));
    assert_eq!(i.keep_going(), None);
    assert_eq!(i.draft(), Some(
        "You added 4 drawers, changed the top from pine to plywood, and generated cut diagrams."
    ));
}

// ---- worked examples ---------------------------------------------------------------

#[test]
fn worked_examples_should_offer_table() {
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
fn worked_examples_draft_table() {
    let done = done_list();
    assert_eq!(
        draft(&done),
        Some(
            "You added 4 drawers, changed the top from pine to plywood, and generated cut diagrams."
                .to_string()
        )
    );
    assert_eq!(
        draft(&done[..2]),
        Some("You added 4 drawers and changed the top from pine to plywood.".to_string())
    );
    assert_eq!(
        draft(&done[2..]),
        Some("You generated cut diagrams.".to_string())
    );
    assert_eq!(draft(&[]), None);
    let eight: Vec<Done> = (0..8)
        .map(|n| Done::Generated {
            what: char::from(b'a' + n as u8).to_string(),
        })
        .collect();
    assert_eq!(
        draft(&eight),
        Some(
            "You generated a, generated b, generated c, generated d, generated e, and 3 more things."
                .to_string()
        )
    );
}

#[test]
fn worked_examples_interview_table_in_order() {
    let done = done_list();
    let draft_text =
        "You added 4 drawers, changed the top from pine to plywood, and generated cut diagrams.";
    let mut i = Interview::new(Trigger::WrapUp, &done);

    // row 1
    assert_eq!(i.current(), Some(Question::Did));

    // row 2
    assert_eq!(
        i.answer(Question::Did, Answer::Text(draft_text.to_string())),
        Ok(())
    );
    assert_eq!(i.current(), Some(Question::Worked));

    // row 3: a Causes answer does not fit Worked; nothing changes
    assert_eq!(
        i.answer(
            Question::Worked,
            Answer::Causes(vec![Cause::Other], String::new())
        ),
        Err(InterviewError::WrongKind(Question::Worked))
    );
    assert_eq!(i.answer_of(Question::Worked), &Answer::Unanswered);
    assert_eq!(i.current(), Some(Question::Worked));

    // row 4
    assert_eq!(
        i.answer(Question::Worked, Answer::Text("  ".to_string())),
        Ok(())
    );
    assert_eq!(i.answer_of(Question::Worked), &Answer::Skipped);

    // row 5
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

    // row 6
    assert_eq!(i.answer(Question::Media, Answer::Media(vec![])), Ok(()));
    assert_eq!(i.answer_of(Question::Media), &Answer::Skipped);

    // row 7
    assert_eq!(
        i.answer(
            Question::Next,
            Answer::Text("Sand the drawer fronts\nBuy pulls".to_string())
        ),
        Ok(())
    );
    assert_eq!(
        i.keep_going(),
        Some("Sand the drawer fronts\nBuy pulls".to_string())
    );

    // row 8
    assert_eq!(i.current(), Some(Question::Mood));

    // row 9
    assert_eq!(
        i.answer(Question::Mood, Answer::Unanswered),
        Err(InterviewError::WrongKind(Question::Mood))
    );

    // row 10
    assert_eq!(
        i.finish(),
        vec![
            Body::Session {
                did: vec![draft_text.to_string()],
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
                prompt: PromptMoment::SessionEnd
            },
        ]
    );

    // row 11
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
            did: vec![draft_text.to_string()],
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
}

#[test]
fn worked_examples_rows_twelve_and_thirteen() {
    // row 12: a new interview with every question Skipped
    let mut i = Interview::new(Trigger::Closing, &done_list());
    for q in QUESTIONS {
        assert_eq!(i.answer(q, Answer::Skipped), Ok(()));
    }
    assert_eq!(
        i.finish(),
        vec![Body::OpenPage {
            prompt: PromptMoment::SessionEnd
        }]
    );

    // row 13: a new interview, nothing answered
    let j = Interview::new(Trigger::LongSession, &done_list());
    assert_eq!(
        j.finish(),
        vec![Body::OpenPage {
            prompt: PromptMoment::SessionEnd
        }]
    );
}

// ---- odd input -----------------------------------------------------------------------

#[test]
fn odd_input_never_panics() {
    let odd = vec![
        Done::Added {
            count: 0,
            what: String::new(),
        },
        Done::Added {
            count: u32::MAX,
            what: "x".repeat(1000),
        },
        Done::Removed {
            count: 7,
            what: "   ".to_string(),
        },
        Done::Changed {
            what: String::new(),
            from: "a\nb".to_string(),
            to: "c\r\nd".to_string(),
        },
        Done::Generated {
            what: "émoji 🪵".to_string(),
        },
    ];
    assert!(draft(&odd).is_some());
    assert!(draft(&gen(1000)).is_some());

    let mut i = Interview::new(Trigger::LongSession, &odd);
    assert_eq!(
        i.answer(Question::Did, Answer::Text("\r\n \n".to_string())),
        Ok(())
    );
    assert_eq!(
        i.answer(Question::Worked, Answer::Text(String::new())),
        Ok(())
    );
    assert_eq!(
        i.answer(Question::Didnt, Answer::Causes(vec![], "   ".to_string())),
        Ok(())
    );
    assert_eq!(i.answer(Question::Media, Answer::Media(vec![])), Ok(()));
    assert_eq!(
        i.answer(Question::Next, Answer::Text("  \r\n  ".to_string())),
        Ok(())
    );
    assert_eq!(
        i.answer(Question::Mood, Answer::Mood(Mood::Worried)),
        Ok(())
    );
    let bodies = i.finish();
    assert_eq!(bodies, i.finish());
    assert_eq!(
        bodies,
        vec![
            Body::Session {
                did: vec![],
                worked: vec![],
                didnt: vec![],
                next: vec![],
                media: vec![],
                mood: Some("worried".to_string()),
            },
            Body::OpenPage {
                prompt: PromptMoment::SessionEnd
            },
        ]
    );
}
