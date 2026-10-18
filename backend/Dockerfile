# ==========================================
# Étape 1 : Préparation du cache de dépendances Cargo Chef
# ==========================================
FROM lukemathwalker/cargo-chef:latest-rust-bookworm AS chef
WORKDIR /app

FROM chef AS planner
COPY Cargo.toml Cargo.lock* ./
# Création d'un dossier src temporaire si lockfile n'existe pas encore
RUN mkdir -p src && echo "fn main() {}" > src/main.rs
RUN cargo chef prepare --recipe-path recipe.json

# ==========================================
# Étape 2 : Compilation des dépendances et du binaire
# ==========================================
FROM chef AS builder
WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json
# Build des dépendances en cache
RUN cargo chef cook --release --recipe-path recipe.json

# Copie du code source complet et compilation
COPY . .
RUN cargo build --release --bin backend

# ==========================================
# Étape 3 : Image d'exécution minimale
# ==========================================
FROM debian:bookworm-slim AS runtime
WORKDIR /app

# Installation des certificats SSL/TLS, Chromium et dépendances pour Headless Chrome
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    curl \
    chromium \
    fonts-liberation \
    && rm -rf /var/lib/apt/lists/*

# Copie du binaire compilé depuis l'étape builder
COPY --from=builder /app/target/release/backend /usr/local/bin/backend

# Variables d'environnement par défaut
ENV RUST_LOG=info \
    PORT=8080 \
    CHROME_PATH=/usr/bin/chromium

EXPOSE 8080

CMD ["/usr/local/bin/backend"]
