use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "daily")]
#[command(about = "Daily Context Archive System for Claude Code")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Config file path
    #[arg(short, long, global = true)]
    pub config: Option<PathBuf>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start web dashboard server
    Show {
        /// Port to listen on (default: 31456, auto-increment if occupied)
        #[arg(short, long)]
        port: Option<u16>,

        /// Host to bind to
        #[arg(long, default_value = "127.0.0.1")]
        host: String,

        /// Do not open browser automatically
        #[arg(long)]
        no_open: bool,
    },

    /// View archives (interactive date selection if no date specified)
    View {
        /// Date to view (format: yyyy-mm-dd)
        #[arg(short, long)]
        date: Option<String>,

        /// Show daily summary only
        #[arg(long)]
        summary_only: bool,

        /// List all sessions for the day
        #[arg(long)]
        list: bool,
    },

    /// View today's archive
    Today {
        /// Show daily summary only
        #[arg(long)]
        summary_only: bool,

        /// List all sessions
        #[arg(long)]
        list: bool,
    },

    /// View yesterday's archive
    Yest {
        /// Show daily summary only
        #[arg(long)]
        summary_only: bool,

        /// List all sessions
        #[arg(long)]
        list: bool,
    },

    /// Manually trigger summarization
    Summarize {
        /// Session transcript path
        #[arg(short, long)]
        transcript: PathBuf,

        /// Task name for the archive
        #[arg(short = 'n', long)]
        task_name: Option<String>,

        /// Working directory of the session
        #[arg(long)]
        cwd: Option<PathBuf>,

        /// Run in foreground (not background)
        #[arg(long)]
        foreground: bool,

        /// Job ID for tracking (internal use)
        #[arg(long)]
        job_id: Option<String>,
    },

    /// Generate daily digest from sessions (consolidate sessions into daily.md)
    Digest {
        /// Relative date (e.g., "yest" or "yesterday" for yesterday)
        #[arg(value_name = "RELATIVE_DATE")]
        relative_date: Option<String>,

        /// Date to digest (format: yyyy-mm-dd, default: today)
        #[arg(short, long)]
        date: Option<String>,

        /// Run in background (default: foreground)
        #[arg(long)]
        background: bool,

        /// Force regenerate daily summary even without session files (re-process existing daily.md)
        #[arg(short, long)]
        force: bool,
    },

    /// Extract skill from archive
    ExtractSkill {
        /// Date to search (default: today)
        #[arg(short, long)]
        date: Option<String>,

        /// Session to extract from
        #[arg(short, long)]
        session: Option<String>,

        /// Output directory for skill
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Extract command from archive
    ExtractCommand {
        /// Date to search (default: today)
        #[arg(short, long)]
        date: Option<String>,

        /// Session to extract from
        #[arg(short, long)]
        session: Option<String>,

        /// Output directory for command
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Review and manage pending skills
    ReviewSkills {
        /// Install a pending skill (format: YYYY-MM-DD/skill-name)
        #[arg(long)]
        install: Option<String>,

        /// Delete a pending skill (format: YYYY-MM-DD/skill-name)
        #[arg(long)]
        delete: Option<String>,
    },

    /// Manage background jobs
    Jobs {
        #[command(subcommand)]
        action: JobsAction,
    },

    /// Initialize configuration (interactive by default)
    Init {
        /// Storage path (default: ~/.claude/daily)
        #[arg(short, long)]
        storage_path: Option<PathBuf>,

        /// Skip interactive prompts, use defaults
        #[arg(short = 'y', long = "yes")]
        yes: bool,

        /// Use haiku model for summarization (default: sonnet)
        #[arg(long)]
        haiku: bool,
    },

    /// Show or update configuration
    Config {
        /// Set storage path
        #[arg(long)]
        set_storage: Option<PathBuf>,

        /// Show current config
        #[arg(long)]
        show: bool,

        /// Interactive configuration mode
        #[arg(short, long)]
        interactive: bool,
    },

    /// Install plugin to Claude Code
    Install {
        /// Scope: user or project
        #[arg(short, long, default_value = "user")]
        scope: String,
    },

    /// Uninstall plugin from Claude Code (removes hooks and commands only, keeps archive data)
    Uninstall {
        /// Scope: user or project
        #[arg(short, long, default_value = "user")]
        scope: String,

        /// Also delete the daily binary itself
        #[arg(long)]
        binary: bool,
    },

    /// Update daily to the latest version
    Update {
        /// Only check for updates, don't install
        #[arg(long)]
        check: bool,

        /// Install specific version (e.g., "v0.2.0" or "0.2.0")
        #[arg(long)]
        version: Option<String>,
    },

    /// Handle Claude Code hooks (internal use)
    Hook {
        #[command(subcommand)]
        hook_type: HookType,
    },
}

#[derive(Subcommand)]
pub enum JobsAction {
    /// List background jobs
    List {
        /// Show all jobs (including completed)
        #[arg(short, long)]
        all: bool,
    },

    /// Show job log
    Log {
        /// Job ID
        job_id: String,

        /// Show only last N lines
        #[arg(short, long)]
        tail: Option<usize>,

        /// Follow log output (like tail -f)
        #[arg(short, long)]
        follow: bool,
    },

    /// Kill a running job
    Kill {
        /// Job ID
        job_id: String,
    },

    /// Cleanup old jobs
    Cleanup {
        /// Keep jobs from last N days (default: 7)
        #[arg(short, long, default_value = "7")]
        days: u32,

        /// Show what would be removed without removing
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Subcommand)]
pub enum HookType {
    /// SessionStart hook handler
    SessionStart,

    /// SessionEnd hook handler
    SessionEnd,
}
