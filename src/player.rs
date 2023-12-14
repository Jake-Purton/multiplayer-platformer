use std::f32::consts::PI;

use ::bevy::prelude::*;
use bevy::sprite::collide_aabb::collide;
use bevy_rapier2d::prelude::{KinematicCharacterController, KinematicCharacterControllerOutput};

use crate::{
    grappling_hook::{Hook, MovingGrappleHook},
    platform::{KillerWall, LowestPoint},
    GameState, GRAPPLE_SPEED, GRAVITY_CONSTANT,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(rapier_player_movement.in_set(OnUpdate(GameState::Gameplay)))
            .add_system(player_death_fall_off_the_map.in_set(OnUpdate(GameState::Gameplay)))
            .add_system(killer_wall.in_set(OnUpdate(GameState::Gameplay)));
    }
}

#[derive(Component)]
pub struct Player {
    pub run_speed: f32,
    pub velocity: Vec2,
    pub jump_velocity: f32,
    pub size: Vec2,
}

pub fn rapier_player_movement(
    // the new query for the player
    mut controllers: Query<(
        &mut KinematicCharacterController,
        &mut Player,
        &KinematicCharacterControllerOutput,
        &Transform,
    )>,
    // keyboard input
    keys: Res<Input<KeyCode>>,
    // the resource that gives me access to the time since the last update
    time: Res<Time>,
    // the query for the grappling hook
    grappling_hook: Query<(&Hook, &Transform), Without<MovingGrappleHook>>,
) {
    // iterate over the players
    for (mut controller, mut player, output, player_transform) in controllers.iter_mut() {
        let delta_s = time.delta_seconds();

        let mut movement = Vec2::new(0.0, 0.0);
        // updates the values if they were divided by 0
        if player.velocity.x.is_nan() {
            player.velocity.x = 0.0;
        }
        if player.velocity.y.is_nan() {
            player.velocity.y = 0.0;
        }
        // make sure it hits the ceiling
        if output.effective_translation.y.is_sign_positive()
            && (output.effective_translation.y * 10.0).round() == 0.0
        {
            // if the player hits the ceiling y velocity is set to 0
            player.velocity.y = 0.0;
        }
        if keys.pressed(KeyCode::D) {
            // move right
            movement += Vec2::new(player.run_speed, 0.0);
        }
        if keys.pressed(KeyCode::A) {
            // move left
            movement += Vec2::new(-player.run_speed, 0.0);
        }
        if !output.grounded {
            // accellerate downwards if in the air 
            player.velocity += GRAVITY_CONSTANT * delta_s;
        } else {
            // if player is on the floor, x velocity is set to 0 (friction)
            player.velocity.x = 0.0;

            if keys.pressed(KeyCode::Space) {
                // player jumps
                player.velocity.y = player.jump_velocity;
            } else {
                // player doesn't jump
                player.velocity.y = 0.0;
            }
        }
        // add the velocity to the movement
        movement += player.velocity;

        if grappling_hook.is_empty() {
            // no grappling hook = update translation
            controller.translation = Some(movement * delta_s);
            // let me see the current velocity and movement for testing purposes
            if keys.just_pressed(KeyCode::F) {
                println!("velocity: {}, movement: {} ", player.velocity, movement);
            }
        } else {
            // get the position of the grappling hook
            let (_, hook_transform) = grappling_hook.single();
            // the direction of the hook as a vec2 
            let mut direction =
                (hook_transform.translation - player_transform.translation).truncate() /*truncate removes the z part (vec3 -> vec2)*/;
            // resolve the forces to get the velocity and movement for the player
            let resolved = resolve_forces(direction, movement);
            let resolved_velocity = resolve_forces(direction, player.velocity);

            movement = resolved;

            if keys.pressed(KeyCode::W) {
                // divide the direction by its magnitude (calculated by pythagoras' theorem)
                direction /= direction.distance(Vec2::ZERO);
                // add this movement towards the hook to the total player movement
                movement += direction * GRAPPLE_SPEED;
            }
            // update the velocity and the translation
            player.velocity = resolved_velocity;
            controller.translation = Some(movement * delta_s);
        }
    }
}

fn resolve_forces(hook_direction: Vec2, velocity: Vec2) -> Vec2 {
    // find the angle from the desired motion
    let angle = (PI / 2.0) - hook_direction.angle_between(velocity);
    // pythagoras to get the magnitude of the hypotenuse
    let magnitude = (velocity.x.powi(2) + velocity.y.powi(2)).sqrt();
    // do some triangle maths to get the velocities relative to the hook angle
    let unresolved_velocity = magnitude * angle.cos();

    let x_y_angle = (PI / 2.0) - hook_direction.angle_between(Vec2::X);

    // ignore the velocity relative to the normal of the hook and focus only on the tangential velocity
    // this means that the player cannot move away from the hook, only around it (radius is constant)
    Vec2::new(
        unresolved_velocity * x_y_angle.cos(),
        unresolved_velocity * x_y_angle.sin(),
    )
}

fn player_death_fall_off_the_map(
    player: Query<&Transform, With<Player>>,
    lowest_point: Res<LowestPoint>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    let player = player.single();
    if player.translation.y <= -lowest_point.point {
        game_state.set(GameState::Death)
    }
}

fn killer_wall(
    walls: Query<(&KillerWall, &Transform)>,
    player: Query<(&Transform, &Player)>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    let player = player.single();

    for wall in walls.iter() {
        if collide(
            wall.1.translation,
            wall.0.size + Vec2::ONE,
            player.0.translation,
            player.1.size,
        )
        .is_some()
        {
            game_state.set(GameState::Death)
        }
    }
}
