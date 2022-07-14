use crate::input::{
    AnswerInput, EntryMatch, QuestionInput, QuestionMode, QuestionStatusInput, QuizInput, QuizMode,
    SectionInput,
};
use derive_getters::Getters;
use regex::Regex;
use std::collections::HashMap;

type StoreResult<T> = Result<T, StoreError>;

#[derive(Debug)]
pub struct StoreError {
    error: StoreErrorEnum,
}

#[derive(Debug)]
enum StoreErrorEnum {
    DuplicateSectionId {
        section_id: usize,
    },
    DuplicateQuestionId {
        section_id: usize,
        question_id: usize,
    },
    DuplicateAnswerId {
        question_id: usize,
        answer_id: usize,
    },
    RegexError {
        source: regex::Error,
    },
}

impl From<regex::Error> for StoreError {
    fn from(item: regex::Error) -> Self {
        Self {
            error: StoreErrorEnum::RegexError { source: item },
        }
    }
}

#[derive(Debug, Getters)]
pub struct AnswerStore {
    id: usize,
    content: String,
}

#[derive(Debug)]
pub enum CompiledEntryMatch {
    Id { id: Vec<usize> },
    Content { content: Vec<Regex> },
}

#[derive(Debug, Getters)]
pub struct QuestionStore {
    id: usize,
    title: Option<String>,
    content: String,
    mode: QuestionMode,
    min_entries: Option<usize>,
    max_entries: Option<usize>,
    min_correct_entries: Option<usize>,
    max_wrong_entries: Option<usize>,
    correct_entry_match: Option<CompiledEntryMatch>,
    answer_ids: Vec<usize>,
    answers: HashMap<usize, AnswerStore>,
}

#[derive(Debug, Getters)]
pub struct SectionStore {
    id: usize,
    title: Option<String>,
    description: Option<String>,
    question_ids: Vec<usize>,
}

#[derive(Debug, Getters)]
pub struct QuizStore {
    uid: String,
    version: usize,
    title: Option<String>,
    description: Option<String>,
    quiz_mode: QuizMode,
    block_answer_updates_for: Option<Vec<QuestionStatusInput>>,
    section_ids: Vec<usize>,
    sections: HashMap<usize, SectionStore>,
    question_ids: Vec<usize>,
    questions: HashMap<usize, QuestionStore>,
}

impl From<&SectionInput> for SectionStore {
    fn from(section: &SectionInput) -> Self {
        Self {
            id: *section.id(),
            title: section.title().clone(),
            description: section.description().clone(),
            question_ids: section
                .questions()
                .iter()
                .map(|question| *question.id())
                .collect(),
        }
    }
}

impl From<&AnswerInput> for AnswerStore {
    fn from(answer: &AnswerInput) -> Self {
        Self {
            id: *answer.id(),
            content: answer.content().clone(),
        }
    }
}

impl TryFrom<&EntryMatch> for CompiledEntryMatch {
    type Error = StoreError;

    fn try_from(entry_match: &EntryMatch) -> StoreResult<Self> {
        let compiled_entry_match = match entry_match {
            EntryMatch::Id { id } => CompiledEntryMatch::Id { id: id.clone() },
            EntryMatch::Content { content } => CompiledEntryMatch::Content {
                content: content
                    .iter()
                    .map(|m| Regex::new(&format!("(?i){}", m)))
                    .collect::<Result<Vec<Regex>, regex::Error>>()?,
            },
        };

        Ok(compiled_entry_match)
    }
}

impl TryFrom<&QuestionInput> for QuestionStore {
    type Error = StoreError;

    fn try_from(question: &QuestionInput) -> StoreResult<Self> {
        let mut answer_ids = Vec::new();
        let mut answers = HashMap::new();

        match question.answers() {
            Some(question_answers) => {
                for answer in question_answers {
                    if answers.contains_key(answer.id()) {
                        return Err(StoreError {
                            error: StoreErrorEnum::DuplicateAnswerId {
                                question_id: *question.id(),
                                answer_id: *answer.id(),
                            },
                        });
                    }

                    answer_ids.push(*answer.id());
                    answers.insert(*answer.id(), AnswerStore::from(answer));
                }
            }
            None => {}
        }

        Ok(QuestionStore {
            id: *question.id(),
            title: question.title().clone(),
            content: question.content().clone(),
            mode: question.mode().clone(),
            min_entries: *question.min_entries(),
            max_entries: *question.max_entries(),
            min_correct_entries: *question.min_correct_entries(),
            max_wrong_entries: *question.max_wrong_entries(),
            correct_entry_match: match question.correct_entry_match() {
                Some(correct_entry_match) => Some(correct_entry_match.try_into()?),
                None => None,
            },
            answer_ids,
            answers,
        })
    }
}

impl TryFrom<&QuizInput> for QuizStore {
    type Error = StoreError;

    fn try_from(quiz: &QuizInput) -> StoreResult<Self> {
        let mut section_ids = Vec::new();
        let mut sections = HashMap::new();
        let mut question_ids = Vec::new();
        let mut questions = HashMap::new();

        for section in quiz.sections() {
            if sections.contains_key(section.id()) {
                return Err(StoreError {
                    error: StoreErrorEnum::DuplicateSectionId {
                        section_id: *section.id(),
                    },
                });
            }

            section_ids.push(*section.id());
            sections.insert(*section.id(), SectionStore::from(section));

            for question in section.questions() {
                if questions.contains_key(question.id()) {
                    return Err(StoreError {
                        error: StoreErrorEnum::DuplicateQuestionId {
                            section_id: *section.id(),
                            question_id: *question.id(),
                        },
                    });
                }

                question_ids.push(*question.id());
                questions.insert(*question.id(), QuestionStore::try_from(question)?);
            }
        }

        Ok(Self {
            uid: quiz.uid().into(),
            version: quiz.version().clone(),
            title: quiz.title().clone(),
            description: quiz.description().clone(),
            quiz_mode: quiz.mode().clone(),
            block_answer_updates_for: quiz.block_answer_updates_for().clone(),
            section_ids,
            sections,
            question_ids,
            questions,
        })
    }
}
