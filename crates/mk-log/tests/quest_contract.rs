//! Contract tests for mk_log::quest, written by Qwen as a PA (the Crew) from crew/contracts/mk_quest.md;
//! the hidden tests of campaign mk-core-10 (one test corrected by Claude).

use mk_log::quest::{
    class_for_size, steps, Answer, AnswerKind, FitPick, Quest, QuestError, ScaleClass, Step,
    StepId, Template,
};

fn st(id: StepId, question: &'static str, kind: AnswerKind) -> Step {
    Step { id, question, kind }
}

fn base_nine() -> Vec<Step> {
    vec![
        st(StepId::Name, "What shall we call it?", AnswerKind::Text),
        st(
            StepId::Goal,
            "What would you like to make?",
            AnswerKind::Text,
        ),
        st(
            StepId::Fit,
            "Where does it need to fit? How big is it?",
            AnswerKind::Fit,
        ),
        st(StepId::Tools, "What tools do you have?", AnswerKind::Items),
        st(
            StepId::Materials,
            "What materials do you have?",
            AnswerKind::Items,
        ),
        st(
            StepId::Deadline,
            "When do you need it?",
            AnswerKind::Deadline,
        ),
        st(StepId::Budget, "What's your budget?", AnswerKind::Budget),
        st(StepId::Inspiration, "What inspires you?", AnswerKind::Items),
        st(StepId::Review, "Here's your workspace", AnswerKind::Review),
    ]
}

fn packaging_eleven() -> Vec<Step> {
    vec![
        st(StepId::Name, "What shall we call it?", AnswerKind::Text),
        st(
            StepId::Goal,
            "What would you like to make?",
            AnswerKind::Text,
        ),
        st(
            StepId::Fit,
            "Where does it need to fit? How big is it?",
            AnswerKind::Fit,
        ),
        st(StepId::Tools, "What tools do you have?", AnswerKind::Items),
        st(
            StepId::Materials,
            "What materials do you have?",
            AnswerKind::Items,
        ),
        st(
            StepId::Deadline,
            "When do you need it?",
            AnswerKind::Deadline,
        ),
        st(StepId::Budget, "What's your budget?", AnswerKind::Budget),
        st(StepId::Inspiration, "What inspires you?", AnswerKind::Items),
        st(
            StepId::BoxContents,
            "What goes in the box?",
            AnswerKind::Text,
        ),
        st(
            StepId::HowMany,
            "How many will you make?",
            AnswerKind::Count,
        ),
        st(StepId::Review, "Here's your workspace", AnswerKind::Review),
    ]
}

/// A3: which answers fit which kind, as the contract states it.
fn a3_ok(kind: AnswerKind, answer: &Answer) -> bool {
    match (kind, answer) {
        (_, Answer::Skipped) => kind != AnswerKind::Review,
        (AnswerKind::Text, Answer::Text(_)) => true,
        (AnswerKind::Fit, Answer::Fit(_)) => true,
        (AnswerKind::Items, Answer::Items(_)) => true,
        (AnswerKind::Deadline, Answer::Deadline(_)) => true,
        (AnswerKind::Budget, Answer::BudgetCap(_, _)) => true,
        (AnswerKind::Budget, Answer::BudgetOnlyWhatIHave) => true,
        (AnswerKind::Budget, Answer::BudgetNotSure) => true,
        (AnswerKind::Count, Answer::Count(_)) => true,
        (AnswerKind::Review, Answer::Confirmed) => true,
        _ => false,
    }
}

// ---- T1-T2: the exact steps of every template --------------------------------

#[test]
fn t1_default_steps_exact() {
    assert_eq!(steps(Template::Default), base_nine());
}

#[test]
fn t1_softgoods_steps_exact() {
    assert_eq!(steps(Template::SoftGoods), base_nine());
}

#[test]
fn t1_room_steps_exact() {
    assert_eq!(steps(Template::Room), base_nine());
}

#[test]
fn t2_packaging_steps_exact() {
    assert_eq!(steps(Template::Packaging), packaging_eleven());
}

