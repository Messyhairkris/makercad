//! Tests for mk_log::prompts written by Qwen (the Crew, campaign mk-core-18) from the
//! contract alone, without seeing the module.

//! Tests for the prompts module: the journal's prompt moments (contract v1).

use mk_log::journal::PromptMoment;
use mk_log::prompts::{Event, PromptError, Prompts, State, MOMENTS};

/// A starting ledger with one moment in `state` and the rest NotDue.
fn one_state(moment: PromptMoment, state: State) -> [State; 7] {
    let mut states = [State::NotDue; 7];
    states[moment as usize] = state;
    states
}

#[test]
fn rule_o1_project_created_every_state() {
    // NotDue: the moment becomes Due and is returned.
    let mut p = Prompts::new();
    assert_eq!(
        p.observe(Event::ProjectCreated),
        Some(PromptMoment::ProjectCreation)
    );
    assert_eq!(p.state(PromptMoment::ProjectCreation), State::Due);

    // Due: already asked, so it is not due again.
    let mut p = Prompts::restore(one_state(PromptMoment::ProjectCreation, State::Due), false);
    assert_eq!(p.observe(Event::ProjectCreated), None);
    assert_eq!(p.state(PromptMoment::ProjectCreation), State::Due);

    // Open: a skipped page lives on, so it is not due again.
    let mut p = Prompts::restore(one_state(PromptMoment::ProjectCreation, State::Open), false);
    assert_eq!(p.observe(Event::ProjectCreated), None);
    assert_eq!(p.state(PromptMoment::ProjectCreation), State::Open);

    // Answered: asked at most once per project.
    let mut p = Prompts::restore(
        one_state(PromptMoment::ProjectCreation, State::Answered),
        false,
    );
    assert_eq!(p.observe(Event::ProjectCreated), None);
    assert_eq!(p.state(PromptMoment::ProjectCreation), State::Answered);
}

#[test]
fn rule_o1_material_chosen_every_state() {
    // NotDue: the moment becomes Due and material_seen becomes true.
    let mut p = Prompts::new();
    assert_eq!(
        p.observe(Event::MaterialChosen),
        Some(PromptMoment::FirstMaterial)
    );
    assert_eq!(p.state(PromptMoment::FirstMaterial), State::Due);
    assert!(p.material_seen());

    // NotDue with material_seen already true: it still becomes Due.
    let mut p = Prompts::restore(one_state(PromptMoment::FirstMaterial, State::NotDue), true);
    assert_eq!(
        p.observe(Event::MaterialChosen),
        Some(PromptMoment::FirstMaterial)
    );
    assert!(p.material_seen());

    // Due: not due again, but material_seen becomes true (always).
    let mut p = Prompts::restore(one_state(PromptMoment::FirstMaterial, State::Due), false);
    assert_eq!(p.observe(Event::MaterialChosen), None);
    assert_eq!(p.state(PromptMoment::FirstMaterial), State::Due);
    assert!(p.material_seen());

    // Open: not due again, but material_seen becomes true (always).
    let mut p = Prompts::restore(one_state(PromptMoment::FirstMaterial, State::Open), false);
    assert_eq!(p.observe(Event::MaterialChosen), None);
    assert_eq!(p.state(PromptMoment::FirstMaterial), State::Open);
    assert!(p.material_seen());

    // Answered: not due again, but material_seen becomes true (always).
    let mut p = Prompts::restore(
        one_state(PromptMoment::FirstMaterial, State::Answered),
        false,
    );
    assert_eq!(p.observe(Event::MaterialChosen), None);
    assert_eq!(p.state(PromptMoment::FirstMaterial), State::Answered);
    assert!(p.material_seen());
}

#[test]
fn rule_o1_fit_to_space_every_state() {
    let moment = PromptMoment::FirstFitToSpace;

    // NotDue: the moment becomes Due and is returned.
    let mut p = Prompts::new();
    assert_eq!(p.observe(Event::FitToSpace), Some(moment));
    assert_eq!(p.state(moment), State::Due);

    // Due: already asked, so it is not due again.
    let mut p = Prompts::restore(one_state(moment, State::Due), false);
    assert_eq!(p.observe(Event::FitToSpace), None);
    assert_eq!(p.state(moment), State::Due);

    // Open: a skipped page lives on, so it is not due again.
    let mut p = Prompts::restore(one_state(moment, State::Open), false);
    assert_eq!(p.observe(Event::FitToSpace), None);
    assert_eq!(p.state(moment), State::Open);

    // Answered: asked at most once per project.
    let mut p = Prompts::restore(one_state(moment, State::Answered), false);
    assert_eq!(p.observe(Event::FitToSpace), None);
    assert_eq!(p.state(moment), State::Answered);
}

