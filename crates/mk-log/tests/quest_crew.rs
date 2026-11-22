//! Tests for mk_log::quest written by Gordon (the Crew, campaign mk-core-10) from the contract alone,
//! without seeing the module.

//! Integration tests for `mk_log::quest`, written from the contract alone.

use mk_log::quest::{
    class_for_size, steps, Answer, AnswerKind, FitPick, Quest, QuestError, ScaleClass, Step,
    StepId, Template,
};

/// Builds one expected `Step` (rule T1's table).
fn step(id: StepId, question: &'static str, kind: AnswerKind) -> Step {
    Step { id, question, kind }
}

/// The nine steps of Default, SoftGoods and Room, in order (rule T1).
fn nine_steps() -> Vec<Step> {
    vec![
        step(StepId::Name, "What shall we call it?", AnswerKind::Text),
        step(
            StepId::Goal,
            "What would you like to make?",
            AnswerKind::Text,
        ),
        step(
            StepId::Fit,
            "Where does it need to fit? How big is it?",
            AnswerKind::Fit,
        ),
        step(StepId::Tools, "What tools do you have?", AnswerKind::Items),
        step(
            StepId::Materials,
            "What materials do you have?",
            AnswerKind::Items,
        ),
        step(
            StepId::Deadline,
            "When do you need it?",
            AnswerKind::Deadline,
        ),
        step(StepId::Budget, "What's your budget?", AnswerKind::Budget),
        step(StepId::Inspiration, "What inspires you?", AnswerKind::Items),
        step(StepId::Review, "Here's your workspace", AnswerKind::Review),
    ]
}

/// Every variant of `Answer`, one of each (rule A3's table).
fn all_answers() -> Vec<Answer> {
    vec![
        Answer::Skipped,
        Answer::Text("x".to_string()),
        Answer::Fit(FitPick::TypeSizes),
        Answer::Items(0),
        Answer::Deadline(None),
        Answer::BudgetCap(100, "USD".to_string()),
        Answer::BudgetOnlyWhatIHave,
        Answer::BudgetNotSure,
        Answer::Count(3),
        Answer::Confirmed,
    ]
}

/// Rule A3 read off the contract by hand: which answers each kind takes.
fn accepts(kind: AnswerKind, answer: &Answer) -> bool {
    if kind == AnswerKind::Review {
        return matches!(answer, Answer::Confirmed);
    }
    if matches!(answer, Answer::Skipped) {
        return true;
    }
    match kind {
        AnswerKind::Text => matches!(answer, Answer::Text(_)),
        AnswerKind::Fit => matches!(answer, Answer::Fit(_)),
        AnswerKind::Items => matches!(answer, Answer::Items(_)),
        AnswerKind::Deadline => matches!(answer, Answer::Deadline(_)),
        AnswerKind::Budget => matches!(
            answer,
            Answer::BudgetCap(..) | Answer::BudgetOnlyWhatIHave | Answer::BudgetNotSure
        ),
        AnswerKind::Count => matches!(answer, Answer::Count(_)),
        AnswerKind::Review => false,
    }
}

/// Tries every answer variant against one step of a fresh Quest.
fn check_kind(template: Template, id: StepId, kind: AnswerKind) {
    for answer in all_answers() {
        let mut q = Quest::new(template);
        let got = q.answer(id.clone(), answer.clone());
        if accepts(kind, &answer) {
            assert_eq!(got, Ok(()), "{:?} should accept {:?}", id, answer);
            assert_eq!(
                q.answer_for(&id),
                Some(&answer),
                "{:?} should record {:?}",
                id,
                answer
            );
        } else {
            assert_eq!(
                got,
                Err(QuestError::WrongKind(id.clone())),
                "{:?} should refuse {:?}",
                id,
                answer
            );
            assert_eq!(
                q.answer_for(&id),
                None,
                "{:?} should not record {:?}",
                id,
                answer
            );
        }
    }
}

#[test]
fn rule_t1_default_steps_are_the_nine_in_order() {
    assert_eq!(steps(Template::Default), nine_steps());
}

#[test]
fn rule_t1_soft_goods_steps_are_the_nine_in_order() {
    assert_eq!(steps(Template::SoftGoods), nine_steps());
}

#[test]
fn rule_t1_room_steps_are_the_nine_in_order() {
    assert_eq!(steps(Template::Room), nine_steps());
}

