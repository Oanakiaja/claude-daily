/// Prompts for Claude CLI summarization
pub struct Prompts;

impl Prompts {
    /// Generate prompt for session summarization
    pub fn session_summary(transcript_text: &str, cwd: &str, git_info: Option<&str>) -> String {
        let git_str = git_info.unwrap_or("N/A");

        format!(
            r#"You are analyzing a Claude Code session transcript. Generate a comprehensive summary in JSON format.

Context:
- Working Directory: {cwd}
- Git Branch: {git_str}

Transcript:
{transcript_text}

Generate a JSON response with this exact structure:
```json
{{
  "summary": "2-3 sentence overview including CONCRETE RESULTS (answers found, solutions implemented, code written). Never just describe the action - always include what was produced or discovered.",
  "decisions": "Key decisions made and their rationale (markdown list format)",
  "learnings": "Key learnings from this session (markdown list format)",
  "skill_hints": "Potential reusable skills (only if passes quality gate, see below)"
}}
```

## Skill Quality Gate (沉淀三问)
Only suggest skills that pass ALL three criteria:
1. **踩过坑吗？** Did debugging, trial-and-error, or non-obvious discovery occur?
2. **下次还会遇到吗？** Is this a recurring problem, not a one-time edge case?
3. **能说清楚吗？** Can the solution be clearly described and verified?

For skill_hints format (only if quality gate passes):
```
- **[skill-name]**: [what it solves]
  - Trigger: [error message or symptom]
  - Why: [root cause]
```

If no skills pass the quality gate, set skill_hints to "None identified in this session."

Output ONLY the JSON block, no additional text."#
        )
    }

    /// Generate prompt for daily summary
    pub fn daily_summary(sessions_json: &str, date: &str) -> String {
        format!(
            r#"You are analyzing all Claude Code sessions from {date}. Generate a comprehensive daily summary.

Sessions completed today (JSON format):
{sessions_json}

Generate a thoughtful daily summary with these sections:

1. **Overview**: 2-3 sentence synthesis of the day's work
2. **Session Details**: Brief description of each session's outcome
3. **Key Insights**: Deep learnings that connect multiple sessions or represent important discoveries
4. **Skills**: Potential skills to extract (name, description, when to use)
5. **Commands**: Potential commands to extract (name, description, use case)
6. **Reflections**: Thoughts on work patterns, productivity, challenges
7. **Tomorrow's Focus**: Based on incomplete tasks or emerging priorities

Output format (JSON):
```json
{{
  "overview": "...",
  "session_details": "markdown formatted list of sessions",
  "insights": "markdown list of insights",
  "skills": "markdown formatted skill suggestions",
  "commands": "markdown formatted command suggestions",
  "reflections": "thoughtful reflection paragraph",
  "tomorrow_focus": "suggestions for tomorrow"
}}
```

Output ONLY the JSON block."#
        )
    }

    /// Generate prompt for skill extraction
    pub fn extract_skill(session_summary: &str, skill_hint: Option<&str>) -> String {
        let hint = skill_hint.unwrap_or("Based on patterns in the session");
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();

        format!(
            r#"You are extracting a reusable skill from a Claude Code session.

## Quality Gate - Answer these three questions first:

1. **踩过坑吗？** Was there trial-and-error, debugging, or a non-obvious discovery?
2. **下次还会遇到吗？** Is this a recurring problem, not a one-time edge case?
3. **能说清楚吗？** Can the solution be clearly described and verified?

If ANY answer is NO, respond with:
```
NOT_EXTRACTABLE: [reason]
```

If ALL answers are YES, generate the skill.

## Session Summary:
{session_summary}

Skill Hint: {hint}

## Output Format:

```markdown
---
name: skill-name-kebab-case
description: "Retrieval-optimized: include error messages, symptoms, or how user might describe the problem. Max 100 tokens."
origin: "{today}/session-name"
confidence: verified
---

# Skill Name

Brief description of what this skill solves.

## When to Use

Trigger this skill when you encounter:
- [Exact error message or symptom, e.g., "ECONNREFUSED on port 3000"]
- [How user might describe it, e.g., "my dev server won't start"]
- [Related scenarios]

## Root Cause

Why does this problem happen? Understanding the cause prevents future issues.

## Solution

Step-by-step resolution:

1. [First step]
2. [Second step]
...

## Verification

How to confirm the problem is solved:
- [Check command or expected output]
```

Output ONLY the markdown content (or NOT_EXTRACTABLE message)."#,
            today = today
        )
    }

    /// Generate prompt for command extraction
    pub fn extract_command(session_summary: &str, command_hint: Option<&str>) -> String {
        let hint = command_hint.unwrap_or("Based on patterns in the session");

        format!(
            r#"Generate a complete slash command file for Claude Code based on this session.

Session Summary:
{session_summary}

Command Hint: {hint}

Generate a command file that:
1. Has a clear description
2. Explains when to use it
3. Provides instructions for Claude to follow
4. Is immediately usable as a /command

Output the complete command markdown following this format:
```markdown
---
description: "Brief description of what this command does"
---

# Command Name

[When to use this command]

## Instructions

[Instructions for Claude to follow when this command is invoked]
```

Output ONLY the markdown content."#
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_summary_prompt() {
        let prompt = Prompts::session_summary(
            "User: Help me fix a bug\nAssistant: I'll help you.",
            "/home/user/project",
            Some("main"),
        );

        assert!(prompt.contains("Working Directory: /home/user/project"));
        assert!(prompt.contains("Git Branch: main"));
    }

    #[test]
    fn test_daily_summary_prompt() {
        let prompt = Prompts::daily_summary(
            r#"[{"title": "test", "summary": "test summary"}]"#,
            "2026-01-16",
        );

        assert!(prompt.contains("2026-01-16"));
    }
}
