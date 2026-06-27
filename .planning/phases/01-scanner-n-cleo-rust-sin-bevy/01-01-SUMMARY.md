# Summary — Phase 01 Plan 01

## Archivos creados

- `Cargo.toml`: crate binario `datatree-rs`, edition 2021, dependencia `ignore = "0.4"`.
- `src/main.rs`: CLI debug que escanea `~/Data`, imprime cada nodo y el total.
- `src/scan.rs`: `FolderNode`, `scan_folder`, exclusiones, manejo de symlinks y 7 tests.

## Verificacion

- `cargo fmt -- --check`: OK.
- `cargo build`: bloqueado. El entorno no puede resolver `index.crates.io` y `ignore` no esta en cache local.
- `cargo build --offline`: bloqueado. Cargo reporta `no matching package named 'ignore' found`.
- `cargo test`: no ejecutado por el mismo bloqueo de dependencia.

## Desviaciones del plan

- `~/Data` es symlink a `/mnt/hdd/Data`; `scan_folder` canonicaliza solo la raiz antes de construir el walker y mantiene `follow_links(false)` para symlinks internos.
- El snapshot real de `/mnt/hdd/Data` no coincide con la research: `01_Now` es un directorio normal, no un symlink. El test conserva la validacion de unicidad de `01_Now`, no inventa el symlink, y exige que existan symlinks reales a directorios en el scan.
- El conteo real actual es mucho mayor que `580..=650`; el test usa un rango `4700..=4900` con comentario porque el arbol real contiene mas datos.
