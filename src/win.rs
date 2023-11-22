use bevy::prelude::*;

use crate::{
    startup_plugin::{despawn_everything, GameTextures},
    GameState, BACKGROUND_COLOUR,
};

pub struct WinPlugin;

impl Plugin for WinPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(setup_win_screen.in_schedule(OnEnter(GameState::Win)))
            .add_system(despawn_everything.in_schedule(OnExit(GameState::Win)));
    }
}

fn setup_win_screen(mut commands: Commands, game_textures: Res<GameTextures>) {
    commands.insert_resource(ClearColor(BACKGROUND_COLOUR));
    commands.spawn(Camera2dBundle::default());
    commands.spawn(SpriteBundle {
        texture: game_textures.you_win.clone(),
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, 20.0),
            scale: Vec3::new(1.0, 1.0, 1.0),
            ..Default::default()
        },
        ..Default::default()
    });
}
