use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub const DAILY_MAX_LIMIT: u32 = 5;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaHistoryEntry {
    #[serde(with = "crate::models::oid_as_hex")]
    pub publication_id: ObjectId,
    pub slot: String,
    #[serde(with = "crate::models::chrono_datetime")]
    pub executed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyQuota {
    #[serde(rename = "_id")]
    pub id: String, // "YYYY-MM-DD"
    pub published_count: u32,
    pub max_limit: u32,
    #[serde(default)]
    pub history: Vec<QuotaHistoryEntry>,
}

impl DailyQuota {
    pub fn new(date_str: String) -> Self {
        Self {
            id: date_str,
            published_count: 0,
            max_limit: DAILY_MAX_LIMIT,
            history: Vec::new(),
        }
    }
}
