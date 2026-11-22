//! The journal's prompt moments (doc 10 section 10.4): which are due, open or answered.
//!
//! Written by Hoss (the Crew, campaign mk-core-18, 2026-11-22) from the contract
//! crew/contracts/mk_prompts.md (ATLAS-api-harness), which Claude finished from Deepak's brief.

use crate::journal::PromptMoment;

/// What the app reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Event {
    ProjectCreated,
    MaterialChosen,
    FitToSpace,
    DesignChanged,
    TestFit,
    SessionEnded,
    Concluded,
}

/// Where one moment stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum State {
    NotDue,
    Due,
    Open,
    Answered,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptError {
    NotDue,
    AlreadyOpen,
    AlreadyAnswered,
}

/// The seven moments in doc 10's order; `states` follows it.
pub const MOMENTS: [PromptMoment; 7] = [
    PromptMoment::ProjectCreation,
    PromptMoment::FirstMaterial,
    PromptMoment::FirstFitToSpace,
    PromptMoment::DesignChangeAfterMaterial,
    PromptMoment::FirstPhysicalTestFit,
    PromptMoment::SessionEnd,
    PromptMoment::Conclusion,
];

/// One project's prompt moments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prompts {
    states: [State; 7],
    material_seen: bool,
}

impl Prompts {
    /// Create a new project with all moments NotDue and no material seen.
    pub fn new() -> Prompts {
        Prompts {
            states: [State::NotDue; 7],
            material_seen: false,
        }
    }

    /// Restore a saved state.
    pub fn restore(states: [State; 7], material_seen: bool) -> Prompts {
        Prompts {
            states,
            material_seen,
        }
    }

    /// Return a copy of all states in MOMENTS order.
    pub fn states(&self) -> [State; 7] {
        self.states
    }

    /// Return whether material has been chosen.
    pub fn material_seen(&self) -> bool {
        self.material_seen
    }

    /// Return the state of a specific moment.
    pub fn state(&self, moment: PromptMoment) -> State {
        self.states[self.idx(moment)]
    }

    /// Observe an event, possibly making a moment Due.
    ///
    /// Returns the moment that became Due, or None if nothing changed.
    pub fn observe(&mut self, event: Event) -> Option<PromptMoment> {
        match event {
            Event::ProjectCreated => {
                let i = self.idx(PromptMoment::ProjectCreation);
                if self.states[i] == State::NotDue {
                    self.states[i] = State::Due;
                    Some(PromptMoment::ProjectCreation)
                } else {
                    None
                }
            }
            Event::MaterialChosen => {
                // material_seen always becomes true
                self.material_seen = true;
                let i = self.idx(PromptMoment::FirstMaterial);
                if self.states[i] == State::NotDue {
                    self.states[i] = State::Due;
                    Some(PromptMoment::FirstMaterial)
                } else {
                    None
                }
            }
            Event::FitToSpace => {
                let i = self.idx(PromptMoment::FirstFitToSpace);
                if self.states[i] == State::NotDue {
                    self.states[i] = State::Due;
                    Some(PromptMoment::FirstFitToSpace)
                } else {
                    None
                }
            }
            Event::DesignChanged => {
                if self.material_seen {
                    let i = self.idx(PromptMoment::DesignChangeAfterMaterial);
                    if self.states[i] == State::NotDue {
                        self.states[i] = State::Due;
                        return Some(PromptMoment::DesignChangeAfterMaterial);
                    }
                }
                None
            }
            Event::TestFit => {
                let i = self.idx(PromptMoment::FirstPhysicalTestFit);
                if self.states[i] == State::NotDue {
                    self.states[i] = State::Due;
                    Some(PromptMoment::FirstPhysicalTestFit)
                } else {
                    None
                }
            }
            Event::SessionEnded => {
                let i = self.idx(PromptMoment::SessionEnd);
                if self.states[i] != State::Due {
                    self.states[i] = State::Due;
                    Some(PromptMoment::SessionEnd)
                } else {
                    None
                }
            }
            Event::Concluded => {
                let i = self.idx(PromptMoment::Conclusion);
                if self.states[i] == State::NotDue {
                    self.states[i] = State::Due;
                    Some(PromptMoment::Conclusion)
                } else {
                    None
                }
            }
        }
    }

    /// Skip a due prompt, turning it into an open page.
    pub fn skip(&mut self, moment: PromptMoment) -> Result<(), PromptError> {
        let i = self.idx(moment);
        match self.states[i] {
            State::Due => {
                self.states[i] = State::Open;
                Ok(())
            }
            State::Open => Err(PromptError::AlreadyOpen),
            State::NotDue => Err(PromptError::NotDue),
            State::Answered => Err(PromptError::AlreadyAnswered),
        }
    }

    /// Answer a prompt (due or open), marking it Answered.
    pub fn answer(&mut self, moment: PromptMoment) -> Result<(), PromptError> {
        let i = self.idx(moment);
        match self.states[i] {
            State::Due | State::Open => {
                self.states[i] = State::Answered;
                Ok(())
            }
            State::NotDue => Err(PromptError::NotDue),
            State::Answered => Err(PromptError::AlreadyAnswered),
        }
    }

    /// List all moments currently Due, in MOMENTS order.
    pub fn due(&self) -> Vec<PromptMoment> {
        MOMENTS
            .iter()
            .enumerate()
            .filter_map(|(i, &m)| {
                if self.states[i] == State::Due {
                    Some(m)
                } else {
                    None
                }
            })
            .collect()
    }

    /// List all moments currently Open (open pages), in MOMENTS order.
    pub fn open_pages(&self) -> Vec<PromptMoment> {
        MOMENTS
            .iter()
            .enumerate()
            .filter_map(|(i, &m)| {
                if self.states[i] == State::Open {
                    Some(m)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Helper: map a PromptMoment to its index in the arrays.
    fn idx(&self, moment: PromptMoment) -> usize {
        match moment {
            PromptMoment::ProjectCreation => 0,
            PromptMoment::FirstMaterial => 1,
            PromptMoment::FirstFitToSpace => 2,
            PromptMoment::DesignChangeAfterMaterial => 3,
            PromptMoment::FirstPhysicalTestFit => 4,
            PromptMoment::SessionEnd => 5,
            PromptMoment::Conclusion => 6,
        }
    }
}

impl Default for Prompts {
    fn default() -> Prompts {
        Prompts::new()
    }
}
