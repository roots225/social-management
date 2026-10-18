use crate::models::{Platform, PlatformCredentials};
use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use tracing::{error, info};

#[derive(Debug, Error)]
pub enum PublishError {
    #[error("Erreur réseau / HTTP : {0}")]
    Network(#[from] reqwest::Error),

    #[error("Identifiants invalides pour la plateforme {0} : {1}")]
    InvalidCredentials(String, String),

    #[error("Erreur API distante {platform} (Code: {code}) : {message}")]
    RemoteApiError {
        platform: String,
        code: u16,
        message: String,
    },

    #[error("Plateforme non supportée : {0}")]
    UnsupportedPlatform(String),

    #[error("Erreur interne du publisher : {0}")]
    Internal(String),
}

#[async_trait]
pub trait SocialPublisher: Send + Sync {
    /// Publie un contenu sur le réseau social
    async fn publish(
        &self,
        text: &str,
        media_urls: &[String],
        credentials: &PlatformCredentials,
    ) -> Result<String, PublishError>;

    /// Vérifie la validité des credentials auprès de l'API distante
    async fn verify_credentials(
        &self,
        credentials: &PlatformCredentials,
    ) -> Result<(), PublishError>;
}

// =========================================================================
// Twitter / X Publisher (API v2)
// =========================================================================
pub struct TwitterPublisher {
    client: reqwest::Client,
}

impl TwitterPublisher {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(15))
                .build()
                .unwrap_or_default(),
        }
    }
}

#[derive(Serialize)]
struct TwitterTweetRequest<'a> {
    text: &'a str,
}

#[derive(Deserialize)]
struct TwitterTweetData {
    id: String,
}

#[derive(Deserialize)]
struct TwitterTweetResponse {
    data: Option<TwitterTweetData>,
    detail: Option<String>,
    title: Option<String>,
}

#[derive(Deserialize)]
struct TwitterUserResponse {
    data: Option<serde_json::Value>,
    errors: Option<Vec<serde_json::Value>>,
}

#[async_trait]
impl SocialPublisher for TwitterPublisher {
    async fn publish(
        &self,
        text: &str,
        _media_urls: &[String],
        credentials: &PlatformCredentials,
    ) -> Result<String, PublishError> {
        let (api_key, _api_secret, access_token, _access_token_secret) = match credentials {
            PlatformCredentials::TwitterV2 {
                api_key,
                api_secret,
                access_token,
                access_token_secret,
            } => (api_key, api_secret, access_token, access_token_secret),
            _ => {
                return Err(PublishError::InvalidCredentials(
                    "TWITTER".into(),
                    "Les identifiants ne sont pas du type TwitterV2".into(),
                ))
            }
        };

        // Support mode Test / Mock pour développement local
        if api_key.starts_with("test_") || api_key.starts_with("mock_") || access_token.starts_with("test_") {
            let mock_id = format!("x_post_{}", chrono::Utc::now().timestamp_millis());
            info!("Mode simulation Twitter activé. ID généré: {}", mock_id);
            return Ok(mock_id);
        }

        let body = TwitterTweetRequest { text };
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", access_token))
                .map_err(|e| PublishError::Internal(e.to_string()))?,
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let res = self
            .client
            .post("https://api.twitter.com/2/tweets")
            .headers(headers)
            .json(&body)
            .send()
            .await?;

        let status = res.status();
        let resp_text = res.text().await.unwrap_or_default();

        if status.is_success() {
            if let Ok(parsed) = serde_json::from_str::<TwitterTweetResponse>(&resp_text) {
                if let Some(data) = parsed.data {
                    return Ok(data.id);
                }
            }
            Ok(format!("tweet_{}", chrono::Utc::now().timestamp_millis()))
        } else {
            error!("Erreur Twitter API ({}): {}", status, resp_text);
            Err(PublishError::RemoteApiError {
                platform: "TWITTER".into(),
                code: status.as_u16(),
                message: resp_text,
            })
        }
    }

    async fn verify_credentials(
        &self,
        credentials: &PlatformCredentials,
    ) -> Result<(), PublishError> {
        let (api_key, access_token) = match credentials {
            PlatformCredentials::TwitterV2 {
                api_key,
                access_token,
                ..
            } => (api_key, access_token),
            _ => {
                return Err(PublishError::InvalidCredentials(
                    "TWITTER".into(),
                    "Les identifiants ne sont pas du type TwitterV2".into(),
                ))
            }
        };

        if api_key.starts_with("test_") || api_key.starts_with("mock_") || access_token.starts_with("test_") {
            info!("Vérification credentials Twitter (mock) : Succès");
            return Ok(());
        }

        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", access_token))
                .map_err(|e| PublishError::Internal(e.to_string()))?,
        );

        let res = self
            .client
            .get("https://api.twitter.com/2/users/me")
            .headers(headers)
            .send()
            .await?;

        let status = res.status();
        if status.is_success() {
            let user_resp: Result<TwitterUserResponse, _> = res.json().await;
            if let Ok(data) = user_resp {
                if data.data.is_some() {
                    return Ok(());
                }
            }
            Ok(())
        } else {
            let msg = res.text().await.unwrap_or_default();
            Err(PublishError::RemoteApiError {
                platform: "TWITTER".into(),
                code: status.as_u16(),
                message: msg,
            })
        }
    }
}

