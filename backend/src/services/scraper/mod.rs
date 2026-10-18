use crate::models::{
    CurationStatus, ItemMetadata, ScrapedItem, Source, SourceCategory,
};
use bson::{doc, oid::ObjectId};
use chrono::Utc;
use chromiumoxide::browser::{Browser, BrowserConfig};
use futures::StreamExt;
use mongodb::Database;
use scraper::{Html, Selector};
use sha2::{Digest, Sha256};
use thiserror::Error;
use tracing::{info, warn};
use url::Url;

#[derive(Debug, Error)]
pub enum ScraperError {
    #[error("Erreur HTTP lors du téléchargement de la source : {0}")]
    Network(#[from] reqwest::Error),

    #[error("Sélecteur CSS invalide ({0}) : {1}")]
    InvalidSelector(String, String),

    #[error("Erreur d'URL : {0}")]
    UrlParse(#[from] url::ParseError),

    #[error("Erreur de base de données : {0}")]
    Database(#[from] mongodb::error::Error),

    #[error("Source non trouvée ou inactive")]
    SourceNotFound,

    #[error("Erreur Headless Chromium : {0}")]
    Headless(String),
}

#[derive(Debug, Clone)]
pub struct ScraperReport {
    pub source_id: ObjectId,
    pub source_name: String,
    pub total_found: usize,
    pub new_inserted: usize,
    pub duplicates_skipped: usize,
}

struct RawExtractedItem {
    url_hash: String,
    original_url: String,
    title: String,
    summary: Option<String>,
    image_url: Option<String>,
    metadata: Option<ItemMetadata>,
}

#[derive(Clone)]
pub struct ScraperService {
    db: Database,
    client: reqwest::Client,
}

impl ScraperService {
    pub fn new(db: Database) -> Self {
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36 SocialMediaManagerBot/1.0")
            .timeout(std::time::Duration::from_secs(20))
            .build()
            .unwrap_or_default();

        Self { db, client }
    }

    /// Calcule le SHA-256 de l'URL canonique pour la déduplication
    pub fn compute_url_hash(url_str: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(url_str.trim().as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Résout une URL relative en URL absolue avec la base_url
    fn resolve_url(base: &Url, relative_or_absolute: &str) -> Option<String> {
        let trimmed = relative_or_absolute.trim();
        if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
            Some(trimmed.to_string())
        } else {
            base.join(trimmed).ok().map(|u| u.to_string())
        }
    }

    /// Télécharge le HTML brut via reqwest (rapide, sans exécution JavaScript)
    pub async fn fetch_html_reqwest(&self, url: &str) -> Result<String, ScraperError> {
        let response = self.client.get(url).send().await?;
        let text = response.text().await?;
        Ok(text)
    }

    /// Navigue sur l'URL avec Chromium Headless (CDP via chromiumoxide) pour exécuter le JS et hydrater le DOM
    pub async fn fetch_html_headless(
        &self,
        url: &str,
        wait_for_selector: Option<&str>,
        wait_duration_ms: Option<u64>,
    ) -> Result<String, ScraperError> {
        info!("Lancement de Chromium Headless pour rendre l'URL : {}", url);

        let mut builder = BrowserConfig::builder()
            .no_sandbox()
            .arg("--disable-dev-shm-usage")
            .arg("--disable-gpu")
            .arg("--disable-extensions")
            .arg("--disable-background-networking")
            .arg("--headless=new");

        // Recherche du binaire Chromium
        if let Ok(custom_path) = std::env::var("CHROME_PATH") {
            builder = builder.chrome_executable(custom_path);
        } else {
            let candidates = [
                "/usr/bin/chromium",
                "/usr/bin/chromium-browser",
                "/usr/bin/google-chrome",
                "/usr/bin/google-chrome-stable",
                "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
            ];
            for candidate in candidates {
                if std::path::Path::new(candidate).exists() {
                    builder = builder.chrome_executable(candidate);
                    break;
                }
            }
        }

        let config = builder
            .build()
            .map_err(|e| ScraperError::Headless(format!("Configuration Chromium invalide : {}", e)))?;

        let (mut browser, mut handler) = Browser::launch(config)
            .await
            .map_err(|e| ScraperError::Headless(format!("Impossible de lancer Chromium : {}", e)))?;

        let handler_task = tokio::spawn(async move {
            while let Some(h) = handler.next().await {
                if h.is_err() {
                    break;
                }
            }
        });

        let page_res = tokio::time::timeout(std::time::Duration::from_secs(35), async {
            let page = browser
                .new_page(url)
                .await
                .map_err(|e| ScraperError::Headless(format!("Navigation vers {} échouée : {}", url, e)))?;

            // Si un sélecteur d'attente est configuré, attendre qu'il apparaisse dans le DOM
            if let Some(sel) = wait_for_selector {
                let trimmed = sel.trim();
                if !trimmed.is_empty() {
                    info!("Attente de l'élément '{}' dans le DOM via Chromium...", trimmed);
                    let _ = tokio::time::timeout(std::time::Duration::from_secs(12), async {
                        for _ in 0..24 {
                            if page.find_element(trimmed).await.is_ok() {
                                break;
                            }
                            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                        }
                    })
                    .await;
                }
            }

            // Pause pour laisser le JS hydrater le DOM et terminer les requêtes réseau
            let delay_ms = wait_duration_ms.unwrap_or(2000).clamp(500, 15000);
            tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;

            let html = page
                .content()
                .await
                .map_err(|e| ScraperError::Headless(format!("Récupération du contenu HTML échouée : {}", e)))?;

            Ok::<String, ScraperError>(html)
        })
        .await;

        let _ = browser.close().await;
        let _ = handler_task.await;

        match page_res {
            Ok(res) => res,
            Err(_) => Err(ScraperError::Headless(
                "Timeout (35s) dépassé lors de l'exécution de Chromium Headless".to_string(),
            )),
        }
    }

    /// Extrait les éléments structurés depuis le code HTML en mémoire
    fn extract_items_from_html(
        html_content: &str,
        source: &Source,
        parsed_base: &Url,
    ) -> Result<Vec<RawExtractedItem>, ScraperError> {
        let document = Html::parse_document(html_content);

        let container_sel = Selector::parse(&source.scraper_config.item_container).map_err(|e| {
            ScraperError::InvalidSelector(source.scraper_config.item_container.clone(), e.to_string())
        })?;

        let title_sel = Selector::parse(&source.scraper_config.title).map_err(|e| {
            ScraperError::InvalidSelector(source.scraper_config.title.clone(), e.to_string())
        })?;

        let url_sel = Selector::parse(&source.scraper_config.url).map_err(|e| {
            ScraperError::InvalidSelector(source.scraper_config.url.clone(), e.to_string())
        })?;

        let summary_sel = if let Some(ref s) = source.scraper_config.summary {
            Some(Selector::parse(s).map_err(|e| ScraperError::InvalidSelector(s.clone(), e.to_string()))?)
        } else {
            None
        };

        let image_sel = if let Some(ref img) = source.scraper_config.image {
            Some(Selector::parse(img).map_err(|e| ScraperError::InvalidSelector(img.clone(), e.to_string()))?)
        } else {
            None
        };

        let mut extracted = Vec::new();

        for element in document.select(&container_sel) {
            // Titre
            let title = match element.select(&title_sel).next() {
                Some(el) => el.text().collect::<Vec<_>>().join(" ").trim().to_string(),
                None => continue,
            };

            if title.is_empty() {
                continue;
            }

            // URL
            let raw_url = match element.select(&url_sel).next() {
                Some(el) => el.value().attr("href").unwrap_or(""),
                None => continue,
            };

            let full_url = match Self::resolve_url(parsed_base, raw_url) {
                Some(u) => u,
                None => continue,
            };

            let url_hash = Self::compute_url_hash(&full_url);

            // Résumé
            let summary = summary_sel.as_ref().and_then(|sel| {
                element
                    .select(sel)
                    .next()
                    .map(|el| el.text().collect::<Vec<_>>().join(" ").trim().to_string())
                    .filter(|s| !s.is_empty())
            });

            // Image
            let image_url = image_sel.as_ref().and_then(|sel| {
                element.select(sel).next().and_then(|el| {
                    el.value()
                        .attr("src")
                        .or_else(|| el.value().attr("data-src"))
                        .and_then(|src| Self::resolve_url(parsed_base, src))
                })
            });

            // Métadonnées
            let metadata = match source.category {
                SourceCategory::News => {
                    let author = source
                        .scraper_config
                        .specific_fields
                        .as_ref()
                        .and_then(|f| f.get("author"))
                        .and_then(|sel_str| Selector::parse(sel_str).ok())
                        .and_then(|sel| {
                            element
                                .select(&sel)
                                .next()
                                .map(|el| el.text().collect::<Vec<_>>().join(" ").trim().to_string())
                        });

                    let category = source
                        .scraper_config
                        .specific_fields
                        .as_ref()
                        .and_then(|f| f.get("category"))
                        .and_then(|sel_str| Selector::parse(sel_str).ok())
                        .and_then(|sel| {
                            element
                                .select(&sel)
                                .next()
                                .map(|el| el.text().collect::<Vec<_>>().join(" ").trim().to_string())
                        });

                    Some(ItemMetadata::News { author, category })
                }
                SourceCategory::JobOffer => {
                    let company = source
                        .scraper_config
                        .specific_fields
                        .as_ref()
                        .and_then(|f| f.get("company"))
                        .and_then(|sel_str| Selector::parse(sel_str).ok())
                        .and_then(|sel| {
                            element
                                .select(&sel)
                                .next()
                                .map(|el| el.text().collect::<Vec<_>>().join(" ").trim().to_string())
                        });

                    let location = source
                        .scraper_config
                        .specific_fields
                        .as_ref()
                        .and_then(|f| f.get("location"))
                        .and_then(|sel_str| Selector::parse(sel_str).ok())
                        .and_then(|sel| {
                            element
                                .select(&sel)
                                .next()
                                .map(|el| el.text().collect::<Vec<_>>().join(" ").trim().to_string())
                        });

                    let contract_type = source
                        .scraper_config
                        .specific_fields
                        .as_ref()
                        .and_then(|f| f.get("contract_type"))
                        .and_then(|sel_str| Selector::parse(sel_str).ok())
                        .and_then(|sel| {
                            element
                                .select(&sel)
                                .next()
                                .map(|el| el.text().collect::<Vec<_>>().join(" ").trim().to_string())
                        });

                    Some(ItemMetadata::JobOffer {
                        company,
                        location,
                        contract_type,
                        expires_at: None,
                    })
                }
            };

            extracted.push(RawExtractedItem {
                url_hash,
                original_url: full_url,
                title,
                summary,
                image_url,
                metadata,
            });
        }

        Ok(extracted)
    }

    /// Scrape une source spécifique par son identifiant
    pub async fn scrape_source_by_id(&self, source_id: ObjectId) -> Result<ScraperReport, ScraperError> {
        let sources_col = self.db.collection::<Source>("sources");
        let source = sources_col
            .find_one(doc! { "_id": source_id })
            .await?
            .ok_or(ScraperError::SourceNotFound)?;

        self.scrape_source(&source).await
    }

    /// Scrape une source donnée et insère les nouveaux éléments de façon dédupliquée
    pub async fn scrape_source(&self, source: &Source) -> Result<ScraperReport, ScraperError> {
        let source_id = source.id.unwrap_or_default();
        info!("Démarrage du scraping pour la source '{}' ({})", source.name, source.base_url);

        let parsed_base = Url::parse(&source.base_url)?;

        // 1. Récupération du HTML (Direct Headless si activé, sinon reqwest avec fallback automatique)
        let raw_items = if source.scraper_config.render_js.unwrap_or(false) {
            info!("Source '{}' configurée avec rendu JavaScript obligatoire (Headless Chromium)", source.name);
            let html = self
                .fetch_html_headless(
                    &source.base_url,
                    source.scraper_config.wait_for_selector.as_deref().or(Some(&source.scraper_config.item_container)),
                    source.scraper_config.wait_duration_ms,
                )
                .await?;
            Self::extract_items_from_html(&html, source, &parsed_base)?
        } else {
            // Tentative 1 : reqwest (rapide, sans overhead de navigateur)
            match self.fetch_html_reqwest(&source.base_url).await {
                Ok(html) => {
                    let items = Self::extract_items_from_html(&html, source, &parsed_base)?;
                    if items.is_empty() {
                        info!(
                            "0 élément extrait avec reqwest pour '{}'. Tentative de fallback via Chromium Headless (rendu JS dynamique)...",
                            source.name
                        );
                        match self
                            .fetch_html_headless(
                                &source.base_url,
                                source.scraper_config.wait_for_selector.as_deref().or(Some(&source.scraper_config.item_container)),
                                source.scraper_config.wait_duration_ms,
                            )
                            .await
                        {
                            Ok(headless_html) => {
                                let headless_items = Self::extract_items_from_html(&headless_html, source, &parsed_base)?;
                                if !headless_items.is_empty() {
                                    info!(
                                        "Chromium Headless a extrait avec succès {} éléments après exécution JavaScript !",
                                        headless_items.len()
                                    );
                                    headless_items
                                } else {
                                    items
                                }
                            }
                            Err(e) => {
                                warn!("Échec du fallback Chromium Headless ({}) : {}", source.name, e);
                                items
                            }
                        }
                    } else {
                        items
                    }
                }
                Err(e) => {
                    info!(
                        "Échec requête HTTP reqwest pour '{}' ({}) : bascule vers Chromium Headless...",
                        source.name, e
                    );
                    let headless_html = self
                        .fetch_html_headless(
                            &source.base_url,
                            source.scraper_config.wait_for_selector.as_deref().or(Some(&source.scraper_config.item_container)),
                            source.scraper_config.wait_duration_ms,
                        )
                        .await?;
                    Self::extract_items_from_html(&headless_html, source, &parsed_base)?
                }
            }
        };

        // 2. Traitement asynchrone et déduplication en base de données
        let total_found = raw_items.len();
        let mut new_inserted = 0;
        let mut duplicates_skipped = 0;

        let scraped_items_col = self.db.collection::<ScrapedItem>("scraped_items");

        for raw in raw_items {
            let existing = scraped_items_col
                .find_one(doc! { "url_hash": &raw.url_hash })
                .await?;

            if existing.is_some() {
                duplicates_skipped += 1;
                continue;
            }

            let item = ScrapedItem {
                id: None,
                source_id,
                url_hash: raw.url_hash,
                original_url: raw.original_url,
                title: raw.title,
                summary: raw.summary,
                image_url: raw.image_url,
                curation_status: CurationStatus::New,
                scraped_at: Utc::now(),
                metadata: raw.metadata,
            };

            match scraped_items_col.insert_one(item).await {
                Ok(_) => {
                    new_inserted += 1;
                }
                Err(e) => {
                    if e.to_string().contains("11000") {
                        duplicates_skipped += 1;
                    }
                }
            }
        }

        // 3. Mise à jour de last_scraped_at sur la source
        let sources_col = self.db.collection::<Source>("sources");
        sources_col
            .update_one(
                doc! { "_id": source_id },
                doc! { "$set": { "last_scraped_at": Utc::now() } },
            )
            .await?;

        info!(
            "Scraping terminé pour '{}' : {} trouvés, {} insérés, {} doublons ignorés",
            source.name, total_found, new_inserted, duplicates_skipped
        );

        Ok(ScraperReport {
            source_id,
            source_name: source.name.clone(),
            total_found,
            new_inserted,
            duplicates_skipped,
        })
    }
}
