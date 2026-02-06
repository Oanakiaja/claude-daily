use serde::Serialize;
use std::collections::HashMap;

/// Token usage data for a single session
#[derive(Debug, Clone, Serialize, Default)]
pub struct SessionUsage {
    pub session_id: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_tokens: u64,
    pub cache_read_tokens: u64,
    pub total_cost_usd: f64,
    /// Model name -> call count
    pub model_calls: HashMap<String, usize>,
    /// Earliest timestamp seen in this session
    pub first_timestamp: Option<String>,
}

/// Aggregated usage for a single day
#[derive(Debug, Clone, Serialize)]
pub struct DailyUsage {
    pub date: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_tokens: u64,
    pub cache_read_tokens: u64,
    pub total_cost_usd: f64,
    pub session_count: usize,
}

/// Model usage distribution entry
#[derive(Debug, Clone, Serialize)]
pub struct ModelUsageCount {
    pub model: String,
    pub count: usize,
    pub total_cost_usd: f64,
}

/// Global usage summary across all sessions
#[derive(Debug, Clone, Serialize)]
pub struct UsageSummary {
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_cache_creation_tokens: u64,
    pub total_cache_read_tokens: u64,
    pub total_cost_usd: f64,
    pub total_sessions: usize,
    pub model_distribution: Vec<ModelUsageCount>,
    pub daily_usage: Vec<DailyUsage>,
}
