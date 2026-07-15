#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PREFIX="${BORJES_PREFIX:-$HOME/.local}"
BIN_DIR="$PREFIX/bin"
SHARE_DIR="$PREFIX/share/borjes"
APP_DIR="$HOME/.local/share/applications"
SERVICE_DIR="$HOME/.config/systemd/user"

cargo build --manifest-path "$ROOT/Cargo.toml" --release --locked

install -Dm755 "$ROOT/target/release/borjes" "$BIN_DIR/borjes"
install -Dm755 "$ROOT/install/borjes-toggle" "$BIN_DIR/borjes-toggle"
mkdir -p "$SHARE_DIR"
rm -rf "$SHARE_DIR/assets"
cp -R "$ROOT/assets" "$SHARE_DIR/assets"

mkdir -p "$APP_DIR" "$SERVICE_DIR"
sed "s|@PREFIX@|$PREFIX|g" "$ROOT/install/borjes.desktop" > "$APP_DIR/borjes.desktop"
sed "s|@PREFIX@|$PREFIX|g" "$ROOT/install/borjes.service" > "$SERVICE_DIR/borjes.service"

systemctl --user daemon-reload
systemctl --user enable --now borjes.service

printf '%s\n' "Borjes installed."
printf '%s\n' "Bind your global shortcut to: $BIN_DIR/borjes-toggle"
