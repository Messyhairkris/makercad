//! Contract tests for mk_log::prompts, written by Gordon as a PA (the Crew) from
//! crew/contracts/mk_prompts.md. They were the hidden tests of campaign mk-core-18.

// Integration tests for the journal's prompt moments (doc 10 section 10.4).

use mk_log::journal::PromptMoment;
use mk_log::prompts::{Event, PromptError, Prompts, State, MOMENTS};

const ALL_STATES: [State; 4] = [State::NotDue, State::Due, State::Open, State::Answered];

fn index_of(moment: PromptMoment) -> usize {
    match MOMENTS.iter().position(|known| *known == moment) {
        Some(index) => index,
        None => MOMENTS.len(),
    }
}

/// A ledger whose moments are all NotDue except `moment`, which holds `state`.
fn ledger(moment: PromptMoment, state: State, material_seen: bool) -> Prompts {
    let mut states = [State::NotDue; 7];
    states[index_of(moment)] = state;
    Prompts::restore(states, material_seen)
}

#[test]
fn new_is_all_not_due_and_has_no_material() {
    let prompts = Prompts::new();
    assert!(prompts.states() == [State::NotDue; 7]);
    assert!(!prompts.material_seen());
    assert!(prompts.due().is_empty());
    assert!(prompts.open_pages().is_empty());
}

#[test]
fn default_is_the_same_as_new() {
    assert!(Prompts::default() == Prompts::new());
}

#[test]
fn moments_are_the_seven_in_doc_order() {
    assert!(
        MOMENTS
            == [
                PromptMoment::ProjectCreation,
                PromptMoment::FirstMaterial,
                PromptMoment::FirstFitToSpace,
                PromptMoment::DesignChangeAfterMaterial,
                PromptMoment::FirstPhysicalTestFit,
                PromptMoment::SessionEnd,
                PromptMoment::Conclusion,
            ]
    );
}

#[test]
fn s4_state_reports_each_moment_and_states_agrees() {
    let states = [
        State::Answered,
        State::Open,
        State::Due,
        State::NotDue,
        State::Answered,
        State::Open,
        State::Due,
    ];
    let prompts = Prompts::restore(states, true);
    assert!(prompts.states() == states);
    for (index, moment) in MOMENTS.iter().enumerate() {
        assert!(prompts.state(*moment) == states[index]);
    }
}

#[test]
fn restore_takes_the_states_and_flag_as_given() {
    let states = [
        State::Due,
        State::NotDue,
        State::Open,
        State::Answered,
        State::NotDue,
        State::Due,
        State::NotDue,
    ];
    let prompts = Prompts::restore(states, false);
    assert!(prompts.states() == states);
    assert!(!prompts.material_seen());
    assert!(prompts.due() == vec![PromptMoment::ProjectCreation, PromptMoment::SessionEnd]);
}

#[test]
fn o1_project_created_is_due_only_from_not_due() {
    for state in ALL_STATES.iter().copied() {
        let mut prompts = ledger(PromptMoment::ProjectCreation, state, false);
        if state == State::NotDue {
            assert!(prompts.observe(Event::ProjectCreated) == Some(PromptMoment::ProjectCreation));
            assert!(prompts.state(PromptMoment::ProjectCreation) == State::Due);
        } else {
            assert!(prompts.observe(Event::ProjectCreated).is_none());
            assert!(prompts.state(PromptMoment::ProjectCreation) == state);
        }
    }
}

#[test]
fn o1_material_chosen_is_due_only_from_not_due_and_remembers_the_material() {
    for state in ALL_STATES.iter().copied() {
        let mut prompts = ledger(PromptMoment::FirstMaterial, state, false);
        if state == State::NotDue {
            assert!(prompts.observe(Event::MaterialChosen) == Some(PromptMoment::FirstMaterial));
            assert!(prompts.state(PromptMoment::FirstMaterial) == State::Due);
        } else {
            assert!(prompts.observe(Event::MaterialChosen).is_none());
            assert!(prompts.state(PromptMoment::FirstMaterial) == state);
        }
        // The material is chosen, whatever became of the prompt.
        assert!(prompts.material_seen());
    }
}

#[test]
fn o1_material_chosen_sets_the_flag_on_an_answered_moment() {
    let mut prompts = ledger(PromptMoment::FirstMaterial, State::Answered, false);
    assert!(prompts.observe(Event::MaterialChosen).is_none());
    assert!(prompts.material_seen());
    assert!(prompts.state(PromptMoment::FirstMaterial) == State::Answered);
}

