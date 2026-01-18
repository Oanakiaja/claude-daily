use anyhow::Result;
use std::path::PathBuf;

use crate::config::{get_config_path, load_config, save_config};

/// Show or update configuration
pub async fn run(set_storage: Option<PathBuf>, show: bool) -> Result<()> {
    let mut config = load_config()?;

    // Update storage path if provided
    if let Some(path) = set_storage {
        config.storage.path = path.clone();
        save_config(&config)?;
        println!("[daily] Storage path updated to: {}", path.display());
        return Ok(());
    }

    // Show current config
    if show || set_storage.is_none() {
        let config_path = get_config_path()?;
        println!("[daily] Configuration file: {}", config_path.display());
        println!();
        println!("Current settings:");
        println!("  Storage path: {}", config.storage.path.display());
        println!("  Summarization model: {}", config.summarization.model);
        println!(
            "  Enable daily summary: {}",
            config.summarization.enable_daily_summary
        );
        println!(
            "  Enable extraction hints: {}",
            config.summarization.enable_extraction_hints
        );
        println!("  SessionStart hook: {}", config.hooks.enable_session_start);
        println!("  SessionEnd hook: {}", config.hooks.enable_session_end);
        println!("  Background timeout: {}s", config.hooks.background_timeout);
        println!();
        println!("Archive settings:");
        println!(
            "  Author: {}",
            config.archive.author.as_deref().unwrap_or("(not set)")
        );
        println!("  Tags: {}", config.archive.tags.join(", "));
        println!("  Include cwd: {}", config.archive.include_cwd);
        println!("  Include git info: {}", config.archive.include_git_info);
    }

    Ok(())
}
