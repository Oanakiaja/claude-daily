use anyhow::{Context, Result};
use serde::Deserialize;
use std::io::{self, Read};
use std::path::PathBuf;

/// Input data received from Claude Code hooks via stdin
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct HookInput {
    pub session_id: String,
    pub transcript_path: PathBuf,
    pub cwd: PathBuf,
    pub hook_event_name: String,
    #[serde(default)]
    pub reason: Option<String>, // Only for SessionEnd: prompt_input_exit, logout, clear, other
    #[serde(default)]
    pub permission_mode: Option<String>,
}

/// Read hook input JSON from stdin
pub fn read_hook_input() -> Result<HookInput> {
    let mut buffer = String::new();
    io::stdin()
        .read_to_string(&mut buffer)
        .context("Failed to read from stdin")?;

    // Handle empty input gracefully
    if buffer.trim().is_empty() {
        anyhow::bail!("No input received from stdin");
    }

    let input: HookInput =
        serde_json::from_str(&buffer).context("Failed to parse hook input JSON")?;

    Ok(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_session_start_input() {
        let json = r#"{
            "session_id": "abc123",
            "transcript_path": "/home/user/.claude/projects/xyz/session.jsonl",
            "cwd": "/home/user/project",
            "hook_event_name": "SessionStart"
        }"#;

        let input: HookInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.session_id, "abc123");
        assert_eq!(input.hook_event_name, "SessionStart");
        assert!(input.reason.is_none());
    }

    #[test]
    fn test_parse_session_end_input() {
        let json = r#"{
            "session_id": "abc123",
            "transcript_path": "/home/user/.claude/projects/xyz/session.jsonl",
            "cwd": "/home/user/project",
            "hook_event_name": "SessionEnd",
            "reason": "user_exit"
        }"#;

        let input: HookInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.reason, Some("user_exit".to_string()));
    }
}
