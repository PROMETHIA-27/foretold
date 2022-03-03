use bevy::prelude::*;

#[derive(Component, Default)]
pub struct LerpToTarget {
    pub ratio: f32,
    pub target: Vec3,
}

pub fn lerp_to_targets(mut query: Query<(&mut Transform, &LerpToTarget)>, time: Res<Time>) {
    for (mut tf, lrp) in query.iter_mut() {
        tf.translation = tf.translation.lerp(lrp.target, lrp.ratio * time.delta_seconds());
    }
}

#[derive(Component, Default)]
pub struct SlerpToTarget {
    pub ratio: f32,
    pub target: Quat,
}

pub fn slerp_to_targets(mut query: Query<(&mut Transform, &SlerpToTarget)>, time: Res<Time>) {
    for (mut tf, slrp) in query.iter_mut() {
        tf.rotation = tf.rotation.slerp(slrp.target, slrp.ratio * time.delta_seconds());
    }
}