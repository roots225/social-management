mod config;
mod crypto;
mod models;
mod routes;
mod services;

use config::{init_mongodb, AppConfig};
use crypto::CryptoService;
use mongodb::Database;
use services::{PublisherService, SchedulerService, ScraperService};
use std::net::SocketAddr;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub crypto: CryptoService,
    pub scraper: ScraperService,
    pub scheduler: SchedulerService,
    pub publisher: PublisherService,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialisation du système de logs Tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Démarrage de la plateforme Social Media Manager (Rust Backend)...");

    // Chargement de la configuration
    let config = AppConfig::from_env();
    info!("Configuration chargée (Port: {}, DB: {})", config.port, config.database_name);

    // Initialisation du service de chiffrement AES-256-GCM
    let crypto = CryptoService::new(&config.master_key)
        .expect("Erreur fatale lors de l'initialisation du chiffrement AES-256-GCM avec la MASTER_KEY");
    info!("Service de chiffrement AES-256-GCM initialisé.");

    // Connexion MongoDB & Indexation
    let db = init_mongodb(&config).await.expect("Impossible de se connecter à MongoDB");

    // Initialisation des services
    let publisher = PublisherService::new();
    let scraper = ScraperService::new(db.clone());
    let scheduler = SchedulerService::new(db.clone(), crypto.clone(), publisher.clone());

    // Démarrage du Cron Scheduler (0 0 6,9,12,15,18 * * *)
    let _job_scheduler = scheduler
        .start()
        .await
        .expect("Erreur lors du démarrage du Job Scheduler");

    // État global de l'application Axum
    let state = AppState {
        db,
        crypto,
        scraper,
        scheduler,
        publisher,
    };

    // Création du routeur Axum
    let app = routes::create_router(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    info!("Serveur HTTP Axum en écoute sur http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("Serveur backend arrêté proprement.");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Échec d'écoute de l'événement Ctrl+C");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Échec d'écoute du signal SIGTERM")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Signal Ctrl+C reçu, arrêt gracieux du serveur...");
        },
        _ = terminate => {
            info!("Signal SIGTERM reçu, arrêt gracieux du serveur...");
        },
    }
}
