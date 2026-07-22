//! Face ACP handlers for Grok-style **bundle plugins** and next-code **lifecycle
//! hooks** under `~/.next-code`.
//!
//! Product model (replaces the old QuickJS/TS `next-code plugin *` stack):
//! - Discover plugin dirs from `~/.next-code/plugins/`, project
//!   `.next-code/plugins/`, and `~/.next-code/installed-plugins/` (git/local
//!   installs). Claude compat: `~/.claude/plugins/` (read-only list).
//! - Wire Face Extensions Plugins tab via `x.ai/plugins/list|action`.
//! - Wire Face Extensions Hooks tab via `x.ai/hooks/list|action` to
//!   `next-code-hooks` (`hooks.toml` layers) — not OpenCode JS plugin hooks.
//! - Marketplace stays brand-hidden.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};
use serde_json::json;
use xai_hooks_plugins_types::{
    ActionOutcome, HookEvent, HookHandlerType, HookInfo, HookStatus, HooksAction,
    HooksListResponse, McpStatus, OutcomeStatus, PluginInfo, PluginOrigin, PluginScope,
    PluginsAction, PluginsListResponse,
};

const STATE_FILE: &str = "plugins-state.json";
const INSTALLED_DIR: &str = "installed-plugins";
const USER_PLUGINS_DIR: &str = "plugins";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct PluginsState {
    #[serde(default)]
    disabled: Vec<String>,
    #[serde(default)]
    enabled: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InstallRegistryFile {
    #[serde(default)]
    repos: HashMap<String, InstalledRepoRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InstalledRepoRecord {
    path: PathBuf,
    #[serde(default)]
    plugin_names: Vec<String>,
    #[serde(default)]
    git_url: Option<String>,
    #[serde(default)]
    is_local: bool,
}

#[derive(Debug, Clone)]
struct Discovered {
    name: String,
    id: String,
    root: PathBuf,
    scope: PluginScope,
    origin: PluginOrigin,
    version: Option<String>,
    description: Option<String>,
    skill_names: Vec<String>,
    agent_names: Vec<String>,
    has_hooks: bool,
    mcp_server_count: usize,
}

fn next_code_home() -> Option<PathBuf> {
    crate::storage::next_code_dir().ok()
}

fn state_path() -> Option<PathBuf> {
    next_code_home().map(|h| h.join(STATE_FILE))
}

fn load_state() -> PluginsState {
    let Some(path) = state_path() else {
        return PluginsState::default();
    };
    fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_state(state: &PluginsState) -> Result<(), String> {
    let path = state_path().ok_or_else(|| "no next-code home".to_string())?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let body = serde_json::to_string_pretty(state).map_err(|e| e.to_string())?;
    fs::write(&path, body).map_err(|e| e.to_string())
}

fn install_registry_path() -> Option<PathBuf> {
    next_code_home().map(|h| h.join(INSTALLED_DIR).join("registry.json"))
}

fn load_install_registry() -> InstallRegistryFile {
    let Some(path) = install_registry_path() else {
        return InstallRegistryFile {
            repos: HashMap::new(),
        };
    };
    fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| InstallRegistryFile {
            repos: HashMap::new(),
        })
}

fn save_install_registry(reg: &InstallRegistryFile) -> Result<(), String> {
    let path = install_registry_path().ok_or_else(|| "no next-code home".to_string())?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let body = serde_json::to_string_pretty(reg).map_err(|e| e.to_string())?;
    fs::write(&path, body).map_err(|e| e.to_string())
}

fn path_hex8(path: &Path) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    path.to_string_lossy().hash(&mut hasher);
    format!("{:08x}", (hasher.finish() as u32))
}

fn plugin_id(scope: PluginScope, root: &Path, name: &str) -> String {
    let label = match scope {
        PluginScope::Cli => "cli",
        PluginScope::Project => "project",
        PluginScope::User => "user",
        PluginScope::Config => "config",
    };
    format!("{label}/{}/{name}", path_hex8(root))
}

fn is_valid_plugin_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        && !name.starts_with('-')
        && !name.ends_with('-')
}

fn name_from_dirname(dirname: &str) -> String {
    let lower = dirname.to_ascii_lowercase().replace('_', "-");
    if is_valid_plugin_name(&lower) {
        lower
    } else {
        "plugin".into()
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ManifestJson {
    name: Option<String>,
    version: Option<String>,
    description: Option<String>,
}

fn load_manifest(root: &Path) -> Option<ManifestJson> {
    for rel in ["plugin.json", ".grok-plugin/plugin.json", ".claude-plugin/plugin.json"] {
        let path = root.join(rel);
        if let Ok(text) = fs::read_to_string(&path)
            && let Ok(m) = serde_json::from_str::<ManifestJson>(&text)
        {
            return Some(m);
        }
    }
    None
}

fn list_skill_names(root: &Path) -> Vec<String> {
    let skills = root.join("skills");
    let Ok(entries) = fs::read_dir(&skills) else {
        return vec![];
    };
    let mut names = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() && path.join("SKILL.md").is_file() {
            if let Some(n) = path.file_name().and_then(|s| s.to_str()) {
                names.push(n.to_string());
            }
        }
    }
    names.sort();
    names
}

fn list_agent_names(root: &Path) -> Vec<String> {
    let agents = root.join("agents");
    let Ok(entries) = fs::read_dir(&agents) else {
        return vec![];
    };
    let mut names = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("md") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                names.push(stem.to_string());
            }
        }
    }
    names.sort();
    names
}

