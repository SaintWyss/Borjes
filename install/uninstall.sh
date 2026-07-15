#!/usr/bin/env bash
set -euo pipefail

PREFIX="${BORJES_PREFIX:-$HOME/.local}"

systemctl --user disable --now borjes.service 2>/dev/null || true
rm -f "$HOME/.config/systemd/user/borjes.service"
rm -f "$HOME/.local/share/applications/borjes.desktop"
rm -f "$PREFIX/bin/borjes" "$PREFIX/bin/borjes-toggle"
rm -rf "$PREFIX/share/borjes"
systemctl --user daemon-reload

printf '%s\n' "Borjes uninstalled. User data in ~/.config/borjes was preserved."
