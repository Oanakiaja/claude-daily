use serde::Serialize;
use std::collections::HashMap;

use crate::archive::ArchiveManager;
use crate::config::Config;

use super::facets::SessionFacet;

/// Per-session insight data combining archive metadata with facet analysis
#[derive(Debug, Clone, Serialize)]
pub struct SessionInsight {
    /// Session archive name (filename stem)
    pub name: String,
    /// Session ID from frontmatter
    pub session_id: String,
    /// Brief summary from facet data
    pub brief_summary: Option<String>,
    /// Outcome: achieved, partially_achieved, not_achieved
    pub outcome: Option<String>,
    /// Goal categories present in this session
    pub goal_categories: Vec<String>,
    /// Friction types encountered
    pub friction_types: Vec<String>,
    /// Friction detail description
    pub friction_detail: Option<String>,
    /// Most common satisfaction level
    pub satisfaction: Option<String>,
    /// Claude helpfulness rating
    pub claude_helpfulness: Option<String>,
}

/// Aggregated day-level insight summary
#[derive(Debug, Clone, Serialize)]
pub struct DayInsightSummary {
    /// Total sessions for the day
    pub total_sessions: usize,
    /// Number of sessions that had friction
    pub sessions_with_friction: usize,
    /// Overall satisfaction aggregated across sessions
    pub overall_satisfaction: Option<String>,
    /// Top goal categories for the day
    pub top_goals: Vec<String>,
    /// Top friction types for the day
    pub top_frictions: Vec<String>,
    /// Programmatically generated recommendations
    pub recommendations: Vec<String>,
}

/// Complete date insights response
#[derive(Debug, Clone, Serialize)]
pub struct DateInsights {
    /// Per-session insight details
    pub sessions: Vec<SessionInsight>,
    /// Aggregated day-level summary
    pub day_summary: DayInsightSummary,
}

impl DateInsights {
    /// Collect insights for a specific date by matching session archives with facet data
    pub fn collect(date: &str, config: &Config) -> anyhow::Result<Self> {
        let manager = ArchiveManager::new(config.clone());
        let session_names = manager.list_sessions(date).unwrap_or_default();

        // Load all facets and index by session_id
        let all_facets = SessionFacet::load_all().unwrap_or_default();
        let facet_map: HashMap<String, SessionFacet> = all_facets.into_iter().collect();

        let mut sessions: Vec<SessionInsight> = Vec::new();
        let mut day_goal_counts: HashMap<String, usize> = HashMap::new();
        let mut day_friction_counts: HashMap<String, usize> = HashMap::new();
        let mut day_satisfaction_counts: HashMap<String, usize> = HashMap::new();
        let mut day_outcome_counts: HashMap<String, usize> = HashMap::new();
        let mut sessions_with_friction = 0;

        for name in &session_names {
            // Read session content and extract session_id from frontmatter
            let content = match manager.read_session(date, name) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let session_id = match extract_session_id(&content) {
                Some(id) => id,
                None => continue,
            };

            // Look up facet data for this session
            let facet = facet_map.get(&session_id);

            let (
                goal_categories,
                friction_types,
                friction_detail,
                satisfaction,
                outcome,
                brief_summary,
                claude_helpfulness,
            ) = if let Some(f) = facet {
                // Aggregate goals
                let goals: Vec<String> = f.goal_categories.keys().cloned().collect();
                for g in &goals {
                    *day_goal_counts.entry(g.clone()).or_insert(0) += 1;
                }

                // Aggregate frictions
                let frictions: Vec<String> = f.friction_counts.keys().cloned().collect();
                for fr in &frictions {
                    *day_friction_counts.entry(fr.clone()).or_insert(0) +=
                        f.friction_counts.get(fr).copied().unwrap_or(0);
                }
                if !frictions.is_empty() {
                    sessions_with_friction += 1;
                }

                // Aggregate satisfaction
                for (k, v) in &f.user_satisfaction_counts {
                    *day_satisfaction_counts.entry(k.clone()).or_insert(0) += v;
                }

                // Aggregate outcomes
                if let Some(ref o) = f.outcome {
                    *day_outcome_counts.entry(o.clone()).or_insert(0) += 1;
                }

                // Determine most common satisfaction for this session
                let session_satisfaction = most_common_key(&f.user_satisfaction_counts);

                (
                    goals,
                    frictions,
                    f.friction_detail.clone(),
                    session_satisfaction,
                    f.outcome.clone(),
                    f.brief_summary.clone(),
                    f.claude_helpfulness.clone(),
                )
            } else {
                (Vec::new(), Vec::new(), None, None, None, None, None)
            };

            sessions.push(SessionInsight {
                name: name.clone(),
                session_id,
                brief_summary,
                outcome,
                goal_categories,
                friction_types,
                friction_detail,
                satisfaction,
                claude_helpfulness,
            });
        }

        // Compute day-level aggregates
        let overall_satisfaction = most_common_key(&day_satisfaction_counts);

        let top_goals = top_n_keys(&day_goal_counts, 5);
        let top_frictions = top_n_keys(&day_friction_counts, 5);

        // Generate recommendations based on patterns
        let recommendations = generate_recommendations(
            &day_friction_counts,
            &day_outcome_counts,
            &day_satisfaction_counts,
            sessions_with_friction,
            session_names.len(),
        );

        let day_summary = DayInsightSummary {
            total_sessions: session_names.len(),
            sessions_with_friction,
            overall_satisfaction,
            top_goals,
            top_frictions,
            recommendations,
        };

        Ok(DateInsights {
            sessions,
            day_summary,
        })
    }
}

