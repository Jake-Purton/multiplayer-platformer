use crate::{
    moving_block::MovableWall,
    player::{rapier_player_movement, Player},
    GameState, BACKGROUND_COLOUR, GRAVITY_CONSTANT,
};
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_kira_audio::{prelude::*, Audio};
use bevy_rapier2d::prelude::{RapierConfiguration, RigidBody, Velocity};

#[derive(Resource)]
pub struct GameTextures {
    pub player: Handle<Image>,
    pub player1: Handle<Image>,
    pub player2: Handle<Image>,
    pub player3: Handle<Image>,
    pub player4: Handle<Image>,
    pub r_to_respawn: Handle<Image>,
    pub you_win: Handle<Image>,
    pub menu: Handle<Image>,
    pub exit: Handle<Image>,
    pub play: Handle<Image>,
    pub hook: Handle<Image>,
    pub online: Handle<Image>,
}

impl GameTextures {
    pub fn rand_player(&self, id: &u64) -> Handle<Image> {
        match (id % 4) + 1 {
            1 => self.player1.clone(),
            2 => self.player2.clone(),
            3 => self.player3.clone(),
            4 => self.player4.clone(),
            _ => self.player1.clone(),
        }
    }
}

#[derive(Component)]
pub struct PlayerCamera;

pub struct StartupPlugin;

impl Plugin for StartupPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_startup_system(pre_startup.in_base_set(StartupSet::PreStartup))
            .add_system(setup.in_schedule(OnEnter(GameState::Gameplay)))
            .add_system(
                camera_follow_player
                    .after(rapier_player_movement)
                    .in_set(OnUpdate(GameState::Gameplay)),
            )
            .add_system(spinny_cube.in_set(OnUpdate(GameState::Gameplay)))
            .add_system(despawn_everything.in_schedule(OnExit(GameState::Gameplay)))
            .add_system(toggle_mute);
    }
}

fn pre_startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
    mut rapier_config: ResMut<RapierConfiguration>,
) {
    commands.insert_resource(GameTextures {
        player: asset_server.load("images/fella.png"),
        player1: asset_server.load("images/fella-1.png"),
        player2: asset_server.load("images/fella-2.png"),
        player3: asset_server.load("images/fella-3.png"),
        player4: asset_server.load("images/fella-4.png"),
        r_to_respawn: asset_server.load("death-messages/respawn.png"),
        you_win: asset_server.load("death-messages/you-win.png"),
        menu: asset_server.load("death-messages/menu.png"),
        exit: asset_server.load("death-messages/exit.png"),
        play: asset_server.load("death-messages/play.png"),
        hook: asset_server.load("images/hook.png"),
        online: asset_server.load("death-messages/online.png"),
    });

    let music = asset_server.load("music/new_bossa.wav");
    audio.play(music).looped().with_volume(0.2);
    audio.pause();

    rapier_config.gravity = GRAVITY_CONSTANT;
}

fn setup(mut commands: Commands) {
    commands.insert_resource(ClearColor(BACKGROUND_COLOUR));
    commands
        .spawn(Camera2dBundle::default())
        .insert(PlayerCamera)
        .insert(Velocity {
            linvel: Vec2::ZERO,
            ..Default::default()
        })
        .insert(RigidBody::Dynamic);
}

fn toggle_mute(audio: Res<Audio>, keys: Res<Input<KeyCode>>) {
    if keys.just_pressed(KeyCode::M) {
        if audio.is_playing_sound() {
            audio.pause();
        } else {
            audio.resume();
        }
    }
}

fn camera_follow_player(
    mut camera: Query<(&Transform, &mut Velocity), (With<PlayerCamera>, Without<Player>)>,
    player: Query<&Transform, With<Player>>,
) {
    let (camera, mut vel) = camera.single_mut();
    let player = player.single();

    let velocity = (player.translation - camera.translation).truncate() * 2.0;
    vel.linvel = (velocity + vel.linvel) * 0.7;
}

pub fn despawn_everything(
    query: Query<Entity, (Without<PrimaryWindow>, Without<PlayerCamera>)>,
    mut commands: Commands,
    camera: Query<Entity, With<PlayerCamera>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }

    for cam in camera.iter() {
        commands.entity(cam).despawn()
    }
}

fn spinny_cube(mut cubes: Query<(&Velocity, &mut Sprite), With<MovableWall>>) {
    for (vel, mut cube) in cubes.iter_mut() {
        if vel.angvel.abs() > 1.0 {
            let x = vel.angvel / vel.angvel.abs();
            cube.color += Color::rgba(0.02 * x, -0.02 * x, 0.01 * x, 0.0);
        }
    }
}
