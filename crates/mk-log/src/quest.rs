//! The Quest: the new-project questions as data, and the rules that walk them.
//!
//! Written by Kimi (the Crew, campaign mk-core-10, 2026-11-21; the task relayed from Nemotron) from
//! the contract crew/contracts/mk_quest.md (ATLAS-api-harness), which Claude finished from Gordon's brief.

/// A step of the Quest.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StepId {
    Name,
    Goal,
    Fit,
    Tools,
    Materials,
    Deadline,
    Budget,
    Inspiration,
    BoxContents,
    HowMany,
    Review,
}

/// A project template (doc 05 section 5.2: templates can add steps).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Template {
    Default,
    Packaging,
    SoftGoods,
    Room,
}

/// The five answers to "Where does it need to fit? How big is it?".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FitPick {
    TypeSizes,
    HaveScan,
    PhotographWithReference,
    SmallNoFixedSpace,
    DecideLater,
}

/// The scale classes of doc 05 section 5.5.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScaleClass {
    Room,
    Furniture,
    Body,
    Bench,
    Macro,
}

/// What kind of answer a step takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnswerKind {
    Text,
    Fit,
    Items,
    Deadline,
    Budget,
    Count,
    Review,
}

/// One step: its id, its question (the doc's words), and the kind of answer it takes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    pub id: StepId,
    pub question: &'static str,
    pub kind: AnswerKind,
}

/// An answer. `Skipped` is an answer too: the step is passed and stays outstanding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Answer {
    Skipped,
    Text(String),
    Fit(FitPick),
    /// Tools, materials or inspiration: how many items were given (0 is allowed).
    Items(u32),
    /// The due date as (year, month, day), or None for "no deadline".
    Deadline(Option<(u16, u8, u8)>),
    /// A cap in minor units with its currency, "use only what I have", or "not sure".
    BudgetCap(u64, String),
    BudgetOnlyWhatIHave,
    BudgetNotSure,
    Count(u32),
    Confirmed,
}

/// Why an answer was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuestError {
    NotInThisQuest(StepId),
    WrongKind(StepId),
    AlreadyConfirmed,
}

/// The steps of a template, in order (rules T1-T2).
pub fn steps(template: Template) -> Vec<Step> {
    let mut steps = vec![
        Step {
            id: StepId::Name,
            question: "What shall we call it?",
            kind: AnswerKind::Text,
        },
        Step {
            id: StepId::Goal,
            question: "What would you like to make?",
            kind: AnswerKind::Text,
        },
        Step {
            id: StepId::Fit,
            question: "Where does it need to fit? How big is it?",
            kind: AnswerKind::Fit,
        },
        Step {
            id: StepId::Tools,
            question: "What tools do you have?",
            kind: AnswerKind::Items,
        },
        Step {
            id: StepId::Materials,
            question: "What materials do you have?",
            kind: AnswerKind::Items,
        },
        Step {
            id: StepId::Deadline,
            question: "When do you need it?",
            kind: AnswerKind::Deadline,
        },
        Step {
            id: StepId::Budget,
            question: "What's your budget?",
            kind: AnswerKind::Budget,
        },
        Step {
            id: StepId::Inspiration,
            question: "What inspires you?",
            kind: AnswerKind::Items,
        },
        Step {
            id: StepId::Review,
            question: "Here's your workspace",
            kind: AnswerKind::Review,
        },
    ];

    if template == Template::Packaging {
        steps.insert(
            8,
            Step {
                id: StepId::BoxContents,
                question: "What goes in the box?",
                kind: AnswerKind::Text,
            },
        );
        steps.insert(
            9,
            Step {
                id: StepId::HowMany,
                question: "How many will you make?",
                kind: AnswerKind::Count,
            },
        );
    }

    steps
}

/// The scale class a longest size suggests (rule S1).
pub fn class_for_size(longest_mm: f64) -> Option<ScaleClass> {
    if !longest_mm.is_finite() || longest_mm <= 0.0 {
        None
    } else if longest_mm < 10.0 {
        Some(ScaleClass::Macro)
    } else if longest_mm <= 500.0 {
        Some(ScaleClass::Bench)
    } else if longest_mm <= 2500.0 {
        Some(ScaleClass::Furniture)
    } else {
        Some(ScaleClass::Room)
    }
}

