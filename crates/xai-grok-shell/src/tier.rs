//! Vendored near-verbatim from upstream `xai-grok-shell::tier` (~30 lines).

pub fn is_restricted_tier_name(tier: &str) -> bool {
    let t = tier.trim().to_ascii_lowercase();
    t.is_empty() || t == "free" || t == "x basic" || t == "x_basic"
}
