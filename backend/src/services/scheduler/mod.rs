use crate::crypto::CryptoService;
use crate::models::{
    DailyQuota, DeliveryStatus, PlatformCredentials, Publication, PublicationStatus, SocialAccount,
};
use crate::services::publisher::PublisherService;
use bson::{doc, oid::ObjectId};
use chrono::{DateTime, NaiveTime, Timelike, Utc};
use mongodb::options::{FindOneAndUpdateOptions, ReturnDocument, UpdateOptions};
use mongodb::Database;
use thiserror::Error;
use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};
use tracing::{error, info, warn};

#[derive(Debug, Error)]
pub enum SchedulerError {
    #[error("Erreur de planification du job : {0}")]
    JobScheduler(#[from] JobSchedulerError),

    #[error("Erreur base de données : {0}")]
    Database(#[from] mongodb::error::Error),

    #[error("Erreur de déchiffrement : {0}")]
    Crypto(String),

    #[error("Publication non trouvée : {0}")]
    PublicationNotFound(String),

    #[error("Quota journalier atteint (5/5)")]
    QuotaExceeded,
}

#[derive(Clone)]
pub struct SchedulerService {
    db: Database,
    crypto: CryptoService,
    publisher_service: PublisherService,
}

impl SchedulerService {
    pub fn new(db: Database, crypto: CryptoService, publisher_service: PublisherService) -> Self {
        Self {
            db,
            crypto,
            publisher_service,
        }
    }

    /// Démarre le scheduler tokio-cron-scheduler avec l'expression exacte 0 0 6,9,12,15,18 * * *
    pub async fn start(&self) -> Result<JobScheduler, SchedulerError> {
        let sched = JobScheduler::new().await?;
        let self_clone = self.clone();

        // Expression Cron exacte : 06h, 09h, 12h, 15h, 18h (secondes, minutes, heures, jour, mois, jour_semaine)
        let cron_expr = "0 0 6,9,12,15,18 * * *";
        info!("Enregistrement du Job Cron de publication: '{}'", cron_expr);

        let job = Job::new_async(cron_expr, move |_uuid, _lock| {
            let service = self_clone.clone();
            Box::pin(async move {
                let slot = format!("CRON_{:02}h", Utc::now().hour());
                info!("Déclenchement automatique du créneau de publication : {}", slot);
                if let Err(e) = service.process_next_scheduled_publication(&slot).await {
                    error!("Erreur lors de l'exécution du créneau {}: {}", slot, e);
                }
            })
        })?;

        sched.add(job).await?;
        sched.start().await?;
        info!("Scheduler de publication démarré avec succès.");

        Ok(sched)
    }

    /// Traite la prochaine publication programmée selon les règles métier strictes
    pub async fn process_next_scheduled_publication(
        &self,
        slot_name: &str,
    ) -> Result<Option<ObjectId>, SchedulerError> {
        let today_str = Utc::now().format("%Y-%m-%d").to_string();
        let daily_quotas_col = self.db.collection::<DailyQuota>("daily_quotas");

        // Règle 4.a : Vérifie dans daily_quotas à la date du jour que published_count < 5
        let quota_doc = daily_quotas_col
            .find_one(doc! { "_id": &today_str })
            .await?;

        let current_count = quota_doc.map(|q| q.published_count).unwrap_or(0);
        if current_count >= 5 {
            warn!(
                "Quota journalier atteint ({}/5) pour la date {}. Aucune publication déclenchée.",
                current_count, today_str
            );
            return Ok(None);
        }

        // Règle 4.b : Récupère et verrouille atomiquement la prochaine publication via find_one_and_update
        let publications_col = self.db.collection::<Publication>("publications");
        let filter = doc! { "status": "PENDING" };
        let update = doc! { "$set": { "status": "PROCESSING" } };
        let opts = FindOneAndUpdateOptions::builder()
            .sort(doc! { "priority": -1, "created_at": 1 })
            .return_document(ReturnDocument::After)
            .build();

        let locked_pub = publications_col
            .find_one_and_update(filter, update)
            .with_options(opts)
            .await?;

        let mut publication = match locked_pub {
            Some(p) => p,
            None => {
                info!("Aucune publication en attente (status PENDING).");
                return Ok(None);
            }
        };

        let pub_id = publication.id.unwrap_or_default();
        info!("Publication verrouillée pour traitement : {}", pub_id);

        // Exécute les deliveries
        self.execute_deliveries(&mut publication, slot_name, &today_str).await?;

        Ok(Some(pub_id))
    }

    /// Exécute immédiatement une publication sans attendre le cron (avec respect ou override du quota)
    pub async fn publish_immediately(
        &self,
        pub_id: ObjectId,
    ) -> Result<Publication, SchedulerError> {
        let publications_col = self.db.collection::<Publication>("publications");
        let filter = doc! { "_id": pub_id };
        let update = doc! { "$set": { "status": "PROCESSING" } };
        let opts = FindOneAndUpdateOptions::builder()
            .return_document(ReturnDocument::After)
            .build();

        let mut publication = publications_col
            .find_one_and_update(filter, update)
            .with_options(opts)
            .await?
            .ok_or_else(|| SchedulerError::PublicationNotFound(pub_id.to_hex()))?;

        let today_str = Utc::now().format("%Y-%m-%d").to_string();
        self.execute_deliveries(&mut publication, "MANUAL_TRIGGER", &today_str).await?;

        Ok(publication)
    }

    /// Exécute l'envoi des deliveries pour une publication
    async fn execute_deliveries(
        &self,
        publication: &mut Publication,
        slot_name: &str,
        today_str: &str,
    ) -> Result<(), SchedulerError> {
        let pub_id = publication.id.unwrap_or_default();
        let social_col = self.db.collection::<SocialAccount>("social_accounts");

        let mut successful_deliveries = 0;
        let mut failed_deliveries = 0;

        for delivery in &mut publication.deliveries {
            // Traiter uniquement les deliveries PENDING ou FAILED
            if delivery.status == DeliveryStatus::Sent {
                successful_deliveries += 1;
                continue;
            }

            // Récupération du compte social
            let account = match social_col.find_one(doc! { "_id": delivery.account_id }).await {
                Ok(Some(acc)) => acc,
                _ => {
                    delivery.status = DeliveryStatus::Failed;
                    delivery.error_message = Some("Compte social introuvable ou supprimé".into());
                    failed_deliveries += 1;
                    continue;
                }
            };

            if !account.is_active {
                delivery.status = DeliveryStatus::Failed;
                delivery.error_message = Some("Compte social inactif".into());
                failed_deliveries += 1;
                continue;
            }

            // Déchiffrement AES-256-GCM des credentials
            let decrypted_raw = match self.crypto.decrypt(&account.credentials_encrypted) {
                Ok(s) => s,
                Err(e) => {
                    error!("Erreur de déchiffrement pour le compte {}: {}", delivery.account_id, e);
                    delivery.status = DeliveryStatus::Failed;
                    delivery.error_message = Some(format!("Erreur de déchiffrement: {}", e));
                    failed_deliveries += 1;
                    continue;
                }
            };

            let credentials: PlatformCredentials = match serde_json::from_str(&decrypted_raw) {
                Ok(c) => c,
                Err(e) => {
                    delivery.status = DeliveryStatus::Failed;
                    delivery.error_message = Some(format!("Format credentials invalide: {}", e));
                    failed_deliveries += 1;
                    continue;
                }
            };

            // Sélection du publisher
            let publisher = match self.publisher_service.get_publisher(&delivery.platform) {
                Ok(p) => p,
                Err(e) => {
                    delivery.status = DeliveryStatus::Failed;
                    delivery.error_message = Some(e.to_string());
                    failed_deliveries += 1;
                    continue;
                }
            };

            // Contenu à publier (custom_text ou default_text)
            let text_to_send = delivery
                .custom_text
                .as_deref()
                .unwrap_or(&publication.default_text);

            info!(
                "Publication en cours vers {:?} pour le compte '{}'...",
                delivery.platform, account.label
            );

            match publisher
                .publish(text_to_send, &publication.media_urls, &credentials)
                .await
            {
                Ok(ext_id) => {
                    info!("Publication réussie sur {:?} (ID: {})", delivery.platform, ext_id);
                    delivery.status = DeliveryStatus::Sent;
                    delivery.external_post_id = Some(ext_id);
                    delivery.published_at = Some(Utc::now());
                    delivery.error_message = None;
                    successful_deliveries += 1;
                }
                Err(e) => {
                    error!("Échec publication sur {:?}: {}", delivery.platform, e);
                    delivery.status = DeliveryStatus::Failed;
                    delivery.error_message = Some(e.to_string());
                    failed_deliveries += 1;
                }
            }
        }

        // Règle 4.d : Détermination du statut global
        let new_status = if successful_deliveries > 0 && failed_deliveries == 0 {
            PublicationStatus::Published
        } else if successful_deliveries > 0 && failed_deliveries > 0 {
            PublicationStatus::PartialFailure
        } else {
            PublicationStatus::Failed
        };

        publication.status = new_status;
        if successful_deliveries > 0 {
            publication.published_at = Some(Utc::now());
        }

        // Sauvegarde de l'état de la publication en base
        let publications_col = self.db.collection::<Publication>("publications");
        publications_col
            .replace_one(doc! { "_id": pub_id }, &*publication)
            .await?;

        // Si au moins un envoi a réussi, on incrémente published_count dans daily_quotas
        if successful_deliveries > 0 {
            let daily_quotas_col = self.db.collection::<bson::Document>("daily_quotas");
            let history_entry = doc! {
                "publication_id": pub_id,
                "slot": slot_name,
                "executed_at": Utc::now()
            };

            let update_quota = doc! {
                "$inc": { "published_count": 1 },
                "$setOnInsert": { "max_limit": 5 },
                "$push": { "history": history_entry }
            };

            let opts = UpdateOptions::builder().upsert(true).build();
            daily_quotas_col
                .update_one(doc! { "_id": today_str }, update_quota)
                .with_options(opts)
                .await?;

            info!(
                "Quota journalier mis à jour (+1) pour {} après diffusion réussie de {}",
                today_str, pub_id
            );
        }

        Ok(())
    }

    /// Calcule le prochain créneau horaire parmi [06h, 09h, 12h, 15h, 18h]
    pub fn get_next_slot() -> (String, DateTime<Utc>) {
        let now = Utc::now();
        let target_hours = [6, 9, 12, 15, 18];

        for &hour in &target_hours {
            let slot_time = now
                .date_naive()
                .and_time(NaiveTime::from_hms_opt(hour, 0, 0).unwrap())
                .and_utc();

            if slot_time > now {
                return (format!("{:02}:00 UTC", hour), slot_time);
            }
        }

        // Si tous les créneaux d'aujourd'hui sont passés, le prochain est demain à 06h00
        let tomorrow = now.date_naive().succ_opt().unwrap_or(now.date_naive());
        let tomorrow_slot = tomorrow
            .and_time(NaiveTime::from_hms_opt(6, 0, 0).unwrap())
            .and_utc();

        ("Demain 06:00 UTC".to_string(), tomorrow_slot)
    }
}
