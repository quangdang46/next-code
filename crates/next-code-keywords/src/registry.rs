//! Keyword registry — all supported keywords, aliases, priorities, and workflow mappings.

use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use strum::{Display, EnumIter, EnumString};

/// Workflow kinds that can be triggered by keywords.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Display, EnumIter, EnumString, Serialize, Deserialize,
)]
#[strum(serialize_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub enum WorkflowKind {
    /// ParallelExecution — spawn sub-agents, coordinate, aggregate
    Ultrawork,
    /// GoalTracking — durable goal + token budget across turns
    Ultragoal,
    /// QACycling — implement → test → fix → repeat
    Ultraqa,
    /// ConsensusPlanning — plan → adversarial review → revise → approve
    Ralplan,
    /// RequirementsGathering — ask questions → score ambiguity → threshold
    DeepInterview,
    /// TestDrivenDev — write test → fail → implement → pass
    Tdd,
    /// CodeReview — spawn reviewer → analyze → report
    CodeReview,
    /// SecurityReview — OWASP scan → secrets → report
    SecurityReview,
    /// ExtendedThinking — deep reasoning, single-turn
    Ultrathink,
    /// CodebaseSearch — multi-strategy search → context map
    Deepsearch,
    /// DeepAnalysis — structured analysis → report
    Analyze,
    /// DocLookup — local + web docs → summary
    Wiki,
    /// SlopCleanup — detect + fix AI low-quality code
    AiSlopCleaner,
    /// BestOfN — spawn parallel candidates, pick best diff
    BestOfN,
    /// CancelAll — stop all modes + cancel tasks
    Cancel,
}

/// A single keyword entry in the registry.
#[derive(Debug, Clone)]
pub struct KeywordEntry {
    /// The canonical keyword trigger (e.g. "$ultrawork")
    pub keyword: &'static str,
    /// Single-token aliases (word-boundary exact). Used in Strict and Loose.
    pub aliases: &'static [&'static str],
    /// Multi-word / phrase aliases. **Loose mode only** (substring match).
    pub phrase_aliases: &'static [&'static str],
    /// Priority: 11 (highest) .. 5 (lowest)
    pub priority: u8,
    /// The workflow this keyword activates
    pub workflow: WorkflowKind,
    /// Human-readable description
    pub description: &'static str,
}

