# 04-01 SUMMARY

## Implementado

- `Cargo.toml`: agregadas `bevy_egui = "0.40.1"`, `open = "5.3.5"` y `unicode-normalization = "0.1"`.
- `src/main.rs`: registrado `EguiPlugin::default()` junto a los plugins existentes, recursos de interaccion, y observers globales con `app.add_observer(...)`.
- `src/render.rs`: cada nodo guarda `OriginalMaterial(Handle<StandardMaterial>)` junto al `MeshMaterial3d` de cluster.
- `src/interaction.rs`: creado con `Selected`, `HighlightMaterial`, `OriginalMaterial`, observers de hover/out/click, guard egui y doble-click manual.

## Detalles de API

- Picking Bevy 0.19: observers `On<Pointer<Over>>`, `On<Pointer<Out>>`, `On<Pointer<Click>>`.
- `bevy_egui` 0.40.1: `EguiContexts` con `contexts.ctx_mut()?`; guard `is_pointer_over_area()` al inicio del click.
- Doble-click: `Local<Option<(Instant, Entity)>>`, misma entidad y umbral de 300 ms.
- Apertura: `std::thread::spawn(move || { let _ = open::that(path); })`.

## Verificacion

- `cargo fmt`: OK.
- Greps de deps, observers, `open::that`, `thread::spawn`, `is_pointer_over_area` y `Local<Option<(Instant, Entity)>>`: OK.
- `cargo build`: fallo antes de compilar por DNS de `index.crates.io`.
- `cargo clippy --all-targets -- -D warnings`: fallo antes de compilar por DNS de `index.crates.io`.
- `cargo test ui::`: no corrio por red; `index.crates.io` no resolvio.
- `cargo test ui:: --offline`: confirma que falta `bevy_egui` en cache local.

## Desviaciones

- No se pudo completar `cargo build`/`clippy` porque las crates nuevas no estan descargadas y el sandbox no tiene resolucion DNS para crates.io.
- `on_node_out` conserva el highlight si el nodo tambien es match de busqueda, para no romper el estado visual de 04-02 al salir del hover.
