#![allow(clippy::type_complexity)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod client;
mod death;
mod grappling_hook;
mod join_menu;
mod main_menu;
mod messages;
mod moving_block;
mod next_level;
mod pinging;
mod platform;
mod player;
mod server;
mod startup_plugin;
mod win;
mod run_if;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use client::MyClientPlugin;
use death::DeathPlugin;
use grappling_hook::GrapplePlugin;
use main_menu::{HostClient, MenuPlugin};
use moving_block::MovingBlockPlugin;
use next_level::NextLevelPlugin;
use platform::PlatformPlugin;
use player::PlayerPlugin;
use server::MyServerPlugin;
use startup_plugin::StartupPlugin;
use std::f32::consts::FRAC_1_SQRT_2;
use win::WinPlugin;

use crate::{join_menu::JoinMenuPlugin, pinging::PingPlugin};

// constants
const SPRITE_SCALE: f32 = FRAC_1_SQRT_2;
const HOOK_SPRITE_SIZE: Vec2 = Vec2::new(24.0, 24.0);
const HOOK_SPEED: f32 = 2000.0;
const GRAPPLE_SPEED: f32 = 200.0;
const FELLA_SPRITE_SIZE: Vec2 = Vec2::new(64.0 * SPRITE_SCALE, 64.0 * SPRITE_SCALE);
const GRAVITY_CONSTANT: Vec2 = Vec2::new(0.0, -1200.0);
const PLAYER_JUMP_VELOCITY: f32 = 800.0;
const PLAYER_RUN_SPEED: f32 = 300.0;
const MAP_SCALE: f32 = 80.0;
const BACKGROUND_COLOUR: Color = Color::rgb(1.0, 0.5, 0.0);

// label the states
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    Menu,
    Gameplay,
    Death,
    NextLevel,
    Win,
    JoinMenu,
    CheckingConnection,
}

#[derive(Resource)]
pub struct MultiplayerSetting(HostClient);

fn main() {
    App::new()
        // add the states
        .add_state::<GameState>()

        // add physics plugin and bevy default plugins
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(DefaultPlugins)

        // insert the default settings
        .insert_resource(MultiplayerSetting(HostClient::Play))
        .insert_resource(CurrentLevel { level_number: 1 })
        
        // add all my custom plugins
        .add_plugin(GrapplePlugin)
        .add_plugin(PlayerPlugin)
        .add_plugin(PlatformPlugin)
        .add_plugin(DeathPlugin)
        .add_plugin(StartupPlugin)
        .add_plugin(NextLevelPlugin)
        .add_plugin(WinPlugin)
        .add_plugin(MenuPlugin)
        .add_plugin(MovingBlockPlugin)
        .add_plugin(MyClientPlugin)
        .add_plugin(MyServerPlugin)
        .add_plugin(JoinMenuPlugin)
        .add_plugin(PingPlugin)
        

        // run the app
        .run();
}

// the resource that tells us what level we are on
#[derive(Resource)]
pub struct CurrentLevel {
    level_number: u8,
}
