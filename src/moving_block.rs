use bevy::{prelude::*, sprite::collide_aabb::collide, window::PrimaryWindow, utils::HashMap};
use bevy_rapier2d::prelude::Velocity;
use bevy_renet::renet::{RenetClient, DefaultChannel};

use crate::{startup_plugin::PlayerCamera, GameState, next_level::{run_if_online, run_if_not_online}, messages::ClientMessageUnreliable};

pub struct MovingBlockPlugin;

impl Plugin for MovingBlockPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(movable_walls.in_set(OnUpdate(GameState::Gameplay)))
            .add_system(moving_wall_multiplayer.run_if(run_if_online).in_set(OnUpdate(GameState::Gameplay)))
            .add_system(moving_wall_singleplayer.run_if(run_if_not_online).in_set(OnUpdate(GameState::Gameplay)))
            .insert_resource(MovableWallMultiplayerVel::new());
    }
}

// ideas:
// walls that the block cannot go through but the player can
// blocks that fall when not being held
// button

#[derive(Component)]
pub struct MovableWall {
    pub size: Vec2,
    pub unique_id: i32,
}

#[derive(Resource)]
pub struct MovableWallMultiplayerVel {
    // (player_id, block_id, level_number), block velocity
    pub velocity: HashMap<i32, Vec2>
}

impl MovableWallMultiplayerVel {
    fn new() -> Self {
        Self { velocity: HashMap::new() }
    }
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

    if let Some(mut position) = window.cursor_position() {
        position.x -= (window.width() / 2.0) - camera.translation.x;
        position.y -= (window.height() / 2.0) - camera.translation.y;

        if mouse.just_pressed(MouseButton::Left) {
            for (transform, wall, entity) in walls.iter() {
                if collide(
                    transform.translation,
                    wall.size,
                    Vec3::new(position.x, position.y, 0.0),
                    Vec2::new(1.0, 1.0),
                )
                .is_some()
                {
                    commands.entity(entity).insert(MovingWall);
                    break;
                }
            }
        }
    }
}

fn moving_wall_multiplayer (
    mut moving_walls: Query<(&mut Velocity, Entity, &Transform, &MovableWall), With<MovingWall>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mouse: Res<Input<MouseButton>>,
    camera: Query<&Transform, (With<PlayerCamera>, Without<MovingWall>)>,
    mut commands: Commands,
    mut client: ResMut<RenetClient>,
    server_vels: Res<MovableWallMultiplayerVel>,
) {
    if !moving_walls.is_empty() {
        if mouse.pressed(MouseButton::Left) {
            let camera = camera.single();
            let window = windows.get_single().unwrap();
            let pos = window.cursor_position().unwrap();

            for (mut vel, _, block_transform, mvbwll) in moving_walls.iter_mut() {
                let pos = Vec3::new(
                    pos.x - (window.width() / 2.0) + camera.translation.x,
                    pos.y - (window.height() / 2.0) + camera.translation.y,
                    block_transform.translation.z,
                );
                let mut velocity = (pos - block_transform.translation).truncate();
                // send this to the server

                let message = ClientMessageUnreliable::MovingWallVelocity { wall_id: mvbwll.unique_id, velocity };

                client.send_message(
                    DefaultChannel::Unreliable,
                    bincode::serialize(&message).unwrap(),
                );

                // adds the server velocities
                for i in server_vels.velocity.keys() {
                    if *i == mvbwll.unique_id {
                        velocity = server_vels.velocity.get(i).unwrap().clone();
                    }
                }

                vel.linvel = (velocity + vel.linvel) * 0.8;
            }
        } else {
            for (_, entity, _, _) in moving_walls.iter() {
                // send velocity = 0 to the server
                commands.entity(entity).remove::<MovingWall>();
            }
        }
    }
}

fn moving_wall_singleplayer (
    mut moving_walls: Query<(&mut Velocity, Entity, &Transform), With<MovingWall>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mouse: Res<Input<MouseButton>>,
    camera: Query<&Transform, (With<PlayerCamera>, Without<MovingWall>)>,
    mut commands: Commands,
) {
    if !moving_walls.is_empty() {
        if mouse.pressed(MouseButton::Left) {
            let camera = camera.single();
            let window = windows.get_single().unwrap();
            let pos = window.cursor_position().unwrap();

            for (mut vel, _, block_transform) in moving_walls.iter_mut() {
                let pos = Vec3::new(
                    pos.x - (window.width() / 2.0) + camera.translation.x,
                    pos.y - (window.height() / 2.0) + camera.translation.y,
                    block_transform.translation.z,
                );
                let velocity = (pos - block_transform.translation).truncate();
                vel.linvel = (velocity + vel.linvel) * 0.8;
            }
        } else {
            for (_, entity, _) in moving_walls.iter() {
                commands.entity(entity).remove::<MovingWall>();
            }
        }
    }
}
