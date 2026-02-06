use serde::Serialize;
use std::collections::HashMap;

use crate::archive::ArchiveManager;
use crate::config::Config;

use super::facets::SessionFacet;
use super::trends::TrendData;

/// Aggregated insights data from daily archives and Claude facets
#[derive(Debug, Clone, Serialize)]
pub struct InsightsData {
    pub total_days: usize,
    pub total_sessions: usize,
    pub daily_stats: Vec<DailyStat>,
    pub goal_distribution: Vec<CategoryCount>,
    pub friction_distribution: Vec<CategoryCount>,
    pub satisfaction_distribution: Vec<CategoryCount>,
    pub language_distribution: Vec<CategoryCount>,
    pub session_type_distribution: Vec<CategoryCount>,
    pub session_details: Vec<SessionInsight>,
    pub trends: Option<TrendData>,
}

/// Per-session insight combining archive metadata with facet analysis data
#[derive(Debug, Clone, Serialize)]
pub struct SessionInsight {
    pub session_id: String,
    pub date: String,
    pub session_name: String,
    pub brief_summary: Option<String>,
    pub outcome: Option<String>,
    pub goal_categories: Vec<String>,
    pub friction_types: Vec<String>,
    pub friction_detail: Option<String>,
    pub satisfaction: Option<String>,
    pub claude_helpfulness: Option<String>,
    pub session_type: Option<String>,
}

/// Statistics for a single day
#[derive(Debug, Clone, Serialize)]
pub struct DailyStat {
    pub date: String,
    pub session_count: usize,
    pub has_digest: bool,
}

/// A category name with its occurrence count
#[derive(Debug, Clone, Serialize)]
pub struct CategoryCount {
    pub name: String,
    pub count: usize,
}

impl InsightsData {
    /// Collect insights data from archives and facets.
    /// `days` limits the number of most recent days to analyze.
    pub fn collect(config: &Config, days: Option<usize>) -> anyhow::Result<Self> {
        let manager = ArchiveManager::new(config.clone());
        let all_dates = manager.list_dates()?;

        let days_limit = days.unwrap_or(30);
        let dates: Vec<String> = all_dates.into_iter().take(days_limit).collect();

        let mut daily_stats = Vec::new();
        let mut total_sessions = 0;

        for date in &dates {
            let sessions = manager.list_sessions(date).unwrap_or_default();
            let session_count = sessions.len();
            total_sessions += session_count;

            let has_digest = manager
                .read_daily_summary(date)
                .map(|content| {
                    content.contains("## Overview") && !content.contains("No sessions recorded yet")
                })
                .unwrap_or(false);

            daily_stats.push(DailyStat {
                date: date.clone(),
                session_count,
                has_digest,
            });
        }

        // Reverse so oldest first (for charts)
        daily_stats.reverse();

        // Load facets from Claude Code, indexed by session_id for fast lookup
        let facets = SessionFacet::load_all().unwrap_or_default();
        let facet_map: HashMap<String, &SessionFacet> = facets
            .iter()
            .map(|(id, facet)| (id.clone(), facet))
            .collect();

        // Aggregate goal_categories (HashMap<String, usize> per facet)
        let goal_distribution = aggregate_hashmap_field(&facets, |f| &f.goal_categories);

        // Aggregate friction_counts (HashMap<String, usize> per facet)
        let friction_distribution = aggregate_hashmap_field(&facets, |f| &f.friction_counts);

        // Aggregate user_satisfaction_counts (HashMap<String, usize> per facet)
        let satisfaction_distribution =
            aggregate_hashmap_field(&facets, |f| &f.user_satisfaction_counts);

        // Aggregate session_type (single string per facet)
        let session_type_distribution = count_option_field(&facets, |f| f.session_type.as_deref());

        // language_distribution is currently empty since facets don't carry language data
        let language_distribution = Vec::new();

        // Build per-session details by scanning archive files and matching with facets
        let mut session_details = Vec::new();
        for date in &dates {
            let sessions = manager.list_sessions(date).unwrap_or_default();
            for session_name in &sessions {
                if let Ok(content) = manager.read_session(date, session_name) {
                    if let Some(session_id) = extract_session_id_from_frontmatter(&content) {
                        let insight = if let Some(facet) = facet_map.get(&session_id) {
                            // Determine the most common satisfaction level
                            let satisfaction = facet
                                .user_satisfaction_counts
                                .iter()
                                .max_by_key(|(_, count)| *count)
                                .map(|(name, _)| name.clone());

                            SessionInsight {
                                session_id: session_id.clone(),
                                date: date.clone(),
                                session_name: session_name.clone(),
                                brief_summary: facet.brief_summary.clone(),
                                outcome: facet.outcome.clone(),
                                goal_categories: facet.goal_categories.keys().cloned().collect(),
                                friction_types: facet.friction_counts.keys().cloned().collect(),
                                friction_detail: facet.friction_detail.clone(),
                                satisfaction,
                                claude_helpfulness: facet.claude_helpfulness.clone(),
                                session_type: facet.session_type.clone(),
                            }
                        } else {
                            // No facet data available for this session
                            SessionInsight {
                                session_id: session_id.clone(),
                                date: date.clone(),
                                session_name: session_name.clone(),
                                brief_summary: None,
                                outcome: None,
                                goal_categories: Vec::new(),
                                friction_types: Vec::new(),
                                friction_detail: None,
                                satisfaction: None,
                                claude_helpfulness: None,
                                session_type: None,
                            }
                        };
                        session_details.push(insight);
                    }
                }
            }
        }

        // Calculate trend data using dates in chronological order (oldest first)
        // daily_stats is already reversed to oldest-first at this point
        let chronological_dates: Vec<String> = daily_stats.iter().map(|s| s.date.clone()).collect();
        let trends = TrendData::calculate(config, &chronological_dates, days_limit);

        Ok(InsightsData {
            total_days: dates.len(),
            total_sessions,
            daily_stats,
            goal_distribution,
            friction_distribution,
            satisfaction_distribution,
            language_distribution,
            session_type_distribution,
            session_details,
            trends,
        })
    }
}

