//! Handlers for the `jcode secrets` subcommand group.
//!
//! Reads and writes the encrypted, OS-keychain-backed secrets store via
//! [`jcode_secrets::SecretsManager`]. Secret values are only ever printed by
//! the explicit `get` command; `set`/`delete`/`list` never echo values.

use anyhow::{Context, Result};
use std::io::Read;

use jcode_secrets::{
    SecretName, SecretScope, SecretsBackendKind, SecretsManager, environment_id_from_cwd,
};
use serde::Serialize;

fn manager() -> Result<SecretsManager> {
    let jcode_home = jcode_storage::jcode_dir().context("resolve jcode home directory")?;
    SecretsManager::new(jcode_home, SecretsBackendKind::Local)
}

/// `--env` selects an environment scope keyed to the current git repo / cwd;
/// otherwise the global scope is used.
fn scope_for(env: bool) -> Result<SecretScope> {
    if env {
        let cwd = std::env::current_dir().context("resolve current directory")?;
        Ok(SecretScope::Environment(environment_id_from_cwd(&cwd)))
    } else {
        Ok(SecretScope::Global)
    }
}

fn read_value_from_stdin() -> Result<String> {
    let mut buf = String::new();
    std::io::stdin()
        .read_to_string(&mut buf)
        .context("read secret value from stdin")?;
    let trimmed = buf.trim_end_matches(['\n', '\r']).to_string();
    if trimmed.is_empty() {
        anyhow::bail!("No secret value provided (stdin was empty)");
    }
    Ok(trimmed)
}

#[derive(Serialize)]
struct SetOutput {
    name: String,
    scope: String,
    status: &'static str,
}

pub fn run_set(name: &str, value: Option<&str>, env: bool, json: bool) -> Result<()> {
    let manager = manager()?;
    let scope = scope_for(env)?;
    let secret_name = SecretName::new(name)?;
    let value = match value {
        Some(v) => v.to_string(),
        None => read_value_from_stdin()?,
    };
    manager.set(&scope, &secret_name, &value)?;

    let out = SetOutput {
        name: secret_name.as_str().to_string(),
        scope: scope.to_string(),
        status: "set",
    };
    if json {
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!("Set {} ({}).", out.name, out.scope);
    }
    Ok(())
}

#[derive(Serialize)]
struct GetOutput {
    name: String,
    scope: String,
    found: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<String>,
}

pub fn run_get(name: &str, env: bool, json: bool) -> Result<()> {
    let manager = manager()?;
    let scope = scope_for(env)?;
    let secret_name = SecretName::new(name)?;
    let value = manager.get(&scope, &secret_name)?;

    if json {
        let out = GetOutput {
            name: secret_name.as_str().to_string(),
            scope: scope.to_string(),
            found: value.is_some(),
            value: value.clone(),
        };
        println!("{}", serde_json::to_string_pretty(&out)?);
        return Ok(());
    }

    match value {
        Some(v) => {
            // Print the raw value so it is scriptable: KEY=$(jcode secrets get NAME)
            println!("{v}");
            Ok(())
        }
        None => anyhow::bail!("No secret named {} in {} scope", secret_name, scope),
    }
}

#[derive(Serialize)]
struct DeleteOutput {
    name: String,
    scope: String,
    deleted: bool,
}

pub fn run_delete(name: &str, env: bool, json: bool) -> Result<()> {
    let manager = manager()?;
    let scope = scope_for(env)?;
    let secret_name = SecretName::new(name)?;
    let deleted = manager.delete(&scope, &secret_name)?;

    let out = DeleteOutput {
        name: secret_name.as_str().to_string(),
        scope: scope.to_string(),
        deleted,
    };
    if json {
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else if deleted {
        println!("Deleted {} ({}).", out.name, out.scope);
    } else {
        println!("No secret named {} in {} scope.", out.name, out.scope);
    }
    Ok(())
}

#[derive(Serialize)]
struct ListEntryOut {
    scope: String,
    name: String,
}

#[derive(Serialize)]
struct ListOutput {
    secrets: Vec<ListEntryOut>,
}

pub fn run_list(env: bool, json: bool) -> Result<()> {
    let manager = manager()?;
    // With --env, restrict to the current environment scope; otherwise list all.
    let filter = if env { Some(scope_for(true)?) } else { None };
    let mut entries = manager.list(filter.as_ref())?;
    entries.sort_by(|a, b| {
        a.scope
            .to_string()
            .cmp(&b.scope.to_string())
            .then_with(|| a.name.as_str().cmp(b.name.as_str()))
    });

    let out = ListOutput {
        secrets: entries
            .iter()
            .map(|e| ListEntryOut {
                scope: e.scope.to_string(),
                name: e.name.as_str().to_string(),
            })
            .collect(),
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else if out.secrets.is_empty() {
        println!("No secrets stored.");
    } else {
        for entry in &out.secrets {
            println!("{}\t{}", entry.scope, entry.name);
        }
    }
    Ok(())
}

#[derive(Serialize)]
struct InitOutput {
    status: &'static str,
}

pub fn run_init(json: bool) -> Result<()> {
    let manager = manager()?;
    manager.initialize()?;
    let out = InitOutput {
        status: "initialized",
    };
    if json {
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!("Initialized encrypted secrets store and OS keychain passphrase.");
    }
    Ok(())
}
