---
phase: 05-lectura-visual-ra-z-configurable-polish
plan: 01
status: done
---

# 05-01 SUMMARY

## Cambios

- `src/render.rs`
  - Agregada `node_scale(child_count: usize) -> f32` con compresión `sqrt` y clamp `[0.5, 3.0]`.
  - Aplicado `Transform::with_scale(Vec3::splat(node_scale(node.child_count)))` al spawn de nodos.
  - Conservado un único mesh compartido `Sphere::new(0.5)`.
  - Agregado material compartido reservado para symlinks con color magenta brillante, distinto de blanco y de la paleta de clusters.
  - `OriginalMaterial` guarda el material efectivamente asignado, sea cluster o symlink.
  - Revisada la paleta de 10 colores de clusters para contraste sobre fondo oscuro.

## Tests / verificación

- `cargo test --lib render`: OK.
- `cargo build`: OK.
- `cargo test`: OK.
- `cargo clippy --all-targets -- -D warnings`: OK.
- Grep de aceptación: `Sphere::new` aparece 1 vez; `node_scale`, `with_scale` e `is_symlink` presentes.

## Desviaciones

- Sin desviaciones funcionales. No se agregaron dependencias y `Cargo.toml` quedó sin cambios.
