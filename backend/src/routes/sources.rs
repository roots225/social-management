use crate::models::{CreateSourceRequest, Source, UpdateSourceRequest};
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use bson::{doc, oid::ObjectId};
use futures::stream::TryStreamExt;
use serde_json::json;
use tracing::{error, info};

pub async fn list_sources(State(state): State<AppState>) -> impl IntoResponse {
    let col = state.db.collection::<Source>("sources");
    match col.find(doc! {}).await {
        Ok(cursor) => match cursor.try_collect::<Vec<Source>>().await {
            Ok(sources) => (StatusCode::OK, Json(sources)).into_response(),
            Err(e) => {
                error!("Erreur lors de la lecture des sources: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": e.to_string() })),
                )
                    .into_response()
            }
        },
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub async fn create_source(
    State(state): State<AppState>,
    Json(payload): Json<CreateSourceRequest>,
) -> impl IntoResponse {
    let col = state.db.collection::<Source>("sources");
    let new_source = Source {
        id: None,
        name: payload.name,
        base_url: payload.base_url,
        category: payload.category,
        scraper_config: payload.scraper_config,
        cron_schedule: payload.cron_schedule,
        is_active: payload.is_active,
        last_scraped_at: None,
    };

    match col.insert_one(&new_source).await {
        Ok(result) => {
            let inserted_id = result.inserted_id.as_object_id().unwrap();
            let mut created = new_source;
            created.id = Some(inserted_id);
            (StatusCode::CREATED, Json(created)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub async fn update_source(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateSourceRequest>,
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

    let mut update_doc = doc! {};
    if let Some(name) = payload.name {
        update_doc.insert("name", name);
    }
    if let Some(base_url) = payload.base_url {
        update_doc.insert("base_url", base_url);
    }
    if let Some(category) = payload.category {
        if let Ok(b) = bson::to_bson(&category) {
            update_doc.insert("category", b);
        }
    }
    if let Some(scraper_config) = payload.scraper_config {
        if let Ok(b) = bson::to_bson(&scraper_config) {
            update_doc.insert("scraper_config", b);
        }
    }
    if let Some(cron) = payload.cron_schedule {
        update_doc.insert("cron_schedule", cron);
    }
    if let Some(is_active) = payload.is_active {
        update_doc.insert("is_active", is_active);
    }

    if update_doc.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Aucun champ à mettre à jour" })),
        )
            .into_response();
    }

    let col = state.db.collection::<Source>("sources");
    match col
        .update_one(doc! { "_id": oid }, doc! { "$set": update_doc })
        .await
    {
        Ok(res) if res.matched_count > 0 => {
            let updated = col.find_one(doc! { "_id": oid }).await.unwrap();
            (StatusCode::OK, Json(updated)).into_response()
        }
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Source introuvable" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub async fn delete_source(
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

    let col = state.db.collection::<Source>("sources");
    match col.delete_one(doc! { "_id": oid }).await {
        Ok(res) if res.deleted_count > 0 => (
            StatusCode::OK,
            Json(json!({ "message": "Source supprimée avec succès" })),
        )
            .into_response(),
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Source introuvable" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub async fn scrape_source_now(
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

    info!("Déclenchement manuel de scraping pour la source {}", id);
    match state.scraper.scrape_source_by_id(oid).await {
        Ok(report) => (
            StatusCode::OK,
            Json(json!({
                "message": "Scraping exécuté avec succès",
                "source_id": report.source_id.to_hex(),
                "source_name": report.source_name,
                "total_found": report.total_found,
                "new_inserted": report.new_inserted,
                "duplicates_skipped": report.duplicates_skipped
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