#[test]
fn rule_t2_packaging_inserts_box_contents_and_how_many_before_review() {
    let mut want = nine_steps();
    want.insert(
        8,
        step(
            StepId::BoxContents,
            "What goes in the box?",
            AnswerKind::Text,
        ),
    );
    want.insert(
        9,
        step(
            StepId::HowMany,
            "How many will you make?",
            AnswerKind::Count,
        ),
    );
    assert_eq!(steps(Template::Packaging), want);
}

#[test]
fn rule_a1_a_step_not_in_this_quest_is_refused() {
    let mut q = Quest::new(Template::Default);
    assert_eq!(
        q.answer(StepId::BoxContents, Answer::Text("socks".to_string())),
        Err(QuestError::NotInThisQuest(StepId::BoxContents))
    );
    assert_eq!(q.answer_for(&StepId::BoxContents), None);
    assert_eq!(
        q.answer(StepId::HowMany, Answer::Count(2)),
        Err(QuestError::NotInThisQuest(StepId::HowMany))
    );
    // A1 comes before A3: a wrong-kind answer to a missing step is still NotInThisQuest.
    assert_eq!(
        q.answer(StepId::HowMany, Answer::Text("two".to_string())),
        Err(QuestError::NotInThisQuest(StepId::HowMany))
    );
    // The same steps are in the Packaging quest.
    let mut p = Quest::new(Template::Packaging);
    assert_eq!(
        p.answer(StepId::BoxContents, Answer::Text("socks".to_string())),
        Ok(())
    );
    assert_eq!(p.answer(StepId::HowMany, Answer::Count(2)), Ok(()));
}

#[test]
fn rule_a2_review_already_confirmed_is_refused() {
    let mut q = Quest::new(Template::Default);
    assert_eq!(q.answer(StepId::Review, Answer::Confirmed), Ok(()));
    assert_eq!(
        q.answer(StepId::Review, Answer::Confirmed),
        Err(QuestError::AlreadyConfirmed)
    );
    assert_eq!(q.answer_for(&StepId::Review), Some(&Answer::Confirmed));
    // A2 comes before A3: a wrong-kind answer to a confirmed Review is AlreadyConfirmed.
    assert_eq!(
        q.answer(StepId::Review, Answer::Skipped),
        Err(QuestError::AlreadyConfirmed)
    );
    assert_eq!(q.answer_for(&StepId::Review), Some(&Answer::Confirmed));
}

#[test]
fn rule_a3_skipped_fits_every_kind_except_review() {
    for s in steps(Template::Packaging) {
        let mut q = Quest::new(Template::Packaging);
        let got = q.answer(s.id.clone(), Answer::Skipped);
        if s.kind == AnswerKind::Review {
            assert_eq!(got, Err(QuestError::WrongKind(StepId::Review)));
            assert_eq!(q.answer_for(&s.id), None);
        } else {
            assert_eq!(got, Ok(()), "{:?} should accept Skipped", s.id);
            assert_eq!(q.answer_for(&s.id), Some(&Answer::Skipped));
        }
    }
}

#[test]
fn rule_a3_text_kind_takes_text_and_skipped_only() {
    check_kind(Template::Default, StepId::Name, AnswerKind::Text);
    check_kind(Template::Default, StepId::Goal, AnswerKind::Text);
}

#[test]
fn rule_a3_fit_kind_takes_fit_and_skipped_only() {
    check_kind(Template::Default, StepId::Fit, AnswerKind::Fit);
}

#[test]
fn rule_a3_items_kind_takes_items_and_skipped_only() {
    check_kind(Template::Default, StepId::Tools, AnswerKind::Items);
    check_kind(Template::Default, StepId::Materials, AnswerKind::Items);
    check_kind(Template::Default, StepId::Inspiration, AnswerKind::Items);
}

#[test]
fn rule_a3_deadline_kind_takes_deadline_and_skipped_only() {
    check_kind(Template::Default, StepId::Deadline, AnswerKind::Deadline);
}

#[test]
fn rule_a3_budget_kind_takes_the_three_budget_answers_and_skipped() {
    check_kind(Template::Default, StepId::Budget, AnswerKind::Budget);
}

#[test]
fn rule_a3_count_kind_takes_count_and_skipped_only() {
    check_kind(Template::Packaging, StepId::HowMany, AnswerKind::Count);
}

#[test]
fn rule_a3_review_kind_takes_confirmed_only() {
    check_kind(Template::Default, StepId::Review, AnswerKind::Review);
}

