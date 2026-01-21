use anyhow::{Context, Result};
use dialoguer::{theme::ColorfulTheme, Confirm, FuzzySelect, Input, Select};
use std::fs;
use std::path::{Path, PathBuf};

use super::install;
use crate::archive::ArchiveManager;
use crate::config::{get_config_path, load_config, save_config, Config};

/// Initialize the daily archive system
pub async fn run(storage_path: Option<PathBuf>, interactive: bool, use_haiku: bool) -> Result<()> {
    println!("[daily] Initializing Daily Context Archive System...");

    // Load or create config
    let mut config = load_config().unwrap_or_else(|_| Config::default());

    // Set model based on flag
    if use_haiku {
        config.summarization.model = "haiku".into();
        println!("[daily] Using haiku model for summarization");
    } else {
        println!("[daily] Using sonnet model for summarization (default)");
    }

    // Determine storage path based on options
    let final_path = if let Some(path) = storage_path {
        // Direct path specified
        Some(expand_path(&path))
    } else if interactive {
        // Interactive mode with fuzzy search
        select_directory_interactive()?
    } else {
        // Use default
        None
    };

    // Update storage path if determined
    if let Some(path) = final_path {
        config.storage.path = path;
    }

    // Configure language and digest settings in interactive mode
    if interactive {
        configure_language_interactive(&mut config)?;
        configure_digest_interactive(&mut config)?;
    }

    // Save config
    save_config(&config)?;
    let config_path = get_config_path()?;
    println!("[daily] Configuration saved to: {}", config_path.display());

    // Create storage directory
    let manager = ArchiveManager::new(config.clone());
    let storage_dir = manager.ensure_storage_dir()?;
    println!("[daily] Storage directory: {}", storage_dir.display());

    // Create today's directory
    let today_dir = manager.ensure_today_dir()?;
    println!("[daily] Today's archive: {}", today_dir.display());

    println!();
    println!("[daily] Initialization complete!");

    // Automatically install hooks
    println!();
    install::run("user".to_string()).await?;

    Ok(())
}

/// Expand ~ in path
fn expand_path(path: &Path) -> PathBuf {
    let path_str = path.to_string_lossy();
    if path_str.starts_with("~") {
        if let Some(home) = dirs::home_dir() {
            return home.join(path_str.trim_start_matches("~/").trim_start_matches("~"));
        }
    }
    path.to_path_buf()
}

/// Interactive directory selection with keyword search
fn select_directory_interactive() -> Result<Option<PathBuf>> {
    let theme = ColorfulTheme::default();

    // Ask user for storage path with default
    let input: String = dialoguer::Input::with_theme(&theme)
        .with_prompt("Daily archive folder (e.g., '~/obsidian-vault/daily', '~/.claude/daily')")
        .default("~/.claude/daily".into())
        .interact_text()
        .context("Failed to read input")?;

    let input = input.trim();

    // If user entered the default or empty, use default path
    if input.is_empty() || input == "~/.claude/daily" {
        return Ok(Some(expand_path(Path::new("~/.claude/daily"))));
    }

    // Check if input looks like a path (contains / or ~)
    if input.contains('/') || input.starts_with('~') {
        // User entered a direct path
        return Ok(Some(expand_path(Path::new(input))));
    }

    // Otherwise treat as keyword search
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let candidates = search_directories_recursive(&home, &input.to_lowercase(), 10);

    if candidates.is_empty() {
        println!("[daily] No matching directories found. Using default path.");
        return Ok(None);
    }

    // Build display items with path info
    let display_items: Vec<String> = candidates
        .iter()
        .map(|p| format_path_display(p.as_path()))
        .collect();

    // Use FuzzySelect for selection (user can type to filter)
    let selection = FuzzySelect::with_theme(&theme)
        .with_prompt("Select archive directory (type to filter)")
        .items(&display_items)
        .default(0)
        .max_length(15)
        .highlight_matches(true)
        .interact_opt()
        .context("Failed to select directory")?;

    match selection {
        Some(idx) => {
            let selected = &candidates[idx];
            // If selected path doesn't end with "daily", append it
            let final_path = if !selected.ends_with("daily") {
                selected.join("daily")
            } else {
                selected.clone()
            };
            Ok(Some(final_path))
        }
        None => {
            println!("[daily] Selection cancelled. Using default path.");
            Ok(None)
        }
    }
}

