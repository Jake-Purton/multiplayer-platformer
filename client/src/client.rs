use bevy::prelude::*;
use bevy_rapier2d::prelude::{RigidBody, Collider};
use bevy_renet::{
    renet::{
        ClientAuthentication, RenetClient, RenetConnectionConfig, DefaultChannel,
    },
    RenetClientPlugin,
};
use serde::{Serialize, Deserialize};

use std::net::SocketAddr;
use local_ip_address::local_ip;

// messages that need to be sent: 
// player position
// player level change
// moving block position / being moved  

use std::{net::UdpSocket, time::SystemTime, collections::HashMap};

use crate::{player::Player, GameState, FELLA_SPRITE_SIZE, startup_plugin::GameTextures, main_menu::HostClient, MultiplayerSetting};

pub struct MyClientPlugin;

impl Plugin for MyClientPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugin(RenetClientPlugin::default())
            .init_resource::<ClientMessages>()
            .insert_resource(UserIdMap(HashMap::new()))
            .add_system(client_update_system.in_set(OnUpdate(GameState::Gameplay)).run_if(run_if_client))
            .add_system(respawn_other_players.in_schedule(OnEnter(GameState::Gameplay)).run_if(run_if_client))
            .insert_resource(new_renet_client())
            .add_system(client_send_input.run_if(run_if_client));

    }
}

fn run_if_client (
    host_or_join: Res<MultiplayerSetting>,
) -> bool {
    match host_or_join.0 {
        HostClient::Client => true,
        _ => false,
    }
}

#[derive(Component)]
pub struct AnotherPlayer (u64);

#[derive(Resource)]
pub struct UserIdMap(HashMap<u64, Vec3>);

pub const PROTOCOL_ID: u64 = 8;

#[derive(Debug, Default, Serialize, Deserialize, Component, Resource)]
pub struct ClientMessages {
    messages: Vec<ClientMessage>
}

#[derive(Debug, Serialize, Deserialize, Component, Resource)]
pub enum ClientMessage {
    PlayerPosition(Vec3),
}

#[derive(Debug, Serialize, Deserialize, Component)]
pub enum ServerMessage {
    PlayerConnected { id: u64 },
    PlayerDisconnected { id: u64 },
    PlayerPosition {id: u64, position: Vec3},
}

pub fn new_renet_client() -> RenetClient {
    let server_addr = "192.168.1.235:8080".parse().unwrap();
    let client_addr = SocketAddr::new(local_ip().unwrap(), 5000);
    let socket = UdpSocket::bind(client_addr).unwrap();
    let connection_config = RenetConnectionConfig::default();
    let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    let client_id = current_time.as_millis() as u64;
    let authentication = ClientAuthentication::Unsecure {
        client_id,
        protocol_id: PROTOCOL_ID,
        server_addr,
        user_data: None,
    };
    RenetClient::new(current_time, socket, connection_config, authentication).unwrap()
}

fn client_send_input(client_messages: Res<ClientMessages>, mut client: ResMut<RenetClient>, player_position: Query<&Transform, With<Player>>) {
    for message in &client_messages.messages {
        
        let input_message = bincode::serialize(&message).unwrap();

        client.send_message(DefaultChannel::Unreliable, input_message);
    }

    for pos in player_position.iter() {

        let message = ClientMessage::PlayerPosition(pos.translation);
        let input_message = bincode::serialize(&message).unwrap();

        client.send_message(DefaultChannel::Unreliable, input_message);

    }
}

fn client_update_system(
    mut client: ResMut<RenetClient>,
    game_textures: Res<GameTextures>,
    mut map: ResMut<UserIdMap>,
    mut commands: Commands,
    mut players: Query<(&AnotherPlayer, &mut Transform)>,
) {

    while let Some(message) = client.receive_message(DefaultChannel::Unreliable) {
        let server_message: ServerMessage = bincode::deserialize(&message).unwrap();

        match server_message {
            ServerMessage::PlayerPosition{ id: a, position: pos} => {
                if map.0.insert(a, pos).is_none() {

                    // spawn the entity and label it a
                    commands
                        .spawn(SpriteBundle {
                            texture: game_textures.player.clone(),
                            sprite: Sprite {
                                custom_size: Some(FELLA_SPRITE_SIZE),
                                ..Default::default()
                            },
                            ..Default::default()
                        })
                        .insert(Collider::cuboid(FELLA_SPRITE_SIZE.x / 2.0, FELLA_SPRITE_SIZE.y / 2.0 ))
                        .insert(RigidBody::Fixed)
                        .insert(AnotherPlayer(a));
                }
            },
            _ => println!("another message")
        }
    }

    for (playerid, mut transform) in players.iter_mut() {

        if let Some(a) = map.0.get(&playerid.0) { 
            transform.translation = a.clone() 
        };

    }
}

fn respawn_other_players (
    mut commands: Commands,
    game_textures: Res<GameTextures>,
    players: Res<UserIdMap>
) {
    
    for (player, pos) in &players.0 {

        commands
        .spawn(SpriteBundle {
            texture: game_textures.player.clone(),
            sprite: Sprite {
                custom_size: Some(FELLA_SPRITE_SIZE),
                ..Default::default()
            },
            transform: Transform::from_translation(*pos),
            ..Default::default()
        })
        .insert(Collider::cuboid(FELLA_SPRITE_SIZE.x / 2.0, FELLA_SPRITE_SIZE.y / 2.0 ))
        .insert(RigidBody::Fixed)
        .insert(AnotherPlayer(*player));

    }

}