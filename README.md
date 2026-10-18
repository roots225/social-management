# 🚀 Social Media Manager (SMM) — Monorepo

Plateforme automatisée de veille, de scraping (sites d'actualités et offres d'emploi) et de diffusion programmée sur les réseaux sociaux (X/Twitter v2, Facebook Graph API, extensible à LinkedIn et Telegram), pilotée par un tableau de bord React (TypeScript).

---

## 🏛️ Architecture Globale

```text
/
├── docker-compose.yml          # Orchestration globale (mongodb, backend, frontend)
├── .env.example                # Variables d'environnement (DB, MASTER_KEY, Ports)
├── .env                        # Configuration active locale
├── backend/                    # API REST + Scraper + Scheduler en Rust
│   ├── Dockerfile              # Multi-stage build (cargo-chef -> debian-slim)
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs             # Point d'entrée, démarrage Axum & tokio-cron
│       ├── config/             # Chargement env & connexion MongoDB avec index automatiques
│       ├── models/             # Modèles BSON/Serde des 5 collections
│       ├── crypto/             # Chiffrement AES-256-GCM (clé 32 octets) & masquage
│       ├── services/
│       │   ├── scraper/        # Moteur de scraping (reqwest + scraper + SHA-256)
│       │   ├── publisher/      # Pattern Strategy (Trait SocialPublisher : Twitter / Facebook)
│       │   └── scheduler/      # Cron 0 0 6,9,12,15,18 * * * & verrou journalier (5 max)
│       └── routes/             # Endpoints REST Axum (v1)
└── frontend/                   # Dashboard d'administration en React 18 + TypeScript + Vite
    ├── Dockerfile              # Multi-stage build (node:20-alpine -> nginx:alpine)
    ├── nginx.conf              # Reverse proxy /api/ vers backend:8080 + routing SPA
    ├── package.json
    └── src/
        ├── api/                # Client Axios typé
        ├── types/              # Modèles TypeScript stricts
        └── views/              # 5 Vues métier (Dashboard, Curation, Queue, Sources, Social)
```

---

## ⚡ Démarrage Rapide avec Docker Compose

### 1. Variables d'environnement
Copiez `.env.example` vers `.env` (déjà initialisé avec une clé de test 32 octets) :
```bash
cp .env.example .env
```

### 2. Lancement de la pile complète
```bash
docker compose up -d --build
```

Services disponibles :
- **Dashboard Web (Frontend React)** : [http://localhost:3000](http://localhost:3000)
- **API REST Rust** : [http://localhost:8080/api/v1/health](http://localhost:8080/api/v1/health)
- **MongoDB 7** : `mongodb://localhost:27017/social_media_manager`

---

## 🔒 Sécurité & Cryptographie (AES-256-GCM)

- **Chiffrement au repos** : Tous les secrets de connexion des réseaux sociaux (`api_secret`, `access_token`, `page_access_token`, `app_secret`) sont chiffrés avec **AES-256-GCM** et un nonce de 96 bits aléatoire généré par `rand::OsRng`.
- **Clé Maître (MASTER_KEY)** : Clé symétrique de 32 octets (format hexadécimal 64 caractères ou chaîne de 32 caractères) stockée exclusivement dans les variables d'environnement.
- **Masquage API** : Lors des requêtes `GET /api/v1/social-accounts`, aucun secret en clair n'est retourné (format masqué sécurisé `****abcd`).

---

## 🕒 Règles Métier & Automatisation

1. **Déduplication au Scraping** :
   - Calcul d'un hash SHA-256 de chaque URL trouvée.
   - Index unique MongoDB sur `scraped_items.url_hash`.
   - Insertion atomique et idempotente (les doublons sont ignorés sans interruption).

2. **Verrou Quota Journalier (5 publications / jour maximum)** :
   - Expression Cron stricte : `0 0 6,9,12,15,18 * * *` (06h, 09h, 12h, 15h, 18h UTC).
   - À chaque slot, le scheduler vérifie `daily_quotas.published_count < 5`.
   - Verrouillage atomique via `find_one_and_update` (`status: PENDING` -> `PROCESSING`, trié par priorité décroissante).
   - Envoi via le Trait `SocialPublisher` pour chaque compte social associé.
   - Mise à jour du statut global (`PUBLISHED`, `PARTIAL_FAILURE`, `FAILED`) et incrémentation atomique du quota si au moins une diffusion a réussi.

---

## 📡 Endpoints REST (`/api/v1`)

| Méthode | Route | Description |
|---|---|---|
| `GET` | `/sources` | Liste des sources de scraping |
| `POST` | `/sources` | Ajout d'une nouvelle source avec sélecteurs CSS |
| `PUT` | `/sources/:id` | Mise à jour d'une source |
| `DELETE` | `/sources/:id` | Suppression d'une source |
| `POST` | `/sources/:id/scrape-now` | Déclenchement manuel immédiat du scraping |
| `GET` | `/scraped-items` | Flux d'articles/emplois (filtres statut & catégorie) |
| `PUT` | `/scraped-items/:id/status` | Mise à jour statut de curation (NEW, QUEUED, REJECTED) |
| `POST` | `/scraped-items/:id/queue` | Transformation en publication PENDING avec livraisons |
| `GET` | `/publications` | File d'attente (triée par priorité et date) |
| `PUT` | `/publications/:id` | Édition du texte par défaut ou customisé par réseau |
| `DELETE` | `/publications/:id` | Suppression d'une publication |
| `POST` | `/publications/:id/retry` | Relance uniquement les livraisons en échec |
| `POST` | `/publications/:id/publish-now` | Déclenchement manuel d'une publication |
| `GET` | `/social-accounts` | Liste des comptes connectés (secrets masqués) |
| `POST` | `/social-accounts` | Ajout d'un compte (chiffrement AES-256-GCM) |
| `DELETE` | `/social-accounts/:id` | Suppression d'un compte |
| `POST` | `/social-accounts/:id/verify` | Test de validité auprès de l'API distante |
| `GET` | `/dashboard/stats` | Quota du jour, prochain créneau et compteurs |
