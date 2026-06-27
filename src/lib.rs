// Lints idiomáticos en Bevy: los Query tuples y la cantidad de system-params
// disparan estos lints por diseño del ECS. Allow a nivel crate (recomendación oficial Bevy).
#![allow(clippy::type_complexity, clippy::too_many_arguments)]

pub mod interaction;
pub mod layout;
pub mod render;
pub mod root;
pub mod scan;
pub mod ui;