fn looks_like_plugin_dir(root: &Path) -> bool {
    if !root.is_dir() {
        return false;
    }
    root.join("plugin.json").is_file()
        || root.join(".grok-plugin").join("plugin.json").is_file()
        || root.join(".claude-plugin").join("plugin.json").is_file()
        || root.join("skills").is_dir()
        || root.join("agents").is_dir()
        || root.join("hooks").join("hooks.json").is_file()
        || root.join(".mcp.json").is_file()
}

fn discover_in_parent(
    parent: &Path,
    scope: PluginScope,
    origin: PluginOrigin,
    out: &mut Vec<Discovered>,
    seen: &mut HashSet<String>,
) {
    let Ok(entries) = fs::read_dir(parent) else {
        return;
    };
    for entry in entries.flatten() {
        let root = entry.path();
        if !looks_like_plugin_dir(&root) {
            continue;
        }
        let dirname = root
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("plugin");
        let manifest = load_manifest(&root);
        let name = manifest
            .as_ref()
            .and_then(|m| m.name.clone())
            .filter(|n| is_valid_plugin_name(n))
            .unwrap_or_else(|| name_from_dirname(dirname));
        let id = plugin_id(scope, &root, &name);
        if !seen.insert(id.clone()) {
            continue;
        }
        let skill_names = list_skill_names(&root);
        let agent_names = list_agent_names(&root);
        let has_hooks = root.join("hooks").join("hooks.json").is_file();
        let mcp_server_count = if root.join(".mcp.json").is_file() {
            1
        } else {
            0
        };
        out.push(Discovered {
            name,
            id,
            root,
            scope,
            origin: origin.clone(),
            version: manifest.as_ref().and_then(|m| m.version.clone()),
            description: manifest.as_ref().and_then(|m| m.description.clone()),
            skill_names,
            agent_names,
            has_hooks,
            mcp_server_count,
        });
    }
}

fn cwd_from_params(params: &serde_json::Value) -> Option<PathBuf> {
    params
        .get("cwd")
        .and_then(|v| v.as_str())
        .map(PathBuf::from)
        .or_else(|| std::env::current_dir().ok())
}

/// Discover all bundle plugins for the Face list.
pub(crate) fn discover_plugins(cwd: Option<&Path>) -> Vec<Discovered> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();

    if let Some(home) = next_code_home() {
        discover_in_parent(
            &home.join(USER_PLUGINS_DIR),
            PluginScope::User,
            PluginOrigin::UserGrok,
            &mut out,
            &mut seen,
        );
        // Installed registry roots (single-plugin checkout or multi-plugin repo).
        let reg = load_install_registry();
        for (_key, repo) in reg.repos {
            let origin = PluginOrigin::MarketplaceInstall {
                source_name: None,
                git_url: repo.git_url.clone(),
            };
            if looks_like_plugin_dir(&repo.path) {
                let dirname = repo
                    .path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("plugin");
                let manifest = load_manifest(&repo.path);
                let name = manifest
                    .as_ref()
                    .and_then(|m| m.name.clone())
                    .filter(|n| is_valid_plugin_name(n))
                    .unwrap_or_else(|| name_from_dirname(dirname));
                let id = plugin_id(PluginScope::User, &repo.path, &name);
                if seen.insert(id.clone()) {
                    out.push(Discovered {
                        name,
                        id,
                        root: repo.path.clone(),
                        scope: PluginScope::User,
                        origin,
                        version: manifest.as_ref().and_then(|m| m.version.clone()),
                        description: manifest.as_ref().and_then(|m| m.description.clone()),
                        skill_names: list_skill_names(&repo.path),
                        agent_names: list_agent_names(&repo.path),
                        has_hooks: repo.path.join("hooks").join("hooks.json").is_file(),
                        mcp_server_count: usize::from(repo.path.join(".mcp.json").is_file()),
                    });
                }
            } else if repo.path.is_dir() {
                discover_in_parent(
                    &repo.path,
                    PluginScope::User,
                    origin,
                    &mut out,
                    &mut seen,
                );
            }
        }
    }

    if let Some(cwd) = cwd {
        let project = cwd.join(".next-code").join(USER_PLUGINS_DIR);
        discover_in_parent(
            &project,
            PluginScope::Project,
            PluginOrigin::ProjectGrok,
            &mut out,
            &mut seen,
        );
    }

    // Claude compat (list-only; enable/disable still tracked in next-code state).
    if let Some(home) = dirs::home_dir() {
        let claude = home.join(".claude").join("plugins");
        discover_in_parent(
            &claude,
            PluginScope::User,
            PluginOrigin::UserClaude,
            &mut out,
            &mut seen,
        );
    }

    out.sort_by(|a, b| a.name.cmp(&b.name).then(a.id.cmp(&b.id)));
    out
}

fn is_enabled(state: &PluginsState, plugin: &Discovered) -> bool {
    if state
        .disabled
        .iter()
        .any(|d| d == &plugin.id || d == &plugin.name)
    {
        return false;
    }
    if state
        .enabled
        .iter()
        .any(|e| e == &plugin.id || e == &plugin.name)
    {
        return true;
    }
    // Default: user/cli/config enabled; project plugins default enabled for next-code.
    true
}

