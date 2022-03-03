use bevy::{
    math::*, 
    prelude::*,
};
use bevy_mod_picking::*;
use std::f32::consts::PI;
use rand::prelude::*;

use crate::*;

pub mod setup;
pub mod playing;

#[derive(Component, Debug)]
pub struct Checker {
    team: Team,
    pos: IVec2,
    alive: bool,
    king: bool,
    model: Entity,
    king_model: Entity
}

#[derive(Component)]
pub struct CheckerPlace {
    pos: IVec2,
    jumps: Vec<IVec2>
}

#[derive(Default)]
pub struct CheckerMap(std::collections::HashMap<IVec2, (Entity, Team)>);

impl CheckerMap {
    pub fn is_place_free(&self, pos: IVec2) -> bool {
        pos.x >= 0 && pos.x <= 7 && pos.y >= 0 && pos.y <= 7 && !self.0.contains_key(&pos)
    }

    pub fn get_place_team(&self, pos: IVec2) -> Option<Team> {
        self.0.get(&pos).map(|(_, t)| *t)
    }

    pub fn get_place_checker(&self, pos: IVec2) -> Option<Entity> {
        self.0.get(&pos).map(|(c, _)| *c)  
    }

    pub fn get_place(&self, pos: IVec2) -> Option<(Entity, Team)> {
        self.0.get(&pos).map(|val| *val) 
    }

    pub fn set_place(&mut self, pos: IVec2, value: Option<(Entity, Team)>) {
        if pos.x < 0 || pos.x > 7 || pos.y < 0 || pos.y > 7 {
            eprintln!("Attempted to set place out of bounds of board");
            return;
        }

        match value {
            Some(val) => self.0.insert(pos, val),
            None => self.0.remove(&pos)
        };
    }

    pub fn get_jumps(&self, pos: IVec2, checker_team: Team, king: bool) -> Vec<(Vec<IVec2>, IVec2)> {
        use std::collections::VecDeque;
        use std::collections::HashSet;

        let opposing_team = match checker_team { Team::Red => Team::Black, Team::Black => Team::Red };

        let offsets = get_piece_offsets(checker_team, king);

        let mut jumps = vec![];

        let mut open_set = VecDeque::new();
        open_set.push_back((vec![], pos));

        let mut computed_spaces = HashSet::new();

        while !open_set.is_empty() {
            let (hopped, pos) = open_set.pop_front().unwrap();
            let mut extended = false;

            computed_spaces.insert(pos);

            for offset in offsets.iter() {
                let offset = *offset;

                if let Some(t) = self.get_place_team(pos + offset) {
                    if computed_spaces.contains(&(pos + (2 * offset))) {
                        continue;
                    }
                    if t == opposing_team && self.is_place_free(pos + (2 * offset)) {
                        let mut hopped_checkers = hopped.clone();
                        hopped_checkers.push(pos + offset);
                        open_set.push_back((hopped_checkers, pos + (2 * offset)));
                        extended = true;
                    }
                }
            }

            if !extended {
                jumps.push((hopped, pos));
            }
        }

        jumps = jumps.into_iter().filter(|(_, p)| *p != pos).collect();

        jumps
    }

    pub fn get_available_moves(&self, pos: IVec2, team: Team, king: bool) -> Vec<IVec2> {
        let offsets = get_piece_offsets(team, king);

        let mut moves = vec![];
        for offset in offsets {
            let pos = pos + offset;

            if self.is_place_free(pos) {
                moves.push(pos);
            }
        }

        moves
    }
}

#[derive(Default)]
pub struct PlaceMap(std::collections::HashMap<IVec2, Entity>);

impl PlaceMap {
    pub fn insert(&mut self, pos: IVec2, entity: Entity) {
        self.0.insert(pos, entity);
    }

    pub fn get(&self, pos: IVec2) -> Entity {
        *self.0.get(&pos).expect(&format!("Failed to get checker place at ({} {})", pos.x, pos.y))
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Team {
    Black,
    Red,
}

#[derive(Copy, Clone, Debug)]
pub struct Turn(pub Team);

fn index_checkboard(i: i32) -> IVec2 {
    ivec2(i % 8, i / 8)
}

fn get_initial_checkboard_coord(i: i32, team: Team) -> IVec2 {
    match team {
        Team::Black => {
            let row_offset = (i / 4) % 2;
            let x = 7 - ((i % 4) * 2) - row_offset;
            let y = 7 - (i / 4);

            ivec2(x, y)
        },
        Team::Red => {
            let row_offset = (i / 4) % 2;
            let x = ((i % 4) * 2) + row_offset;
            let y = i / 4;

            ivec2(x, y)
        }
    }
}

fn get_checkboard_pos(coord: IVec2) -> Vec3 {
    let tile_width = 0.061;
    let origin = vec3(0.214, 0.085, -0.214); // Player's left corner

    origin + vec3(-tile_width * coord.x as f32, 0.0, tile_width * coord.y as f32)
}

fn get_piece_offsets(team: Team, king: bool) -> Vec<IVec2> {
    match (team, king) {
        (Team::Black, false) => vec![
            ivec2(-1, -1),
            ivec2(1, -1),
        ],
        (Team::Red, false) => vec![
            ivec2(-1, 1),
            ivec2(1, 1),
        ],
        (_, true) => vec![
            ivec2(-1, 1),
            ivec2(1, 1),
            ivec2(-1, -1),
            ivec2(1, -1),
        ]
    }
}