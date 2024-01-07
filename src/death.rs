use bevy::prelude::*;

use crate::{
    startup_plugin::{despawn_everything, GameTextures},
    GameState, BACKGROUND_COLOUR,
};

pub struct DeathPlugin;

impl Plugin for DeathPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(setup_death.in_schedule(OnEnter(GameState::Death)))
            .add_system(restart.in_set(OnUpdate(GameState::Death)))
            .add_system(background.in_set(OnUpdate(GameState::Death)))
            .add_system(despawn_everything.in_schedule(OnExit(GameState::Death)));
    }
}

fn setup_death(mut commands: Commands, game_textures: Res<GameTextures>) {
    // setup the menu and text
    commands.insert_resource(ClearColor(BACKGROUND_COLOUR));
    commands.spawn(Camera2dBundle::default());
    commands.spawn(SpriteBundle {
        texture: game_textures.r_to_respawn.clone(),
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, 20.0),
            scale: Vec3::new(1.0, 1.0, 1.0),
            ..Default::default()
        },
        ..Default::default()
    });
}

fn restart(keys: Res<Input<KeyCode>>, mut game_state: ResMut<NextState<GameState>>) {
    // go back to gameplay when r is pressed
    if keys.just_pressed(KeyCode::R) {
        game_state.set(GameState::Gameplay);
    }
}

fn background(time: Res<Time>, mut commands: Commands) {
    // changes the colour of the background over time
    let time = (time.elapsed_seconds() * 2.0).sin() / 8.0;
    commands.insert_resource(ClearColor(BACKGROUND_COLOUR + Color::rgb(time, time, 0.0)));
}
