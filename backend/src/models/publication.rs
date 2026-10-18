use super::social_account::Platform;
use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PublicationStatus {
    Pending,
    Processing,
    Published,
    PartialFailure,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DeliveryStatus {
    Pending,
    Sent,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delivery {
    #[serde(with = "crate::models::oid_as_hex")]
    pub account_id: ObjectId,
    pub platform: Platform,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_text: Option<String>,
    pub status: DeliveryStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_post_id: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "crate::models::opt_chrono_datetime"
    )]
    pub published_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Publication {
    #[serde(
        rename = "_id",
        skip_serializing_if = "Option::is_none",
        default,
        with = "crate::models::opt_oid_as_hex"
    )]
    pub id: Option<ObjectId>,
    #[serde(with = "crate::models::oid_as_hex")]
    pub scraped_item_id: ObjectId,
    pub status: PublicationStatus,
    pub priority: i32,
    pub default_text: String,
    #[serde(default)]
    pub media_urls: Vec<String>,
    #[serde(with = "crate::models::chrono_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "crate::models::opt_chrono_datetime"
    )]
    pub published_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub deliveries: Vec<Delivery>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdatePublicationRequest {
    pub priority: Option<i32>,
    pub default_text: Option<String>,
    pub media_urls: Option<Vec<String>>,
    pub deliveries: Option<Vec<UpdateDeliveryItem>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateDeliveryItem {
    pub account_id: String,
    pub custom_text: Option<String>,
}
