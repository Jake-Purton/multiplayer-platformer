mod platform;
mod player;
mod death;
mod startup_plugin;
mod next_level;
mod win;
mod main_menu;
mod moving_block;
mod grappling_hook;
mod client;
mod server;
mod messages;
mod join_menu;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use bevy_kira_audio::prelude::*;
use client::MyClientPlugin;
use death::DeathPlugin;
use grappling_hook::GrapplePlugin;
use local_ip_address::local_ip;
use main_menu::{MenuPlugin, HostClient};
use moving_block::MovingBlockPlugin;
use next_level::NextLevelPlugin;
use platform::PlatformPlugin;
use player::PlayerPlugin;
use server::MyServerPlugin;
use startup_plugin::StartupPlugin;
use win::WinPlugin;

use crate::join_menu::JoinMenuPlugin;

const SPRITE_SCALE: f32 = 0.707106;
const HOOK_SPRITE_SIZE: Vec2 = Vec2::new(24.0, 24.0);
const HOOK_SPEED: f32 = 2000.0;
const GRAPPLE_SPEED: f32 = 200.0;
const FELLA_SPRITE_SIZE: Vec2 = Vec2::new(64.0 * SPRITE_SCALE, 64.0 * SPRITE_SCALE);
const GRAVITY_CONSTANT: Vec2 = Vec2::new(0.0, -1200.0);
const PLAYER_JUMP_VELOCITY: f32 = 800.0;
const PLAYER_RUN_SPEED: f32 = 300.0;
const MAP_SCALE: f32 = 80.0;

pub fn level_directory(level_number: u8) -> String {
    format!("assets/levels/level-{}.txt", level_number)
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    Menu,
    Gameplay,
    Death,
    NextLevel,
    Win,
    JoinMenu,
}

#[derive(Resource)]
pub struct MultiplayerSetting(HostClient);

fn main() {

    println!("{}", local_ip().unwrap());

    App::new()
        .add_state::<GameState>()
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(DefaultPlugins)
        .insert_resource(CurrentLevel {level_number: 1})
        .add_plugin(GrapplePlugin)
        .add_plugin(PlayerPlugin)
        .add_plugin(PlatformPlugin)
        .add_plugin(AudioPlugin)
        .add_plugin(DeathPlugin)
        .add_plugin(StartupPlugin)
        .add_plugin(NextLevelPlugin)
        .add_plugin(WinPlugin)
        .add_plugin(MenuPlugin)
        .add_plugin(MovingBlockPlugin)
        .add_plugin(MyClientPlugin)
        .add_plugin(MyServerPlugin)
        .add_plugin(JoinMenuPlugin)
        .insert_resource(MultiplayerSetting(HostClient::Play))
        // .add_plugin(WorldInspectorPlugin::default())
        // .add_plugin(RapierDebugRenderPlugin::default())
        .run();
}

#[derive(Resource)]
pub struct CurrentLevel {
    level_number: u8,
}