#[test]
fn quest_steps_match_template_steps() {
    for template in [
        Template::Default,
        Template::Packaging,
        Template::SoftGoods,
        Template::Room,
    ] {
        let q = Quest::new(template);
        assert_eq!(q.steps().to_vec(), steps(template));
    }
}

// ---- A1-A5: recording answers -------------------------------------------------

#[test]
fn a1_not_in_this_quest() {
    let mut q = Quest::new(Template::Default);
    assert_eq!(
        q.answer(StepId::BoxContents, Answer::Text("socks".to_string())),
        Err(QuestError::NotInThisQuest(StepId::BoxContents))
    );
    assert_eq!(
        q.answer(StepId::HowMany, Answer::Count(1)),
        Err(QuestError::NotInThisQuest(StepId::HowMany))
    );
    // The reverse: both steps exist in the Packaging quest.
    let mut p = Quest::new(Template::Packaging);
    assert!(p
        .answer(StepId::BoxContents, Answer::Text("socks".to_string()))
        .is_ok());
    assert!(p.answer(StepId::HowMany, Answer::Count(1)).is_ok());
}

#[test]
fn a2_already_confirmed() {
    let mut q = Quest::new(Template::Default);
    assert!(q.answer(StepId::Review, Answer::Confirmed).is_ok());
    assert_eq!(
        q.answer(StepId::Review, Answer::Confirmed),
        Err(QuestError::AlreadyConfirmed)
    );
}

#[test]
fn a2_already_confirmed_beats_wrong_kind() {
    // A2 is checked before A3: once confirmed, even a kind-mismatched answer
    // is refused as AlreadyConfirmed.
    let mut q = Quest::new(Template::Default);
    let _ = q.answer(StepId::Review, Answer::Confirmed);
    assert_eq!(
        q.answer(StepId::Review, Answer::Skipped),
        Err(QuestError::AlreadyConfirmed)
    );
    assert_eq!(
        q.answer(StepId::Review, Answer::Text("x".to_string())),
        Err(QuestError::AlreadyConfirmed)
    );
}

#[test]
fn a3_every_answer_against_every_step_kind() {
    let answers = vec![
        Answer::Skipped,
        Answer::Text("x".to_string()),
        Answer::Fit(FitPick::TypeSizes),
        Answer::Items(1),
        Answer::Deadline(Some((2026, 12, 25))),
        Answer::Deadline(None),
        Answer::BudgetCap(100, "USD".to_string()),
        Answer::BudgetOnlyWhatIHave,
        Answer::BudgetNotSure,
        Answer::Count(3),
        Answer::Confirmed,
    ];
    for template in [
        Template::Default,
        Template::Packaging,
        Template::SoftGoods,
        Template::Room,
    ] {
        for step in steps(template) {
            for answer in &answers {
                let mut q = Quest::new(template);
                let got = q.answer(step.id.clone(), answer.clone());
                if a3_ok(step.kind, answer) {
                    assert!(got.is_ok(), "{:?} {:?} {:?}", template, step.id, answer);
                } else {
                    assert_eq!(got, Err(QuestError::WrongKind(step.id.clone())));
                }
            }
        }
    }
}

#[test]
fn a4_replacing_an_answer() {
    let mut q = Quest::new(Template::Default);
    let _ = q.answer(StepId::Goal, Answer::Text("first".to_string()));
    let _ = q.answer(StepId::Goal, Answer::Text("second".to_string()));
    assert_eq!(
        q.answer_for(&StepId::Goal),
        Some(&Answer::Text("second".to_string()))
    );
}

#[test]
fn a4_skipped_step_can_be_revisited() {
    let mut q = Quest::new(Template::Default);
    let _ = q.answer(StepId::Goal, Answer::Skipped);
    assert!(q.intent_missing());
    let _ = q.answer(
        StepId::Goal,
        Answer::Text("A four-drawer dresser".to_string()),
    );
    assert!(!q.intent_missing());
    assert_eq!(
        q.outstanding(),
        vec![
            StepId::Name,
            StepId::Fit,
            StepId::Tools,
            StepId::Materials,
            StepId::Deadline,
            StepId::Budget,
            StepId::Inspiration
        ]
    );
}

