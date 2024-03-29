use crate::{main_menu::HostClient, moving_block::MovableWall, FELLA_SPRITE_SIZE};

use crate::{player::Player, startup_plugin::GameTextures, CurrentLevel, GameState, MAP_SCALE};
use bevy::{prelude::*, sprite::collide_aabb::collide, utils::HashMap};
use bevy_rapier2d::prelude::*;

#[derive(Component)]
pub struct KillerWall {
    pub size: Vec2,
}

#[derive(Component)]
pub struct Wall {
    pub size: Vec2,
}

#[derive(Component)]
pub struct Goal {
    size: Vec2,
}

pub struct PlatformPlugin;

impl Plugin for PlatformPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Maps {
            maps: HashMap::new(),
        })
        .add_system(platform_from_map_system.in_schedule(OnEnter(GameState::Gameplay)))
        .add_system(next_level_system.in_set(OnUpdate(GameState::Gameplay)));
    }
}

// this function returns a level directory based on the setting and the level number
pub fn level_directory(level_number: u8, hc: &HostClient) -> String {
    match hc {
        HostClient::Client => format!("assets/levels/downloads/level-{}.txt", level_number),
        HostClient::Host => format!("assets/levels/multiplayer/level-{}.txt", level_number),
        HostClient::Play => format!("assets/levels/level-{}.txt", level_number),
    }
}

// macro to create a normal white wall
macro_rules! create_wall {
    ($commands:expr, $x:expr, $y:expr, $size:expr) => {{
        $commands
            .spawn(SpriteBundle {
                sprite: Sprite {
                    color: Color::rgba(1.0, 1.0, 1.0, 1.0),
                    custom_size: Some($size),
                    ..default()
                },
                ..Default::default()
            })
            .insert(RigidBody::Fixed)
            .insert(TransformBundle::from(Transform::from_xyz($x, $y, 10.0)))
            .insert(Collider::cuboid($size.x / 2.0, $size.y / 2.0))
            .insert(Wall { size: $size });
    }};
}

// macro to create a green goal
macro_rules! create_level_end {
    ($commands:expr, $x:expr, $y:expr, $size:expr) => {{
        $commands
            .spawn(SpriteBundle {
                sprite: Sprite {
                    color: Color::rgba(0.0, 1.0, 0.0, 1.0),
                    custom_size: Some($size),
                    ..default()
                },
                transform: Transform {
                    translation: Vec3 {
                        x: $x,
                        y: $y,
                        z: 10.0,
                    },
                    ..default()
                },
                ..Default::default()
            })
            .insert(Goal { size: $size })
            .insert(RigidBody::Fixed)
            .insert(TransformBundle::from(Transform::from_xyz($x, $y, 10.0)))
            .insert(Collider::cuboid($size.x / 2.0, $size.y / 2.0));
    }};
}

// macro to create a red killer wall
macro_rules! create_killer_wall {
    ($commands:expr, $x:expr, $y:expr, $size:expr) => {{
        $commands
            .spawn(SpriteBundle {
                sprite: Sprite {
                    color: Color::rgba(1.0, 0.0, 0., 1.0),
                    custom_size: Some($size),
                    ..default()
                },
                transform: Transform {
                    translation: Vec3 {
                        x: $x,
                        y: $y,
                        z: 10.0,
                    },
                    ..default()
                },
                ..Default::default()
            })
            .insert(KillerWall { size: $size })
            .insert(RigidBody::Fixed)
            .insert(TransformBundle::from(Transform::from_xyz($x, $y, 10.0)))
            .insert(Collider::cuboid($size.x / 2.0, $size.y / 2.0));
    }};
}