/// Recursively search for directories matching the keyword
fn search_directories_recursive(root: &Path, keyword: &str, max_depth: usize) -> Vec<PathBuf> {
    let mut results = Vec::new();
    search_recursive_helper(root, keyword, max_depth, 0, &mut results);

    // Sort: prefer shorter paths and paths containing "daily"
    results.sort_by(|a, b| {
        let a_has_daily = a.to_string_lossy().to_lowercase().contains("daily");
        let b_has_daily = b.to_string_lossy().to_lowercase().contains("daily");
        match (a_has_daily, b_has_daily) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.to_string_lossy().len().cmp(&b.to_string_lossy().len()),
        }
    });

    results.truncate(100);
    results
}

fn search_recursive_helper(
    dir: &Path,
    keyword: &str,
    max_depth: usize,
    current_depth: usize,
    results: &mut Vec<PathBuf>,
) {
    if current_depth > max_depth || results.len() >= 200 {
        return;
    }

    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        // Skip most hidden directories but allow some special ones
        if name.starts_with('.') && name != ".claude" {
            continue;
        }

        // Skip system directories that are unlikely to contain user data
        if matches!(
            name,
            "node_modules"
                | "target"
                | "build"
                | "dist"
                | ".git"
                | "vendor"
                | "cache"
                | "Cache"
                | "Caches"
        ) {
            continue;
        }

        // Check if directory name matches keyword
        if name.to_lowercase().contains(keyword) && !results.contains(&path) {
            results.push(path.clone());
        }

        // Continue searching deeper
        search_recursive_helper(&path, keyword, max_depth, current_depth + 1, results);
    }
}

/// Format path for display, showing abbreviated home path
fn format_path_display(path: &Path) -> String {
    let path_str = path.to_string_lossy();

    if let Some(home) = dirs::home_dir() {
        let home_str = home.to_string_lossy();
        if path_str.starts_with(home_str.as_ref()) {
            let relative = path_str.replacen(home_str.as_ref(), "~", 1);
            let exists = if path.exists() { "" } else { " (will create)" };
            return format!("{}{}", relative, exists);
        }
    }

    let exists = if path.exists() { "" } else { " (will create)" };
    format!("{}{}", path_str, exists)
}

/// Interactive configuration for language settings
fn configure_language_interactive(config: &mut Config) -> Result<()> {
    let theme = ColorfulTheme::default();

    println!();
    println!("[daily] Language Configuration");

    let languages = ["English", "中文 (Chinese)"];
    let language_codes = ["en", "zh"];

    let selection = Select::with_theme(&theme)
        .with_prompt("Select summary language")
        .items(&languages)
        .default(0)
        .interact()
        .context("Failed to select language")?;

    config.summarization.summary_language = language_codes[selection].to_string();
    println!("[daily] Summary language set to: {}", languages[selection]);

    Ok(())
}

/// Interactive configuration for digest settings
fn configure_digest_interactive(config: &mut Config) -> Result<()> {
    let theme = ColorfulTheme::default();

    println!();
    println!("[daily] Digest Configuration");
    println!("  Sessions are archived individually. Digest consolidates them into daily.md.");
    println!();

    // Ask about auto-digest
    let auto_digest = Confirm::with_theme(&theme)
        .with_prompt("Enable auto-digest of previous day's sessions?")
        .default(true)
        .interact()
        .unwrap_or(true);

    config.summarization.auto_digest_enabled = auto_digest;

    if auto_digest {
        // Ask for digest time
        let digest_time: String = Input::with_theme(&theme)
            .with_prompt("Digest time (HH:MM format, when to consolidate yesterday's sessions)")
            .default("06:00".into())
            .validate_with(|input: &String| -> std::result::Result<(), &str> {
                let parts: Vec<&str> = input.split(':').collect();
                if parts.len() != 2 {
                    return Err("Format must be HH:MM");
                }
                match (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                    (Ok(h), Ok(m)) if h < 24 && m < 60 => Ok(()),
                    _ => Err("Invalid time (use 00-23 for hour, 00-59 for minute)"),
                }
            })
            .interact_text()
            .context("Failed to read digest time")?;

        config.summarization.digest_time = digest_time;
    }

    Ok(())
}
