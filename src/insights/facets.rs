use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents facet data for a single Claude Code session.
/// Loaded from JSON files in ~/.claude/usage-data/facets/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionFacet {
    /// Brief summary of the session
    #[serde(default)]
    pub brief_summary: Option<String>,
    /// The underlying goal of the session
    #[serde(default)]
    pub underlying_goal: Option<String>,
    /// Goal categories with counts: e.g. {"debugging": 1, "feature_improvement": 1}
    #[serde(default)]
    pub goal_categories: HashMap<String, usize>,
    /// Session outcome: "achieved", "partially_achieved", "not_achieved"
    #[serde(default)]
    pub outcome: Option<String>,
    /// User satisfaction counts: e.g. {"happy": 1, "likely_satisfied": 2}
    #[serde(default)]
    pub user_satisfaction_counts: HashMap<String, usize>,
    /// Claude helpfulness: "very_helpful", "slightly_helpful", etc.
    #[serde(default)]
    pub claude_helpfulness: Option<String>,
    /// Session type: "single_task", "multi_task", "iterative_refinement", etc.
    #[serde(default)]
    pub session_type: Option<String>,
    /// Friction counts: e.g. {"misunderstood_request": 2}
    #[serde(default)]
    pub friction_counts: HashMap<String, usize>,
    /// Friction detail description
    #[serde(default)]
    pub friction_detail: Option<String>,
    /// Primary success type
    #[serde(default)]
    pub primary_success: Option<String>,
    /// Session ID
    #[serde(default)]
    pub session_id: Option<String>,
}

impl SessionFacet {
    /// Load all facets from the default Claude Code facets directory
    pub fn load_all() -> anyhow::Result<Vec<(String, Self)>> {
        let facets_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?
            .join(".claude/usage-data/facets");

        if !facets_dir.exists() {
            return Ok(Vec::new());
        }

        let mut facets = Vec::new();
        for entry in std::fs::read_dir(&facets_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "json") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    // Try to parse - skip invalid files
                    if let Ok(facet) = serde_json::from_str::<SessionFacet>(&content) {
                        let session_id = path
                            .file_stem()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();
                        facets.push((session_id, facet));
                    }
                }
            }
        }
        Ok(facets)
    }
}
