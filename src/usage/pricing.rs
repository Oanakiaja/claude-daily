use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

const LITELLM_PRICING_URL: &str =
    "https://raw.githubusercontent.com/BerriAI/litellm/main/model_prices_and_context_window.json";

const FETCH_TIMEOUT_SECS: u64 = 15;

/// Token threshold for tiered pricing in 1M context window models.
/// Claude models charge higher rates for tokens above this threshold.
const TIERED_THRESHOLD: u64 = 200_000;

const PROVIDER_PREFIXES: &[&str] = &[
    "anthropic/",
    "claude-3-5-",
    "claude-3-",
    "claude-",
    "openai/",
    "azure/",
    "openrouter/openai/",
];

/// Per-model pricing data from LiteLLM dataset.
/// All costs are per individual token (e.g. 3e-6 = $3 per million tokens).
#[derive(Debug, Clone, Deserialize)]
pub struct ModelPricing {
    pub input_cost_per_token: Option<f64>,
    pub output_cost_per_token: Option<f64>,
    pub cache_creation_input_token_cost: Option<f64>,
    pub cache_read_input_token_cost: Option<f64>,
    // Tiered pricing for 1M context window models (200k threshold)
    pub input_cost_per_token_above_200k_tokens: Option<f64>,
    pub output_cost_per_token_above_200k_tokens: Option<f64>,
    pub cache_creation_input_token_cost_above_200k_tokens: Option<f64>,
    pub cache_read_input_token_cost_above_200k_tokens: Option<f64>,
}

/// Loaded pricing data for all models.
pub struct PricingData {
    models: HashMap<String, ModelPricing>,
}

impl PricingData {
    /// Load pricing data: fetch from URL → file cache → hardcoded fallback.
    pub async fn load() -> Self {
        // Try fetching from LiteLLM
        match Self::fetch_from_url().await {
            Ok(data) => {
                // Save to file cache for offline use (best-effort)
                let _ = Self::save_cache(&data);
                return data;
            }
            Err(e) => {
                eprintln!(
                    "[daily] Failed to fetch pricing from LiteLLM: {}, trying cache...",
                    e
                );
            }
        }

        // Try loading from file cache
        match Self::load_cache() {
            Ok(data) => {
                eprintln!("[daily] Using cached pricing data");
                return data;
            }
            Err(_) => {
                eprintln!("[daily] No cached pricing, using hardcoded fallback");
            }
        }

        // Ultimate fallback: embedded LiteLLM snapshot
        Self::embedded_fallback()
    }

