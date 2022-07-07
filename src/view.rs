use crate::input::{QuestionMode, QuizMode};
use derive_getters::Getters;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum ViewStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

pub trait View {
    fn title(&self) -> Option<String>;
    fn description(&self) -> Option<String>;
    fn mode(&self) -> QuizMode;
    fn status(&self) -> ViewStatus;
    fn sections(&self) -> Vec<SectionView>;
    fn questions(&self) -> Vec<QuestionView>;
    fn section(&self, section_id: usize) -> Option<SectionView>;
    fn question(&self, question_id: usize) -> Option<QuestionView>;
}

#[derive(Serialize, Deserialize, Debug, Getters)]
#[serde(rename_all = "camelCase")]
pub struct AnswerView {
    id: usize,
    content: String,
    selected: bool,
}

#[derive(Serialize, Deserialize, Debug, Getters)]
#[serde(rename_all = "camelCase")]
pub struct QuestionView {
    id: usize,
    status: ViewStatus,
    title: Option<String>,
    content: String,
    mode: QuestionMode,
    min_entries: Option<usize>,
    max_entries: Option<usize>,
    answers: Vec<AnswerView>,
}

#[derive(Serialize, Deserialize, Debug, Getters)]
#[serde(rename_all = "camelCase")]
pub struct SectionView {
    id: usize,
    status: ViewStatus,
    title: Option<String>,
    description: Option<String>,
    questions: Vec<QuestionView>,
}
