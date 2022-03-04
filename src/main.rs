use bevy::math::*;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext, EguiPlugin};
use bevy_kira_audio::{Audio, AudioPlugin, AudioSource};
use bevy_mod_picking::*;
use std::borrow::Cow;

mod flicker;
mod game;
mod lerp;
mod quadratic;
mod stare;
mod timer;

use flicker::*;
use game::*;
use game::intro::*;
use game::outro::*;
use game::playing::*;
use game::setup::*;
use lerp::*;
use quadratic::*;
use stare::*;
use timer::*;


/*
Notes:

A game of checkers where the opponent can read your next move and react to it preemptively. You can win because they will heavily prioritize direct responses to your incoming moves,
and not plan ahead at all. Because I don't think I have enough time to write good checkers AI

_________________________
\      ___________      /
 \    /           \    /
  \  /             \  /
   \                 /
    \               /
     \  |       |  /
      \ |       | /
       \         /
        \       /
         \     /
          \   /
           \ /
            

*/

fn main() {
    App::new()
    .add_plugins(DefaultPlugins)
    .add_plugin(EguiPlugin)
    // .add_plugin(EditorPlugin)
    .add_plugins(DefaultPickingPlugins)
    .add_plugin(AudioPlugin)
    .insert_resource(ClearColor(Color::BLACK))
    .insert_resource(SelectedChecker(None))
    .insert_resource(CheckerMap::default())
    .insert_resource(PlaceMap::default())
    .insert_resource(Turn::PreGame)
    .insert_resource(RedMove { start: ivec2(-1, -1), jumped: vec![], jumps: vec![], performed: true })
    .insert_resource(Header(Cow::Borrowed("")))
    .insert_resource(ScreenText("".to_string(), 1.0, [0.5, 0.5, 0.5]))
    .insert_resource(Table(None))
    .insert_resource(EndingPhase::red())
    .insert_resource(EndingDurationSinceStartup(std::time::Duration::ZERO))
    .insert_resource(RedChipStack::default())
    .insert_resource(BlackChipStack::default())
    .insert_resource(BlackHasMoved(false))
    .insert_resource(ClickNoise(None))
    .add_event::<SelectedCheckerEvent>()
    .add_event::<BeginGameEvent>()
    .add_event::<EndGameEvent>()
    .add_event::<QuadLandEvent>()
    .add_startup_system(set_up_camera)
    .add_startup_system(set_up_checkerboard)
    .add_startup_system(load_audio)
    .add_system_to_stage(CoreStage::PostUpdate, set_up_checker_pickables)
    .add_system(light_flicker)
    .add_system(lerp_to_targets)
    .add_system(slerp_to_targets)
    .add_system(advance_timers)
    .add_system(movement)
    .add_system(selecting)
    .add_system(enable_valid_spaces)
    .add_system(enemy_play)
    .add_system(animate_red_turn)
    .add_system(update_header_with_turn)
    .add_system(text_header)
    .add_system(screen_text_display)
    .add_system(update_intro_text)
    .add_system(begin)
    .add_system(end)
    .add_system(update_end_game_text)
    .add_system(check_player_loss)
    .add_system(make_kings)
    .add_system(move_quadratics)
    .add_system(animate_black_turn)
    .add_system(update_starers)
    .add_system(play_checker_noises)
    .run();

    // .add_startup_system(main_menu_setup)
    // .add_system(main_menu)
}

// enum EguiImage {
//     MainMenuBackground = 0,
// }

// fn main_menu_setup(mut images: ResMut<Assets<Image>>, mut ctx: ResMut<EguiContext>) {
//     let img = Image::from_buffer(
//         include_bytes!("../assets/images/TEMP-main_menu_background.jpeg"), 
//         bevy::render::texture::ImageType::Extension("jpeg")
//     ).unwrap();

//     let handle = images.add(img);

//     ctx.set_egui_texture(EguiImage::MainMenuBackground as u64, handle);

//     let mut font_def = egui::FontDefinitions::default();
//     font_def.family_and_size.insert(egui::TextStyle::Heading, (egui::FontFamily::Proportional, 162.));
//     ctx.ctx_mut().set_fonts(font_def);
// }

// fn main_menu(mut commands: Commands, mut ctx: ResMut<EguiContext>)
// {
//     egui::Area::new("MainMenuBg")
//     .anchor(egui::Align2::LEFT_TOP, egui::vec2(0., 0.))
//     .order(egui::Order::Background)
//     .show(ctx.ctx_mut(), |ui| {
//         ui.centered_and_justified(|ui| {
//             ui.image(egui::TextureId::User(EguiImage::MainMenuBackground as u64), ui.available_size())
//         })
//     });

//     egui::Area::new("MainMenu")
//     .anchor(egui::Align2::LEFT_TOP, egui::vec2(0., 0.))
//     .show(ctx.ctx_mut(), |ui| {
//         ui.with_layout(egui::Layout::right_to_left(), |ui| {
//             ui.allocate_ui(egui::vec2(ui.available_width(), ui.min_size().y), |ui| {
//                 ui.label(
//                     egui::RichText::new("FORETOLD")
//                     .color(egui::Color32::from_rgb(244, 244, 237))
//                     .background_color(egui::Color32::from_rgb(21, 22, 19))
//                     .heading()
//                 );
//             });

//             let _ = ui.button("Test!");
//         })
//     });
// }
