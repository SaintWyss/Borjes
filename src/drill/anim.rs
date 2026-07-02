use bevy::prelude::*;

use super::*;

// Grow-in (scale 0→base, ease-out) + smooth hover bump, per node.
pub fn animate_nodes(
    time: Res<Time>,
    hover: Res<HoverState>,
    mut nodes: Query<(
        &mut Transform,
        &mut NodeAnim,
        Option<&LevelChild>,
        Option<&ParentNode>,
        Option<&EchoNode>,
    )>,
) {
    let dt = time.delta_secs();
    let t = time.elapsed_secs();
    for (mut tf, mut anim, child, parent, echo) in &mut nodes {
        anim.age += dt;
        let staggered = (anim.age - anim.delay).max(0.0);
        let g = (staggered / GROW_SECS).clamp(0.0, 1.0);
        let grow = 1.0 - (1.0 - g).powi(3); // ease-out cubic

        let hovered = match hover.target {
            HoverTarget::Child(id) => child.is_some_and(|c| c.0 == id),
            HoverTarget::Parent => parent.is_some(),
            HoverTarget::Echo => echo.is_some(),
            HoverTarget::None => false,
        };
        // El eco respira despacio: recuerdo vivo, no escenografía muerta.
        let breathe = if echo.is_some() {
            1.0 + 0.04 * (t * 0.8).sin()
        } else {
            1.0
        };
        let target = anim.base * grow * breathe * if hovered { HOVER_BUMP } else { 1.0 };

        let current = tf.scale.x;
        let next = current + (target - current) * (dt * 12.0).min(1.0);
        tf.scale = Vec3::splat(next);

        // Children wobble organically around home (mock: cos(t*0.6)/sin(t*0.5)).
        if child.is_some() {
            let dx = (t * 0.6 + anim.phase).cos() * DRIFT_AMP;
            let dy = (t * 0.5 + anim.phase).sin() * DRIFT_AMP;
            tf.translation = anim.home + Vec3::new(dx, dy, 0.0);
        }
    }
}

// Re-aim each hub→child connector at the child's drifting position so the edges
// stay attached and sway with the nodes. O(n²) over children, but n is tiny.
pub fn animate_connectors(
    children: Query<(&LevelChild, &GlobalTransform)>,
    mut connectors: Query<(&Connector, &mut Transform)>,
) {
    for (conn, mut tf) in &mut connectors {
        let Some((_, gt)) = children.iter().find(|(c, _)| c.0 == conn.child) else {
            continue;
        };
        let p = gt.translation();
        let len = p.length().max(0.001);
        tf.translation = p * 0.5;
        tf.rotation = Quat::from_rotation_arc(Vec3::Y, p / len);
        tf.scale = Vec3::new(1.0, len, 1.0);
    }
}
