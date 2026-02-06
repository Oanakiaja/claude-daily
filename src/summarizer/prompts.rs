use chrono::Timelike;
use std::collections::HashMap;

use super::template::TemplateEngine;

/// Prompts for Claude CLI summarization
pub struct Prompts;

// Default template constants for session summary
const SESSION_SUMMARY_EN: &str = r#"You are analyzing a Claude Code session transcript. Generate a comprehensive summary in JSON format.

Context:
- Working Directory: {{cwd}}
- Git Branch: {{git_branch}}

Transcript:
{{transcript}}

Generate a JSON response with this exact structure:
```json
{
  "topic": "Short kebab-case topic for filename (2-4 words, e.g., 'fix-auth-bug', 'add-dark-mode', 'refactor-api')",
  "summary": "2-3 sentence overview including CONCRETE RESULTS (answers found, solutions implemented, code written). Never just describe the action - always include what was produced or discovered.",
  "decisions": "Key decisions made and their rationale (markdown list format)",
  "learnings": "Key learnings from this session (markdown list format)",
  "skill_hints": "Potential reusable skills (only if passes quality gate, see below)"
}
```

## Skill Quality Gate
Only suggest skills that pass ALL three criteria:
1. **Did you hit a pitfall?** Did debugging, trial-and-error, or non-obvious discovery occur?
2. **Will it happen again?** Is this a recurring problem, not a one-time edge case?
3. **Can you explain it clearly?** Can the solution be clearly described and verified?

For skill_hints format (only if quality gate passes):
```
- **[skill-name]**: [what it solves]
  - Trigger: [error message or symptom]
  - Why: [root cause]
```

If no skills pass the quality gate, set skill_hints to "None identified in this session."

Output ONLY the JSON block, no additional text."#;

const SESSION_SUMMARY_ZH: &str = r#"你正在分析一个 Claude Code 会话记录。请生成一个全面的 JSON 格式摘要。

上下文：
- 工作目录：{{cwd}}
- Git 分支：{{git_branch}}

会话记录：
{{transcript}}

生成以下结构的 JSON 响应：
```json
{
  "topic": "简短的 kebab-case 主题用于文件名（2-4个词，例如：'fix-auth-bug'、'add-dark-mode'、'refactor-api'）",
  "summary": "2-3句话概述，包含具体成果（找到的答案、实现的解决方案、编写的代码）。不要只描述动作，总是包含产出或发现。",
  "decisions": "关键决策及其理由（markdown 列表格式）",
  "learnings": "本次会话的关键收获（markdown 列表格式）",
  "skill_hints": "可复用的技能提示（仅当通过质量门禁时）"
}
```

## 技能质量门禁（沉淀三问）
只有通过全部三个标准才能提取技能：
1. **踩过坑吗？** 是否经历了调试、试错或非显而易见的发现？
2. **下次还会遇到吗？** 这是一个反复出现的问题，还是一次性边缘案例？
3. **能说清楚吗？** 解决方案能否被清晰描述和验证？

技能提示格式（仅当通过质量门禁）：
```
- **[skill-name]**: [解决什么问题]
  - 触发条件: [错误信息或症状]
  - 原因: [根本原因]
```

如果没有技能通过质量门禁，设置 skill_hints 为 "本次会话未发现可沉淀技能。"

仅输出 JSON 块，不要有其他文本。"#;

// Default template constants for skill extraction
const SKILL_EXTRACT_EN: &str = r#"You are extracting a reusable skill from a Claude Code session.

IMPORTANT: A skill is a modular package that extends Claude's capabilities with specialized knowledge, workflows, and tools. Think of it as an "onboarding guide" for specific domains or tasks.

## Quality Gate - Answer these three questions first:

1. **Did you hit a pitfall?** Was there trial-and-error, debugging, or a non-obvious discovery?
2. **Will it happen again?** Is this a recurring problem, not a one-time edge case?
3. **Can you explain it clearly?** Can the solution be clearly described and verified?

If ANY answer is NO, respond with:
```
NOT_EXTRACTABLE: [reason]
```

If ALL answers are YES, generate the skill.

## Session Summary:
{{session_content}}

Skill Hint: {{skill_hint}}

## Skill Structure

