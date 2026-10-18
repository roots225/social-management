use crate::models::{
    CurationStatus, Delivery, DeliveryStatus, Publication, PublicationStatus, QueueItemRequest,
    ScrapedItem, SocialAccount, Source, SourceCategory,
};
use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use bson::{doc, oid::ObjectId};
use chrono::Utc;
use futures::stream::TryStreamExt;
use mongodb::options::FindOptions;
use serde::Deserialize;
use serde_json::json;
use tracing::info;

#[derive(Debug, Deserialize)]
pub struct ScrapedItemsQuery {
    pub status: Option<CurationStatus>,
    pub category: Option<SourceCategory>,
    pub limit: Option<i64>,
    pub skip: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateStatusRequest {
    pub status: CurationStatus,
}

pub async fn list_scraped_items(
    State(state): State<AppState>,
    Query(params): Query<ScrapedItemsQuery>,
) -> impl IntoResponse {
    let col = state.db.collection::<ScrapedItem>("scraped_items");
    let mut filter = doc! {};

    if let Some(status) = params.status {
        if let Ok(b) = bson::to_bson(&status) {
            filter.insert("curation_status", b);
        }
    }

    if let Some(category) = params.category {
        match category {
            SourceCategory::News => {
                filter.insert("metadata.item_type", "News");
            }
            SourceCategory::JobOffer => {
                filter.insert("metadata.item_type", "JobOffer");
            }
        }
    }

    let limit = params.limit.unwrap_or(50).min(100);
    let skip = params.skip.unwrap_or(0);

    let find_opts = FindOptions::builder()
        .sort(doc! { "scraped_at": -1 })
        .limit(limit)
        .skip(skip)
        .build();

    match col.find(filter).with_options(find_opts).await {
        Ok(cursor) => match cursor.try_collect::<Vec<ScrapedItem>>().await {
            Ok(items) => (StatusCode::OK, Json(items)).into_response(),
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

pub async fn update_item_status(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateStatusRequest>,
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

    let col = state.db.collection::<ScrapedItem>("scraped_items");
    let status_bson = match bson::to_bson(&payload.status) {
        Ok(b) => b,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    };

    match col
        .update_one(
            doc! { "_id": oid },
            doc! { "$set": { "curation_status": status_bson } },
        )
        .await
    {
        Ok(res) if res.matched_count > 0 => (
            StatusCode::OK,
            Json(json!({ "message": "Statut de curation mis à jour avec succès" })),
        )
            .into_response(),
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Élément introuvable" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub async fn queue_scraped_item(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<QueueItemRequest>,
) -> axum::response::Response {
    let item_oid = match ObjectId::parse_str(&id) {
        Ok(o) => o,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Identifiant ObjectId invalide" })),
            )
                .into_response()
        }
    };

    let scraped_col = state.db.collection::<ScrapedItem>("scraped_items");
    let item = match scraped_col.find_one(doc! { "_id": item_oid }).await {
        Ok(Some(i)) => i,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Élément scrapé introuvable" })),
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

    // Récupération de la source associée pour connaître la catégorie
    let sources_col = state.db.collection::<Source>("sources");
    let source = match sources_col.find_one(doc! { "_id": item.source_id }).await {
        Ok(Some(s)) => s,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Source parente introuvable" })),
            )
                .into_response()
        }
    };

    // Récupération des comptes sociaux actifs correspondants
    let social_col = state.db.collection::<SocialAccount>("social_accounts");
    let category_bson = bson::to_bson(&source.category).unwrap();

    let account_filter = if let Some(target_ids) = payload.target_account_ids {
        let oids: Vec<ObjectId> = target_ids
            .iter()
            .filter_map(|s| ObjectId::parse_str(s).ok())
            .collect();
        doc! {
            "_id": { "$in": oids },
            "is_active": true
        }
    } else {
        doc! {
            "is_active": true,
            "auto_publish_categories": category_bson
        }
    };

    let active_accounts: Vec<SocialAccount> = match social_col.find(account_filter).await {
        Ok(cursor) => cursor.try_collect().await.unwrap_or_default(),
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    };

    // Génération des deliveries pour chaque compte social éligible
    let deliveries: Vec<Delivery> = active_accounts
        .into_iter()
        .map(|acc| Delivery {
            account_id: acc.id.unwrap_or_default(),
            platform: acc.platform,
            custom_text: payload.custom_text.clone(),
            status: DeliveryStatus::Pending,
            external_post_id: None,
            published_at: None,
            error_message: None,
        })
        .collect();

    // Formatage du texte par défaut
    let default_text = if let Some(custom) = payload.custom_text {
        custom
    } else {
        match (&item.summary, &source.category) {
            (Some(summary), SourceCategory::News) => {
                format!("📰 {}\n\n{}\n\n👉 {}", item.title, summary, item.original_url)
            }
            (Some(summary), SourceCategory::JobOffer) => {
                format!("💼 Offre : {}\n\n{}\n\nPostuler ici 👉 {}", item.title, summary, item.original_url)
            }
            (None, _) => format!("📢 {}\n\n{}", item.title, item.original_url),
        }
    };

    let mut media_urls = Vec::new();
    if let Some(img) = item.image_url {
        media_urls.push(img);
    }

    let publication = Publication {
        id: None,
        scraped_item_id: item_oid,
        status: PublicationStatus::Pending,
        priority: payload.priority.unwrap_or(0),
        default_text,
        media_urls,
        created_at: Utc::now(),
        published_at: None,
        deliveries,
    };

    let publications_col = state.db.collection::<Publication>("publications");
    let pub_res = match publications_col.insert_one(&publication).await {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    };

    let pub_id = pub_res.inserted_id.as_object_id().unwrap();

    // Mise à jour du statut de l'item à QUEUED
    let _ = scraped_col
        .update_one(
            doc! { "_id": item_oid },
            doc! { "$set": { "curation_status": "QUEUED" } },
        )
        .await;

    let mut created_pub = publication;
    created_pub.id = Some(pub_id);

    info!(
        "Élément scrapé {} transformé avec succès en publication PENDING {} avec {} livraisons planifiées",
        item_oid, pub_id, created_pub.deliveries.len()
    );

    (StatusCode::CREATED, Json(created_pub)).into_response()
}
