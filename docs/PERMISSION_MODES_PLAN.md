# Permission Modes Implementation Plan

> Synthesized from: 9-repo research, dcg-core analysis, 3 rounds QA interview, discussion
> Date: 2026-05-30
> Branch: experiment/dcg-permission-modes

---

## 0. Decisions Made (from QA + Discussion)

| Topic | Decision | Source |
|-------|----------|--------|
| **Scope** | Match Claude Code — full 6-mode pipeline | Round 1 QA |
| **Modes** | All 6: Default, Plan, AcceptEdits, DontAsk, Auto, BypassPermissions | Round 1 QA |
| **Auto/YOLO classifier** | Yes, LLM-based (provider TBD, discuss further to keep dcg clean) | Round 1 QA |
| **Denial tracking** | Yes, match Claude Code: 3 consecutive / 20 total → fallback to prompt | Round 2 QA |
| **Dangerous patterns** | Build in dcg-core, not jcode. Reduce jcode code by leveraging dcg | Round 2 QA |
| **Safe command whitelist** | Yes, full whitelist (~50 commands like codex readOnlyCommandValidation) | Round 2 QA |
| **Mode cycling** | Yes, full Shift+Tab cycle in TUI (6 modes) | Round 2 QA |
| **Where to build missing features** | Build in dcg-core (keep clean, reusable, not jcode-specific) | Round 3 QA |
| **OS sandboxing** | No, app-level only for now | Round 3 QA |
| **YOLO LLM provider** | Needs further discussion — avoid making dcg dirty with jcode-specific logic | Round 3 QA |
| **Protected paths** | Claude Code defaults: ~/.ssh, ~/.aws, ~/.config/gh, .git, .env | Round 3 QA |
| **Pack rules** | Yes, Phase 2 in dcg-core — integrate dcg-cli's 50+ security packs | Round 3 QA |
| **Config format** | TOML (match dcg ecosystem) | Round 3 QA |
| **MCP permissions** | Needs further discussion — present permission mode first | Round 3 QA |
| **Strict mode** | One-way tightening (from oh-my-claudecode) — cannot be weakened by project config | Discussion |
| **Path-aware escalation** | Edit .env/secrets/ → auto escalate even in AcceptEdits mode | Discussion |
| **Consumer-agnostic** | dcg-core exports generic Mode enum, each consumer maps own CLI flags | Discussion |
| **Bypass safety net** | BypassPermissions should have iteration cap + audit log | Discussion |

---

## 1. Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                        jcode (consumer)                         │
│                                                                 │
│  CLI flags: --permission-mode, --dangerously-skip-permissions   │
│  Config: .jcode/config.toml (TOML → maps to dcg types)         │
│  TUI: mode cycling (Shift+Tab), permission dialogs              │
│  YOLO: implements dcg-core trait, injects active LLM provider  │
│                                                                 │
│  ┌─────────────────── dcg_bridge.rs ───────────────────────┐    │
│  │  action_to_tool_call() → ToolCall + Effects             │    │
│  │  classify() → Engine::evaluate() → BridgeDecision       │    │
│  │  set_mode() / current_mode()                            │    │
│  └─────────────────────────────────────────────────────────┘    │
│                          │                                      │
└──────────────────────────┼──────────────────────────────────────┘
                           │ depends on (git URL)
