# My Cloud Provider

Un cloud provider minimaliste écrit from scratch en Rust.

## Architecture
Internet
│
▼
network (proxy :8080)
├── /health     → public
├── /metrics    → public
├── /api/*      → orchestrator (:8081)
└── /storage/*  → storage (:8082)

## Services

| Service | Port | Description |
|---------|------|-------------|
| network | 8080 | Reverse proxy + load balancer + auth |
| orchestrator | 8081 | Gestion des containers |
| storage | 8082 | Object storage (style S3) |
| observability | 9090 | Métriques système |
| vpn | - | Tunnel chiffré ChaCha20-Poly1305 |
| runtime | - | Isolation Linux (namespaces + cgroups) |

## Prérequis

- Ubuntu 20.04+
- Rust 1.70+
- sudo (pour namespaces et cgroups)

## Installation

```bash
git clone https://github.com/ton-user/my-cloud
cd my-cloud
cp .env.example .env
# édite .env avec ton token
cargo build --workspace
```

## Démarrage

```bash
# Services HTTP
cargo run -p network
cargo run -p storage
cargo run -p observability

# Orchestrateur (nécessite sudo)
sudo ./target/debug/orchestrator
```

## API

### Containers
```bash
# Créer un container
curl -X POST http://localhost:8080/api/containers \
  -H "Authorization: Bearer ton-token" \
  -H "Content-Type: application/json" \
  -d '{"name":"test","image":"/bin/sleep","args":["3600"],"start":true}'

# Lister les containers
curl http://localhost:8080/api/containers

# Supprimer un container
curl -X DELETE http://localhost:8080/api/containers/{id} \
  -H "Authorization: Bearer ton-token"
```

### Storage
```bash
# Stocker un fichier
curl -X PUT http://localhost:8080/storage/fichier.txt \
  -H "Authorization: Bearer ton-token" \
  -d "contenu du fichier"

# Récupérer un fichier
curl http://localhost:8080/storage/fichier.txt
```

### Métriques
```bash
curl http://localhost:8080/metrics
```

## Technologies

- **Rust** — langage principal
- **ChaCha20-Poly1305** — chiffrement VPN
- **Linux namespaces** — isolation des containers
- **Cgroups** — limitation des ressources
- **OverlayFS** — filesystem en couches
