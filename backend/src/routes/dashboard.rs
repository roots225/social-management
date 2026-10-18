use crate::models::{DailyQuota, Publication, ScrapedItem, SocialAccount, Source};
use crate::services::scheduler::SchedulerService;
use crate::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use bson::doc;
use chrono::Utc;
use futures::stream::TryStreamExt;
use serde_json::json;

pub async fn get_dashboard_stats(State(state): State<AppState>) -> impl IntoResponse {
    let today_str = Utc::now().format("%Y-%m-%d").to_string();

    // 1. Quota du jour
    let daily_quotas_col = state.db.collection::<DailyQuota>("daily_quotas");
    let quota_doc = daily_quotas_col
        .find_one(doc! { "_id": &today_str })
        .await
        .unwrap_or(None);

    let published_count = quota_doc.as_ref().map(|q| q.published_count).unwrap_or(0);
    let max_limit = quota_doc.as_ref().map(|q| q.max_limit).unwrap_or(5);
    let history = quota_doc.map(|q| q.history).unwrap_or_default();

    // 2. Prochain créneau d'envoi
    let (next_slot_label, next_slot_time) = SchedulerService::get_next_slot();

    // 3. Publications en attente
    let publications_col = state.db.collection::<Publication>("publications");
    let pending_pubs_count = publications_col
        .count_documents(doc! { "status": "PENDING" })
        .await
        .unwrap_or(0);

    let published_pubs_count = publications_col
        .count_documents(doc! { "status": "PUBLISHED" })
        .await
        .unwrap_or(0);

    let failed_pubs_count = publications_col
        .count_documents(doc! { "status": { "$in": ["FAILED", "PARTIAL_FAILURE"] } })
        .await
        .unwrap_or(0);

    // 4. Comptes sociaux & santé
    let social_col = state.db.collection::<SocialAccount>("social_accounts");
    let all_accounts: Vec<SocialAccount> = match social_col.find(doc! {}).await {
        Ok(cursor) => cursor.try_collect().await.unwrap_or_default(),
        Err(_) => Vec::new(),
    };

    let total_accounts = all_accounts.len();
    let active_accounts = all_accounts.iter().filter(|a| a.is_active).count();
    let healthy_accounts = all_accounts
        .iter()
        .filter(|a| a.is_active && a.health_status.is_valid)
        .count();
    let accounts_in_error = all_accounts
        .iter()
        .filter(|a| !a.health_status.is_valid)
        .count();

    // 5. Items scrapés
    let scraped_col = state.db.collection::<ScrapedItem>("scraped_items");
    let total_scraped = scraped_col.count_documents(doc! {}).await.unwrap_or(0);
    let new_scraped = scraped_col
        .count_documents(doc! { "curation_status": "NEW" })
        .await
        .unwrap_or(0);
    let queued_scraped = scraped_col
        .count_documents(doc! { "curation_status": "QUEUED" })
        .await
        .unwrap_or(0);
    let rejected_scraped = scraped_col
        .count_documents(doc! { "curation_status": "REJECTED" })
        .await
        .unwrap_or(0);

    // 6. Sources
    let sources_col = state.db.collection::<Source>("sources");
    let total_sources = sources_col.count_documents(doc! {}).await.unwrap_or(0);
    let active_sources = sources_col
        .count_documents(doc! { "is_active": true })
        .await
        .unwrap_or(0);

    (
        StatusCode::OK,
        Json(json!({
            "today": today_str,
            "quota": {
                "published_count": published_count,
                "max_limit": max_limit,
                "remaining": max_limit.saturating_sub(published_count),
                "is_full": published_count >= max_limit,
                "history": history
            },
            "next_slot": {
                "label": next_slot_label,
                "target_time": next_slot_time
            },
            "publications": {
                "pending": pending_pubs_count,
                "published": published_pubs_count,
                "failed": failed_pubs_count
            },
            "social_accounts": {
                "total": total_accounts,
                "active": active_accounts,
                "healthy": healthy_accounts,
                "errors": accounts_in_error
            },
            "curation": {
                "total": total_scraped,
                "new": new_scraped,
                "queued": queued_scraped,
                "rejected": rejected_scraped
            },
            "sources": {
                "total": total_sources,
                "active": active_sources
            }
        })),
    )
        .into_response()
}
