#![allow(clippy::type_complexity)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod client;
mod death;
mod grappling_hook;
mod join_menu;
mod main_menu;
mod messages;
mod next_level;
mod pinging;
mod platform;
mod player;
mod server;
mod startup_plugin;
mod win;

use std::f32::consts::FRAC_1_SQRT_2;

use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use bevy_rapier2d::prelude::*;
use client::MyClientPlugin;
use death::DeathPlugin;
use main_menu::{HostClient, MenuPlugin};
use next_level::NextLevelPlugin;
use platform::PlatformPlugin;
use player::PlayerPlugin;
use server::MyServerPlugin;
use startup_plugin::StartupPlugin;
use win::WinPlugin;

use crate::{join_menu::JoinMenuPlugin, pinging::PingPlugin};

const SPRITE_SCALE: f32 = FRAC_1_SQRT_2;
const HOOK_SPRITE_SIZE: Vec2 = Vec2::new(24.0, 24.0);
const HOOK_SPEED: f32 = 2000.0;
const GRAPPLE_SPEED: f32 = 200.0;
const FELLA_SPRITE_SIZE: Vec2 = Vec2::new(64.0 * SPRITE_SCALE, 64.0 * SPRITE_SCALE);
const GRAVITY_CONSTANT: Vec2 = Vec2::new(0.0, -1200.0);
const PLAYER_JUMP_VELOCITY: f32 = 800.0;
const PLAYER_RUN_SPEED: f32 = 300.0;
const MAP_SCALE: f32 = 80.0;

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
        .add_state::<GameState>()
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(DefaultPlugins)
        .insert_resource(CurrentLevel { level_number: 1 })
        .add_plugin(PlayerPlugin)
        .add_plugin(PlatformPlugin)
        .add_plugin(AudioPlugin)
        .add_plugin(DeathPlugin)
        .add_plugin(StartupPlugin)
        .add_plugin(NextLevelPlugin)
        .add_plugin(WinPlugin)
        .add_plugin(MenuPlugin)
        .add_plugin(MyClientPlugin)
        .add_plugin(MyServerPlugin)
        .add_plugin(JoinMenuPlugin)
        .add_plugin(PingPlugin)
        .insert_resource(MultiplayerSetting(HostClient::Play))
        // .add_plugin(WorldInspectorPlugin::default())
        // .add_plugin(RapierDebugRenderPlugin::default())
        .run();
}

#[derive(Resource)]
pub struct CurrentLevel {
    level_number: u8,
}
