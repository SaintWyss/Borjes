#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="$ROOT/assets/icons"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

if ! command -v rsvg-convert >/dev/null 2>&1; then
  echo "missing rsvg-convert" >&2
  exit 1
fi

if ! command -v npm >/dev/null 2>&1; then
  echo "missing npm" >&2
  exit 1
fi

mkdir -p "$OUT"
npm pack lucide-static --pack-destination "$TMP" >/dev/null
tar -xzf "$TMP"/lucide-static-*.tgz -C "$TMP"

icons=(
  archive book-open briefcase cpu database file file-code file-text folder
  gamepad-2 globe graduation-cap hard-drive image inbox music network package
  settings shield terminal video wrench
)

for icon in "${icons[@]}"; do
  src="$TMP/package/icons/$icon.svg"
  dst="$OUT/$icon.png"
  if [[ ! -f "$src" ]]; then
    echo "missing lucide icon: $icon" >&2
    continue
  fi
  rsvg-convert \
    --width 256 \
    --height 256 \
    --keep-aspect-ratio \
    --background-color transparent \
    "$src" > "$dst"
done

echo "wrote ${#icons[@]} icons to $OUT"
