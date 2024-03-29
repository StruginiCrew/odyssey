use crate::input::{QuestionStatusInput, QuizMode};
use crate::store::{CompiledEntryMatch, QuestionStore, QuizStore, SectionStore};
use derive_getters::Getters;
use std::collections::{HashMap, HashSet};

type StateResult<T> = Result<T, StateError>;

#[derive(Debug)]
pub struct StateError {
    error: StateErrorEnum,
}

#[derive(Debug)]
enum StateErrorEnum {
    SectionNotFound {
        section_id: usize,
    },
    QuestionNotFound {
        question_id: usize,
    },
    AnswerNotFound {
        question_id: usize,
        answer_id: usize,
    },
    QuestionNotAvailable {
        question_id: usize,
    },
    QuestionHasNoSelectableAnswers {
        question_id: usize,
    },
    QuestionCanNotBeUpdated {
        question_id: usize,
    },
    AnswerSelectionMismatch {
        question_id: usize,
        answer_ids: Vec<usize>,
    },
    QuizFinished,
}

#[derive(Debug, PartialEq)]
pub enum AnswerStateStatus {
    Answered,
    AnsweredCorrectly(usize),
    AnsweredWrongly,
}

#[derive(Debug, Getters)]
pub struct AnswerState {
    id: Option<usize>,
    content: String,
    status: AnswerStateStatus,
}

impl AnswerState {
    fn new_selection(question_store: &QuestionStore, answer_id: usize) -> StateResult<Self> {
        let answer = question_store.answers().get(&answer_id).ok_or(StateError {
            error: StateErrorEnum::AnswerNotFound {
                question_id: question_store.id().clone(),
                answer_id,
            },
        })?;

        let status = match question_store.correct_entry_match() {
            Some(entry_match) => match entry_match {
                CompiledEntryMatch::Id { id: match_ids } => {
                    match match_ids.iter().position(|id| id == &answer_id) {
                        Some(index) => AnswerStateStatus::AnsweredCorrectly(index),
                        None => AnswerStateStatus::AnsweredWrongly,
                    }
                }
                CompiledEntryMatch::Content {
                    content: match_contents,
                } => {
                    match match_contents
                        .iter()
                        .position(|match_content| match_content.is_match(answer.content()))
                    {
                        Some(index) => AnswerStateStatus::AnsweredCorrectly(index),
                        None => AnswerStateStatus::AnsweredWrongly,
                    }
                }
            },
            None => AnswerStateStatus::Answered,
        };

        Ok(Self {
            id: Some(answer_id),
            content: answer.content().clone(),
            status,
        })
    }

    fn new_input(question_store: &QuestionStore, content: String) -> StateResult<Self> {
        let status = match question_store.correct_entry_match() {
            Some(entry_match) => match entry_match {
                CompiledEntryMatch::Id { id: _ } => AnswerStateStatus::Answered,
                CompiledEntryMatch::Content {
                    content: match_contents,
                } => {
                    match match_contents
                        .iter()
                        .position(|match_content| match_content.is_match(&content))
                    {
                        Some(index) => AnswerStateStatus::AnsweredCorrectly(index),
                        None => AnswerStateStatus::AnsweredWrongly,
                    }
                }
            },
            None => AnswerStateStatus::Answered,
        };

        Ok(Self {
            id: None,
            content,
            status,
        })
    }
}

#[derive(Debug, PartialEq)]
pub enum QuestionStateStatus {
    InProgress,
    Answered,
    AnsweredCorrectly,
    AnsweredWrongly,
}

impl From<&QuestionStatusInput> for QuestionStateStatus {
    fn from(item: &QuestionStatusInput) -> Self {
        match item {
            QuestionStatusInput::InProgress => Self::InProgress,
            QuestionStatusInput::Answered => Self::Answered,
            QuestionStatusInput::AnsweredCorrectly => Self::AnsweredCorrectly,
            QuestionStatusInput::AnsweredWrongly => Self::AnsweredWrongly,
        }
    }
}

#[derive(Debug, Getters)]
pub struct QuestionState {
    answer_states: Vec<AnswerState>,
    status: QuestionStateStatus,
}

