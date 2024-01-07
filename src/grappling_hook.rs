use bevy::{prelude::*, sprite::collide_aabb::collide, window::PrimaryWindow};

use crate::{
    platform::Wall,
    player::Player,
    startup_plugin::{GameTextures, PlayerCamera},
    GameState, HOOK_SPEED, HOOK_SPRITE_SIZE,
};

pub struct GrapplePlugin;

impl Plugin for GrapplePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(send_out_hook.in_set(OnUpdate(GameState::Gameplay)))
            .add_system(
                hook_sensor
                    // this system runs after the hook movement system, not in paralel
                    .after(hook_movement)
                    .in_set(OnUpdate(GameState::Gameplay)),
            )
            .add_system(hook_movement.in_set(OnUpdate(GameState::Gameplay)))
            .add_system(delete_and_rotate_hooks.in_set(OnUpdate(GameState::Gameplay)));
    }
}

#[derive(Component)]
pub struct MovingGrappleHook {
    // the component that describes a grappling
    // hook as it's moving
    direction: Vec2,
    size: Vec2,
    timer: Timer,
}

#[derive(Component)]
pub struct Hook;

// sends out a hitbox to act as the hook
fn send_out_hook(
    mouse: Res<Input<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut commands: Commands,
    camera: Query<&Transform, With<PlayerCamera>>,
    player: Query<&Transform, With<Player>>,
    game_textures: Res<GameTextures>,
    hooks: Query<&Hook>,
) {
    // when you rightclick
    if mouse.just_pressed(MouseButton::Right) && hooks.is_empty() {
        
        let window = windows.get_single().unwrap();
        let camera = camera.single();

        if let Some(mut position) = window.cursor_position() {
            // calculate the cursor position
            position.x -= (window.width() / 2.0) - camera.translation.x;
            position.y -= (window.height() / 2.0) - camera.translation.y;

            let player = player.single();

            // vector from the player towards the cursor
            let mut direction = position - player.translation.truncate();

            // normalise the vector
            direction /= direction.length();

            // the angle that the hook makes against the player
            let angle = Vec2::Y.angle_between(direction);

            // spawn the hook
            commands
                .spawn(SpriteBundle {
                    texture: game_textures.hook.clone(),
                    sprite: Sprite {
                        custom_size: Some(HOOK_SPRITE_SIZE),
                        ..Default::default()
                    },
                    transform: Transform {
                        // spawn the hook 20 pixels away from the center of the player
                        // .extend() adds a z value
                        translation: player.translation + (20.0 * direction).extend(11.0),
                        // rotate it about the z axis so that it faces away from the player
                        rotation: Quat::from_rotation_z(angle),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(MovingGrappleHook {
                    // add the moving grapple component
                    direction,
                    size: HOOK_SPRITE_SIZE,
                    timer: Timer::from_seconds(0.7, TimerMode::Once),
                })
                .insert(Hook);
        }
    }
}

fn hook_sensor(
    hooks: Query<(Entity, &MovingGrappleHook, &Transform)>,
    walls: Query<(&Wall, &Transform)>,
    mut commands: Commands,
) {
    for (entity, hook, hook_transform) in hooks.iter() {
        for (wall, wall_transform) in walls.iter() {
            // if the hook collides with the wall
            if collide(
                hook_transform.translation,
                hook.size,
                wall_transform.translation,
                wall.size,
            )
            .is_some()
            {
                // remove the "moving" component so that the hook stops moving
                // and doesnt get despawned
                commands.entity(entity).remove::<MovingGrappleHook>();
            }
        }
    }
}

fn hook_movement(
    // get the moving hooks
    mut hooks: Query<(Entity, &mut MovingGrappleHook, &mut Transform)>,

    time: Res<Time>,
    mut commands: Commands,
) {
    for (entity, mut hook, mut transform) in hooks.iter_mut() {
        // tick the timer
        hook.timer.tick(time.delta());

        // if time's up
        if hook.timer.just_finished() {
            //despawn
            commands.entity(entity).despawn();

        // otherwise
        } else {
            // move the hook in the direction it's going
            transform.translation +=
                (HOOK_SPEED * hook.direction * time.delta_seconds()).extend(0.0);
        }
    }
}

fn delete_and_rotate_hooks(
    mut grappling_hook: Query<(Entity, &mut Transform), (Without<MovingGrappleHook>, With<Hook>)>,
    player: Query<&Transform, (With<Player>, Without<Hook>)>,
    mut commands: Commands,
    keys: Res<Input<KeyCode>>,
) {
    for (hook, mut hook_t) in grappling_hook.iter_mut() {

        // if they pressed space
        if keys.just_pressed(KeyCode::Space) {
            // hook is deleted
            commands.entity(hook).despawn()

        // otherwise
        } else {

            // update the hook to be angled away from the player as they swing
            let transform = player.single();
            let direction = hook_t.translation - transform.translation;
            let angle = Vec2::Y.angle_between(direction.truncate());

            hook_t.rotation = Quat::from_rotation_z(angle);
        }
    }
}
