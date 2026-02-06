use serde::Serialize;
use std::collections::HashMap;

use crate::archive::ArchiveManager;
use crate::config::Config;

use super::facets::SessionFacet;

/// Trend data for period-over-period comparison
#[derive(Debug, Clone, Serialize)]
pub struct TrendData {
    pub period_label: String,
    pub comparison_label: String,

    pub current_sessions: usize,
    pub previous_sessions: usize,
    pub sessions_change_pct: f64,

    pub current_friction_rate: f64,
    pub previous_friction_rate: f64,
    pub friction_change_pct: f64,

    pub current_success_rate: f64,
    pub previous_success_rate: f64,
    pub success_change_pct: f64,

    pub current_satisfaction_score: f64,
    pub previous_satisfaction_score: f64,
    pub satisfaction_change_pct: f64,

    pub weekly_stats: Vec<WeeklyStat>,
}

/// Statistics for a single week
#[derive(Debug, Clone, Serialize)]
pub struct WeeklyStat {
    pub week_label: String,
    pub session_count: usize,
    pub friction_rate: f64,
    pub success_rate: f64,
}

/// Facet data matched to a specific date
struct DatedFacet {
    date: String,
    facet: SessionFacet,
}

impl TrendData {
    /// Calculate trend data by splitting the date range into two halves and comparing metrics.
    ///
    /// The `dates` should be sorted oldest-first (chronological order) and `daily_session_counts`
    /// maps date -> session count. Facets are loaded globally and matched to dates via session_id
    /// found in session archive frontmatter.
    pub fn calculate(config: &Config, dates: &[String], days: usize) -> Option<Self> {
        if dates.len() < 2 {
            return None;
        }

        let manager = ArchiveManager::new(config.clone());

        // Load all facets indexed by session_id
        let all_facets = SessionFacet::load_all().unwrap_or_default();
        let facet_map: HashMap<String, SessionFacet> = all_facets.into_iter().collect();

        // Build a mapping: date -> Vec<SessionFacet> by reading session frontmatter
        let mut date_facets: Vec<DatedFacet> = Vec::new();
        let mut date_session_counts: HashMap<String, usize> = HashMap::new();

        for date in dates {
            let sessions = manager.list_sessions(date).unwrap_or_default();
            date_session_counts.insert(date.clone(), sessions.len());

            for session_name in &sessions {
                if let Ok(content) = manager.read_session(date, session_name) {
                    if let Some(session_id) = extract_session_id_from_frontmatter(&content) {
                        if let Some(facet) = facet_map.get(&session_id) {
                            date_facets.push(DatedFacet {
                                date: date.clone(),
                                facet: facet.clone(),
                            });
                        }
                    }
                }
            }
        }

        // Split dates into two halves: first half = previous, second half = current
        let mid = dates.len() / 2;
        let previous_dates = &dates[..mid];
        let current_dates = &dates[mid..];

        if previous_dates.is_empty() || current_dates.is_empty() {
            return None;
        }

        // Compute session counts per period
        let previous_sessions: usize = previous_dates
            .iter()
            .map(|d| date_session_counts.get(d).copied().unwrap_or(0))
            .sum();
        let current_sessions: usize = current_dates
            .iter()
            .map(|d| date_session_counts.get(d).copied().unwrap_or(0))
            .sum();

        // Partition facets into periods
        let previous_facets: Vec<&SessionFacet> = date_facets
            .iter()
            .filter(|df| previous_dates.contains(&df.date))
            .map(|df| &df.facet)
            .collect();
        let current_facets: Vec<&SessionFacet> = date_facets
            .iter()
            .filter(|df| current_dates.contains(&df.date))
            .map(|df| &df.facet)
            .collect();

        // Calculate metrics for each period
        let previous_friction_rate = calc_friction_rate(&previous_facets);
        let current_friction_rate = calc_friction_rate(&current_facets);

        let previous_success_rate = calc_success_rate(&previous_facets);
        let current_success_rate = calc_success_rate(&current_facets);

        let previous_satisfaction_score = calc_satisfaction_score(&previous_facets);
        let current_satisfaction_score = calc_satisfaction_score(&current_facets);

        // Calculate percentage changes
        let sessions_change_pct = pct_change(previous_sessions as f64, current_sessions as f64);
        let friction_change_pct = pct_change(previous_friction_rate, current_friction_rate);
        let success_change_pct = pct_change(previous_success_rate, current_success_rate);
        let satisfaction_change_pct =
            pct_change(previous_satisfaction_score, current_satisfaction_score);

        // Calculate weekly breakdown
        let weekly_stats = calc_weekly_stats(dates, &date_session_counts, &date_facets);

        // Build period labels
        let half_days = days / 2;
        let period_label = format!("Last {} days", half_days);
        let comparison_label = format!("vs previous {} days", half_days);

        Some(TrendData {
            period_label,
            comparison_label,
            current_sessions,
            previous_sessions,
            sessions_change_pct,
            current_friction_rate,
            previous_friction_rate,
            friction_change_pct,
            current_success_rate,
            previous_success_rate,
            success_change_pct,
            current_satisfaction_score,
            previous_satisfaction_score,
            satisfaction_change_pct,
            weekly_stats,
        })
    }
}

