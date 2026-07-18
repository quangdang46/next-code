# Pre-merge gate — `next-code-base` lib suite

| Field | Value |
| --- | --- |
| Package | `next-code-base` (`--lib`) |
| Branch / HEAD | `orch/premerge/check-base` @ `00428e461` |
| Tip message | `refactor: remove residual next-code hosted subscription surface` |
| Command | `NEXT_CODE_NO_TELEMETRY=1 cargo test -p next-code-base --lib -- --test-threads=1` |
| Log | `/tmp/premerge-base-lib.txt` (copy: `$CLAUDE_JOB_DIR/tmp/premerge-base-lib.txt`) |
| Duration | 55.58s |
| Date | 2026-07-18 |

## 1. Pass / fail counts

| Result | Count |
| --- | ---: |
| **Passed** | **1072** |
| **Failed** | **12** |
| **Ignored** | **1** |
| Measured / filtered | 0 / 0 |
| Total run | 1085 |

`test result: FAILED. 1072 passed; 12 failed; 1 ignored; 0 measured; 0 filtered out; finished in 55.58s`

Serial (`--test-threads=1`) completed well under the 15-minute budget; no parallel re-run was needed.

## 2. Full list of failing tests

1. `config::tests::test_generated_default_config_uses_low_openai_reasoning_effort`
2. `config::tests::tool_config_acp_profile_allows_core_coding_plus_batch`
3. `config::tests::tool_config_minimal_profile_allows_core_coding_tools`
4. `memory::tests::hybrid_fuse_returns_dense_hits_without_lexical_overlap`
5. `memory::tests::score_and_filter_prioritizes_matching_skill_memories`
6. `platform::platform_tests::spawn_detached_creates_new_session`
7. `provider::tests::test_should_failover_on_access_denied`
8. `session::tests::cases::initial_session_context_can_refresh_before_real_conversation`
9. `session::tests::cases::initial_session_context_does_not_refresh_after_real_conversation`
10. `session::tests::cases::initial_session_context_preserves_explicitly_bound_cwd_when_inserted`
11. `session::tests::cases::test_render_messages_shows_auto_poke_continuations_as_system_not_user`
12. `skill::tests::reload_global_excludes_project_local_skills`

## 3. Per-failure classification

Classification key:

- **A)** residual rebrand / telemetry / subscription surface
- **B)** pre-existing / unrelated product drift (tests lag product)
- **C)** environmental flake / isolation hazard

### Config (3)

#### `test_generated_default_config_uses_low_openai_reasoning_effort`
- **Symptom:** panic — `generated default config should document the Luna memory sidecar default`
- **Hypothesis:** Test expects generated `config.toml` to contain `memory_model = "gpt-5.6-luna"` and a `"reasoning effort \"none\""` doc string. `create_default_config_file` template currently has `# memory_model = "claude-haiku-4"` (commented) and no Luna/none wording. Note: `swarm_spawn_mode = "inline"` assertion can pass spuriously via a keybinding *comment* while the real `[agents]` default in the template is `swarm_spawn_mode = "visible"`. Sidecar code still defines `SIDECAR_OPENAI_MODEL = "gpt-5.6-luna"`, so this is template/test drift, not a telemetry purge miss.
- **Class:** **B**

#### `tool_config_acp_profile_allows_core_coding_plus_batch`
- **Symptom:** `assertion failed: allowed.contains("apply_patch")`
- **Hypothesis:** ACP allow-list in `ToolConfig::base_allowed_tools` is now `bash, read, write, edit, ffs_grep, ffs_glob, ls, batch`. Product unified patching under the single `edit` tool (`edit_mode` backends include `apply_patch`); profile lists no longer expose a top-level `apply_patch` tool. Test still asserts the old tool name.
- **Class:** **B**

