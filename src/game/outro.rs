use super::*;
use std::time::Duration;

pub struct EndGameEvent(pub Team);

pub fn end(
    mut events: EventReader<EndGameEvent>,
    turn: Res<Turn>,
    // mut checkers: Query<(&mut Checker, &mut LerpToTarget)>,
    mut phase: ResMut<EndingPhase>,
    mut ending_moment: ResMut<EndingDurationSinceStartup>,
    mut camera: Query<(&mut LerpToTarget, &mut SlerpToTarget), (With<Camera>, Without<Checker>)>,
    mut text: ResMut<ScreenText>,
    time: Res<Time>,
) {
    if *turn != Turn::PostGame {
        return; 
    }

    let mut end = false;

    for e in events.iter() {
        *phase = match e.0 {
            Team::Black => EndingPhase::black(),
            Team::Red => EndingPhase::red(),
        };

        println!("Processed endgame event!");

        end = true;
    }

    if !end {
        return;
    }

    if let Ok((mut lerp, mut slerp)) = camera.get_single_mut() {
        lerp.target = LEAN_BACK_POS.into();
        lerp.ratio = 5.0;

        slerp.target = Quat::from_scaled_axis(LEAN_BACK_ANGLE.into());
        slerp.ratio = 5.0;
    }

    *text = ScreenText("".to_string(), 1.0, [0.5, 0.5, 0.5]);

    ending_moment.0 = time.time_since_startup();
}

pub struct EndingDurationSinceStartup(pub Duration);

pub enum EndingPhase {
    Red(RedEndingPhase),
    Black(BlackEndingPhase),
}

impl EndingPhase {
    pub fn red() -> Self {
        EndingPhase::Red(RedEndingPhase::Empty)
    }

    pub fn black() -> Self {
        EndingPhase::Black(BlackEndingPhase::Empty)
    }
}

pub enum RedEndingPhase {
    Empty, // Wait one second
    OhItAppears, // Oh. It appears you've beaten me.
    YouKnow, // R: You know,  
    ForSomeone, // R: for someone that can see the future,
    YouDont, // R: you don't plan ahead an awful lot.
    Ellipse, // ...
    Fall, // R: Fall off the table
}

pub enum BlackEndingPhase {
    Empty, // Wait one second
    Aha, // Aha. I've won.
    Again, // Let's play again. (immediately reset game state)
}

pub fn update_end_game_text(
    mut commands: Commands,
    mut turn: ResMut<Turn>,
    mut text: ResMut<ScreenText>,
    mut phase: ResMut<EndingPhase>,
    time: Res<Time>,
    ending_moment: Res<EndingDurationSinceStartup>,
    mut exit: EventWriter<bevy::app::AppExit>,
    mut checkers: Query<(Entity, &mut Checker, &Transform)>,
    mut map: ResMut<CheckerMap>,
    red_stack: ResMut<RedChipStack>,
    black_stack: ResMut<BlackChipStack>,
    mut camera: Query<(Entity, &Transform), (With<PlayerCamera>, Without<Checker>)>,
) {
    if *turn != Turn::PostGame {
        return;
    }

    let timer = (time.time_since_startup() - ending_moment.0).as_secs_f64();

    match &mut *phase {
        EndingPhase::Red(phase) => match phase {
            RedEndingPhase::Empty => {
                if timer > 3.0 {
                    text.0.push_str("Oh. It appears you've beaten me.");
                    *phase = RedEndingPhase::OhItAppears;
                }
            },
            RedEndingPhase::OhItAppears => {
                if timer > 6.0 {
                    text.0 = "You know,".to_string(); 
                    text.2 = [0.8, 0.0, 0.0];
                    *phase = RedEndingPhase::YouKnow;
                }
            },
            RedEndingPhase::YouKnow => {
                if timer > 7.5 {
                    text.0.push_str(" for someone that can see the future,");
                    *phase = RedEndingPhase::ForSomeone;
                }
            },
            RedEndingPhase::ForSomeone => {
                if timer > 9.0 {
                    text.0.push_str("\nyou don't plan ahead an awful lot.");
                    *phase = RedEndingPhase::YouDont;
                }
            },
            RedEndingPhase::YouDont => {
                if timer > 13.0 {
                    text.0 = "...".to_string();
                    text.2 = [0.5, 0.5, 0.5];
                    *phase = RedEndingPhase::Ellipse;
                }
            },
            RedEndingPhase::Ellipse => {
                if timer > 18.0 {
                    text.0 = "".to_string();
                    *phase = RedEndingPhase::Fall;

                    // TODO: Fall

                    let (cam, tf) = camera.single_mut();
                    commands.entity(cam)
                    .insert(QuadJump::new(
                        tf.translation,
                        vec3(0.0, -15.0, -20.0),
                        0.1,
                        5.0
                    ))
                    .insert(StareAt(vec3(0.0, 0.25, 0.0)));
                }
            },
            RedEndingPhase::Fall => {
                if timer > 23.0 {
                    exit.send(bevy::app::AppExit);
                }
            },
        },
        EndingPhase::Black(phase) => match phase {
            BlackEndingPhase::Empty => {
                if timer > 1.0 {
                    text.0.push_str("Aha. I've won.");
                    *phase = BlackEndingPhase::Aha;
                }
            },
            BlackEndingPhase::Aha => {
                if timer > 3.0 {
                    text.0.push_str("\nLet's play again.");
                    *phase = BlackEndingPhase::Again; 

                    map.reset(&mut commands, &mut checkers, red_stack, black_stack);
                }
            },
            BlackEndingPhase::Again => {
                if timer > 4.0 {
                    if text.1 > 0.0 {
                        text.1 = (text.1 - (2.0 * time.delta_seconds())).clamp(0.0, 1.0);
                    } else {
                        *turn = Turn::BlackForesight;
                    }
                }
            },
        },
    }
}