use anyhow::{Context, Result};
use serde::Deserialize;
use std::io::Write;
use std::process::{Command, Stdio};

use super::prompts::Prompts;
use crate::archive::{ArchiveManager, DailySummary, SessionArchive};
use crate::config::Config;
use crate::transcript::TranscriptParser;

/// Response structure from session summarization
#[derive(Debug, Deserialize)]
struct SessionSummaryResponse {
    summary: String,
    decisions: String,
    learnings: String,
    skill_hints: String,
}

/// Response structure from daily summarization
#[derive(Debug, Deserialize)]
struct DailySummaryResponse {
    overview: String,
    session_details: String,
    insights: String,
    skills: String,
    commands: String,
    reflections: String,
    tomorrow_focus: String,
}

/// Engine for summarizing transcripts using Claude CLI
pub struct SummarizerEngine {
    config: Config,
}

impl SummarizerEngine {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Invoke Claude CLI with a prompt and return the response
    fn invoke_claude(&self, prompt: &str) -> Result<String> {
        let mut child = Command::new("claude")
            .args([
                "--model",
                &self.config.summarization.model,
                "--print", // Print response and exit
                "-p",      // Prompt mode
                // Disable hooks to prevent infinite loop (daily hooks -> claude -> daily hooks -> ...)
                "--settings",
                r#"{"hooks":{}}"#,
                // Disable session persistence to avoid generating transcripts for internal calls
                "--no-session-persistence",
                // Disable MCP to avoid file watcher errors in non-interactive mode
                "--strict-mcp-config",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to spawn claude CLI. Is it installed?")?;

        // Write prompt to stdin
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(prompt.as_bytes())
                .context("Failed to write prompt to claude")?;
        }

        let output = child
            .wait_with_output()
            .context("Failed to wait for claude")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Claude CLI failed: {}", stderr);
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Extract JSON from Claude's response (handles markdown code blocks)
    fn extract_json(&self, response: &str) -> Result<String> {
        // Try to find JSON in code block first
        if let Some(start) = response.find("```json") {
            if let Some(end) = response[start..]
                .find("```\n")
                .or(response[start..].rfind("```"))
            {
                let json_start = start + 7; // Skip ```json
                let json_end = start + end;
                if json_end > json_start {
                    return Ok(response[json_start..json_end].trim().to_string());
                }
            }
        }

        // Try to find raw JSON object
        if let Some(start) = response.find('{') {
            if let Some(end) = response.rfind('}') {
                if end > start {
                    return Ok(response[start..=end].to_string());
                }
            }
        }

        // Return as-is if no JSON found
        Ok(response.to_string())
    }

    /// Summarize a session transcript and create archive
    pub async fn summarize_session(
        &self,
        transcript_path: &std::path::Path,
        task_name: &str,
        cwd: &str,
    ) -> Result<SessionArchive> {
        // Parse transcript
        let transcript_data = TranscriptParser::parse(transcript_path)?;
        let transcript_text = TranscriptParser::to_condensed_text(&transcript_data);

        // Get git branch
        let git_branch = crate::archive::session::get_git_branch(cwd);

        // Build prompt and invoke Claude
        let prompt = Prompts::session_summary(&transcript_text, cwd, git_branch.as_deref());

        let response = self.invoke_claude(&prompt)?;
        let json_str = self.extract_json(&response)?;

        // Parse response
        let summary_response: SessionSummaryResponse =
            serde_json::from_str(&json_str).context("Failed to parse summary response")?;

        // Build archive
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let session_id = transcript_path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let archive =
            SessionArchive::new(task_name.to_string(), today, session_id, cwd.to_string())
                .with_transcript_data(&transcript_data)
                .with_summary(
                    summary_response.summary,
                    summary_response.decisions,
                    summary_response.learnings,
                    summary_response.skill_hints,
                );

        // Set git branch
        let mut archive = archive;
        archive.git_branch = git_branch;

        Ok(archive)
    }

    /// Update daily summary with all sessions
    pub async fn update_daily_summary(&self, date: &str) -> Result<DailySummary> {
        let manager = ArchiveManager::new(self.config.clone());

        // Get all sessions for this date
        let sessions = manager.list_sessions(date)?;

        if sessions.is_empty() {
            // Return empty summary
            return Ok(DailySummary::new(date.to_string()));
        }

        // Collect session summaries
        let mut session_data = Vec::new();
        for session_name in &sessions {
            if let Ok(content) = manager.read_session(date, session_name) {
                // Extract summary from markdown (simplified extraction)
                let summary = extract_summary_from_markdown(&content);
                session_data.push(serde_json::json!({
                    "name": session_name,
                    "content": summary
                }));
            }
        }

        let sessions_json = serde_json::to_string_pretty(&session_data)?;

        // Build prompt and invoke Claude
        let prompt = Prompts::daily_summary(&sessions_json, date);
        let response = self.invoke_claude(&prompt)?;
        let json_str = self.extract_json(&response)?;

        // Parse response
        let daily_response: DailySummaryResponse =
            serde_json::from_str(&json_str).context("Failed to parse daily summary response")?;

        // Build daily summary
        let mut summary = DailySummary::new(date.to_string());
        summary.sessions = sessions;
        summary = summary.with_content(
            daily_response.overview,
            daily_response.session_details,
            daily_response.insights,
            daily_response.skills,
            daily_response.commands,
            daily_response.reflections,
            daily_response.tomorrow_focus,
        );

        Ok(summary)
    }

    /// Extract skill from session
    pub async fn extract_skill(&self, session_content: &str, hint: Option<&str>) -> Result<String> {
        let prompt = Prompts::extract_skill(session_content, hint);
        let response = self.invoke_claude(&prompt)?;

        // Extract markdown from response
        extract_markdown_from_response(&response)
    }

    /// Extract command from session
    pub async fn extract_command(
        &self,
        session_content: &str,
        hint: Option<&str>,
    ) -> Result<String> {
        let prompt = Prompts::extract_command(session_content, hint);
        let response = self.invoke_claude(&prompt)?;

        // Extract markdown from response
        extract_markdown_from_response(&response)
    }
}

/// Extract summary section from session markdown
fn extract_summary_from_markdown(content: &str) -> String {
    // Look for ## Summary section
    if let Some(start) = content.find("## Summary") {
        let after_header = &content[start + 10..];
        if let Some(end) = after_header.find("\n## ") {
            return after_header[..end].trim().to_string();
        }
        // Return rest if no next section
        return after_header.trim().to_string();
    }

    // Return truncated content if no summary section
    content.chars().take(500).collect()
}

/// Extract markdown content from Claude response
fn extract_markdown_from_response(response: &str) -> Result<String> {
    // Try to find markdown in code block
    if let Some(start) = response.find("```markdown") {
        let after_start = &response[start + 11..];
        if let Some(end) = after_start.find("```") {
            return Ok(after_start[..end].trim().to_string());
        }
    }

    // Try generic code block
    if let Some(start) = response.find("```") {
        let after_start = &response[start + 3..];
        // Skip language identifier if present
        let content_start = after_start.find('\n').unwrap_or(0) + 1;
        let after_lang = &after_start[content_start..];
        if let Some(end) = after_lang.find("```") {
            return Ok(after_lang[..end].trim().to_string());
        }
    }

    // Return as-is
    Ok(response.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_summary_from_markdown() {
        let content = r#"# Test

## Summary

This is the summary.

## Next Section

More content.
"#;
        let summary = extract_summary_from_markdown(content);
        assert!(summary.contains("This is the summary"));
    }

    #[test]
    fn test_extract_markdown_from_response() {
        let response = r#"Here is the skill:

```markdown
---
name: test-skill
---
# Test
```

Done!"#;
        let md = extract_markdown_from_response(response).unwrap();
        assert!(md.contains("name: test-skill"));
    }
}