// =========================================================================
// Facebook Graph API Publisher
// =========================================================================
pub struct FacebookPublisher {
    client: reqwest::Client,
}

impl FacebookPublisher {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(15))
                .build()
                .unwrap_or_default(),
        }
    }
}

#[derive(Serialize)]
struct FacebookPostRequest<'a> {
    message: &'a str,
    access_token: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    link: Option<&'a str>,
}

#[derive(Deserialize)]
struct FacebookPostResponse {
    id: Option<String>,
    error: Option<FacebookErrorDetails>,
}

#[derive(Deserialize)]
struct FacebookErrorDetails {
    message: String,
    code: u16,
}

#[async_trait]
impl SocialPublisher for FacebookPublisher {
    async fn publish(
        &self,
        text: &str,
        media_urls: &[String],
        credentials: &PlatformCredentials,
    ) -> Result<String, PublishError> {
        let (page_id, page_access_token) = match credentials {
            PlatformCredentials::FacebookGraph {
                page_id,
                page_access_token,
                ..
            } => (page_id, page_access_token),
            _ => {
                return Err(PublishError::InvalidCredentials(
                    "FACEBOOK".into(),
                    "Les identifiants ne sont pas du type FacebookGraph".into(),
                ))
            }
        };

        // Support mode Test / Mock pour développement local
        if page_id.starts_with("test_") || page_id.starts_with("mock_") || page_access_token.starts_with("test_") {
            let mock_id = format!("{}_fb_{}", page_id, chrono::Utc::now().timestamp_millis());
            info!("Mode simulation Facebook activé. ID généré: {}", mock_id);
            return Ok(mock_id);
        }

        let link = media_urls.first().map(|s| s.as_str());
        let url = format!("https://graph.facebook.com/v19.0/{}/feed", page_id);

        let params = FacebookPostRequest {
            message: text,
            access_token: page_access_token,
            link,
        };

        let res = self.client.post(&url).json(&params).send().await?;
        let status = res.status();
        let resp_json: Result<FacebookPostResponse, _> = res.json().await;

        match resp_json {
            Ok(fb_res) => {
                if let Some(id) = fb_res.id {
                    Ok(id)
                } else if let Some(err) = fb_res.error {
                    Err(PublishError::RemoteApiError {
                        platform: "FACEBOOK".into(),
                        code: err.code,
                        message: err.message,
                    })
                } else {
                    Err(PublishError::Internal("Réponse inattendue de Facebook".into()))
                }
            }
            Err(e) => {
                if status.is_success() {
                    Ok(format!("fb_{}", chrono::Utc::now().timestamp_millis()))
                } else {
                    Err(PublishError::RemoteApiError {
                        platform: "FACEBOOK".into(),
                        code: status.as_u16(),
                        message: e.to_string(),
                    })
                }
            }
        }
    }

    async fn verify_credentials(
        &self,
        credentials: &PlatformCredentials,
    ) -> Result<(), PublishError> {
        let (page_id, page_access_token) = match credentials {
            PlatformCredentials::FacebookGraph {
                page_id,
                page_access_token,
                ..
            } => (page_id, page_access_token),
            _ => {
                return Err(PublishError::InvalidCredentials(
                    "FACEBOOK".into(),
                    "Les identifiants ne sont pas du type FacebookGraph".into(),
                ))
            }
        };

        if page_id.starts_with("test_") || page_access_token.starts_with("test_") || page_id.starts_with("mock_") {
            info!("Vérification credentials Facebook (mock) : Succès");
            return Ok(());
        }

        let url = format!(
            "https://graph.facebook.com/v19.0/{}?fields=id,name&access_token={}",
            page_id, page_access_token
        );

        let res = self.client.get(&url).send().await?;
        let status = res.status();

        if status.is_success() {
            Ok(())
        } else {
            let msg = res.text().await.unwrap_or_default();
            Err(PublishError::RemoteApiError {
                platform: "FACEBOOK".into(),
                code: status.as_u16(),
                message: msg,
            })
        }
    }
}

// =========================================================================
// Publisher Dispatcher / Factory
// =========================================================================
#[derive(Clone)]
pub struct PublisherService {
    twitter: Arc<TwitterPublisher>,
    facebook: Arc<FacebookPublisher>,
}

impl PublisherService {
    pub fn new() -> Self {
        Self {
            twitter: Arc::new(TwitterPublisher::new()),
            facebook: Arc::new(FacebookPublisher::new()),
        }
    }

    pub fn get_publisher(&self, platform: &Platform) -> Result<Arc<dyn SocialPublisher>, PublishError> {
        match platform {
            Platform::Twitter => Ok(self.twitter.clone()),
            Platform::Facebook => Ok(self.facebook.clone()),
            Platform::Linkedin => Err(PublishError::UnsupportedPlatform("LINKEDIN (Support futur)".into())),
            Platform::Telegram => Err(PublishError::UnsupportedPlatform("TELEGRAM (Support futur)".into())),
        }
    }
}
