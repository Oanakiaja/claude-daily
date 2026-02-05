use anyhow::{Context, Result};
use serde_json::Value;
use std::fs;
use std::io::{self, Write};

/// Uninstall plugin from Claude Code
pub async fn run(scope: String, delete_binary: bool) -> Result<()> {
    let target_dir = match scope.as_str() {
        "user" => dirs::home_dir()
            .context("Failed to get home directory")?
            .join(".claude"),
        "project" => std::env::current_dir()
            .context("Failed to get current directory")?
            .join(".claude"),
        _ => {
            anyhow::bail!("Invalid scope: {}. Use 'user' or 'project'", scope);
        }
    };

    println!("[daily] Uninstalling plugin from: {}", target_dir.display());

    let mut removed_count = 0;

    // Remove hooks configuration file
    let hooks_file = target_dir.join("hooks").join("daily-hooks.json");
    if hooks_file.exists() {
        fs::remove_file(&hooks_file)?;
        println!("[daily] Removed: {}", hooks_file.display());
        removed_count += 1;
    }

    // Remove command files
    let commands_dir = target_dir.join("commands");
    let command_files = [
        "daily-view.md",
        "daily-get-skill.md",
        "daily-get-command.md",
    ];

    for cmd_file in &command_files {
        let file_path = commands_dir.join(cmd_file);
        if file_path.exists() {
            fs::remove_file(&file_path)?;
            println!("[daily] Removed: {}", file_path.display());
            removed_count += 1;
        }
    }

    // Remove daily hooks from settings.json
    let settings_file = target_dir.join("settings.json");
    if settings_file.exists() {
        let content = fs::read_to_string(&settings_file).context("Failed to read settings.json")?;
        let mut settings: Value =
            serde_json::from_str(&content).context("Failed to parse settings.json")?;

        if remove_daily_hooks(&mut settings) {
            let output = serde_json::to_string_pretty(&settings)?;
            fs::write(&settings_file, output)?;
            println!("[daily] Removed hooks from: {}", settings_file.display());
            removed_count += 1;
        }
    }

    println!();
    if removed_count > 0 {
        println!(
            "[daily] Uninstall complete! Removed {} items.",
            removed_count
        );
        println!("[daily] Note: Archive data (~/.claude/daily/) was preserved.");
    } else {
        println!("[daily] Nothing to uninstall. Plugin was not installed.");
    }

    // Handle binary deletion if requested
    if delete_binary {
        println!();
        delete_daily_binary()?;
    }

    Ok(())
}

/// Delete the daily binary itself
fn delete_daily_binary() -> Result<()> {
    let current_exe = std::env::current_exe().context("Failed to get current executable path")?;
    let exe_path = current_exe
        .canonicalize()
        .unwrap_or_else(|_| current_exe.clone());

    println!("[daily] Binary location: {}", exe_path.display());

    // Confirm deletion
    print!("[daily] Delete this binary? [y/N] ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if input.trim().to_lowercase() == "y" {
        fs::remove_file(&exe_path).context("Failed to delete binary")?;
        println!("[daily] Binary deleted: {}", exe_path.display());
        println!("[daily] Goodbye!");
    } else {
        println!("[daily] Binary deletion cancelled.");
    }

    Ok(())
}

/// Remove daily hooks from settings, returns true if changes were made
fn remove_daily_hooks(settings: &mut Value) -> bool {
    let mut changed = false;

    if let Some(hooks) = settings.get_mut("hooks").and_then(|h| h.as_object_mut()) {
        // Remove daily hooks from SessionStart
        if let Some(session_start) = hooks.get_mut("SessionStart") {
            if let Some(arr) = session_start.as_array_mut() {
                let original_len = arr.len();
                arr.retain(|entry| !is_daily_hook_entry(entry, "daily hook session-start"));
                if arr.len() != original_len {
                    changed = true;
                }
                // Remove the event entirely if no hooks remain
                if arr.is_empty() {
                    hooks.remove("SessionStart");
                }
            }
        }

        // Remove daily hooks from SessionEnd
        if let Some(session_end) = hooks.get_mut("SessionEnd") {
            if let Some(arr) = session_end.as_array_mut() {
                let original_len = arr.len();
                arr.retain(|entry| !is_daily_hook_entry(entry, "daily hook session-end"));
                if arr.len() != original_len {
                    changed = true;
                }
                // Remove the event entirely if no hooks remain
                if arr.is_empty() {
                    hooks.remove("SessionEnd");
                }
            }
        }

        // Remove hooks object entirely if empty
        if hooks.is_empty() {
            settings.as_object_mut().unwrap().remove("hooks");
        }
    }

    changed
}

/// Check if a hook entry contains the specified daily command
fn is_daily_hook_entry(entry: &Value, command: &str) -> bool {
    if let Some(inner_hooks) = entry.get("hooks").and_then(|h| h.as_array()) {
        for inner_hook in inner_hooks {
            if let Some(cmd) = inner_hook.get("command").and_then(|c| c.as_str()) {
                if cmd == command {
                    return true;
                }
            }
        }
    }
    false
}
