use crate::{
    player::{rapier_player_movement, Player},
    GameState, BACKGROUND_COLOUR, GRAVITY_CONSTANT, client::UserIdMap,
};
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_rapier2d::prelude::{RapierConfiguration, RigidBody, Velocity};

#[derive(Resource)]
pub struct GameTextures {
    // all of the assets
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
    // the code that gives other players a random sprite based on their id
    // their sprite will be the same colour for as long as they are playing on that session.
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
            .add_system(despawn_everything.in_schedule(OnExit(GameState::Gameplay)));
    }
}

fn pre_startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut rapier_config: ResMut<RapierConfiguration>,
) {
    // this loads all of the textures on startup so they do not need to
    // be loaded at any other time. It is faster to pre-load textures in
    // this way
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

    // configuring the gravitational constant
    rapier_config.gravity = GRAVITY_CONSTANT;
}

fn setup(mut commands: Commands) {
    // set the background colour
    commands.insert_resource(ClearColor(BACKGROUND_COLOUR));
    // spawn the camera. This same camera is used throughout the game
    commands
        .spawn(Camera2dBundle::default())
        .insert(PlayerCamera)
        // it has a velocity component so i can move it smoothly to follow the player
        .insert(Velocity {
            linvel: Vec2::ZERO,
            ..Default::default()
        })
        // rigid body component so bevy rapier can
        // move it for me
        .insert(RigidBody::Dynamic);
}

// makes the camera follow the player
fn camera_follow_player(
    mut camera: Query<(&Transform, &mut Velocity), (With<PlayerCamera>, Without<Player>)>,
    player: Query<&Transform, With<Player>>,
) {
    // get the camera and player
    let (camera, mut vel) = camera.single_mut();
    let player = player.single();

    // the vector going from the camera to the player multiplied by 2
    let velocity = (player.translation - camera.translation).truncate() * 2.0;
    // add this velocity to the previous velocity but dampen it.
    // if the multiple was 1 it would continue to move like a pendulum
    // it would go past the player and then back and then past etc.
    vel.linvel = (velocity + vel.linvel) * 0.7;
}

pub fn despawn_everything(
    // get all entities except for the window and camera
    query: Query<Entity, (Without<PrimaryWindow>, Without<PlayerCamera>)>,
    mut commands: Commands,

    // get the camera entity
    camera: Query<Entity, With<PlayerCamera>>,

    // get the hashmap of spawned and non-spawned players
    mut player_map: ResMut<UserIdMap>,
) {
    // despawn entities
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }

    // all players are despawned so set their booleans to false
    for player in player_map.0.iter_mut() {
        player.1.2 = false
    }

    // despawn the camera
    for cam in camera.iter() {
        commands.entity(cam).despawn()
    }
}