#[test]
fn o1_fit_to_space_is_due_only_from_not_due() {
    for state in ALL_STATES.iter().copied() {
        let mut prompts = ledger(PromptMoment::FirstFitToSpace, state, true);
        if state == State::NotDue {
            assert!(prompts.observe(Event::FitToSpace) == Some(PromptMoment::FirstFitToSpace));
            assert!(prompts.state(PromptMoment::FirstFitToSpace) == State::Due);
        } else {
            assert!(prompts.observe(Event::FitToSpace).is_none());
            assert!(prompts.state(PromptMoment::FirstFitToSpace) == state);
        }
    }
}

#[test]
fn o1_design_changed_after_material_is_due_only_from_not_due() {
    for state in ALL_STATES.iter().copied() {
        let mut prompts = ledger(PromptMoment::DesignChangeAfterMaterial, state, true);
        if state == State::NotDue {
            assert!(
                prompts.observe(Event::DesignChanged)
                    == Some(PromptMoment::DesignChangeAfterMaterial)
            );
            assert!(prompts.state(PromptMoment::DesignChangeAfterMaterial) == State::Due);
        } else {
            assert!(prompts.observe(Event::DesignChanged).is_none());
            assert!(prompts.state(PromptMoment::DesignChangeAfterMaterial) == state);
        }
    }
}

#[test]
fn o1_design_changed_before_any_material_changes_nothing() {
    let mut prompts = Prompts::new();
    assert!(prompts.observe(Event::DesignChanged).is_none());
    assert!(!prompts.material_seen());
    assert!(prompts.states() == [State::NotDue; 7]);
}

#[test]
fn o1_design_changed_becomes_due_after_a_later_material_choice() {
    let mut prompts = Prompts::new();
    assert!(prompts.observe(Event::DesignChanged).is_none());
    assert!(prompts.observe(Event::MaterialChosen) == Some(PromptMoment::FirstMaterial));
    assert!(prompts.observe(Event::DesignChanged) == Some(PromptMoment::DesignChangeAfterMaterial));
    assert!(prompts.state(PromptMoment::DesignChangeAfterMaterial) == State::Due);
}

#[test]
fn o1_test_fit_is_due_only_from_not_due() {
    for state in ALL_STATES.iter().copied() {
        let mut prompts = ledger(PromptMoment::FirstPhysicalTestFit, state, false);
        if state == State::NotDue {
            assert!(prompts.observe(Event::TestFit) == Some(PromptMoment::FirstPhysicalTestFit));
            assert!(prompts.state(PromptMoment::FirstPhysicalTestFit) == State::Due);
        } else {
            assert!(prompts.observe(Event::TestFit).is_none());
            assert!(prompts.state(PromptMoment::FirstPhysicalTestFit) == state);
        }
    }
}

#[test]
fn o1_session_end_is_due_from_every_state_but_due() {
    for state in ALL_STATES.iter().copied() {
        let mut prompts = ledger(PromptMoment::SessionEnd, state, true);
        if state == State::Due {
            assert!(prompts.observe(Event::SessionEnded).is_none());
            assert!(prompts.state(PromptMoment::SessionEnd) == State::Due);
        } else {
            assert!(prompts.observe(Event::SessionEnded) == Some(PromptMoment::SessionEnd));
            assert!(prompts.state(PromptMoment::SessionEnd) == State::Due);
        }
    }
}

#[test]
fn o1_concluded_is_due_only_from_not_due() {
    for state in ALL_STATES.iter().copied() {
        let mut prompts = ledger(PromptMoment::Conclusion, state, false);
        if state == State::NotDue {
            assert!(prompts.observe(Event::Concluded) == Some(PromptMoment::Conclusion));
            assert!(prompts.state(PromptMoment::Conclusion) == State::Due);
        } else {
            assert!(prompts.observe(Event::Concluded).is_none());
            assert!(prompts.state(PromptMoment::Conclusion) == state);
        }
    }
}

#[test]
fn a_first_moment_is_asked_at_most_once() {
    let pairs = [
        (Event::ProjectCreated, PromptMoment::ProjectCreation),
        (Event::MaterialChosen, PromptMoment::FirstMaterial),
        (Event::FitToSpace, PromptMoment::FirstFitToSpace),
        (
            Event::DesignChanged,
            PromptMoment::DesignChangeAfterMaterial,
        ),
        (Event::TestFit, PromptMoment::FirstPhysicalTestFit),
        (Event::Concluded, PromptMoment::Conclusion),
    ];
    for (event, moment) in pairs.iter().copied() {
        // Material seen, so DesignChanged counts as "after material picked".
        let mut prompts = Prompts::restore([State::NotDue; 7], true);
        assert!(prompts.observe(event) == Some(moment));
        assert!(prompts.answer(moment).is_ok());
        assert!(prompts.observe(event).is_none());
        assert!(prompts.state(moment) == State::Answered);
    }
}

