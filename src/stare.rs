use super::*;

#[derive(Component)]
pub struct StareAt(pub Vec3);

pub fn update_starers(
    mut q: Query<(&mut Transform, &StareAt)>,
) {
    for (mut tf, stare) in q.iter_mut() {
        tf.look_at(stare.0, Vec3::Y);
    }
}