use crate::models::{
    DeliveryStatus, Publication, PublicationStatus, UpdatePublicationRequest,
};
use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use bson::{doc, oid::ObjectId};
use futures::stream::TryStreamExt;
use mongodb::options::FindOptions;
use serde::Deserialize;
use serde_json::json;
use tracing::info;

#[derive(Debug, Deserialize)]
pub struct PublicationsQuery {
    pub status: Option<PublicationStatus>,
    pub limit: Option<i64>,
    pub skip: Option<u64>,
}

pub async fn list_publications(
    State(state): State<AppState>,
    Query(params): Query<PublicationsQuery>,
) -> impl IntoResponse {
    let col = state.db.collection::<Publication>("publications");
    let mut filter = doc! {};

    if let Some(status) = params.status {
        if let Ok(b) = bson::to_bson(&status) {
            filter.insert("status", b);
        }
    }

    let limit = params.limit.unwrap_or(50).min(100);
    let skip = params.skip.unwrap_or(0);

    let find_opts = FindOptions::builder()
        .sort(doc! { "priority": -1, "created_at": -1 })
        .limit(limit)
        .skip(skip)
        .build();

    match col.find(filter).with_options(find_opts).await {
        Ok(cursor) => match cursor.try_collect::<Vec<Publication>>().await {
            Ok(pubs) => (StatusCode::OK, Json(pubs)).into_response(),
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

pub async fn update_publication(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdatePublicationRequest>,
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

    let col = state.db.collection::<Publication>("publications");
    let mut publication = match col.find_one(doc! { "_id": oid }).await {
        Ok(Some(p)) => p,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Publication introuvable" })),
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

    if let Some(priority) = payload.priority {
        publication.priority = priority;
    }
    if let Some(default_text) = payload.default_text {
        publication.default_text = default_text;
    }
    if let Some(media_urls) = payload.media_urls {
        publication.media_urls = media_urls;
    }

    // Mise à jour éventuelle des custom_text sur les deliveries
    if let Some(updated_deliveries) = payload.deliveries {
        for upd in updated_deliveries {
            if let Ok(acc_oid) = ObjectId::parse_str(&upd.account_id) {
                if let Some(d) = publication
                    .deliveries
                    .iter_mut()
                    .find(|d| d.account_id == acc_oid)
                {
                    d.custom_text = upd.custom_text;
                }
            }
        }
    }

    match col.replace_one(doc! { "_id": oid }, &publication).await {
        Ok(_) => (StatusCode::OK, Json(publication)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub async fn delete_publication(
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

    let col = state.db.collection::<Publication>("publications");
    match col.delete_one(doc! { "_id": oid }).await {
        Ok(res) if res.deleted_count > 0 => (
            StatusCode::OK,
            Json(json!({ "message": "Publication supprimée avec succès" })),
        )
            .into_response(),
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Publication introuvable" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub async fn retry_publication(
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

    let col = state.db.collection::<Publication>("publications");
    let mut publication = match col.find_one(doc! { "_id": oid }).await {
        Ok(Some(p)) => p,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Publication introuvable" })),
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

    // Relance uniquement les deliveries en échec
    let mut retried_count = 0;
    for delivery in &mut publication.deliveries {
        if delivery.status == DeliveryStatus::Failed {
            delivery.status = DeliveryStatus::Pending;
            delivery.error_message = None;
            retried_count += 1;
        }
    }

    if retried_count == 0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "message": "Aucune livraison en échec à relancer" })),
        )
            .into_response();
    }

    publication.status = PublicationStatus::Pending;

    match col.replace_one(doc! { "_id": oid }, &publication).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({
                "message": format!("{} livraisons remises en file d'attente (PENDING)", retried_count),
                "publication": publication
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub async fn publish_now(
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

    info!("Déclenchement immédiat de la publication {}", id);
    match state.scheduler.publish_immediately(oid).await {
        Ok(pub_result) => (
            StatusCode::OK,
            Json(json!({
                "message": "Publication exécutée avec succès",
                "status": pub_result.status,
                "publication": pub_result
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}
