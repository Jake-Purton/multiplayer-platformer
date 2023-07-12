use bevy::prelude::*;

use crate::{
    client::UserIdMap, main_menu::HostClient, platform::Maps, CurrentLevel, GameState,
    MultiplayerSetting,
};

#[derive(Resource)]
pub struct LevelTimer {
    timer: Timer,
}

pub struct NextLevelPlugin;

impl Plugin for NextLevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(next_level_system.in_schedule(OnEnter(GameState::NextLevel)))
            .add_system(
                delete_hashmap
                    .in_schedule(OnEnter(GameState::NextLevel))
                    .run_if(run_if_online),
            )
            .add_system(back_to_gameplay.in_set(OnUpdate(GameState::NextLevel)));
    }
}

pub fn run_if_online(host: Res<MultiplayerSetting>) -> bool {
    !matches!(host.0, HostClient::Play)
}

pub fn delete_hashmap(mut map: ResMut<UserIdMap>) {
    for i in map.0.clone().keys() {
        map.0.remove(i);
        println!("{}", i)
    }
}

fn next_level_system(mut commands: Commands) {
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
    timer.timer.tick(time.delta());
    let percent = timer.timer.percent_left();
    commands.insert_resource(ClearColor(Color::rgb(7.0, percent, percent)));

    if timer.timer.finished() {
        for entity in entities.iter() {
            commands.entity(entity).despawn()
        }

        // if the level it's trying to access is in the list
        if maps.maps.get(&current_level.level_number).is_none() {
            game_state.set(GameState::Win)
        } else {
            game_state.set(GameState::Gameplay)
        }
    }
}