#[test]
fn a5_answer_for_returns_recorded_answers() {
    let mut q = Quest::new(Template::Default);
    assert_eq!(q.answer_for(&StepId::Name), None);
    let _ = q.answer(StepId::Name, Answer::Skipped);
    assert_eq!(q.answer_for(&StepId::Name), Some(&Answer::Skipped));
    let _ = q.answer(StepId::Budget, Answer::BudgetCap(15000, "USD".to_string()));
    assert_eq!(
        q.answer_for(&StepId::Budget),
        Some(&Answer::BudgetCap(15000, "USD".to_string()))
    );
}

// ---- N1-N4: walking the quest ---------------------------------------------------

#[test]
fn n1_next_step_walks_in_order() {
    let mut q = Quest::new(Template::Default);
    assert_eq!(q.next_step().map(|s| s.id.clone()), Some(StepId::Name));
    let _ = q.answer(StepId::Name, Answer::Text("Dresser".to_string()));
    assert_eq!(q.next_step().map(|s| s.id.clone()), Some(StepId::Goal));
    let _ = q.answer(StepId::Goal, Answer::Text("A dresser".to_string()));
    assert_eq!(q.next_step().map(|s| s.id.clone()), Some(StepId::Fit));
    let _ = q.answer(StepId::Fit, Answer::Fit(FitPick::DecideLater));
    assert_eq!(q.next_step().map(|s| s.id.clone()), Some(StepId::Tools));
    let _ = q.answer(StepId::Tools, Answer::Items(0));
    assert_eq!(q.next_step().map(|s| s.id.clone()), Some(StepId::Materials));
    let _ = q.answer(StepId::Materials, Answer::Items(2));
    assert_eq!(q.next_step().map(|s| s.id.clone()), Some(StepId::Deadline));
    let _ = q.answer(StepId::Deadline, Answer::Deadline(None));
    assert_eq!(q.next_step().map(|s| s.id.clone()), Some(StepId::Budget));
    let _ = q.answer(StepId::Budget, Answer::BudgetNotSure);
    assert_eq!(
        q.next_step().map(|s| s.id.clone()),
        Some(StepId::Inspiration)
    );
    let _ = q.answer(StepId::Inspiration, Answer::Items(0));
    assert_eq!(q.next_step().map(|s| s.id.clone()), Some(StepId::Review));
    let _ = q.answer(StepId::Review, Answer::Confirmed);
    assert_eq!(q.next_step(), None);
}

#[test]
fn n1_next_step_after_skips() {
    // N1: a Skipped step counts as answered, so the flow moves on.
    let mut q = Quest::new(Template::Default);
    let _ = q.answer(StepId::Name, Answer::Skipped);
    assert_eq!(q.next_step().map(|s| s.id.clone()), Some(StepId::Goal));
    let _ = q.answer(StepId::Goal, Answer::Skipped);
    assert_eq!(q.next_step().map(|s| s.id.clone()), Some(StepId::Fit));
    q.skip_to_designing();
    assert_eq!(q.next_step(), None);
}

#[test]
fn n1_packaging_order() {
    // N1: the next step is the first with no answer, so steps 1-8 are answered first
    // (corrected by Claude: the PA's version answered only Inspiration and expected BoxContents).
    let mut q = Quest::new(Template::Packaging);
    assert_eq!(q.next_step().map(|s| s.id.clone()), Some(StepId::Name));
    let _ = q.answer(StepId::Name, Answer::Text("Gift box".to_string()));
    let _ = q.answer(StepId::Goal, Answer::Skipped);
    let _ = q.answer(StepId::Fit, Answer::Fit(FitPick::SmallNoFixedSpace));
    let _ = q.answer(StepId::Tools, Answer::Items(0));
    let _ = q.answer(StepId::Materials, Answer::Skipped);
    let _ = q.answer(StepId::Deadline, Answer::Deadline(None));
    let _ = q.answer(StepId::Budget, Answer::BudgetNotSure);
    assert_eq!(
        q.next_step().map(|s| s.id.clone()),
        Some(StepId::Inspiration)
    );
    let _ = q.answer(StepId::Inspiration, Answer::Items(1));
    assert_eq!(
        q.next_step().map(|s| s.id.clone()),
        Some(StepId::BoxContents)
    );
    let _ = q.answer(StepId::BoxContents, Answer::Text("socks".to_string()));
    assert_eq!(q.next_step().map(|s| s.id.clone()), Some(StepId::HowMany));
    let _ = q.answer(StepId::HowMany, Answer::Count(12));
    assert_eq!(q.next_step().map(|s| s.id.clone()), Some(StepId::Review));
}

