use crate::store::QuizStore;
use std::collections::HashMap;

struct QuestionState {
    answers: Vec<AnswerVariant>,
}

enum AnswerVariant {
    Selection(usize),
    Input(String),
}

pub struct Quiz {
    store: QuizStore,
    current_question_id: usize,
    question_state: HashMap<usize, QuestionState>,
}

impl Quiz {
    pub fn new(store: QuizStore) -> Self {
        // TODO: Handle with error
        let current_question_id = store.question_ids()[0];

        Self {
            store,
            current_question_id,
            question_state: HashMap::new(),
        }
    }

    pub fn select_answer(&mut self, question_id: usize, answer_ids: Vec<usize>) {}
    pub fn input_answer(&mut self, question_id: usize, inputs: Vec<String>) {}
    pub fn clear_answers(&mut self, question_id: usize) {}
}