#[test]
fn rule_o1_design_changed_every_state() {
    let moment = PromptMoment::DesignChangeAfterMaterial;

    // material_seen false: not "after material picked", so nothing happens.
    let mut p = Prompts::new();
    assert_eq!(p.observe(Event::DesignChanged), None);
    assert_eq!(p.state(moment), State::NotDue);
    assert!(!p.material_seen());

    // material_seen true, NotDue: the moment becomes Due.
    let mut p = Prompts::restore(one_state(moment, State::NotDue), true);
    assert_eq!(p.observe(Event::DesignChanged), Some(moment));
    assert_eq!(p.state(moment), State::Due);

    // material_seen true, Due: already asked, so it is not due again.
    let mut p = Prompts::restore(one_state(moment, State::Due), true);
    assert_eq!(p.observe(Event::DesignChanged), None);
    assert_eq!(p.state(moment), State::Due);

    // material_seen true, Open: a skipped page lives on.
    let mut p = Prompts::restore(one_state(moment, State::Open), true);
    assert_eq!(p.observe(Event::DesignChanged), None);
    assert_eq!(p.state(moment), State::Open);

    // material_seen true, Answered: asked at most once per project.
    let mut p = Prompts::restore(one_state(moment, State::Answered), true);
    assert_eq!(p.observe(Event::DesignChanged), None);
    assert_eq!(p.state(moment), State::Answered);
}

#[test]
fn rule_o1_test_fit_every_state() {
    let moment = PromptMoment::FirstPhysicalTestFit;

    // NotDue: the moment becomes Due and is returned.
    let mut p = Prompts::new();
    assert_eq!(p.observe(Event::TestFit), Some(moment));
    assert_eq!(p.state(moment), State::Due);

    // Due: already asked, so it is not due again.
    let mut p = Prompts::restore(one_state(moment, State::Due), false);
    assert_eq!(p.observe(Event::TestFit), None);
    assert_eq!(p.state(moment), State::Due);

    // Open: a skipped page lives on, so it is not due again.
    let mut p = Prompts::restore(one_state(moment, State::Open), false);
    assert_eq!(p.observe(Event::TestFit), None);
    assert_eq!(p.state(moment), State::Open);

    // Answered: asked at most once per project.
    let mut p = Prompts::restore(one_state(moment, State::Answered), false);
    assert_eq!(p.observe(Event::TestFit), None);
    assert_eq!(p.state(moment), State::Answered);
}

#[test]
fn rule_o1_session_ended_every_state() {
    let moment = PromptMoment::SessionEnd;

    // NotDue: every session ends with its interview, so it becomes Due.
    let mut p = Prompts::new();
    assert_eq!(p.observe(Event::SessionEnded), Some(moment));
    assert_eq!(p.state(moment), State::Due);

    // Due: it is still due, so it is not due again.
    let mut p = Prompts::restore(one_state(moment, State::Due), false);
    assert_eq!(p.observe(Event::SessionEnded), None);
    assert_eq!(p.state(moment), State::Due);

    // Open: anything but Due, so a new session makes it Due again.
    let mut p = Prompts::restore(one_state(moment, State::Open), false);
    assert_eq!(p.observe(Event::SessionEnded), Some(moment));
    assert_eq!(p.state(moment), State::Due);

    // Answered: it recurs, so a new session makes it Due again.
    let mut p = Prompts::restore(one_state(moment, State::Answered), false);
    assert_eq!(p.observe(Event::SessionEnded), Some(moment));
    assert_eq!(p.state(moment), State::Due);
}

