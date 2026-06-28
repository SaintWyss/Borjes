---
phase: 09-autostart-integracion
status: complete
---

# Summary — Phase 9: Autostart + integración escritorio

Config-only phase (no code). Executed directly on saint-acer.

## What was built

- `~/.config/autostart/datatree-rs.desktop` — KDE autostart entry that launches
  the release binary (hidden, `visible: false`) at login, so Meta+1 reveals the
  overlay without a manual start. `desktop-file-validate` clean.
- The launch-or-toggle wrapper `~/.local/bin/datatree-toggle` (added during the
  Phase 7/8 Meta+1 work) relaunches the overlay if it was ever closed.

## Verification (success criteria)

1. ✅ Autostart entry created (per-machine) — launches the app at login.
2. ✅ Registered in sync-control system-ops.tsv as op 018 (user OK: "fase 9").
3. ✅ Simulated login launch: process comes up hidden (`wmctrl` shows no mapped
   window until Meta+1), pid confirmed. Real reboot will exercise it at next login.

## Notes

Per-machine (KDE config does not sync). Replicate on other machines via
sync-control op 018. Phase 6 abandoned the literal "desktop window" type in
favor of the on-demand overlay, so "aparece como escritorio" here means the
process runs at login ready for Meta+1, not a permanent wallpaper-layer window.

## Milestone v2 "Desktop mode" — COMPLETE (9/9 phases)
Overlay on-demand (F6) + file-burst on click (F7) + pins/persistence (F8) +
autostart (F9), all shipped on acer.