/// Extract session_id from YAML frontmatter in session archive markdown
fn extract_session_id_from_frontmatter(content: &str) -> Option<String> {
    if let Some(stripped) = content.strip_prefix("---\n") {
        if let Some(end) = stripped.find("\n---") {
            let frontmatter = &stripped[..end];
            for line in frontmatter.lines() {
                if let Some((key, value)) = line.split_once(':') {
                    let key = key.trim();
                    if key == "session_id" {
                        let value = value.trim().trim_matches('"');
                        if !value.is_empty() && value != "N/A" {
                            return Some(value.to_string());
                        }
                    }
                }
            }
        }
    }
    None
}

/// Calculate friction rate: fraction of sessions that have any friction counts
fn calc_friction_rate(facets: &[&SessionFacet]) -> f64 {
    if facets.is_empty() {
        return 0.0;
    }
    let with_friction = facets
        .iter()
        .filter(|f| {
            let total: usize = f.friction_counts.values().sum();
            total > 0
        })
        .count();
    (with_friction as f64 / facets.len() as f64) * 100.0
}

/// Calculate success rate: fraction of sessions with "achieved" or "partially_achieved" outcome
fn calc_success_rate(facets: &[&SessionFacet]) -> f64 {
    if facets.is_empty() {
        return 0.0;
    }
    let successful = facets
        .iter()
        .filter(|f| {
            matches!(
                f.outcome.as_deref(),
                Some("achieved") | Some("partially_achieved")
            )
        })
        .count();
    (successful as f64 / facets.len() as f64) * 100.0
}

/// Calculate weighted satisfaction score (0-100):
/// happy=100, likely_satisfied=75, neutral=50, frustrated=25
fn calc_satisfaction_score(facets: &[&SessionFacet]) -> f64 {
    let mut total_weight = 0.0;
    let mut total_count = 0usize;

    for facet in facets {
        for (key, &count) in &facet.user_satisfaction_counts {
            let weight = match key.as_str() {
                "happy" => 100.0,
                "likely_satisfied" => 75.0,
                "neutral" => 50.0,
                "frustrated" => 25.0,
                _ => 50.0, // default for unknown categories
            };
            total_weight += weight * count as f64;
            total_count += count;
        }
    }

    if total_count == 0 {
        return 0.0;
    }
    total_weight / total_count as f64
}

/// Calculate percentage change between previous and current values.
/// Returns 0.0 if the previous value is zero.
fn pct_change(previous: f64, current: f64) -> f64 {
    if previous.abs() < f64::EPSILON {
        if current.abs() < f64::EPSILON {
            0.0
        } else {
            100.0 // went from 0 to something
        }
    } else {
        ((current - previous) / previous) * 100.0
    }
}

