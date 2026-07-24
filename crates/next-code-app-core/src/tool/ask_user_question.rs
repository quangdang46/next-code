//! AskUserQuestion — structured Q&A overlay via Face ACP reverse request.
//!
//! Distinct from ambient `request_permission` (tool approval) and from
//! freeform `StdinRequest`. Fail closed when no Face/client bridge is wired
//! (no fire-and-forget `QuestionsSent` stub).

use super::{AskUserQuestionInputRequest, Tool, ToolContext, ToolOutput};
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{Value, json};
use xai_grok_tools::implementations::grok_build::ask_user_question::{
    AskUserQuestionExtResponse, AskUserQuestionParams, CANCEL_TEXT, Question,
    UserQuestionResponse, format_accepted_tool_result, format_chat_about_this,
    format_skip_interview,
};

pub struct AskUserQuestionTool;

impl AskUserQuestionTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for AskUserQuestionTool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
struct AskUserQuestionInput {
    questions: Vec<Question>,
}

#[async_trait]
impl Tool for AskUserQuestionTool {
    fn name(&self) -> &str {
        "AskUserQuestion"
    }

    fn description(&self) -> &str {
        r#"Ask the user one or more multiple-choice questions.

- Every question automatically gets an "Other" choice where the user can type their own answer.
- Put your recommended option first and append "(Recommended)" to its label."#
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["questions"],
            "properties": {
                "intent": super::intent_schema_property(),
                "questions": {
                    "type": "array",
                    "description": "The questions to ask, each with its own options.",
                    "minItems": 1,
                    "items": {
                        "type": "object",
                        "required": ["question", "options"],
                        "properties": {
                            "question": {
                                "type": "string",
                                "description": "The question to ask, phrased as a full question."
                            },
                            "options": {
                                "type": "array",
                                "description": "The choices for this question.",
                                "items": {
                                    "type": "object",
                                    "required": ["label", "description"],
                                    "properties": {
                                        "label": {
                                            "type": "string",
                                            "description": "Option text shown to the user. A few words at most."
                                        },
                                        "description": {
                                            "type": "string",
                                            "description": "What picking this option means or implies."
                                        },
                                        "preview": {
                                            "type": "string",
                                            "description": "Optional content shown while the option is focused."
                                        }
                                    }
                                }
                            },
                            "multi_select": {
                                "type": "boolean",
                                "description": "Let the user pick more than one option (default false)."
                            }
                        }
                    }
                }
            }
        })
    }

    fn declared_tier(&self) -> Option<next_code_tool_types::ToolTier> {
        Some(next_code_tool_types::ToolTier::Read)
    }

    async fn execute(&self, input: Value, ctx: ToolContext) -> Result<ToolOutput> {
        let params: AskUserQuestionInput = serde_json::from_value(input)?;
        if params.questions.is_empty() {
            return Ok(ToolOutput::new(
                "No questions provided. Continue with the task.",
            ));
        }

        {
            let mut seen = std::collections::HashSet::new();
            for q in &params.questions {
                if !seen.insert(q.question.as_str()) {
                    return Err(anyhow!("Duplicate question text: \"{}\"", q.question));
                }
            }
        }

        let Some(tx) = ctx.ask_user_question_tx.as_ref() else {
            // Fail closed: no Face/client bridge — do not pretend QuestionsSent.
            return Ok(ToolOutput::new(CANCEL_TEXT));
        };

        let (response_tx, response_rx) = tokio::sync::oneshot::channel();
        let request_id = format!("ask-{}", ctx.tool_call_id);
        let questions_json = serde_json::to_value(&params.questions)?;
        if tx
            .send(AskUserQuestionInputRequest {
                request_id,
                session_id: ctx.session_id.clone(),
                tool_call_id: ctx.tool_call_id.clone(),
                questions: questions_json,
                mode: "default".to_string(),
                response_tx,
            })
            .is_err()
        {
            return Err(anyhow!(
                "AskUserQuestion session ended unexpectedly (bridge channel closed)"
            ));
        }

        let wait = AskUserQuestionParams::default().wait_budget();
        let outcome = match wait {
            Some(dur) => tokio::time::timeout(dur, response_rx).await,
            None => Ok(response_rx.await),
        };

        let response_value = match outcome {
            Ok(Ok(Ok(v))) => v,
            Ok(Ok(Err(msg))) => {
                return Err(anyhow!("Failed to reach the client for user question: {msg}"));
            }
            Ok(Err(_closed)) => {
                return Err(anyhow!(
                    "AskUserQuestion session ended unexpectedly (client may have disconnected)"
                ));
            }
            Err(_elapsed) => {
                return Ok(ToolOutput::new(CANCEL_TEXT));
            }
        };

        let ext: AskUserQuestionExtResponse = serde_json::from_value(response_value)
            .map_err(|e| anyhow!("Client returned an invalid response to user question: {e}"))?;

        let message = match ext.into_response(params.questions.clone()) {
            UserQuestionResponse::Accepted {
                answers,
                annotations,
            } => format_accepted_tool_result(&answers, &annotations),
            UserQuestionResponse::ChatAboutThis {
                questions,
                partial_answers,
            } => format_chat_about_this(&questions, &partial_answers),
            UserQuestionResponse::SkipInterview {
                questions,
                partial_answers,
            } => format_skip_interview(&questions, &partial_answers),
            UserQuestionResponse::Cancelled => CANCEL_TEXT.to_string(),
        };

        Ok(ToolOutput::new(message))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use xai_grok_tools::implementations::grok_build::ask_user_question::{
        Question, QuestionOption,
    };

    fn sample_questions() -> Vec<Question> {
        vec![Question {
            question: "Which database?".to_string(),
            options: vec![
                QuestionOption {
                    label: "Postgres (Recommended)".to_string(),
                    description: "Relational".to_string(),
                    preview: None,
                    id: None,
                },
                QuestionOption {
                    label: "SQLite".to_string(),
                    description: "Embedded".to_string(),
                    preview: None,
                    id: None,
                },
            ],
            multi_select: None,
            id: None,
        }]
    }

    #[tokio::test]
    async fn accepted_path_formats_answers() {
        let (tx, mut rx) =
            tokio::sync::mpsc::unbounded_channel::<crate::tool::AskUserQuestionInputRequest>();
        let tool = AskUserQuestionTool::new();
        let questions = sample_questions();
        let input = json!({ "questions": questions });

        let handle = tokio::spawn(async move {
            let req = rx.recv().await.expect("request");
            let resp = AskUserQuestionExtResponse::Accepted {
                answers: indexmap::IndexMap::from([(
                    "Which database?".to_string(),
                    vec!["Postgres (Recommended)".to_string()],
                )]),
                annotations: None,
            };
            let _ = req
                .response_tx
                .send(Ok(serde_json::to_value(resp).unwrap()));
        });

        let ctx = ToolContext {
            session_id: "sess-1".into(),
            tool_call_id: "tc-1".into(),
            ask_user_question_tx: Some(tx),
            ..Default::default()
        };
        let out = tool.execute(input, ctx).await.unwrap();
        assert!(out.output.contains("User has answered your questions"));
        assert!(out.output.contains("Postgres (Recommended)"));
        handle.await.unwrap();
    }

    #[tokio::test]
    async fn cancelled_path_is_not_tool_error() {
        let (tx, mut rx) =
            tokio::sync::mpsc::unbounded_channel::<crate::tool::AskUserQuestionInputRequest>();
        let tool = AskUserQuestionTool::new();
        let input = json!({ "questions": sample_questions() });

        let handle = tokio::spawn(async move {
            let req = rx.recv().await.expect("request");
            let resp = AskUserQuestionExtResponse::Cancelled;
            let _ = req
                .response_tx
                .send(Ok(serde_json::to_value(resp).unwrap()));
        });

        let ctx = ToolContext {
            ask_user_question_tx: Some(tx),
            tool_call_id: "tc-2".into(),
            ..Default::default()
        };
        let out = tool.execute(input, ctx).await.unwrap();
        assert_eq!(out.output, CANCEL_TEXT);
        handle.await.unwrap();
    }

    #[tokio::test]
    async fn missing_bridge_fail_closed_cancel_text() {
        let tool = AskUserQuestionTool::new();
        let input = json!({ "questions": sample_questions() });
        let out = tool
            .execute(input, ToolContext::default())
            .await
            .unwrap();
        assert_eq!(out.output, CANCEL_TEXT);
    }
}
