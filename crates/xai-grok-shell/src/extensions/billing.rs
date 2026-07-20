//! Façade stub of upstream `xai-grok-shell::extensions::billing`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Cent {
    #[serde(default)]
    pub val: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UsagePeriod {
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub period_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BillingConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credit_usage_percent: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_period: Option<UsagePeriod>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monthly_limit: Option<Cent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub used: Option<Cent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub billing_period_end: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_demand_cap: Option<Cent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_demand_used: Option<Cent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prepaid_balance: Option<Cent>,
    #[serde(default)]
    pub is_unified_billing_user: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BillingConfigResponse {
    pub config: Option<BillingConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on_demand_enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subscription_tier: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AutoTopupRule {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub threshold: Option<Cent>,
    #[serde(default)]
    pub topup_amount: Option<Cent>,
    #[serde(default)]
    pub max_amount_per_month: Option<Cent>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GetAutoTopupRuleResponse {
    #[serde(default)]
    pub rule: Option<AutoTopupRule>,
}
