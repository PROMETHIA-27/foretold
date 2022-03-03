use super::*;

pub const LEAN_BACK_POS: [f32; 3] = [0.0, 0.25, -1.3];
pub const LEAN_BACK_ANGLE: [f32; 3] = [0.0, PI, 0.0];
pub const LEAN_FORWARD_POS: [f32; 3] = [0.0, 1.3, 0.0];
pub const LEAN_FORWARD_ANGLE: [f32; 3] = [0.0, PI / std::f32::consts::SQRT_2, PI / std::f32::consts::SQRT_2];

pub fn movement(mut q: Query<(&mut LerpToTarget, &mut SlerpToTarget), With<Camera>>, input: Res<Input<KeyCode>>) {
    let lerp_ratio = 5.0;
    let slerp_ratio = 5.0;

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

pub fn selecting(
    mut events: EventReader<PickingEvent>, 
    checker_meshes: Query<&CheckerMesh>, 
    mut checkers: Query<(&mut Checker, &mut LerpToTarget)>, 
    places: Query<&CheckerPlace>, 
    mut selected: ResMut<SelectedChecker>,
    mut selected_events: EventWriter<SelectedCheckerEvent>,
    mut map: ResMut<CheckerMap>,
    mut turn: ResMut<Turn>,
) {
    if let Turn(Team::Black) = *turn {
        // TODO: Complain at the player
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

                        // Moving a checker
                        if_chain::if_chain!(
                            if let Ok(place) = places.get(*entity);
                            if let Some(checker_id) = selected.0;
                            if let Ok((checker, lerp)) = checkers.get_mut(checker_id);
                            then {
                                move_piece(&mut map, checker_id, checker, lerp, place.pos);

                                for pos in place.jumps.iter() {
                                    let pos = *pos;

                                    let target = map.get_place_checker(pos).unwrap();
                                    let (checker, lerp) = checkers.get_mut(target).unwrap();

                                    kill_piece(&mut map, checker, lerp);
                                }

                                selected.0 = None;
                                turn.0 = Team::Black;
                            }
                        );
                    },
                    SelectionEvent::JustDeselected(entity) => {
                        // Deselecting a checker
                        if let Ok(_) = checkers.get(*entity) {
                            selected.0 = None;
                        }
                    },
                }
            },
            _ => (),
        }
    }
}

pub fn enable_valid_spaces(
    mut events: EventReader<SelectedCheckerEvent>,
    mut places: Query<(&mut Visibility, &mut CheckerPlace)>,
    checkers: Query<&Checker>,
    check_map: Res<CheckerMap>,
    place_map: Res<PlaceMap>
) {
    for event in events.iter() {
        for (mut visibility, mut place) in places.iter_mut() {
            visibility.is_visible = false;
            place.jumps = vec![];
        }

        if let Ok(checker) = checkers.get(event.0) {
            let pos = checker.pos;

            let jumps = check_map.get_jumps(pos, Team::Red, false);

            if jumps.len() == 0 {
                let offsets = get_piece_offsets(Team::Red, false);

                for offset in offsets {                    
                    let pos = pos + offset;

                    if pos.x < 0 || pos.x > 7 || pos.y < 0 || pos.y > 7 {
                        continue;
                    }

                    if check_map.is_place_free(pos) {
                        let (mut vis, _) = places.get_mut(place_map.get(pos)).unwrap();
                        vis.is_visible = true;
                    }
                }
            } else {
                for (jumps, pos) in jumps {
                    let (mut vis, mut place) = places.get_mut(place_map.get(pos)).unwrap();
                    vis.is_visible = true;
                    place.jumps = jumps;
                }
            }
        }
    }
}

pub fn enemy_play(
    mut turn: ResMut<Turn>,
    mut checkers: Query<(Entity, &mut Checker, &mut LerpToTarget)>,
    
    mut check_map: ResMut<CheckerMap>,
    // mut place_map: ResMut<PlaceMap>,
) {
    if let Turn(Team::Red) = *turn {
        return;
    }

    // Enemy thought process:
    // 1. Take the longest possible jump
    // 2. If possible, approach the checker the player is moving next turn
    // 3. If possible, king a checker
    // 4. Move a random checker forward, prioritizing kings

    let mut best_jump: Option<(_, Vec<IVec2>, _)> = None;

    for (entity, checker, _) in checkers.iter_mut() {
        if checker.team == Team::Black && checker.alive == true {
            let jumps = check_map.get_jumps(checker.pos, Team::Black, false);
            for (hopped, dest) in jumps {
                if best_jump.is_none() || hopped.len() > best_jump.as_ref().unwrap().1.len() {
                    best_jump = Some((entity, hopped, dest));
                }
            }
        }
    }

    if let Some((checker_id, hopped, dest)) = best_jump {
        let (_, checker, lerp) = checkers.get_mut(checker_id).unwrap();
        
        move_piece(&mut check_map, checker_id, checker, lerp, dest);

        for pos in hopped {
            let hopped_checker = check_map.get_place_checker(pos).unwrap();
            let (_, checker, lerp) = checkers.get_mut(hopped_checker).unwrap();
            
            kill_piece(&mut check_map, checker, lerp);
        }

        turn.0 = Team::Red;
        return;
    }
    else {
        for (checker_id, checker, lerp) in checkers.iter_mut() {
            if checker.team == Team::Black {
                let moves = check_map.get_available_moves(checker.pos, Team::Black, false);
                if moves.len() > 0 {
                    let mov = moves[rand::thread_rng().gen_range(0..moves.len())];

                    move_piece(&mut check_map, checker_id, checker, lerp, mov);

                    turn.0 = Team::Red;
                    return;
                }
            }
        }
    }

    // If we get here, Black loses because they cannot move

    turn.0 = Team::Red;
}

fn move_piece(map: &mut CheckerMap, checker_id: Entity, mut checker: Mut<Checker>, mut lerp: Mut<LerpToTarget>, dest: IVec2) {
    map.set_place(checker.pos, None);
    map.set_place(dest, Some((checker_id, checker.team)));
    checker.pos = dest;
    lerp.target = get_checkboard_pos(dest);
}

fn kill_piece(map: &mut CheckerMap, mut checker: Mut<Checker>, mut lerp: Mut<LerpToTarget>) {
    map.set_place(checker.pos, None);
    checker.pos = ivec2(-1, -1);
    lerp.ratio = 0.1;
    lerp.target = lerp.target.extend(0.0).xwz();
}

fn make_king(mut checker: Mut<Checker>) {

}