fn to_info(plugin: &Discovered, enabled: bool) -> PluginInfo {
    let hook_status = if !plugin.has_hooks {
        HookStatus::None
    } else if enabled {
        HookStatus::Active
    } else {
        HookStatus::Blocked
    };
    let mcp_status = if plugin.mcp_server_count == 0 {
        McpStatus::None
    } else if enabled {
        McpStatus::Active
    } else {
        McpStatus::Blocked
    };
    PluginInfo {
        name: plugin.name.clone(),
        id: plugin.id.clone(),
        root: plugin.root.display().to_string(),
        scope: plugin.scope,
        trusted: true,
        enabled,
        version: plugin.version.clone(),
        description: plugin.description.clone(),
        skill_count: plugin.skill_names.len(),
        skill_names: plugin.skill_names.clone(),
        agent_count: plugin.agent_names.len(),
        agent_names: plugin.agent_names.clone(),
        hook_status,
        hook_count: if plugin.has_hooks { 1 } else { 0 },
        mcp_server_count: plugin.mcp_server_count,
        mcp_status,
        marketplace_source: None,
        origin: Some(plugin.origin.clone()),
        conflict: None,
    }
}

fn wrap_result(value: impl Serialize) -> serde_json::Value {
    json!({ "result": value })
}

fn outcome(status: OutcomeStatus, message: impl Into<String>, reload: bool) -> serde_json::Value {
    wrap_result(ActionOutcome {
        status,
        message: message.into(),
        requires_reload: reload,
        requires_restart: false,
    })
}

/// `x.ai/plugins/list`
pub fn plugins_list_payload(cwd: Option<&Path>) -> serde_json::Value {
    let state = load_state();
    let plugins: Vec<PluginInfo> = discover_plugins(cwd)
        .iter()
        .map(|p| to_info(p, is_enabled(&state, p)))
        .collect();
    wrap_result(PluginsListResponse { plugins })
}

/// `x.ai/hooks/list` — next-code `hooks.toml` layers (user / project / env).
pub fn hooks_list_payload() -> serde_json::Value {
    let (entries, load_errors) = next_code_hooks::list_hook_layer_entries();
    let hooks: Vec<HookInfo> = entries.iter().map(hook_layer_to_info).collect();
    wrap_result(HooksListResponse {
        hooks,
        project_trusted: true,
        load_errors,
    })
}

fn hook_layer_to_info(entry: &next_code_hooks::HookLayerEntry) -> HookInfo {
    let name = next_code_hooks::face_hook_name(entry.scope, &entry.event, entry.index);
    let source_dir = entry
        .config_path
        .parent()
        .unwrap_or(entry.config_path.as_path())
        .display()
        .to_string();
    let (handler_type, command, url, timeout_secs, matcher, disabled) =
        match &entry.handler {
            next_code_hooks::HookHandlerConfig::Command(cmd) => (
                HookHandlerType::Command,
                Some(cmd.command.clone()),
                None,
                cmd.timeout_secs,
                matcher_display(cmd.matcher.as_ref()),
                !cmd.enabled,
            ),
            next_code_hooks::HookHandlerConfig::Http(http) => (
                HookHandlerType::Http,
                None,
                Some(http.url.clone()),
                http.timeout_secs,
                matcher_display(http.matcher.as_ref()),
                !http.enabled,
            ),
            next_code_hooks::HookHandlerConfig::Agent(agent) => (
                HookHandlerType::Command,
                Some(format!("agent:{}", agent.agent_id)),
                None,
                Some(agent.timeout_secs),
                matcher_display(agent.matcher.as_ref()),
                !agent.enabled,
            ),
            next_code_hooks::HookHandlerConfig::Plugin(plugin) => (
                HookHandlerType::Command,
                Some(format!("plugin:{}", plugin.path)),
                None,
                Some(plugin.timeout_secs),
                matcher_display(plugin.matcher.as_ref()),
                !plugin.enabled,
            ),
        };
    let timeout_ms = timeout_secs.unwrap_or(30).saturating_mul(1000);
    HookInfo {
        name,
        event: map_hook_event(&entry.event),
        handler_type,
        matcher,
        command,
        url,
        timeout_ms,
        source_dir,
        disabled,
    }
}

fn matcher_display(m: Option<&next_code_hooks::HookMatcher>) -> Option<String> {
    m.map(|m| match m {
        next_code_hooks::HookMatcher::Wildcard => "*".to_string(),
        next_code_hooks::HookMatcher::Exact(v) => v.clone(),
        next_code_hooks::HookMatcher::Multi(parts) => parts.join("|"),
        next_code_hooks::HookMatcher::Regex(pat) => format!("/{pat}/"),
    })
}

fn map_hook_event(event: &str) -> HookEvent {
    use next_code_hooks::HookEvent as Nc;
    match Nc::parse(event) {
        Some(Nc::PreToolUse) => HookEvent::PreToolUse,
        Some(Nc::PostToolUse) => HookEvent::PostToolUse,
        Some(Nc::PostToolUseFailure) => HookEvent::PostToolUseFailure,
        Some(Nc::ToolError) => HookEvent::ToolError,
        Some(Nc::UserPromptSubmit) | Some(Nc::UserPromptExpansion) => HookEvent::UserPromptSubmit,
        Some(Nc::SessionStart) => HookEvent::SessionStart,
        Some(Nc::SessionEnd) => HookEvent::SessionEnd,
        Some(Nc::SessionIdle) => HookEvent::SessionIdle,
        Some(Nc::PermissionDenied) => HookEvent::PermissionDenied,
        Some(Nc::PermissionRequest) | Some(Nc::PermissionAsked) => HookEvent::PermissionRequest,
        Some(Nc::SubagentStart) | Some(Nc::AgentStart) => HookEvent::SubagentStart,
        Some(Nc::SubagentStop) | Some(Nc::AgentEnd) => HookEvent::SubagentStop,
        Some(Nc::TurnEnd) => HookEvent::TurnEnd,
        Some(Nc::Stop) => HookEvent::Stop,
        Some(Nc::PreCompact) | Some(Nc::AutoCompactionControl) => HookEvent::PreCompact,
        Some(Nc::PostCompact) => HookEvent::PostCompact,
        Some(Nc::SessionUpdated)
        | Some(Nc::SessionDiff)
        | Some(Nc::SessionError)
        | Some(Nc::PermissionReplied)
        | Some(Nc::TaskCreated)
        | Some(Nc::TaskCompleted)
        | Some(Nc::Setup)
        | Some(Nc::FileChanged)
        | Some(Nc::Custom(_))
        | None => HookEvent::Notification,
    }
}

