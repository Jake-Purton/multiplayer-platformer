use bevy::prelude::*;
use bevy_rapier2d::prelude::{Collider, RigidBody};
use bevy_renet::{
    renet::{ClientAuthentication, DefaultChannel, RenetClient, RenetConnectionConfig},
    RenetClientPlugin,
};

use std::net::{IpAddr, SocketAddr};

// messages that need to be sent:
// player position and level
// moving block position / being moved

use std::{collections::HashMap, net::UdpSocket, time::SystemTime};

use crate::{
    main_menu::HostClient,
    messages::{ClientMessageUnreliable, ServerMessageUnreliable},
    moving_block::BlockMap,
    player::Player,
    server::{CLIENT_PORT, SERVER_PORT},
    startup_plugin::GameTextures,
    CurrentLevel, GameState, MultiplayerSetting, FELLA_SPRITE_SIZE,
};

pub struct MyClientPlugin;

impl Plugin for MyClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(RenetClientPlugin::default())
            // .init_resource::<ClientMessages>()
            .insert_resource(UserIdMap(HashMap::new()))
            .add_system(
                client_update_system
                    .in_set(OnUpdate(GameState::Gameplay))
                    .run_if(run_if_client),
            )
            .add_system(client_send_input.run_if(run_if_client));
    }
}

fn run_if_client(host_or_join: Res<MultiplayerSetting>) -> bool {
    matches!(host_or_join.0, HostClient::Client | HostClient::Host)
}

#[derive(Component)]
pub struct AnotherPlayer {
    pub id: u64,
}

#[derive(Resource)]
pub struct UserIdMap(pub HashMap<u64, (Vec3, u8)>);

pub const PROTOCOL_ID: u64 = 6;

pub fn new_renet_client(number: u16, ip: IpAddr) -> RenetClient {
    let server_addr = SocketAddr::new(ip, SERVER_PORT);
    let client_addr = SocketAddr::new("0.0.0.0".parse().unwrap(), CLIENT_PORT + number);

    if let Ok(socket) = UdpSocket::bind(client_addr) {
        let connection_config = RenetConnectionConfig::default();
        let current_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
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
    level: Res<CurrentLevel>,
) {
    // for message in &client_messages.messages {

    //     let input_message = bincode::serialize(&message).unwrap();

    //     client.send_message(DefaultChannel::Unreliable, input_message);
    // }

    for pos in player_position.iter() {
        let message = ClientMessageUnreliable::PlayerPosition {
            pos: pos.translation,
            level: level.level_number,
        };
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
    mut block_map: ResMut<BlockMap>,
) {
    while let Some(message) = client.receive_message(DefaultChannel::Unreliable) {
        let server_message: ServerMessageUnreliable = bincode::deserialize(&message).unwrap();

        match server_message {
            ServerMessageUnreliable::PlayerPosition {
                id,
                position: pos,
                level,
            } => {
                if level == current_level.level_number {
                    let exists_in_map = map.0.insert(id, (pos, level)).is_some();

                    if !exists_in_map {
                        // spawn the entity and label it a
                        commands
                            .spawn(SpriteBundle {
                                texture: game_textures.rand_player(&id),
                                sprite: Sprite {
                                    custom_size: Some(FELLA_SPRITE_SIZE),
                                    ..Default::default()
                                },
                                transform: Transform::from_translation(pos),
                                ..Default::default()
                            })
                            .insert(Collider::cuboid(
                                FELLA_SPRITE_SIZE.x / 2.0,
                                FELLA_SPRITE_SIZE.y / 2.0,
                            ))
                            .insert(RigidBody::Fixed)
                            .insert(AnotherPlayer { id });
                    }
                } else {
                    let exists_in_map = map.0.insert(id, (pos, level)).is_some();

                    if !exists_in_map {
                        map.0.remove(&id);
                    }
                }
            }
            ServerMessageUnreliable::Map { map: _, number: _ } => {
                println!("server just sent me a map even though im gaming rn")
            }
            ServerMessageUnreliable::WallPos {
                level,
                client_id,
                wall_id,
                pos,
            } => {
                // make a system that adds the blocks to a hashmap
                // hashmap key is the level so the client can easily find the right blocks yk
                if let Some(a) = block_map.blocks.get_mut(&level) {
                    let mut t = false;

                    for i in &mut *a {
                        if i.0 == client_id && i.1 == wall_id {
                            i.2 = pos;
                            t = true;
                            break;
                        }
                    }

                    if !t {
                        a.push((client_id, wall_id, pos))
                    }
                } else {
                    let vec = vec![(client_id, wall_id, pos)];
                    block_map.blocks.insert(level, vec);
                }

                // println!("{:?}", block_map.blocks);
            }
        }
    }

    for (entity, playerid, mut transform) in players.iter_mut() {
        if let Some((pos, level)) = map.0.get(&playerid.id) {
            if *level == current_level.level_number {
                transform.translation = *pos;
            } else {
                commands.entity(entity).despawn();
                map.0.remove(&playerid.id);
                println!("here, {:?}", map.0);
            }
        };
    }
}
