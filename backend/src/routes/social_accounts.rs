use crate::models::{
    AccountHealthStatus, CreateSocialAccountRequest, PlatformCredentials, SocialAccount,
    SocialAccountResponse,
};
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use bson::{doc, oid::ObjectId};
use chrono::Utc;
use futures::stream::TryStreamExt;
use serde_json::json;
use tracing::info;

pub async fn list_social_accounts(State(state): State<AppState>) -> impl IntoResponse {
    let col = state.db.collection::<SocialAccount>("social_accounts");
    match col.find(doc! {}).await {
        Ok(cursor) => match cursor.try_collect::<Vec<SocialAccount>>().await {
            Ok(accounts) => {
                let responses: Vec<SocialAccountResponse> = accounts
                    .into_iter()
                    .map(|acc| {
                        let id_str = acc.id.map(|o| o.to_hex()).unwrap_or_default();
                        // Déchiffre les credentials pour générer la version masquée "****abcd"
                        let masked = match state.crypto.decrypt(&acc.credentials_encrypted) {
                            Ok(plain) => match serde_json::from_str::<PlatformCredentials>(&plain) {
                                Ok(creds) => creds.to_masked(),
                                Err(_) => acc.credentials_encrypted.clone().into(),
                            },
                            Err(_) => acc.credentials_encrypted.clone().into(),
                        };

                        SocialAccountResponse {
                            id: id_str,
                            label: acc.label,
                            platform: acc.platform,
                            is_active: acc.is_active,
                            auto_publish_categories: acc.auto_publish_categories,
                            credentials: masked,
                            health_status: acc.health_status,
                        }
                    })
                    .collect();

                (StatusCode::OK, Json(responses)).into_response()
            }
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
                .into_response(),
        },
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub async fn create_social_account(
    State(state): State<AppState>,
    Json(payload): Json<CreateSocialAccountRequest>,
) -> impl IntoResponse {
    // 1. Sérialisation des credentials en JSON
    let creds_json = match serde_json::to_string(&payload.credentials) {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": format!("Format de credentials invalide: {}", e) })),
            )
                .into_response()
        }
    };

    // 2. Chiffrement AES-256-GCM avant sauvegarde en base
    let encrypted = match state.crypto.encrypt(&creds_json) {
        Ok(enc) => enc,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Erreur lors du chiffrement des secrets: {}", e) })),
            )
                .into_response()
        }
    };

    let new_account = SocialAccount {
        id: None,
        label: payload.label,
        platform: payload.platform,
        is_active: payload.is_active,
        auto_publish_categories: payload.auto_publish_categories,
        credentials_encrypted: encrypted,
        health_status: AccountHealthStatus::default(),
    };

    let col = state.db.collection::<SocialAccount>("social_accounts");
    match col.insert_one(&new_account).await {
        Ok(res) => {
            let oid = res.inserted_id.as_object_id().unwrap();
            let masked = payload.credentials.to_masked();
            let response = SocialAccountResponse {
                id: oid.to_hex(),
                label: new_account.label,
                platform: new_account.platform,
                is_active: new_account.is_active,
                auto_publish_categories: new_account.auto_publish_categories,
                credentials: masked,
                health_status: new_account.health_status,
            };
            (StatusCode::CREATED, Json(response)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub async fn delete_social_account(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> axum::response::Response {
    let oid = match ObjectId::parse_str(&id) {
        Ok(o) => o,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Identifiant ObjectId invalide" })),
            )
                .into_response()
        }
    };

    let col = state.db.collection::<SocialAccount>("social_accounts");
    match col.delete_one(doc! { "_id": oid }).await {
        Ok(res) if res.deleted_count > 0 => (
            StatusCode::OK,
            Json(json!({ "message": "Compte social supprimé avec succès" })),
        )
            .into_response(),
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Compte social introuvable" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub async fn verify_social_account(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> axum::response::Response {
    let oid = match ObjectId::parse_str(&id) {
        Ok(o) => o,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Identifiant ObjectId invalide" })),
            )
                .into_response()
        }
    };

    let col = state.db.collection::<SocialAccount>("social_accounts");
    let account = match col.find_one(doc! { "_id": oid }).await {
        Ok(Some(a)) => a,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Compte social introuvable" })),
            )
                .into_response()
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    };

    // Déchiffrement des credentials
    let decrypted_raw = match state.crypto.decrypt(&account.credentials_encrypted) {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Erreur de déchiffrement: {}", e) })),
            )
                .into_response()
        }
    };

    let credentials: PlatformCredentials = match serde_json::from_str(&decrypted_raw) {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Erreur désérialisation: {}", e) })),
            )
                .into_response()
        }
    };

    let publisher = match state.publisher.get_publisher(&account.platform) {
        Ok(p) => p,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    };

    info!("Vérification des accès pour le compte '{}' ({:?})", account.label, account.platform);

    let verify_result = publisher.verify_credentials(&credentials).await;

    let (is_valid, error_msg) = match verify_result {
        Ok(_) => (true, None),
        Err(e) => (false, Some(e.to_string())),
    };

    let health_update = doc! {
        "health_status.is_valid": is_valid,
        "health_status.last_verified_at": Utc::now(),
        "health_status.last_error": error_msg.as_deref()
    };

    let _ = col
        .update_one(doc! { "_id": oid }, doc! { "$set": health_update })
        .await;

    (
        StatusCode::OK,
        Json(json!({
            "is_valid": is_valid,
            "last_verified_at": Utc::now(),
            "last_error": error_msg,
            "message": if is_valid { "Identifiants vérifiés avec succès !" } else { "Échec de validation des identifiants" }
        })),
    )
        .into_response()
}

// Implémentation de conversion de secours pour MaskedCredentials
impl From<String> for crate::models::MaskedCredentials {
    fn from(_: String) -> Self {
        crate::models::MaskedCredentials::TwitterV2 {
            api_key: "****".into(),
            api_secret: "****".into(),
            access_token: "****".into(),
            access_token_secret: "****".into(),
        }
    }
}
