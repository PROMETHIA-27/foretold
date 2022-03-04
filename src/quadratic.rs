use super::*;

pub struct QuadLandEvent(pub Entity);

#[derive(Component)]
pub struct QuadJump {
    start: Vec3,
    end: Vec3,
    /// Squash/stretch/flip the quadratic
    factor: f32,
    t: f32,
    /// How long in seconds it takes to go from A to B
    time: f32,
}

impl QuadJump {
    pub fn new(start: Vec3, end: Vec3, factor: f32, time: f32) -> Self {
        QuadJump {
            start,
            end,
            factor,
            t: 0.0,
            time
        }
    }
}

#[derive(Component)]
pub struct MultiQuadJump {
    jumps: Vec<QuadJump>,
}

impl MultiQuadJump {
    pub fn new(jumps: Vec<QuadJump>) -> Self {
        MultiQuadJump {
            jumps
        }
    }
}

pub fn move_quadratics(
    mut commands: Commands,
    mut q: Query<(Entity, &mut Transform, &mut QuadJump)>,
    mut multis: Query<(Entity, &mut Transform, &mut MultiQuadJump), Without<QuadJump>>,
    time: Res<Time>,
    mut events: EventWriter<QuadLandEvent>,
) {
    for (id, mut tf, mut quad) in q.iter_mut() {
        let dt = time.delta_seconds();

        let (x, y, start_to_end);
        {
            y = quad.end.y - quad.start.y;
            let height_adjusted_end = quad.end - vec3(0.0, y, 0.0);
            x = quad.start.distance(height_adjusted_end);
            start_to_end = (height_adjusted_end - quad.start).normalize();
        }

        let a = -1.0 * quad.factor;

        let b = (y - (a * x.powi(2))) / x;

        let mut t = quad.t + (dt / quad.time);
        if t > 1.0 {
            t = 1.0;
            commands.entity(id).remove::<QuadJump>();
            events.send(QuadLandEvent(id));
        }
        quad.t = t;

        let (out_x, out_y) = (
            x * t,
            a * (x * t).powi(2) + b * x * t
        );

        let mut out_pos = quad.start + (start_to_end * out_x);
        out_pos.y += out_y;

        tf.translation = out_pos;
    }

    for (id, mut tf, mut multi) in multis.iter_mut() {
        if let Some(quad) = multi.jumps.first_mut() {
            let start = quad.start;
    
            let dt = time.delta_seconds();
    
            let (x, y, start_to_end);
            {
                y = quad.end.y - start.y;
                let height_adjusted_end = quad.end - vec3(0.0, y, 0.0);
                x = start.distance(height_adjusted_end);
                start_to_end = (height_adjusted_end - start).normalize();
            }
    
            let a = -1.0 * quad.factor;
    
            let b = (y - (a * x.powi(2))) / x;
    
            let mut t = quad.t + (dt / quad.time);
            quad.t = t;
            if t > 1.0 {
                t = 1.0;
                multi.jumps.remove(0);
                events.send(QuadLandEvent(id));
            }
    
            let (out_x, out_y) = (
                x * t,
                a * (x * t).powi(2) + b * x * t
            );
    
            let mut out_pos = start + (start_to_end * out_x);
            out_pos.y += out_y;

            tf.translation = out_pos;
        } else {
            commands.entity(id).remove::<MultiQuadJump>();
        }
    }
}