#### `tool_config_minimal_profile_allows_core_coding_tools`
- **Symptom:** same `apply_patch` assertion failure
- **Hypothesis:** Same product change as above; minimal/lite profile is `bash, read, write, edit, ffs_grep, ffs_glob, ls`. Default-file comments still mention `apply_patch` as a “core coding tool,” which is doc lag, not rebrand residue.
- **Class:** **B**

### Memory ranking (2)

#### `hybrid_fuse_returns_dense_hits_without_lexical_overlap`
- **Symptom:** `assertion failed: !ranked.is_empty()`
- **Hypothesis:** `hybrid_fuse` only dense-scores entries whose `effective_embedding_model()` matches `embedding_backend::active_model_id()`. Test fixtures use `with_embedding(...)` (no model tag) → legacy `minilm-l6-v2`. If the active backend is not that id (or dense pool is empty and BM25 has zero lexical overlap with `zzz_nonmatching_query_token`), RRF returns empty. Product added cross-model gating; unit fixtures were not updated to `with_embedding_for_model` / force local backend.
- **Class:** **B**

#### `score_and_filter_prioritizes_matching_skill_memories`
- **Symptom:** `left: 0` vs `right: 2` on ranked length
- **Hypothesis:** Same model-space gate in `score_and_filter` filters out both untagged fixtures before skill-bonus ranking runs, so the function returns `Ok([])` instead of two scored hits.
- **Class:** **B**

### Platform (1)

#### `spawn_detached_creates_new_session`
- **Symptom:** `child should exit successfully` after `spawn_detached` + `wait`
- **Hypothesis:** Unix path uses `pre_exec` + `setsid()` then runs `sh -c 'ps -o sid= -p $$ > …'`. Failure is on child exit status, not the SID asserts — likely environment (sandbox / restricted `ps` / shell) rather than product logic. Unrelated to rebrand.
- **Class:** **C**

### Provider failover (1)

#### `test_should_failover_on_access_denied`
- **Symptom:** `classify_failover_error("Access denied: account suspended").should_failover()` is false
- **Hypothesis:** `next_code_provider_core` maps structured `accessdenied` / permission failures to `ErrorCode::PermissionDenied` with `FailoverDecision::None` (non-retryable). Bare message `"Access denied: account suspended"` has no 401/403 status and is not in `RETRYABLE_MESSAGE_PATTERNS`, so classifier correctly (per current design) refuses failover. Test still expects the older “any access denied → failover” behavior.
- **Class:** **B**

### Session context / render (4)

#### `initial_session_context_can_refresh_before_real_conversation`
- **Symptom:** context preview does not contain `Working directory: {first_dir}`
- **Hypothesis:** On macOS, `set_current_dir(temp.path())` then `current_dir()` often resolves through `/var` → `/private/var`, while `TempDir::path().display()` stays on the uncanonicalized form. Session stores the process cwd string; the test compares against the temp path object. Assertion panics **before** the cwd restore runs, so the process is left sitting in a directory that is then deleted when `TempDir` drops.
- **Class:** **B** (path canonicalization / test hygiene; not rebrand)

#### `initial_session_context_does_not_refresh_after_real_conversation`
- **Symptom:** `Error: No such file or directory (os error 2)`
- **Hypothesis:** Cascade from the previous panic: process cwd is already a deleted temp path, so `std::env::current_dir()` / `set_current_dir` at test start fails. Same underlying isolation issue.
- **Class:** **B** (cascade) / **C** (isolation hazard)

#### `initial_session_context_preserves_explicitly_bound_cwd_when_inserted`
- **Symptom:** same `os error 2`
- **Hypothesis:** Same cwd cascade as above.
- **Class:** **B** / **C**

#### `test_render_messages_shows_auto_poke_continuations_as_system_not_user`
- **Symptom:** quality continuation should render as system — asserts content contains `"before finalizing"`, but actual text is  
  `Your completion confidence is missing or not high enough. Validate the completed result more thoroughly, address any remaining issues, and then reassess whether the work is ready to finalize.`
