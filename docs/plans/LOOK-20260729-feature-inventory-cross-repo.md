# LOOK â€” Cross-repo TUI/agent UX feature inventory (Face gap analysis)

**Date:** 2026-07-29  
**Base:** `origin/review` @ 9f5961f52 (includes Face PRs through **#135**)  
**Method:** Read each repo under `C:\Users\ADMIN\Documents\Projects\next-code\.tmp\`, list concrete TUI/agent UX features with file/symbol pointers, compare repos, then score Face honestly (**has / partial / missing**).  
**Anti-pattern this doc corrects:** Prior `LOOK-20260729-face-best-of-repos-matrix.md` stamped most clusters â€œDoneâ€ after shallow P0 shipping. Merged PRs #112/#124â€“#135 are **real but incomplete** vs reference depth. Prefer dozens of honest gaps over rubber stamps.

**Repos surveyed:** `claude-code`, `opencode`, `codex`, `crush`, `codebuff`, `grok-build`, `gajae-code`, `oh-my-pi`, `oh-my-openagent`, `pi`, `pi_agent_rust` (+ Face = `xai-grok-pager` + related crates).

---

## How to read Face status

| Tag | Meaning |
|-----|---------|
| **Has** | User-visible parity good enough; polish only |
| **Partial** | Shipped surface exists but thinner than best reference (missing keys, segments, dialogs, keyboard path, etc.) |
| **Missing** | No Face product surface (or only dead/test mention) |

Prior PR numbers in Gap column mean â€œrelated work landedâ€ â€” **not** â€œcluster closed.â€

---

## 1. Per-repo feature inventory

### 1.1 claude-code (Ink / React TUI)

**Roots:** `.tmp/claude-code/src/{components,commands,keybindings,modes,memdir,tasks,bridge}`

**Statusline / footer / snag**
- `BuiltinStatusLine` + `StatusLine` + `StatusNotices` â€” rate-limit countdown, notices (`components/BuiltinStatusLine.tsx`, `StatusLine.tsx`)
- `/statusline` agent-assisted setup (`commands/statusline.tsx`)
- **Footer keyboard snag:** `context: 'Footer'` bindings `footer:up|down|next|previous|openSelected|clearSelection` (`keybindings/defaultBindings.ts` ~L243+)
- Prompt footer chrome: `PromptInputFooter*`, mode indicator, queued cmds, suggestions, sandbox hint (`components/PromptInput/*`)

**Permissions & plan**
- Mode cycle Shift+Tab / Alt+M: `default` â†” `acceptEdits` â†” `plan` â†” `bypassPermissions` (`types/permissions.ts`, PromptInput cycle)
- Per-tool permission cards: Bash / FileEdit / FileWrite / Notebook / PowerShell / Sandbox / ComputerUse / AskUser / Enter|ExitPlanMode (`components/permissions/**`)
- Plan enter/exit permission requests (`EnterPlanModePermissionRequest`, `ExitPlanModePermissionRequest`)
- Auto-mode opt-in dialog (`AutoModeOptInDialog` if present in tree)

**Background / multitask / teams**
- `BackgroundTasksDialog`, `BackgroundTask`, selectors, shell/remote/dream detail dialogs (`components/tasks/*`)
- `TeamsDialog` + `TeamStatus` (`components/teams/*`)
- In-process teammate tasks (`tasks/InProcessTeammateTask`)
- Coordinator agent status (`CoordinatorAgentStatus.tsx`)
- Swarm banner in prompt (`PromptInput/useSwarmBanner.ts`)

**Edit / diff / review**
- `/diff` â†’ DiffDialog / file list (`commands/diff/`, `components/diff/`, `StructuredDiff`)
- File permission dialogs show edit diffs (`FileWriteToolDiff`, `ideDiffConfig`)

**Connect / auth / model**
- `/login` `/logout` `/oauth-refresh`, TrustDialog, ManagedSettingsSecurityDialog
- `/model`, `/effort`, `EffortPanel` (`components/EffortPanel/`)
- Alt+T `chat:thinkingToggle` (`keybindings/defaultBindings.ts`)

**Memory / rules**
- memdir typed memory + `/memory` `/local-memory` `/memory-stores` (`memdir/`, `commands/memory*`, `components/memory/`)
- CLAUDE.md / project rules (product docs + memdir; not all TUI)

**Prompt chrome / vim / keybindings**
- Sticky user prompt while scrolled (`FullscreenLayout.tsx` sticky behavior)
- `/vim` prompt Normal/Insert (`commands/vim`, `src/vim/*` if present)
- Full remappable keybindings schema + `/keybindings` (`keybindings/*`, `commands/keybindings`)
- History search input (`PromptInput/HistorySearchInput.tsx`)
- Notifications in prompt (`PromptInput/Notifications.tsx`)
- Voice indicator (`VoiceIndicator.tsx`)
- `!` bang shell mode (interactive mode; PromptInput inputModes)

**Mentions**
- File suggestions / typeahead (`hooks/fileSuggestions.ts`, `useTypeahead.tsx`)
- Agent/worker badges on permissions (`permissions/WorkerBadge.tsx`)

**Tools / LSP / MCP**
- `LspRecommendation` component; MCP UI under `components/mcp/`
- `/mcp`, `/ide`, IdeStatusIndicator
- Skills: `/skills`, skill-store, skill-learning, skill-search

**Notifications / progress**
- StatusNotices; slave notifications hook (`hooks/useSlaveNotifications.ts`)
- Spinner / Passes / feedback survey chrome

**Session / history / resume**
- `/resume`, `/history`, `/branch`, `/fork`, `/rewind`, `/rename`, `/clear`, `/export`, `/copy`, `/session`, `/attach`/`/detach`, `/teleport`
- `/recap`, `/btw`, `/summary`, `/tag`

**Slash surface (selected unique)**
- Huge catalog: `agents`, `agents-platform`, `artifacts`, `autofix-pr`, `bridge`, `buddy`, `bughunter`, `chrome`, `compact`, `context`/`ctx_viz`, `daemon`, `desktop`/`mobile`, `doctor`, `fast`, `files`, `goal`, `hooks`, `install-github-app`, `job`, `output-style`, `passes`, `peers`, `permissions`, `plan`, `plugin`, `privacy-settings`, `pr_comments`, `remote-*`, `review`, `sandbox-toggle`, `schedule`, `share`, `stats`, `stickers`, `tasks`, `terminalSetup`, `theme`, `thinkback*`, `ultraplan`, `usage`, `vault`, `voice`, `workflows`, â€¦

**Thinking / effort**
- EffortPanel + `/effort` + Alt+T thinking toggle + ultrathink culture

**Terminal / bash**
- Shell permission cards; `!` bang; `/sandbox-toggle`

**Settings / themes**
- `/config`, `/theme`, `/color`, `/keybindings`, Settings components (`components/Settings/`)

**Explicit non-portable for Face:** buddy sprites, stickers, desktop/mobile QR, Slack/GitHub app installers, Claude Desktop bridge product, â€œpoor modeâ€ budget flags.

---

### 1.2 opencode (OpenTUI / Solid)

**Roots:** `.tmp/opencode/packages/{tui,ui,session-ui,opencode,core}`

**Statusline / footer**
- Session footer + subagent footer (`tui/src/routes/session/footer.tsx`, `subagent-footer.tsx`)
- Home/sidebar footers (`feature-plugins/home/footer.tsx`, `sidebar/footer.tsx`)
- Status dialog (`component/dialog-status.tsx`)

**Permissions & plan**
- Session permission route (`routes/session/permission.tsx`)
- Permission protocol/schema (`packages/schema/src/permission*.ts`, `packages/core/src/v1/permission.ts`)
- Context permission (`tui/src/context/permission.tsx`)

**Background / multitask / teams**
- Background package (`packages/opencode/src/background`)
- Subagent dialog (`routes/session/dialog-subagent.tsx`)
- Worktree create/list/unavailable dialogs (`dialog-workspace-*`)

**Edit / diff / review**
- Workspace file changes dialog (`dialog-workspace-file-changes.tsx`)
- Snapshot / patch packages in core

**Connect / auth / model**
- Provider dialog (`dialog-provider.tsx`) â€” **best-in-class connect UX reference**
- Model / variant dialogs (`dialog-model.tsx`, `dialog-variant.tsx`)
- MCP dialog + auth flows (`dialog-mcp.tsx`); `use-connected.tsx`
- Console org dialog

**Memory / rules**
- Skills dialog (`dialog-skill.tsx`); skill package in opencode core
- Agents MD / project config (core)

**Prompt chrome / keybindings**
- **Which-key overlay** (`feature-plugins/system/which-key.tsx`) â€” pending-key preview, dock/overlay layouts, group tabs
- Keybind config (`config/keybind.ts`)
- Prompt stash multi-entry dialog (`component/dialog-stash.tsx` + `prompt/stash`)
- Toast system (`ui/toast.tsx`)
- Notifications feature plugin (`feature-plugins/system/notifications.ts`)

**Session / history**
- Session list / rename / delete-failed / move (`dialog-session-*`, `dialog-move-session.tsx`)
- Timeline + fork-from-timeline (`dialog-timeline.tsx`, `dialog-fork-from-timeline.tsx`)
- Message dialog; tag dialog (`dialog-tag.tsx`)
- Theme list (`dialog-theme-list.tsx`); retry-action dialog

**Tools / LSP / MCP**
- LSP package in opencode core; MCP in protocol + dialogs
- Plugin TUI API (`packages/plugin/src/tui.ts`)

**Slash / commands**
- Command package (`packages/opencode/src/command`); TUI command palette patterns

**Unique strengths:** which-key, provider connect, stash dialog, workspace/session dialogs, toast, timeline fork, tag, retry-action.

---

### 1.3 codex (Rust ratatui)

**Roots:** `.tmp/codex/codex-rs/tui/src/{bottom_pane,status,notifications,chatwidget,â€¦}`

**Statusline / footer**
- Rich `StatusLineItem` enum: model, model+reasoning, reasoning, cwd, project-root, git-branch, PR#, branch-changes, run-state, permissions, approval-mode, context-used/remaining, thread-title, â€¦ (`bottom_pane/status_line_setup.rs`)
- `status_line_style`, `status_surface_preview`, `effort_status_line`, `unified_exec_footer`
- Footer widget (`bottom_pane/footer.rs`)

**Permissions & plan**
- Approval overlay (`bottom_pane/approval_overlay.rs`, `approval_events.rs`)
- Pending thread approvals; permission compat; experimental features view
- Sandbox crates (OS policy â€” not Face UI clone)

**Background / multitask**
- Multi-agents (`multi_agents.rs`); cloud-tasks crates adjacent
- Collaboration modes (`collaboration_modes.rs`)

**Edit / diff**
- `diff_model.rs`, `diff_render.rs`, `get_git_diff.rs`, branch_summary

**Connect / auth / model**
- Model catalog / migration; OSS selection; local ChatGPT auth; login crate
- MCP server elicitation UI (`bottom_pane/mcp_server_elicitation.rs`)

**Memory**
- Memories settings view (`bottom_pane/memories_settings_view.rs`); `memories` crate

**Prompt chrome / keybindings**
- Chat composer + history + effort ignition (`chat_composer*.rs`, `effort_ignition*.rs`)
- File search popup; skill popup; slash commands popup
- Paste burst; pending input preview; custom prompt view
- Keymap + keymap_setup; external editor
- Mentions codec (`mention_codec.rs`)

**Notifications**
- `notifications/{bel,osc9,mod}.rs` â€” terminal bell / OSC9

**Session**
- Resume picker; session archive commands; thread transcript; onboarding

**Slash**
- `slash_command.rs` â€” large enum (model, compact, diff, â€¦)

**Unique strengths:** statusline item richness, effort ignition, approval overlay, MCP elicitation, OSC9/BEL notifications, hooks browser view, skills toggle view.

**Skip for Face:** pets, seatbelt/landlock as UI.

---

### 1.4 crush (Go Charm/Bubble Tea)

**Roots:** `.tmp/crush/internal/{ui,commands,lsp,permission,question,agent}`

**Statusline / footer / chrome**
- Compact chat agent UI (`ui/chat/agent.go`, `assistant.go`, `user.go`)
- Model dialog + reasoning dialog (`ui/dialog/models*.go`, `reasoning.go`)
- Notifications dialog (`ui/dialog/notifications.go`)

**Permissions**
- Permissions dialog + tests (`ui/dialog/permissions.go`)

**Edit / diff**
- **Split + unified DiffView** with syntax, line numbers, scroll offsets (`ui/diffview/**`, golden tests)
- Unified diff in chat (`ui/chat/unified_diff.go`)

**Connect / auth / model**
- API key input, OAuth (incl. Copilot/Hyper), MCP auth (`ui/dialog/api_key_input.go`, `oauth*.go`, `mcp_auth.go`)
- Sessions dialog

**Tools / LSP (standout)**
- In-chat LSP: diagnostics, definition, references, symbols, call hierarchy, rename, replace-symbol, lsp_restart (`ui/chat/{diagnostics,definition,references,symbols,call_hierarchy,rename,replace_symbol,lsp_restart}.go`)
- MCP + docker MCP tool renders (`ui/chat/mcp.go`, `docker_mcp.go`)

**Prompt / questions**
- Rich question suite: confirm, editor, form, freetext, multi, single, yesno (`ui/dialog/question_*.go`)
- Completions (`ui/completions`)
- Filepicker; inline editor

**Commands**
- Custom commands from markdown + MCP prompts + skills catalog (`internal/commands/commands.go`)

**Unique strengths:** LSP-in-transcript UX, diffview quality, OAuth/MCP auth dialogs, question form variety.

---

### 1.5 codebuff (Ink CLI)

**Roots:** `.tmp/codebuff/cli/src/{components,commands,blocks,tools}`

**Statusline**
- `status-bar.tsx`, bottom/top banners, usage/subscription banners

**Permissions / ask-user**
- Full ask-user accordion multi-question (`components/ask-user/**`)
- Agent mode toggle; build-mode buttons

**Background / BoN / agents**
- **Implementor cards / masonry grid** (`blocks/agent-block-grid.tsx`, `implementor-row.tsx`, `grid-layout.tsx`)
- Agent checklist; message-with-agents; agent branch items
- Review screen (`review-screen.tsx`) â‰  BoN but related

**Edit / diff**
- Tool renders: apply-patch, str-replace, write-file, diff-viewer (`components/tools/*`)

**Connect**
- Login modal; ChatGPT connect banner; freebuff model selector

**Prompt**
- Multiline input; suggestion menu; selected chips; pending attachments; input-mode banner
- Thinking block; shimmer; progress bar; elapsed timer

**Mentions**
- `@` suggestion patterns in suggestion-menu / chips (Codebuff `@Agent` culture)

**Slash / commands**
- Small set: help, init, image, usage, publish, copy-conversation, process-diagnostics, ads, bash-command routing (`commands/*`)

**Unique strengths:** BoN implementor card UX, ask-user accordion, suggestion chips, review screen.

**Skip:** freebuff landing / referral / ads branding.

---

### 1.6 grok-build (Rust â€” Face ancestor)

**Roots:** `.tmp/grok-build/crates` (+ Face lives in next-code as vendored `xai-grok-*`)

**Strengths already largely in Face:** pager/scrollback, ACP, slash registry, settings modal, dashboard, voice, mermaid, announcements, plugins marketplace patterns, shell.

**Caution:** soft YOLO defaults / brand chrome â€” do not reintroduce.

---

### 1.7 gajae-code (pi fork + Rust natives)

**Roots:** `.tmp/gajae-code/packages/{coding-agent,tui,agent}` + `crates/pi-*`

**Notable UX-adjacent**
- Full **vim engine** for prompt (`coding-agent/src/vim/{buffer,commands,engine,parser,render,types}.ts`)
- Hashline package; plan-mode; memories; LSP; DAP; coordinator; deep-interview; research-plan; workflow; SSH; STT
- TUI primitives (`packages/tui/src/components`: editor, select-list, settings-list, markdown, â€¦)
- pi-shell / hashline crates

**Unique:** prompt vim engine depth; coordinator/deep-interview flows.

---

### 1.8 oh-my-pi

**Roots:** `.tmp/oh-my-pi/packages/{coding-agent,hashline,tui,mnemopi,swarm-extension,collab-web}`

**Notable**
- Hashline package (apply/patcher/diff-preview/recovery-session-chain)
- Autolearn, hindsight, mnemopi memory backends
- Plan-mode; collab QR (`commands/collab-qrcode.ts`); SSH; swarm-extension
- Slash/commands catalog (acp, agents, mcp, session-pin, stats-dashboard, todo, usage-report, worktree, â€¦)
- TUI components (editor, tab-bar, scroll-view, â€¦)

**Unique:** hashline edit model + recovery; mnemopi; collab QR; swarm extension.

---

### 1.9 oh-my-openagent

**Roots:** `.tmp/oh-my-openagent/packages/{hashline-core,lsp-*,team-core,tmux-core,claude-code-compat-core,delegate-core,â€¦}`

**Notable (mostly library, not full TUI)**
- hashline-core edit ops
- lsp-core / lsp-daemon / lsp-tools-mcp â€” diagnostics freshness, tools surface
- team-core + tmux-core (pane close, rebalance, stale session sweep)
- claude-code-compat modes; rules-engine; skills-loader; boulder-state; comment-checker
- pi-goal; prompts-core; model-core

**Unique for Face portability:** LSP daemon patterns; tmux team pane ops; hashline-core; Claude-compat mode mapping â€” not pixel TUI.

---

### 1.10 pi (upstream)

**Roots:** `.tmp/pi/packages/{coding-agent,tui,agent,ai}`

**Notable**
- Modes; extensions; TUI editor/select-list/markdown
- Lighter than gajae/oh-my-pi forks

---

### 1.11 pi_agent_rust

**Roots:** `.tmp/pi_agent_rust/src/interactive`

**Notable**
- Slash: model, thinking, scoped-models, fork, compact, reload, template, share, mcp, login/logout, bash (`commands.rs`)
- Conversation **tree UI** (`tree.rs`, `tree_ui.rs`)
- File refs; model selector UI; keybindings; tool render; share; ext_session
- Themes dir at repo root

**Unique:** conversation tree navigation UX.

---

### 1.12 Face on `origin/review` (baseline)

**Roots:** `crates/xai-grok-pager/src/{slash,views,actions,app,scrollback,notifications}` + `xai-grok-shared/src/status_line.rs` + related `next-code-tui-*`

**Already strong / recently merged (still may be Partial vs best)**
- Permission cards + plan gate + AcceptEdits cycle â€” #112/#125
- Diff hunks / hashline in scrollback â€” #124
- Session `/diff` DiffReview â€” #134
- Statusline segments config (`StatusLineSegment`: mode/model/context/cwd/git) + `/statusline` â€” #133 adjacent
- Footer **mouse** pills (tasks / `@agent`) â€” #133
- Multitask / teams / BoN cards / mermaid sidebar / memory edit / connect â€” #126â€“#132
- Alt+T effort toggle â€” #135
- Sticky scrollback headers â€” `scrollback/sticky.rs`, `scrollback_pane.rs`
- Ctrl+R history search â€” `views/history_search`, `agent_view/prompt.rs`
- `!` bash mode â€” `ActionId::BashMode`, prompt.rs
- Remappable `~/.next-code/keybindings.json` + `/keybindings` â€” `actions/user_bindings.rs`
- ShortcutsHelp modal (Ctrl+./Ctrl+X) â€” **not** which-key pending overlay
- `/vim-mode` â€” **scrollback** vim letters only (`slash/commands/vim_mode.rs`)
- Notifications package (progress/title/tmux/hooks) â€” not OpenCode toast parity
- Large slash set under `slash/commands/*` (see Â§3)

---

## 2. Cross-repo comparison (cluster â†’ strengths â†’ Face winner)

| Feature cluster | Best references | Runner-up | Winner for Face portability | Face now |
|-----------------|-----------------|-----------|-----------------------------|----------|
| Statusline segments | **Codex** `StatusLineItem` | Claude BuiltinStatusLine | Codex vocabulary + Claude density | **Partial** (5 segments) |
| Footer keyboard snag | **Claude** Footer context | â€” | Claude | **Partial** (mouse only) |
| Which-key / chord preview | **OpenCode** which-key.tsx | Face ShortcutsHelp | OpenCode overlay | **Missing** |
| Permissions cards | **Claude** per-tool cards | Codex approval overlay | Claude cards (already direction) | **Partial** |
| Plan enter/exit gate | **Claude** ExitPlanMode | Codex experimental | Claude | **Has** (gate) / polish |
| Bg tasks hub | **Claude** BackgroundTasksDialog | Crush readiness | Claude | **Partial** |
| Multitask / teams | Claude teams + Cursor multitask | oh-my tmux team | Hybrid (shipped) | **Partial** |
| BoN cards | **Codebuff** implementor grid | â€” | Codebuff | **Partial** (#128) |
| Session `/diff` | Claude DiffDialog | Crush DiffView | Claude dialog + Crush split quality | **Partial** (#134; no split/unified tabs) |
| Hashline edits | **oh-my-pi** / hashline-core | gajae | oh-my-pi model | **Partial** (#124) |
| Connect / provider | **OpenCode** dialog-provider | Crush OAuth | OpenCode | **Partial** |
| Prompt stash multi | **OpenCode** dialog-stash | Claude StashNotice | OpenCode | **Partial** (single interrupt stash) |
| Sticky prompt | Claude FullscreenLayout | Face sticky.rs | Claude semantics (Face has sticky headers) | **Has**/polish |
| Prompt vim N/I | **gajae/Claude** vim engine | pi | gajae engine ideas | **Missing** (scrollback-only `/vim-mode`) |
| Keybindings file | Claude schema | Face user_bindings | Claude-shaped (Face has) | **Has** |
| Mentions `@file/@agent` | Codebuff chips + Claude typeahead | Codex mention_codec | Hybrid | **Partial** |
| LSP-in-chat | **Crush** diagnostics/symbols/â€¦ | oh-my lsp-daemon | Crush UX + oh-my daemon | **Missing** |
| MCP auth UX | Crush mcp_auth/OAuth | OpenCode dialog-mcp | Crush/OpenCode | **Partial** |
| Toasts / OSC notify | OpenCode toast + Codex OSC9 | Face notifications | Both | **Partial** |
| Session tags / timeline fork | **OpenCode** tag + timeline | â€” | OpenCode | **Missing**/thin |
| Workspace file changes | **OpenCode** dialog | â€” | OpenCode | **Missing** |
| Memory browser | Claude memdir types | Codex memories settings | Claude | **Partial** (`/memory`) |
| Effort ignition chrome | **Codex** effort_ignition | Claude EffortPanel | Codex+Claude | **Partial** (`/effort`+Alt+T; no panel/ignition) |
| Output styles | **Claude** `/output-style` | â€” | Claude | **Missing** |
| Doctor / health | **Claude** `/doctor` | â€” | Claude | **Missing** |
| Conversation tree | **pi_agent_rust** tree_ui | â€” | pi_agent_rust | **Missing** |
| Bang bash | Claude `!` | Face BashMode | Claude (Face has) | **Has** |
| History search | Claude HistorySearch | Face Ctrl+R | Either | **Has** |
| External $EDITOR prompt | Claude Ctrl+G / Codex external_editor | Face SuspendForEditor | Wire for **prompt** buffer | **Partial** |
| Skills store UX | Claude skill-store | Codex skills_toggle | Claude | **Partial** |
| Hooks browser | Codex hooks_browser_view | Claude hooks | Codex view | **Partial** |
| Question forms | Crush question_* + Claude AskUser | Codebuff accordion | Already Face direction | **Partial** |
| Collab / tmux teams | oh-my-openagent tmux-core | oh-my-pi collab QR | Optional later | **Missing** |
| Settings / themes | All have | OpenCode theme-list | Face settings modal OK | **Has**/polish |

---

## 3. Face gap list (by cluster)

Priority: **P0** = high daily UX leverage, portable, fits Face chrome; **P1** = clear win next; **P2** = valuable; **P3** = niche / heavy / optional.

### Statusline / footer / snag
| Gap | Face | Pri | Port from |
|-----|------|-----|-----------|
| Footer **keyboard** snag (â†/â†’/â†‘/â†“/Enter on pills: tasks, agents, diff, loop) | Partial (#133 mouse) | **P0** | Claude `Footer` bindings |
| Statusline items: reasoning, run-state, context-remaining, thread-title, approval-mode, git PR/branch-changes | Partial (5 segs) | **P0** | Codex `StatusLineItem` |
| Rate-limit / notices strip | Partial | P1 | Claude `StatusNotices` / `formatCountdown` |
| Shell-script / agent-authored statusline | Missing | P3 | Claude `/statusline` agent setup |

### Permissions & plan
| Gap | Face | Pri | Port from |
|-----|------|-----|-----------|
| Deeper per-tool card parity (sandbox preview, notebook, computer-use) | Partial | P1 | Claude `components/permissions/*` |
| Permission explanation / debug toggles in-card | Partial | P2 | Claude keybindings `permission:toggle*` |
| Plan artifact side-by-side while approving | Partial | P1 | Claude ExitPlanMode UX |

### Background / multitask / teams
| Gap | Face | Pri | Port from |
|-----|------|-----|-----------|
| Tasks hub detail dialogs (shell progress, remote session, dream) | Partial | P1 | Claude `components/tasks/*` |
| Teams dialog keyboard parity | Partial | P1 | Claude `TeamsDialog` |
| Tmux/team pane ops | Missing | P3 | oh-my-openagent `tmux-core` |
| BoN masonry density / Â± progress polish | Partial | P2 | Codebuff blocks |

### Edit / diff / hashline / review
| Gap | Face | Pri | Port from |
|-----|------|-----|-----------|
| DiffReview **split vs unified** tabs + syntax | Partial (#134) | **P0** | Crush `ui/diffview` |
| Hashline mismatch recovery UX | Partial | P1 | oh-my-pi hashline recovery |
| Inline edit permission diff fidelity | Partial | P1 | Claude File*ToolDiff |

### Connect / auth / model
| Gap | Face | Pri | Port from |
|-----|------|-----|-----------|
| Provider dialog empty/error/retry polish | Partial | P1 | OpenCode `dialog-provider` |
| MCP OAuth / elicitation modal | Partial | P1 | Crush + Codex elicitation |
| Model variant dialog | Partial | P2 | OpenCode `dialog-variant` |

### Memory / rules
| Gap | Face | Pri | Port from |
|-----|------|-----|-----------|
| Typed memdir browser (user/feedback/project/reference) depth | Partial | P1 | Claude memdir |
| Memory delete via AskUserQuestion | Partial (double-press) | P2 | Claude |
| Memories settings page | Partial | P2 | Codex `memories_settings_view` |
| Autolearn / hindsight / mnemopi | Missing | P3 | oh-my-pi |

### Prompt chrome / vim / keybindings
| Gap | Face | Pri | Port from |
|-----|------|-----|-----------|
| **Which-key** pending-key overlay | Missing | **P0** | OpenCode `which-key.tsx` |
| Prompt **vim Normal/Insert** | Missing | **P0** | gajae `vim/*` + Claude `/vim` |
| Multi-entry **prompt stash** dialog | Partial | **P0** | OpenCode `dialog-stash` |
| External editor for **current prompt** (not only keybindings file) | Partial | P1 | Codex `external_editor` / Claude Ctrl+G |
| Queued-commands / suggestions footer polish | Partial | P2 | Claude PromptInputFooter* |

### Mentions
| Gap | Face | Pri | Port from |
|-----|------|-----|-----------|
| `@Agent` mention insert + chip | Partial (footer pills) | P1 | Codebuff |
| `@file` fuzzy mention codec | Partial (`@` picker) | P1 | Codex `mention_codec` + Claude typeahead |

### Tools / LSP / MCP UX
| Gap | Face | Pri | Port from |
|-----|------|-----|-----------|
| LSP diagnostics/definition/refs/symbols in transcript | Missing | **P0** | Crush `ui/chat/*` + oh-my lsp |
| LspRecommendation nudge | Missing | P2 | Claude |
| MCP prompt-as-slash commands | Partial | P2 | Crush `LoadMCPPrompts` |

### Notifications / toasts / progress
| Gap | Face | Pri | Port from |
|-----|------|-----|-----------|
| Toast stack (success/warn/error, dismiss) | Partial | P1 | OpenCode `toast.tsx` |
| OSC9 / BEL attention when unfocused | Partial | P2 | Codex `notifications/*` |

### Session / history / resume
| Gap | Face | Pri | Port from |
|-----|------|-----|-----------|
| Session **tags** | Missing | P1 | OpenCode `dialog-tag` |
| Timeline scrub + fork-from-timeline | Partial (`/timeline`) | P1 | OpenCode timeline dialogs |
| Workspace file-changes overview | Missing | P1 | OpenCode |
| Move session / workspace list | Missing | P2 | OpenCode |
| Conversation **tree** nav | Missing | P2 | pi_agent_rust |
| Share link polish | Has | P3 | â€” |

### Slash surface area
| Gap | Face | Pri | Port from |
|-----|------|-----|-----------|
| `/doctor` health | Missing | P1 | Claude |
| `/output-style` | Missing | P2 | Claude |
| `/sandbox` user toggle | Missing | P2 | Claude |
| `/files` browser | Missing | P2 | Claude |
| Skill-store / skill-learning | Partial | P2 | Claude |
| Many Claude remote/desktop cmds | Missing | P3 | skip |

Face already has a large set: `always_approve`, `announcements`, `auto`, `btw`, `cd`, `compact`, `connect`, `context`, `copy`, `dashboard`, `diff`, `effort`, `experimental`, `fork`, `goal`, `history`, `keybindings`, `memory`, `model`, `multitask`, `plan`, `plugin`, `queue`, `recap`, `resume`, `rewind`, `share`, `statusline`, `tasks`, `theme`, `timeline`, `usage`, `vim_mode`, `voice`, â€¦ (`slash/commands/`).

### Thinking / effort UX
| Gap | Face | Pri | Port from |
|-----|------|-----|-----------|
| Effort **ignition** / panel chrome | Partial | P1 | Codex effort_ignition + Claude EffortPanel |
| Thinking fold vs effort toggle confusion in help | Partial | P2 | docs/shortcuts copy |

### Terminal / bash
| Gap | Face | Pri | Port from |
|-----|------|-----|-----------|
| Unified exec footer while bash runs | Partial | P2 | Codex `unified_exec_footer` |
| Paste-burst / pending input preview | Partial | P2 | Codex |

### Settings / themes
| Gap | Face | Pri | Port from |
|-----|------|-----|-----------|
| Theme list dialog with live preview polish | Partial | P2 | OpenCode `dialog-theme-list` |
| Hooks browser view | Partial | P2 | Codex |

---

## 4. Recommended implement order (next sessions)

Do **not** claim â€œP0s doneâ€ until the P0 row below is honestly **Has** or consciously deferred with reason.

1. **P0 â€” Footer keyboard snag** (Claude) â€” complete #133  
2. **P0 â€” Which-key overlay** (OpenCode) â€” distinct from ShortcutsHelp  
3. **P0 â€” Prompt stash multi-entry dialog** (OpenCode)  
4. **P0 â€” Statusline segment expansion** (Codex reasoning / run-state / context-remaining)  
5. **P0 â€” DiffReview split|unified** (Crush)  
6. **P0 â€” Prompt vim Normal/Insert** (gajae/Claude) â€” larger; own PR  
7. **P0 â€” LSP diagnostics-in-chat** (Crush) â€” needs daemon story; may span PRs  
8. P1 â€” `/doctor`, workspace file-changes, session tags, toast stack, external-editor-for-prompt, EffortPanel, memdir depth, `@Agent` chips, MCP elicitation, â€¦

**Skip re-doing unless inventory proves incomplete:** solid cores of #124â€“#135 (diff hunks, AcceptEdits chrome, BoN cards, multitask spawn, Alt+T binding, `/diff` entrypoint, memory black-screen fix). Prefer **deepening** those surfaces (e.g. split diff on top of #134) over greenfield duplicates.

---

## 5. Explicit non-copies

- Claude buddy/stickers/desktop/mobile/Slack installers  
- Codex pets; OS sandbox policy as Face UI  
- Codebuff ads / freebuff marketing  
- Full OpenCode TS-in-process hooks rewrite  
- Pixel-clone Ink/Solid â€” Face stays ratatui pager chrome  
- Soft-default YOLO from grok-build  

---

## 6. Research notes

- Local trees under `.tmp/` are the source of truth for this inventory (not npm latest).  
- Face ActionId surface: `crates/xai-grok-pager/src/actions/mod.rs`.  
- Face slash builtins: `crates/xai-grok-pager/src/slash/commands/`.  
- Prior shallow matrix: `docs/plans/LOOK-20260729-face-best-of-repos-matrix.md` â€” treat as superseded for â€œdoneâ€ claims.  
- Earlier Claude gap look (2026-07-24): `LOOK-20260724-claude-code-ux-gaps-for-face.md` â€” many items shipped as Partial; this doc re-scores vs all repos.