#[test]
fn skipping_does_not_make_a_first_moment_due_again() {
    let mut prompts = Prompts::new();
    assert!(prompts.observe(Event::ProjectCreated) == Some(PromptMoment::ProjectCreation));
    assert!(prompts.skip(PromptMoment::ProjectCreation).is_ok());
    assert!(prompts.observe(Event::ProjectCreated).is_none());
    assert!(prompts.state(PromptMoment::ProjectCreation) == State::Open);
}

#[test]
fn s1_skip_turns_due_into_open() {
    let mut prompts = ledger(PromptMoment::FirstFitToSpace, State::Due, false);
    assert!(prompts.skip(PromptMoment::FirstFitToSpace).is_ok());
    assert!(prompts.state(PromptMoment::FirstFitToSpace) == State::Open);
    assert!(prompts.open_pages() == vec![PromptMoment::FirstFitToSpace]);
    assert!(prompts.due().is_empty());
}

#[test]
fn s1_skip_an_open_page_is_already_open() {
    let mut prompts = ledger(PromptMoment::SessionEnd, State::Open, false);
    assert!(prompts.skip(PromptMoment::SessionEnd) == Err(PromptError::AlreadyOpen));
    assert!(prompts.state(PromptMoment::SessionEnd) == State::Open);
}

#[test]
fn s1_skip_a_moment_not_due_is_refused() {
    let mut prompts = Prompts::new();
    assert!(prompts.skip(PromptMoment::Conclusion) == Err(PromptError::NotDue));
    assert!(prompts.state(PromptMoment::Conclusion) == State::NotDue);
}

#[test]
fn s1_skip_an_answered_moment_is_already_answered() {
    let mut prompts = ledger(PromptMoment::FirstMaterial, State::Answered, true);
    assert!(prompts.skip(PromptMoment::FirstMaterial) == Err(PromptError::AlreadyAnswered));
    assert!(prompts.state(PromptMoment::FirstMaterial) == State::Answered);
}

#[test]
fn s1_refused_skips_leave_the_ledger_alone() {
    let refused = [
        (State::NotDue, PromptError::NotDue),
        (State::Open, PromptError::AlreadyOpen),
        (State::Answered, PromptError::AlreadyAnswered),
    ];
    for (state, error) in refused.iter().copied() {
        let mut prompts = ledger(PromptMoment::FirstPhysicalTestFit, state, true);
        let before = prompts.clone();
        assert!(prompts.skip(PromptMoment::FirstPhysicalTestFit) == Err(error));
        assert!(prompts == before);
        assert!(prompts.state(PromptMoment::FirstPhysicalTestFit) == state);
        assert!(prompts.due().is_empty());
        assert!(prompts.material_seen());
    }
}

#[test]
fn s2_answer_turns_due_into_answered() {
    let mut prompts = ledger(PromptMoment::FirstFitToSpace, State::Due, false);
    assert!(prompts.answer(PromptMoment::FirstFitToSpace).is_ok());
    assert!(prompts.state(PromptMoment::FirstFitToSpace) == State::Answered);
    assert!(prompts.due().is_empty());
}

#[test]
fn s2_answer_closes_an_open_page() {
    let mut prompts = ledger(PromptMoment::FirstMaterial, State::Due, true);
    assert!(prompts.skip(PromptMoment::FirstMaterial).is_ok());
    assert!(prompts.open_pages() == vec![PromptMoment::FirstMaterial]);
    let before = prompts.clone();
    assert!(prompts.answer(PromptMoment::FirstMaterial).is_ok());
    assert!(prompts.state(PromptMoment::FirstMaterial) == State::Answered);
    assert!(prompts.open_pages().is_empty());
    assert!(prompts != before);
}

#[test]
fn s2_answer_a_moment_not_due_is_refused() {
    let mut prompts = Prompts::new();
    assert!(prompts.answer(PromptMoment::FirstPhysicalTestFit) == Err(PromptError::NotDue));
    assert!(prompts.state(PromptMoment::FirstPhysicalTestFit) == State::NotDue);
}

#[test]
fn s2_answer_an_answered_moment_is_already_answered() {
    let mut prompts = ledger(PromptMoment::Conclusion, State::Answered, true);
    assert!(prompts.answer(PromptMoment::Conclusion) == Err(PromptError::AlreadyAnswered));
    assert!(prompts.state(PromptMoment::Conclusion) == State::Answered);
}