- **Hypothesis:** Renderer correctly demotes auto-poke / completion continuations to `role: "system"`. Only the **substring** in the test is stale (`before finalizing` vs current `ready to finalize` wording in `TODO_COMPLETION_CONTINUATION_MESSAGE`). Product behavior looks correct.
- **Class:** **B**

### Skills (1)

#### `reload_global_excludes_project_local_skills`
- **Symptom:** `cwd: Os { code: 2, kind: NotFound }` at `std::env::current_dir().expect("cwd")`
- **Hypothesis:** Same deleted-cwd cascade left by the session context panics (despite `lock_test_env`). Not a skill-loader regression and not rebrand-related.
- **Class:** **C** (downstream of B session isolation)

## 4. Summary by class

| Class | Count | Tests |
| --- | ---: | --- |
| **A — residual rebrand / telemetry / subscription** | **0** | — |
| **B — pre-existing / product–test drift** | **10** | config×3, memory×2, failover×1, session path/cascade×3, auto-poke substring×1 |
| **C — environmental / isolation flake** | **2** (+ cascade on 2 session tests) | `spawn_detached…`, `reload_global…` |

Telemetry purge / hosted subscription surface removal (`NEXT_CODE_NO_TELEMETRY=1`, recent commits) did **not** surface as any of the 12 failures. The failure set matches the earlier ~12–15 “unrelated product drift” pattern (config defaults, apply_patch profiles, memory ranking, session cwd, skill cwd, platform spawn, failover).

## 5. Merge recommendation (this package only)

### **ok-with-known-failures**

**Rationale**

- 1072/1085 lib tests pass on current HEAD under serial execution.
- **Zero** failures classified as residual rebrand / telemetry / subscription (class A).
- All 12 failures are either:
  - stale assertions vs intentional product changes (`edit` vs `apply_patch`, Luna template docs, failover PermissionDenied policy, auto-poke copy, embedding model gating), or
  - cwd/path isolation and platform environment issues that predate this tip’s purge work.
- No evidence that merging this package’s current tree would reintroduce hosted subscription login surface or product telemetry call sites in `next-code-base` lib behavior covered by this suite.

**Not a clean green gate.** If policy requires a fully green `next-code-base --lib` before merge, treat as **block-merge** until the B-class tests are updated (and session cwd tests gain a restore `Drop` guard + canonicalized path compares). That work is test/doc alignment, not product-behavior rollback of the rebrand.

**Suggested follow-ups (out of scope for this gate; no edits made):**

1. Update ACP/minimal profile tests to expect `edit` (and optionally `ffs_grep`/`ffs_glob`) instead of `apply_patch`/`glob`.
2. Align default-config template assertions with actual template (or update template docs for Luna/`swarm_spawn_mode` deliberately).
3. Tag memory test embeddings with `with_embedding_for_model(active_model_id())` or force local backend in those unit tests.
4. Decide product policy for bare `"Access denied…"` messages vs update the failover test.
5. Fix auto-poke test substring to match `TODO_COMPLETION_CONTINUATION_MESSAGE`.
6. Session cwd tests: compare against `session.working_dir` / canonicalized paths; always restore cwd via `Drop` even on panic.
7. Re-check `spawn_detached` in a non-sandbox interactive shell if it remains red in CI.

### Edits made this run

None (read-only gate).

### Authoritative log excerpt

```
running 1085 tests
...
test result: FAILED. 1072 passed; 12 failed; 1 ignored; 0 measured; 0 filtered out; finished in 55.58s
```

---

## DONE

**Recommendation for `next-code-base` only: ok-with-known-failures**  
1072 passed / 12 failed / 1 ignored; **0 class-A (rebrand/telemetry/subscription) failures**; remaining failures are pre-existing product–test drift and cwd/platform isolation issues. Do not merge to main from this job; do not push. No code edits applied.