    /// Fetch pricing data from LiteLLM GitHub URL
    async fn fetch_from_url() -> anyhow::Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(FETCH_TIMEOUT_SECS))
            .build()?;

        let response = client.get(LITELLM_PRICING_URL).send().await?;

        if !response.status().is_success() {
            anyhow::bail!("HTTP {}", response.status());
        }

        let raw: HashMap<String, serde_json::Value> = response.json().await?;
        let models = Self::parse_raw_data(raw);

        Ok(PricingData { models })
    }

    /// Parse raw JSON data into typed ModelPricing, skipping entries that fail
    fn parse_raw_data(raw: HashMap<String, serde_json::Value>) -> HashMap<String, ModelPricing> {
        let mut models = HashMap::new();
        for (name, value) in raw {
            if let Ok(pricing) = serde_json::from_value::<ModelPricing>(value) {
                // Only keep entries that have at least one cost field
                if pricing.input_cost_per_token.is_some() || pricing.output_cost_per_token.is_some()
                {
                    models.insert(name, pricing);
                }
            }
        }
        models
    }

    /// Cache file path: ~/.config/daily/pricing_cache.json
    fn cache_path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("daily").join("pricing_cache.json"))
    }

    /// Save pricing data to file cache
    fn save_cache(data: &PricingData) -> anyhow::Result<()> {
        let path = Self::cache_path().ok_or_else(|| anyhow::anyhow!("No config dir"))?;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let serializable: HashMap<&String, &ModelPricing> = data.models.iter().collect();
        let json = serde_json::to_string(&serializable)?;
        std::fs::write(&path, json)?;

        Ok(())
    }

    /// Load pricing data from file cache
    fn load_cache() -> anyhow::Result<PricingData> {
        let path = Self::cache_path().ok_or_else(|| anyhow::anyhow!("No config dir"))?;
        let json = std::fs::read_to_string(&path)?;
        let models: HashMap<String, ModelPricing> = serde_json::from_str(&json)?;
        Ok(PricingData { models })
    }

    /// Embedded fallback pricing from LiteLLM snapshot (compile-time embedded).
    /// This covers all Claude/Anthropic models without network access.
    fn embedded_fallback() -> Self {
        let json_data = include_str!("litellm_pricing.json");
        match serde_json::from_str::<HashMap<String, ModelPricing>>(json_data) {
            Ok(models) => PricingData { models },
            Err(_) => PricingData {
                models: HashMap::new(),
            },
        }
    }

    /// Create PricingData from pre-built HashMap (for testing)
    #[cfg(test)]
    pub fn from_map(models: HashMap<String, ModelPricing>) -> Self {
        PricingData { models }
    }

    /// Look up pricing for a model name, trying provider prefix candidates and fuzzy match.
    pub fn get_model_pricing(&self, model_name: &str) -> Option<&ModelPricing> {
        // 1. Direct match
        if let Some(pricing) = self.models.get(model_name) {
            return Some(pricing);
        }

        // 2. Try with provider prefixes
        for prefix in PROVIDER_PREFIXES {
            let candidate = format!("{}{}", prefix, model_name);
            if let Some(pricing) = self.models.get(&candidate) {
                return Some(pricing);
            }
        }

        // 3. Fuzzy match: case-insensitive substring
        let lower = model_name.to_lowercase();
        for (key, value) in &self.models {
            let key_lower = key.to_lowercase();
            if key_lower.contains(&lower) || lower.contains(&key_lower) {
                return Some(value);
            }
        }

        None
    }

    /// Calculate the total cost for token usage with tiered pricing support.
    ///
    /// Looks up model pricing, then applies tiered pricing for tokens
    /// above the 200k threshold when applicable.
    /// Returns 0.0 if model pricing is not found.
    pub fn calculate_cost(
        &self,
        model: &str,
        input_tokens: u64,
        output_tokens: u64,
        cache_creation_tokens: u64,
        cache_read_tokens: u64,
    ) -> f64 {
        let pricing = match self.get_model_pricing(model) {
            Some(p) => p,
            None => return 0.0,
        };

        let input_cost = tiered_cost(
            input_tokens,
            pricing.input_cost_per_token,
            pricing.input_cost_per_token_above_200k_tokens,
        );

        let output_cost = tiered_cost(
            output_tokens,
            pricing.output_cost_per_token,
            pricing.output_cost_per_token_above_200k_tokens,
        );

        let cache_creation_cost = tiered_cost(
            cache_creation_tokens,
            pricing.cache_creation_input_token_cost,
            pricing.cache_creation_input_token_cost_above_200k_tokens,
        );

        let cache_read_cost = tiered_cost(
            cache_read_tokens,
            pricing.cache_read_input_token_cost,
            pricing.cache_read_input_token_cost_above_200k_tokens,
        );

        input_cost + output_cost + cache_creation_cost + cache_read_cost
    }
}

/// Calculate cost with tiered pricing.
///
/// If tokens exceed the 200k threshold and a tiered price exists,
/// tokens below the threshold use base_price and tokens above use tiered_price.
fn tiered_cost(tokens: u64, base_price: Option<f64>, tiered_price: Option<f64>) -> f64 {
    if tokens == 0 {
        return 0.0;
    }

    if tokens > TIERED_THRESHOLD {
        if let Some(tp) = tiered_price {
            let below = TIERED_THRESHOLD as f64 * base_price.unwrap_or(0.0);
            let above = (tokens - TIERED_THRESHOLD) as f64 * tp;
            return below + above;
        }
    }

    tokens as f64 * base_price.unwrap_or(0.0)
}

