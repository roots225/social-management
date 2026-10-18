use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CurationStatus {
    New,
    Queued,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "item_type")]
pub enum ItemMetadata {
    News {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        author: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        category: Option<String>,
    },
    JobOffer {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        company: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        location: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        contract_type: Option<String>,
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            with = "crate::models::opt_chrono_datetime"
        )]
        expires_at: Option<DateTime<Utc>>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapedItem {
    #[serde(
        rename = "_id",
        skip_serializing_if = "Option::is_none",
        default,
        with = "crate::models::opt_oid_as_hex"
    )]
    pub id: Option<ObjectId>,
    #[serde(with = "crate::models::oid_as_hex")]
    pub source_id: ObjectId,
    pub url_hash: String,
    pub original_url: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    pub curation_status: CurationStatus,
    #[serde(with = "crate::models::chrono_datetime")]
    pub scraped_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<ItemMetadata>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QueueItemRequest {
    #[serde(default)]
    pub priority: Option<i32>,
    #[serde(default)]
    pub custom_text: Option<String>,
    #[serde(default)]
    pub target_account_ids: Option<Vec<String>>,
}
