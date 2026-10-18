use bson::doc;
use mongodb::{
    options::{ClientOptions, IndexOptions},
    Client, Database, IndexModel,
};
use std::env;
use tracing::info;

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub port: u16,
    pub mongodb_uri: String,
    pub database_name: String,
    pub master_key: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        let port = env::var("PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .expect("PORT doit être un entier valide");

        let mongodb_uri = env::var("MONGODB_URI")
            .unwrap_or_else(|_| "mongodb://mongodb:27017/social_media_manager".to_string());

        let database_name = env::var("DATABASE_NAME")
            .unwrap_or_else(|_| "social_media_manager".to_string());

        let master_key = env::var("MASTER_KEY")
            .unwrap_or_else(|_| "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string());

        Self {
            port,
            mongodb_uri,
            database_name,
            master_key,
        }
    }
}

/// Initialise la connexion MongoDB et applique les index obligatoires
pub async fn init_mongodb(config: &AppConfig) -> Result<Database, mongodb::error::Error> {
    info!("Connexion à MongoDB sur {}...", config.mongodb_uri);

    let client_options = ClientOptions::parse(&config.mongodb_uri).await?;
    let client = Client::with_options(client_options)?;
    let db = client.database(&config.database_name);

    // Vérification de la connexion avec un ping
    db.run_command(doc! { "ping": 1 }).await?;
    info!("Connecté à MongoDB avec succès (Base: {})", config.database_name);

    // Création des index automatiques
    init_indexes(&db).await?;

    Ok(db)
}

async fn init_indexes(db: &Database) -> Result<(), mongodb::error::Error> {
    // 1. Index unique sur scraped_items.url_hash pour la déduplication
    let scraped_items_col = db.collection::<bson::Document>("scraped_items");
    let url_hash_opts = IndexOptions::builder().unique(true).build();
    let url_hash_index = IndexModel::builder()
        .keys(doc! { "url_hash": 1 })
        .options(url_hash_opts)
        .build();
    scraped_items_col.create_index(url_hash_index).await?;
    info!("Index unique créé sur scraped_items.url_hash");

    // 2. Index composé sur publications { status: 1, priority: -1, created_at: 1 }
    let publications_col = db.collection::<bson::Document>("publications");
    let pub_index = IndexModel::builder()
        .keys(doc! { "status": 1, "priority": -1, "created_at": 1 })
        .build();
    publications_col.create_index(pub_index).await?;
    info!("Index composé créé sur publications {{ status: 1, priority: -1, created_at: 1 }}");

    // 3. Index sur sources.is_active
    let sources_col = db.collection::<bson::Document>("sources");
    let source_index = IndexModel::builder()
        .keys(doc! { "is_active": 1 })
        .build();
    sources_col.create_index(source_index).await?;
    info!("Index créé sur sources.is_active");

    Ok(())
}
