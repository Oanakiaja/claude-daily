use anyhow::Result;
use std::fs;
use std::path::PathBuf;

use crate::config::load_config;

/// Review pending skills
pub async fn run_review(install: Option<String>, delete: Option<String>) -> Result<()> {
    let config = load_config()?;
    let pending_dir = config.storage.path.join("pending-skills");

    if !pending_dir.exists() {
        println!("No pending skills to review.");
        return Ok(());
    }

    // Handle install action
    if let Some(skill_path) = install {
        return install_skill(&pending_dir, &skill_path);
    }

    // Handle delete action
    if let Some(skill_path) = delete {
        return delete_skill(&pending_dir, &skill_path);
    }

    // List all pending skills
    list_pending_skills(&pending_dir)
}

/// List all pending skills
fn list_pending_skills(pending_dir: &PathBuf) -> Result<()> {
    let mut skills: Vec<(String, String, PathBuf)> = Vec::new();

    if let Ok(entries) = fs::read_dir(pending_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let date = entry.file_name().to_string_lossy().to_string();
                if let Ok(files) = fs::read_dir(entry.path()) {
                    for file in files.flatten() {
                        if file.path().extension().map_or(false, |e| e == "md") {
                            let name = file
                                .path()
                                .file_stem()
                                .map(|s| s.to_string_lossy().to_string())
                                .unwrap_or_default();
                            skills.push((date.clone(), name, file.path()));
                        }
                    }
                }
            }
        }
    }

    if skills.is_empty() {
        println!("No pending skills to review.");
        return Ok(());
    }

    println!("Pending Skills ({} total):", skills.len());
    println!("{}", "─".repeat(60));

    for (date, name, path) in &skills {
        println!();
        println!("📦 {}/{}", date, name);

        // Read and show preview
        if let Ok(content) = fs::read_to_string(path) {
            // Extract description from frontmatter
            if let Some(desc) = extract_description(&content) {
                println!("   {}", desc);
            }

            // Show trigger conditions if present
            if let Some(trigger) = extract_section(&content, "## When to Use") {
                let preview: String = trigger.lines().take(3).collect::<Vec<_>>().join("\n   ");
                println!("   Trigger: {}", preview.trim());
            }
        }

        println!();
        println!("   Actions:");
        println!("     daily review-skills --install {}/{}", date, name);
        println!("     daily review-skills --delete {}/{}", date, name);
    }

    println!();
    println!("{}", "─".repeat(60));
    println!("Or ask Claude: \"install skill {}/{}\"", skills[0].0, skills[0].1);

    Ok(())
}

/// Install a skill to user's skills directory
fn install_skill(pending_dir: &PathBuf, skill_ref: &str) -> Result<()> {
    let (date, name) = parse_skill_ref(skill_ref)?;
    let skill_path = pending_dir.join(&date).join(format!("{}.md", name));

    if !skill_path.exists() {
        anyhow::bail!("Skill not found: {}/{}", date, name);
    }

    // Read skill content
    let content = fs::read_to_string(&skill_path)?;

    // Install to ~/.claude/skills/{name}/SKILL.md
    let target_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude")
        .join("skills")
        .join(&name);

    fs::create_dir_all(&target_dir)?;
    let target_file = target_dir.join("SKILL.md");
    fs::write(&target_file, &content)?;

    // Remove from pending
    fs::remove_file(&skill_path)?;

    // Clean up empty date directory
    let date_dir = pending_dir.join(&date);
    if fs::read_dir(&date_dir)?.next().is_none() {
        fs::remove_dir(&date_dir)?;
    }

    println!("✓ Skill installed: {}", target_file.display());
    println!();
    println!("The skill is now active and Claude will automatically use it");
    println!("when matching conditions are detected.");

    Ok(())
}

/// Delete a pending skill
fn delete_skill(pending_dir: &PathBuf, skill_ref: &str) -> Result<()> {
    let (date, name) = parse_skill_ref(skill_ref)?;
    let skill_path = pending_dir.join(&date).join(format!("{}.md", name));

    if !skill_path.exists() {
        anyhow::bail!("Skill not found: {}/{}", date, name);
    }

    fs::remove_file(&skill_path)?;

    // Clean up empty date directory
    let date_dir = pending_dir.join(&date);
    if fs::read_dir(&date_dir)?.next().is_none() {
        fs::remove_dir(&date_dir)?;
    }

    println!("✓ Skill deleted: {}/{}", date, name);

    Ok(())
}

/// Parse skill reference like "2026-01-18/skill-name"
fn parse_skill_ref(skill_ref: &str) -> Result<(String, String)> {
    let parts: Vec<&str> = skill_ref.split('/').collect();
    if parts.len() != 2 {
        anyhow::bail!("Invalid skill reference. Use format: YYYY-MM-DD/skill-name");
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}

/// Extract description from YAML frontmatter
fn extract_description(content: &str) -> Option<String> {
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("description:") {
            let desc = line.trim_start_matches("description:").trim();
            let desc = desc.trim_matches('"').trim_matches('\'');
            if !desc.is_empty() {
                return Some(desc.to_string());
            }
        }
    }
    None
}

/// Extract a section from markdown content
fn extract_section(content: &str, header: &str) -> Option<String> {
    if let Some(start) = content.find(header) {
        let after_header = &content[start + header.len()..];
        // Find next section or end
        let end = after_header
            .find("\n## ")
            .unwrap_or(after_header.len().min(500));
        return Some(after_header[..end].trim().to_string());
    }
    None
}