fn find_plugin<'a>(plugins: &'a [Discovered], plugin_id: &str) -> Option<&'a Discovered> {
    plugins
        .iter()
        .find(|p| p.id == plugin_id || p.name == plugin_id)
}

fn set_disabled(plugin_id: &str, disable: bool) -> Result<(), String> {
    let mut state = load_state();
    let key = plugin_id.to_string();
    state.disabled.retain(|d| d != &key);
    state.enabled.retain(|e| e != &key);
    // Also strip by trailing name segment.
    let name = plugin_id.rsplit('/').next().unwrap_or(plugin_id);
    state.disabled.retain(|d| d != name);
    state.enabled.retain(|e| e != name);
    if disable {
        state.disabled.push(key);
    } else {
        state.enabled.push(key);
    }
    save_state(&state)
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    fs::create_dir_all(dst).map_err(|e| e.to_string())?;
    for entry in fs::read_dir(src).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if from.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else {
            fs::copy(&from, &to).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn install_local(path: &Path) -> Result<String, String> {
    if !looks_like_plugin_dir(path) {
        return Err(format!(
            "not a plugin directory (need plugin.json, skills/, agents/, or hooks/): {}",
            path.display()
        ));
    }
    let home = next_code_home().ok_or_else(|| "no next-code home".to_string())?;
    let plugins_dir = home.join(USER_PLUGINS_DIR);
    fs::create_dir_all(&plugins_dir).map_err(|e| e.to_string())?;
    let dirname = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("plugin");
    let manifest = load_manifest(path);
    let name = manifest
        .as_ref()
        .and_then(|m| m.name.clone())
        .filter(|n| is_valid_plugin_name(n))
        .unwrap_or_else(|| name_from_dirname(dirname));
    let dest = plugins_dir.join(&name);
    if dest.exists() {
        fs::remove_dir_all(&dest).map_err(|e| e.to_string())?;
    }
    copy_dir_recursive(path, &dest)?;
    let mut reg = load_install_registry();
    reg.repos.insert(
        name.clone(),
        InstalledRepoRecord {
            path: dest,
            plugin_names: vec![name.clone()],
            git_url: None,
            is_local: true,
        },
    );
    save_install_registry(&reg)?;
    Ok(format!("Installed plugin '{name}' into ~/.next-code/plugins"))
}

fn install_git(url: &str) -> Result<String, String> {
    let home = next_code_home().ok_or_else(|| "no next-code home".to_string())?;
    let install_root = home.join(INSTALLED_DIR);
    fs::create_dir_all(&install_root).map_err(|e| e.to_string())?;
    let repo_key = url
        .trim_end_matches('/')
        .trim_end_matches(".git")
        .rsplit('/')
        .next()
        .unwrap_or("plugin")
        .to_ascii_lowercase()
        .replace('_', "-");
    let dest = install_root.join(&repo_key);
    if dest.exists() {
        fs::remove_dir_all(&dest).map_err(|e| e.to_string())?;
    }
    let status = Command::new("git")
        .args(["clone", "--depth", "1", url])
        .arg(&dest)
        .status()
        .map_err(|e| format!("git clone failed to start: {e}"))?;
    if !status.success() {
        return Err(format!("git clone failed for {url}"));
    }
    let mut names = Vec::new();
    if looks_like_plugin_dir(&dest) {
        names.push(repo_key.clone());
    } else {
        if let Ok(entries) = fs::read_dir(&dest) {
            for entry in entries.flatten() {
                let p = entry.path();
                if looks_like_plugin_dir(&p) {
                    if let Some(n) = p.file_name().and_then(|s| s.to_str()) {
                        names.push(name_from_dirname(n));
                    }
                }
            }
        }
    }
    if names.is_empty() {
        let _ = fs::remove_dir_all(&dest);
        return Err("cloned repo contains no recognizable plugin dirs".into());
    }
    let mut reg = load_install_registry();
    reg.repos.insert(
        repo_key.clone(),
        InstalledRepoRecord {
            path: dest,
            plugin_names: names.clone(),
            git_url: Some(url.to_string()),
            is_local: false,
        },
    );
    save_install_registry(&reg)?;
    Ok(format!(
        "Installed {} plugin(s) from git into ~/.next-code/installed-plugins/{repo_key}",
        names.len()
    ))
}

fn parse_install_source(source: &str, cwd: &Path) -> Result<InstallKind, String> {
    let source = source.trim();
    if source.is_empty() {
        return Err("empty install source".into());
    }
    if source.starts_with("http://")
        || source.starts_with("https://")
        || source.starts_with("git@")
        || source.ends_with(".git")
    {
        return Ok(InstallKind::Git(source.to_string()));
    }
    // GitHub shorthand user/repo
    if !source.starts_with('.')
        && !source.starts_with('~')
        && !source.starts_with('/')
        && !source.contains('\\')
        && source.matches('/').count() == 1
    {
        return Ok(InstallKind::Git(format!("https://github.com/{source}")));
    }
    let path = if let Some(rest) = source.strip_prefix("~/") {
        dirs::home_dir()
            .ok_or_else(|| "no home directory".to_string())?
            .join(rest)
    } else if Path::new(source).is_absolute() {
        PathBuf::from(source)
    } else {
        cwd.join(source)
    };
    Ok(InstallKind::Local(path))
}

enum InstallKind {
    Git(String),
    Local(PathBuf),
}

fn uninstall_plugin(plugin_id: &str, confirmed: bool) -> Result<String, String> {
    let cwd = std::env::current_dir().ok();
    let plugins = discover_plugins(cwd.as_deref());
    let Some(plugin) = find_plugin(&plugins, plugin_id) else {
        return Err(format!("plugin not found: {plugin_id}"));
    };
    // Never delete Claude-compat paths from disk via uninstall.
    if matches!(plugin.origin, PluginOrigin::UserClaude | PluginOrigin::ProjectClaude) {
        return Err("cannot uninstall Claude-compat plugins from next-code; disable instead".into());
    }
    if matches!(plugin.scope, PluginScope::Project) && !confirmed {
        // Still allow with confirm for project copies.
    }
    let root = plugin.root.clone();
    let name = plugin.name.clone();
    if root.exists() {
        fs::remove_dir_all(&root).map_err(|e| e.to_string())?;
    }
    let mut reg = load_install_registry();
    reg.repos.retain(|_, repo| {
        !repo.plugin_names.iter().any(|n| n == &name) && repo.path != root
    });
    let _ = save_install_registry(&reg);
    let mut state = load_state();
    state.disabled.retain(|d| d != &plugin.id && d != &name);
    state.enabled.retain(|e| e != &plugin.id && e != &name);
    let _ = save_state(&state);
    Ok(format!("Uninstalled plugin '{name}'"))
}

/// `x.ai/plugins/action`
pub fn plugins_action_payload(params: &serde_json::Value) -> serde_json::Value {
    let cwd = cwd_from_params(params);
    let action: PluginsAction = match serde_json::from_value(
        params
            .get("action")
            .cloned()
            .unwrap_or(serde_json::Value::Null),
    ) {
        Ok(a) => a,
        Err(e) => {
            return outcome(
                OutcomeStatus::ValidationError,
                format!("invalid plugins action: {e}"),
                false,
            );
        }
    };

    match action {
        PluginsAction::Reload => outcome(OutcomeStatus::Success, "Plugins reloaded", true),
        PluginsAction::Enable { plugin_id } => match set_disabled(&plugin_id, false) {
            Ok(()) => outcome(
                OutcomeStatus::Success,
                format!("Enabled {plugin_id}"),
                true,
            ),
            Err(e) => outcome(OutcomeStatus::InternalError, e, false),
        },
        PluginsAction::Disable { plugin_id } => match set_disabled(&plugin_id, true) {
            Ok(()) => outcome(
                OutcomeStatus::Success,
                format!("Disabled {plugin_id}"),
                true,
            ),
            Err(e) => outcome(OutcomeStatus::InternalError, e, false),
        },
        PluginsAction::Install { source } | PluginsAction::Add { path: source } => {
            let cwd_path = cwd.unwrap_or_else(|| PathBuf::from("."));
            match parse_install_source(&source, &cwd_path) {
                Ok(InstallKind::Local(path)) => match install_local(&path) {
                    Ok(msg) => outcome(OutcomeStatus::Success, msg, true),
                    Err(e) => outcome(OutcomeStatus::ValidationError, e, false),
                },
                Ok(InstallKind::Git(url)) => match install_git(&url) {
                    Ok(msg) => outcome(OutcomeStatus::Success, msg, true),
                    Err(e) => outcome(OutcomeStatus::InternalError, e, false),
                },
                Err(e) => outcome(OutcomeStatus::ValidationError, e, false),
            }
        }
        PluginsAction::Uninstall {
            plugin_id,
            confirmed,
        } => match uninstall_plugin(&plugin_id, confirmed) {
            Ok(msg) => outcome(OutcomeStatus::Success, msg, true),
            Err(e) if e.contains("not found") => outcome(OutcomeStatus::NotFound, e, false),
            Err(e) => outcome(OutcomeStatus::InternalError, e, false),
        },
        PluginsAction::Remove { path } => {
            // Treat as uninstall by path match.
            let plugins = discover_plugins(cwd.as_deref());
            if let Some(p) = plugins.iter().find(|p| p.root == Path::new(&path) || p.id == path || p.name == path)
            {
                match uninstall_plugin(&p.id, true) {
                    Ok(msg) => outcome(OutcomeStatus::Success, msg, true),
                    Err(e) => outcome(OutcomeStatus::InternalError, e, false),
                }
            } else {
                outcome(
                    OutcomeStatus::NotFound,
                    format!("no plugin at {path}"),
                    false,
                )
            }
        }
        PluginsAction::Update { plugin_id } => {
            // Best-effort: re-clone git installs when we have a URL.
            let reg = load_install_registry();
            let targets: Vec<_> = if let Some(id) = plugin_id.as_deref() {
                let name = id.rsplit('/').next().unwrap_or(id);
                reg.repos
                    .into_iter()
                    .filter(|(_, r)| r.plugin_names.iter().any(|n| n == name) || r.path.ends_with(name))
                    .collect()
            } else {
                reg.repos.into_iter().collect()
            };
            let mut updated = 0usize;
            let mut errors = Vec::new();
            for (key, repo) in targets {
                if let Some(url) = &repo.git_url {
                    match install_git(url) {
                        Ok(_) => updated += 1,
                        Err(e) => errors.push(format!("{key}: {e}")),
                    }
                }
            }
            if !errors.is_empty() && updated == 0 {
                outcome(
                    OutcomeStatus::InternalError,
                    errors.join("; "),
                    false,
                )
            } else if updated == 0 {
                outcome(
                    OutcomeStatus::Success,
                    "No git-backed plugins to update",
                    false,
                )
            } else {
                outcome(
                    OutcomeStatus::Success,
                    format!("Updated {updated} plugin repo(s)"),
                    true,
                )
            }
        }
    }
}

/// `x.ai/hooks/action` — reload + enable/disable against next-code `hooks.toml`.
pub fn hooks_action_payload(params: &serde_json::Value) -> serde_json::Value {
    let action: HooksAction = match serde_json::from_value(
        params
            .get("action")
            .cloned()
            .unwrap_or(serde_json::Value::Null),
    ) {
        Ok(a) => a,
        Err(e) => {
            return outcome(
                OutcomeStatus::ValidationError,
                format!("invalid hooks action: {e}"),
                false,
            );
        }
    };

    match action {
        HooksAction::Reload => outcome(
            OutcomeStatus::Success,
            "Hooks reloaded from ~/.next-code/hooks.toml layers",
            true,
        ),
        HooksAction::Enable { hook_name } => {
            match next_code_hooks::set_hook_enabled_by_face_name(&hook_name, true) {
                Ok(()) => outcome(
                    OutcomeStatus::Success,
                    format!("Enabled {hook_name}"),
                    true,
                ),
                Err(e) if e.contains("not found") || e.contains("no hooks") => {
                    outcome(OutcomeStatus::NotFound, e, false)
                }
                Err(e) if e.contains("invalid hook name") || e.contains("out of range") => {
                    outcome(OutcomeStatus::ValidationError, e, false)
                }
                Err(e) => outcome(OutcomeStatus::InternalError, e, false),
            }
        }
        HooksAction::Disable { hook_name } => {
            match next_code_hooks::set_hook_enabled_by_face_name(&hook_name, false) {
                Ok(()) => outcome(
                    OutcomeStatus::Success,
                    format!("Disabled {hook_name}"),
                    true,
                ),
                Err(e) if e.contains("not found") || e.contains("no hooks") => {
                    outcome(OutcomeStatus::NotFound, e, false)
                }
                Err(e) if e.contains("invalid hook name") || e.contains("out of range") => {
                    outcome(OutcomeStatus::ValidationError, e, false)
                }
                Err(e) => outcome(OutcomeStatus::InternalError, e, false),
            }
        }
        HooksAction::ToggleSource {
            hook_names,
            disable,
        } => {
            let mut ok = 0usize;
            let mut last_err: Option<String> = None;
            for name in &hook_names {
                match next_code_hooks::set_hook_enabled_by_face_name(name, !disable) {
                    Ok(()) => ok += 1,
                    Err(e) => last_err = Some(e),
                }
            }
            if ok == 0 {
                outcome(
                    OutcomeStatus::InternalError,
                    last_err.unwrap_or_else(|| "no hooks toggled".into()),
                    false,
                )
            } else {
                let verb = if disable { "Disabled" } else { "Enabled" };
                outcome(
                    OutcomeStatus::Success,
                    format!("{verb} {ok} hook(s)"),
                    true,
                )
            }
        }
        HooksAction::Trust | HooksAction::Untrust => outcome(
            OutcomeStatus::Unsupported,
            "Project hook trust is not required for next-code hooks.toml (always loaded)",
            false,
        ),
        HooksAction::Add { path } => {
            let source = std::path::PathBuf::from(path.trim());
            if path.trim().is_empty() {
                return outcome(
                    OutcomeStatus::ValidationError,
                    "Add requires a path to a hooks.toml file",
                    false,
                );
            }
            match next_code_hooks::merge_hooks_toml_into_user(&source) {
                Ok(n) => outcome(
                    OutcomeStatus::Success,
                    format!(
                        "Merged {n} handler(s) from {} into ~/.next-code/hooks.toml",
                        source.display()
                    ),
                    true,
                ),
                Err(e) if e.contains("not found") => {
                    outcome(OutcomeStatus::NotFound, e, false)
                }
                Err(e) => outcome(OutcomeStatus::ValidationError, e, false),
            }
        }
        HooksAction::Remove { path } => {
            let key = path.trim();
            if key.is_empty() {
                return outcome(
                    OutcomeStatus::ValidationError,
                    "Remove requires a hook id (user/Event[0])",
                    false,
                );
            }
            // Face passes the selected hook's ACP name (user/PreToolUse[0]).
            match next_code_hooks::remove_hook_by_face_name(key) {
                Ok(()) => outcome(
                    OutcomeStatus::Success,
                    format!("Removed {key}"),
                    true,
                ),
                Err(e) if e.contains("not found") || e.contains("no hooks") => {
                    outcome(OutcomeStatus::NotFound, e, false)
                }
                Err(e) if e.contains("invalid hook name") || e.contains("out of range") => {
                    outcome(OutcomeStatus::ValidationError, e, false)
                }
                Err(e) => outcome(OutcomeStatus::InternalError, e, false),
            }
        }
    }
}

/// Skill dirs contributed by enabled next-code bundle plugins (for SkillRegistry).
pub fn enabled_plugin_skill_dirs(cwd: Option<&Path>) -> Vec<PathBuf> {
    let state = load_state();
    discover_plugins(cwd)
        .into_iter()
        .filter(|p| is_enabled(&state, p))
        .filter_map(|p| {
            let skills = p.root.join("skills");
            skills.is_dir().then_some(skills)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use tempfile::TempDir;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn write_plugin(root: &Path, name: &str, skill: &str) {
        let dir = root.join(name);
        fs::create_dir_all(dir.join("skills").join(skill)).unwrap();
        fs::write(
            dir.join("plugin.json"),
            format!(r#"{{"name":"{name}","version":"0.1.0","description":"test"}}"#),
        )
        .unwrap();
        fs::write(
            dir.join("skills").join(skill).join("SKILL.md"),
            format!("---\nname: {skill}\ndescription: d\n---\n# {skill}\n"),
        )
        .unwrap();
    }

    #[test]
    fn list_discovers_user_plugin_under_next_code_home() {
        let _guard = ENV_LOCK.lock().unwrap();
        let tmp = TempDir::new().unwrap();
        let prev = std::env::var_os("NEXT_CODE_HOME");
        crate::env::set_var("NEXT_CODE_HOME", tmp.path());
        write_plugin(&tmp.path().join("plugins"), "demo-plugin", "demo-skill");

        let payload = plugins_list_payload(None);
        let plugins = payload["result"]["plugins"].as_array().unwrap();
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0]["name"], "demo-plugin");
        assert_eq!(plugins[0]["skillCount"], 1);
        assert_eq!(plugins[0]["enabled"], true);

        match prev {
            Some(v) => crate::env::set_var("NEXT_CODE_HOME", v),
            None => crate::env::remove_var("NEXT_CODE_HOME"),
        }
    }

    #[test]
    fn enable_disable_persists() {
        let _guard = ENV_LOCK.lock().unwrap();
        let tmp = TempDir::new().unwrap();
        let prev = std::env::var_os("NEXT_CODE_HOME");
        crate::env::set_var("NEXT_CODE_HOME", tmp.path());
        write_plugin(&tmp.path().join("plugins"), "tog-plugin", "s");

        let list = plugins_list_payload(None);
        let id = list["result"]["plugins"][0]["id"].as_str().unwrap().to_string();

        let disable = plugins_action_payload(&json!({
            "sessionId": "s",
            "action": { "type": "disable", "plugin_id": id }
        }));
        assert_eq!(disable["result"]["status"], "success", "{disable}");

        let list2 = plugins_list_payload(None);
        assert_eq!(list2["result"]["plugins"][0]["enabled"], false);

        let enable = plugins_action_payload(&json!({
            "sessionId": "s",
            "action": { "type": "enable", "plugin_id": id }
        }));
        assert_eq!(enable["result"]["status"], "success", "{enable}");
        let list3 = plugins_list_payload(None);
        assert_eq!(list3["result"]["plugins"][0]["enabled"], true);

        match prev {
            Some(v) => crate::env::set_var("NEXT_CODE_HOME", v),
            None => crate::env::remove_var("NEXT_CODE_HOME"),
        }
    }

    #[test]
    fn install_local_copies_into_plugins() {
        let _guard = ENV_LOCK.lock().unwrap();
        let tmp = TempDir::new().unwrap();
        let prev = std::env::var_os("NEXT_CODE_HOME");
        crate::env::set_var("NEXT_CODE_HOME", tmp.path());

        write_plugin(tmp.path(), "src-plugin", "hello");
        let src_dir = tmp.path().join("src-plugin");
        assert!(src_dir.is_dir());

        let out = plugins_action_payload(&json!({
            "sessionId": "s",
            "cwd": tmp.path().to_string_lossy(),
            "action": { "type": "install", "source": src_dir.to_string_lossy() }
        }));
        assert_eq!(out["result"]["status"], "success", "{out}");
        assert!(
            tmp.path().join("plugins").join("src-plugin").is_dir(),
            "expected install under NEXT_CODE_HOME/plugins"
        );

        match prev {
            Some(v) => crate::env::set_var("NEXT_CODE_HOME", v),
            None => crate::env::remove_var("NEXT_CODE_HOME"),
        }
    }

    fn write_user_hooks_toml(home: &Path) {
        fs::write(
            home.join("hooks.toml"),
            r#"
[settings]
timeout_secs = 30

[[events.PreToolUse]]
type = "command"
enabled = true
command = "echo pre"

[[events.SessionStart]]
type = "command"
enabled = true
command = "echo start"
"#,
        )
        .unwrap();
    }

    #[test]
    fn hooks_list_reads_next_code_hooks_toml() {
        let _guard = ENV_LOCK.lock().unwrap();
        let tmp = TempDir::new().unwrap();
        let prev = std::env::var_os("NEXT_CODE_HOME");
        let prev_disable = std::env::var_os("DISABLE_NEXT_CODE_HOOKS");
        crate::env::remove_var("DISABLE_NEXT_CODE_HOOKS");
        crate::env::set_var("NEXT_CODE_HOME", tmp.path());
        write_user_hooks_toml(tmp.path());

        let payload = hooks_list_payload();
        let hooks = payload["result"]["hooks"].as_array().unwrap();
        assert!(
            hooks.len() >= 2,
            "expected handlers from hooks.toml, got {payload}"
        );
        let names: Vec<&str> = hooks
            .iter()
            .filter_map(|h| h["name"].as_str())
            .collect();
        assert!(
            names.iter().any(|n| n.starts_with("user/PreToolUse[")),
            "names={names:?}"
        );
        assert!(
            hooks.iter().any(|h| h["command"] == "echo pre"),
            "{payload}"
        );
        assert!(
            hooks
                .iter()
                .any(|h| h["sourceDir"].as_str() == Some(tmp.path().to_str().unwrap())),
            "sourceDir should be NEXT_CODE_HOME: {payload}"
        );

        match prev {
            Some(v) => crate::env::set_var("NEXT_CODE_HOME", v),
            None => crate::env::remove_var("NEXT_CODE_HOME"),
        }
        match prev_disable {
            Some(v) => crate::env::set_var("DISABLE_NEXT_CODE_HOOKS", v),
            None => crate::env::remove_var("DISABLE_NEXT_CODE_HOOKS"),
        }
    }

    #[test]
    fn hooks_action_enable_disable_rewrites_toml() {
        let _guard = ENV_LOCK.lock().unwrap();
        let tmp = TempDir::new().unwrap();
        let prev = std::env::var_os("NEXT_CODE_HOME");
        let prev_disable = std::env::var_os("DISABLE_NEXT_CODE_HOOKS");
        crate::env::remove_var("DISABLE_NEXT_CODE_HOOKS");
        crate::env::set_var("NEXT_CODE_HOME", tmp.path());
        write_user_hooks_toml(tmp.path());

        let list = hooks_list_payload();
        let name = list["result"]["hooks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|h| h["command"] == "echo pre")
            .and_then(|h| h["name"].as_str())
            .unwrap()
            .to_string();

        let disable = hooks_action_payload(&json!({
            "sessionId": "s",
            "action": { "type": "disable", "hook_name": name }
        }));
        assert_eq!(disable["result"]["status"], "success", "{disable}");

        let list2 = hooks_list_payload();
        let row = list2["result"]["hooks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|h| h["name"] == name)
            .unwrap();
        assert_eq!(row["disabled"], true, "{list2}");

        let enable = hooks_action_payload(&json!({
            "sessionId": "s",
            "action": { "type": "enable", "hook_name": name }
        }));
        assert_eq!(enable["result"]["status"], "success", "{enable}");
        let list3 = hooks_list_payload();
        let row3 = list3["result"]["hooks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|h| h["name"] == name)
            .unwrap();
        assert_eq!(row3["disabled"], false, "{list3}");

        let reload = hooks_action_payload(&json!({
            "sessionId": "s",
            "action": { "type": "reload" }
        }));
        assert_eq!(reload["result"]["status"], "success", "{reload}");

        match prev {
            Some(v) => crate::env::set_var("NEXT_CODE_HOME", v),
            None => crate::env::remove_var("NEXT_CODE_HOME"),
        }
        match prev_disable {
            Some(v) => crate::env::set_var("DISABLE_NEXT_CODE_HOOKS", v),
            None => crate::env::remove_var("DISABLE_NEXT_CODE_HOOKS"),
        }
    }

    #[test]
    fn hooks_action_add_merges_and_remove_deletes() {
        let _guard = ENV_LOCK.lock().unwrap();
        let tmp = TempDir::new().unwrap();
        let prev = std::env::var_os("NEXT_CODE_HOME");
        let prev_disable = std::env::var_os("DISABLE_NEXT_CODE_HOOKS");
        crate::env::remove_var("DISABLE_NEXT_CODE_HOOKS");
        crate::env::set_var("NEXT_CODE_HOME", tmp.path());
        write_user_hooks_toml(tmp.path());

        let import = tmp.path().join("import.toml");
        fs::write(
            &import,
            r#"
[[events.TurnEnd]]
type = "command"
command = "echo imported"
"#,
        )
        .unwrap();

        let add = hooks_action_payload(&json!({
            "sessionId": "s",
            "action": { "type": "add", "path": import.to_string_lossy() }
        }));
        assert_eq!(add["result"]["status"], "success", "{add}");

        let list = hooks_list_payload();
        let imported = list["result"]["hooks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|h| h["command"] == "echo imported")
            .expect("imported hook missing");
        let name = imported["name"].as_str().unwrap().to_string();

        let remove = hooks_action_payload(&json!({
            "sessionId": "s",
            "action": { "type": "remove", "path": name }
        }));
        assert_eq!(remove["result"]["status"], "success", "{remove}");

        let list2 = hooks_list_payload();
        assert!(
            list2["result"]["hooks"]
                .as_array()
                .unwrap()
                .iter()
                .all(|h| h["command"] != "echo imported"),
            "{list2}"
        );

        match prev {
            Some(v) => crate::env::set_var("NEXT_CODE_HOME", v),
            None => crate::env::remove_var("NEXT_CODE_HOME"),
        }
        match prev_disable {
            Some(v) => crate::env::set_var("DISABLE_NEXT_CODE_HOOKS", v),
            None => crate::env::remove_var("DISABLE_NEXT_CODE_HOOKS"),
        }
    }
}
