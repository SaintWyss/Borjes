#!/usr/bin/env sh
set -eu

systemctl --user disable --now borjes.service 2>/dev/null || true
rm -f \
    "${HOME}/.local/bin/borjes" \
    "${HOME}/.local/bin/borjes-toggle" \
    "${HOME}/.local/share/applications/borjes.desktop" \
    "${HOME}/.config/systemd/user/borjes.service"
rm -rf "${HOME}/.local/share/borjes"
systemctl --user daemon-reload

printf '%s\n' "Borjes desinstalado. La configuración personal se conservó en ~/.config/borjes."
