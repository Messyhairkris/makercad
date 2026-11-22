//! The session-end interview (doc 07 section 7.7): offered, drafted, answered, and turned into
//!
//! Written by Kimi (the Crew, campaign mk-core-16, 2026-11-22) from the contract
//! crew/contracts/mk_interview.md (ATLAS-api-harness), which Claude finished from Deepak's brief.
//! journal records.

use crate::journal::{Body, MediaRef, PromptMoment};

/// Why the interview is offered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Trigger {
    Closing,
    LongSession,
    WrapUp,
}

/// The six questions, in the order they are asked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Question {
    Did,
    Worked,
    Didnt,
    Media,
    Next,
    Mood,
}

/// The cause chips of "What didn't, and why?", in the doc's order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Cause {
    MeasurementOff,
    MaterialUnexpected,
    ToolLimitation,
    DesignChanged,
    OutOfTime,
    Other,
}

/// "How are you feeling about it?" (shown as three faces).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mood {
    Worried,
    Fine,
    Excited,
}

/// One thing the session's command log says was done.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Done {
    Added {
        count: u32,
        what: String,
    },
    Removed {
        count: u32,
        what: String,
    },
    Changed {
        what: String,
        from: String,
        to: String,
    },
    Generated {
        what: String,
    },
}

/// The answer to one question.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Answer {
    Unanswered,
    Skipped,
    Text(String),
    Causes(Vec<Cause>, String),
    Media(Vec<MediaRef>),
    Mood(Mood),
}

/// An answer supplied for a question whose kind does not accept it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterviewError {
    WrongKind(Question),
}

/// The questions in asking order.
pub const QUESTIONS: [Question; 6] = [
    Question::Did,
    Question::Worked,
    Question::Didnt,
    Question::Media,
    Question::Next,
    Question::Mood,
];

/// The cause chips in the doc's order.
pub const CAUSES: [Cause; 6] = [
    Cause::MeasurementOff,
    Cause::MaterialUnexpected,
    Cause::ToolLimitation,
    Cause::DesignChanged,
    Cause::OutOfTime,
    Cause::Other,
];

/// Activity that makes a session long enough to offer the interview.
pub const LONG_SESSION_MINUTES: u32 = 120;

/// The most things the draft names before it says "and N more things".
pub const DRAFT_MAX: usize = 6;

/// One interview in progress.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Interview {
    trigger: Trigger,
    draft: Option<String>,
    answers: Vec<Answer>,
}

impl Cause {
    /// The phrase used for this chip in a session body.
    pub fn label(self) -> &'static str {
        match self {
            Cause::MeasurementOff => "measurement was off",
            Cause::MaterialUnexpected => "material behaved unexpectedly",
            Cause::ToolLimitation => "tool limitation",
            Cause::DesignChanged => "design changed",
            Cause::OutOfTime => "ran out of time",
            Cause::Other => "other",
        }
    }
}

impl Mood {
    /// The journal key for this mood.
    pub fn key(self) -> &'static str {
        match self {
            Mood::Worried => "worried",
            Mood::Fine => "fine",
            Mood::Excited => "excited",
        }
    }
}

/// Rule O1: the first matching offer condition wins.
pub fn should_offer(
    closing: bool,
    wrap_up: bool,
    active_minutes: u32,
    already_offered: bool,
) -> Option<Trigger> {
    if wrap_up {
        Some(Trigger::WrapUp)
    } else if already_offered {
        None
    } else if closing {
        Some(Trigger::Closing)
    } else if active_minutes >= LONG_SESSION_MINUTES {
        Some(Trigger::LongSession)
    } else {
        None
    }
}

/// Rule D1: turn the command log into a short statement of what was done.
pub fn draft(done: &[Done]) -> Option<String> {
    let mut phrases: Vec<String> = done.iter().map(done_phrase).collect();

    if phrases.len() > DRAFT_MAX {
        let remaining = phrases.len() - (DRAFT_MAX - 1);
        phrases.truncate(DRAFT_MAX - 1);
        phrases.push(format!("{remaining} more things"));
    }

    match phrases.len() {
        0 => None,
        1 => Some(format!("You {}.", phrases[0])),
        2 => Some(format!("You {} and {}.", phrases[0], phrases[1])),
        _ => {
            let (last, earlier) = phrases.split_last()?;
            Some(format!("You {}, and {}.", earlier.join(", "), last))
        }
    }
}

impl Interview {
    /// Rule I1: start with the supplied trigger and draft, and no answers.
    pub fn new(trigger: Trigger, done: &[Done]) -> Interview {
        Interview {
            trigger,
            draft: draft(done),
            answers: vec![Answer::Unanswered; QUESTIONS.len()],
        }
    }

    /// Why this interview was offered.
    pub fn trigger(&self) -> Trigger {
        self.trigger
    }

    /// The pre-filled statement of what was done, when there was anything to say.
    pub fn draft(&self) -> Option<&str> {
        self.draft.as_deref()
    }