impl QuestionState {
    fn new_with_selections(
        question_store: &QuestionStore,
        answer_ids: Vec<usize>,
    ) -> StateResult<Self> {
        let answer_ids: Vec<usize> = match question_store.max_entries() {
            Some(max_entries) => answer_ids.into_iter().take(*max_entries).collect(),
            None => answer_ids,
        };

        let answers = answer_ids
            .iter()
            .map(|id| AnswerState::new_selection(question_store, *id))
            .collect::<StateResult<Vec<AnswerState>>>()?;
        Ok(Self::new_with_answers(question_store, answers))
    }

    fn new_with_inputs(question_store: &QuestionStore, inputs: Vec<String>) -> StateResult<Self> {
        let inputs: Vec<String> = match question_store.max_entries() {
            Some(max_entries) => inputs.into_iter().take(*max_entries).collect(),
            None => inputs,
        };

        let answers = inputs
            .into_iter()
            .map(|input| AnswerState::new_input(question_store, input))
            .collect::<StateResult<Vec<AnswerState>>>()?;
        Ok(Self::new_with_answers(question_store, answers))
    }

    fn new_with_answers(question_store: &QuestionStore, answer_states: Vec<AnswerState>) -> Self {
        if answer_states.len() < question_store.min_entries().unwrap_or(0) {
            return Self {
                answer_states,
                status: QuestionStateStatus::InProgress,
            };
        }

        let min_correct_entries = question_store.min_correct_entries().unwrap_or(0);
        let max_wrong_entries = question_store
            .max_wrong_entries()
            .unwrap_or(answer_states.len());

        let mut neutral_count = 0;
        let mut wrong_count = 0;

        let mut used_answer_matches: HashSet<usize> = HashSet::new();

        for answer_state in &answer_states {
            match answer_state.status {
                AnswerStateStatus::Answered => neutral_count += 1,
                AnswerStateStatus::AnsweredCorrectly(index) => {
                    used_answer_matches.insert(index);
                }
                AnswerStateStatus::AnsweredWrongly => wrong_count += 1,
            }
        }

        let correct_count = used_answer_matches.len();

        let status = if correct_count > 0 {
            if correct_count >= min_correct_entries && wrong_count <= max_wrong_entries {
                QuestionStateStatus::AnsweredCorrectly
            } else {
                QuestionStateStatus::AnsweredWrongly
            }
        } else if neutral_count > 0 {
            QuestionStateStatus::Answered
        } else {
            QuestionStateStatus::InProgress
        };

        Self {
            answer_states,
            status,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum QuizStateStatus {
    InProgress,
    Completed,
    Failed,
}

#[derive(Debug, Getters)]
pub struct QuizState {
    store: QuizStore,
    question_state: HashMap<usize, QuestionState>,
}

impl QuizState {
    pub fn new(store: QuizStore) -> Self {
        Self {
            store,
            question_state: HashMap::new(),
        }
    }

    pub fn find_section(&self, section_id: usize) -> StateResult<&SectionStore> {
        self.store.sections().get(&section_id).ok_or(StateError {
            error: StateErrorEnum::SectionNotFound { section_id },
        })
    }

    fn find_question_for_update(&self, question_id: usize) -> StateResult<&QuestionStore> {
        if self.quiz_status() != QuizStateStatus::InProgress {
            return Err(StateError {
                error: StateErrorEnum::QuizFinished,
            });
        }

        let question = self.find_question(question_id)?;

        match self.question_state.get(&question_id) {
            Some(question_state) => match self.store.block_answer_updates_for() {
                Some(blocked_statuses) => {
                    if blocked_statuses.iter().any(|blocked_status| {
                        question_state.status() == &QuestionStateStatus::from(blocked_status)
                    }) {
                        Err(StateError {
                            error: StateErrorEnum::QuestionCanNotBeUpdated { question_id },
                        })
                    } else {
                        Ok(question)
                    }
                }
                None => Ok(question),
            },
            None => Ok(question),
        }
    }

    pub fn find_question(&self, question_id: usize) -> StateResult<&QuestionStore> {
        let question = self.store.questions().get(&question_id).ok_or(StateError {
            error: StateErrorEnum::QuestionNotFound { question_id },
        })?;

        match self.store.quiz_mode() {
            QuizMode::Open => Ok(question),
            QuizMode::Linear => {
                let previous_question_id = match self
                    .store
                    .question_ids()
                    .iter()
                    .position(|&id| id == question_id)
                {
                    Some(index) if index > 0 => Some(self.store.question_ids()[index - 1]),
                    Some(_) => None,
                    None => None,
                };

                match previous_question_id {
                    Some(previous_question_id) => {
                        match self.question_state.get(&previous_question_id) {
                            Some(QuestionState {
                                status:
                                    QuestionStateStatus::InProgress
                                    | QuestionStateStatus::Answered
                                    | QuestionStateStatus::AnsweredWrongly,
                                answer_states: _,
                            })
                            | None => Err(StateError {
                                error: StateErrorEnum::QuestionNotAvailable { question_id },
                            }),
                            Some(QuestionState {
                                status: QuestionStateStatus::AnsweredCorrectly,
                                answer_states: _,
                            }) => Ok(question),
                        }
                    }
                    None => Ok(question),
                }
            }
        }
    }

    pub fn answered_questions_count(&self) -> usize {
        self.question_state
            .values()
            .filter(|q| match q.status() {
                QuestionStateStatus::InProgress => false,
                _ => true,
            })
            .count()
    }

    pub fn correct_questions_count(&self) -> usize {
        self.question_state
            .values()
            .filter(|q| match q.status() {
                QuestionStateStatus::AnsweredCorrectly => true,
                _ => false,
            })
            .count()
    }

    pub fn wrong_questions_count(&self) -> usize {
        self.question_state
            .values()
            .filter(|q| match q.status() {
                QuestionStateStatus::AnsweredWrongly => true,
                _ => false,
            })
            .count()
    }

    pub fn quiz_status(&self) -> QuizStateStatus {
        if self
            .store
            .questions()
            .values()
            .filter_map(|question_store| {
                if *question_store.optional() {
                    None
                } else {
                    Some(question_store.id().clone())
                }
            })
            .any(|question_id| match self.question_state.get(&question_id) {
                Some(question_state)
                    if question_state.status() == &QuestionStateStatus::InProgress
                        || question_state.status() == &QuestionStateStatus::AnsweredWrongly =>
                {
                    true
                }
                _ => false,
            })
        {
            return QuizStateStatus::InProgress;
        }

        match (
            self.store.min_answered_questions(),
            self.store.max_answered_questions(),
            self.store.min_correct_questions(),
            self.store.max_wrong_questions(),
        ) {
            (Some(min_answered_questions), _, _, _)
                if self.answered_questions_count() < *min_answered_questions =>
            {
                QuizStateStatus::InProgress
            }
            (_, _, _, Some(max_wrong_questions))
                if self.wrong_questions_count() >= *max_wrong_questions =>
            {
                QuizStateStatus::Failed
            }
            (_, _, Some(min_correct_questions), _)
                if self.correct_questions_count() >= *min_correct_questions =>
            {
                QuizStateStatus::Completed
            }
            (_, Some(max_answered_questions), _, _)
                if self.answered_questions_count() >= *max_answered_questions =>
            {
                QuizStateStatus::Completed
            }
            _ => QuizStateStatus::Completed,
        }
    }

    pub fn select_answers(
        &mut self,
        question_id: usize,
        answer_ids: Vec<usize>,
    ) -> StateResult<()> {
        let question = self.find_question_for_update(question_id)?;
        let question_state = QuestionState::new_with_selections(&question, answer_ids)?;
        self.question_state.insert(question_id, question_state);
        Ok(())
    }

    pub fn input_answers(&mut self, question_id: usize, inputs: Vec<String>) -> StateResult<()> {
        let question = self.find_question_for_update(question_id)?;
        let question_state = QuestionState::new_with_inputs(&question, inputs)?;
        self.question_state.insert(question_id, question_state);
        Ok(())
    }

    pub fn clear_answers(&mut self, question_id: usize) -> StateResult<()> {
        let question = self.find_question_for_update(question_id)?;
        self.question_state.remove(&question_id);
        Ok(())
    }
}
