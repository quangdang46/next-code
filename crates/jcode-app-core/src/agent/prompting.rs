use super::Agent;
use crate::logging;
use crate::message::{Message, ToolDefinition};

impl Agent {
    pub(super) fn log_prompt_prefix_accounting(
        &self,
        split: &crate::prompt::SplitSystemPrompt,
        tools: &[ToolDefinition],
    ) {
        let system_tokens = split.estimated_tokens();
        let tool_tokens = ToolDefinition::aggregate_prompt_token_estimate(tools);
        let prefix_tokens = system_tokens + tool_tokens;
        logging::info(&format!(
            "Prompt prefix estimate: total={} tokens (system={} tools={})",
            prefix_tokens, system_tokens, tool_tokens
        ));
    }

    pub(super) fn build_memory_prompt_nonblocking_shared(
        &self,
        messages: std::sync::Arc<[Message]>,
        _memory_event_tx: Option<crate::memory::MemoryEventSink>,
    ) -> Option<crate::memory::PendingMemory> {
        if !self.memory_enabled {
            return None;
        }

        let session_id = &self.session.id;

        let pending = if crate::message::ends_with_fresh_user_turn(&messages) {
            crate::memory::take_pending_memory(session_id)
        } else {
            None
        };

        // Use the persistent memory-agent pipeline as the single source of truth.
        // Running both this and the legacy MemoryManager background retrieval path
        // can prepare overlapping pending prompts for the same turn, which makes
        // memory injection feel overly aggressive.
        crate::memory_agent::update_context_sync_with_dir(
            session_id,
            messages,
            self.session.working_dir.clone(),
        );

        pending
    }

    fn append_current_turn_system_reminder(&self, split: &mut crate::prompt::SplitSystemPrompt) {
        let Some(reminder) = self
            .current_turn_system_reminder
            .as_ref()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
        else {
            return;
        };

        if !split.dynamic_part.is_empty() {
            split.dynamic_part.push_str("\n\n");
        }
        split.dynamic_part.push_str("# System Reminder\n\n");
        split.dynamic_part.push_str(reminder);
    }

    /// Build split system prompt for better caching
    /// Returns static (cacheable) and dynamic (not cached) parts separately
    pub(super) fn build_system_prompt_split(
        &self,
        memory_prompt: Option<&str>,
    ) -> crate::prompt::SplitSystemPrompt {
        if let Some(ref override_prompt) = self.system_prompt_override {
            return crate::prompt::SplitSystemPrompt {
                static_part: override_prompt.clone(),
                dynamic_part: String::new(),
            };
        }

        let skills = self.current_skills_snapshot();
        let skill_prompt = self
            .active_skill
            .as_ref()
            .and_then(|name| skills.get(name).map(|skill| skill.get_prompt().to_string()));

        let available_skills: Vec<crate::prompt::SkillInfo> = self
            .current_skills_snapshot()
            .list()
            .iter()
            .map(|skill| crate::prompt::SkillInfo {
                name: skill.name.clone(),
                description: skill.description.clone(),
            })
            .collect();

        let working_dir = self
            .session
            .working_dir
            .as_ref()
            .map(std::path::PathBuf::from);

        // Detect keywords, update mode state, execute workflows, build prompt.
        // We skip the whole pipeline on empty input so an empty/system-only turn
        // does not burn a turn of any active mode's budget.
        let keyword_prompt = {
            let latest_input = self
                .session
                .messages
                .iter()
                .rev()
                .find(|m| matches!(m.role, crate::message::Role::User))
                .and_then(|m| {
                    m.content.iter().find_map(|b| match b {
                        crate::message::ContentBlock::Text { text, .. } => Some(text.as_str()),
                        _ => None,
                    })
                })
                .unwrap_or("");
            if latest_input.is_empty() {
                None
            } else {
                let detections = jcode_keywords::detect_keywords(latest_input);
                let mut mode_state =
                    jcode_keywords::state::update_modes(&detections, working_dir.as_deref());

                // Surface any mode conflicts (TDD + ultrawork, etc.) to logs.
                let active_kinds: Vec<jcode_keywords::registry::WorkflowKind> =
                    mode_state.active_modes.iter().map(|m| m.workflow).collect();
                for conflict in jcode_keywords::conflict::check_conflicts(&active_kinds) {
                    crate::logging::warn(&jcode_keywords::conflict::format_warning(&conflict));
                }

                // Process PREVIOUS turn's LLM response (phase transitions, completion)
                // This runs at the START of the current turn, using last turn's response
                if let Some(last_assistant) = self
                    .session
                    .messages
                    .iter()
                    .rev()
                    .find(|m| matches!(m.role, crate::message::Role::Assistant))
                    .and_then(|m| {
                        m.content.iter().find_map(|b| match b {
                            crate::message::ContentBlock::Text { text, .. } => Some(text.as_str()),
                            _ => None,
                        })
                    })
                {
                    let response_actions =
                        jcode_keywords::process_turn_response(&mode_state, last_assistant);
                    if !response_actions.is_empty() {
                        let _ = jcode_keywords::apply_actions(&mut mode_state, &response_actions);
                    }
                }

                // Classify task size so heavy workflows can suppress themselves for
                // trivial requests (e.g. a one-line "$ultrawork fix typo").
                let task_size = jcode_keywords::task_size::classify(latest_input);

                // Execute active workflows for THIS turn
                let actions = jcode_keywords::execute_active_workflows(
                    &mode_state,
                    latest_input,
                    working_dir.as_deref(),
                    &self.session.id,
                    task_size,
                );
                if !actions.is_empty() {
                    let (summaries, deferred) =
                        jcode_keywords::apply_actions(&mut mode_state, &actions);
                    for s in &summaries {
                        crate::logging::info(&format!("Keyword workflow: {}", s));
                    }
                    if !deferred.is_empty() {
                        crate::logging::warn(&format!(
                            "Keyword workflow: {} spawn action(s) deferred — they will not run until SubagentTool is wired from the agent runtime. (See issue #391 follow-up.)",
                            deferred.len()
                        ));
                    }
                }

                // Persist metadata to disk
                jcode_keywords::state::save_state(&mode_state, working_dir.as_deref());

                // Build workflow prompt
                let prompt = jcode_keywords::build_workflow_prompt(&mode_state);
                if prompt.is_empty() {
                    None
                } else {
                    Some(prompt)
                }
            }
        };

        let (mut split, _context_info) = crate::prompt::build_system_prompt_split(
            skill_prompt.as_deref(),
            &available_skills,
            self.session.is_canary,
            memory_prompt,
            working_dir.as_deref(),
            keyword_prompt,
        );

        self.append_current_turn_system_reminder(&mut split);

        split
    }

    /// Non-blocking memory prompt - takes pending result and spawns check for next turn
    pub(super) fn build_memory_prompt_nonblocking(
        &self,
        messages: &[Message],
        _memory_event_tx: Option<crate::memory::MemoryEventSink>,
    ) -> Option<crate::memory::PendingMemory> {
        self.build_memory_prompt_nonblocking_shared(messages.to_vec().into(), _memory_event_tx)
    }
}