/// Extract session_id from YAML frontmatter in session markdown content
fn extract_session_id(content: &str) -> Option<String> {
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

/// Find the most common key in a HashMap<String, usize>
fn most_common_key(counts: &HashMap<String, usize>) -> Option<String> {
    counts
        .iter()
        .max_by_key(|(_, v)| *v)
        .map(|(k, _)| k.clone())
}

/// Return the top N keys sorted by count descending
fn top_n_keys(counts: &HashMap<String, usize>, n: usize) -> Vec<String> {
    let mut entries: Vec<(&String, &usize)> = counts.iter().collect();
    entries.sort_by(|a, b| b.1.cmp(a.1));
    entries
        .into_iter()
        .take(n)
        .map(|(k, _)| k.clone())
        .collect()
}

/// Generate actionable recommendations based on day-level patterns
fn generate_recommendations(
    friction_counts: &HashMap<String, usize>,
    outcome_counts: &HashMap<String, usize>,
    satisfaction_counts: &HashMap<String, usize>,
    sessions_with_friction: usize,
    total_sessions: usize,
) -> Vec<String> {
    let mut recs = Vec::new();

    // Check friction patterns
    if let Some(count) = friction_counts.get("misunderstood_request") {
        if *count >= 2 {
            recs.push(
                "Try to be more specific in your initial prompts — several requests were misunderstood today."
                    .to_string(),
            );
        }
    }

    if let Some(count) = friction_counts.get("user_rejected_action") {
        if *count >= 2 {
            recs.push(
                "Review Claude's suggestions more carefully before accepting — multiple actions were rejected."
                    .to_string(),
            );
        }
    }

    if let Some(count) = friction_counts.get("required_multiple_attempts") {
        if *count >= 2 {
            recs.push(
                "Consider providing more context upfront to reduce back-and-forth iterations."
                    .to_string(),
            );
        }
    }

    if let Some(count) = friction_counts.get("wrong_tool_used") {
        if *count >= 1 {
            recs.push(
                "Guide Claude toward the right tools by specifying file paths or tool preferences in your prompt."
                    .to_string(),
            );
        }
    }

    // Check outcome patterns
    let not_achieved = outcome_counts.get("not_achieved").copied().unwrap_or(0);
    let partially = outcome_counts
        .get("partially_achieved")
        .copied()
        .unwrap_or(0);
    if not_achieved >= 2 || (not_achieved + partially > total_sessions / 2 && total_sessions > 0) {
        recs.push(
            "Consider breaking complex tasks into smaller, more focused steps for better outcomes."
                .to_string(),
        );
    }

    // Check friction ratio
    if total_sessions > 0 && sessions_with_friction > total_sessions / 2 {
        recs.push(
            "More than half of today's sessions had friction — consider reviewing your prompting patterns."
                .to_string(),
        );
    }

    // Positive feedback when things go well
    let total_satisfaction: usize = satisfaction_counts.values().sum();
    let happy = satisfaction_counts.get("happy").copied().unwrap_or(0);
    let likely_satisfied = satisfaction_counts
        .get("likely_satisfied")
        .copied()
        .unwrap_or(0);
    if total_satisfaction > 0 && (happy + likely_satisfied) as f64 / total_satisfaction as f64 > 0.7
    {
        recs.push("Great collaboration today! Satisfaction levels are high.".to_string());
    }

    let achieved = outcome_counts.get("achieved").copied().unwrap_or(0);
    let total_outcomes: usize = outcome_counts.values().sum();
    if total_outcomes > 0 && achieved as f64 / total_outcomes as f64 > 0.8 {
        recs.push(
            "Most goals were achieved — your prompting strategy is working well!".to_string(),
        );
    }

    // If no recommendations were generated, provide a neutral one
    if recs.is_empty() && total_sessions > 0 {
        recs.push("Session data available but no strong patterns detected today.".to_string());
    }

    recs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_session_id() {
        let content =
            "---\ntitle: Test Session\nsession_id: abc-123-def\ndate: 2026-01-20\n---\n# Content";
        assert_eq!(extract_session_id(content), Some("abc-123-def".to_string()));
    }

    #[test]
    fn test_extract_session_id_missing() {
        let content = "---\ntitle: Test Session\ndate: 2026-01-20\n---\n# Content";
        assert_eq!(extract_session_id(content), None);
    }

    #[test]
    fn test_most_common_key() {
        let mut counts = HashMap::new();
        counts.insert("happy".to_string(), 3);
        counts.insert("frustrated".to_string(), 1);
        assert_eq!(most_common_key(&counts), Some("happy".to_string()));
    }

    #[test]
    fn test_top_n_keys() {
        let mut counts = HashMap::new();
        counts.insert("debugging".to_string(), 5);
        counts.insert("feature".to_string(), 3);
        counts.insert("refactoring".to_string(), 1);
        let top = top_n_keys(&counts, 2);
        assert_eq!(top, vec!["debugging".to_string(), "feature".to_string()]);
    }

    #[test]
    fn test_generate_recommendations_friction() {
        let mut friction = HashMap::new();
        friction.insert("misunderstood_request".to_string(), 3);
        let outcomes = HashMap::new();
        let satisfaction = HashMap::new();
        let recs = generate_recommendations(&friction, &outcomes, &satisfaction, 2, 3);
        assert!(recs.iter().any(|r| r.contains("more specific")));
    }

    #[test]
    fn test_generate_recommendations_positive() {
        let friction = HashMap::new();
        let mut outcomes = HashMap::new();
        outcomes.insert("achieved".to_string(), 5);
        let mut satisfaction = HashMap::new();
        satisfaction.insert("happy".to_string(), 4);
        satisfaction.insert("likely_satisfied".to_string(), 1);
        let recs = generate_recommendations(&friction, &outcomes, &satisfaction, 0, 5);
        assert!(recs.iter().any(|r| r.contains("Great collaboration")));
    }
}
