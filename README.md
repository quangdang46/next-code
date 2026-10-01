# ⚠️ ARCHIVED — this repository is no longer maintained

> **This project has been archived.** The repository is in read-only mode: no further
> development, bug fixes, or pull requests will be accepted here.

## Why this was archived

`next-code` is a Rust coding agent (TUI + multi-model LLM client + swarm coordination,
~1.2M lines of Rust across 120 crates). The Rust implementation is genuinely excellent
in terms of **runtime performance** — the TUI is blazing fast, memory footprint is small,
and the agent loop is highly responsive.

However, the **development and testing loop** proved to be the bottleneck:

- **Slow compile-test cycles** — a workspace of this size means even `cargo check` on a
  single crate change can take minutes, and a full `cargo test` run is very expensive.
- **Long dev → commit → push turnaround** — iterating on a feature requires waiting on
  builds and tests that are disproportionately slow relative to the size of the change.
- **High iteration cost** — the time cost of verifying a change often exceeded the time
  cost of writing it, which made day-to-day development painful.

In short: Rust delivers outstanding runtime performance, but the dev/test iteration
speed did not keep up with the project's growth.

## Where the work goes next

The valuable logic from `next-code` is being migrated to
**[ultrabuilders/ultraworkers](https://github.com/ultrabuilders/ultraworkers)** as soon
as possible. That project is a plugin-based coding agent monorepo where the good ideas
and proven components from `next-code` will live on with a faster development loop.

If you are looking for the active continuation of this work, please follow
[ultrabuilders/ultraworkers](https://github.com/ultrabuilders/ultraworkers).

## Historical README (for reference only)

<details>
<summary>Original project description (pre-archive)</summary>

### next-code

> Possibly the greatest coding agent ever built — blazing-fast TUI, multi-model,
> swarm coordination, 30+ tools.

- **Language:** Rust (edition 2024)
- **Version:** 0.32.0
- **Workspace:** 120 crates, ~1.2M lines of Rust
- **Architecture:** modular monorepo — agent runtime, TUI (ratatui), multi-provider
  LLM client (OpenAI, Anthropic, Gemini, Bedrock, Copilot, Grok, OpenRouter, …),
  swarm coordination, plugin platform, hooks, memory, compaction, and more.
- **CI:** GitHub Actions (`ci.yml`, `release.yml`, `freebsd-cross.yml`,
  `freebsd-smoke.yml`, `windows-smoke.yml`, `require-issue.yml`)

See `docs/` (100+ design and architecture documents) and `AGENTS.md` for the full
development workflow that was in use.

</details>

---

*Archived October 2026. Thank you to everyone who contributed.*
