/// Calculate cost in USD for a given model and token counts.
///
/// Pricing is per million tokens (matching LiteLLM/ccusage rates).
/// Cache rates apply to prompt caching.
pub fn calculate_cost(
    model: &str,
    input_tokens: u64,
    output_tokens: u64,
    cache_creation_tokens: u64,
    cache_read_tokens: u64,
) -> f64 {
    let (input_rate, output_rate, cache_write_rate, cache_read_rate) = model_rates(model);

    let input_cost = input_tokens as f64 * input_rate / 1_000_000.0;
    let output_cost = output_tokens as f64 * output_rate / 1_000_000.0;
    let cache_write_cost = cache_creation_tokens as f64 * cache_write_rate / 1_000_000.0;
    let cache_read_cost = cache_read_tokens as f64 * cache_read_rate / 1_000_000.0;

    input_cost + output_cost + cache_write_cost + cache_read_cost
}

/// Returns (input_per_mtok, output_per_mtok, cache_write_per_mtok, cache_read_per_mtok)
///
/// Pricing aligned with LiteLLM/ccusage. Key distinction:
/// - Opus 4.5+ (including 4.6): $5/$25 (new pricing)
/// - Opus 4.1 and earlier (3 Opus, 4.0): $15/$75 (legacy pricing)
/// - Haiku 4.5: $1/$5
/// - Haiku 3.5 and earlier: $0.80/$4 (3.5) or $0.25/$1.25 (3.0)
/// - All Sonnet variants: $3/$15
fn model_rates(model: &str) -> (f64, f64, f64, f64) {
    let model_lower = model.to_lowercase();

    if model_lower.contains("opus") {
        if model_lower.contains("opus-4-5") || model_lower.contains("opus-4-6") {
            // Claude Opus 4.5/4.6: $5/$25, cache write $6.25, cache read $0.50
            (5.0, 25.0, 6.25, 0.50)
        } else {
            // Claude Opus 4.1 and earlier (3 Opus, 4.0): $15/$75
            (15.0, 75.0, 18.75, 1.50)
        }
    } else if model_lower.contains("haiku") {
        if model_lower.contains("haiku-4-5") || model_lower.contains("haiku-4-6") {
            // Claude Haiku 4.5+: $1/$5, cache write $1.25, cache read $0.10
            (1.0, 5.0, 1.25, 0.10)
        } else if model_lower.contains("3-haiku") {
            // Claude 3 Haiku: $0.25/$1.25
            (0.25, 1.25, 0.30, 0.03)
        } else {
            // Claude 3.5 Haiku (default haiku): $0.80/$4
            (0.80, 4.0, 1.00, 0.08)
        }
    } else {
        // All Sonnet variants (4.5, 4, 3.7, 3.5): $3/$15
        (3.0, 15.0, 3.75, 0.30)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sonnet_pricing() {
        let cost = calculate_cost("claude-sonnet-4-5-20250929", 1_000_000, 1_000_000, 0, 0);
        // $3 input + $15 output = $18
        assert!((cost - 18.0).abs() < 0.001);
    }

    #[test]
    fn test_opus_4_6_pricing() {
        let cost = calculate_cost("claude-opus-4-6", 1_000_000, 100_000, 0, 0);
        // $5 input + $2.5 output = $7.5
        assert!((cost - 7.5).abs() < 0.001);
    }

    #[test]
    fn test_opus_4_5_pricing() {
        let cost = calculate_cost("claude-opus-4-5-20250929", 1_000_000, 100_000, 0, 0);
        // $5 input + $2.5 output = $7.5
        assert!((cost - 7.5).abs() < 0.001);
    }

    #[test]
    fn test_opus_legacy_pricing() {
        let cost = calculate_cost("claude-opus-4-1-20250414", 1_000_000, 100_000, 0, 0);
        // $15 input + $7.5 output = $22.5
        assert!((cost - 22.5).abs() < 0.001);
    }

    #[test]
    fn test_opus_3_pricing() {
        let cost = calculate_cost("claude-3-opus-20240229", 1_000_000, 100_000, 0, 0);
        // $15 input + $7.5 output = $22.5
        assert!((cost - 22.5).abs() < 0.001);
    }

    #[test]
    fn test_haiku_4_5_pricing() {
        let cost = calculate_cost("claude-haiku-4-5-20251001", 1_000_000, 1_000_000, 0, 0);
        // $1 input + $5 output = $6
        assert!((cost - 6.0).abs() < 0.001);
    }

    #[test]
    fn test_haiku_3_5_pricing() {
        let cost = calculate_cost("claude-3-5-haiku-20241022", 1_000_000, 1_000_000, 0, 0);
        // $0.80 input + $4 output = $4.80
        assert!((cost - 4.80).abs() < 0.001);
    }

    #[test]
    fn test_haiku_3_pricing() {
        let cost = calculate_cost("claude-3-haiku-20240307", 1_000_000, 1_000_000, 0, 0);
        // $0.25 input + $1.25 output = $1.50
        assert!((cost - 1.50).abs() < 0.001);
    }

    #[test]
    fn test_cache_pricing_sonnet() {
        let cost = calculate_cost("claude-sonnet-4-5-20250929", 0, 0, 1_000_000, 1_000_000);
        // $3.75 cache write + $0.30 cache read = $4.05
        assert!((cost - 4.05).abs() < 0.001);
    }

    #[test]
    fn test_cache_pricing_opus_4_6() {
        let cost = calculate_cost("claude-opus-4-6", 0, 0, 1_000_000, 1_000_000);
        // $6.25 cache write + $0.50 cache read = $6.75
        assert!((cost - 6.75).abs() < 0.001);
    }

    #[test]
    fn test_zero_tokens() {
        let cost = calculate_cost("claude-sonnet-4-5-20250929", 0, 0, 0, 0);
        assert!((cost).abs() < 0.0001);
    }
}
