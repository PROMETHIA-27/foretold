use bevy::prelude::*;

pub enum FlickerSettings {
    Constant {
        intensity: f32,
        color: Vec3,
        range: f32,
    },
    Sin {
        amplitude: f32,
        frequency: f32,
        color: Vec3,
        range: f32
    }
}

#[derive(Component)]
pub struct FlickerLight {
    pub settings: Vec<FlickerSettings>,
}

pub fn light_flicker(mut lights: Query<(&mut PointLight, &FlickerLight)>, time: Res<Time>) {
    for (mut light, flicker) in lights.iter_mut() {
        let mut color = Vec3::default();
        let mut color_weight = 0.;
        let mut intensity = 0.;
        let mut range = 0.;

        for setting in &flicker.settings {
            match setting {
                FlickerSettings::Constant{ intensity: int, color: col, range: r } => { 
                    intensity += int; 
                    color += *col;
                    color_weight += int; 
                    range += r;
                },
                FlickerSettings::Sin { amplitude: amp, frequency: freq, color: col, range: r } => {
                    let wave_val = ((freq * time.time_since_startup().as_secs_f32() * 2. * std::f32::consts::PI).sin() * amp) + amp;
        
                    color = color.lerp(*col, (wave_val / color_weight).clamp(0., 1.));
                    color_weight += wave_val;
                    intensity += wave_val;
                    range += wave_val / (amp * 2.);
                }
            }
        }

        light.color = Color::rgb(color.x, color.y, color.z);
        light.intensity = intensity;
    }
}