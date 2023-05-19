use std::f32::consts::PI;

use ::bevy::prelude::*;
use bevy::sprite::collide_aabb::collide;
use bevy_rapier2d::prelude::{KinematicCharacterController, KinematicCharacterControllerOutput};

use crate::{
    platform::{LowestPoint, KillerWall},
    GRAVITY_CONSTANT, GameState, grappling_hook::{Hook, MovingGrappleHook}, GRAPPLE_SPEED,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system(rapier_player_movement.in_set(OnUpdate(GameState::Gameplay)))
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

pub fn rapier_player_movement (
    mut controllers: Query<(
        &mut KinematicCharacterController, 
        &mut Player, 
        &KinematicCharacterControllerOutput,
        &Transform,
    )>,
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    grappling_hook: Query<(&Hook, &Transform), Without<MovingGrappleHook>>,
) {
    for (mut controller, mut player, output, player_transform) in controllers.iter_mut() {

        let delta_s = time.delta_seconds();

        let mut movement = Vec2::new(0.0, 0.0);

        if player.velocity.x.is_nan() {
            player.velocity.x = 0.0;
        } 
        if player.velocity.y.is_nan() {
            player.velocity.y = 0.0;
        }

        // make sure it hits the ceiling
        if output.effective_translation.y.is_sign_positive() && (output.effective_translation.y * 10.0).round() == 0.0 {
            player.velocity.y = 0.0;
        }

        if keys.pressed(KeyCode::D) {
            movement += Vec2::new(player.run_speed, 0.0);
        }        
        if keys.pressed(KeyCode::A) {
            movement += Vec2::new(-player.run_speed, 0.0);
        }        

        if !output.grounded {
            player.velocity += GRAVITY_CONSTANT * delta_s;
        } else {

            player.velocity.x = 0.0;

            if keys.pressed(KeyCode::Space) {
                player.velocity.y = player.jump_velocity;
            } else {
                player.velocity.y = 0.0;
            }
        }

        movement += player.velocity;

        if grappling_hook.is_empty() {

            controller.translation = Some(movement * delta_s);

            if keys.just_pressed(KeyCode::F) {
                println!("velocity: {}, movement: {} ", player.velocity, movement);
            }

        } else {

            let (_, hook_transform) = grappling_hook.single();

            let mut direction = (hook_transform.translation - player_transform.translation).truncate();
            let resolved = resolve_forces(direction, movement);
            let resolved_velocity = resolve_forces(direction, player.velocity);
            
            movement = resolved;

            if keys.pressed(KeyCode::W) {
                direction /= direction.distance(Vec2::ZERO);
                movement += direction * GRAPPLE_SPEED;
            }

            player.velocity = resolved_velocity;
            controller.translation = Some(movement * delta_s);

        }



    }
}

fn resolve_forces (
    hook_direction: Vec2,
    velocity: Vec2,
) -> Vec2 {
    let angle = (PI / 2.0) - hook_direction.angle_between(velocity);
    let magnitude = (velocity.x.powi(2) + velocity.y.powi(2)).sqrt();
    let unresolved_velocity = magnitude * angle.cos();
    let x_y_angle =  (PI / 2.0) - hook_direction.angle_between(Vec2::X);

    Vec2::new(unresolved_velocity * x_y_angle.cos(), unresolved_velocity * x_y_angle.sin())
}

fn player_death_fall_off_the_map (
    player: Query<&Transform, With<Player>>,
    lowest_point: Res<LowestPoint>,
    mut game_state: ResMut<NextState<GameState>>,
) {

    let player = player.single();
    if player.translation.y <= -lowest_point.point {
        game_state.set(GameState::Death)
    }
}

fn killer_wall (
    walls: Query<(&KillerWall, &Transform)>,
    player: Query<(&Transform, &Player)>,
    mut game_state: ResMut<NextState<GameState>>,
) {

    let player = player.single();

    for wall in walls.iter() {
        if collide(wall.1.translation, wall.0.size + Vec2::ONE, player.0.translation, player.1.size).is_some() {
            game_state.set(GameState::Death)
        }
    }
}
