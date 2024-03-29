use crate::input::{QuestionMode, QuizMode};
use crate::state::{
    AnswerStateStatus, QuestionState, QuestionStateStatus, QuizState, QuizStateStatus,
};
use crate::store::{QuestionStore, SectionStore};
use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum AnswerViewStatus {
    Pending,
    Answered,
    AnsweredCorrectly(usize),
    AnsweredWrongly,
}

impl From<&AnswerStateStatus> for AnswerViewStatus {
    fn from(item: &AnswerStateStatus) -> Self {
        match item {
            AnswerStateStatus::Answered => AnswerViewStatus::Answered,
            AnswerStateStatus::AnsweredCorrectly(index) => {
                AnswerViewStatus::AnsweredCorrectly(*index)
            }
            AnswerStateStatus::AnsweredWrongly => AnswerViewStatus::AnsweredWrongly,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum QuestionViewStatus {
    Pending,
    InProgress,
    Answered,
    AnsweredCorrectly,
    AnsweredWrongly,
}

impl From<&QuestionStateStatus> for QuestionViewStatus {
    fn from(item: &QuestionStateStatus) -> Self {
        match item {
            QuestionStateStatus::InProgress => QuestionViewStatus::InProgress,
            QuestionStateStatus::Answered => QuestionViewStatus::Answered,
            QuestionStateStatus::AnsweredCorrectly => QuestionViewStatus::AnsweredCorrectly,
            QuestionStateStatus::AnsweredWrongly => QuestionViewStatus::AnsweredWrongly,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Getters, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AnswerView {
    id: Option<usize>,
    content: String,
    status: AnswerViewStatus,
}

#[derive(Serialize, Deserialize, Debug, Getters, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QuestionView {
    id: usize,
    status: QuestionViewStatus,
    title: Option<String>,
    content: String,
    mode: QuestionMode,
    min_entries: Option<usize>,
    max_entries: Option<usize>,
    answers: Vec<AnswerView>,
}

impl QuestionView {
    pub fn new(question_store: &QuestionStore, question_state: Option<&QuestionState>) -> Self {
        let status = match question_state {
            Some(question_state) => question_state.status().into(),
            None => QuestionViewStatus::Pending,
        };

        Self {
            id: question_store.id().clone(),
            status,
            title: question_store.title().clone(),
            content: question_store.content().clone(),
            mode: question_store.mode().clone(),
            min_entries: question_store.min_entries().clone(),
            max_entries: question_store.max_entries().clone(),
            answers: match question_store.mode() {
                QuestionMode::Select => {
                    let mut views: HashMap<usize, AnswerView> = question_store
                        .answer_ids()
                        .iter()
                        .filter_map(|answer_id| match question_store.answers().get(answer_id) {
                            Some(answer_store) => Some((
                                *answer_id,
                                AnswerView {
                                    id: Some(answer_id.clone()),
                                    content: answer_store.content().clone(),
                                    status: AnswerViewStatus::Pending,
                                },
                            )),
                            None => None,
                        })
                        .collect();

                    if let Some(question_state) = question_state {
                        for answer_state in question_state.answer_states() {
                            if let Some(answer_id) = answer_state.id() {
                                if let Some(view) = views.get_mut(answer_id) {
                                    view.status = answer_state.status().into();
                                }
                            }
                        }
                    }

                    question_store
                        .answer_ids()
                        .iter()
                        .filter_map(|answer_id| views.remove(answer_id))
                        .collect()
                }
                QuestionMode::Input => match question_state {
                    Some(question_state) => question_state
                        .answer_states()
                        .iter()
                        .map(|answer_state| AnswerView {
                            id: None,
                            content: answer_state.content().clone(),
                            status: answer_state.status().into(),
                        })
                        .collect(),
                    None => Vec::new(),
                },
            },
        }
    }

    pub fn correct_answer_match_indexes(&self) -> Vec<usize> {
        self.answers
            .iter()
            .filter_map(|answer| match answer.status {
                AnswerViewStatus::AnsweredCorrectly(index) => Some(index),
                _ => None,
            })
            .collect()
    }
}

#[derive(Serialize, Deserialize, Debug, Getters, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SectionView {
    id: usize,
    title: Option<String>,
    description: Option<String>,
    questions: Vec<QuestionView>,
}

impl SectionView {
    pub fn new(section_store: &SectionStore, quiz_state: &QuizState) -> Self {
        SectionView {
            id: section_store.id().clone(),
            title: section_store.title().clone(),
            description: section_store.description().clone(),
            questions: section_store
                .question_ids()
                .iter()
                .filter_map(|id| match quiz_state.store().questions().get(id) {
                    Some(question_store) => Some(QuestionView::new(
                        question_store,
                        quiz_state.question_state().get(id),
                    )),
                    None => None,
                })
                .collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum QuizViewStatus {
    InProgress,
    Completed,
    Failed,
}

impl From<QuizStateStatus> for QuizViewStatus {
    fn from(item: QuizStateStatus) -> Self {
        match item {
            QuizStateStatus::InProgress => Self::InProgress,
            QuizStateStatus::Completed => Self::Completed,
            QuizStateStatus::Failed => Self::Failed,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Getters, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QuizView {
    uid: String,
    version: usize,
    title: Option<String>,
    description: Option<String>,
    quiz_mode: QuizMode,
    status: QuizViewStatus,
    answered_questions_count: usize,
    correct_questions_count: usize,
    wrong_questions_count: usize,
    sections: Vec<SectionView>,
}

impl QuizView {
    pub fn new(quiz_state: &QuizState) -> Self {
        let quiz_store = quiz_state.store();

        QuizView {
            uid: quiz_store.uid().clone(),
            version: quiz_store.version().clone(),
            title: quiz_store.title().clone(),
            description: quiz_store.description().clone(),
            quiz_mode: quiz_store.quiz_mode().clone(),
            status: quiz_state.quiz_status().into(),
            answered_questions_count: quiz_state.answered_questions_count(),
            correct_questions_count: quiz_state.correct_questions_count(),
            wrong_questions_count: quiz_state.wrong_questions_count(),
            sections: quiz_store
                .section_ids()
                .iter()
                .filter_map(|id| match quiz_state.store().sections().get(id) {
                    Some(section_store) => Some(SectionView::new(section_store, quiz_state)),
                    None => None,
                })
                .collect(),
        }
    }
}
