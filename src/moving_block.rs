use bevy::{prelude::*, sprite::collide_aabb::collide, utils::HashMap, window::PrimaryWindow};
use bevy_rapier2d::{dynamics::RigidBody, geometry::Collider, prelude::Velocity};
use bevy_renet::renet::{DefaultChannel, RenetClient};

use crate::{
    messages::ClientMessageUnreliable, run_if::run_if_online, startup_plugin::PlayerCamera,
    CurrentLevel, GameState, MAP_SCALE,
};

pub struct MovingBlockPlugin;

impl Plugin for MovingBlockPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(movable_walls.in_set(OnUpdate(GameState::Gameplay)))
            .add_system(moving_wall.in_set(OnUpdate(GameState::Gameplay)))
            .add_system(spawn_multiplayer_walls.in_set(OnUpdate(GameState::Gameplay)))
            .add_system(
                send_block_positions
                    .run_if(run_if_online)
                    .in_set(OnUpdate(GameState::Gameplay)),
            )
            .insert_resource(BlockMap::new());
    }
}

#[derive(Component)]
pub struct MovableWall {
    pub size: Vec2,
    pub unique_id: i32,
}

#[derive(Component)]
struct MultiplayerWall {
    client_id: u64,
    wall_id: i32,
}

#[derive(Component)]
pub struct MovingWall;

fn movable_walls(
    walls: Query<(&Transform, &MovableWall, Entity), Without<MovingWall>>,
    mouse: Res<Input<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut commands: Commands,
    camera: Query<&Transform, With<PlayerCamera>>,
) {
    let window = windows.get_single().unwrap();
    let camera = camera.single();

    // get the mouse cursor position
    if let Some(mut position) = window.cursor_position() {
        position.x -= (window.width() / 2.0) - camera.translation.x;
        position.y -= (window.height() / 2.0) - camera.translation.y;

        // if the user clicked the left mouse button
        if mouse.just_pressed(MouseButton::Left) {
            // iterate over every wall
            for (transform, wall, entity) in walls.iter() {

                // check if the wall intersects the cursor
                if collide(
                    transform.translation,
                    wall.size,
                    Vec3::new(position.x, position.y, 0.0),
                    Vec2::new(1.0, 1.0),
                )
                .is_some()
                {
                    // if it intersects, the wall is moving
                    commands.entity(entity).insert(MovingWall);
                    break;
                }
            }
        }
    }
}

// a data structure that has a hashmap
// the key is the level number
// the value is a tuple of (playerid, block_id, position_of_block)
#[derive(Resource)]
pub struct BlockMap {
    // level num - (playerid, block_id, pos)
    pub blocks: HashMap<u8, Vec<(u64, i32, Vec2)>>,
}

// a function to spawn the walls that other players control
fn spawn_multiplayer_walls(
    // the resource with the positions of the walls
    block_map: Res<BlockMap>,

    // the walls that have already been spawned
    mut walls: Query<(&mut Transform, &MultiplayerWall)>,

    // the level that our player is on
    current_level: Res<CurrentLevel>,

    mut commands: Commands,
) {
    // if there are any blocks already on this level that need to be spawned or updated
    if let Some(block_vec) = block_map.blocks.get(&current_level.level_number) {
        // makes a vector with tuples of (player_id, block_id, position, boolean)
        // in regards to the boolean, true means it needs to be spawned
        // false means it has already been spawned
        let mut vec_bool: Vec<(u64, i32, Vec2, bool)> =
            block_vec.iter().map(|a| (a.0, a.1, a.2, true)).collect();

        // iterating over the walls that have already been spawned
        for (mut transform, multiplayer) in walls.iter_mut() {
            for i in &mut vec_bool {
                // if the wall matches
                if multiplayer.wall_id == i.1 && multiplayer.client_id == i.0 {
                    // updates position
                    transform.translation.x = i.2.x;
                    transform.translation.y = i.2.y;
                    // wall has been updated so boolean is set to false
                    i.3 = false;
                }
            }
        }
        // loops over and pops the stack
        while let Some((client_id, wall_id, pos, bool)) = vec_bool.pop() {
            // doesn't need to be updated
            if !bool {
                continue;
            } else {
                // spawns a block with the correct components and in the right position
                let size = Vec2::new(MAP_SCALE, MAP_SCALE);
                commands
                    .spawn(SpriteBundle {
                        sprite: Sprite {
                            color: Color::rgba(1.0, 0.1, 0.5, 0.7),
                            custom_size: Some(size),
                            ..default()
                        },
                        transform: Transform {
                            translation: Vec3 {
                                x: pos.x,
                                y: pos.y,
                                z: 15.0,
                            },
                            ..default()
                        },
                        ..Default::default()
                    })
                    .insert(RigidBody::Fixed)
                    .insert(TransformBundle::from(Transform::from_xyz(
                        pos.x, pos.y, 10.0,
                    )))
                    .insert(Collider::cuboid(size.x / 2.0, size.y / 2.0))
                    .insert(MultiplayerWall { client_id, wall_id })
                    .insert(Velocity::default());
            }
        }
    }
}

// a system that sends the current positions of all visible blocks to the server.
fn send_block_positions(
    walls: Query<(&Transform, &MovableWall)>,
    level: Res<CurrentLevel>,
    mut client: ResMut<RenetClient>,
) {
    // iterates over all of the movable wall entities and sends the level, wall_id and position to the server
    for wall in walls.iter() {
        let message = ClientMessageUnreliable::WallPos {
            level: level.level_number,
            wall_id: wall.1.unique_id,
            pos: wall.0.translation.truncate(),
        };
        let input_message = bincode::serialize(&message).unwrap();
        client.send_message(DefaultChannel::Unreliable, input_message);
    }
}

fn moving_wall(
    mut moving_walls: Query<(&mut Velocity, Entity, &Transform), With<MovingWall>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mouse: Res<Input<MouseButton>>,
    camera: Query<&Transform, (With<PlayerCamera>, Without<MovingWall>)>,
    mut commands: Commands,
) {
    // if there are moving walls
    if !moving_walls.is_empty() {
        // if the user is dragging the mouse
        if mouse.pressed(MouseButton::Left) {
            let camera = camera.single();
            let window = windows.get_single().unwrap();
            let pos = window.cursor_position().unwrap();

            for (mut vel, _, block_transform) in moving_walls.iter_mut() {

                // move the wall towards the mouse with a velocity
                let pos = Vec3::new(
                    pos.x - (window.width() / 2.0) + camera.translation.x,
                    pos.y - (window.height() / 2.0) + camera.translation.y,
                    block_transform.translation.z,
                );
                let velocity = (pos - block_transform.translation).truncate();
                vel.linvel = (velocity + vel.linvel) * 0.8;
            }
        } else {
            // if they are not dragging the mouse, wall is no longer moving
            for (_, entity, _) in moving_walls.iter() {
                commands.entity(entity).remove::<MovingWall>();
            }
        }
    }
}

impl BlockMap {
    fn new() -> Self {
        BlockMap {
            blocks: HashMap::new(),
        }
    }
}
