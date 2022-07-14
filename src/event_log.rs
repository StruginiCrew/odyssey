use derive_getters::Getters;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Getters)]
#[serde(rename_all = "camelCase")]
pub struct EventLog {
    uid: String,
    version: usize,
    events: Vec<Event>,
}

impl EventLog {
    pub fn new(uid: String, version: usize, events: Vec<Event>) -> Self {
        Self {
            uid,
            version,
            events,
        }
    }

    pub fn generation(&self) -> usize {
        self.events.len()
    }

    pub fn push(&mut self, event: Event) {
        self.events.push(event);
    }

    pub fn extract_events(self) -> Vec<Event> {
        self.events
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "event")]
#[serde(rename_all = "camelCase")]
pub enum Event {
    #[serde(rename_all = "camelCase")]
    SelectAnswers {
        question_id: usize,
        answer_ids: Vec<usize>,
    },
    #[serde(rename_all = "camelCase")]
    InputAnswers {
        question_id: usize,
        inputs: Vec<String>,
    },
    #[serde(rename_all = "camelCase")]
    ClearAnswers { question_id: usize },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_deserializes_event_log_json() {
        let input_json = include_str!("../tests/input/open_exam_event_log.json");
        let event_log = serde_json::from_str::<EventLog>(&input_json);
        assert!(event_log.is_ok());
    }
}
