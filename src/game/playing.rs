use super::*;

pub const LEAN_BACK_POS: [f32; 3] = [0.0, 0.25, -1.3];
pub const LEAN_BACK_ANGLE: [f32; 3] = [0.0, PI, 0.0];
pub const LEAN_FORWARD_POS: [f32; 3] = [0.0, 1.3, 0.0];
pub const LEAN_FORWARD_ANGLE: [f32; 3] = [0.0, PI / std::f32::consts::SQRT_2, PI / std::f32::consts::SQRT_2];

pub fn movement(mut q: Query<(&mut LerpToTarget, &mut SlerpToTarget), With<Camera>>, input: Res<Input<KeyCode>>, turn: Res<Turn>) {
    let lerp_ratio = 5.0;
    let slerp_ratio = 5.0;

    match *turn {
        Turn::Black | Turn::Red | Turn::BlackForesight | Turn::RedCorrection => (),
        Turn::PreGame | Turn::PostGame => return,
    };

    if input.just_pressed(KeyCode::W) {
        if let Ok((mut lerp, mut slerp)) = q.get_single_mut() {
            lerp.target = LEAN_FORWARD_POS.into();
            lerp.ratio = lerp_ratio;

            slerp.target = Quat::from_scaled_axis(LEAN_FORWARD_ANGLE.into());
            slerp.ratio = slerp_ratio;
        }
    }

    if input.just_pressed(KeyCode::S) {
        if let Ok((mut lerp, mut slerp)) = q.get_single_mut() {
            lerp.target = LEAN_BACK_POS.into();
            lerp.ratio = lerp_ratio;

            slerp.target = Quat::from_scaled_axis(LEAN_BACK_ANGLE.into());
            slerp.ratio = slerp_ratio;
        }
    }
}

pub struct SelectedChecker(pub Option<Entity>);
pub struct SelectedCheckerEvent(pub Entity);
pub struct RedMove {
    pub start: IVec2,
    pub jumped: Vec<IVec2>,
    pub jumps: Vec<IVec2>,
    pub performed: bool,
}

pub fn selecting(
    mut commands: Commands,
    mut events: EventReader<PickingEvent>, 
    checker_meshes: Query<&CheckerMesh>, 
    mut checkers: Query<(Entity, &mut Checker)>,
    places: Query<&CheckerPlace>, 
    mut selected: ResMut<SelectedChecker>,
    mut selected_events: EventWriter<SelectedCheckerEvent>,
    mut map: ResMut<CheckerMap>,
    mut turn: ResMut<Turn>,
    mut red_move: ResMut<RedMove>,
    mut header: ResMut<Header>,
    mut red_stack: ResMut<RedChipStack>,
    mut black_stack: ResMut<BlackChipStack>,
    input: Res<Input<KeyCode>>,
    mut end_game: EventWriter<EndGameEvent>,
) {
    if let Some(c) = selected.0 {
        if let Ok((_, checker)) = checkers.get(c) {
            if !checker.alive {
                selected.0 = None;
            }
        }
    }

    if input.just_pressed(KeyCode::F3) {
        end_game.send(EndGameEvent(Team::Red));
        *turn = Turn::PostGame;
        return;
    }

    for event in events.iter() {
        match event {
            PickingEvent::Selection(e) => {
                match e {
                    SelectionEvent::JustSelected(entity) => {
                        // Selecting a checker
                        if let Ok(mesh) = checker_meshes.get(*entity) {
                            selected.0 = Some(mesh.0);
                            selected_events.send(SelectedCheckerEvent(mesh.0));
                        }
                        match *turn {                            
                            Turn::BlackForesight => {
                                // Moving a checker
                                if let Ok(place) = places.get(*entity) {
                                    if !place.valid {
                                        println!("Value at place: {:?}", map.get_place(place.pos));
                                        header.0 = Cow::Borrowed("You cannot move that checker to that position");
                                        return;
                                    }

                                    if let Some(checker_id) = selected.0 {
                                        if let Ok((_, checker)) = checkers.get_mut(checker_id) {
                                            println!("Moving!");

                                            *red_move = RedMove {
                                                start: checker.pos, 
                                                jumped: place.jumped.clone(),
                                                jumps: place.jumps.clone(),
                                                performed: false,
                                            };
            
                                            selected.0 = None;
                                            *turn = Turn::Black;
                                        }
                                    }
                                }
                            },
                            Turn::RedCorrection => {
                                if let Ok(place) = places.get(*entity) {
                                    if !place.valid {
                                        header.0 = Cow::Borrowed("You cannot move that checker to that position");
                                        return;
                                    }

                                    if let Some(checker_id) = selected.0 {
                                        if let Ok((_, checker)) = checkers.get_mut(checker_id) {
                                            move_piece(&mut commands, &mut map, checker_id, checker, place.jumps.clone());
            
                                            for pos in place.jumped.iter() {
                                                let pos = *pos;

                                                let target = map.get_place_checker(pos)
                                                .expect(&format!("Failed to get checker at pos {:?}", pos));
                                                let (_, checker) = checkers.get_mut(target).unwrap();

                                                println!("Kill piece 3!");
                                                kill_piece(&mut commands, &mut map, checker, &mut red_stack, &mut black_stack);
                                            }
            
                                            selected.0 = None;
                                            *turn = Turn::BlackForesight;
                                        }
                                    }
                                }
                            },
                            _ => ()
                        }
                    },
                    SelectionEvent::JustDeselected(entity) => {
                        // Deselecting a checker
                        if selected.0 == Some(*entity) {
                            selected.0 = None;
                        }
                    },
                }
            },
            _ => (),
        }
    }
}

