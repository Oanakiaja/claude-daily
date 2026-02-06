use anyhow::Result;
use colored::Colorize;

use crate::config::load_config;
use crate::insights::collector::InsightsData;

/// Run the insights command, displaying aggregated archive and facet data
pub async fn run(days: usize) -> Result<()> {
    let config = load_config()?;

    println!(
        "\n{}",
        format!("  Daily Insights (last {} days)", days)
            .bold()
            .bright_yellow()
    );
    println!("{}", "  ─────────────────────────────".dimmed());

    let data = InsightsData::collect(&config, Some(days))?;

    // Overview stats
    println!(
        "\n  {} {} days, {} sessions",
        "Overview:".bold(),
        data.total_days.to_string().bright_yellow(),
        data.total_sessions.to_string().bright_yellow()
    );

    // Daily activity (simple bar chart)
    if !data.daily_stats.is_empty() {
        println!("\n  {}", "Activity Timeline:".bold());
        let max_count = data
            .daily_stats
            .iter()
            .map(|d| d.session_count)
            .max()
            .unwrap_or(1);
        for stat in &data.daily_stats {
            let bar_len = if max_count > 0 {
                (stat.session_count * 30) / max_count
            } else {
                0
            };
            let bar: String = "\u{2588}".repeat(bar_len);
            let digest_marker = if stat.has_digest { "\u{2713}" } else { " " };
            println!(
                "  {} {} {} {}",
                stat.date.dimmed(),
                digest_marker.green(),
                bar.bright_yellow(),
                stat.session_count.to_string().dimmed()
            );
        }
    }

    // Goal distribution
    if !data.goal_distribution.is_empty() {
        println!("\n  {}", "Goal Distribution:".bold());
        for item in &data.goal_distribution {
            println!(
                "    {} {}",
                format!("{:>20}", item.name).cyan(),
                format!("{}", item.count).dimmed()
            );
        }
    }

    // Friction points
    if !data.friction_distribution.is_empty() {
        println!("\n  {}", "Friction Points:".bold());
        for item in &data.friction_distribution {
            println!(
                "    {} {}",
                format!("{:>20}", item.name).red(),
                format!("{}", item.count).dimmed()
            );
        }
    }

    // Satisfaction
    if !data.satisfaction_distribution.is_empty() {
        println!("\n  {}", "Satisfaction:".bold());
        for item in &data.satisfaction_distribution {
            let color_name = match item.name.as_str() {
                "happy" => item.name.green(),
                "satisfied" => item.name.bright_green(),
                "neutral" => item.name.yellow(),
                "frustrated" => item.name.red(),
                _ => item.name.normal(),
            };
            println!(
                "    {:>20} {}",
                color_name,
                format!("{}", item.count).dimmed()
            );
        }
    }

    // Languages
    if !data.language_distribution.is_empty() {
        println!("\n  {}", "Languages:".bold());
        for item in data.language_distribution.iter().take(10) {
            println!(
                "    {} {}",
                format!("{:>20}", item.name).bright_blue(),
                format!("{}", item.count).dimmed()
            );
        }
    }

    println!();
    Ok(())
}