#[test]
fn rule_o1_concluded_every_state() {
    let moment = PromptMoment::Conclusion;

    // NotDue: the moment becomes Due and is returned.
    let mut p = Prompts::new();
    assert_eq!(p.observe(Event::Concluded), Some(moment));
    assert_eq!(p.state(moment), State::Due);

    // Due: already asked, so it is not due again.
    let mut p = Prompts::restore(one_state(moment, State::Due), false);
    assert_eq!(p.observe(Event::Concluded), None);
    assert_eq!(p.state(moment), State::Due);

    // Open: a skipped page lives on, so it is not due again.
    let mut p = Prompts::restore(one_state(moment, State::Open), false);
    assert_eq!(p.observe(Event::Concluded), None);
    assert_eq!(p.state(moment), State::Open);

    // Answered: asked at most once per project.
    let mut p = Prompts::restore(one_state(moment, State::Answered), false);
    assert_eq!(p.observe(Event::Concluded), None);
    assert_eq!(p.state(moment), State::Answered);
}

#[test]
fn rule_s1_skip() {
    let moment = PromptMoment::ProjectCreation;

    // Due becomes Open.
    let mut p = Prompts::restore(one_state(moment, State::Due), false);
    assert_eq!(p.skip(moment), Ok(()));
    assert_eq!(p.state(moment), State::Open);

    // Open is Err(AlreadyOpen); a refused call changes nothing.
    let mut p = Prompts::restore(one_state(moment, State::Open), false);
    assert_eq!(p.skip(moment), Err(PromptError::AlreadyOpen));
    assert_eq!(p.state(moment), State::Open);

    // NotDue is Err(NotDue); a refused call changes nothing.
    let mut p = Prompts::new();
    assert_eq!(p.skip(moment), Err(PromptError::NotDue));
    assert_eq!(p.state(moment), State::NotDue);

    // Answered is Err(AlreadyAnswered); a refused call changes nothing.
    let mut p = Prompts::restore(one_state(moment, State::Answered), false);
    assert_eq!(p.skip(moment), Err(PromptError::AlreadyAnswered));
    assert_eq!(p.state(moment), State::Answered);
}

#[test]
fn rule_s2_answer() {
    let moment = PromptMoment::FirstMaterial;

    // Due becomes Answered.
    let mut p = Prompts::restore(one_state(moment, State::Due), true);
    assert_eq!(p.answer(moment), Ok(()));
    assert_eq!(p.state(moment), State::Answered);

    // Open becomes Answered: an open page stays open until answered.
    let mut p = Prompts::restore(one_state(moment, State::Open), true);
    assert_eq!(p.answer(moment), Ok(()));
    assert_eq!(p.state(moment), State::Answered);

    // NotDue is Err(NotDue); a refused call changes nothing.
    let mut p = Prompts::new();
    assert_eq!(p.answer(moment), Err(PromptError::NotDue));
    assert_eq!(p.state(moment), State::NotDue);

    // Answered is Err(AlreadyAnswered); a refused call changes nothing.
    let mut p = Prompts::restore(one_state(moment, State::Answered), true);
    assert_eq!(p.answer(moment), Err(PromptError::AlreadyAnswered));
    assert_eq!(p.state(moment), State::Answered);
}

#[test]
fn rule_s3_due_in_moments_order() {
    // Due moments set in reverse order come back in MOMENTS order.
    let mut states = [State::NotDue; 7];
    states[PromptMoment::Conclusion as usize] = State::Due;
    states[PromptMoment::SessionEnd as usize] = State::Due;
    states[PromptMoment::FirstMaterial as usize] = State::Due;
    let p = Prompts::restore(states, true);
    assert_eq!(
        p.due(),
        vec![
            PromptMoment::FirstMaterial,
            PromptMoment::SessionEnd,
            PromptMoment::Conclusion,
        ]
    );

    // A fresh ledger has nothing due.
    assert_eq!(Prompts::new().due(), vec![]);
}

#[test]
fn rule_s3_open_pages_in_moments_order() {
    // Open moments set in reverse order come back in MOMENTS order.
    let mut states = [State::NotDue; 7];
    states[PromptMoment::Conclusion as usize] = State::Open;
    states[PromptMoment::FirstFitToSpace as usize] = State::Open;
    states[PromptMoment::ProjectCreation as usize] = State::Open;
    let p = Prompts::restore(states, false);
    assert_eq!(
        p.open_pages(),
        vec![
            PromptMoment::ProjectCreation,
            PromptMoment::FirstFitToSpace,
            PromptMoment::Conclusion,
        ]
    );

    // A fresh ledger has no open pages.
    assert_eq!(Prompts::new().open_pages(), vec![]);
}