pub fn check_player_loss(
    checkers: Query<&Checker>,
    map: Res<CheckerMap>,
    mut turn: ResMut<Turn>,
    mut end_game: EventWriter<EndGameEvent>,
) {
    match *turn {
        Turn::BlackForesight | Turn::RedCorrection => (),
        _ => return,
    }

    let mut lost = true;
    for checker in checkers.iter() {
        if checker.team == Team::Red && checker.alive {
            let pos = checker.pos;

            let jumps = map.get_jumps(pos, Team::Red, checker.king);

            if jumps.len() != 0 {
                lost = false; 
                break;
            }

            let moves = map.get_available_moves(pos, Team::Red, checker.king);

            if moves.len() != 0 {
                lost = false;
                break;
            }
        }
    }

    if lost {
        println!("Player lost the game");
        end_game.send(EndGameEvent(Team::Black));
        *turn = Turn::PostGame;
    }
}

pub fn enable_valid_spaces(
    mut events: EventReader<SelectedCheckerEvent>,
    mut places: Query<&mut CheckerPlace>,
    checkers: Query<(Entity, &Checker)>,
    check_map: Res<CheckerMap>,
    place_map: Res<PlaceMap>,
    mut header: ResMut<Header>,
) {
    let mut checkers_with_jumps = vec![];
    for (id, checker) in checkers.iter().filter(|(_, c)| c.team == Team::Red && c.alive) {
        let jump_count = check_map.get_jumps(checker.pos, Team::Red, checker.king).len();
        if jump_count > 0 {
            checkers_with_jumps.push(id);
        }
    }

    for event in events.iter() {
        for mut place in places.iter_mut() {
            place.valid = false;
            place.jumps = vec![];
        }

        let (id, checker) = checkers.get(event.0).unwrap();

        if checkers_with_jumps.len() > 0 && !checkers_with_jumps.contains(&id) {
            header.0 = Cow::Borrowed("You must take any available jumps");
            continue;
        }

        let pos = checker.pos;

        let jumps = check_map.get_jumps(pos, Team::Red, checker.king);

        if jumps.len() == 0 {
            let moves = check_map.get_available_moves(pos, Team::Red, checker.king);

            for mov in moves {
                let mut place = places.get_mut(place_map.get(mov)).unwrap();
                place.valid = true;
                place.jumped = vec![];
                place.jumps = vec![mov];
            }
        } else {
            for (jumped, jumps) in jumps {
                let mut place = places.get_mut(place_map.get(*jumps.last().unwrap())).unwrap();
                place.valid = true;
                place.jumped = jumped;
                place.jumps = jumps;
            }
        }
    }
}

