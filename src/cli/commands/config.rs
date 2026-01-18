use anyhow::{Context, Result};
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use std::path::PathBuf;

use crate::config::{get_config_path, load_config, save_config};

/// Show or update configuration
pub async fn run(set_storage: Option<PathBuf>, show: bool, interactive: bool) -> Result<()> {
    let mut config = load_config()?;

    // Interactive mode
    if interactive {
        return configure_interactive(&mut config).await;
    }

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
        println!();
        println!("Tip: Use 'daily config -i' for interactive configuration");
    }

    Ok(())
}

/// Interactive configuration
async fn configure_interactive(
    config: &mut crate::config::Config,
) -> Result<()> {
    let theme = ColorfulTheme::default();

    println!("[daily] Interactive Configuration");
    println!();

    // Model selection
    let models = vec!["haiku (faster, cheaper)", "sonnet (smarter, slower)"];
    let current_model_idx = if config.summarization.model == "sonnet" {
        1
    } else {
        0
    };

    let model_selection = Select::with_theme(&theme)
        .with_prompt("Select summarization model")
        .items(&models)
        .default(current_model_idx)
        .interact()
        .context("Failed to select model")?;

    config.summarization.model = if model_selection == 1 {
        "sonnet".into()
    } else {
        "haiku".into()
    };

    // Enable daily summary
    let enable_daily_summary = Confirm::with_theme(&theme)
        .with_prompt("Enable daily summary generation?")
        .default(config.summarization.enable_daily_summary)
        .interact()
        .unwrap_or(config.summarization.enable_daily_summary);
    config.summarization.enable_daily_summary = enable_daily_summary;

    // Enable extraction hints
    let enable_extraction = Confirm::with_theme(&theme)
        .with_prompt("Enable skill/command extraction hints?")
        .default(config.summarization.enable_extraction_hints)
        .interact()
        .unwrap_or(config.summarization.enable_extraction_hints);
    config.summarization.enable_extraction_hints = enable_extraction;

    // Auto digest
    let auto_digest = Confirm::with_theme(&theme)
        .with_prompt("Enable auto-digest of previous day's sessions?")
        .default(config.summarization.auto_digest_enabled)
        .interact()
        .unwrap_or(config.summarization.auto_digest_enabled);
    config.summarization.auto_digest_enabled = auto_digest;

    if auto_digest {
        let digest_time: String = Input::with_theme(&theme)
            .with_prompt("Digest time (HH:MM)")
            .default(config.summarization.digest_time.clone())
            .validate_with(|input: &String| -> std::result::Result<(), &str> {
                let parts: Vec<&str> = input.split(':').collect();
                if parts.len() != 2 {
                    return Err("Format must be HH:MM");
                }
                match (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                    (Ok(h), Ok(m)) if h < 24 && m < 60 => Ok(()),
                    _ => Err("Invalid time"),
                }
            })
            .interact_text()
            .context("Failed to read digest time")?;
        config.summarization.digest_time = digest_time;
    }

    // Author
    let author: String = Input::with_theme(&theme)
        .with_prompt("Author name (leave empty to skip)")
        .default(config.archive.author.clone().unwrap_or_default())
        .allow_empty(true)
        .interact_text()
        .context("Failed to read author")?;
    config.archive.author = if author.is_empty() {
        None
    } else {
        Some(author)
    };

    // Save config
    save_config(config)?;

    println!();
    println!("[daily] Configuration saved!");
    println!();
    println!("Updated settings:");
    println!("  Model: {}", config.summarization.model);
    println!(
        "  Daily summary: {}",
        config.summarization.enable_daily_summary
    );
    println!(
        "  Extraction hints: {}",
        config.summarization.enable_extraction_hints
    );
    println!(
        "  Auto digest: {}",
        config.summarization.auto_digest_enabled
    );
    if config.summarization.auto_digest_enabled {
        println!("  Digest time: {}", config.summarization.digest_time);
    }
    println!(
        "  Author: {}",
        config.archive.author.as_deref().unwrap_or("(not set)")
    );

    Ok(())
}
