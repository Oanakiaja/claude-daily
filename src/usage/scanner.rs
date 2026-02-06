use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use super::pricing::calculate_cost;
use super::types::*;

/// Scan all JSONL session files under `~/.claude/projects/`.
///
/// If `session_ids` is provided, only scan files whose filename stem matches.
/// Returns a map from session_id to SessionUsage.
pub fn scan_all_sessions(session_ids: Option<&[String]>) -> HashMap<String, SessionUsage> {
    let projects_dir = match dirs::home_dir() {
        Some(home) => home.join(".claude").join("projects"),
        None => return HashMap::new(),
    };

    if !projects_dir.exists() {
        return HashMap::new();
    }

    let jsonl_files = collect_jsonl_files(&projects_dir);
    let mut result: HashMap<String, SessionUsage> = HashMap::new();

    for path in jsonl_files {
        let session_id = match path.file_stem().and_then(|s| s.to_str()) {
            Some(stem) => stem.to_string(),
            None => continue,
        };

        // Skip if we have a filter and this session is not in it
        if let Some(ids) = session_ids {
            if !ids.iter().any(|id| id == &session_id) {
                continue;
            }
        }

        if let Some(usage) = parse_session_file(&path, &session_id) {
            result.insert(session_id, usage);
        }
    }

    result
}

/// Aggregate session usages into a global summary.
///
/// If `date_filter` is provided (as YYYY-MM-DD strings), only include sessions
/// whose first_timestamp falls on one of those dates.
pub fn aggregate_usage(
    session_usages: &HashMap<String, SessionUsage>,
    date_filter: Option<&[String]>,
) -> UsageSummary {
    let mut total_input = 0u64;
    let mut total_output = 0u64;
    let mut total_cache_creation = 0u64;
    let mut total_cache_read = 0u64;
    let mut total_cost = 0.0f64;
    let mut total_sessions = 0usize;
    let mut model_counts: HashMap<String, (usize, f64)> = HashMap::new();
    let mut daily_map: HashMap<String, DailyUsageAccum> = HashMap::new();

    for usage in session_usages.values() {
        let session_date = usage
            .first_timestamp
            .as_deref()
            .and_then(extract_date_from_timestamp);

        // Apply date filter if provided
        if let Some(dates) = date_filter {
            match &session_date {
                Some(d) => {
                    if !dates.contains(d) {
                        continue;
                    }
                }
                None => continue,
            }
        }

        total_input += usage.input_tokens;
        total_output += usage.output_tokens;
        total_cache_creation += usage.cache_creation_tokens;
        total_cache_read += usage.cache_read_tokens;
        total_cost += usage.total_cost_usd;
        total_sessions += 1;

        for (model, count) in &usage.model_calls {
            let entry = model_counts.entry(model.clone()).or_insert((0, 0.0));
            entry.0 += count;
            // Approximate per-model cost by distributing session cost proportionally
            let total_calls: usize = usage.model_calls.values().sum();
            if total_calls > 0 {
                entry.1 += usage.total_cost_usd * (*count as f64 / total_calls as f64);
            }
        }

        if let Some(date) = session_date {
            let daily = daily_map.entry(date.clone()).or_insert(DailyUsageAccum {
                date,
                input_tokens: 0,
                output_tokens: 0,
                cache_creation_tokens: 0,
                cache_read_tokens: 0,
                total_cost_usd: 0.0,
                session_count: 0,
            });
            daily.input_tokens += usage.input_tokens;
            daily.output_tokens += usage.output_tokens;
            daily.cache_creation_tokens += usage.cache_creation_tokens;
            daily.cache_read_tokens += usage.cache_read_tokens;
            daily.total_cost_usd += usage.total_cost_usd;
            daily.session_count += 1;
        }
    }

    let mut model_distribution: Vec<ModelUsageCount> = model_counts
        .into_iter()
        .map(|(model, (count, cost))| ModelUsageCount {
            model,
            count,
            total_cost_usd: cost,
        })
        .collect();
    model_distribution.sort_by(|a, b| b.count.cmp(&a.count));

    let mut daily_usage: Vec<DailyUsage> = daily_map
        .into_values()
        .map(|d| DailyUsage {
            date: d.date,
            input_tokens: d.input_tokens,
            output_tokens: d.output_tokens,
            cache_creation_tokens: d.cache_creation_tokens,
            cache_read_tokens: d.cache_read_tokens,
            total_cost_usd: d.total_cost_usd,
            session_count: d.session_count,
        })
        .collect();
    daily_usage.sort_by(|a, b| a.date.cmp(&b.date));

    UsageSummary {
        total_input_tokens: total_input,
        total_output_tokens: total_output,
        total_cache_creation_tokens: total_cache_creation,
        total_cache_read_tokens: total_cache_read,
        total_cost_usd: total_cost,
        total_sessions,
        model_distribution,
        daily_usage,
    }
}

