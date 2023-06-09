use bevy::prelude::*;
use bevy_rapier2d::prelude::{RigidBody, Collider};
use bevy_renet::{
    renet::{
        ClientAuthentication, RenetClient, RenetConnectionConfig, DefaultChannel,
    },
    RenetClientPlugin,
};

use std::net::{SocketAddr, IpAddr};

// messages that need to be sent: 
// player position and level
// moving block position / being moved  

use std::{net::UdpSocket, time::SystemTime, collections::HashMap};

use crate::{player::Player, GameState, FELLA_SPRITE_SIZE, startup_plugin::GameTextures, main_menu::HostClient, MultiplayerSetting, server::{CLIENT_PORT, SERVER_PORT}, messages::{ClientMessageUnreliable, ServerMessageUnreliable}, CurrentLevel};

pub struct MyClientPlugin;

impl Plugin for MyClientPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugin(RenetClientPlugin::default())
            // .init_resource::<ClientMessages>()
            .insert_resource(UserIdMap(HashMap::new()))
            .add_system(client_update_system.in_set(OnUpdate(GameState::Gameplay)).run_if(run_if_client))
            .add_system(client_send_input.run_if(run_if_client));

    }
}

fn run_if_client (
    host_or_join: Res<MultiplayerSetting>,
) -> bool {
    matches!(host_or_join.0, HostClient::Client | HostClient::Host)
}

#[derive(Component)]
pub struct AnotherPlayer {
    pub id: u64,
}

#[derive(Resource)]
pub struct UserIdMap(HashMap<u64, (Vec3, u8)>);

pub const PROTOCOL_ID: u64 = 8;

pub fn new_renet_client(number: u16, ip: IpAddr) -> RenetClient {
    let server_addr = SocketAddr::new(ip , SERVER_PORT);
    println!("{}", server_addr);
    // let server_addr = "109.145.5.231:5000".parse().unwrap();
    let client_addr = SocketAddr::new("0.0.0.0".parse().unwrap(), CLIENT_PORT + number);

    if let Ok(socket) = UdpSocket::bind(client_addr) {

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

    } else {

        new_renet_client(number + 1, ip)

    }
}

fn client_send_input(
    // client_messages: Res<ClientMessages>, 
    mut client: ResMut<RenetClient>, 
    player_position: Query<&Transform, With<Player>>,
    level: Res<CurrentLevel>
) {
    // for message in &client_messages.messages {
        
    //     let input_message = bincode::serialize(&message).unwrap();

    //     client.send_message(DefaultChannel::Unreliable, input_message);
    // }

    for pos in player_position.iter() {

        let message = ClientMessageUnreliable::PlayerPosition{pos: pos.translation, level: level.level_number};
        let input_message = bincode::serialize(&message).unwrap();

        client.send_message(DefaultChannel::Unreliable, input_message);

    }
}

fn client_update_system(
    mut client: ResMut<RenetClient>,
    game_textures: Res<GameTextures>,
    mut map: ResMut<UserIdMap>,
    mut commands: Commands,
    mut players: Query<(Entity, &AnotherPlayer, &mut Transform)>,
    current_level: Res<CurrentLevel>,
) {

    while let Some(message) = client.receive_message(DefaultChannel::Unreliable) {
        let server_message: ServerMessageUnreliable = bincode::deserialize(&message).unwrap();

        match server_message {
            ServerMessageUnreliable::PlayerPosition{ id, position: pos, level} => {

                let exists_in_map = map.0.insert(id, (pos, level)).is_some();

                if level == current_level.level_number && !exists_in_map {

                    // spawn the entity and label it a
                    commands
                        .spawn(SpriteBundle {
                            texture: game_textures.player.clone(),
                            sprite: Sprite {
                                custom_size: Some(FELLA_SPRITE_SIZE),
                                ..Default::default()
                            },
                            transform: Transform::from_translation(pos),
                            ..Default::default()
                        })
                        .insert(Collider::cuboid(FELLA_SPRITE_SIZE.x / 2.0, FELLA_SPRITE_SIZE.y / 2.0 ))
                        .insert(RigidBody::Fixed)
                        .insert(AnotherPlayer { id });

                }
            },
        }
    }

    for (entity, playerid, mut transform) in players.iter_mut() {

        if let Some((pos, level)) = map.0.get(&playerid.id)  {

            if *level == current_level.level_number {

                transform.translation = *pos;

            } else {
                commands.entity(entity).despawn();
            }
        };
    }
}