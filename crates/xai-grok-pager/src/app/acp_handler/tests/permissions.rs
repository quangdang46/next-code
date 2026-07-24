#![cfg_attr(rustfmt, rustfmt::skip)]
    use super::*;

    /// The permission prompt must surface the payload an MCP call would
    /// send — both `UseTool` (meta-dispatch) and `MCPTool` (natively
    /// registered) raw_input shapes.
    #[test]
    fn mcp_args_lines_extracts_planned_tool_input() {
        for variant in ["UseTool", "MCPTool"] {
            let req = permission_req_with_raw_input(Some(serde_json::json!({
                "variant": variant,
                "tool_name": "jira__AddjiraComment",
                "tool_input": {"issue": "ABC-123", "body": "hello"},
            })));
            let lines = mcp_args_lines(&req);
            let joined = lines.join("\n");
            assert!(
                joined.contains("\"issue\": \"ABC-123\""),
                "{variant}: {joined}"
            );
            assert!(
                joined.contains("\"body\": \"hello\""),
                "{variant}: {joined}"
            );
        }
    }

    /// Non-MCP raw_input (bash, edit, gateway `{command}` shapes) must not
    /// grow a JSON dump — those prompts have dedicated displays.
    #[test]
    fn mcp_args_lines_empty_for_non_mcp_shapes() {
        for raw in [
            None,
            Some(serde_json::json!({"variant": "Bash", "command": "ls", "description": "d"})),
            Some(serde_json::json!({"command": "rm -rf /"})),
            Some(serde_json::json!({"file_path": "/tmp/x"})),
            Some(serde_json::json!("not-an-object")),
        ] {
            let req = permission_req_with_raw_input(raw.clone());
            assert!(
                mcp_args_lines(&req).is_empty(),
                "expected empty for {raw:?}"
            );
        }
    }

    /// A `tool_input` that is missing or JSON null renders nothing rather
    /// than a misleading `null`.
    #[test]
    fn mcp_args_lines_empty_for_missing_or_null_input() {
        for raw in [
            serde_json::json!({"variant": "UseTool", "tool_name": "t"}),
            serde_json::json!({"variant": "UseTool", "tool_name": "t", "tool_input": null}),
        ] {
            let req = permission_req_with_raw_input(Some(raw));
            assert!(mcp_args_lines(&req).is_empty());
        }
    }

    /// A pathological single-line value (e.g. an embedded base64 blob) is
    /// elided at `MCP_ARGS_MAX_LINE_CHARS` so per-frame wrap cost stays
    /// bounded. Uses a multi-byte char to pin char (not byte) slicing.
    #[test]
    fn mcp_args_lines_caps_line_length() {
        let req = permission_req_with_raw_input(Some(serde_json::json!({
            "variant": "UseTool",
            "tool_name": "t",
            "tool_input": {"blob": "é".repeat(MCP_ARGS_MAX_LINE_CHARS * 2)},
        })));
        let lines = mcp_args_lines(&req);
        let long = lines
            .iter()
            .find(|l| l.contains("é"))
            .expect("blob line present");
        assert_eq!(long.chars().count(), MCP_ARGS_MAX_LINE_CHARS + 1);
        assert!(long.ends_with('…'));
    }

    /// Pathologically large payloads are capped in storage with an explicit
    /// hidden-line count (the overlay clips further at render time).
    #[test]
    fn mcp_args_lines_caps_stored_lines() {
        let big: serde_json::Map<String, serde_json::Value> = (0..MCP_ARGS_MAX_LINES + 50)
            .map(|i| (format!("k{i:04}"), serde_json::Value::from(i)))
            .collect();
        let req = permission_req_with_raw_input(Some(serde_json::json!({
            "variant": "UseTool",
            "tool_name": "t",
            "tool_input": big,
        })));
        let lines = mcp_args_lines(&req);
        assert_eq!(lines.len(), MCP_ARGS_MAX_LINES + 1);
        let last = lines.last().unwrap();
        assert!(
            last.starts_with("… (+") && last.ends_with(" more lines)"),
            "unexpected tail: {last}"
        );
    }

    /// Manual recap with an uncommitted in-flight spinner: filled in place
    /// (no second block), animation stopped.
    #[test]
    fn recap_fills_uncommitted_spinner_in_place() {
        let mut agent = make_agent(Some("s1"));
        let spinner = agent
            .scrollback
            .push(crate::scrollback::entry::ScrollbackEntry::running(
                recap_block(""),
            ));
        agent.pending_recap_entry = Some(spinner);

        apply_recap_block(&mut agent, false, recap_block("THE RECAP"));

        assert_eq!(agent.scrollback.len(), 1, "filled in place, not appended");
        let entry = agent.scrollback.get_by_id(spinner).expect("entry kept");
        assert!(!entry.is_running, "spinner animation stopped");
        assert!(agent.pending_recap_entry.is_none());
    }

    /// Regression (minimal mode): the spinner was already committed into
    /// native scrollback (print-once) — an in-place fill would never reach the
    /// terminal. The stale committed entry is dropped from state and the recap
    /// appended as a fresh (uncommitted) block so the commit pass prints it.
    #[test]
    fn recap_reprints_fresh_block_when_spinner_already_committed() {
        let mut agent = make_agent(Some("s1"));
        let spinner = agent
            .scrollback
            .push(crate::scrollback::entry::ScrollbackEntry::running(
                recap_block(""),
            ));
        agent.pending_recap_entry = Some(spinner);
        // The minimal idle commit pass consumed the spinner.
        agent.scrollback.finish_running(spinner);
        agent.scrollback.mark_committed(0);
        agent.scrollback.set_commit_scan_cursor(1);
        assert!(agent.scrollback.is_committed(spinner));

        apply_recap_block(&mut agent, false, recap_block("THE RECAP"));

        assert_eq!(
            agent.scrollback.len(),
            1,
            "stale committed spinner dropped, fresh block appended"
        );
        let fresh = agent.scrollback.get(0).expect("fresh block");
        assert_ne!(fresh.id, spinner, "a NEW entry, not the committed one");
        assert!(
            !agent.scrollback.is_committed(fresh.id),
            "fresh block is uncommitted so the commit pass will print it"
        );
    }

    /// An automatic recap never consumes the manual loading slot — it always
    /// appends its own block and leaves the pending spinner alone.
    #[test]
    fn auto_recap_appends_and_leaves_manual_spinner_pending() {
        let mut agent = make_agent(Some("s1"));
        let spinner = agent
            .scrollback
            .push(crate::scrollback::entry::ScrollbackEntry::running(
                recap_block(""),
            ));
        agent.pending_recap_entry = Some(spinner);

        apply_recap_block(&mut agent, true, recap_block("AUTO RECAP"));

        assert_eq!(agent.scrollback.len(), 2, "auto recap appended");
        assert_eq!(agent.pending_recap_entry, Some(spinner));
    }

    #[test]
    fn late_auto_recap_dropped_when_agent_not_idle() {
        assert!(should_drop_late_auto_recap(true, false, false));
        assert!(
            !should_drop_late_auto_recap(true, false, true),
            "idle agent: show auto recap"
        );
        assert!(
            !should_drop_late_auto_recap(false, false, false),
            "manual /recap always shown"
        );
        assert!(
            !should_drop_late_auto_recap(true, true, false),
            "history replay rebuilds scrollback even mid-turn"
        );
    }

    fn permission_req_execute(raw: serde_json::Value, title: &str) -> acp::RequestPermissionRequest {
        let fields = acp::ToolCallUpdateFields::new()
            .title(title.to_string())
            .kind(acp::ToolKind::Execute)
            .raw_input(raw);
        acp::RequestPermissionRequest::new(
            acp::SessionId::new(std::sync::Arc::from("s1")),
            acp::ToolCallUpdate::new(
                acp::ToolCallId::new(std::sync::Arc::from("call-1")),
                fields,
            ),
            vec![],
        )
    }

    fn permission_req_edit(raw: serde_json::Value, title: &str) -> acp::RequestPermissionRequest {
        let fields = acp::ToolCallUpdateFields::new()
            .title(title.to_string())
            .kind(acp::ToolKind::Edit)
            .raw_input(raw);
        let options = vec![acp::PermissionOption::new(
            acp::PermissionOptionId::new(std::sync::Arc::from("allow-always")),
            "Always allow edits".to_string(),
            acp::PermissionOptionKind::AllowAlways,
        )];
        acp::RequestPermissionRequest::new(
            acp::SessionId::new(std::sync::Arc::from("s1")),
            acp::ToolCallUpdate::new(
                acp::ToolCallId::new(std::sync::Arc::from("call-1")),
                fields,
            ),
            options,
        )
    }

    #[test]
    fn build_permission_display_bash_shows_command_cwd_and_risk() {
        let req = permission_req_execute(
            serde_json::json!({
                "command": "rm -rf /tmp/x",
                "description": "Clean scratch",
                "cwd": "/repo",
                "permission_reason": "policy gate",
            }),
            "Clean scratch",
        );
        let (title, description, bash_cmd) = build_permission_display(&req, None);
        assert_eq!(title, "Clean scratch");
        assert_eq!(bash_cmd.as_deref(), Some("rm -rf /tmp/x"));
        let joined = description.join("\n");
        assert!(joined.contains("cwd: /repo"), "{joined}");
        assert!(joined.contains("reason: policy gate"), "{joined}");
        assert!(joined.contains("risk: destructive delete"), "{joined}");
    }

    #[test]
    fn build_permission_display_edit_shows_path_and_diff_preview() {
        let req = permission_req_edit(
            serde_json::json!({
                "file_path": "src/lib.rs",
                "old_string": "fn a() {}\n",
                "new_string": "fn a() { todo!() }\n",
            }),
            "Allow Edit to src/lib.rs?",
        );
        let (title, description, bash_cmd) = build_permission_display(&req, None);
        assert_eq!(title, "Allow Edit to src/lib.rs?");
        assert!(bash_cmd.is_none());
        let joined = description.join("\n");
        assert!(joined.contains("file: src/lib.rs"), "{joined}");
        assert!(joined.contains("- fn a() {}"), "{joined}");
        assert!(joined.contains("+ fn a() { todo!() }"), "{joined}");
    }

    #[test]
    fn build_permission_display_write_shows_content_preview() {
        let req = permission_req_edit(
            serde_json::json!({
                "file_path": "README.md",
                "content": "# Hello\nworld\n",
            }),
            "Allow Write to README.md?",
        );
        let (title, description, _) = build_permission_display(&req, None);
        assert_eq!(title, "Allow Write to README.md?");
        let joined = description.join("\n");
        assert!(joined.contains("file: README.md"), "{joined}");
        assert!(joined.contains("+ # Hello"), "{joined}");
    }

