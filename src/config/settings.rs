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
}

fn default_digest_time() -> String {
    "06:00".into()
}

fn default_auto_digest() -> bool {
    true
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
                model: "haiku".into(),
                max_tokens: 4096,
                enable_daily_summary: true,
                enable_extraction_hints: true,
                digest_time: "06:00".into(),
                auto_digest_enabled: true,
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
        assert_eq!(config.summarization.model, "haiku");
    }

    #[test]
    fn test_today_dir() {
        let config = Config::default();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        assert!(config.today_dir().to_string_lossy().contains(&today));
    }
}
