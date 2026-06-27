---
phase: 05-lectura-visual-ra-z-configurable-polish
plan: 02
status: done
---

# 05-02 SUMMARY

## Cambios

- `src/root.rs`
  - Nuevo módulo compartido para `RootConfig`, `PendingRescan` y `expand_tilde`.
  - Default de raíz expandido desde `~/Data` a `$HOME/Data`.
  - Tests para expansión de `~`, preservación de rutas absolutas y contrato de `scan_folder` con ruta inexistente.
- `src/lib.rs` y `src/main.rs`
  - Declarado el módulo `root` en ambos targets.
- `src/main.rs`
  - Startup ahora inserta `GraphData(Vec::new())` y deja que `start_layout` haga scan + layout dentro de la task async.
  - `LayoutTask` conserva la ruta escaneada para validar errores y evitar aplicar estados obsoletos si hay un reescaneo pendiente.
  - Agregado `handle_rescan` antes de `apply_layout`: guarda pending si ya hay task, despawnea entidades `NodeRef`, limpia `Selected`, `SearchState`, `SearchTarget` y `GraphData`, actualiza la ruta y lanza una nueva task.
  - `apply_layout` muestra error si el scan termina en 0 nodos o si la ruta escaneada no existe/no es directorio.
- `src/ui.rs`
  - Agregado input de raíz junto al buscador y botón `Reescanear`.
  - Ruta inválida/inexistente setea `RootConfig.error` y no dispara `PendingRescan`.
  - Error visible en rojo en el panel.

## Tests / verificación

- `cargo build`: OK.
- `cargo test`: OK.
- `cargo clippy --all-targets -- -D warnings`: OK.
- Greps de aceptación: `Reescanear`, `fn handle_rescan`, `PendingRescan`, `With<render::NodeRef>`, `scan_folder` y `run_layout` presentes.

## Desviaciones

- Se agregó `src/root.rs` para no romper el target `lib`: `ui.rs` necesita importar `RootConfig`, `PendingRescan` y `expand_tilde`, así que esos tipos no podían vivir sólo en `main.rs`.
- En `apply_layout`, si termina una task vieja mientras hay un reescaneo pendiente para otra ruta, se descarta su resultado y se limpia `GraphData` en vez de respawnear nodos obsoletos durante un frame.
