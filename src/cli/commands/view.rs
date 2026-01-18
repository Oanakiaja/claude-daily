use anyhow::Result;
use chrono::{Duration, Local};
use colored::*;
use dialoguer::{theme::ColorfulTheme, FuzzySelect};

use crate::archive::ArchiveManager;
use crate::config::load_config;

/// View archives with interactive selection
pub async fn run(date: Option<String>, summary_only: bool, list: bool) -> Result<()> {
    let config = load_config()?;
    let manager = ArchiveManager::new(config);

    // If date is provided, view that date directly
    if let Some(view_date) = date {
        return view_date_archive(&manager, &view_date, summary_only, list).await;
    }

    // Otherwise, show interactive date selection
    let dates = manager.list_dates()?;

    if dates.is_empty() {
        println!("{}", "No archives found.".yellow());
        return Ok(());
    }

    // Build display items with session counts
    let items: Vec<String> = dates
        .iter()
        .map(|d| {
            let sessions = manager.list_sessions(d).unwrap_or_default();
            let count = sessions.len();
            let label = format_date_label(d);
            format!("{} {} ({} sessions)", d, label, count)
        })
        .collect();

    let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select a date to view")
        .items(&items)
        .default(0)
        .interact_opt()?;

    match selection {
        Some(idx) => {
            let view_date = &dates[idx];
            println!();
            view_date_archive(&manager, view_date, summary_only, list).await
        }
        None => {
            println!("{}", "Cancelled.".dimmed());
            Ok(())
        }
    }
}

/// View today's archive
pub async fn run_today(summary_only: bool, list: bool) -> Result<()> {
    let config = load_config()?;
    let manager = ArchiveManager::new(config);
    let today = Local::now().format("%Y-%m-%d").to_string();
    view_date_archive(&manager, &today, summary_only, list).await
}

/// View yesterday's archive
pub async fn run_yesterday(summary_only: bool, list: bool) -> Result<()> {
    let config = load_config()?;
    let manager = ArchiveManager::new(config);
    let yesterday = (Local::now() - Duration::days(1))
        .format("%Y-%m-%d")
        .to_string();
    view_date_archive(&manager, &yesterday, summary_only, list).await
}

/// Format date with relative label (today, yesterday, etc.)
fn format_date_label(date: &str) -> String {
    let today = Local::now().format("%Y-%m-%d").to_string();
    let yesterday = (Local::now() - Duration::days(1))
        .format("%Y-%m-%d")
        .to_string();

    if date == today {
        "(today)".green().to_string()
    } else if date == yesterday {
        "(yesterday)".cyan().to_string()
    } else {
        String::new()
    }
}

/// View archive for a specific date
async fn view_date_archive(
    manager: &ArchiveManager,
    date: &str,
    summary_only: bool,
    list: bool,
) -> Result<()> {
    if list {
        return list_sessions(manager, date).await;
    }

    if summary_only {
        return show_daily_summary(manager, date).await;
    }

    show_full_archive(manager, date).await
}

async fn list_sessions(manager: &ArchiveManager, date: &str) -> Result<()> {
    let sessions = manager.list_sessions(date)?;

    if sessions.is_empty() {
        println!("{}", format!("No sessions found for {}", date).yellow());
        return Ok(());
    }

    println!("{}", format!("Sessions for {}:", date).cyan().bold());
    println!();

    for (i, session) in sessions.iter().enumerate() {
        println!("  {}. {}", (i + 1).to_string().green(), session);
    }

    println!();
    println!("Total: {} sessions", sessions.len());

    Ok(())
}

async fn show_daily_summary(manager: &ArchiveManager, date: &str) -> Result<()> {
    match manager.read_daily_summary(date) {
        Ok(content) => {
            println!("{}", format!("Daily Summary - {}", date).cyan().bold());
            println!("{}", "=".repeat(50));
            println!();
            println!("{}", content);
            Ok(())
        }
        Err(_) => {
            println!(
                "{}",
                format!("No daily summary found for {}", date).yellow()
            );
            Ok(())
        }
    }
}

async fn show_full_archive(manager: &ArchiveManager, date: &str) -> Result<()> {
    // Show daily summary first
    println!("{}", format!("Daily Archive - {}", date).cyan().bold());
    println!("{}", "=".repeat(50));
    println!();

    // Try to show daily summary
    if let Ok(content) = manager.read_daily_summary(date) {
        // Extract just the overview section for brevity
        if let Some(start) = content.find("## Overview") {
            let after_header = &content[start..];
            if let Some(end) = after_header.find("\n## Sessions") {
                println!("{}", &after_header[..end]);
            } else {
                println!(
                    "{}",
                    after_header.lines().take(10).collect::<Vec<_>>().join("\n")
                );
            }
        }
    }

    println!();

    // List sessions
    let sessions = manager.list_sessions(date)?;

    if sessions.is_empty() {
        println!("{}", "No sessions archived yet.".yellow());
        return Ok(());
    }

    println!("{}", "Sessions:".green().bold());
    println!();

    for session in &sessions {
        println!("  {} {}", "●".green(), session);

        // Show brief summary if available
        if let Ok(content) = manager.read_session(date, session) {
            // Extract first line of summary
            if let Some(start) = content.find("## Summary") {
                let after_header = &content[start + 11..];
                let first_line = after_header.lines().next().unwrap_or("");
                if !first_line.is_empty() && !first_line.starts_with('#') {
                    let truncated: String = first_line.chars().take(80).collect();
                    println!("    {}", truncated.dimmed());
                }
            }
        }
    }

    println!();
    println!(
        "Use {} to see a specific session",
        "daily view --date DATE".cyan()
    );

    Ok(())
}
