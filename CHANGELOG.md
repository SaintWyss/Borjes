# Changelog

## 1.0.0 — 2026-07-15

### Producto

- Instalador, desinstalador, desktop entry y servicio systemd.
- Assets independientes del checkout.
- Raíz configurable por argumento o variable de entorno.
- Migración automática `datatree-rs` → `borjes`.
- Licencia MIT y releases reproducibles.

### Rendimiento

- Event loop despertado directamente por `SIGUSR1`.
- Eliminado el polling oculto de 250 ms.
- Render adaptativo y reposo event-driven.

### Experiencia visual

- Radar contextual.
- Peso visual por estructura y frecencia.
- Bursts de archivos estables y ordenados.

### Calidad

- CI con formato, Clippy estricto, tests, release build y RustSec.
