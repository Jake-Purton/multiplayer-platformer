// click and it sends out a circle hitbox from the player
// when it hits something 
// - the player is brought towards the object 
// - the object is brought towards the player
// on right click
use bevy::{prelude::*, sprite::collide_aabb::collide, window::PrimaryWindow};

use crate::{startup_plugin::{PlayerCamera, GameTextures}, player::Player, GameState, HOOK_SPRITE_SIZE, platform::Wall, HOOK_SPEED};
// use crate::moving_block::MovableWall;

pub struct GrapplePlugin;

impl Plugin for GrapplePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system(send_out_hook.in_set(OnUpdate(GameState::Gameplay)))
            .add_system(hook_sensor.after(hook_movement).in_set(OnUpdate(GameState::Gameplay)))
            .add_system(hook_movement.in_set(OnUpdate(GameState::Gameplay)))
            .add_system(delete_and_rotate_hooks.in_set(OnUpdate(GameState::Gameplay)));

    }
}

#[derive(Component)]
pub struct MovingGrappleHook {
    direction: Vec2,
    size: Vec2,
    timer: Timer,
}

#[derive(Component)]
pub struct Hook;

// sends out a hitbox to act as the hook
fn send_out_hook (
    mouse: Res<Input<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut commands: Commands,
    camera: Query<&Transform, With<PlayerCamera>>,
    player: Query<&Transform, With<Player>>,
    game_textures: Res<GameTextures>,
    hooks: Query<&Hook>,
) {
    if mouse.just_pressed(MouseButton::Right) {

        if hooks.is_empty() {

            let window = windows.get_single().unwrap();
            let camera = camera.single();
    
            if let Some(mut position) = window.cursor_position() {
                position.x -= (window.width() / 2.0) - camera.translation.x;
                position.y -= (window.height() / 2.0) - camera.translation.y;
    
                let player = player.single();
    
                let mut direction = position - player.translation.truncate();
    
                direction /= direction.length();
    
                let angle = Vec2::Y.angle_between(direction);
    
                commands
                    .spawn(SpriteBundle {
                        texture: game_textures.hook.clone(),
                        sprite: Sprite {
                            custom_size: Some(HOOK_SPRITE_SIZE),
                            ..Default::default()
                        },
                        transform: Transform {
                            translation: player.translation + (20.0 * direction).extend(11.0),
                            rotation: Quat::from_rotation_z(angle),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .insert(MovingGrappleHook {
                        direction,
                        size: HOOK_SPRITE_SIZE,
                        timer: Timer::from_seconds(0.7, TimerMode::Once)
                    })
                    .insert(Hook);
            
            }
        }
    }
}

fn hook_sensor (
    hooks: Query<(Entity, &MovingGrappleHook, &Transform)>,
    walls: Query<(&Wall, &Transform)>,
    // platforms: Query<(&MovableWall, &Transform), Without<Wall>>,
    mut commands: Commands,
) {

    for (entity, hook, hook_transform) in hooks.iter() {

        for (wall, wall_transform) in walls.iter() {

            if collide(
                hook_transform.translation, 
                hook.size, 
                wall_transform.translation, 
                wall.size
            ).is_some() {

                commands.entity(entity).remove::<MovingGrappleHook>();

            }
        }

        // for (wall, wall_transform) in platforms.iter() {

        //     if collide(
        //         hook_transform.translation, 
        //         hook.size, 
        //         wall_transform.translation, 
        //         wall.size
        //     ).is_some() {

        //         commands.entity(entity).remove::<MovingGrappleHook>();

        //     }
        // }
        
    }
}

fn hook_movement (
    mut hooks: Query<(Entity, &mut MovingGrappleHook, &mut Transform)>,
    time: Res<Time>,
    mut commands: Commands,
) {

    for (entity, mut hook, mut transform)in hooks.iter_mut() {

        hook.timer.tick(time.delta());

        if hook.timer.just_finished() {

            commands.entity(entity).despawn();

        } else {

            transform.translation += (HOOK_SPEED * hook.direction * time.delta_seconds()).extend(0.0);

        }


    }
    
}

fn delete_and_rotate_hooks (
    mut grappling_hook: Query<(Entity, &mut Transform), (Without<MovingGrappleHook>, With<Hook>)>,
    player: Query<&Transform, (With<Player>, Without<Hook>)>,
    mut commands: Commands,
    keys: Res<Input<KeyCode>>
) {
    for (hook, mut hook_t) in grappling_hook.iter_mut() {
        if keys.just_pressed(KeyCode::Space) {
            commands.entity(hook).despawn()
        } else {

            let transform = player.single();
            let direction = hook_t.translation - transform.translation;
            let angle = Vec2::Y.angle_between(direction.truncate());

            hook_t.rotation = Quat::from_rotation_z(angle);

        }
    }
}