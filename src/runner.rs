use crate::event_log::{Event, EventLog};
use crate::input::QuizInput;
use crate::state::{QuizState, StateError};
use crate::store::{QuizStore, StoreError};
use crate::view::{QuestionView, QuizView, SectionView};
use serde_json::Error as JsonError;
use std::collections::HashMap;

type RunnerResult<T> = Result<T, RunnerError>;

pub struct RunnerError {
    error: RunnerErrorEnum,
}

enum RunnerErrorEnum {
    InputError { source: JsonError },
    StoreError { source: StoreError },
    StateError { source: StateError },
}

impl From<JsonError> for RunnerError {
    fn from(item: JsonError) -> Self {
        RunnerError {
            error: RunnerErrorEnum::InputError { source: item },
        }
    }
}

impl From<StoreError> for RunnerError {
    fn from(item: StoreError) -> Self {
        RunnerError {
            error: RunnerErrorEnum::StoreError { source: item },
        }
    }
}

impl From<StateError> for RunnerError {
    fn from(item: StateError) -> Self {
        RunnerError {
            error: RunnerErrorEnum::StateError { source: item },
        }
    }
}

pub struct Runner {
    state: QuizState,
    event_log: EventLog,
}

impl Runner {
    pub fn new(input: &str) -> RunnerResult<Self> {
        let input: QuizInput = serde_json::from_str(&input)?;
        let store = QuizStore::try_from(&input)?;
        let event_log = EventLog::new(store.uid().clone(), store.version().clone(), Vec::new());
        let state = QuizState::new(store);

        Ok(Self { state, event_log })
    }

    pub fn new_with_events(input: &str, event_log_input: &str) -> RunnerResult<Self> {
        let input: QuizInput = serde_json::from_str(&input)?;
        let store = QuizStore::try_from(&input)?;
        let event_log = serde_json::from_str::<EventLog>(&event_log_input)?;
        let state = QuizState::new(store);

        Ok(Self { state, event_log })
    }

    pub fn select_answers(
        &mut self,
        question_id: usize,
        answer_ids: Vec<usize>,
    ) -> RunnerResult<QuestionView> {
        self.event(Event::SelectAnswers {
            question_id,
            answer_ids,
        })?;

        self.question_view(question_id)
    }

    pub fn input_answers(
        &mut self,
        question_id: usize,
        inputs: Vec<String>,
    ) -> RunnerResult<QuestionView> {
        self.event(Event::InputAnswers {
            question_id,
            inputs,
        })?;

        self.question_view(question_id)
    }

    pub fn clear_answers(&mut self, question_id: usize) -> RunnerResult<QuestionView> {
        self.event(Event::ClearAnswers { question_id })?;

        self.question_view(question_id)
    }

    pub fn question_view(&mut self, question_id: usize) -> RunnerResult<QuestionView> {
        let question_store = self.state.find_question(question_id)?;

        let view = QuestionView::new(
            &question_store,
            self.state.question_state().get(&question_id),
        );

        Ok(view)
    }

    pub fn section_view(&mut self, section_id: usize) -> RunnerResult<SectionView> {
        let section_store = self.state.find_section(section_id)?;

        let view = SectionView::new(&section_store, &self.state);

        Ok(view)
    }

    pub fn quiz_view(&mut self) -> RunnerResult<QuizView> {
        let view = QuizView::new(&self.state);

        Ok(view)
    }

    pub fn event_log(&self) -> &EventLog {
        &self.event_log
    }

    fn event(&mut self, event: Event) -> RunnerResult<()> {
        match &event {
            Event::SelectAnswers {
                question_id,
                answer_ids,
            } => self
                .state
                .select_answers(*question_id, answer_ids.clone())?,
            Event::InputAnswers {
                question_id,
                inputs,
            } => self.state.input_answers(*question_id, inputs.clone())?,
            Event::ClearAnswers { question_id } => self.state.clear_answers(*question_id)?,
        }

        self.event_log.push(event);

        Ok(())
    }
}