A skill consists of:
- **SKILL.md** (required): YAML frontmatter + markdown instructions
- **scripts/** (optional): Executable code for deterministic tasks
- **references/** (optional): Documentation loaded as needed
- **assets/** (optional): Files used in output (templates, etc.)

## Core Principles

1. **Concise is Key**: Only add context Claude doesn't already have. Challenge each piece of information.
2. **Set Appropriate Degrees of Freedom**:
   - High freedom (text instructions) for flexible approaches
   - Medium freedom (pseudocode) for preferred patterns
   - Low freedom (specific scripts) for fragile operations
3. **Progressive Disclosure**: Keep SKILL.md under 500 lines. Move detailed content to references/.

## Output Format:

Generate ONLY the SKILL.md content. If you identify scripts, references, or assets that would be helpful, mention them in the instructions but don't generate them.

```markdown
---
name: skill-name-kebab-case
description: "Comprehensive description that serves as the PRIMARY TRIGGERING MECHANISM. Include: (1) What the skill does, (2) Specific triggers/contexts for when to use it. Mention error messages, symptoms, or how users might describe the problem. This is the ONLY field Claude reads to determine when the skill gets used."
---

# Skill Title

[Brief overview of what this skill provides]

## [Appropriate sections based on skill type]

[Instructions and guidance for using this skill. Structure depends on what the skill does:]

For troubleshooting skills:
- Symptom description and triggers
- Diagnostic steps
- Solution workflow
- Verification steps

For workflow skills:
- When to use this workflow
- Step-by-step instructions
- Decision points and variations

For tool integration skills:
- Tool overview
- Common operations
- Examples and patterns

[If scripts would be helpful, mention: "For deterministic execution, consider adding scripts/script-name.py"]
[If detailed docs needed, mention: "For detailed reference, see references/topic.md"]
[If templates needed, mention: "Template files in assets/"]

## Examples

[Concrete examples demonstrating the skill in action]
```

CRITICAL GUIDELINES:
- description field MUST be comprehensive - it's how Claude decides to use this skill
- Keep SKILL.md focused on essential instructions, not exhaustive documentation
- Use imperative/infinitive form for instructions
- Only include information Claude doesn't already know
- Prefer concise examples over verbose explanations

Output ONLY the markdown content (or NOT_EXTRACTABLE message)."#;

const SKILL_EXTRACT_ZH: &str = r#"你正在从一个 Claude Code 会话中提取可复用的技能。

重要：技能是一个模块化包，通过提供专业知识、工作流程和工具来扩展 Claude 的能力。把它看作是特定领域或任务的"入职指南"。

## 质量门禁（沉淀三问）- 先回答这三个问题：

1. **踩过坑吗？** 是否经历了试错、调试或非显而易见的发现？
2. **下次还会遇到吗？** 这是一个反复出现的问题，还是一次性边缘案例？
3. **能说清楚吗？** 解决方案能否被清晰描述和验证？

如果任何一个答案是否定的，回复：
```
NOT_EXTRACTABLE: [原因]
```

如果所有答案都是肯定的，生成技能。

## 会话摘要：
{{session_content}}

技能提示：{{skill_hint}}

## 技能结构

一个技能包含：
- **SKILL.md**（必需）：YAML frontmatter + markdown 指令
- **scripts/**（可选）：用于确定性任务的可执行代码
- **references/**（可选）：按需加载的文档
- **assets/**（可选）：输出中使用的文件（模板等）

## 核心原则

1. **简洁是关键**：只添加 Claude 还不知道的上下文。挑战每一条信息。
2. **设置适当的自由度**：
   - 高自由度（文本指令）用于灵活方法
   - 中等自由度（伪代码）用于首选模式
   - 低自由度（具体脚本）用于脆弱操作
3. **渐进式披露**：保持 SKILL.md 在 500 行以内。将详细内容移至 references/。

## 输出格式：

仅生成 SKILL.md 内容。如果你识别出有用的 scripts、references 或 assets，在指令中提及它们但不生成它们。

```markdown
---
name: skill-name-kebab-case
description: "全面的描述，作为主要触发机制。包含：(1) 技能做什么，(2) 何时使用它的具体触发条件/上下文。提及错误消息、症状或用户可能描述问题的方式。这是 Claude 用来决定何时使用此技能的唯一字段。"
---

# 技能标题

[简要概述这个技能提供什么]

## [基于技能类型的适当章节]

[使用此技能的指令和指导。结构取决于技能的功能：]

对于故障排除技能：
- 症状描述和触发条件
- 诊断步骤
- 解决方案工作流
- 验证步骤

对于工作流技能：
- 何时使用此工作流
- 逐步指令
- 决策点和变体

对于工具集成技能：
- 工具概述
- 常见操作
- 示例和模式

[如果脚本会有帮助，提及："对于确定性执行，考虑添加 scripts/script-name.py"]
[如果需要详细文档，提及："详细参考见 references/topic.md"]
[如果需要模板，提及："模板文件在 assets/"]

## 示例

[演示技能实际应用的具体示例]
```

关键指南：
- description 字段必须全面 - 这是 Claude 决定使用此技能的方式
- 保持 SKILL.md 专注于基本指令，而不是详尽的文档
- 使用祈使句/不定式形式的指令
- 只包含 Claude 还不知道的信息
- 优先使用简洁的示例而不是冗长的解释

仅输出 markdown 内容（或 NOT_EXTRACTABLE 消息）。"#;

// Default template constants for command extraction
const COMMAND_EXTRACT_EN: &str = r#"Generate a complete slash command file for Claude Code based on this session.

Session Summary:
{{session_content}}

Command Hint: {{command_hint}}

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

Output ONLY the markdown content."#;

const COMMAND_EXTRACT_ZH: &str = r#"基于此会话生成一个完整的 Claude Code 斜杠命令文件。

会话摘要：
{{session_content}}

命令提示：{{command_hint}}

生成一个命令文件，要求：
1. 有清晰的描述
2. 解释何时使用
3. 提供 Claude 需要遵循的指令
4. 可以立即作为 /command 使用

按照以下格式输出完整的命令 markdown：
```markdown
---
description: "简要描述这个命令做什么"
---

# 命令名称

[何时使用此命令]

## 指令

[调用此命令时 Claude 需要遵循的指令]
```

仅输出 markdown 内容。"#;

// Default template constants for daily summary
const DAILY_SUMMARY_EN: &str = r#"You are analyzing Claude Code sessions from {{date}}. Generate a daily digest.

## Time Context
- Current time: {{current_time}} ({{current_period}})
- Time periods: {{periods_desc}}
{{existing_section}}
{{sessions_section}}

## Your Task

Generate a narrative digest that answers: "What did I accomplish today? What did I learn? What's next?"

### Rules

- **DO NOT** include any session names, session IDs, or timestamps like "17_48-fix-xxx" in your output
- **DO NOT** list sessions individually — group related work by theme/area
- Focus on substance: what was done, what was discovered, what was decided
- Write in a natural narrative style, not a mechanical session log

### Output Structure

1. **Overview**: 3-5 sentences describing the day's work. Mention the general time period (morning/afternoon/evening) and the main themes. This should read like a brief journal entry.

2. **Key Work**: Group all work by theme/area (e.g., "Feature Development", "Bug Fixes", "Research", "DevOps"). For each theme:
   - Brief description of what was accomplished
   - Key decisions made
   - Problems solved
   Do NOT reference individual session names.

3. **Key Insights**: Technical discoveries worth remembering:
   - Root causes found and solutions implemented
   - Patterns and connections observed
   - Non-obvious learnings

4. **Reflections**: Thoughts on work patterns, what went well, what could improve. 2-3 paragraphs.

5. **Tomorrow's Focus**: Prioritized action items:
   - Unfinished tasks
   - Problems discovered but not yet solved
   - Natural next steps

6. **Skills & Commands**: Reusable patterns that could become skills or commands (if any, otherwise say "None identified"). Only include high-quality suggestions that pass the quality gate (was there a pitfall? will it recur? can you explain it clearly?).

Output format (JSON):
```json
{
  "overview": "narrative overview paragraph",
  "session_details": "markdown: work grouped by theme, NO session names",
  "insights": "markdown list of key insights",
  "reflections": "thoughtful reflection paragraphs",
  "tomorrow_focus": "prioritized action items",
  "skills": "markdown skill suggestions (or 'None identified')",
  "commands": "markdown command suggestions (or 'None identified')"
}
```

Output ONLY the JSON block. Ensure all strings in JSON are properly escaped (especially quotes and newlines)."#;

const DAILY_SUMMARY_ZH: &str = r#"你正在分析 {{date}} 的 Claude Code 会话。生成日报。

## 时间上下文
- 当前时间：{{current_time}}（{{current_period}}）
- 时间段：{{periods_desc}}
{{existing_section}}
{{sessions_section}}

## 你的任务

生成一份叙事性日报来回答："今天做了什么？学到了什么？接下来要做什么？"

### 规则

- **禁止** 在输出中包含任何会话名称、会话 ID 或类似 "17_48-fix-xxx" 的时间戳标识
- **禁止** 逐个列出会话 — 将相关工作按主题/领域归类
- 聚焦于实质内容：做了什么、发现了什么、做了什么决策
- 用自然的叙事风格撰写，而不是机械的会话日志

### 输出结构

1. **概述**：3-5 句话描述今天的工作。提及大致的时间段（上午/下午/晚上）和主要主题。像简短的工作日记一样书写。

2. **核心工作**：将所有工作按主题/领域分组（如「功能开发」「问题修复」「技术调研」「DevOps」「架构设计」）。每个主题：
   - 简要描述完成了什么
   - 做了哪些关键决策
   - 解决了什么问题
   不要引用具体的会话名称。

3. **关键洞察**：值得记住的技术发现：
   - 找到的根本原因和实施的解决方案
   - 观察到的模式和联系
   - 非显而易见的学习收获

4. **反思**：关于工作模式、做得好的地方、可以改进的地方的思考。2-3 段。

5. **明日规划**：按优先级排列的行动项：
   - 未完成的任务
   - 发现但尚未解决的问题
   - 自然的下一步

6. **技能与命令**：可复用的模式，可以沉淀为技能或命令（如果有的话，否则说「暂未发现」）。只包含通过质量门禁的高质量建议（踩过坑吗？会复现吗？能说清楚吗？）。

输出格式（JSON）：
```json
{
  "overview": "叙事性概述段落",
  "session_details": "markdown：按主题分组的工作内容，不含会话名称",
  "insights": "markdown 格式的关键洞察列表",
  "reflections": "深思熟虑的反思段落",
  "tomorrow_focus": "按优先级排列的行动项",
  "skills": "markdown 格式的技能建议（或「暂未发现」）",
  "commands": "markdown 格式的命令建议（或「暂未发现」）"
}
```

仅输出 JSON 块。确保 JSON 中的所有字符串都正确转义（特别是引号和换行符）。"#;

impl Prompts {
    // ============================================
    // Default Template Getters
    // ============================================

    /// Get the default session summary template for a language
    pub fn default_session_summary_template(language: &str) -> &'static str {
        if language == "zh" {
            SESSION_SUMMARY_ZH
        } else {
            SESSION_SUMMARY_EN
        }
    }

    /// Get the default skill extraction template for a language
    pub fn default_skill_extract_template(language: &str) -> &'static str {
        if language == "zh" {
            SKILL_EXTRACT_ZH
        } else {
            SKILL_EXTRACT_EN
        }
    }

    /// Get the default command extraction template for a language
    pub fn default_command_extract_template(language: &str) -> &'static str {
        if language == "zh" {
            COMMAND_EXTRACT_ZH
        } else {
            COMMAND_EXTRACT_EN
        }
    }

    /// Get the default daily summary template for a language
    pub fn default_daily_summary_template(language: &str) -> &'static str {
        if language == "zh" {
            DAILY_SUMMARY_ZH
        } else {
            DAILY_SUMMARY_EN
        }
    }

    // ============================================
    // Template-based Prompt Generation
    // ============================================

    /// Generate prompt for session summarization with optional custom template
    pub fn session_summary_with_template(
        custom_template: Option<&str>,
        transcript_text: &str,
        cwd: &str,
        git_info: Option<&str>,
        language: &str,
    ) -> String {
        let git_str = git_info.unwrap_or("N/A");

        let template =
            custom_template.unwrap_or_else(|| Self::default_session_summary_template(language));

        let mut vars = HashMap::new();
        vars.insert("transcript", transcript_text);
        vars.insert("cwd", cwd);
        vars.insert("git_branch", git_str);
        vars.insert("language", language);

        TemplateEngine::render(template, &vars)
    }

    /// Generate prompt for skill extraction with optional custom template
    pub fn extract_skill_with_template(
        custom_template: Option<&str>,
        session_summary: &str,
        skill_hint: Option<&str>,
        language: &str,
    ) -> String {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let hint = if language == "zh" {
            skill_hint.unwrap_or("基于会话中的模式")
        } else {
            skill_hint.unwrap_or("Based on patterns in the session")
        };

        let template =
            custom_template.unwrap_or_else(|| Self::default_skill_extract_template(language));

        let mut vars = HashMap::new();
        vars.insert("session_content", session_summary);
        vars.insert("skill_hint", hint);
        vars.insert("today", today.as_str());
        vars.insert("language", language);

        TemplateEngine::render(template, &vars)
    }

    /// Generate prompt for command extraction with optional custom template
    pub fn extract_command_with_template(
        custom_template: Option<&str>,
        session_summary: &str,
        command_hint: Option<&str>,
        language: &str,
    ) -> String {
        let hint = if language == "zh" {
            command_hint.unwrap_or("基于会话中的模式")
        } else {
            command_hint.unwrap_or("Based on patterns in the session")
        };

        let template =
            custom_template.unwrap_or_else(|| Self::default_command_extract_template(language));

        let mut vars = HashMap::new();
        vars.insert("session_content", session_summary);
        vars.insert("command_hint", hint);
        vars.insert("language", language);

        TemplateEngine::render(template, &vars)
    }

    /// Generate prompt for daily summary with optional custom template
    pub fn daily_summary_with_template(
        custom_template: Option<&str>,
        sessions_json: &str,
        date: &str,
        existing_summary: Option<&str>,
        language: &str,
    ) -> String {
        let now = chrono::Local::now();
        let current_time = now.format("%H:%M").to_string();
        let current_hour = now.hour();

        // Determine current period for context
        let (current_period, periods_desc) = if language == "zh" {
            let period = match current_hour {
                0..=5 => "凌晨",
                6..=11 => "早上",
                12..=17 => "下午",
                _ => "晚上",
            };
            (
                period,
                "凌晨 (00:00-05:59), 早上 (06:00-11:59), 下午 (12:00-17:59), 晚上 (18:00-23:59)",
            )
        } else {
            let period = match current_hour {
                0..=5 => "early morning",
                6..=11 => "morning",
                12..=17 => "afternoon",
                _ => "evening",
            };
            (
                period,
                "early morning (00:00-05:59), morning (06:00-11:59), afternoon (12:00-17:59), evening (18:00-23:59)",
            )
        };

        // Check if this is a regenerate scenario (no new sessions but existing summary)
        let is_regenerate = sessions_json.trim() == "[]" && existing_summary.is_some();

        let existing_section = if let Some(existing) = existing_summary {
            if is_regenerate {
                if language == "zh" {
                    format!(
                        "\n## 重新生成模式\n\n你正在重新生成一个现有的日报摘要。原始内容如下。\n你的任务是用更好的结构和时间准确性来重写它，而不是添加新内容。\n\n原始 daily.md 内容：\n```\n{}\n```\n\n重新生成的重要事项：\n- 从原始内容的 Sessions 部分提取会话信息\n- 解析会话名称中的时间戳以确定实际时间段\n- 重写概述以反映实际的时间分布\n- 保留所有见解、反思和明日重点，但提高清晰度\n- 不要捏造原始内容中没有的会话或内容\n",
                        existing
                    )
                } else {
                    format!(
                        "\n## REGENERATE MODE\n\nYou are regenerating an existing daily summary. The original content is below.\nYour task is to REWRITE it with better structure and time accuracy, NOT to add new content.\n\nOriginal daily.md content:\n```\n{}\n```\n\nIMPORTANT for regeneration:\n- Extract session information from the Sessions section in the original content\n- Parse timestamps from session names to determine ACTUAL time periods\n- Rewrite the overview to reflect the ACTUAL time distribution\n- Preserve all insights, reflections, and tomorrow's focus but improve clarity\n- Do NOT fabricate sessions or content that wasn't in the original\n",
                        existing
                    )
                }
            } else if language == "zh" {
                format!(
                    "\n## 现有日报摘要（来自之前的汇总）\n\n以下内容是从今天早些时候的会话生成的。你必须完整保留现有内容，并在每个 section 追加新内容：\n\n```\n{}\n```\n\n## 追加规则（非常重要）\n\n你的任务是**追加**，而不是**重写**。对于每个 section：\n\n1. **概述（overview）**：保留现有概述的完整内容，然后追加新会话的内容。格式：\"[现有概述内容] 后来，[新会话内容描述]\"\n\n2. **会话（session_details）**：保留现有的所有会话条目，在列表末尾追加新会话。不要重新排序或删除任何现有条目。\n\n3. **见解（insights）**：保留现有的所有见解条目，在列表末尾追加新的见解。如果新见解与现有见解重复，跳过不添加。\n\n4. **技能（skills）**：保留现有的所有技能建议，追加新发现的技能。\n\n5. **命令（commands）**：保留现有的所有命令建议，追加新发现的命令。\n\n6. **反思（reflections）**：保留现有反思，追加新的反思内容。可以用段落分隔。\n\n7. **明日重点（tomorrow_focus）**：保留现有的待办项，追加新发现的待办项。如果某项已完成，在其后标注 ✅。\n\n**绝对禁止**：删除、缩减、总结或重写任何现有内容。你只能追加。\n",
                    existing
                )
            } else {
                format!(
                    "\n## Existing Daily Summary (from previous digest)\n\nThe following content was generated from earlier sessions today. You MUST preserve the existing content IN FULL and APPEND new content to each section:\n\n```\n{}\n```\n\n## Append Rules (CRITICAL)\n\nYour task is to **APPEND**, not **REWRITE**. For each section:\n\n1. **Overview**: Keep existing overview content VERBATIM, then append new session content. Format: \"[existing overview content] Later, [new session content description]\"\n\n2. **Session Details**: Keep ALL existing session entries, append new sessions at the END of the list. Do NOT reorder or remove any existing entries.\n\n3. **Insights**: Keep ALL existing insight entries, append new insights at the END. If a new insight duplicates an existing one, skip it.\n\n4. **Skills**: Keep ALL existing skill suggestions, append newly discovered skills.\n\n5. **Commands**: Keep ALL existing command suggestions, append newly discovered commands.\n\n6. **Reflections**: Keep existing reflections, append new reflection content. Use paragraph breaks to separate.\n\n7. **Tomorrow's Focus**: Keep existing TODO items, append newly discovered items. If an item was completed, mark it with ✅.\n\n**STRICTLY FORBIDDEN**: Deleting, condensing, summarizing, or rewriting ANY existing content. You may ONLY append.\n",
                    existing
                )
            }
        } else {
            String::new()
        };

        // Skip sessions section in regenerate mode since it's empty
        let sessions_section = if is_regenerate {
            String::new()
        } else {
            format!("## Sessions (JSON format):\n{}", sessions_json)
        };

        let template =
            custom_template.unwrap_or_else(|| Self::default_daily_summary_template(language));

        let mut vars = HashMap::new();
        vars.insert("date", date);
        vars.insert("current_time", current_time.as_str());
        vars.insert("current_period", current_period);
        vars.insert("periods_desc", periods_desc);
        vars.insert("existing_section", existing_section.as_str());
        vars.insert("sessions_section", sessions_section.as_str());
        vars.insert("sessions_json", sessions_json);
        vars.insert("language", language);

        TemplateEngine::render(template, &vars)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_summary_prompt_en() {
        let prompt = Prompts::session_summary_with_template(
            None,
            "User: Help me fix a bug\nAssistant: I'll help you.",
            "/home/user/project",
            Some("main"),
            "en",
        );

        assert!(prompt.contains("Working Directory: /home/user/project"));
        assert!(prompt.contains("Git Branch: main"));
    }

    #[test]
    fn test_session_summary_prompt_zh() {
        let prompt = Prompts::session_summary_with_template(
            None,
            "User: Help me fix a bug\nAssistant: I'll help you.",
            "/home/user/project",
            Some("main"),
            "zh",
        );

        assert!(prompt.contains("工作目录：/home/user/project"));
        assert!(prompt.contains("Git 分支：main"));
    }

    #[test]
    fn test_daily_summary_prompt() {
        let prompt = Prompts::daily_summary_with_template(
            None,
            r#"[{"title": "test", "summary": "test summary"}]"#,
            "2026-01-16",
            None,
            "en",
        );

        assert!(prompt.contains("2026-01-16"));
    }

    #[test]
    fn test_daily_summary_prompt_with_existing() {
        let prompt = Prompts::daily_summary_with_template(
            None,
            r#"[{"title": "new", "summary": "new summary"}]"#,
            "2026-01-16",
            Some("Previous overview content"),
            "en",
        );

        assert!(prompt.contains("2026-01-16"));
        assert!(prompt.contains("Previous overview content"));
        assert!(prompt.contains("Existing Daily Summary"));
    }

    #[test]
    fn test_daily_summary_prompt_zh() {
        let prompt = Prompts::daily_summary_with_template(
            None,
            r#"[{"title": "test", "summary": "test summary"}]"#,
            "2026-01-16",
            None,
            "zh",
        );

        assert!(prompt.contains("2026-01-16"));
        assert!(prompt.contains("时间上下文"));
    }
}