/// A Quest in progress: its steps and the answers so far.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Quest {
    steps: Vec<Step>,
    answers: Vec<(StepId, Answer)>,
}

impl Quest {
    /// A new Quest for a template, with no answers.
    pub fn new(template: Template) -> Quest {
        Quest {
            steps: steps(template),
            answers: Vec::new(),
        }
    }

    /// The steps, in order.
    pub fn steps(&self) -> &[Step] {
        &self.steps
    }

    /// Records an answer (rules A1-A5).
    pub fn answer(&mut self, id: StepId, answer: Answer) -> Result<(), QuestError> {
        let Some(step_index) = self.steps.iter().position(|step| step.id == id) else {
            return Err(QuestError::NotInThisQuest(id));
        };

        if id == StepId::Review && matches!(self.answer_for(&id), Some(Answer::Confirmed)) {
            return Err(QuestError::AlreadyConfirmed);
        }

        if !answer_fits(self.steps[step_index].kind, &answer) {
            return Err(QuestError::WrongKind(id));
        }

        if let Some(answer_index) = self
            .answers
            .iter()
            .position(|(answer_id, _)| answer_id == &id)
        {
            self.answers[answer_index].1 = answer;
        } else {
            self.answers.push((id, answer));
        }

        Ok(())
    }

    /// The answer recorded for a step, if any.
    pub fn answer_for(&self, id: &StepId) -> Option<&Answer> {
        self.answers
            .iter()
            .find(|(answer_id, _)| answer_id == id)
            .map(|(_, answer)| answer)
    }

    /// The next step to ask (rule N1).
    pub fn next_step(&self) -> Option<&Step> {
        self.steps
            .iter()
            .find(|step| self.answer_for(&step.id).is_none())
    }

    /// Skipped or unanswered steps, in step order, Review left out (rule N2).
    pub fn outstanding(&self) -> Vec<StepId> {
        self.steps
            .iter()
            .filter(|step| {
                step.id != StepId::Review
                    && matches!(self.answer_for(&step.id), None | Some(Answer::Skipped))
            })
            .map(|step| step.id.clone())
            .collect()
    }

    /// Whether the maker confirmed the review (rule N3).
    pub fn is_done(&self) -> bool {
        matches!(self.answer_for(&StepId::Review), Some(Answer::Confirmed))
    }

    /// "Skip to designing" (rule N4).
    pub fn skip_to_designing(&mut self) {
        let unanswered: Vec<StepId> = self
            .steps
            .iter()
            .filter(|step| step.id != StepId::Review && self.answer_for(&step.id).is_none())
            .map(|step| step.id.clone())
            .collect();

        for id in unanswered {
            self.answers.push((id, Answer::Skipped));
        }

        if self.answer_for(&StepId::Review).is_none() {
            self.answers.push((StepId::Review, Answer::Confirmed));
        }
    }

    /// Whether the project still needs its goal (rule F1).
    pub fn intent_missing(&self) -> bool {
        !matches!(
            self.answer_for(&StepId::Goal),
            Some(Answer::Text(text)) if !text.trim().is_empty()
        )
    }

    /// Whether any materials were given (rule F2).
    pub fn materials_given(&self) -> bool {
        matches!(
            self.answer_for(&StepId::Materials),
            Some(Answer::Items(n)) if *n >= 1
        )
    }

    /// The scale class the Fit answer suggests (rule F3).
    pub fn suggested_class(&self) -> Option<ScaleClass> {
        if matches!(
            self.answer_for(&StepId::Fit),
            Some(Answer::Fit(FitPick::SmallNoFixedSpace))
        ) {
            Some(ScaleClass::Bench)
        } else {
            None
        }
    }
}

fn answer_fits(kind: AnswerKind, answer: &Answer) -> bool {
    match answer {
        Answer::Skipped => kind != AnswerKind::Review,
        Answer::Text(_) => kind == AnswerKind::Text,
        Answer::Fit(_) => kind == AnswerKind::Fit,
        Answer::Items(_) => kind == AnswerKind::Items,
        Answer::Deadline(_) => kind == AnswerKind::Deadline,
        Answer::BudgetCap(_, _) | Answer::BudgetOnlyWhatIHave | Answer::BudgetNotSure => {
            kind == AnswerKind::Budget
        }
        Answer::Count(_) => kind == AnswerKind::Count,
        Answer::Confirmed => kind == AnswerKind::Review,
    }
}
