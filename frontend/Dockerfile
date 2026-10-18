# Étape 1 : Build de l'application React avec Node
FROM node:20-alpine AS builder

WORKDIR /app

# Copie des fichiers de dépendances
COPY package*.json ./
RUN npm ci

# Copie du code source et compilation
COPY . .
RUN npm run build

# Étape 2 : Serveur de production Nginx
FROM nginx:alpine

# Copie du build React dans le répertoire public Nginx
COPY --from=builder /app/dist /usr/share/nginx/html

# Configuration personnalisée Nginx avec reverse proxy
COPY nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