#[test]
fn rule_a4_a_refused_answer_changes_nothing() {
    let mut q = Quest::new(Template::Default);
    assert_eq!(
        q.answer(StepId::Name, Answer::Text("Dresser".to_string())),
        Ok(())
    );
    assert_eq!(
        q.answer(StepId::Name, Answer::Items(3)),
        Err(QuestError::WrongKind(StepId::Name))
    );
    assert_eq!(
        q.answer_for(&StepId::Name),
        Some(&Answer::Text("Dresser".to_string()))
    );
    assert_eq!(
        q.next_step(),
        Some(&step(
            StepId::Goal,
            "What would you like to make?",
            AnswerKind::Text
        ))
    );
    assert_eq!(
        q.outstanding(),
        vec![
            StepId::Goal,
            StepId::Fit,
            StepId::Tools,
            StepId::Materials,
            StepId::Deadline,
            StepId::Budget,
            StepId::Inspiration,
        ]
    );
}

#[test]
fn rule_a4_an_answer_can_be_replaced() {
    let mut q = Quest::new(Template::Default);
    assert_eq!(
        q.answer(StepId::Name, Answer::Text("Dresser".to_string())),
        Ok(())
    );
    assert_eq!(
        q.answer(StepId::Name, Answer::Text("Chest".to_string())),
        Ok(())
    );
    assert_eq!(
        q.answer_for(&StepId::Name),
        Some(&Answer::Text("Chest".to_string()))
    );
    // A skipped step can be revisited later.
    assert_eq!(q.answer(StepId::Goal, Answer::Skipped), Ok(()));
    assert_eq!(
        q.answer(StepId::Goal, Answer::Text("A chest".to_string())),
        Ok(())
    );
    assert_eq!(
        q.answer_for(&StepId::Goal),
        Some(&Answer::Text("A chest".to_string()))
    );
    assert!(!q.intent_missing());
    // Replacing does not duplicate the step in outstanding.
    assert_eq!(
        q.outstanding(),
        vec![
            StepId::Fit,
            StepId::Tools,
            StepId::Materials,
            StepId::Deadline,
            StepId::Budget,
            StepId::Inspiration,
        ]
    );
}

#[test]
fn rule_a5_answer_for_returns_skipped() {
    let mut q = Quest::new(Template::Default);
    assert_eq!(q.answer_for(&StepId::Goal), None);
    assert_eq!(q.answer(StepId::Goal, Answer::Skipped), Ok(()));
    assert_eq!(q.answer_for(&StepId::Goal), Some(&Answer::Skipped));
}

#[test]
fn rule_n1_next_step_moves_on_after_a_skip() {
    let mut q = Quest::new(Template::Default);
    assert_eq!(q.answer(StepId::Name, Answer::Skipped), Ok(()));
    assert_eq!(
        q.next_step(),
        Some(&step(
            StepId::Goal,
            "What would you like to make?",
            AnswerKind::Text
        ))
    );
    assert_eq!(q.answer(StepId::Goal, Answer::Skipped), Ok(()));
    assert_eq!(
        q.next_step(),
        Some(&step(
            StepId::Fit,
            "Where does it need to fit? How big is it?",
            AnswerKind::Fit
        ))
    );
}

#[test]
fn rule_n1_next_step_is_none_when_every_step_has_an_answer() {
    let mut q = Quest::new(Template::Default);
    q.skip_to_designing();
    assert_eq!(q.next_step(), None);
}

#[test]
fn rule_n2_outstanding_never_includes_review() {
    let mut q = Quest::new(Template::Default);
    assert!(!q.outstanding().contains(&StepId::Review));
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
            StepId::Inspiration,
        ]
    );
    q.skip_to_designing();
    assert!(!q.outstanding().contains(&StepId::Review));
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
            StepId::Inspiration,
        ]
    );
}

#[test]
fn rule_n3_is_done_only_when_review_is_confirmed() {
    let mut q = Quest::new(Template::Default);
    assert!(!q.is_done());
    q.skip_to_designing();
    assert!(q.is_done());
    let mut q = Quest::new(Template::Default);
    assert_eq!(q.answer(StepId::Review, Answer::Confirmed), Ok(()));
    assert!(q.is_done());
}

