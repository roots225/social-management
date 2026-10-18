use super::source::SourceCategory;
use crate::crypto::mask_secret;
use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Platform {
    Twitter,
    Facebook,
    Linkedin,
    Telegram,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "auth_type")]
pub enum PlatformCredentials {
    TwitterV2 {
        api_key: String,
        api_secret: String,
        access_token: String,
        access_token_secret: String,
    },
    FacebookGraph {
        page_id: String,
        page_access_token: String,
        app_id: String,
        app_secret: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "auth_type")]
pub enum MaskedCredentials {
    TwitterV2 {
        api_key: String,
        api_secret: String,
        access_token: String,
        access_token_secret: String,
    },
    FacebookGraph {
        page_id: String,
        page_access_token: String,
        app_id: String,
        app_secret: String,
    },
}

impl PlatformCredentials {
    pub fn to_masked(&self) -> MaskedCredentials {
        match self {
            Self::TwitterV2 {
                api_key,
                api_secret,
                access_token,
                access_token_secret,
            } => MaskedCredentials::TwitterV2 {
                api_key: mask_secret(api_key),
                api_secret: mask_secret(api_secret),
                access_token: mask_secret(access_token),
                access_token_secret: mask_secret(access_token_secret),
            },
            Self::FacebookGraph {
                page_id,
                page_access_token,
                app_id,
                app_secret,
            } => MaskedCredentials::FacebookGraph {
                page_id: page_id.clone(),
                page_access_token: mask_secret(page_access_token),
                app_id: mask_secret(app_id),
                app_secret: mask_secret(app_secret),
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountHealthStatus {
    pub is_valid: bool,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "crate::models::opt_chrono_datetime"
    )]
    pub last_verified_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "crate::models::opt_chrono_datetime"
    )]
    pub token_expires_at: Option<DateTime<Utc>>,
}

impl Default for AccountHealthStatus {
    fn default() -> Self {
        Self {
            is_valid: true,
            last_verified_at: None,
            last_error: None,
            token_expires_at: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialAccount {
    #[serde(
        rename = "_id",
        skip_serializing_if = "Option::is_none",
        default,
        with = "crate::models::opt_oid_as_hex"
    )]
    pub id: Option<ObjectId>,
    pub label: String,
    pub platform: Platform,
    pub is_active: bool,
    pub auto_publish_categories: Vec<SourceCategory>,
    pub credentials_encrypted: String,
    pub health_status: AccountHealthStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialAccountResponse {
    #[serde(rename = "_id")]
    pub id: String,
    pub label: String,
    pub platform: Platform,
    pub is_active: bool,
    pub auto_publish_categories: Vec<SourceCategory>,
    pub credentials: MaskedCredentials,
    pub health_status: AccountHealthStatus,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateSocialAccountRequest {
    pub label: String,
    pub platform: Platform,
    #[serde(default = "default_true")]
    pub is_active: bool,
    pub auto_publish_categories: Vec<SourceCategory>,
    pub credentials: PlatformCredentials,
}

fn default_true() -> bool {
    true
}