┌──────────────────────────┼──────────────────────────────────────┐
│                    dcg-core (library)                            │
│                          ▼                                      │
│  Engine::evaluate(session, tool_call, mode, effects)            │
│    │                                                            │
│    ├─► Mode::pre_check() → AllowImmediately / Deny / Continue  │
│    ├─► ProtectedPaths check (path-aware escalation)             │
│    ├─► [Phase 2] Pack rule evaluation (50+ security packs)     │
│    ├─► Dangerous command patterns (26-50 regex)                 │
│    ├─► Safe command whitelist (~50 read-only commands)          │
│    ├─► [Phase 2] YOLO classifier trait (consumer implements)   │
│    ├─► Denial escalation (3 consecutive / 20 total)            │
│    └─► Decision: Allow / Prompt{reason,alternatives} / Deny   │
│                                                                 │
│  Already has (v0.6.0-rc.1):                                     │
│    Mode (6 variants + pre_check)                                │
│    Effect (7 variants + is_read_only + is_subset)               │
│    ToolCall (5 variants: Bash/Edit/Write/Read/Network)          │
│    Decision (Allow/Prompt/Deny with reasons + alternatives)     │
│    Session (allow-once codes + per-command deny counter)        │
│    ProtectedPaths (prefix matcher + ~ expansion)                │
│    EngineConfig builder (working_dir + protected_paths)         │
│                                                                 │
│  Phase 2 additions:                                             │
│    DangerousCommandRegistry (26-50 patterns + severity)         │
│    SafeCommandWhitelist (~50 read-only commands)                │
│    DenialEscalation (consecutive + total tracking)              │
│    PathAwareEscalation (.env/secrets → auto Prompt)             │
│    StrictMode (one-way tightening, cannot weaken)               │
│    YoloClassifier trait (interface for consumer LLM)            │
│    PackRuleEngine (from dcg-cli, Aho-Corasick matching)        │
│    PerToolOverrides (TOML: allow/deny/prompt per tool pattern)  │
│    NetworkPolicy (host allowlist/denylist)                      │
└─────────────────────────────────────────────────────────────────┘
```

---

## 2. Phase Breakdown

### Phase 1 — Current State (DONE ✅)

Already implemented in branch `experiment/dcg-permission-modes`:

| Item | Status | File |
|------|--------|------|
| dcg-core git dep (not local path) | ✅ Done | `Cargo.toml` |
| `--permission-mode` CLI flag (6 modes) | ✅ Done | `src/cli/args.rs` |
| `--dangerously-skip-permissions` CLI flag | ✅ Done | `src/cli/args.rs` |
| `JCODE_PERMISSION_MODE` env var support | ✅ Done | `src/cli/startup.rs` |
| dcg_bridge adapter module | ✅ Done | `src/dcg_bridge.rs` |
| BridgeDecision → ActionTier mapping | ✅ Done | `src/safety.rs` |
| Engine + Session + ProtectedPaths integration | ✅ Done | `src/dcg_bridge.rs` |
| Legacy AUTO_ALLOWED compatibility (Default/Auto modes) | ✅ Done | `src/dcg_bridge.rs` |
| Tests for Default, Plan, Bypass modes | ✅ Done | `src/dcg_bridge.rs` |
| 9-repo research document | ✅ Done | `docs/PERMISSION_MODES_RESEARCH.md` |

### Phase 2 — dcg-core Enhancements (in dcg repo)

Build these in `/data/projects/destructive_command_guard/crates/dcg-core/`:

#### 2.1 Dangerous Command Patterns [P0]

**What:** 26-50 regex patterns classifying commands by danger level, inspired by claude-code, oh-my-pi, pi-agent-rust.

**New types:**
```rust
pub struct DangerousPattern {
    pub pattern: Regex,
    pub severity: DangerSeverity,
    pub category: DangerCategory,
    pub reason: String,
    pub alternatives: Vec<String>,
}

pub enum DangerSeverity {
    Low,      // Unusual but not destructive (e.g., curl without pipe)
    Medium,   // Potentially harmful (e.g., git push --force)
    High,     // Destructive (e.g., rm -rf, sudo)
    Critical, // Irreversible/system-level (e.g., dd, mkfs, fork bomb)
}

pub enum DangerCategory {
    RecursiveDelete,
    DiskDestruction,
    ForkBomb,
    RemoteFetchAndExecute,
    PermissionEscalation,
    SystemShutdown,
    CredentialModification,
    ReverseShell,
    NetworkExfiltration,
    ForcePush,
    // extensible
}
```

**Patterns to include (from research):**
- `rm -rf /`, `rm -rf *`, `rm -rf ~`
- `sudo` chains, `chmod 777`, `chown`
- `curl | sh`, `wget | bash`, `pip install | python`
- `:(){:|:&};:` (fork bomb), `bomb()`
- `dd if=`, `mkfs`, `shred`
- `git push --force`, `git clean -fdx`
- `nc -l`, `ncat`, reverse shell patterns
- `aws s3 rm --recursive`, `gcloud compute instances delete`
- `kubectl delete namespace`, `docker system prune -a`
- `DROP TABLE`, `TRUNCATE`, `DELETE FROM` (SQL)

**Integration point:** `Engine::evaluate()` calls `DangerousPatternRegistry::check(&tool_call)` before mode-based evaluation. High/Critical severity overrides mode decision → Prompt or Deny.

#### 2.2 Safe Command Whitelist [P0]

**What:** Explicit allowlist of known-safe read-only commands that auto-approve in all modes (except DontAsk deny-listed ones).

**New types:**
```rust
pub struct SafeCommandEntry {
    pub command: &'static str,        // e.g., "git"
    pub allowed_subcommands: &'static [&'static str],  // e.g., ["status", "log", "diff", "show", "branch"]
    pub safe_flags: &'static [&'static str],           // e.g., ["--oneline", "--color"]
}

