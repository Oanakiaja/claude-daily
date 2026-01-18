# Daily

![Daily](assets/logo.png)

[English](README.md) | 中文

**不只是日志 — 一个让你的上下文复利增长的系统。**

Daily 是一个用于 [Claude Code](https://claude.ai/code) 的上下文归档系统，自动记录、总结并从你的 AI 辅助工作会话中提炼可复用的技能。用得越多，你的个人上下文库就越强大。

## 为什么选择 Daily？

每次会话都蕴含宝贵的洞察 — 解决问题的模式、工作流优化和领域知识。但这些洞察会随时间消逝。Daily 自动捕获它们，并转化为**可复用的技能和命令**，随时间复利增长。

> **上下文复利（Compound Context）**：就像你专业知识的复利。每次会话都在积累你的个人知识库，让未来的会话更加强大 — 无论你是在编程、写作、研究还是学习。

## 功能特性

- **自动记录** - 通过 Hook 捕获 Claude Code 会话记录
- **智能总结** - 后台 AI 处理生成有意义的摘要
- **每日洞察** - 将所有会话聚合为可操作的每日总结
- **技能提取** - 从会话中提炼可复用的技能和命令 — 你的上下文库随时间复利增长

## 安装

### 一键安装（推荐）

```bash
curl -fsSL https://raw.githubusercontent.com/oanakiaja/claude-daily/main/scripts/install.sh | bash
```

### 从源码构建

```bash
git clone https://github.com/oanakiaja/claude-daily.git
cd claude-daily
cargo install --path .
```

## 快速开始

```bash
# 1. 初始化 Daily
daily init

# 2. 安装 Claude Code hooks
daily install

# 3. 打开 Web 仪表盘
daily show
```

## 工作原理

```mermaid
flowchart LR
    A[会话结束] --> B[触发 Hook]
    B --> C[后台任务]
    C --> D[解析记录]
    D --> E[AI 总结]
    E --> F[归档存储]
```

1. **会话结束** - Claude Code 触发 SessionEnd hook
2. **后台任务** - 生成非阻塞的后台进程进行总结
3. **AI 总结** - Claude API 处理会话记录
4. **归档存储** - 会话摘要和每日总结保存到 `~/.claude/daily/`
5. **技能沉淀** - 自动评估会话是否包含可提取的知识

## 技能沉淀（自动学习）

Daily 会自动从你的会话中识别并提取可复用的知识。会话结束时，它会通过 **"沉淀三问"** 质量门控来评估是否值得保留：

1. **踩过坑吗？** - 是否经历了调试、试错或非显而易见的发现？
2. **下次还会遇到吗？** - 这是否是一个会重复出现的问题，而不是一次性特例？
3. **能说清楚吗？** - 解决方案是否可以被清晰描述和验证？

### 工作流程

```
会话结束 → 总结 → 质量门控 → 待审核技能 → 下次会话提醒
```

1. 会话结束时，Daily 分析是否有可提取的知识
2. 通过质量门控的技能保存到 `~/.claude/daily/pending-skills/`
3. 下次会话开始时，你会看到提醒：

```
[daily] ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
[daily] 2 个待审核技能:
[daily]   • 2024-01-18/fix-econnrefused
[daily]   • 2024-01-17/next-build-optimization
[daily]
[daily] 审核命令: daily review-skills
[daily] 或让 Claude: "review my pending skills"
[daily] ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### 管理待审核技能

```bash
# 列出所有待审核技能
daily review-skills

# 安装技能到 ~/.claude/skills/（Claude 会自动使用）
daily review-skills --install 2024-01-18/fix-econnrefused

# 删除不需要的技能
daily review-skills --delete 2024-01-18/fix-econnrefused
```

安装后的技能会放置在 `~/.claude/skills/{skill-name}/SKILL.md`，Claude Code 会在检测到相关条件时自动发现并应用它们。

## 命令

| 命令                           | 描述                                                      |
| ------------------------------ | --------------------------------------------------------- |
| `daily init`                   | 初始化系统并创建存储目录                                  |
| `daily install`                | 安装 Claude Code hooks 和斜杠命令                         |
| `daily show`                   | 在浏览器中打开 Web 仪表盘（默认：http://127.0.0.1:31456） |
| `daily show --port 8080`       | 在自定义端口启动仪表盘                                    |
| `daily show --no-open`         | 启动服务但不自动打开浏览器                                |
| `daily view`                   | 查看今日归档（交互式日期选择）                            |
| `daily view --date 2024-01-15` | 查看指定日期的归档                                        |
| `daily view --list`            | 列出当天所有会话                                          |
| `daily today`                  | 查看今日归档的快捷方式                                    |
| `daily yest`                   | 查看昨日归档的快捷方式                                    |
| `daily config --show`          | 显示当前配置                                              |
| `daily extract-skill`          | 从会话中提取可复用技能                                    |
| `daily extract-command`        | 从会话中提取可复用命令                                    |
| `daily review-skills`          | 列出待审核的技能                                          |
| `daily review-skills --install`| 安装待审核技能到 ~/.claude/skills/                        |
| `daily review-skills --delete` | 删除待审核技能                                            |
| `daily jobs list`              | 列出后台任务                                              |
| `daily jobs log <id>`          | 查看任务日志                                              |

### Claude Code 斜杠命令

运行 `daily install` 后，以下命令在 Claude Code 中可用：

| 命令                 | 描述                 |
| -------------------- | -------------------- |
| `/daily-view`        | 查看每日归档         |
| `/daily-get-skill`   | 从会话洞察中提取技能 |
| `/daily-get-command` | 从会话洞察中提取命令 |

## 配置

使用 `daily config --show` 查看当前配置。

配置文件位置（macOS）：`~/Library/Application Support/rs.daily/config.toml`

主要设置：

- `storage.path` - 归档存储位置（默认：`~/.claude/daily`）
- `summarization.model` - 总结使用的 AI 模型（默认：`sonnet`）
- `hooks.enable_session_end` - 启用/禁用自动归档

## 归档结构

```
~/.claude/daily/
├── 2024-01-15/
│   ├── daily.md           # 每日总结
│   ├── fix-bug.md         # 会话归档
│   └── new-feature.md     # 会话归档
├── pending-skills/        # 自动提取的待审核技能
│   └── 2024-01-15/
│       └── fix-econnrefused.md
└── jobs/
    └── *.json, *.log      # 后台任务追踪
```

## 系统要求

- Rust 1.70+（用于构建）
- Claude Code CLI

## 许可证

MIT License - 详见 [LICENSE](LICENSE)。
