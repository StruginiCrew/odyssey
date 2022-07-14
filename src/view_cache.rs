use crate::view::{QuestionView, QuizView, SectionView};
use std::collections::HashMap;

struct CachedView<T> {
    generation: usize,
    view: T,
}

pub struct ViewCache {
    quiz_view: Option<CachedView<QuizView>>,
    question_views: HashMap<usize, CachedView<QuestionView>>,
    section_views: HashMap<usize, CachedView<SectionView>>,
}

impl ViewCache {
    pub fn new() -> Self {
        Self {
            quiz_view: None,
            question_views: HashMap::new(),
            section_views: HashMap::new(),
        }
    }

    pub fn cache_question(&mut self, generation: usize, question: QuestionView) -> QuestionView {
        self.question_views.insert(
            question.id().clone(),
            CachedView {
                generation,
                view: question.clone(),
            },
        );

        question
    }

    pub fn cache_section(&mut self, generation: usize, section: SectionView) -> SectionView {
        self.section_views.insert(
            section.id().clone(),
            CachedView {
                generation,
                view: section.clone(),
            },
        );

        section
    }

    pub fn cache_quiz(&mut self, generation: usize, quiz: QuizView) -> QuizView {
        self.quiz_view = Some(CachedView {
            generation,
            view: quiz.clone(),
        });

        quiz
    }

    pub fn question(&self, generation: usize, question_id: usize) -> Option<QuestionView> {
        match &self.question_views.get(&question_id) {
            Some(cached_view) if cached_view.generation == generation => {
                Some(cached_view.view.clone())
            }
            _ => None,
        }
    }

    pub fn section(&self, generation: usize, section_id: usize) -> Option<SectionView> {
        match &self.section_views.get(&section_id) {
            Some(cached_view) if cached_view.generation == generation => {
                Some(cached_view.view.clone())
            }
            _ => None,
        }
    }

    pub fn quiz(&self, generation: usize) -> Option<QuizView> {
        match &self.quiz_view {
            Some(view) if view.generation == generation => Some(view.view.clone()),
            _ => None,
        }
    }
}
