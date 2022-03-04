use bevy::{
    math::*, 
    prelude::*,
};
use bevy_mod_picking::*;
use std::{f32::consts::PI, borrow::Cow};
use rand::prelude::*;

use crate::*;

pub mod intro;
pub mod outro;
pub mod playing;
pub mod setup;

pub struct ScreenText(pub String, pub f32, pub [f32; 3]);

pub fn screen_text_display(
    turn: Res<Turn>,
    mut ctx: ResMut<EguiContext>,
    text: Res<ScreenText>,
) {
    match *turn {
        Turn::PreGame | Turn::PostGame => (),
        _ => return,
    };

    let ctx = ctx.ctx_mut();

    egui::Area::new("SpeechText")
    .anchor(egui::Align2::LEFT_TOP, (0., 0.))
    .movable(false)
    .interactable(false)
    .show(ctx, |ui| {
        ui.centered_and_justified(|ui| {
            let color = egui::Rgba::from_rgba_unmultiplied(text.2[0], text.2[1], text.2[2], text.1); // default color with alpha;
            let text = egui::RichText::new(&text.0).color(color);
            ui.heading(text);
        })
    });
}

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
    jumps: Vec<IVec2>,
    jumped: Vec<IVec2>,
    valid: bool
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

    pub fn get_jumps(&self, pos: IVec2, checker_team: Team, king: bool) -> Vec<(Vec<IVec2>, Vec<IVec2>)> {
        use std::collections::VecDeque;
        use std::collections::HashSet;

        let opposing_team = match checker_team { Team::Red => Team::Black, Team::Black => Team::Red };

        let offsets = get_piece_offsets(checker_team, king);

        let mut jumps = vec![];

        let mut open_set = VecDeque::new();
        open_set.push_back((vec![], vec![], pos));

        let mut computed_spaces = HashSet::new();

        while !open_set.is_empty() {
            let (hopped, hops, pos) = open_set.pop_front().unwrap();
            let mut extended = false;

            computed_spaces.insert(pos);

            for offset in offsets.iter() {
                let offset = *offset;

                if let Some(t) = self.get_place_team(pos + offset) {
                    if computed_spaces.contains(&(pos + (2 * offset))) {
                        continue;
                    }
                    if t == opposing_team && self.is_place_free(pos + (2 * offset)) {
                        let mut hops = hops.clone();
                        let mut hopped_checkers = hopped.clone();
                        hops.push(pos + (2 * offset));
                        hopped_checkers.push(pos + offset);
                        open_set.push_back((hopped_checkers, hops, pos + (2 * offset)));
                        extended = true;
                    }
                }
            }

            if !extended {
                jumps.push((hopped, hops, pos));
            }
        }

        jumps.into_iter()
        .filter(|(_, _, p)| *p != pos)
        .map(|(jumped, jumps, _)| {
            (jumped, jumps)
        })
        .collect()
    }

    pub fn get_available_moves(&self, pos: IVec2, team: Team, king: bool) -> Vec<IVec2> {
        get_piece_offsets(team, king).into_iter()
        .map(|o| pos + o)
        .filter(|m| self.is_place_free(*m))
        .collect()
    }

    pub fn reset(
        &mut self, 
        commands: &mut Commands, 
        checkers: &mut Query<(Entity, &mut Checker, &Transform)>,
        mut red_stack: ResMut<RedChipStack>,
        mut black_stack: ResMut<BlackChipStack>,
    ) {
        self.0.clear();
        red_stack.height = 0;
        black_stack.height = 0;

        let mut b = 0;
        let mut r = 0;

        for (id, mut checker, tf) in checkers.iter_mut() {
            match checker.team {
                Team::Red => {
                    let initial_coord = get_initial_checkboard_coord(r, Team::Red);
                    let initial_pos = get_checkboard_pos(initial_coord);

                    checker.pos = initial_coord;
                    checker.king = false;
                    checker.alive = true;

                    self.set_place(initial_coord, Some((id, Team::Red)));

                    commands.entity(id)
                    .insert(Kingify(false))
                    .insert(QuadJump::new(tf.translation, initial_pos, 50.0, 0.5));

                    r += 1;
                },
                Team::Black => {
                    let initial_coord = get_initial_checkboard_coord(b, Team::Black);
                    let initial_pos = get_checkboard_pos(initial_coord);

                    checker.pos = initial_coord;
                    checker.king = false;
                    checker.alive = true;

                    self.set_place(initial_coord, Some((id, Team::Black)));

                    commands.entity(id)
                    .insert(Kingify(false))
                    .insert(QuadJump::new(tf.translation, initial_pos, 50.0, 0.5));

                    b += 1;
                }
            }
        }
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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Turn {
    PreGame,
    Black,
    BlackForesight,
    Red,
    RedCorrection,
    PostGame
}

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

pub struct LastTurn(Turn);
impl Default for LastTurn {
    fn default() -> Self {
        Self(Turn::PreGame)
    }
}

pub fn update_header_with_turn(
    turn: Res<Turn>,
    mut last_turn: Local<LastTurn>,
    mut header: ResMut<Header>
) {
    if *turn != last_turn.0 {
        header.0 = Cow::Borrowed(match *turn {
            Turn::PreGame => "",
            Turn::Black => "Black's Turn",
            Turn::BlackForesight => "Choose your next move",
            Turn::Red => "Your Turn",
            Turn::RedCorrection => "Your chosen move was invalidated by Black, choose another",
            Turn::PostGame => "",
        });

        last_turn.0 = *turn;
    }
}

pub struct Header(pub Cow<'static, str>);

pub fn text_header(
    mut ctx: ResMut<EguiContext>,
    header: Res<Header>,
) {
    let ctx = ctx.ctx_mut();

    egui::Area::new("Header")
    .anchor(egui::Align2::LEFT_TOP, (0., 0.))
    .movable(false)
    .order(egui::Order::Background)
    .show(ctx, |ui| {
        ui.vertical_centered_justified(|ui| {
            ui.heading(
                match &header.0 {
                    Cow::Borrowed(str) => *str,
                    Cow::Owned(str) => str,
                }
            );
        })
    });
}

#[derive(Default)]
pub struct RedChipStack {
    height: usize
}

#[derive(Default)]
pub struct BlackChipStack {
    height: usize
}

fn get_stack_pos(i: usize, team: Team) -> Vec3 {
    let base_height = 0.034037;
    let chip_height = 0.0096;
    
    let x = match team {
        Team::Red => 0.3,
        Team::Black => -0.3,
    };

    let z = 0.0;

    let y = base_height + (chip_height * i as f32);

    vec3(x, y, z)
}