#[test]
fn n1_packaging_none_when_every_step_answered() {
    let mut q = Quest::new(Template::Packaging);
    let _ = q.answer(StepId::Name, Answer::Text("Gift box".to_string()));
    let _ = q.answer(StepId::Goal, Answer::Text("A gift box".to_string()));
    let _ = q.answer(StepId::Fit, Answer::Fit(FitPick::TypeSizes));
    let _ = q.answer(StepId::Tools, Answer::Items(0));
    let _ = q.answer(StepId::Materials, Answer::Items(1));
    let _ = q.answer(StepId::Deadline, Answer::Deadline(Some((2026, 12, 1))));
    let _ = q.answer(StepId::Budget, Answer::BudgetCap(999, "EUR".to_string()));
    let _ = q.answer(StepId::Inspiration, Answer::Items(2));
    let _ = q.answer(StepId::BoxContents, Answer::Text("socks".to_string()));
    let _ = q.answer(StepId::HowMany, Answer::Count(12));
    let _ = q.answer(StepId::Review, Answer::Confirmed);
    assert_eq!(q.next_step(), None);
    assert!(q.is_done());
    assert_eq!(q.outstanding(), Vec::<StepId>::new());
}

#[test]
fn n2_outstanding_fresh_default() {
    let q = Quest::new(Template::Default);
    assert_eq!(
        q.outstanding(),
        vec![
            StepId::Name,
            StepId::Goal,
            StepId::Fit,
            StepId::Tools,
            StepId::Materials,
            StepId::Deadline,
            StepId::Budget,
            StepId::Inspiration
        ]
    );
}

#[test]
fn n2_outstanding_never_lists_review() {
    // Every non-Review step answered, Review left unanswered: outstanding is
    // empty, and Review is still the next step.
    let mut q = Quest::new(Template::Default);
    let _ = q.answer(StepId::Name, Answer::Text("Dresser".to_string()));
    let _ = q.answer(StepId::Goal, Answer::Text("A dresser".to_string()));
    let _ = q.answer(StepId::Fit, Answer::Fit(FitPick::DecideLater));
    let _ = q.answer(StepId::Tools, Answer::Items(0));
    let _ = q.answer(StepId::Materials, Answer::Items(0));
    let _ = q.answer(StepId::Deadline, Answer::Deadline(None));
    let _ = q.answer(StepId::Budget, Answer::BudgetNotSure);
    let _ = q.answer(StepId::Inspiration, Answer::Items(0));
    assert_eq!(q.outstanding(), Vec::<StepId>::new());
    assert_eq!(q.next_step().map(|s| s.id.clone()), Some(StepId::Review));
}

#[test]
fn n2_outstanding_skipped_and_missing() {
    let mut q = Quest::new(Template::Default);
    let _ = q.answer(StepId::Name, Answer::Text("Dresser".to_string()));
    let _ = q.answer(StepId::Goal, Answer::Skipped);
    assert_eq!(
        q.outstanding(),
        vec![
            StepId::Goal,
            StepId::Fit,
            StepId::Tools,
            StepId::Materials,
            StepId::Deadline,
            StepId::Budget,
            StepId::Inspiration
        ]
    );
}

