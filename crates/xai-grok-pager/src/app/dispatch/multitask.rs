//! `/multitask` dispatch — drain queue into parallel headless workers.

use crate::app::actions::Effect;
use crate::app::app_view::{ActiveView, AppView};
use crate::scrollback::block::RenderBlock;
use crate::slash::commands::multitask::MultitaskArgs;

/// Default concurrency floor matching swarm light fan-out; hard ceiling still
/// comes from server-side `agents.swarm_max_concurrent_agents`.
const MULTITASK_CLIENT_CAP: usize = 32;

/// `/multitask` — spawn parallel background workers, or steer one with `--to`.
pub(super) fn dispatch_multitask(app: &mut AppView, args: MultitaskArgs) -> Vec<Effect> {
    let ActiveView::Agent(id) = app.active_view else {
        return vec![];
    };
    let Some(agent) = app.agents.get_mut(&id) else {
        return vec![];
    };
    let Some(session_id) = agent.session.session_id.clone() else {
        agent
            .scrollback
            .push_block(RenderBlock::system("No active session".to_string()));
        return vec![];
    };

    // Steer / follow-up into a running worker (swarm DM). Prefer agent-team
    // panel selection for day-to-day use; `--to` is the explicit slash path.
    if let Some(target) = args.to_session {
        let message = args.prompts.join("\n").trim().to_string();
        if message.is_empty() {
            agent.scrollback.push_block(RenderBlock::system(
                "Usage: /multitask --to <worker_session_id> <follow-up message>".to_string(),
            ));
            return vec![];
        }
        agent.soft_message_swarm_member(&target, &message);
        agent.prompt.set_text("");
        agent.scrollback.push_block(RenderBlock::system(format!(
            "Multitask: sent follow-up to worker `{target}` (steers the running task)."
        )));
        return agent.effects_message_swarm_member(target, message);
    }

    let mut prompts: Vec<String> = args.prompts;

    // Consume local drip-feed queue (FIFO).
    while let Some(entry) = agent.session.dequeue_prompt() {
        let text = entry.text.trim().to_string();
        if !text.is_empty() {
            prompts.push(text);
        }
    }

    // Consume server shared queue rows → QueueRemove effects.
    let shared = std::mem::take(&mut agent.shared_queue);
    let mut remove_effects = Vec::new();
    for entry in shared {
        let text = entry.text.trim().to_string();
        if !text.is_empty() {
            prompts.push(text);
        }
        remove_effects.push(Effect::QueueRemove {
            session_id: session_id.clone(),
            id: entry.id,
            expected_version: entry.version,
        });
    }

    if prompts.is_empty() {
        agent.scrollback.push_block(RenderBlock::system(
            "Nothing to multitask. Queue prompts first, or pass them inline:\n\
             /multitask fix auth --- add tests\n\
             Steer a running worker: select it in the agent team panel, or\n\
             /multitask --to <worker_session_id> <message>"
                .to_string(),
        ));
        return vec![];
    }

    let total = prompts.len();
    let (spawn_now, overflow): (Vec<_>, Vec<_>) = if prompts.len() > MULTITASK_CLIENT_CAP {
        let mut iter = prompts.into_iter();
        let now: Vec<_> = iter.by_ref().take(MULTITASK_CLIENT_CAP).collect();
        (now, iter.collect())
    } else {
        (prompts, Vec::new())
    };

    // Overflow stays on the local queue for a later /multitask (cap honesty).
    for text in overflow {
        agent.session.enqueue_prompt(text);
    }

    let spawned = spawn_now.len();
    let mut effects = remove_effects;
    for prompt in spawn_now {
        effects.push(Effect::MultitaskSpawn {
            session_id: session_id.clone(),
            prompt,
        });
    }

    agent.prompt.set_text("");
    let overflow_note = if total > spawned {
        format!(
            " ({extra} left queued — over client cap {MULTITASK_CLIENT_CAP}; run /multitask again)",
            extra = total - spawned
        )
    } else {
        String::new()
    };
    agent.scrollback.push_block(RenderBlock::system(format!(
        "Multitask: starting {spawned} background worker(s){overflow_note}. \
         Lead stays free; summaries report back when each finishes. \
         Select a worker in the agent team panel (or /multitask --to <id> …) to steer it."
    )));

    effects
}