struct DailyUsageAccum {
    date: String,
    input_tokens: u64,
    output_tokens: u64,
    cache_creation_tokens: u64,
    cache_read_tokens: u64,
    total_cost_usd: f64,
    session_count: usize,
}

/// Collect all .jsonl files recursively under a directory
fn collect_jsonl_files(dir: &PathBuf) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(collect_jsonl_files(&path));
            } else if path.extension().and_then(|e| e.to_str()) == Some("jsonl") {
                files.push(path);
            }
        }
    }
    files
}

/// Parse a single JSONL session file and extract usage data
fn parse_session_file(path: &PathBuf, session_id: &str) -> Option<SessionUsage> {
    let file = std::fs::File::open(path).ok()?;
    let reader = BufReader::new(file);

    let mut usage = SessionUsage {
        session_id: session_id.to_string(),
        ..Default::default()
    };

    let mut found_any = false;

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };

        if line.trim().is_empty() {
            continue;
        }

        // Quick filter: only parse lines that look like assistant messages with usage
        if !line.contains("\"type\":\"assistant\"") {
            continue;
        }

        let entry: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        if entry.get("type").and_then(|v| v.as_str()) != Some("assistant") {
            continue;
        }

        let message = match entry.get("message") {
            Some(m) => m,
            None => continue,
        };

        // Extract model
        if let Some(model) = message.get("model").and_then(|v| v.as_str()) {
            *usage.model_calls.entry(model.to_string()).or_insert(0) += 1;
        }

        // Extract usage tokens and calculate per-message cost
        if let Some(msg_usage) = message.get("usage") {
            let input = msg_usage
                .get("input_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let output = msg_usage
                .get("output_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let cache_creation = msg_usage
                .get("cache_creation_input_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let cache_read = msg_usage
                .get("cache_read_input_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            usage.input_tokens += input;
            usage.output_tokens += output;
            usage.cache_creation_tokens += cache_creation;
            usage.cache_read_tokens += cache_read;
            found_any = true;

            // Calculate cost per message using the actual model for this message
            let msg_model = message
                .get("model")
                .and_then(|v| v.as_str())
                .unwrap_or("claude-sonnet");
            usage.total_cost_usd +=
                calculate_cost(msg_model, input, output, cache_creation, cache_read);
        }

        // Extract first timestamp
        if usage.first_timestamp.is_none() {
            if let Some(ts) = entry.get("timestamp").and_then(|v| v.as_str()) {
                usage.first_timestamp = Some(ts.to_string());
            }
        }
    }

    if !found_any {
        return None;
    }

    Some(usage)
}

/// Extract YYYY-MM-DD date from an ISO 8601 timestamp string
fn extract_date_from_timestamp(ts: &str) -> Option<String> {
    // Handles "2026-02-05T18:48:19.274Z" format
    if ts.len() >= 10 {
        let date = &ts[..10];
        // Basic validation
        if date.chars().nth(4) == Some('-') && date.chars().nth(7) == Some('-') {
            return Some(date.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_date_from_timestamp() {
        assert_eq!(
            extract_date_from_timestamp("2026-02-05T18:48:19.274Z"),
            Some("2026-02-05".to_string())
        );
        assert_eq!(
            extract_date_from_timestamp("2026-01-15T00:00:00Z"),
            Some("2026-01-15".to_string())
        );
        assert_eq!(extract_date_from_timestamp("bad"), None);
        assert_eq!(extract_date_from_timestamp(""), None);
    }

    #[test]
    fn test_aggregate_empty() {
        let empty: HashMap<String, SessionUsage> = HashMap::new();
        let summary = aggregate_usage(&empty, None);
        assert_eq!(summary.total_sessions, 0);
        assert_eq!(summary.total_input_tokens, 0);
        assert!((summary.total_cost_usd).abs() < 0.0001);
    }

    #[test]
    fn test_aggregate_with_date_filter() {
        let mut sessions = HashMap::new();
        sessions.insert(
            "s1".to_string(),
            SessionUsage {
                session_id: "s1".to_string(),
                input_tokens: 1000,
                output_tokens: 500,
                total_cost_usd: 0.01,
                first_timestamp: Some("2026-02-05T10:00:00Z".to_string()),
                ..Default::default()
            },
        );
        sessions.insert(
            "s2".to_string(),
            SessionUsage {
                session_id: "s2".to_string(),
                input_tokens: 2000,
                output_tokens: 1000,
                total_cost_usd: 0.02,
                first_timestamp: Some("2026-02-06T10:00:00Z".to_string()),
                ..Default::default()
            },
        );

        let filter = vec!["2026-02-05".to_string()];
        let summary = aggregate_usage(&sessions, Some(&filter));
        assert_eq!(summary.total_sessions, 1);
        assert_eq!(summary.total_input_tokens, 1000);
    }
}