/// Serialize ModelPricing for cache file
impl serde::Serialize for ModelPricing {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("ModelPricing", 8)?;
        state.serialize_field("input_cost_per_token", &self.input_cost_per_token)?;
        state.serialize_field("output_cost_per_token", &self.output_cost_per_token)?;
        state.serialize_field(
            "cache_creation_input_token_cost",
            &self.cache_creation_input_token_cost,
        )?;
        state.serialize_field(
            "cache_read_input_token_cost",
            &self.cache_read_input_token_cost,
        )?;
        state.serialize_field(
            "input_cost_per_token_above_200k_tokens",
            &self.input_cost_per_token_above_200k_tokens,
        )?;
        state.serialize_field(
            "output_cost_per_token_above_200k_tokens",
            &self.output_cost_per_token_above_200k_tokens,
        )?;
        state.serialize_field(
            "cache_creation_input_token_cost_above_200k_tokens",
            &self.cache_creation_input_token_cost_above_200k_tokens,
        )?;
        state.serialize_field(
            "cache_read_input_token_cost_above_200k_tokens",
            &self.cache_read_input_token_cost_above_200k_tokens,
        )?;
        state.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sonnet_pricing() -> PricingData {
        let mut models = HashMap::new();
        models.insert(
            "claude-sonnet-4-5-20250929".to_string(),
            ModelPricing {
                input_cost_per_token: Some(3e-6),
                output_cost_per_token: Some(15e-6),
                cache_creation_input_token_cost: Some(3.75e-6),
                cache_read_input_token_cost: Some(0.30e-6),
                input_cost_per_token_above_200k_tokens: None,
                output_cost_per_token_above_200k_tokens: None,
                cache_creation_input_token_cost_above_200k_tokens: None,
                cache_read_input_token_cost_above_200k_tokens: None,
            },
        );
        PricingData::from_map(models)
    }

    fn tiered_pricing() -> PricingData {
        let mut models = HashMap::new();
        models.insert(
            "anthropic/claude-sonnet-4-5-20250929".to_string(),
            ModelPricing {
                input_cost_per_token: Some(3e-6),
                output_cost_per_token: Some(15e-6),
                cache_creation_input_token_cost: Some(3.75e-6),
                cache_read_input_token_cost: Some(0.30e-6),
                input_cost_per_token_above_200k_tokens: Some(6e-6),
                output_cost_per_token_above_200k_tokens: Some(22.5e-6),
                cache_creation_input_token_cost_above_200k_tokens: Some(7.5e-6),
                cache_read_input_token_cost_above_200k_tokens: Some(0.6e-6),
            },
        );
        PricingData::from_map(models)
    }

    #[test]
    fn test_sonnet_pricing() {
        let pricing = sonnet_pricing();
        let cost = pricing.calculate_cost("claude-sonnet-4-5-20250929", 1_000_000, 1_000_000, 0, 0);
        // $3 input + $15 output = $18
        assert!((cost - 18.0).abs() < 0.001);
    }

    #[test]
    fn test_cache_pricing() {
        let pricing = sonnet_pricing();
        let cost = pricing.calculate_cost("claude-sonnet-4-5-20250929", 0, 0, 1_000_000, 1_000_000);
        // $3.75 cache write + $0.30 cache read = $4.05
        assert!((cost - 4.05).abs() < 0.001);
    }

    #[test]
    fn test_zero_tokens() {
        let pricing = sonnet_pricing();
        let cost = pricing.calculate_cost("claude-sonnet-4-5-20250929", 0, 0, 0, 0);
        assert!((cost).abs() < 0.0001);
    }

    #[test]
    fn test_unknown_model_returns_zero() {
        let pricing = sonnet_pricing();
        let cost = pricing.calculate_cost("unknown-model-xyz", 1_000_000, 1_000_000, 0, 0);
        assert!((cost).abs() < 0.0001);
    }

    #[test]
    fn test_provider_prefix_matching() {
        let pricing = tiered_pricing();
        // Model stored as "anthropic/claude-sonnet-4-5-20250929"
        // Query with just "claude-sonnet-4-5-20250929" should match via prefix
        let result = pricing.get_model_pricing("claude-sonnet-4-5-20250929");
        assert!(result.is_some());
    }

    #[test]
    fn test_fuzzy_matching() {
        let pricing = tiered_pricing();
        // Fuzzy match: query substring contained in key
        let result = pricing.get_model_pricing("sonnet-4-5");
        assert!(result.is_some());
    }

    #[test]
    fn test_tiered_pricing_below_threshold() {
        let pricing = tiered_pricing();
        // 100k tokens, below 200k threshold → use base price only
        let cost = pricing.calculate_cost("claude-sonnet-4-5-20250929", 100_000, 0, 0, 0);
        assert!((cost - 100_000.0 * 3e-6).abs() < 0.001);
    }

    #[test]
    fn test_tiered_pricing_at_boundary() {
        let pricing = tiered_pricing();
        // Exactly 200k tokens → use base price only (threshold not exceeded)
        let cost = pricing.calculate_cost("claude-sonnet-4-5-20250929", 200_000, 0, 0, 0);
        assert!((cost - 200_000.0 * 3e-6).abs() < 0.001);
    }