    /// Rule A1: accept only a fitting answer, normalise it, and replace any earlier answer.
    pub fn answer(&mut self, question: Question, answer: Answer) -> Result<(), InterviewError> {
        if !answer_fits(question, &answer) {
            return Err(InterviewError::WrongKind(question));
        }

        self.answers[question_index(question)] = normalise(answer);
        Ok(())
    }

    /// The current stored answer to one question.
    pub fn answer_of(&self, question: Question) -> &Answer {
        &self.answers[question_index(question)]
    }

    /// Rule A2: the first unanswered question, if any.
    pub fn current(&self) -> Option<Question> {
        QUESTIONS
            .into_iter()
            .find(|question| matches!(self.answer_of(*question), Answer::Unanswered))
    }

    /// Rule A3: the trimmed "what's next" text, when one was supplied.
    pub fn keep_going(&self) -> Option<String> {
        match self.answer_of(Question::Next) {
            Answer::Text(text) => Some(text.trim().to_string()),
            _ => None,
        }
    }

    /// Rule F1: build the session body and any open page without changing the interview.
    pub fn finish(&self) -> Vec<Body> {
        let mut bodies = Vec::with_capacity(2);

        if !self.answers.iter().all(is_blank) {
            bodies.push(Body::Session {
                did: answer_lines(self.answer_of(Question::Did)),
                worked: answer_lines(self.answer_of(Question::Worked)),
                didnt: didnt_lines(self.answer_of(Question::Didnt)),
                next: answer_lines(self.answer_of(Question::Next)),
                media: answer_media(self.answer_of(Question::Media)),
                mood: answer_mood(self.answer_of(Question::Mood)),
            });
        }

        let leaves_page_open = [
            Question::Did,
            Question::Worked,
            Question::Didnt,
            Question::Next,
        ]
        .into_iter()
        .any(|question| is_blank(self.answer_of(question)));

        if leaves_page_open {
            bodies.push(Body::OpenPage {
                prompt: PromptMoment::SessionEnd,
            });
        }

        bodies
    }
}

/// Rule D1: the exact phrase for one command-log entry.
fn done_phrase(done: &Done) -> String {
    match done {
        Done::Added { count, what } => format!("added {count} {what}"),
        Done::Removed { count, what } => format!("removed {count} {what}"),
        Done::Changed { what, from, to } => {
            format!("changed {what} from {from} to {to}")
        }
        Done::Generated { what } => format!("generated {what}"),
    }
}

fn question_index(question: Question) -> usize {
    match question {
        Question::Did => 0,
        Question::Worked => 1,
        Question::Didnt => 2,
        Question::Media => 3,
        Question::Next => 4,
        Question::Mood => 5,
    }
}

/// Rule A1: the answer-kind compatibility table.
fn answer_fits(question: Question, answer: &Answer) -> bool {
    match answer {
        Answer::Skipped => true,
        Answer::Text(_) => matches!(question, Question::Did | Question::Worked | Question::Next),
        Answer::Causes(_, _) => question == Question::Didnt,
        Answer::Media(_) => question == Question::Media,
        Answer::Mood(_) => question == Question::Mood,
        Answer::Unanswered => false,
    }
}

/// Rule A1: normalisation happens only after compatibility has been established.
fn normalise(answer: Answer) -> Answer {
    match answer {
        Answer::Text(text) if text.trim().is_empty() => Answer::Skipped,
        Answer::Causes(chips, note) => {
            let mut unique = Vec::with_capacity(chips.len());
            for chip in chips {
                if !unique.contains(&chip) {
                    unique.push(chip);
                }
            }

            if unique.is_empty() && note.trim().is_empty() {
                Answer::Skipped
            } else {
                Answer::Causes(unique, note)
            }
        }
        Answer::Media(refs) if refs.is_empty() => Answer::Skipped,
        other => other,
    }
}

fn is_blank(answer: &Answer) -> bool {
    matches!(answer, Answer::Skipped | Answer::Unanswered)
}

/// Rule F1: CRLF works because trimming each '\n'-separated line removes the '\r'.
fn text_lines(text: &str) -> Vec<String> {
    text.split('\n')
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect()
}

fn answer_lines(answer: &Answer) -> Vec<String> {
    match answer {
        Answer::Text(text) => text_lines(text),
        _ => Vec::new(),
    }
}

fn didnt_lines(answer: &Answer) -> Vec<String> {
    match answer {
        Answer::Causes(chips, note) => {
            let mut lines: Vec<String> =
                chips.iter().map(|chip| chip.label().to_string()).collect();
            lines.extend(text_lines(note));
            lines
        }
        _ => Vec::new(),
    }
}

fn answer_media(answer: &Answer) -> Vec<MediaRef> {
    match answer {
        Answer::Media(refs) => refs.clone(),
        _ => Vec::new(),
    }
}

fn answer_mood(answer: &Answer) -> Option<String> {
    match answer {
        Answer::Mood(mood) => Some(mood.key().to_string()),
        _ => None,
    }
}
