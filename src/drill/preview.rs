use bevy::prelude::*;

use super::*;

#[derive(Component)]
pub struct PreviewNode;

// Nombre de la subcarpeta que representa cada dendrita del preview.
#[derive(Component)]
pub struct PreviewLabel(pub String);

#[derive(Component)]
pub struct PreviewAnim {
    pub age: f32,
    pub full: f32,
    // Brote secuencial: cada dendrita aparece un instante después de la anterior.
    pub delay: f32,
}

#[derive(Resource, Default)]
pub struct PreviewState {
    pub target: Option<usize>,
    pub elapsed: f32,
    pub spawned_for: Option<usize>,
    pub overflow: Option<(usize, usize)>,
}

#[derive(Clone)]
pub struct CachedPreviewAssets {
    material: Handle<StandardMaterial>,
    sphere: Handle<Mesh>,
}

impl CachedPreviewAssets {
    fn build(meshes: &mut Assets<Mesh>, materials: &mut Assets<StandardMaterial>) -> Self {
        let mut material = glow_material(Color::srgba(0.3, 0.9, 0.85, 0.35), 1.2);
        material.alpha_mode = AlphaMode::Blend;
        Self {
            material: materials.add(material),
            sphere: meshes.add(Sphere::new(1.0)),
        }
    }
}

pub fn update_hover_preview(
    mut commands: Commands,
    time: Res<Time>,
    nav: Res<NavState>,
    hover: Res<HoverState>,
    children: Query<(Entity, &LevelChild, &GlobalTransform)>,
    previews: Query<Entity, With<PreviewNode>>,
    mut state: ResMut<PreviewState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut cache: Local<Option<CachedPreviewAssets>>,
) {
    let target = match hover.target {
        HoverTarget::Child(id) if !nav.dirty => Some(id),
        _ => None,
    };

    if target != state.target || nav.dirty {
        // try_despawn: si el nivel se reconstruyó, el padre ya se llevó a las
        // dendritas (relación ChildOf) y el despawn directo duplicaría.
        for entity in &previews {
            commands.entity(entity).try_despawn();
        }
        state.target = target;
        state.elapsed = 0.0;
        state.spawned_for = None;
        state.overflow = None;
    }

    let Some(id) = state.target else {
        return;
    };

    state.elapsed += time.delta_secs();
    if state.elapsed < 0.15 || state.spawned_for == Some(id) {
        return;
    }

    let own_children = nav.tree.children(id);
    if own_children.is_empty() {
        state.spawned_for = Some(id);
        return;
    }

    let Some((hovered_entity, _, _)) = children.iter().find(|(_, child, _)| child.0 == id) else {
        return;
    };

    for entity in &previews {
        commands.entity(entity).try_despawn();
    }

    let assets = cache
        .get_or_insert_with(|| CachedPreviewAssets::build(&mut meshes, &mut materials))
        .clone();
    let shown = own_children.len().min(8);

    // Anillo en el plano de pantalla (XY, la cámara mira por -Z) y como HIJOS del
    // nodo hovereado: heredan su drift y su bump, las dendritas viajan con él.
    // Empieza arriba (12 en punto) y brota en sentido horario, una por vez.
    for (idx, &preview_child) in own_children.iter().take(shown).enumerate() {
        let angle = std::f32::consts::FRAC_PI_2 - idx as f32 * std::f32::consts::TAU / shown as f32;
        let offset = Vec3::new(angle.cos() * 1.8, angle.sin() * 1.8, 0.35);
        let name = nav
            .tree
            .node(preview_child)
            .map(|node| node.name.clone())
            .unwrap_or_default();
        commands.spawn((
            Mesh3d(assets.sphere.clone()),
            MeshMaterial3d(assets.material.clone()),
            Transform::from_translation(offset).with_scale(Vec3::ZERO),
            ChildOf(hovered_entity),
            PreviewNode,
            PreviewLabel(name),
            PreviewAnim {
                age: 0.0,
                full: 0.24,
                delay: idx as f32 * 0.025,
            },
        ));
    }

    state.spawned_for = Some(id);
    state.overflow = (own_children.len() > 8).then_some((id, own_children.len() - 8));
}

pub fn animate_previews(
    time: Res<Time>,
    mut previews: Query<(&mut Transform, &mut PreviewAnim), With<PreviewNode>>,
) {
    let dt = time.delta_secs();
    for (mut tf, mut anim) in &mut previews {
        anim.age += dt;
        let staggered = (anim.age - anim.delay).max(0.0);
        let g = (staggered / 0.12).clamp(0.0, 1.0);
        // Ease-out con mini overshoot: brota, pasa apenas y asienta.
        let grow = 1.0 - (1.0 - g).powi(3);
        let pop = 1.0 + 0.25 * (g * std::f32::consts::PI).sin() * (1.0 - g);
        tf.scale = Vec3::splat(anim.full * grow * pop);
    }
}
