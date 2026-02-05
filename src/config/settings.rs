use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const APP_NAME: &str = "daily";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub storage: StorageConfig,
    pub archive: ArchiveConfig,
    pub summarization: SummarizationConfig,
    pub hooks: HooksConfig,
    pub output: OutputConfig,
    /// Custom prompt templates (None = use built-in defaults)
    #[serde(default)]
    pub prompt_templates: PromptTemplatesConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StorageConfig {
    pub path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ArchiveConfig {
    pub author: Option<String>,
    pub tags: Vec<String>,
    pub include_cwd: bool,
    pub include_git_info: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SummarizationConfig {
    pub model: String,
    pub max_tokens: u32,
    pub enable_daily_summary: bool,
    pub enable_extraction_hints: bool,
    /// Time to auto-digest previous day's sessions (format: "HH:MM", default: "06:00")
    #[serde(default = "default_digest_time")]
    pub digest_time: String,
    /// Enable auto-digest of previous day's sessions on session start
    #[serde(default = "default_auto_digest")]
    pub auto_digest_enabled: bool,
    /// Language for summary output ("en" for English, "zh" for Chinese)
    #[serde(default = "default_summary_language")]
    pub summary_language: String,
    /// Enable auto-summarization of unsummarized sessions on daily show
    #[serde(default = "default_auto_summarize_enabled")]
    pub auto_summarize_enabled: bool,
    /// Time to trigger auto-summarization (format: "HH:MM", default: "06:00")
    #[serde(default = "default_auto_summarize_time")]
    pub auto_summarize_time: String,
    /// Last time auto-summarization check was performed (ISO 8601 format)
    #[serde(default)]
    pub last_auto_summarize_check: Option<String>,
    /// Trigger auto-summarization every time `daily show` is opened (ignores time-based trigger)
    #[serde(default = "default_auto_summarize_on_show")]
    pub auto_summarize_on_show: bool,
    /// Minutes of inactivity before a transcript is considered "inactive" and eligible for auto-summarization
    #[serde(default = "default_auto_summarize_inactive_minutes")]
    pub auto_summarize_inactive_minutes: u64,
}

fn default_summary_language() -> String {
    "en".into()
}

fn default_digest_time() -> String {
    "06:00".into()
}

fn default_auto_digest() -> bool {
    true
}

fn default_auto_summarize_enabled() -> bool {
    false // Disabled by default to prevent fork bomb until transcript tracking is fixed
}

fn default_auto_summarize_time() -> String {
    "06:00".into()
}

fn default_auto_summarize_on_show() -> bool {
    false // Disabled by default, user must opt-in
}

fn default_auto_summarize_inactive_minutes() -> u64 {
    30 // 30 minutes of inactivity before considering a session ended
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HooksConfig {
    pub enable_session_start: bool,
    pub enable_session_end: bool,
    pub background_timeout: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OutputConfig {
    pub terminal_format: String,
    pub date_format: String,
    pub time_format: String,
}

/// Custom prompt templates configuration
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct PromptTemplatesConfig {
    /// Custom session summary template (None = use default)
    #[serde(default)]
    pub session_summary: Option<String>,

    /// Custom daily summary template (None = use default)
    #[serde(default)]
    pub daily_summary: Option<String>,

    /// Custom skill extraction template (None = use default)
    #[serde(default)]
    pub skill_extract: Option<String>,

    /// Custom command extraction template (None = use default)
    #[serde(default)]
    pub command_extract: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        let default_path = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".claude")
            .join("daily");

        Self {
            storage: StorageConfig { path: default_path },
            archive: ArchiveConfig {
                author: None,
                tags: vec!["claude-code".into(), "daily-archive".into()],
                include_cwd: true,
                include_git_info: true,
            },
            summarization: SummarizationConfig {
                model: "sonnet".into(),
                max_tokens: 4096,
                enable_daily_summary: true,
                enable_extraction_hints: true,
                digest_time: "06:00".into(),
                auto_digest_enabled: true,
                summary_language: "en".into(),
                auto_summarize_enabled: true,
                auto_summarize_time: "06:00".into(),
                last_auto_summarize_check: None,
                auto_summarize_on_show: false,
                auto_summarize_inactive_minutes: 30,
            },
            hooks: HooksConfig {
                enable_session_start: true,
                enable_session_end: true,
                background_timeout: 300,
            },
            output: OutputConfig {
                terminal_format: "colored".into(),
                date_format: "%Y-%m-%d".into(),
                time_format: "%H:%M:%S".into(),
            },
            prompt_templates: PromptTemplatesConfig::default(),
        }
    }
}

impl Config {
    /// Get the storage path, expanding ~ if present
    pub fn storage_path(&self) -> PathBuf {
        let path_str = self.storage.path.to_string_lossy();
        if path_str.starts_with("~") {
            if let Some(home) = dirs::home_dir() {
                return home.join(path_str.trim_start_matches("~/"));
            }
        }
        self.storage.path.clone()
    }

    /// Get today's archive directory
    pub fn today_dir(&self) -> PathBuf {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        self.storage_path().join(today)
    }

    /// Get archive directory for a specific date
    pub fn date_dir(&self, date: &str) -> PathBuf {
        self.storage_path().join(date)
    }
}

/// Load configuration from file or create default
pub fn load_config() -> Result<Config> {
    let config: Config =
        confy::load(APP_NAME, Some("config")).context("Failed to load configuration")?;
    Ok(config)
}

/// Save configuration to file
pub fn save_config(config: &Config) -> Result<()> {
    confy::store(APP_NAME, Some("config"), config).context("Failed to save configuration")?;
    Ok(())
}

/// Get the configuration file path
pub fn get_config_path() -> Result<PathBuf> {
    let path = confy::get_configuration_file_path(APP_NAME, Some("config"))
        .context("Failed to get configuration path")?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config
            .storage
            .path
            .to_string_lossy()
            .contains(".claude/daily"));
        assert_eq!(config.summarization.model, "sonnet");
    }

    #[test]
    fn test_today_dir() {
        let config = Config::default();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        assert!(config.today_dir().to_string_lossy().contains(&today));
    }
}
