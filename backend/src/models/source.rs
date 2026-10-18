use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SourceCategory {
    News,
    JobOffer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScraperConfig {
    pub item_container: String,
    pub title: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub specific_fields: Option<HashMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub render_js: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wait_for_selector: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wait_duration_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    #[serde(
        rename = "_id",
        skip_serializing_if = "Option::is_none",
        default,
        with = "crate::models::opt_oid_as_hex"
    )]
    pub id: Option<ObjectId>,
    pub name: String,
    pub base_url: String,
    pub category: SourceCategory,
    pub scraper_config: ScraperConfig,
    pub cron_schedule: String,
    pub is_active: bool,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "crate::models::opt_chrono_datetime"
    )]
    pub last_scraped_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateSourceRequest {
    pub name: String,
    pub base_url: String,
    pub category: SourceCategory,
    pub scraper_config: ScraperConfig,
    pub cron_schedule: String,
    #[serde(default = "default_true")]
    pub is_active: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateSourceRequest {
    pub name: Option<String>,
    pub base_url: Option<String>,
    pub category: Option<SourceCategory>,
    pub scraper_config: Option<ScraperConfig>,
    pub cron_schedule: Option<String>,
    pub is_active: Option<bool>,
}

fn default_true() -> bool {
    true
}
