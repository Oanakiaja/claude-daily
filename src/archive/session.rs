use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::manager::ArchiveManager;
use super::templates::Templates;
use crate::config::Config;
use crate::transcript::TranscriptData;

/// Represents a summarized session ready for archiving
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionArchive {
    pub title: String,
    pub date: String,
    pub session_id: String,
    pub cwd: String,
    pub git_branch: Option<String>,
    pub summary: String,
    pub decisions: String,
    pub code_changes: String,
    pub learnings: String,
    pub skill_hints: String,
}

impl SessionArchive {
    /// Create a new session archive from raw data
    pub fn new(title: String, date: String, session_id: String, cwd: String) -> Self {
        Self {
            title,
            date,
            session_id,
            cwd,
            git_branch: None,
            summary: String::new(),
            decisions: String::new(),
            code_changes: String::new(),
            learnings: String::new(),
            skill_hints: String::new(),
        }
    }

    /// Fill in data from transcript
    pub fn with_transcript_data(mut self, data: &TranscriptData) -> Self {
        // Build code changes from files modified
        if !data.files_modified.is_empty() {
            self.code_changes = data
                .files_modified
                .iter()
                .map(|f| format!("- `{}`", f))
                .collect::<Vec<_>>()
                .join("\n");
        } else {
            self.code_changes = "_No files modified._".to_string();
        }

        self
    }

    /// Fill in summary data from AI analysis
    pub fn with_summary(
        mut self,
        summary: String,
        decisions: String,
        learnings: String,
        skill_hints: String,
    ) -> Self {
        self.summary = summary;
        self.decisions = decisions;
        self.learnings = learnings;
        self.skill_hints = skill_hints;
        self
    }

    /// Generate Markdown content for this archive
    pub fn to_markdown(&self) -> String {
        Templates::session_archive(
            &self.title,
            &self.date,
            &self.session_id,
            &self.cwd,
            self.git_branch.as_deref(),
            &self.summary,
            &self.decisions,
            &self.code_changes,
            &self.learnings,
            &self.skill_hints,
        )
    }

    /// Save this archive to disk
    pub fn save(&self, config: &Config) -> Result<std::path::PathBuf> {
        let manager = ArchiveManager::new(config.clone());
        let content = self.to_markdown();
        manager.write_session(&self.date, &self.title, &content)
    }
}

/// Get git branch from working directory
pub fn get_git_branch(cwd: &str) -> Option<String> {
    std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(cwd)
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
            } else {
                None
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_archive_new() {
        let archive = SessionArchive::new(
            "test-session".to_string(),
            "2026-01-16".to_string(),
            "abc123".to_string(),
            "/home/user/project".to_string(),
        );

        assert_eq!(archive.title, "test-session");
    }

    #[test]
    fn test_session_archive_to_markdown() {
        let archive = SessionArchive::new(
            "test-session".to_string(),
            "2026-01-16".to_string(),
            "abc123".to_string(),
            "/home/user/project".to_string(),
        );

        let md = archive.to_markdown();
        assert!(md.contains("title: \"test-session\""));
        assert!(md.contains("# test-session"));
    }
}
