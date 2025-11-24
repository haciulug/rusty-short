use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Link {
    pub id: Uuid,
    pub key: String,
    pub original_url: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub click_count: i64,
    pub owner_id: Option<Uuid>,
}

impl Link {
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            expires_at < Utc::now()
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLinkRequest {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_alias: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_in: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkResponse {
    pub key: String,
    pub short_url: String,
    pub original_url: String,
    pub qr_code_url: String,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkStats {
    pub key: String,
    pub original_url: String,
    pub click_count: i64,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LinkAnalytics {
    pub id: Uuid,
    pub link_id: Uuid,
    pub clicked_at: DateTime<Utc>,
    pub referrer: Option<String>,
    pub user_agent: Option<String>,
    pub ip_hash: Option<String>,
    pub country_code: Option<String>,
    pub browser: Option<String>,
    pub os: Option<String>,
    pub device_type: Option<String>,
    pub city: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsSummary {
    pub total_clicks: i64,
    pub unique_visitors: i64,
    pub top_referrers: Vec<ReferrerStats>,
    pub device_breakdown: DeviceBreakdown,
    pub geographic_distribution: Vec<CountryStats>,
    pub browser_stats: Vec<BrowserStats>,
    pub time_series: Vec<TimeSeriesPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferrerStats {
    pub domain: String,
    pub count: i64,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceBreakdown {
    pub desktop: i64,
    pub mobile: i64,
    pub tablet: i64,
    pub bot: i64,
    pub other: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountryStats {
    pub country_code: String,
    pub count: i64,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserStats {
    pub browser: String,
    pub count: i64,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesPoint {
    pub date: String,
    pub clicks: i64,
    pub unique_visitors: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