#[test]
fn n3_is_done() {
    let mut q = Quest::new(Template::Default);
    assert!(!q.is_done());
    let _ = q.answer(StepId::Review, Answer::Confirmed);
    assert!(q.is_done());
}

#[test]
fn n4_skip_to_designing_worked_example() {
    // Worked example, rows 1-8, then row 9.
    let mut q = Quest::new(Template::Default);
    let _ = q.answer(StepId::Name, Answer::Text("Dresser".to_string()));
    let _ = q.answer(StepId::Goal, Answer::Skipped);
    let _ = q.answer(StepId::Fit, Answer::Fit(FitPick::SmallNoFixedSpace));
    let _ = q.answer(StepId::Materials, Answer::Items(1));
    q.skip_to_designing();
    assert!(q.is_done());
    assert_eq!(
        q.outstanding(),
        vec![
            StepId::Goal,
            StepId::Tools,
            StepId::Deadline,
            StepId::Budget,
            StepId::Inspiration
        ]
    );
}

#[test]
fn n4_skip_keeps_earlier_answers() {
    let mut q = Quest::new(Template::Packaging);
    let _ = q.answer(StepId::Name, Answer::Text("Gift box".to_string()));
    let _ = q.answer(StepId::Fit, Answer::Fit(FitPick::SmallNoFixedSpace));
    let _ = q.answer(StepId::Materials, Answer::Items(3));
    let _ = q.answer(StepId::HowMany, Answer::Count(50));
    q.skip_to_designing();
    assert_eq!(
        q.answer_for(&StepId::Name),
        Some(&Answer::Text("Gift box".to_string()))
    );
    assert_eq!(
        q.answer_for(&StepId::Fit),
        Some(&Answer::Fit(FitPick::SmallNoFixedSpace))
    );
    assert_eq!(q.answer_for(&StepId::Materials), Some(&Answer::Items(3)));
    assert_eq!(q.answer_for(&StepId::HowMany), Some(&Answer::Count(50)));
    assert_eq!(
        q.outstanding(),
        vec![
            StepId::Goal,
            StepId::Tools,
            StepId::Deadline,
            StepId::Budget,
            StepId::Inspiration,
            StepId::BoxContents
        ]
    );
}

#[test]
fn n4_skip_on_fresh_quest() {
    let mut q = Quest::new(Template::Default);
    q.skip_to_designing();
    assert!(q.is_done());
    assert_eq!(q.next_step(), None);
    assert_eq!(q.answer_for(&StepId::Name), Some(&Answer::Skipped));
    assert_eq!(
        q.outstanding(),
        vec![
            StepId::Name,
            StepId::Goal,
            StepId::Fit,
            StepId::Tools,
            StepId::Materials,
            StepId::Deadline,
            StepId::Budget,
            StepId::Inspiration
        ]
    );
}

#[test]
fn n4_skip_twice_is_stable() {
    let mut q = Quest::new(Template::Default);
    let _ = q.answer(StepId::Name, Answer::Text("Dresser".to_string()));
    q.skip_to_designing();
    let outstanding = q.outstanding();
    q.skip_to_designing();
    assert!(q.is_done());
    assert_eq!(q.outstanding(), outstanding);
    assert_eq!(
        q.answer_for(&StepId::Name),
        Some(&Answer::Text("Dresser".to_string()))
    );
}

// ---- F1-F3, S1: facts read from the answers --------------------------------------

#[test]
fn f1_intent_missing() {
    let mut q = Quest::new(Template::Default);
    assert!(q.intent_missing());
    let _ = q.answer(
        StepId::Goal,
        Answer::Text("A four-drawer dresser".to_string()),
    );
    assert!(!q.intent_missing());
    let _ = q.answer(StepId::Goal, Answer::Text("   ".to_string()));
    assert!(q.intent_missing());
    let _ = q.answer(StepId::Goal, Answer::Skipped);
    assert!(q.intent_missing());
}

