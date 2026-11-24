#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CLI_BIN="$REPO_ROOT/target/release/agy-theme"
DAEMON_BIN="$REPO_ROOT/target/release/agy-theme-daemon"

if [ ! -x "$CLI_BIN" ] || [ ! -x "$DAEMON_BIN" ]; then
    if ! command -v cargo >/dev/null 2>&1; then
        echo "[install-unix] Error: Rust release binaries missing and 'cargo' not found in PATH." >&2
        exit 1
    fi
    echo "[install-unix] Building Rust release binaries..."
    (cd "$REPO_ROOT" && cargo build --release)
fi

cmd="${1:-install}"
shift || true

case "$cmd" in
    --foreground|foreground|run)
        exec "$CLI_BIN" --root "$REPO_ROOT" daemon "$@"
        ;;
    stop)
        exec "$CLI_BIN" --root "$REPO_ROOT" stop
        ;;
    status)
        exec "$CLI_BIN" --root "$REPO_ROOT" status
        ;;
    uninstall)
        exec "$CLI_BIN" --root "$REPO_ROOT" uninstall
        ;;
    install)
        exec "$CLI_BIN" --root "$REPO_ROOT" install "$@"
        ;;
    launch)
        exec "$CLI_BIN" --root "$REPO_ROOT" launch "$@"
        ;;
    *)
        echo "Usage: $0 [install|launch|--foreground|stop|status|uninstall] [--no-startup] [--watch-themes] [--restart]" >&2
        exit 2
        ;;
esac
