use derive_getters::Getters;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum QuizMode {
    Open,
    Linear,
}

#[derive(Serialize, Deserialize, Debug, Getters)]
#[serde(rename_all = "camelCase")]
pub struct AnswerInput {
    id: usize,
    content: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum QuestionMode {
    Select,
    Input,
    Mixed,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
#[serde(rename_all = "camelCase")]
pub enum EntryMatch {
    #[serde(rename_all = "camelCase")]
    Id { id: Vec<usize> },
    #[serde(rename_all = "camelCase")]
    Content { content: Vec<String> },
    #[serde(rename_all = "camelCase")]
    IdOrContent {
        id: Vec<usize>,
        content: Vec<String>,
    },
}

#[derive(Serialize, Deserialize, Debug, Getters)]
#[serde(rename_all = "camelCase")]
pub struct QuestionInput {
    id: usize,
    title: Option<String>,
    content: String,
    mode: QuestionMode,
    min_entries: Option<usize>,
    max_entries: Option<usize>,
    min_correct_entries: Option<usize>,
    max_wrong_entries: Option<usize>,
    correct_entry_match: Option<EntryMatch>,
    answers: Option<Vec<AnswerInput>>,
}

#[derive(Serialize, Deserialize, Debug, Getters)]
#[serde(rename_all = "camelCase")]
pub struct SectionInput {
    id: usize,
    title: Option<String>,
    description: Option<String>,
    questions: Vec<QuestionInput>,
}

#[derive(Serialize, Deserialize, Debug, Getters)]
#[serde(rename_all = "camelCase")]
pub struct QuizInput {
    uid: String,
    version: usize,
    title: Option<String>,
    description: Option<String>,
    mode: QuizMode,
    sections: Vec<SectionInput>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_deserializes_quiz_json() {
        let input_json = include_str!("../tests/input/open_exam_quiz.json");
        let quiz_input = serde_json::from_str::<QuizInput>(&input_json);
        assert!(quiz_input.is_ok());
    }
}