pub fn enemy_play(
    mut commands: Commands,
    mut turn: ResMut<Turn>,
    mut checkers: Query<(Entity, &mut Checker)>,
    mut check_map: ResMut<CheckerMap>,
    mut events: EventWriter<EndGameEvent>,
    mut red_stack: ResMut<RedChipStack>,
    mut black_stack: ResMut<BlackChipStack>,
    mut moved: ResMut<BlackHasMoved>,
) {
    if *turn != Turn::Black || moved.0 {
        return;
    }

    // Enemy thought process:
    // 1. Take the longest possible jump
    // 2. If possible, approach the checker the player is moving next turn
    // 3. If possible, king a checker
    // 4. Move a random checker forward, prioritizing kings

    let mut best_jump: Option<(_, Vec<IVec2>, _)> = None;

    for (entity, checker) in checkers.iter_mut() {
        if checker.team == Team::Black && checker.alive == true {
            let jumps = check_map.get_jumps(checker.pos, Team::Black, checker.king);
            for (hopped, hops) in jumps {
                if best_jump.is_none() || hopped.len() > best_jump.as_ref().unwrap().1.len() {
                    best_jump = Some((entity, hopped, hops));
                }
            }
        }
    }

    if let Some((checker_id, jumped, jumps)) = best_jump {
        let (_, checker) = checkers.get_mut(checker_id).unwrap();
        
        move_piece(&mut commands, &mut check_map, checker_id, checker, jumps);

        for pos in jumped {
            let hopped_checker = check_map.get_place_checker(pos).unwrap();
            let (_, checker) = checkers.get_mut(hopped_checker).unwrap();
            
            kill_piece(&mut commands, &mut check_map, checker, &mut red_stack, &mut black_stack);
        }

        moved.0 = true;
        return;
    }
    else {
        // TODO: Actually pick a random checker
        for (checker_id, checker) in checkers.iter_mut() {
            if checker.team == Team::Black && checker.alive {
                let moves = check_map.get_available_moves(checker.pos, Team::Black, checker.king);
                if moves.len() > 0 {
                    let mov = moves[rand::thread_rng().gen_range(0..moves.len())];

                    move_piece(&mut commands, &mut check_map, checker_id, checker, vec![mov]);

                    moved.0 = true;
                    return;
                }
            }
        }
    }

    // If we get here, Black loses because they cannot move

    println!("Black lost the game");
    events.send(EndGameEvent(Team::Red));
    *turn = Turn::PostGame;
}

pub struct BlackHasMoved(pub bool);

pub fn animate_black_turn(
    jumpers: Query<Entity, Or<(With<QuadJump>, With<MultiQuadJump>)>>,
    mut moved: ResMut<BlackHasMoved>,
    mut turn: ResMut<Turn>,
) {
    if *turn != Turn::Black || !moved.0 {
        return;
    }

    let mut animating = false;
    for _ in jumpers.iter() {
        animating = true;
    }

    if !animating {
        *turn = Turn::Red;
        moved.0 = false;
    }
}

pub fn animate_red_turn(
    commands: Commands,
    mut turn: ResMut<Turn>,
    mut red_move: ResMut<RedMove>,
    map: ResMut<CheckerMap>,
    mut checkers: Query<(Entity, &mut Checker)>,
    jumpers: Query<Entity, Or<(With<QuadJump>, With<MultiQuadJump>)>>,
    red_stack: ResMut<RedChipStack>,
    black_stack: ResMut<BlackChipStack>,
) {
    if *turn == Turn::Red {
        if !red_move.performed {
            let dptoc = do_player_turn_or_correction(commands, &red_move, map, &mut checkers, red_stack, black_stack);
            if let None = dptoc {
                *turn = Turn::RedCorrection;
                println!("Turn invalidated");
                return;
            }

            red_move.performed = true;
            return;
        }

        let mut animating = false;
        for _ in jumpers.iter() {
            animating = true;
        }

        if !animating {
            println!("Finished animating");
            *turn = Turn::BlackForesight;
        }
    }
}

