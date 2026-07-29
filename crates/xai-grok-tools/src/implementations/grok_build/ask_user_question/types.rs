//! Shared protocol and channel types for the AskUserQuestion blocking flow.
//!
//! These types define the request/response contract between:
//!
//! - **`xai-grok-tools`** — format helpers + wire types (Face already imports these).
//! - **next-code daemon** — tool blocks on a oneshot via [`UserQuestionSender`].
//! - **`pager_agent` / Face** — ACP `ext_method("x.ai/ask_user_question")` round-trip.
//!
//! Adapted from stock grok-build (no `educe` / `register_resource` — shim crate).

use std::collections::HashMap;
use std::fmt;

use indexmap::IndexMap;
use tokio::sync::{mpsc, oneshot};

use super::Question;

/// Annotation on a single question's answer.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QuestionAnnotation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// Mode context for the question UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AskUserQuestionMode {
    /// Normal mode. Client shows only Accept and Cancel.
    Default,
    /// Plan mode. Client shows Accept, Cancel, Chat about this, Skip interview.
    Plan,
}

/// ACP `ext_method` request payload (coordinator → client/pager).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AskUserQuestionExtRequest {
    pub session_id: String,
    pub tool_call_id: String,
    pub questions: Vec<Question>,
    pub mode: AskUserQuestionMode,
}

fn deserialize_string_or_vec_answers<'de, D>(
    deserializer: D,
) -> Result<IndexMap<String, Vec<String>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(serde::Deserialize)]
    #[serde(untagged)]
    enum StringOrVec {
        Vec(Vec<String>),
        String(String),
    }

    let raw: IndexMap<String, StringOrVec> = serde::Deserialize::deserialize(deserializer)?;
    Ok(raw
        .into_iter()
        .map(|(k, v)| match v {
            StringOrVec::Vec(vec) => (k, vec),
            StringOrVec::String(s) => (k, vec![s]),
        })
        .collect())
}

/// ACP `ext_method` response payload (client/pager → coordinator).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum AskUserQuestionExtResponse {
    Accepted {
        #[serde(deserialize_with = "deserialize_string_or_vec_answers")]
        answers: IndexMap<String, Vec<String>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        annotations: Option<HashMap<String, QuestionAnnotation>>,
    },
    ChatAboutThis {
        #[serde(default)]
        partial_answers: HashMap<String, String>,
    },
    SkipInterview {
        #[serde(default)]
        partial_answers: HashMap<String, String>,
    },
    Cancelled,
}

/// In-process result: coordinator → tool.
pub type UserQuestionResult = Result<UserQuestionResponse, UserQuestionError>;

/// Successful user response (all 4 user paths).
#[derive(Debug, Clone)]
pub enum UserQuestionResponse {
    Accepted {
        answers: IndexMap<String, Vec<String>>,
        annotations: Option<HashMap<String, QuestionAnnotation>>,
    },
    ChatAboutThis {
        questions: Vec<Question>,
        partial_answers: HashMap<String, String>,
    },
    SkipInterview {
        questions: Vec<Question>,
        partial_answers: HashMap<String, String>,
    },
    Cancelled,
}

/// Infrastructure failure (NOT a user action).
#[derive(Debug, Clone)]
pub enum UserQuestionError {
    TransportError(String),
    MalformedResponse(String),
}

/// In-process request: tool → coordinator (carries oneshot for reply).
pub struct UserQuestionRequest {
    pub tool_call_id: String,
    pub questions: Vec<Question>,
    pub result_tx: oneshot::Sender<UserQuestionResult>,
}

impl fmt::Debug for UserQuestionRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UserQuestionRequest")
            .field("tool_call_id", &self.tool_call_id)
            .field("questions", &self.questions)
            .field("result_tx", &"<oneshot>")
            .finish()
    }
}

/// Resource: `mpsc` sender injected so tools can emit [`UserQuestionRequest`].
#[derive(Clone)]
pub struct UserQuestionSender(pub mpsc::UnboundedSender<UserQuestionRequest>);

impl fmt::Debug for UserQuestionSender {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("UserQuestionSender")
            .field(&"<channel>")
            .finish()
    }
}

impl AskUserQuestionExtResponse {
    /// Convert wire ACP response into the in-process response type.
    pub fn into_response(self, questions: Vec<Question>) -> UserQuestionResponse {
        match self {
            Self::Accepted {
                answers,
                annotations,
            } => UserQuestionResponse::Accepted {
                answers,
                annotations,
            },
            Self::ChatAboutThis { partial_answers } => UserQuestionResponse::ChatAboutThis {
                questions,
                partial_answers,
            },
            Self::SkipInterview { partial_answers } => UserQuestionResponse::SkipInterview {
                questions,
                partial_answers,
            },
            Self::Cancelled => UserQuestionResponse::Cancelled,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::implementations::grok_build::ask_user_question::QuestionOption;

    fn sample_questions() -> Vec<Question> {
        vec![Question {
            question: "Which database?".to_string(),
            options: vec![QuestionOption {
                label: "Postgres".to_string(),
                description: "SQL".to_string(),
                preview: None,
                id: None,
            }],
            multi_select: None,
            header: None,
            id: None,
        }]
    }

    #[test]
    fn ext_response_accepted_roundtrip() {
        let answers = IndexMap::from([("Which database?".to_string(), vec!["Postgres".to_string()])]);
        let ext = AskUserQuestionExtResponse::Accepted {
            answers: answers.clone(),
            annotations: None,
        };
        let json = serde_json::to_string(&ext).unwrap();
        assert!(json.contains("\"outcome\":\"accepted\""));
        let back: AskUserQuestionExtResponse = serde_json::from_str(&json).unwrap();
        match back {
            AskUserQuestionExtResponse::Accepted { answers: a, .. } => {
                assert_eq!(a, answers);
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn into_response_chat_injects_questions() {
        let q = sample_questions();
        let ext = AskUserQuestionExtResponse::ChatAboutThis {
            partial_answers: HashMap::new(),
        };
        match ext.into_response(q.clone()) {
            UserQuestionResponse::ChatAboutThis { questions, .. } => {
                assert_eq!(questions.len(), 1);
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn accepted_answers_accept_legacy_string_values() {
        let json = r#"{
            "outcome": "accepted",
            "answers": { "Which database?": "Postgres" }
        }"#;
        let ext: AskUserQuestionExtResponse = serde_json::from_str(json).unwrap();
        match ext {
            AskUserQuestionExtResponse::Accepted { answers, .. } => {
                assert_eq!(
                    answers.get("Which database?").map(|v| v.as_slice()),
                    Some(["Postgres".to_string()].as_slice())
                );
            }
            other => panic!("unexpected {other:?}"),
        }
    }
}