#[test]
fn rule_s4_state_and_states() {
    let states = [
        State::Answered,
        State::Due,
        State::Open,
        State::NotDue,
        State::Answered,
        State::Due,
        State::Open,
    ];
    let p = Prompts::restore(states, true);

    // states() is all seven in MOMENTS order.
    assert_eq!(p.states(), states);

    // state(moment) is that moment's state.
    assert_eq!(p.state(PromptMoment::ProjectCreation), State::Answered);
    assert_eq!(p.state(PromptMoment::FirstMaterial), State::Due);
    assert_eq!(p.state(PromptMoment::FirstFitToSpace), State::Open);
    assert_eq!(
        p.state(PromptMoment::DesignChangeAfterMaterial),
        State::NotDue
    );
    assert_eq!(p.state(PromptMoment::FirstPhysicalTestFit), State::Answered);
    assert_eq!(p.state(PromptMoment::SessionEnd), State::Due);
    assert_eq!(p.state(PromptMoment::Conclusion), State::Open);
    assert!(p.material_seen());
}

#[test]
fn worked_example_in_order() {
    let mut p = Prompts::new();

    // 1: a design change before any material is not "after material picked".
    assert_eq!(p.observe(Event::DesignChanged), None);
    assert!(!p.material_seen());

    // 2: project creation becomes due.
    assert_eq!(
        p.observe(Event::ProjectCreated),
        Some(PromptMoment::ProjectCreation)
    );
    assert_eq!(p.due(), vec![PromptMoment::ProjectCreation]);

    // 3: it is not due again.
    assert_eq!(p.observe(Event::ProjectCreated), None);

    // 4: skipping opens the page.
    assert_eq!(p.skip(PromptMoment::ProjectCreation), Ok(()));
    assert_eq!(p.open_pages(), vec![PromptMoment::ProjectCreation]);
    assert_eq!(p.due(), vec![]);

    // 5: skipping an open page is refused.
    assert_eq!(
        p.skip(PromptMoment::ProjectCreation),
        Err(PromptError::AlreadyOpen)
    );

    // 6: the first material becomes due; material_seen is true from then on.
    assert_eq!(
        p.observe(Event::MaterialChosen),
        Some(PromptMoment::FirstMaterial)
    );
    assert!(p.material_seen());

    // 7: it is not due again.
    assert_eq!(p.observe(Event::MaterialChosen), None);

    // 8: a design change after material picked becomes due.
    assert_eq!(
        p.observe(Event::DesignChanged),
        Some(PromptMoment::DesignChangeAfterMaterial)
    );
    assert_eq!(
        p.due(),
        vec![
            PromptMoment::FirstMaterial,
            PromptMoment::DesignChangeAfterMaterial
        ]
    );

    // 9: answering the first material closes it.
    assert_eq!(p.answer(PromptMoment::FirstMaterial), Ok(()));
    assert_eq!(p.due(), vec![PromptMoment::DesignChangeAfterMaterial]);

    // 10: answering it again is refused.
    assert_eq!(
        p.answer(PromptMoment::FirstMaterial),
        Err(PromptError::AlreadyAnswered)
    );

    // 11: the design-change moment is already due, so it is not due again.
    assert_eq!(p.observe(Event::DesignChanged), None);

    // 12: answering a moment that is not due is refused.
    assert_eq!(
        p.answer(PromptMoment::FirstFitToSpace),
        Err(PromptError::NotDue)
    );

    // 13: a session ends with its interview.
    assert_eq!(
        p.observe(Event::SessionEnded),
        Some(PromptMoment::SessionEnd)
    );

    // 14: while due, it is not due again.
    assert_eq!(p.observe(Event::SessionEnded), None);

    // 15: the interview is answered.
    assert_eq!(p.answer(PromptMoment::SessionEnd), Ok(()));

    // 16: a new session ends: due again.
    assert_eq!(
        p.observe(Event::SessionEnded),
        Some(PromptMoment::SessionEnd)
    );

    // 17: the open page from row 4 is answered.
    assert_eq!(p.answer(PromptMoment::ProjectCreation), Ok(()));
    assert_eq!(p.open_pages(), vec![]);

    // 18: the conclusion becomes due.
    assert_eq!(p.observe(Event::Concluded), Some(PromptMoment::Conclusion));
    assert_eq!(
        p.due(),
        vec![
            PromptMoment::DesignChangeAfterMaterial,
            PromptMoment::SessionEnd,
            PromptMoment::Conclusion,
        ]
    );

    // 19: the whole ledger.
    assert_eq!(
        p.states(),
        [
            State::Answered,
            State::Answered,
            State::NotDue,
            State::Due,
            State::NotDue,
            State::Due,
            State::Due,
        ]
    );
}

