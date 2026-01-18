use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

use crate::archive::ArchiveManager;
use crate::config::load_config;
use crate::summarizer::SummarizerEngine;

/// Extract skill from archive
pub async fn run_skill(
    date: Option<String>,
    session: Option<String>,
    output: Option<PathBuf>,
) -> Result<()> {
    let config = load_config()?;
    let manager = ArchiveManager::new(config.clone());
    let engine = SummarizerEngine::new(config.clone());

    // Determine date
    let view_date = date.unwrap_or_else(|| chrono::Local::now().format("%Y-%m-%d").to_string());

    // Get session content
    let session_content = get_session_content(&manager, &view_date, session.as_deref()).await?;

    println!("[daily] Extracting skill from session...");
    println!("[daily] Applying quality gate (踩过坑? 下次还会遇到? 能说清楚?)...");

    // Extract skill using Claude
    let skill_content = engine.extract_skill(&session_content, None).await?;

    // Check if skill is extractable
    if skill_content.trim().starts_with("NOT_EXTRACTABLE:") {
        let reason = skill_content
            .trim()
            .strip_prefix("NOT_EXTRACTABLE:")
            .unwrap_or("Unknown reason")
            .trim();
        println!();
        println!("[daily] ⚠ Skill not extractable: {}", reason);
        println!();
        println!("This session may not contain reusable knowledge that meets the quality bar:");
        println!("  - 踩过坑吗？ Was there debugging or non-obvious discovery?");
        println!("  - 下次还会遇到吗？ Is this a recurring problem?");
        println!("  - 能说清楚吗？ Can it be clearly described?");
        println!();
        println!("Try a different session with `daily extract-skill --session <name>`");
        return Ok(());
    }

    // Determine output path
    let output_path = if let Some(path) = output {
        path
    } else {
        // Default to ~/.claude/skills/
        let skills_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".claude")
            .join("skills");

        // Extract skill name from content
        let skill_name = extract_name_from_yaml(&skill_content, "extracted-skill");
        skills_dir.join(&skill_name)
    };

    // Create directory and write file
    fs::create_dir_all(&output_path)?;
    let skill_file = output_path.join("SKILL.md");
    fs::write(&skill_file, &skill_content)?;

    println!("[daily] Skill extracted to: {}", skill_file.display());
    println!();
    println!("Preview:");
    println!("{}", "-".repeat(50));

    // Show first 20 lines
    for line in skill_content.lines().take(20) {
        println!("{}", line);
    }
    println!("...");
    println!("{}", "-".repeat(50));

    println!();
    println!("To install this skill:");
    println!(
        "  User-level: mv {} ~/.claude/skills/",
        output_path.display()
    );
    println!(
        "  Project-level: mv {} .claude/skills/",
        output_path.display()
    );

    Ok(())
}

/// Extract command from archive
pub async fn run_command(
    date: Option<String>,
    session: Option<String>,
    output: Option<PathBuf>,
) -> Result<()> {
    let config = load_config()?;
    let manager = ArchiveManager::new(config.clone());
    let engine = SummarizerEngine::new(config.clone());

    // Determine date
    let view_date = date.unwrap_or_else(|| chrono::Local::now().format("%Y-%m-%d").to_string());

    // Get session content
    let session_content = get_session_content(&manager, &view_date, session.as_deref()).await?;

    println!("[daily] Extracting command from session...");

    // Extract command using Claude
    let command_content = engine.extract_command(&session_content, None).await?;

    // Determine output path
    let output_path = if let Some(path) = output {
        path
    } else {
        // Default to ~/.claude/commands/
        let commands_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".claude")
            .join("commands");

        // Extract command name from content
        let command_name = extract_name_from_content(&command_content, "extracted-command");
        commands_dir.join(format!("{}.md", command_name))
    };

    // Create parent directory and write file
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&output_path, &command_content)?;

    println!("[daily] Command extracted to: {}", output_path.display());
    println!();
    println!("Preview:");
    println!("{}", "-".repeat(50));

    // Show first 20 lines
    for line in command_content.lines().take(20) {
        println!("{}", line);
    }
    if command_content.lines().count() > 20 {
        println!("...");
    }
    println!("{}", "-".repeat(50));

    println!();
    println!("To install this command:");
    println!(
        "  User-level: mv {} ~/.claude/commands/",
        output_path.display()
    );
    println!(
        "  Project-level: mv {} .claude/commands/",
        output_path.display()
    );

    Ok(())
}

/// Get session content, either from specific session or most recent
async fn get_session_content(
    manager: &ArchiveManager,
    date: &str,
    session: Option<&str>,
) -> Result<String> {
    if let Some(session_name) = session {
        manager
            .read_session(date, session_name)
            .context(format!("Failed to read session: {}", session_name))
    } else {
        // Get most recent session
        let sessions = manager.list_sessions(date)?;
        if sessions.is_empty() {
            anyhow::bail!("No sessions found for {}", date);
        }

        let latest = sessions.last().unwrap();
        manager
            .read_session(date, latest)
            .context(format!("Failed to read session: {}", latest))
    }
}

/// Extract name from YAML frontmatter
fn extract_name_from_yaml(content: &str, default: &str) -> String {
    // Look for name: in frontmatter
    for line in content.lines() {
        if line.starts_with("name:") {
            let name = line.trim_start_matches("name:").trim();
            let name = name.trim_matches('"').trim_matches('\'');
            if !name.is_empty() {
                return name.to_string();
            }
        }
    }
    default.to_string()
}

/// Extract name from content (for commands)
fn extract_name_from_content(content: &str, default: &str) -> String {
    // Look for # heading
    for line in content.lines() {
        if line.starts_with("# ") {
            let name = line.trim_start_matches("# ").trim();
            if !name.is_empty() {
                // Convert to kebab-case
                return name
                    .to_lowercase()
                    .replace(' ', "-")
                    .chars()
                    .filter(|c| c.is_alphanumeric() || *c == '-')
                    .collect();
            }
        }
    }
    default.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_name_from_yaml() {
        let content = r#"---
name: my-skill
description: test
---"#;
        assert_eq!(extract_name_from_yaml(content, "default"), "my-skill");
    }

    #[test]
    fn test_extract_name_from_content() {
        let content = r#"---
description: test
---

# My Command Name

Instructions here."#;
        assert_eq!(
            extract_name_from_content(content, "default"),
            "my-command-name"
        );
    }
}