/// Aggregate a HashMap<String, usize> field across all facets
fn aggregate_hashmap_field<F>(facets: &[(String, SessionFacet)], extractor: F) -> Vec<CategoryCount>
where
    F: Fn(&SessionFacet) -> &HashMap<String, usize>,
{
    let mut counts: HashMap<String, usize> = HashMap::new();
    for (_, facet) in facets {
        for (key, value) in extractor(facet) {
            *counts.entry(key.clone()).or_insert(0) += value;
        }
    }
    let mut result: Vec<CategoryCount> = counts
        .into_iter()
        .map(|(name, count)| CategoryCount { name, count })
        .collect();
    result.sort_by(|a, b| b.count.cmp(&a.count));
    result
}

/// Count occurrences of an Option<&str> field across all facets
fn count_option_field<F>(facets: &[(String, SessionFacet)], extractor: F) -> Vec<CategoryCount>
where
    F: Fn(&SessionFacet) -> Option<&str>,
{
    let mut counts: HashMap<String, usize> = HashMap::new();
    for (_, facet) in facets {
        if let Some(value) = extractor(facet) {
            *counts.entry(value.to_string()).or_insert(0) += 1;
        }
    }
    let mut result: Vec<CategoryCount> = counts
        .into_iter()
        .map(|(name, count)| CategoryCount { name, count })
        .collect();
    result.sort_by(|a, b| b.count.cmp(&a.count));
    result
}

/// Extract session_id from YAML frontmatter in a session archive markdown file.
/// Looks for `session_id: <value>` between `---` markers.
fn extract_session_id_from_frontmatter(content: &str) -> Option<String> {
    if let Some(stripped) = content.strip_prefix("---\n") {
        if let Some(end) = stripped.find("\n---") {
            let frontmatter = &stripped[..end];
            for line in frontmatter.lines() {
                if let Some((key, value)) = line.split_once(':') {
                    let key = key.trim();
                    if key == "session_id" {
                        let value = value.trim().trim_matches('"');
                        if !value.is_empty() {
                            return Some(value.to_string());
                        }
                    }
                }
            }
        }
    }
    None
}