/// Build the full keyword registry, sorted by priority (highest first).
pub fn build_registry() -> &'static [&'static KeywordEntry] {
    static REGISTRY: OnceLock<&'static [&'static KeywordEntry]> = OnceLock::new();
    REGISTRY.get_or_init(|| {
        let mut entries: Vec<KeywordEntry> = vec![
            KeywordEntry {
                keyword: "$ralplan",
                // hyperplan/hpp: oh-my-openagent; ultraplan: Claude Code.
                aliases: &["ralplan", "hyperplan", "hpp", "ultraplan"],
                phrase_aliases: &["consensus plan"],
                priority: 11,
                workflow: WorkflowKind::Ralplan,
                description: "Consensus planning — plan → adversarial review → revise → approve",
            },
            KeywordEntry {
                keyword: "$ultrawork",
                // Bare `ultrawork` also matches via detector `$`-strip (omo/Claude parity).
                aliases: &["ulw", "uw"],
                phrase_aliases: &["work on", "dont stop", "must complete", "team mode"],
                // Note: "parallel", "ultra", bare "think" intentionally NOT included (too broad).
                priority: 10,
                workflow: WorkflowKind::Ultrawork,
                description: "Parallel execution — break down work and coordinate subtasks",
            },
            KeywordEntry {
                keyword: "$ultragoal",
                aliases: &["ultragoal"],
                phrase_aliases: &[],
                priority: 10,
                workflow: WorkflowKind::Ultragoal,
                description: "Goal tracking — durable goal + token budget across turns",
            },
            KeywordEntry {
                keyword: "cancelnext",
                aliases: &["stopnext"],
                phrase_aliases: &[],
                priority: 9,
                workflow: WorkflowKind::Cancel,
                description: "Cancel all active modes and stop running tasks",
            },
            KeywordEntry {
                keyword: "$ultraqa",
                aliases: &["ultraqa"],
                phrase_aliases: &["qa cycle"],
                priority: 8,
                workflow: WorkflowKind::Ultraqa,
                description: "QA cycling — implement → test → fix → repeat",
            },
            KeywordEntry {
                keyword: "$deep-interview",
                aliases: &["ouroboros"],
                phrase_aliases: &["interview me", "gather requirements"],
                priority: 8,
                workflow: WorkflowKind::DeepInterview,
                description: "Requirements gathering — ask questions → score ambiguity → threshold",
            },
            KeywordEntry {
                keyword: "$ultrathink",
                // Bare `ultrathink` via `$`-strip; do not alias bare `think` (too noisy).
                aliases: &["ultrathink"],
                phrase_aliases: &["think hard", "think deeply"],
                priority: 7,
                workflow: WorkflowKind::Ultrathink,
                description: "Extended thinking — deep reasoning, single-turn",
            },
            KeywordEntry {
                keyword: "$deepsearch",
                aliases: &["deepsearch"],
                phrase_aliases: &["search the codebase", "find in codebase"],
                priority: 7,
                workflow: WorkflowKind::Deepsearch,
                description: "Codebase search — multi-strategy search → context map",
            },
            KeywordEntry {
                keyword: "$tdd",
                aliases: &["tdd"],
                phrase_aliases: &["test first", "red green"],
                priority: 7,
                workflow: WorkflowKind::Tdd,
                description: "Test-driven development — write test → fail → implement → pass",
            },
            KeywordEntry {
                keyword: "$code-review",
                // ultrareview: Claude Code rainbow keyword.
                aliases: &["ultrareview"],
                phrase_aliases: &["code review", "review code"],
                priority: 6,
                workflow: WorkflowKind::CodeReview,
                description: "Code review — spawn reviewer → analyze → report",
            },
            KeywordEntry {
                keyword: "$security-review",
                aliases: &[],
                phrase_aliases: &["security review", "audit security"],
                priority: 6,
                workflow: WorkflowKind::SecurityReview,
                description: "Security review — OWASP scan → secrets → report",
            },
            KeywordEntry {
                keyword: "$analyze",
                aliases: &["analyze", "deep-analyze"],
                phrase_aliases: &["deep analysis"],
                priority: 6,
                workflow: WorkflowKind::Analyze,
                description: "Deep analysis — structured analysis → report",
            },
            KeywordEntry {
                keyword: "$wiki",
                aliases: &["wiki"],
                phrase_aliases: &["wiki this", "look up docs"],
                priority: 5,
                workflow: WorkflowKind::Wiki,
                description: "Doc lookup — local + web docs → summary",
            },
            KeywordEntry {
                keyword: "ai-slop-cleaner",
                aliases: &[],
                phrase_aliases: &["clean ai slop", "fix ai code"],
                priority: 5,
                workflow: WorkflowKind::AiSlopCleaner,
                description: "AI slop cleanup — detect + fix AI low-quality code",
            },
            KeywordEntry {
                keyword: "$bestofn",
                aliases: &["bestofn", "bon"],
                phrase_aliases: &[],
                priority: 6,
                workflow: WorkflowKind::BestOfN,
                description: "Best-of-N editing — spawn parallel candidates, pick best",
            },
            // oh-my-openagent `team mode` / `team-mode` / `team_mode` / `teammode`
            // — no separate Team workflow; activate Ultrawork orchestration.
            KeywordEntry {
                keyword: "teammode",
                aliases: &["team-mode", "team_mode"],
                phrase_aliases: &[],
                priority: 10,
                workflow: WorkflowKind::Ultrawork,
                description: "Team mode (oh-my-openagent) — orchestration via ultrawork",
            },
        ];

        entries.sort_by_key(|a| std::cmp::Reverse(a.priority));
        let leaked: &'static [KeywordEntry] = Box::leak(entries.into_boxed_slice());
        let refs: &'static [&'static KeywordEntry] =
            Box::leak(leaked.iter().collect::<Vec<_>>().into_boxed_slice());
        refs
    })
}

/// Human-readable list of canonical `$keywords` for help/docs.
pub fn list_canonical_keywords() -> Vec<&'static str> {
    build_registry().iter().map(|e| e.keyword).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_sorted_by_priority() {
        let registry = build_registry();
        for window in registry.windows(2) {
            assert!(
                window[0].priority >= window[1].priority,
                "Registry not sorted: {} ({}) > {} ({})",
                window[0].keyword,
                window[0].priority,
                window[1].keyword,
                window[1].priority,
            );
        }
    }

    #[test]
    fn registry_has_all_workflows() {
        let registry = build_registry();
        let kinds: std::collections::HashSet<WorkflowKind> =
            registry.iter().map(|e| e.workflow).collect();
        assert_eq!(kinds.len(), 15);
    }

    #[test]
    fn omo_and_claude_token_aliases_present() {
        let registry = build_registry();
        let has_alias = |needle: &str| {
            registry.iter().any(|e| {
                e.keyword.eq_ignore_ascii_case(needle)
                    || e.aliases.iter().any(|a| a.eq_ignore_ascii_case(needle))
                    || e.keyword
                        .strip_prefix('$')
                        .is_some_and(|b| b.eq_ignore_ascii_case(needle))
            })
        };
        // oh-my-openagent
        for tok in [
            "ultrawork",
            "ulw",
            "hyperplan",
            "hpp",
            "teammode",
            "team-mode",
            "ultrathink",
        ] {
            assert!(has_alias(tok), "missing omo/claude token {tok}");
        }
        // Claude Code
        for tok in ["ultraplan", "ultrareview"] {
            assert!(has_alias(tok), "missing claude token {tok}");
        }
    }

    #[test]
    fn strict_token_aliases_have_no_spaces() {
        for entry in build_registry() {
            for alias in entry.aliases {
                assert!(
                    !alias.contains(char::is_whitespace),
                    "token alias {:?} for {} must not contain spaces",
                    alias,
                    entry.keyword
                );
            }
        }
    }
}