/// Build weekly breakdown statistics from dates
fn calc_weekly_stats(
    dates: &[String],
    date_session_counts: &HashMap<String, usize>,
    date_facets: &[DatedFacet],
) -> Vec<WeeklyStat> {
    if dates.is_empty() {
        return Vec::new();
    }

    // Parse dates and group by ISO week
    let mut weeks: Vec<(String, Vec<String>)> = Vec::new();
    let mut current_week_label = String::new();
    let mut current_week_dates: Vec<String> = Vec::new();

    for date_str in dates {
        let parsed = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d");
        if let Ok(date) = parsed {
            let week_label = format_week_label(date);
            if week_label != current_week_label {
                if !current_week_dates.is_empty() {
                    weeks.push((current_week_label.clone(), current_week_dates.clone()));
                    current_week_dates.clear();
                }
                current_week_label = week_label;
            }
            current_week_dates.push(date_str.clone());
        }
    }
    // Push final week
    if !current_week_dates.is_empty() {
        weeks.push((current_week_label, current_week_dates));
    }

    weeks
        .into_iter()
        .map(|(week_label, week_dates)| {
            let session_count: usize = week_dates
                .iter()
                .map(|d| date_session_counts.get(d).copied().unwrap_or(0))
                .sum();

            // Gather facets for this week
            let week_facets: Vec<&SessionFacet> = date_facets
                .iter()
                .filter(|df| week_dates.contains(&df.date))
                .map(|df| &df.facet)
                .collect();

            let friction_rate = calc_friction_rate(&week_facets);
            let success_rate = calc_success_rate(&week_facets);

            WeeklyStat {
                week_label,
                session_count,
                friction_rate,
                success_rate,
            }
        })
        .collect()
}

/// Format a week label like "Jan 19-25"
fn format_week_label(date: chrono::NaiveDate) -> String {
    use chrono::{Datelike, Duration};

    // Find the Monday of this week
    let weekday = date.weekday().num_days_from_monday();
    let monday = date - Duration::days(weekday as i64);
    let sunday = monday + Duration::days(6);

    let month = monday.format("%b").to_string();
    format!("{} {}-{}", month, monday.day(), sunday.day())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pct_change_zero_base() {
        assert_eq!(pct_change(0.0, 0.0), 0.0);
        assert_eq!(pct_change(0.0, 50.0), 100.0);
    }

    #[test]
    fn test_pct_change_normal() {
        let result = pct_change(50.0, 75.0);
        assert!((result - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_pct_change_decrease() {
        let result = pct_change(100.0, 80.0);
        assert!((result - (-20.0)).abs() < 0.001);
    }

    #[test]
    fn test_format_week_label() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 1, 22).unwrap();
        let label = format_week_label(date);
        assert_eq!(label, "Jan 19-25");
    }

    #[test]
    fn test_calc_friction_rate_empty() {
        let facets: Vec<&SessionFacet> = vec![];
        assert_eq!(calc_friction_rate(&facets), 0.0);
    }

    #[test]
    fn test_calc_success_rate_empty() {
        let facets: Vec<&SessionFacet> = vec![];
        assert_eq!(calc_success_rate(&facets), 0.0);
    }

    #[test]
    fn test_calc_satisfaction_score_empty() {
        let facets: Vec<&SessionFacet> = vec![];
        assert_eq!(calc_satisfaction_score(&facets), 0.0);
    }

    #[test]
    fn test_extract_session_id() {
        let content = "---\ntitle: \"test\"\ndate: 2026-01-31\nsession_id: abc-123\n---\n# Test";
        assert_eq!(
            extract_session_id_from_frontmatter(content),
            Some("abc-123".to_string())
        );
    }

    #[test]
    fn test_extract_session_id_missing() {
        let content = "---\ntitle: \"test\"\ndate: 2026-01-31\n---\n# Test";
        assert_eq!(extract_session_id_from_frontmatter(content), None);
    }
}