#[test]
fn restore_round_trip() {
    // Build a ledger, rebuild it from states() and material_seen(), compare whole.
    let mut p = Prompts::new();
    let _ = p.observe(Event::ProjectCreated);
    let _ = p.skip(PromptMoment::ProjectCreation);
    let _ = p.observe(Event::MaterialChosen);
    let _ = p.observe(Event::DesignChanged);
    let _ = p.answer(PromptMoment::FirstMaterial);
    let _ = p.observe(Event::SessionEnded);

    let restored = Prompts::restore(p.states(), p.material_seen());
    assert_eq!(restored, p);
    assert!(restored.material_seen());
}

#[test]
fn new_and_default_start_not_due() {
    let p = Prompts::new();
    assert_eq!(p.states(), [State::NotDue; 7]);
    assert!(!p.material_seen());
    assert_eq!(p.due(), vec![]);
    assert_eq!(p.open_pages(), vec![]);

    let d = Prompts::default();
    assert_eq!(d.states(), [State::NotDue; 7]);
    assert!(!d.material_seen());
}

#[test]
fn refused_calls_change_nothing() {
    // Every state is represented, so each refusal is checked against a full ledger.
    let states = [
        State::NotDue,
        State::Due,
        State::Open,
        State::Answered,
        State::NotDue,
        State::Due,
        State::Open,
    ];
    let mut p = Prompts::restore(states, true);
    let open = PromptMoment::FirstFitToSpace;
    let not_due = PromptMoment::FirstPhysicalTestFit;
    let answered = PromptMoment::DesignChangeAfterMaterial;

    // S1: skip is refused on Open, NotDue and Answered moments.
    assert_eq!(p.skip(open), Err(PromptError::AlreadyOpen));
    assert_eq!(p.skip(not_due), Err(PromptError::NotDue));
    assert_eq!(p.skip(answered), Err(PromptError::AlreadyAnswered));
    assert_eq!(p.states(), states);
    assert!(p.material_seen());

    // S2: answer is refused on NotDue and Answered moments.
    assert_eq!(p.answer(not_due), Err(PromptError::NotDue));
    assert_eq!(p.answer(answered), Err(PromptError::AlreadyAnswered));
    assert_eq!(p.states(), states);
    assert!(p.material_seen());

    // O1: events whose moment is not NotDue change nothing.
    assert_eq!(p.observe(Event::FitToSpace), None);
    assert_eq!(p.observe(Event::DesignChanged), None);
    assert_eq!(p.observe(Event::SessionEnded), None);
    assert_eq!(p.observe(Event::Concluded), None);
    assert_eq!(p.states(), states);
    assert!(p.material_seen());
}

#[test]
fn moments_constant_order() {
    assert_eq!(
        MOMENTS,
        [
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
fn rule_o1_design_changed_before_material_changes_nothing() {
    let mut p = Prompts::new();
    assert_eq!(p.observe(Event::DesignChanged), None);
    assert_eq!(p.observe(Event::DesignChanged), None);
    assert_eq!(p.states(), [State::NotDue; 7]);
    assert!(!p.material_seen());
    assert_eq!(p.due(), vec![]);
}

#[test]
fn rule_o1_session_ended_recurs_after_skip() {
    // A session ends, its interview is skipped, and the next session ends again.
    let mut p = Prompts::new();
    assert_eq!(
        p.observe(Event::SessionEnded),
        Some(PromptMoment::SessionEnd)
    );
    assert_eq!(p.skip(PromptMoment::SessionEnd), Ok(()));
    assert_eq!(p.open_pages(), vec![PromptMoment::SessionEnd]);
    assert_eq!(
        p.observe(Event::SessionEnded),
        Some(PromptMoment::SessionEnd)
    );
    assert_eq!(p.state(PromptMoment::SessionEnd), State::Due);
}