fn do_player_turn_or_correction(
    mut commands: Commands,
    red_move: &ResMut<RedMove>,
    mut map: ResMut<CheckerMap>,
    checkers: &mut Query<(Entity, &mut Checker)>,
    mut red_stack: ResMut<RedChipStack>,
    mut black_stack: ResMut<BlackChipStack>,
) -> Option<()> {
    let checker_id = map.get_place_checker(red_move.start)?;

    let mut kills = vec![];

    // TODO: Prevent jumps that bounce on spaces that have since been filled

    // Verify all kills
    for pos in red_move.jumped.iter() {
        let pos = *pos;

        let target = map.get_place_checker(pos)?;
        let (target_id, _) = checkers.get_mut(target).ok()?;
        kills.push(target_id);
    }

    // Verify move and perform move + kills
    if map.is_place_free(*red_move.jumps.last().unwrap()) {
        let (checker_id, checker) = checkers.get_mut(checker_id).ok()?;

        move_piece(&mut commands, &mut map, checker_id, checker, red_move.jumps.clone());

        for kill in kills {
            let (_, target) = checkers.get_mut(kill).unwrap();

            kill_piece(&mut commands, &mut map, target, &mut red_stack, &mut black_stack);
        }
    } else {
        return None;
    };

    Some(())
}

fn move_piece(
    commands: &mut Commands,
    map: &mut CheckerMap,
    checker_id: Entity,
    mut checker: Mut<Checker>,
    mut jumps: Vec<IVec2>,
) {
    let dest = *jumps.last().unwrap();

    map.set_place(checker.pos, None);
    map.set_place(dest, Some((checker_id, checker.team)));
    jumps.insert(0, checker.pos);
    commands.entity(checker_id).insert(MultiQuadJump::new(
        jumps.windows(2)
        .map(|jumps| QuadJump::new(
            get_checkboard_pos(jumps[0]),
            get_checkboard_pos(jumps[1]),
            30.0,
            0.25 
        ))
        .collect()
    ));
    checker.pos = dest;

    if (checker.team == Team::Black && dest.y == 0) || (checker.team == Team::Red && dest.y == 7) {
        commands.entity(checker_id).insert(Kingify(true));
    }
}

fn kill_piece(
    commands: &mut Commands,
    map: &mut CheckerMap, 
    mut checker: Mut<Checker>,
    red_stack: &mut ResMut<RedChipStack>,
    black_stack: &mut ResMut<BlackChipStack>,
) {
    commands.entity(map.get_place_checker(checker.pos).unwrap())
    .insert(QuadJump::new(
        get_checkboard_pos(checker.pos),
        get_stack_pos(match checker.team {
            Team::Red => red_stack.height,
            Team::Black => black_stack.height,
        }, checker.team),
        5.0,
        0.5,
    ));
    
    checker.alive = false;
    map.set_place(checker.pos, None);
    checker.pos = ivec2(-1, -1);

    match checker.team {
        Team::Red => red_stack.height += 1,
        Team::Black => black_stack.height += 1,
    };
}

#[derive(Component)]
pub struct Kingify(pub bool);

pub fn make_kings(
    mut commands: Commands,
    mut checkers: Query<(Entity, &mut Checker, &Kingify)>,
    models: Query<&CheckerMeshReference, Without<Checker>>, 
    mut meshes: Query<&mut Visibility, Without<Checker>>, 
) {
    for (id, mut checker, kingify) in checkers.iter_mut() {
        checker.king = kingify.0;
        match kingify.0 {
            true => {
                meshes.get_mut(models.get(checker.model).unwrap().0).unwrap().is_visible = false;
                meshes.get_mut(models.get(checker.king_model).unwrap().0).unwrap().is_visible = true;
            },
            false => {
                meshes.get_mut(models.get(checker.model).unwrap().0).unwrap().is_visible = true;
                meshes.get_mut(models.get(checker.king_model).unwrap().0).unwrap().is_visible = false;
            }
        }
        commands.entity(id).remove::<Kingify>();
    }
}

pub fn play_checker_noises(
    audio: Res<Audio>,
    click: Res<ClickNoise>,
    mut events: EventReader<QuadLandEvent>,
    checkers: Query<&Checker>
) {
    for landing in events.iter() {
        if let Ok(_) = checkers.get(landing.0) {
            audio.play(click.0.as_ref().unwrap().clone());
        }
    }
}