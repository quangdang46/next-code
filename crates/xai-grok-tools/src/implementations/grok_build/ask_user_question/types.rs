use std::collections::HashMap;

use indexmap::IndexMap;

use super::Question;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QuestionAnnotation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AskUserQuestionMode {
    Default,
    Plan,
}

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
        questions: Vec<Question>,
        #[serde(default)]
        partial_answers: HashMap<String, String>,
    },
    SkipInterview {
        #[serde(default)]
        questions: Vec<Question>,
        #[serde(default)]
        partial_answers: HashMap<String, String>,
    },
    Cancelled,
}
