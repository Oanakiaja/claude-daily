use anyhow::{Context, Result};
use serde_json::{json, Map, Value};
use std::fs;

use crate::config::load_config;

/// Install plugin to Claude Code
pub async fn run(scope: String) -> Result<()> {
    let _config = load_config()?;

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

    println!("[daily] Installing plugin to: {}", target_dir.display());

    // Create directories
    let commands_dir = target_dir.join("commands");
    let hooks_dir = target_dir.join("hooks");

    fs::create_dir_all(&commands_dir)?;
    fs::create_dir_all(&hooks_dir)?;

    // Write hooks configuration
    let hooks_config = r#"{
  "description": "Daily Context Archive hooks for automatic session archiving",
  "hooks": {
    "SessionStart": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "daily hook session-start"
          }
        ]
      }
    ],
    "SessionEnd": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "daily hook session-end"
          }
        ]
      }
    ]
  }
}
"#;

    let hooks_file = hooks_dir.join("daily-hooks.json");
    fs::write(&hooks_file, hooks_config)?;
    println!("[daily] Hooks installed: {}", hooks_file.display());

    // Write daily-view command
    let view_command = r#"---
description: "View today's daily archive or a specific date"
argument-hint: "[--date YYYY-MM-DD] [--list] [--summary-only]"
allowed-tools: ["Bash(daily view:*)"]
---

# Daily View Command

View the daily context archive.

## Usage

To view today's archive:
```bash
daily view
```

To view a specific date:
```bash
daily view --date $ARGUMENTS
```

To list all sessions:
```bash
daily view --list
```

To view only the daily summary:
```bash
daily view --summary-only
```

Display the output to the user in a readable format.
"#;

    let view_file = commands_dir.join("daily-view.md");
    fs::write(&view_file, view_command)?;
    println!("[daily] Command installed: {}", view_file.display());

    // Write daily-get-skill command
    let skill_command = r#"---
description: "Extract and generate a skill from daily archive insights"
argument-hint: "[--session NAME] [--output PATH]"
allowed-tools: ["Bash(daily extract-skill:*)", "Write(**/skills/**/*.md)"]
---

# Extract Skill Command

Extract a reusable skill from today's session insights.

## Workflow

1. First, list today's sessions and skill suggestions:
```bash
daily view --list
```

2. Review the skill hints from the daily summary:
```bash
daily view --summary-only
```

3. Extract the skill:
```bash
daily extract-skill $ARGUMENTS
```

4. The skill will be generated and saved. Review the output and offer to:
   - Install to user skills: `~/.claude/skills/`
   - Install to project skills: `.claude/skills/`
   - Modify the generated skill

Ask the user where they want to install the skill and make any requested modifications.
"#;

    let skill_file = commands_dir.join("daily-get-skill.md");
    fs::write(&skill_file, skill_command)?;
    println!("[daily] Command installed: {}", skill_file.display());

    // Write daily-get-command command
    let cmd_command = r#"---
description: "Extract and generate a command from daily archive insights"
argument-hint: "[--session NAME] [--output PATH]"
allowed-tools: ["Bash(daily extract-command:*)", "Write(**/commands/**/*.md)"]
---

# Extract Command

Extract a reusable command from today's session insights.

## Workflow

1. First, list today's sessions and command suggestions:
```bash
daily view --list
```

2. Review the command hints from the daily summary:
```bash
daily view --summary-only
```

3. Extract the command:
```bash
daily extract-command $ARGUMENTS
```

4. The command will be generated and saved. Review the output and offer to:
   - Install to user commands: `~/.claude/commands/`
   - Install to project commands: `.claude/commands/`
   - Modify the generated command

Ask the user where they want to install the command and make any requested modifications.
"#;

    let cmd_file = commands_dir.join("daily-get-command.md");
    fs::write(&cmd_file, cmd_command)?;
    println!("[daily] Command installed: {}", cmd_file.display());

    // Update settings.json to enable hooks
    let settings_file = target_dir.join("settings.json");
    let daily_hooks = create_daily_hooks();

    if settings_file.exists() {
        // Read and merge with existing settings
        let content =
            fs::read_to_string(&settings_file).context("Failed to read existing settings.json")?;
        let mut settings: Value =
            serde_json::from_str(&content).context("Failed to parse settings.json")?;

        let merged = merge_hooks(&mut settings, &daily_hooks);
        if merged {
            let output = serde_json::to_string_pretty(&settings)?;
            fs::write(&settings_file, output)?;
            println!("[daily] Hooks merged into: {}", settings_file.display());
        } else {
            println!(
                "[daily] Hooks already configured in: {}",
                settings_file.display()
            );
        }
    } else {
        // Create new settings file
        let settings = json!({
            "hooks": daily_hooks
        });
        let output = serde_json::to_string_pretty(&settings)?;
        fs::write(&settings_file, output)?;
        println!("[daily] Settings installed: {}", settings_file.display());
    }

    println!();
    println!("[daily] Installation complete!");
    println!();
    println!("Available commands:");
    println!("  /daily-view          - View today's archive");
    println!("  /daily-get-skill     - Extract a skill from insights");
    println!("  /daily-get-command   - Extract a command from insights");
    println!();
    println!("Hooks are now active. Sessions will be automatically archived.");

    Ok(())
}

