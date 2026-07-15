#!/usr/bin/env sh
set -eu

repo_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
bin_dir="${HOME}/.local/bin"
data_dir="${HOME}/.local/share/borjes"
app_dir="${HOME}/.local/share/applications"
service_dir="${HOME}/.config/systemd/user"

cd "$repo_dir"
cargo build --locked --release

install -d "$bin_dir" "$data_dir" "$app_dir" "$service_dir"
install -m 0755 target/release/borjes "$bin_dir/borjes"
install -m 0755 scripts/borjes-toggle "$bin_dir/borjes-toggle"
cp -R assets/. "$data_dir/"
install -m 0644 packaging/borjes.desktop "$app_dir/borjes.desktop"
install -m 0644 packaging/borjes.service "$service_dir/borjes.service"

systemctl --user daemon-reload
systemctl --user enable --now borjes.service
printf '%s\n' "Borjes instalado. Asigná tu atajo global a: ${bin_dir}/borjes-toggle"
