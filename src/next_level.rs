use bevy::prelude::*;

use crate::{
    platform::Maps, CurrentLevel, GameState,
};

#[derive(Resource)]
// doesn't go to the next level instantly
// but waits for the timer
pub struct LevelTimer {
    timer: Timer,
}

pub struct NextLevelPlugin;

impl Plugin for NextLevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(next_level_system.in_schedule(OnEnter(GameState::NextLevel)))
            .add_system(back_to_gameplay.in_set(OnUpdate(GameState::NextLevel)));
    }
}

fn next_level_system(mut commands: Commands) {
    // setup the background
    // start the timer
    commands.insert_resource(LevelTimer {
        timer: Timer::from_seconds(0.5, TimerMode::Once),
    });
    commands.spawn(Camera2dBundle::default());
    commands.insert_resource(ClearColor(Color::rgb(1.0, 1.0, 1.0)))
}

fn back_to_gameplay(
    mut game_state: ResMut<NextState<GameState>>,
    mut timer: ResMut<LevelTimer>,
    time: Res<Time>,
    entities: Query<Entity, Without<Window>>,
    mut commands: Commands,
    current_level: Res<CurrentLevel>,
    maps: Res<Maps>,
) {
    // tick the timer
    timer.timer.tick(time.delta());

    // change the background colour over time
    let percent = timer.timer.percent_left();
    commands.insert_resource(ClearColor(Color::rgb(7.0, percent, percent)));

    // if the timer finished
    if timer.timer.finished() {
        // despawn everything
        for entity in entities.iter() {
            commands.entity(entity).despawn()
        }

        
        if maps.maps.get(&current_level.level_number).is_none() {
            // if there are no other levels go to the win screen
            game_state.set(GameState::Win)
        } else {
            // if there is another level go there
            game_state.set(GameState::Gameplay)
        }
    }
}
