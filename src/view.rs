use crate::input::QuestionMode;
use crate::state::{AnswerState, AnswerStateStatus, QuestionState, QuestionStateStatus, QuizState};
use crate::store::{AnswerStore, QuestionStore, SectionStore};
use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum AnswerViewStatus {
    Pending,
    Answered,
    AnsweredCorrectly,
    AnsweredWrongly,
}

impl From<&AnswerStateStatus> for AnswerViewStatus {
    fn from(item: &AnswerStateStatus) -> Self {
        match item {
            AnswerStateStatus::Answered => AnswerViewStatus::Answered,
            AnswerStateStatus::AnsweredCorrectly => AnswerViewStatus::AnsweredCorrectly,
            AnswerStateStatus::AnsweredWrongly => AnswerViewStatus::AnsweredWrongly,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug, Getters)]
#[serde(rename_all = "camelCase")]
pub struct AnswerView {
    id: Option<usize>,
    content: String,
    status: AnswerViewStatus,
}

#[derive(Serialize, Deserialize, Debug, Getters)]
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
}

#[derive(Serialize, Deserialize, Debug, Getters)]
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
