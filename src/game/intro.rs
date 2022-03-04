use super::*;

pub enum IntroPhase {
    Empty, // 
    PlayAGame, // Let's play a game.
    Checkers, // You know checkers, right?
    OrDraughts, // Or draughts. Whatever you prefer.
    Empty2, // 
    ButBoring, // But that'd be boring, wouldn't it?
    Twist, // Let's add a twist.
    TellMe, // You must tell me your next move before I make mine.
    IllTake, // I'll take black, so I go first.
}

impl Default for IntroPhase {
    fn default() -> Self {
        IntroPhase::Empty
    }
}

pub struct BeginGameEvent;

pub fn update_intro_text(
    turn: ResMut<Turn>,
    time: Res<Time>,
    mut text: ResMut<ScreenText>,
    mut phase: Local<IntroPhase>,
    input: Res<Input<KeyCode>>,
    mut events: EventWriter<BeginGameEvent>,
) {
    if *turn != Turn::PreGame {
        return;
    }

    if input.just_pressed(KeyCode::Escape) {
        events.send(BeginGameEvent);
        return;
    }

    let timer = time.seconds_since_startup();
    match *phase {
        IntroPhase::Empty => {
            if timer > 2.0 {
                text.0.push_str("Let's play a game.");
                *phase = IntroPhase::PlayAGame;
            }
        },
        IntroPhase::PlayAGame => {
            if timer > 4.0 {
                text.0.push_str("\nYou know checkers, right?");
                *phase = IntroPhase::Checkers;
            }
        },
        IntroPhase::Checkers => {
            if timer > 6.0 {
                text.0.push_str("\nOr draughts. Whatever you prefer.");
                *phase = IntroPhase::OrDraughts;
            }
        },
        IntroPhase::OrDraughts => {
            if timer > 8.0 {
                text.0 = "".to_string();
                *phase = IntroPhase::Empty2;
            }
        },
        IntroPhase::Empty2 => {
            if timer > 10.0 {
                text.0.push_str("But that'd be boring, wouldn't it?");
                *phase = IntroPhase::ButBoring;
            }
        },
        IntroPhase::ButBoring => {
            if timer > 12.0 {
                text.0.push_str("\nLet's add a twist.");
                *phase = IntroPhase::Twist;
            }
        },
        IntroPhase::Twist => {
            if timer > 14.0 {
                text.0.push_str("\nYou must tell me your next move before I make mine.");
                *phase = IntroPhase::TellMe;
            }
        },
        IntroPhase::TellMe => {
            if timer > 16.0 {
                text.0.push_str("\nI'll take black, so I go first.");
                *phase = IntroPhase::IllTake;
            }
        },
        IntroPhase::IllTake => {
            if timer > 18.0 {
                if text.1 > 0.0 {
                    text.1 = (text.1 - (0.33 * time.delta_seconds())).clamp(0.0, 1.0);
                } else {
                    events.send(BeginGameEvent);
                }
            }
        },
    };    
}

pub fn begin(
    mut events: EventReader<BeginGameEvent>,
    mut turn: ResMut<Turn>,
    mut commands: Commands,
    checkers: Query<(Entity, &Checker)>,
    table: Res<Table>,
) {
    if events.iter().count() == 0 {
        return;
    }

    commands.entity(table.0.unwrap())
    .insert(LerpToTarget { ratio: 5.0, target: Vec3::ZERO});

    for (id, checker) in checkers.iter() {
        let target = get_checkboard_pos(checker.pos);

        commands.entity(id)
        .with_delay(match checker.team { Team::Red => 2.0, Team::Black => 1.0 }, move |c| {
            c
            .insert(QuadJump::new(vec3(0.0, 1.0, 0.0), target, 10.0, 0.5));
        });
    }

    *turn = Turn::BlackForesight;
}