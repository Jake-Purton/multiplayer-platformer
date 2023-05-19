use std::fs::File;

use bevy::prelude::*;

use crate::{GameState, CurrentLevel, level_directory};

#[derive(Resource)]
pub struct LevelTimer {
    timer: Timer,
}

pub struct NextLevelPlugin;

impl Plugin for NextLevelPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system(next_level_system.in_schedule(OnEnter(GameState::NextLevel)))
            .add_system(back_to_gameplay.in_set(OnUpdate(GameState::NextLevel)));
    }
}

fn next_level_system (
    mut commands: Commands,
) {
    commands.insert_resource(LevelTimer{timer: Timer::from_seconds(0.5, TimerMode::Once)});
    commands.spawn(Camera2dBundle::default());
    commands.insert_resource(ClearColor(Color::rgb(1.0, 1.0, 1.0)))
}

fn back_to_gameplay (
    mut game_state: ResMut<NextState<GameState>>,
    mut timer: ResMut<LevelTimer>,
    time: Res<Time>,
    entities: Query<Entity>,
    mut commands: Commands,
    current_level: Res<CurrentLevel>,
) {

    timer.timer.tick(time.delta());
    let percent = timer.timer.percent_left();
    commands.insert_resource(ClearColor(Color::rgb(7.0, percent, percent)));

    if timer.timer.finished() {
        for entity in entities.iter() {
            commands.entity(entity).despawn()
        }

        match File::open(level_directory(current_level.level_number)) {
            Ok(_) => {
                game_state.set(GameState::Gameplay)
            },
            Err(_) => {
                game_state.set(GameState::Win)
            },
        }
        

    }

}