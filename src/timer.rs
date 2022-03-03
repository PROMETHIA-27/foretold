use bevy::{prelude::*, ecs::system::EntityCommands};

#[derive(Component)]
pub struct TimerAction {
    secs_left: f32,
    paused: bool,
    action: Option<Box<dyn FnOnce(Entity, &mut Commands) + Send + Sync>>
}

impl TimerAction {
    pub fn new<F: 'static + FnOnce(Entity, &mut Commands) + Send + Sync>(time: f32, action: F) -> TimerAction {
        TimerAction {
            secs_left: time,
            paused: false,
            action: Some(Box::new(action))
        }
    }
}

pub fn advance_timers(mut query: Query<(Entity, &mut TimerAction)>, time: Res<Time>, mut commands: Commands) {
    for (e, mut timer) in query.iter_mut() {
        if !timer.paused {
            timer.secs_left -= time.delta_seconds();
            
            if timer.secs_left <= 0.0 {
                if let Some(act) = timer.action.take() {
                    act(e, &mut commands);
                }
                
                commands.entity(e).remove::<TimerAction>();
            }
        }
    }
}

/// Helper trait to allow easily delaying actions
pub trait CommandWithDelay where Self: Sized {
    fn with_delay<F: 'static + FnOnce(&mut EntityCommands) + Sync + Send>(&mut self, delay: f32, actions: F) -> &mut Self;
}

impl<'w, 's, 'a> CommandWithDelay for EntityCommands<'w, 's, 'a> {
    fn with_delay<F: 'static + FnOnce(&mut EntityCommands) + Sync + Send>(&mut self, delay: f32, actions: F) -> &mut Self {
        self.insert(TimerAction::new(delay, Box::new(|e, commands: &mut Commands| {
            actions(&mut commands.entity(e));
        })))
    }
}