#[test]
fn f2_materials_given() {
    let mut q = Quest::new(Template::Default);
    assert!(!q.materials_given());
    let _ = q.answer(StepId::Materials, Answer::Items(0));
    assert!(!q.materials_given());
    let _ = q.answer(StepId::Materials, Answer::Items(1));
    assert!(q.materials_given());
    let _ = q.answer(StepId::Materials, Answer::Skipped);
    assert!(!q.materials_given());
}

#[test]
fn f3_suggested_class() {
    let mut q = Quest::new(Template::Default);
    assert_eq!(q.suggested_class(), None);
    for pick in [
        FitPick::TypeSizes,
        FitPick::HaveScan,
        FitPick::PhotographWithReference,
        FitPick::DecideLater,
    ] {
        let _ = q.answer(StepId::Fit, Answer::Fit(pick));
        assert_eq!(q.suggested_class(), None);
    }
    let _ = q.answer(StepId::Fit, Answer::Fit(FitPick::SmallNoFixedSpace));
    assert_eq!(q.suggested_class(), Some(ScaleClass::Bench));
    let _ = q.answer(StepId::Fit, Answer::Skipped);
    assert_eq!(q.suggested_class(), None);
}

#[test]
fn s1_class_for_size_boundaries() {
    assert_eq!(class_for_size(5.0), Some(ScaleClass::Macro));
    assert_eq!(class_for_size(9.999), Some(ScaleClass::Macro));
    assert_eq!(class_for_size(10.0), Some(ScaleClass::Bench));
    assert_eq!(class_for_size(500.0), Some(ScaleClass::Bench));
    assert_eq!(class_for_size(500.1), Some(ScaleClass::Furniture));
    assert_eq!(class_for_size(2500.0), Some(ScaleClass::Furniture));
    assert_eq!(class_for_size(2500.1), Some(ScaleClass::Room));
    assert_eq!(class_for_size(10000.0), Some(ScaleClass::Room));
}

#[test]
fn s1_class_for_size_not_finite_or_nonpositive() {
    assert_eq!(class_for_size(0.0), None);
    assert_eq!(class_for_size(-1.0), None);
    assert_eq!(class_for_size(f64::NAN), None);
    assert_eq!(class_for_size(f64::INFINITY), None);
    assert_eq!(class_for_size(f64::NEG_INFINITY), None);
}

// ---- the contract's worked examples -------------------------------------------------

#[test]
fn worked_example_quest_in_order() {
    let mut q = Quest::new(Template::Default);

    // Row 1.
    assert_eq!(q.next_step().map(|s| s.id.clone()), Some(StepId::Name));
    assert_eq!(
        q.outstanding(),
        vec![
            StepId::Name,
            StepId::Goal,
            StepId::Fit,
            StepId::Tools,
            StepId::Materials,
            StepId::Deadline,
            StepId::Budget,
            StepId::Inspiration
        ]
    );

    // Row 2.
    assert!(q
        .answer(StepId::Name, Answer::Text("Dresser".to_string()))
        .is_ok());
    assert_eq!(q.next_step().map(|s| s.id.clone()), Some(StepId::Goal));

    // Row 3.
    assert!(q.answer(StepId::Goal, Answer::Skipped).is_ok());
    assert_eq!(q.next_step().map(|s| s.id.clone()), Some(StepId::Fit));
    assert_eq!(
        q.outstanding(),
        vec![
            StepId::Goal,
            StepId::Fit,
            StepId::Tools,
            StepId::Materials,
            StepId::Deadline,
            StepId::Budget,
            StepId::Inspiration
        ]
    );
    assert!(q.intent_missing());

    // Row 4.
    assert_eq!(
        q.answer(StepId::Fit, Answer::Text("x".to_string())),
        Err(QuestError::WrongKind(StepId::Fit))
    );
    assert_eq!(q.answer_for(&StepId::Fit), None);

    // Row 5.
    assert!(q
        .answer(StepId::Fit, Answer::Fit(FitPick::SmallNoFixedSpace))
        .is_ok());
    assert_eq!(q.suggested_class(), Some(ScaleClass::Bench));

    // Row 6.
    assert_eq!(
        q.answer(StepId::BoxContents, Answer::Text("socks".to_string())),
        Err(QuestError::NotInThisQuest(StepId::BoxContents))
    );

    // Row 7.
    assert!(q.answer(StepId::Materials, Answer::Items(1)).is_ok());
    assert!(q.materials_given());
    assert_eq!(q.next_step().map(|s| s.id.clone()), Some(StepId::Tools));

    // Row 8.
    assert_eq!(
        q.answer(StepId::Review, Answer::Skipped),
        Err(QuestError::WrongKind(StepId::Review))
    );

    // Row 9.
    q.skip_to_designing();
    assert!(q.is_done());
    assert_eq!(
        q.outstanding(),
        vec![
            StepId::Goal,
            StepId::Tools,
            StepId::Deadline,
            StepId::Budget,
            StepId::Inspiration
        ]
    );

    // Row 10.
    assert!(q
        .answer(
            StepId::Goal,
            Answer::Text("A four-drawer dresser".to_string())
        )
        .is_ok());
    assert!(!q.intent_missing());
    assert_eq!(
        q.outstanding(),
        vec![
            StepId::Tools,
            StepId::Deadline,
            StepId::Budget,
            StepId::Inspiration
        ]
    );

    // Row 11.
    assert_eq!(
        q.answer(StepId::Review, Answer::Confirmed),
        Err(QuestError::AlreadyConfirmed)
    );

    // Row 12.
    assert!(q
        .answer(StepId::Goal, Answer::Text("   ".to_string()))
        .is_ok());
    assert!(q.intent_missing());
}

