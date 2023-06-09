use bevy::prelude::*;

use crate::{GameState, startup_plugin::{despawn_everything, GameTextures}, next_level::{run_if_online, delete_hashmap}};

pub struct DeathPlugin;

impl Plugin for DeathPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .add_system(setup_death.in_schedule(OnEnter(GameState::Death)))
            .add_system(restart.in_set(OnUpdate(GameState::Death)))
            .add_system(background.in_set(OnUpdate(GameState::Death)))
            .add_system(despawn_everything.in_schedule(OnExit(GameState::Death)))
            .add_system(delete_hashmap.in_schedule(OnEnter(GameState::Death)).run_if(run_if_online));
    }
}


fn setup_death(mut commands: Commands, game_textures: Res<GameTextures>) {
        commands.insert_resource(ClearColor(Color::rgb(1.0, 0.5, 0.0)));
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

fn restart (keys: Res<Input<KeyCode>>, mut game_state: ResMut<NextState<GameState>>) {
    if keys.just_pressed(KeyCode::R) {
        game_state.set(GameState::Gameplay);
    }
}

fn background (time: Res<Time>, mut commands: Commands) {

    let time = (time.elapsed_seconds() * 2.0).sin() / 8.0;
    commands.insert_resource(ClearColor(Color::rgb(1.0 + time, 0.6 - time, 0.0)));

}