#[test]
fn rule_n4_skip_to_designing_keeps_earlier_answers() {
    let mut q = Quest::new(Template::Default);
    assert_eq!(
        q.answer(StepId::Name, Answer::Text("Dresser".to_string())),
        Ok(())
    );
    assert_eq!(q.answer(StepId::Materials, Answer::Items(2)), Ok(()));
    q.skip_to_designing();
    assert_eq!(
        q.answer_for(&StepId::Name),
        Some(&Answer::Text("Dresser".to_string()))
    );
    assert_eq!(q.answer_for(&StepId::Materials), Some(&Answer::Items(2)));
    assert_eq!(q.answer_for(&StepId::Goal), Some(&Answer::Skipped));
    assert_eq!(q.answer_for(&StepId::Review), Some(&Answer::Confirmed));
    assert!(q.is_done());
    assert_eq!(
        q.outstanding(),
        vec![
            StepId::Goal,
            StepId::Fit,
            StepId::Tools,
            StepId::Deadline,
            StepId::Budget,
            StepId::Inspiration,
        ]
    );
}

#[test]
fn rule_f1_intent_missing_for_a_blank_goal() {
    let mut q = Quest::new(Template::Default);
    assert!(q.intent_missing());
    assert_eq!(q.answer(StepId::Goal, Answer::Skipped), Ok(()));
    assert!(q.intent_missing());
    assert_eq!(q.answer(StepId::Goal, Answer::Text(String::new())), Ok(()));
    assert!(q.intent_missing());
    assert_eq!(
        q.answer(StepId::Goal, Answer::Text("   ".to_string())),
        Ok(())
    );
    assert!(q.intent_missing());
    assert_eq!(
        q.answer(StepId::Goal, Answer::Text("\t\n ".to_string())),
        Ok(())
    );
    assert!(q.intent_missing());
    assert_eq!(
        q.answer(StepId::Goal, Answer::Text("A dresser".to_string())),
        Ok(())
    );
    assert!(!q.intent_missing());
    assert_eq!(
        q.answer(StepId::Goal, Answer::Text("  A dresser  ".to_string())),
        Ok(())
    );
    assert!(!q.intent_missing());
}

#[test]
fn rule_f2_materials_given_for_items_0_and_1() {
    let mut q = Quest::new(Template::Default);
    assert!(!q.materials_given());
    assert_eq!(q.answer(StepId::Materials, Answer::Items(0)), Ok(()));
    assert!(!q.materials_given());
    assert_eq!(q.answer(StepId::Materials, Answer::Items(1)), Ok(()));
    assert!(q.materials_given());
    assert_eq!(q.answer(StepId::Materials, Answer::Items(7)), Ok(()));
    assert!(q.materials_given());
    assert_eq!(q.answer(StepId::Materials, Answer::Skipped), Ok(()));
    assert!(!q.materials_given());
}

#[test]
fn rule_f3_suggested_class_from_the_fit_answer() {
    let mut q = Quest::new(Template::Default);
    assert_eq!(q.suggested_class(), None);
    assert_eq!(
        q.answer(StepId::Fit, Answer::Fit(FitPick::SmallNoFixedSpace)),
        Ok(())
    );
    assert_eq!(q.suggested_class(), Some(ScaleClass::Bench));
    for pick in [
        FitPick::TypeSizes,
        FitPick::HaveScan,
        FitPick::PhotographWithReference,
        FitPick::DecideLater,
    ] {
        let mut q = Quest::new(Template::Default);
        assert_eq!(q.answer(StepId::Fit, Answer::Fit(pick)), Ok(()));
        assert_eq!(
            q.suggested_class(),
            None,
            "{:?} should not suggest a class",
            pick
        );
    }
    let mut q = Quest::new(Template::Default);
    assert_eq!(q.answer(StepId::Fit, Answer::Skipped), Ok(()));
    assert_eq!(q.suggested_class(), None);
}

#[test]
fn rule_s1_class_for_size_cutoffs() {
    assert_eq!(class_for_size(5.0), Some(ScaleClass::Macro));
    assert_eq!(class_for_size(9.999), Some(ScaleClass::Macro));
    assert_eq!(class_for_size(10.0), Some(ScaleClass::Bench));
    assert_eq!(class_for_size(500.0), Some(ScaleClass::Bench));
    assert_eq!(class_for_size(500.1), Some(ScaleClass::Furniture));
    assert_eq!(class_for_size(2500.0), Some(ScaleClass::Furniture));
    assert_eq!(class_for_size(2500.1), Some(ScaleClass::Room));
    assert_eq!(class_for_size(1.0e9), Some(ScaleClass::Room));
}

#[test]
fn rule_s1_class_for_size_rejects_non_finite_and_non_positive() {
    assert_eq!(class_for_size(0.0), None);
    assert_eq!(class_for_size(-1.0), None);
    assert_eq!(class_for_size(f64::NAN), None);
    assert_eq!(class_for_size(f64::INFINITY), None);
    assert_eq!(class_for_size(f64::NEG_INFINITY), None);
}