// create a light blue/green movable wallw
macro_rules! create_movable_wall {
    ($commands:expr, $x:expr, $y:expr, $size:expr, $level_number:expr) => {{

        // multiply them by different large primes to guarantee each block has 
        // a unique number, but each block has the same number every time.
        // movable walls need unique identifiers for multiplayer mode.
        let n1: i32 = ($x as i32 * 1117) + ($y as i32 * 4339) + ($level_number as i32 * 27);

        $commands
            .spawn(SpriteBundle {
                sprite: Sprite {
                    color: Color::rgba(0.0, 1.0, 1.0, 0.7),
                    custom_size: Some($size),
                    ..default()
                },
                transform: Transform {
                    translation: Vec3 {
                        x: $x,
                        y: $y,
                        z: 15.0,
                    },
                    ..default()
                },
                ..Default::default()
            })
            .insert(MovableWall { size: $size, unique_id: n1 })
            .insert(RigidBody::Dynamic)
            .insert(TransformBundle::from(Transform::from_xyz($x, $y, 10.0)))
            .insert(Collider::cuboid($size.x / 2.0, $size.y / 2.0))
            .insert(Velocity::default());
    }};
}

// this resource tells us the lowest point so the player despawns 
// when it falls off of the map
#[derive(Resource)]
pub struct LowestPoint {
    pub point: f32,
}

#[derive(Resource)]
pub struct Maps {
    // a vector of all of the maps
    pub maps: HashMap<u8, Vec<Vec<u8>>>,
}

fn platform_from_map_system(
    mut commands: Commands,
    game_textures: Res<GameTextures>,
    current_level: Res<CurrentLevel>,
    maps: Res<Maps>,
) {
    // get the map
    let map = maps
        .maps
        .get(&(current_level.level_number))
        .unwrap()
        .clone();

    // get the lowest point on the map
    commands.insert_resource(LowestPoint {
        point: (map.len() as f32 * MAP_SCALE / 2.0) + MAP_SCALE + 100.0,
    });

    let mut player_spawned = false;

    for (y, array) in map.iter().enumerate() {
        for (x, val) in array.iter().enumerate() {

            // get the x and y position
            let x = x as f32 * MAP_SCALE;
            let y = y as f32 * MAP_SCALE;

            if *val == 1 {
                // spawn a normal wall
                create_wall!(commands, x, y, Vec2::new(MAP_SCALE, MAP_SCALE))
            } else if *val == 2 {
                // spawn a movable wall
                create_movable_wall!(
                    commands,
                    x,
                    y,
                    Vec2::new(MAP_SCALE, MAP_SCALE),
                    current_level.level_number
                )
            } else if *val == 3 && !player_spawned {

                player_spawned = true;

                // SPAWN A PLAYER
                commands
                    .spawn(SpriteBundle {
                        texture: game_textures.player.clone(),
                        sprite: Sprite {
                            custom_size: Some(FELLA_SPRITE_SIZE),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .insert(Player::default())
                    .insert(Collider::cuboid(
                        FELLA_SPRITE_SIZE.x / 2.0,
                        FELLA_SPRITE_SIZE.y / 2.0,
                    ))
                    .insert(KinematicCharacterController {
                        autostep: Some(CharacterAutostep {
                            max_height: CharacterLength::Absolute(0.5),
                            min_width: CharacterLength::Absolute(0.2),
                            include_dynamic_bodies: true,
                        }),
                        apply_impulse_to_dynamic_bodies: true,
                        snap_to_ground: Some(CharacterLength::Absolute(0.1)),
                        custom_mass: Some(1000.0),
                        ..Default::default()
                    })
                    .insert(KinematicCharacterControllerOutput::default())
                    .insert(TransformBundle::from(Transform::from_xyz(x, y, 10.0)));
            } else if *val == 4 {
                // Spawn a killer wall Thats slightly smaller than the other blocks in height
                create_killer_wall!(commands, x, y, Vec2::new(MAP_SCALE, MAP_SCALE - 10.0))
            } else if *val == 5 {
                // spawn a goal
                create_level_end!(commands, x, y, Vec2::new(MAP_SCALE, MAP_SCALE))
            }
        }
    }
}

fn next_level_system(
    player: Query<(&Player, &Transform)>,
    goals: Query<(&Goal, &Transform)>,
    mut level: ResMut<CurrentLevel>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    let (player, player_transform) = player.single();
    for (goal, goal_transform) in goals.iter() {
        // if the player collides with the goal
        if collide(
            player_transform.translation,
            player.size,
            goal_transform.translation,
            goal.size + Vec2::ONE,
        )
        .is_some()
        {
            // increment the level number
            level.level_number += 1;
            // go to the next level stage
            game_state.set(GameState::NextLevel)
        }
    }
}