    #[test]
    fn test_tiered_pricing_above_threshold() {
        let pricing = tiered_pricing();
        // 300k input: 200k at $3/M + 100k at $6/M = $0.60 + $0.60 = $1.20
        let cost = pricing.calculate_cost("claude-sonnet-4-5-20250929", 300_000, 0, 0, 0);
        let expected = 200_000.0 * 3e-6 + 100_000.0 * 6e-6;
        assert!((cost - expected).abs() < 0.001);
    }

    #[test]
    fn test_tiered_pricing_all_token_types() {
        let pricing = tiered_pricing();
        // 300k input, 250k output, 300k cache creation, 250k cache read
        let cost = pricing.calculate_cost(
            "claude-sonnet-4-5-20250929",
            300_000,
            250_000,
            300_000,
            250_000,
        );

        let expected = (200_000.0 * 3e-6 + 100_000.0 * 6e-6)       // input
            + (200_000.0 * 15e-6 + 50_000.0 * 22.5e-6)             // output
            + (200_000.0 * 3.75e-6 + 100_000.0 * 7.5e-6)           // cache creation
            + (200_000.0 * 0.30e-6 + 50_000.0 * 0.6e-6); // cache read
        assert!((cost - expected).abs() < 0.001);
    }

    #[test]
    fn test_no_tiered_pricing_ignores_threshold() {
        // Model without tiered pricing: all tokens at base rate
        let mut models = HashMap::new();
        models.insert(
            "gpt-5".to_string(),
            ModelPricing {
                input_cost_per_token: Some(1e-6),
                output_cost_per_token: Some(2e-6),
                cache_creation_input_token_cost: None,
                cache_read_input_token_cost: None,
                input_cost_per_token_above_200k_tokens: None,
                output_cost_per_token_above_200k_tokens: None,
                cache_creation_input_token_cost_above_200k_tokens: None,
                cache_read_input_token_cost_above_200k_tokens: None,
            },
        );
        let pricing = PricingData::from_map(models);

        let cost = pricing.calculate_cost("gpt-5", 300_000, 250_000, 0, 0);
        assert!((cost - (300_000.0 * 1e-6 + 250_000.0 * 2e-6)).abs() < 0.001);
    }

    #[test]
    fn test_tiered_cost_function() {
        // Below threshold
        assert!((tiered_cost(100_000, Some(3e-6), Some(6e-6)) - 100_000.0 * 3e-6).abs() < 1e-10);

        // At threshold
        assert!((tiered_cost(200_000, Some(3e-6), Some(6e-6)) - 200_000.0 * 3e-6).abs() < 1e-10);

        // Above threshold
        let expected = 200_000.0 * 3e-6 + 100_000.0 * 6e-6;
        assert!((tiered_cost(300_000, Some(3e-6), Some(6e-6)) - expected).abs() < 1e-10);

        // Zero tokens
        assert!((tiered_cost(0, Some(3e-6), Some(6e-6))).abs() < 1e-10);

        // No base price, above threshold: only charges above-threshold tokens
        let expected = 100_000.0 * 6e-6;
        assert!((tiered_cost(300_000, None, Some(6e-6)) - expected).abs() < 1e-10);

        // No prices at all
        assert!((tiered_cost(300_000, None, None)).abs() < 1e-10);
    }

    #[test]
    fn test_embedded_fallback_has_models() {
        let fallback = PricingData::embedded_fallback();
        assert!(!fallback.models.is_empty());
        assert!(fallback
            .get_model_pricing("claude-sonnet-4-5-20250929")
            .is_some());
        assert!(fallback.get_model_pricing("claude-opus-4-6").is_some());
        assert!(fallback
            .get_model_pricing("claude-haiku-4-5-20251001")
            .is_some());
        assert!(fallback
            .get_model_pricing("claude-opus-4-5-20251101")
            .is_some());
    }

    #[test]
    fn test_parse_raw_data_skips_invalid() {
        let mut raw = HashMap::new();
        raw.insert(
            "valid-model".to_string(),
            serde_json::json!({
                "input_cost_per_token": 3e-6,
                "output_cost_per_token": 15e-6,
            }),
        );
        raw.insert(
            "no-cost-model".to_string(),
            serde_json::json!({
                "max_tokens": 4096,
            }),
        );
        raw.insert(
            "not-an-object".to_string(),
            serde_json::json!("string value"),
        );

        let models = PricingData::parse_raw_data(raw);
        assert_eq!(models.len(), 1);
        assert!(models.contains_key("valid-model"));
    }
}