#[test]
fn rule_s1_body_is_never_suggested_from_a_size_alone() {
    for size in [0.5, 5.0, 10.0, 100.0, 500.0, 1000.0, 2500.0, 3000.0, 1.0e6] {
        assert_ne!(
            class_for_size(size),
            Some(ScaleClass::Body),
            "size {} should not suggest Body",
            size
        );
    }
}

#[test]
fn worked_example_table_in_order() {
    let mut q = Quest::new(Template::Default);

    // Row 1: next is Name; all eight non-Review steps outstanding.
    assert_eq!(
        q.next_step(),
        Some(&step(
            StepId::Name,
            "What shall we call it?",
            AnswerKind::Text
        ))
    );
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
            StepId::Inspiration,
        ]
    );

    // Row 2: Name answered, next is Goal.
    assert_eq!(
        q.answer(StepId::Name, Answer::Text("Dresser".to_string())),
        Ok(())
    );
    assert_eq!(
        q.next_step(),
        Some(&step(
            StepId::Goal,
            "What would you like to make?",
            AnswerKind::Text
        ))
    );

    // Row 3: Goal skipped, next is Fit, outstanding starts with Goal, intent missing.
    assert_eq!(q.answer(StepId::Goal, Answer::Skipped), Ok(()));
    assert_eq!(
        q.next_step(),
        Some(&step(
            StepId::Fit,
            "Where does it need to fit? How big is it?",
            AnswerKind::Fit
        ))
    );
    assert_eq!(q.outstanding().first(), Some(&StepId::Goal));
    assert!(q.intent_missing());

    // Row 4: wrong kind refused, nothing changes.
    assert_eq!(
        q.answer(StepId::Fit, Answer::Text("x".to_string())),
        Err(QuestError::WrongKind(StepId::Fit))
    );
    assert_eq!(q.answer_for(&StepId::Fit), None);
    assert_eq!(
        q.next_step(),
        Some(&step(
            StepId::Fit,
            "Where does it need to fit? How big is it?",
            AnswerKind::Fit
        ))
    );

    // Row 5: Fit answered, Bench suggested.
    assert_eq!(
        q.answer(StepId::Fit, Answer::Fit(FitPick::SmallNoFixedSpace)),
        Ok(())
    );
    assert_eq!(q.suggested_class(), Some(ScaleClass::Bench));

    // Row 6: BoxContents is not in the Default quest.
    assert_eq!(
        q.answer(StepId::BoxContents, Answer::Text("socks".to_string())),
        Err(QuestError::NotInThisQuest(StepId::BoxContents))
    );

    // Row 7: Materials given, next is Tools.
    assert_eq!(q.answer(StepId::Materials, Answer::Items(1)), Ok(()));
    assert!(q.materials_given());
    assert_eq!(
        q.next_step(),
        Some(&step(
            StepId::Tools,
            "What tools do you have?",
            AnswerKind::Items
        ))
    );

    // Row 8: Review does not take Skipped.
    assert_eq!(
        q.answer(StepId::Review, Answer::Skipped),
        Err(QuestError::WrongKind(StepId::Review))
    );

    // Row 9: skip to designing opens the project; Goal stays outstanding.
    q.skip_to_designing();
    assert!(q.is_done());
    assert_eq!(
        q.outstanding(),
        vec![
            StepId::Goal,
            StepId::Tools,
            StepId::Deadline,
            StepId::Budget,
            StepId::Inspiration,
        ]
    );

    // Row 10: Goal filled in later; intent no longer missing.
    assert_eq!(
        q.answer(
            StepId::Goal,
            Answer::Text("A four-drawer dresser".to_string())
        ),
        Ok(())
    );
    assert!(!q.intent_missing());
    assert_eq!(
        q.outstanding(),
        vec![
            StepId::Tools,
            StepId::Deadline,
            StepId::Budget,
            StepId::Inspiration,
        ]
    );

    // Row 11: Review is already confirmed.
    assert_eq!(
        q.answer(StepId::Review, Answer::Confirmed),
        Err(QuestError::AlreadyConfirmed)
    );

    // Row 12: a whitespace-only goal is still missing.
    assert_eq!(
        q.answer(StepId::Goal, Answer::Text("   ".to_string())),
        Ok(())
    );
    assert!(q.intent_missing());
}
