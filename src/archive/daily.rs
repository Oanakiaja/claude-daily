use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::manager::ArchiveManager;
use super::templates::Templates;
use crate::config::Config;

/// A single card within a daily summary section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryCard {
    pub title: String,
    pub content: String,
}

/// Represents a daily summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailySummary {
    pub date: String,
    pub sessions: Vec<String>,
    pub overview: String,
    pub session_details: String,
    pub insights: Vec<SummaryCard>,
    pub skills: Vec<SummaryCard>,
    pub commands: Vec<SummaryCard>,
    pub reflections: String,
    pub tomorrow_focus: Vec<SummaryCard>,
}

impl DailySummary {
    /// Create a new daily summary for a date
    pub fn new(date: String) -> Self {
        Self {
            date,
            sessions: Vec::new(),
            overview: "_No overview yet._".to_string(),
            session_details: String::new(),
            insights: Vec::new(),
            skills: Vec::new(),
            commands: Vec::new(),
            reflections: String::new(),
            tomorrow_focus: Vec::new(),
        }
    }

    /// Add a session to the summary
    #[allow(dead_code)]
    pub fn add_session(&mut self, session_name: &str) {
        if !self.sessions.contains(&session_name.to_string()) {
            self.sessions.push(session_name.to_string());
        }
    }

    /// Update summary content from AI analysis
    #[allow(clippy::too_many_arguments)]
    pub fn with_content(
        mut self,
        overview: String,
        session_details: String,
        insights: Vec<SummaryCard>,
        skills: Vec<SummaryCard>,
        commands: Vec<SummaryCard>,
        reflections: String,
        tomorrow_focus: Vec<SummaryCard>,
    ) -> Self {
        self.overview = overview;
        self.session_details = session_details;
        self.insights = insights;
        self.skills = skills;
        self.commands = commands;
        self.reflections = reflections;
        self.tomorrow_focus = tomorrow_focus;
        self
    }

    /// Generate Markdown content for this summary
    pub fn to_markdown(&self) -> String {
        Templates::daily_summary(
            &self.date,
            self.sessions.len(),
            &self.overview,
            &self.session_details,
            &self.insights,
            &self.skills,
            &self.commands,
            &self.reflections,
            &self.tomorrow_focus,
        )
    }

    /// Render a slice of SummaryCards as markdown subsections
    #[allow(dead_code)]
    pub fn cards_to_markdown(cards: &[SummaryCard]) -> String {
        cards
            .iter()
            .map(|card| format!("### {}\n\n{}", card.title, card.content))
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    /// Save this summary to disk
    pub fn save(&self, config: &Config) -> Result<std::path::PathBuf> {
        let manager = ArchiveManager::new(config.clone());
        let content = self.to_markdown();
        manager.write_daily_summary(&self.date, &content)
    }

    /// Load daily summary from disk, or create new if not exists
    #[allow(dead_code)]
    pub fn load_or_create(config: &Config, date: &str) -> Result<Self> {
        let manager = ArchiveManager::new(config.clone());

        // Get list of sessions for this date
        let sessions = manager.list_sessions(date)?;

        // Try to read existing summary
        match manager.read_daily_summary(date) {
            Ok(_content) => {
                // Parse frontmatter to extract existing data
                // For now, just create new with sessions
                let mut summary = Self::new(date.to_string());
                summary.sessions = sessions;
                Ok(summary)
            }
            Err(_) => {
                // Create new summary
                let mut summary = Self::new(date.to_string());
                summary.sessions = sessions;
                Ok(summary)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_daily_summary_new() {
        let summary = DailySummary::new("2026-01-16".to_string());
        assert_eq!(summary.date, "2026-01-16");
        assert!(summary.sessions.is_empty());
    }

    #[test]
    fn test_daily_summary_add_session() {
        let mut summary = DailySummary::new("2026-01-16".to_string());
        summary.add_session("test-session");
        summary.add_session("test-session"); // Duplicate should not be added
        assert_eq!(summary.sessions.len(), 1);
    }

    #[test]
    fn test_daily_summary_to_markdown() {
        let mut summary = DailySummary::new("2026-01-16".to_string());
        summary.add_session("test-session");

        let md = summary.to_markdown();
        assert!(md.contains("date: 2026-01-16"));
        assert!(md.contains("session_count: 1"));
    }
}