pub fn is_known_safe_command(cmd: &str) -> bool;
```

**Commands to whitelist (from codex + claude-code research):**
- `cat`, `head`, `tail`, `less`, `more`
- `ls`, `find` (safe flags only), `stat`, `pwd`, `whoami`, `which`
- `grep`, `rg`, `ag` (safe flags only)
- `git status`, `git log`, `git diff`, `git show`, `git branch` (read-only subcommands)
- `gh issue view`, `gh pr list`, `gh pr status` (read-only)
- `npm run lint`, `npm run check`, `npm run typecheck`
- `cargo check`, `cargo clippy`, `cargo test` (no --release)
- `tsc`, `eslint`, `prettier --check`
- `base64` (safe opts only), `wc`, `tr`, `cut`, `sort`, `uniq`

#### 2.3 Denial Escalation [P1]

**What:** Use existing `Session::deny_counter` to escalate after N denials. Add session-wide tracking.

**New types:**
```rust
pub struct DenialConfig {
    pub max_consecutive: u32,  // default: 3
    pub max_total: u32,        // default: 20
}

// In Session:
pub fn total_denials(&self) -> u32;
pub fn consecutive_denials(&self) -> u32;
pub fn reset_consecutive(&self);  // called on allow
```

**Behavior:** When `consecutive_denials >= max_consecutive` OR `total_denials >= max_total`, override mode decision to `Prompt` (force user interaction). Matches Claude Code behavior exactly.

#### 2.4 Path-Aware Escalation [P1]

**What:** Even in AcceptEdits mode, writing to sensitive paths triggers Prompt.

**New paths to auto-escalate (from oh-my-claudecode + research):**
```
.env, .env.*, .gitconfig, .bashrc, .zshrc, .profile
.mcp.json, .claude.json, .claude/settings.json
.ssh/, .aws/, .gnupg/
**/secrets/**, **/credentials/**
**/.env*, **/.ssh/**
```

**Integration:** Extend `ProtectedPaths` with severity levels. Some paths always Prompt (even bypass — matches Claude Code Step 1g safety checks), others Prompt only in non-bypass modes.

#### 2.5 Strict Mode / One-Way Tightening [P1]

**What:** A master strict flag that can only tighten, never relax. From oh-my-claudecode.

**New types:**
```rust
pub enum StrictnessLevel {
    Default,  // normal operation
    Strict,   // tightens: no bypass, reduced safe whitelist, lower denial limits
}

// In EngineConfig:
pub fn with_strictness(mut self, level: StrictnessLevel) -> Self;
```

**Strict mode effects:**
- Disable BypassPermissions (cannot be activated)
- Reduce max_denials to 5 (from 20)
- Restrict safe command whitelist to minimal set
- Force Prompt for all network operations
- Disable AcceptEdits auto-allow

#### 2.6 Per-Tool User Overrides (TOML) [P2]

**What:** TOML config for allow/deny/prompt per tool pattern.

**Config schema:**
```toml
[permissions]
default_mode = "default"
strict = false

[permissions.protected_paths]
always_prompt = ["~/.ssh", "~/.aws", ".git", ".env"]
always_prompt_recursive = ["**/secrets/**", "**/.ssh/**"]

[permissions.tools]
bash = "prompt"           # Always prompt for bash
edit = "allow"            # Always allow edits (overrides mode)
read = "allow"            # Always allow reads
webfetch = "prompt"       # Always prompt for network
"bash:git *" = "allow"   # Pattern-specific: allow git commands
"bash:rm *" = "deny"     # Pattern-specific: always deny rm

[permissions.denial]
max_consecutive = 3
max_total = 20

[permissions.safe_commands]
# Override safe whitelist
enabled = true
extra = ["just", "make check"]     # Add custom safe commands
deny = ["git branch -D"]           # Remove from safe list
```

**Config resolution chain:**
```
CLI flag > env var > project .jcode/config.toml > user ~/.jcode/config.toml > Engine defaults
```

#### 2.7 Pack Rule Integration [P2]

**What:** Move dcg-cli's 50+ security packs into dcg-core for direct consumption.

**Source:** `/data/projects/destructive_command_guard/crates/dcg-cli/src/packs/`

**Key components to migrate:**
- `PackRegistry` with Aho-Corasick keyword pre-filter + RegexSet batch matching
- `SafePattern` (34 whitelist regex patterns)
- `DestructivePattern` (blacklist with severity, alternatives, Tier-A effects)
- 20+ pack categories (core, database, cloud, kubernetes, containers, system...)
- Allowlist system (project `.dcg/allowlist.toml`, user, system)

#### 2.8 YOLO Classifier Trait [P2]

**What:** Define trait interface in dcg-core, consumer implements with chosen LLM.

**Trait design (keep dcg clean):**
```rust
pub trait YoloClassifier: Send + Sync {
    /// Classify whether a tool call should be auto-approved.
    /// Returns None if classifier cannot determine (fallback to mode default).
    fn classify(
        &self,
        session: &Session,
        tool_call: &ToolCall,
        effects: &[Effect],
        mode: Mode,
    ) -> Option<Result<YoloDecision, YoloError>>;
}

pub enum YoloDecision {
    Allow,
    Deny { reason: String },
}

pub enum YoloError {
    Timeout,
    ClassifierUnavailable,
    TranscriptTooLong,
}
```

**Consumer (jcode) implements:**
- Uses active provider (Claude/Gemini/OpenAI) as subagent
- Respects denial limits — 3 consecutive YOLO denials → fallback to interactive
- Configurable model (fast: haiku, thorough: opus)

**Open question:** Exact LLM provider strategy — reuse active vs dedicated cheap model? Need more discussion to keep dcg clean.

---

### Phase 3 — jcode Integration (in jcode repo)

#### 3.1 TUI Mode Cycling (Shift+Tab) [P0]

**What:** Runtime mode switching via keyboard shortcut.

**Cycle order:** `default → acceptEdits → plan → auto → dontAsk → bypassPermissions → default`

**Implementation:**
- Add keybinding handler in TUI event loop
- Call `dcg_bridge::set_mode(next_mode)` on cycle
- Show mode indicator in status bar with color
- Confirm dialog before entering BypassPermissions (match Claude Code)
- Update mode display in real-time

**UI mockup:**
```
┌──────────────────────────────────────────────────────┐
│ jcode v0.13.0  │ 🔒 Plan Mode  │ Claude Opus 4.8    │
├──────────────────────────────────────────────────────┤
│                                                      │
│  [Shift+Tab to change mode]                          │
│                                                      │
└──────────────────────────────────────────────────────┘
```

#### 3.2 TUI Permission Dialogs [P0]

**What:** Interactive approval/deny/always-allow for Prompt decisions.

**Dialog types:**
- Bash command: show command, Approve/Deny/Always-approve
- File edit: show diff, Approve/Deny
- Network: show URL, Approve/Deny
- Protected path: show path + warning, Approve/Deny

**Match dcg-core Decision:**
- `Decision::Allow` → auto-execute, no dialog
- `Decision::Prompt { reason, allow_once_code, alternatives }` → show dialog with reason + alternatives
- `Decision::Deny { reason, alternatives }` → show denied message with alternatives

#### 3.3 Config Loading [P1]

**What:** Load TOML config and pass to dcg-core.

**Resolution:**
```
CLI --permission-mode > JCODE_PERMISSION_MODE env > .jcode/config.toml > ~/.jcode/config.toml > Mode::Default
```

**Implementation:**
- Parse `config.toml` → dcg-core types
- `EngineConfig::builder()` with protected_paths from config
- Per-tool overrides fed to engine evaluation
- Denial config from TOML → `DenialConfig`

#### 3.4 YOLO Classifier Implementation [P2]

**What:** Implement `dcg_core::YoloClassifier` trait in jcode.

**Implementation:**
- Create `src/yolo_classifier.rs`
- Use active provider's chat completion API as subagent
- Send transcript + action → get allow/deny decision
- Respect denial limits (3 consecutive → stop calling YOLO, show prompt)
- Two-stage: fast check (small model) → thinking check (large model) if uncertain

#### 3.5 Subagent Permission Restriction [P2]

**What:** When spawning subagents (swarm/team), derive restricted permissions from parent.

**Pattern (from opencode + oh-my-openagent):**
```rust
fn derive_subagent_permissions(parent: &PermissionContext, subagent: &SubagentConfig) -> PermissionConfig {
    // Inherit parent denies + external_directory rules
    // Default deny todowrite/task for subagents
    // Force subagents to yolo mode (parent = auth boundary)
}
```

---

## 3. Dependency Map

```
Phase 1 (DONE)
    │
    ▼
Phase 2.1 Dangerous Patterns ─────┐
Phase 2.2 Safe Command Whitelist ─┤ (can run in parallel)
Phase 2.3 Denial Escalation ──────┤
Phase 2.4 Path-Aware Escalation ──┤
Phase 2.5 Strict Mode ────────────┘
    │
    ▼
Phase 3.1 TUI Mode Cycling
Phase 3.2 TUI Permission Dialogs
Phase 3.3 Config Loading
    │
    ▼
Phase 2.6 Per-Tool Overrides (TOML) ──┐
Phase 2.7 Pack Rule Integration ──────┤ (dcg-core, can run in parallel)
Phase 2.8 YOLO Classifier Trait ──────┘
    │
    ▼
Phase 3.4 YOLO Implementation
Phase 3.5 Subagent Permissions
    │
    ▼
Phase 4 — MCP Permissions (future)
```

---

## 4. Open Questions (still need discussion)

| # | Question | Why it matters | Options |
|---|----------|----------------|---------|
| 1 | **YOLO classifier: trait in dcg-core vs all in jcode?** | Affects dcg-core's dependency surface. If dcg-core defines trait, it stays clean (no LLM dep). If jcode builds it, dcg-core stays minimal. | (a) Trait in dcg-core (Recommended) — dcg-core has `YoloClassifier` trait, jcode implements<br>(b) All in jcode — dcg-core only has mode enum, jcode handles YOLO<br>(c) Separate `dcg-yolo` crate — shared between consumers |
| 2 | **YOLO: which LLM provider?** | Affects cost, latency, quality. | (a) Reuse active provider (zero extra cost)<br>(b) Dedicated cheap model (haiku/gpt-4o-mini)<br>(c) Configurable per-user |
| 3 | **MCP permissions: unified or separate?** | MCP tools are dynamic (not known at startup). Different from builtin tools. | (a) Unified pipeline — MCP tools go through same Engine::evaluate<br>(b) Separate system — MCP has own allow/deny config<br>(c) Phase 4 — defer |
| 4 | **Sandboxing future?** | App-level only for now, but codex proves OS-level is the gold standard. | (a) Phase 5 — bubblewrap (Linux) + Seatbelt (macOS)<>(b) Never — rely on app-level + user consent<br>(c) Container-based (Docker/Podman wrapper) |
| 5 | **Multi-agent/swarm permission inheritance** | When jcode spawns subagents, how do permissions propagate? | (a) Inherit parent mode with deny-list restrictions (opencode pattern)<br>(b) Force yolo for subagents (oh-my-pi pattern)<br>(c) Configurable per-spawn |

---

## 5. Success Criteria

- [ ] All 6 permission modes work end-to-end via CLI flag + env var + config
- [ ] Mode cycling works in TUI with Shift+Tab
- [ ] TUI permission dialogs show for Prompt decisions
- [ ] Dangerous commands detected and blocked/prompted in all modes
- [ ] Safe commands auto-approve in Plan/Default/AcceptEdits modes
- [ ] Denial tracking: 3 consecutive / 20 total → fallback to prompt
- [ ] Protected paths always prompt even in AcceptEdits
- [ ] Strict mode available and one-way tightening
- [ ] TOML config overrides work per-tool and per-pattern
- [ ] YOLO classifier operational (even if rule-based initially)
- [ ] dcg-core dependency via git URL (not local path, not crates.io)
- [ ] `cargo check` passes with zero errors
- [ ] Test coverage for all modes + edge cases