#[test]
fn s2_refused_answers_leave_the_ledger_alone() {
    let refused = [
        (State::NotDue, PromptError::NotDue),
        (State::Answered, PromptError::AlreadyAnswered),
    ];
    for (state, error) in refused.iter().copied() {
        let mut prompts = ledger(PromptMoment::ProjectCreation, state, false);
        let before = prompts.clone();
        assert!(prompts.answer(PromptMoment::ProjectCreation) == Err(error));
        assert!(prompts == before);
        assert!(prompts.state(PromptMoment::ProjectCreation) == state);
        assert!(prompts.open_pages().is_empty());
    }
}

#[test]
fn s2_an_answered_open_page_can_no_longer_be_skipped() {
    let mut prompts = ledger(PromptMoment::SessionEnd, State::Open, true);
    assert!(prompts.answer(PromptMoment::SessionEnd).is_ok());
    assert!(prompts.skip(PromptMoment::SessionEnd) == Err(PromptError::AlreadyAnswered));
    assert!(prompts.state(PromptMoment::SessionEnd) == State::Answered);
}

#[test]
fn s3_due_lists_the_due_moments_in_moments_order() {
    let states = [
        State::Due,
        State::NotDue,
        State::Due,
        State::NotDue,
        State::Open,
        State::Due,
        State::Open,
    ];
    let prompts = Prompts::restore(states, false);
    assert!(
        prompts.due()
            == vec![
                PromptMoment::ProjectCreation,
                PromptMoment::FirstFitToSpace,
                PromptMoment::SessionEnd,
            ]
    );
}

#[test]
fn s3_open_pages_list_them_in_moments_order() {
    let states = [
        State::Open,
        State::NotDue,
        State::Open,
        State::NotDue,
        State::Open,
        State::Open,
        State::Open,
    ];
    let prompts = Prompts::restore(states, false);
    assert!(
        prompts.open_pages()
            == vec![
                PromptMoment::ProjectCreation,
                PromptMoment::FirstFitToSpace,
                PromptMoment::FirstPhysicalTestFit,
                PromptMoment::SessionEnd,
                PromptMoment::Conclusion,
            ]
    );
    assert!(prompts.due().is_empty());
}

#[test]
fn a_skipped_page_survives_other_events_until_answered() {
    let mut prompts = Prompts::new();
    assert!(prompts.observe(Event::MaterialChosen) == Some(PromptMoment::FirstMaterial));
    assert!(prompts.skip(PromptMoment::FirstMaterial).is_ok());
    assert!(prompts.observe(Event::FitToSpace) == Some(PromptMoment::FirstFitToSpace));
    assert!(prompts.observe(Event::DesignChanged) == Some(PromptMoment::DesignChangeAfterMaterial));
    assert!(prompts.state(PromptMoment::FirstMaterial) == State::Open);
    assert!(prompts.open_pages() == vec![PromptMoment::FirstMaterial]);
    assert!(prompts.answer(PromptMoment::FirstMaterial).is_ok());
    assert!(prompts.open_pages().is_empty());
    assert!(prompts.state(PromptMoment::FirstMaterial) == State::Answered);
}

#[test]
fn an_earlier_sessions_skip_does_not_block_the_next_session_end() {
    let mut prompts = Prompts::new();
    assert!(prompts.observe(Event::SessionEnded) == Some(PromptMoment::SessionEnd));
    assert!(prompts.skip(PromptMoment::SessionEnd).is_ok());
    assert!(prompts.state(PromptMoment::SessionEnd) == State::Open);
    assert!(prompts.observe(Event::SessionEnded) == Some(PromptMoment::SessionEnd));
    assert!(prompts.state(PromptMoment::SessionEnd) == State::Due);
    assert!(prompts.open_pages().is_empty());
    assert!(prompts.answer(PromptMoment::SessionEnd).is_ok());
    assert!(prompts.observe(Event::SessionEnded) == Some(PromptMoment::SessionEnd));
}