/// Install hooks only (re-enable automatic summarization)
pub async fn run_hooks_only(scope: String) -> Result<()> {
    let _config = load_config()?;

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

    println!("[daily] Installing hooks to: {}", target_dir.display());

    // Create hooks directory
    let hooks_dir = target_dir.join("hooks");
    fs::create_dir_all(&hooks_dir)?;

    // Write hooks configuration
    let hooks_config = r#"{
  "description": "Daily Context Archive hooks for automatic session archiving",
  "hooks": {
    "SessionStart": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "daily hook session-start"
          }
        ]
      }
    ],
    "SessionEnd": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "daily hook session-end"
          }
        ]
      }
    ]
  }
}
"#;

    let hooks_file = hooks_dir.join("daily-hooks.json");
    fs::write(&hooks_file, hooks_config)?;
    println!("[daily] Hooks installed: {}", hooks_file.display());

    // Update settings.json to enable hooks
    let settings_file = target_dir.join("settings.json");
    let daily_hooks = create_daily_hooks();

    if settings_file.exists() {
        let content =
            fs::read_to_string(&settings_file).context("Failed to read existing settings.json")?;
        let mut settings: Value =
            serde_json::from_str(&content).context("Failed to parse settings.json")?;

        let merged = merge_hooks(&mut settings, &daily_hooks);
        if merged {
            let output = serde_json::to_string_pretty(&settings)?;
            fs::write(&settings_file, output)?;
            println!("[daily] Hooks merged into: {}", settings_file.display());
        } else {
            println!(
                "[daily] Hooks already configured in: {}",
                settings_file.display()
            );
        }
    } else {
        let settings = json!({
            "hooks": daily_hooks
        });
        let output = serde_json::to_string_pretty(&settings)?;
        fs::write(&settings_file, output)?;
        println!("[daily] Settings installed: {}", settings_file.display());
    }

    println!();
    println!("[daily] Hooks installed! Automatic summarization is now enabled.");
    println!("[daily] Tip: Use 'daily uninstall-hooks' to disable automatic summarization.");

    Ok(())
}

/// Create the daily hooks configuration
fn create_daily_hooks() -> Map<String, Value> {
    let mut hooks = Map::new();

    let session_start_hook = json!([{
        "hooks": [{
            "type": "command",
            "command": "daily hook session-start"
        }]
    }]);

    let session_end_hook = json!([{
        "hooks": [{
            "type": "command",
            "command": "daily hook session-end"
        }]
    }]);

    hooks.insert("SessionStart".to_string(), session_start_hook);
    hooks.insert("SessionEnd".to_string(), session_end_hook);
    hooks
}

/// Check if a hook array already contains the daily hook command
fn has_daily_hook(hooks_array: &[Value], command: &str) -> bool {
    for hook_entry in hooks_array {
        if let Some(inner_hooks) = hook_entry.get("hooks").and_then(|h| h.as_array()) {
            for inner_hook in inner_hooks {
                if let Some(cmd) = inner_hook.get("command").and_then(|c| c.as_str()) {
                    if cmd == command {
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// Merge daily hooks into existing settings, returns true if changes were made
fn merge_hooks(settings: &mut Value, daily_hooks: &Map<String, Value>) -> bool {
    let mut changed = false;

    // Ensure hooks object exists
    if settings.get("hooks").is_none() {
        settings["hooks"] = json!({});
    }

    let hooks = settings["hooks"].as_object_mut().unwrap();

    for (event_name, daily_hook_value) in daily_hooks {
        let command = match event_name.as_str() {
            "SessionStart" => "daily hook session-start",
            "SessionEnd" => "daily hook session-end",
            _ => continue,
        };

        if let Some(existing) = hooks.get_mut(event_name) {
            // Event exists, check if daily hook is already present
            if let Some(existing_array) = existing.as_array_mut() {
                if !has_daily_hook(existing_array, command) {
                    // Append daily hook to existing array
                    if let Some(daily_array) = daily_hook_value.as_array() {
                        for item in daily_array {
                            existing_array.push(item.clone());
                        }
                        changed = true;
                    }
                }
            }
        } else {
            // Event doesn't exist, add it
            hooks.insert(event_name.clone(), daily_hook_value.clone());
            changed = true;
        }
    }

    changed
}
