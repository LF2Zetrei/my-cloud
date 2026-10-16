#!/bin/bash

set -e

ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT"
BIN="$ROOT/target/release"

# Arrêter tous les services proprement : "kill 0" envoie le signal à tout le
# groupe de processus, y compris les enfants de npm (next dev).
trap 'trap - SIGINT SIGTERM; echo ""; echo "Stopping..."; kill 0 2>/dev/null; exit 0' SIGINT SIGTERM

echo "🔨 Building workspace..."
cargo build --workspace --release

if [ ! -d "$ROOT/dashboard/node_modules" ]; then
    echo "📦 Installing dashboard dependencies..."
    (cd "$ROOT/dashboard" && npm install)
fi

echo ""
echo "🚀 Starting services..."

# Les binaires sont lancés directement (pas via "cargo run") pour que le signal
# d'arrêt atteigne bien le serveur et pas seulement cargo.
"$BIN/observability" &
echo "[STARTED] observability"

"$BIN/storage" &
echo "[STARTED] storage"

# Note: sudo requis uniquement quand start=true (OverlayFS + cgroups)
"$BIN/orchestrator" &
echo "[STARTED] orchestrator"

"$BIN/network" &
echo "[STARTED] network"

(cd "$ROOT/dashboard" && npm run dev) &
echo "[STARTED] dashboard"

echo ""
echo "✅ All services started !"
echo ""
echo "  Dashboard  → http://localhost:3000"
echo "  Proxy      → http://localhost:8080"
echo "  Metrics    → http://localhost:8080/metrics"
echo "  Health     → http://localhost:8080/health"
echo ""
echo "Press Ctrl+C to stop all services"

wait
