//! Façade stub of upstream `xai-grok-shell::extensions` — only the
//! notification/session_search/mcp/task/billing sub-modules the future
//! pager imports directly (per plan doc frequency ranking). Upstream also
//! has auth/git/hooks/jj/pr/rewind/skills/etc. ext-method modules; those
//! are not stubbed (no known import site yet).

pub mod billing;
pub mod mcp;
pub mod notification;
pub mod prompt_meta;
pub mod session_search;
pub mod task;