/// The worked example's rows 1 to 18, in order, each result and each "then".
fn ledger_after_worked_example_row_18() -> Prompts {
    let mut prompts = Prompts::new();

    // 1: a design change before any material changes nothing.
    assert!(prompts.observe(Event::DesignChanged).is_none());
    assert!(!prompts.material_seen());

    // 2: the project is created.
    assert!(prompts.observe(Event::ProjectCreated) == Some(PromptMoment::ProjectCreation));
    assert!(prompts.due() == vec![PromptMoment::ProjectCreation]);

    // 3: a second creation is not due again.
    assert!(prompts.observe(Event::ProjectCreated).is_none());
    assert!(prompts.due() == vec![PromptMoment::ProjectCreation]);

    // 4: skipping makes an open page.
    assert!(prompts.skip(PromptMoment::ProjectCreation).is_ok());
    assert!(prompts.open_pages() == vec![PromptMoment::ProjectCreation]);
    assert!(prompts.due().is_empty());

    // 5: skipping an open page is refused.
    assert!(prompts.skip(PromptMoment::ProjectCreation) == Err(PromptError::AlreadyOpen));
    assert!(prompts.open_pages() == vec![PromptMoment::ProjectCreation]);

    // 6: the first material.
    assert!(prompts.observe(Event::MaterialChosen) == Some(PromptMoment::FirstMaterial));
    assert!(prompts.material_seen());
    assert!(prompts.due() == vec![PromptMoment::FirstMaterial]);

    // 7: a second material is not a second prompt.
    assert!(prompts.observe(Event::MaterialChosen).is_none());

    // 8: now a design change is "after material picked".
    assert!(prompts.observe(Event::DesignChanged) == Some(PromptMoment::DesignChangeAfterMaterial));
    assert!(
        prompts.due()
            == vec![
                PromptMoment::FirstMaterial,
                PromptMoment::DesignChangeAfterMaterial,
            ]
    );

    // 9: answering the material prompt.
    assert!(prompts.answer(PromptMoment::FirstMaterial).is_ok());
    assert!(prompts.due() == vec![PromptMoment::DesignChangeAfterMaterial]);

    // 10: answering twice is refused and changes nothing.
    assert!(prompts.answer(PromptMoment::FirstMaterial) == Err(PromptError::AlreadyAnswered));
    assert!(prompts.state(PromptMoment::FirstMaterial) == State::Answered);

    // 11: the design-change moment is asked once only.
    assert!(prompts.observe(Event::DesignChanged).is_none());

    // 12: answering a moment that is not due is refused.
    assert!(prompts.answer(PromptMoment::FirstFitToSpace) == Err(PromptError::NotDue));
    assert!(prompts.state(PromptMoment::FirstFitToSpace) == State::NotDue);

    // 13: the first session ends.
    assert!(prompts.observe(Event::SessionEnded) == Some(PromptMoment::SessionEnd));
    assert!(
        prompts.due()
            == vec![
                PromptMoment::DesignChangeAfterMaterial,
                PromptMoment::SessionEnd,
            ]
    );

    // 14: the same session ending again leaves it Due.
    assert!(prompts.observe(Event::SessionEnded).is_none());
    assert!(prompts.state(PromptMoment::SessionEnd) == State::Due);

    // 15: the interview is answered.
    assert!(prompts.answer(PromptMoment::SessionEnd).is_ok());
    assert!(prompts.state(PromptMoment::SessionEnd) == State::Answered);

    // 16: a new session is due again.
    assert!(prompts.observe(Event::SessionEnded) == Some(PromptMoment::SessionEnd));
    assert!(
        prompts.due()
            == vec![
                PromptMoment::DesignChangeAfterMaterial,
                PromptMoment::SessionEnd,
            ]
    );

    // 17: the open page is answered.
    assert!(prompts.answer(PromptMoment::ProjectCreation).is_ok());
    assert!(prompts.open_pages().is_empty());

    // 18: the conclusion.
    assert!(prompts.observe(Event::Concluded) == Some(PromptMoment::Conclusion));
    assert!(
        prompts.due()
            == vec![
                PromptMoment::DesignChangeAfterMaterial,
                PromptMoment::SessionEnd,
                PromptMoment::Conclusion,
            ]
    );

    prompts
}

#[test]
fn worked_example_ends_in_its_documented_states() {
    let prompts = ledger_after_worked_example_row_18();

    // 19: the ledger's states, in MOMENTS order.
    assert!(
        prompts.states()
            == [
                State::Answered,
                State::Answered,
                State::NotDue,
                State::Due,
                State::NotDue,
                State::Due,
                State::Due,
            ]
    );
    assert!(prompts.material_seen());
    assert!(prompts.open_pages().is_empty());
    assert!(prompts.due().len() == 3);
}

#[test]
fn restore_round_trip_matches_the_ledger_after_row_18() {
    let prompts = ledger_after_worked_example_row_18();
    let restored = Prompts::restore(prompts.states(), prompts.material_seen());

    assert!(restored == prompts);
    assert!(restored.material_seen());
    assert!(restored.states() == prompts.states());
    for (index, moment) in MOMENTS.iter().enumerate() {
        assert!(restored.state(*moment) == prompts.state(*moment));
        assert!(restored.state(*moment) == prompts.states()[index]);
    }
    assert!(restored.due() == prompts.due());
    assert!(restored.open_pages() == prompts.open_pages());
}
