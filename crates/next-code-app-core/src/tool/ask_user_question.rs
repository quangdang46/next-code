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
    AskUserQuestionExtResponse, AskUserQuestionParams, CANCEL_TEXT, MAX_OPTIONS_PER_QUESTION,
    MAX_QUESTIONS, MIN_OPTIONS_PER_QUESTION, Question, UserQuestionResponse,
    format_accepted_tool_result, format_chat_about_this, format_skip_interview,
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
        r#"Asks the user multiple choice questions to gather information, clarify ambiguity, understand preferences, make decisions or offer them choices.

Use this tool when you need to ask the user questions during execution. This allows you to:
1. Gather user preferences or requirements
2. Clarify ambiguous instructions
3. Get decisions on implementation choices as you work
4. Offer choices to the user about what direction to take.

Usage notes:
- Users will always be able to select "Other" to provide custom text input
- Ask 1–4 questions per call; each question should have 2–4 options
- Provide a short `header` chip label (≤12 chars) per question for the tab bar
- Use multiSelect: true (or multi_select: true) to allow multiple answers for a question; mutually exclusive choices stay single-select (default)
- If you recommend a specific option, make that the first option in the list and add "(Recommended)" at the end of the label"#
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["questions"],
            "properties": {
                "intent": super::intent_schema_property(),
                "questions": {
                    "type": "array",
                    "description": "Questions to ask the user (1-4 questions).",
                    "minItems": 1,
                    "maxItems": 4,
                    "items": {
                        "type": "object",
                        "required": ["question", "options"],
                        "properties": {
                            "question": {
                                "type": "string",
                                "description": "The complete question to ask the user. Should be clear, specific, and end with a question mark. If multiSelect is true, phrase it accordingly (e.g. \"Which features do you want to enable?\")."
                            },
                            "header": {
                                "type": "string",
                                "description": "Very short label displayed as a chip/tag (max 12 chars). Examples: \"Auth method\", \"Library\", \"Approach\"."
                            },
                            "options": {
                                "type": "array",
                                "description": "The available choices for this question. Must have 2-4 options. Do not include an 'Other' option — that is provided automatically.",
                                "minItems": 2,
                                "maxItems": 4,
                                "items": {
                                    "type": "object",
                                    "required": ["label", "description"],
                                    "properties": {
                                        "label": {
                                            "type": "string",
                                            "description": "The display text for this option. Concise (1-5 words). Put the recommended option first and append \"(Recommended)\" to its label."
                                        },
                                        "description": {
                                            "type": "string",
                                            "description": "Explanation of what this option means or what will happen if chosen."
                                        },
                                        "preview": {
                                            "type": "string",
                                            "description": "Optional preview content shown while the option is focused (single-select only)."
                                        }
                                    }
                                }
                            },
                            "multiSelect": {
                                "type": "boolean",
                                "description": "Set to true to allow the user to select multiple options instead of just one. Use when choices are not mutually exclusive. Default false. (Alias: multi_select.)"
                            },
                            "multi_select": {
                                "type": "boolean",
                                "description": "Snake_case alias of multiSelect. Prefer multiSelect for Claude Code parity."
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
        if params.questions.len() > MAX_QUESTIONS {
            return Err(anyhow!(
                "AskUserQuestion accepts at most {MAX_QUESTIONS} questions per call (got {})",
                params.questions.len()
            ));
        }

        {
            let mut seen = std::collections::HashSet::new();
            for q in &params.questions {
                if !seen.insert(q.question.as_str()) {
                    return Err(anyhow!("Duplicate question text: \"{}\"", q.question));
                }
                let n = q.options.len();
                if n < MIN_OPTIONS_PER_QUESTION || n > MAX_OPTIONS_PER_QUESTION {
                    return Err(anyhow!(
                        "Question \"{}\" must have {MIN_OPTIONS_PER_QUESTION}-{MAX_OPTIONS_PER_QUESTION} options (got {n})",
                        q.question
                    ));
                }
                let mut labels = std::collections::HashSet::new();
                for opt in &q.options {
                    if !labels.insert(opt.label.as_str()) {
                        return Err(anyhow!(
                            "Duplicate option label \"{}\" in question \"{}\"",
                            opt.label,
                            q.question
                        ));
                    }
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
            header: Some("Database".to_string()),
            id: None,
        }]
    }

    #[test]
    fn schema_exposes_header_multiselect_and_limits() {
        let schema = AskUserQuestionTool::new().parameters_schema();
        let questions = &schema["properties"]["questions"];
        assert_eq!(questions["maxItems"], 4);
        let item = &questions["items"]["properties"];
        assert!(item.get("header").is_some());
        assert!(item.get("multiSelect").is_some());
        assert_eq!(item["options"]["minItems"], 2);
        assert_eq!(item["options"]["maxItems"], 4);
    }

    #[test]
    fn accepts_claude_shaped_input_json() {
        let input = json!({
            "questions": [{
                "question": "Which features should we enable?",
                "header": "Features",
                "options": [
                    {"label": "Auth (Recommended)", "description": "Login"},
                    {"label": "Logging", "description": "Logs"},
                    {"label": "Metrics", "description": "Telemetry"}
                ],
                "multiSelect": true
            }]
        });
        let parsed: AskUserQuestionInput = serde_json::from_value(input).unwrap();
        assert_eq!(parsed.questions.len(), 1);
        assert_eq!(parsed.questions[0].header.as_deref(), Some("Features"));
        assert_eq!(parsed.questions[0].multi_select, Some(true));
    }

    #[tokio::test]
    async fn accepted_path_formats_answers() {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<AskUserQuestionInputRequest>();
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
    async fn accepted_multi_select_joins_labels() {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<AskUserQuestionInputRequest>();
        let tool = AskUserQuestionTool::new();
        let input = json!({
            "questions": [{
                "question": "Which features?",
                "header": "Features",
                "multiSelect": true,
                "options": [
                    {"label": "Auth", "description": "a"},
                    {"label": "Logging", "description": "b"}
                ]
            }]
        });

        let handle = tokio::spawn(async move {
            let req = rx.recv().await.expect("request");
            let resp = AskUserQuestionExtResponse::Accepted {
                answers: indexmap::IndexMap::from([(
                    "Which features?".to_string(),
                    vec!["Auth".to_string(), "Logging".to_string()],
                )]),
                annotations: None,
            };
            let _ = req
                .response_tx
                .send(Ok(serde_json::to_value(resp).unwrap()));
        });

        let ctx = ToolContext {
            ask_user_question_tx: Some(tx),
            tool_call_id: "tc-ms".into(),
            ..Default::default()
        };
        let out = tool.execute(input, ctx).await.unwrap();
        assert!(out.output.contains("Auth, Logging"));
        handle.await.unwrap();
    }

    #[tokio::test]
    async fn rejects_too_many_questions() {
        let tool = AskUserQuestionTool::new();
        let q = |i: usize| {
            json!({
                "question": format!("Q{i}?"),
                "header": format!("H{i}"),
                "options": [
                    {"label": "A", "description": "a"},
                    {"label": "B", "description": "b"}
                ]
            })
        };
        let input = json!({ "questions": [q(1), q(2), q(3), q(4), q(5)] });
        let err = tool
            .execute(input, ToolContext::default())
            .await
            .unwrap_err();
        assert!(err.to_string().contains("at most 4"));
    }

    #[tokio::test]
    async fn cancelled_path_is_not_tool_error() {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<AskUserQuestionInputRequest>();
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