#[test]
fn worked_example_class_for_size() {
    assert_eq!(class_for_size(5.0), Some(ScaleClass::Macro));
    assert_eq!(class_for_size(10.0), Some(ScaleClass::Bench));
    assert_eq!(class_for_size(500.0), Some(ScaleClass::Bench));
    assert_eq!(class_for_size(500.1), Some(ScaleClass::Furniture));
    assert_eq!(class_for_size(2500.0), Some(ScaleClass::Furniture));
    assert_eq!(class_for_size(2500.1), Some(ScaleClass::Room));
    assert_eq!(class_for_size(0.0), None);
    assert_eq!(class_for_size(-1.0), None);
    assert_eq!(class_for_size(f64::NAN), None);
    assert_eq!(class_for_size(f64::INFINITY), None);
}

#[test]
fn refused_answer_changes_nothing() {
    let mut q = Quest::new(Template::Default);
    let _ = q.answer(StepId::Name, Answer::Text("Dresser".to_string()));
    let _ = q.answer(StepId::Goal, Answer::Skipped);
    // Row 4: a Text answer on Fit is refused; the quest is exactly as before.
    assert_eq!(
        q.answer(StepId::Fit, Answer::Text("x".to_string())),
        Err(QuestError::WrongKind(StepId::Fit))
    );
    assert_eq!(q.answer_for(&StepId::Fit), None);
    assert_eq!(q.next_step().map(|s| s.id.clone()), Some(StepId::Fit));
    assert_eq!(
        q.outstanding(),
        vec![
            StepId::Goal,
            StepId::Fit,
            StepId::Tools,
            StepId::Materials,
            StepId::Deadline,
            StepId::Budget,
            StepId::Inspiration
        ]
    );
    assert!(!q.is_done());
    // Row 6: a step of another template is refused; the quest is exactly as before.
    assert_eq!(
        q.answer(StepId::BoxContents, Answer::Text("socks".to_string())),
        Err(QuestError::NotInThisQuest(StepId::BoxContents))
    );
    assert_eq!(q.answer_for(&StepId::BoxContents), None);
    assert_eq!(q.next_step().map(|s| s.id.clone()), Some(StepId::Fit));
    // Row 8: Skipped does not fit Review; the quest is exactly as before.
    assert_eq!(
        q.answer(StepId::Review, Answer::Skipped),
        Err(QuestError::WrongKind(StepId::Review))
    );
    assert!(!q.is_done());
    assert_eq!(q.next_step().map(|s| s.id.clone()), Some(StepId::Fit));
}
