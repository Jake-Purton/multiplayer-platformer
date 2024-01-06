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
            .insert_resource(UserIdMap(HashMap::new()))
            // add the client system to run when in client mode
            .add_system(
                client_update_system
                    .in_set(OnUpdate(GameState::Gameplay))
                    .run_if(run_if_client),
            )
            // add the update player system to run when in client mode
            .add_system(
                update_players
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
// a hashmap where the key is the userid, and the value is a tuple of the
// player's position, the level that player is on and wether it needs to be spawned
pub struct UserIdMap(pub HashMap<u64, (Vec3, u8, bool)>);

pub const PROTOCOL_ID: u64 = 6;

pub fn new_renet_client(number: u16, ip: IpAddr) -> RenetClient {
    // the ip and port of the server
    let server_addr = SocketAddr::new(ip, SERVER_PORT);
    // the ip and port of the client
    let client_addr = SocketAddr::new("0.0.0.0".parse().unwrap(), CLIENT_PORT + number);
    // if the socket is valid
    if let Ok(socket) = UdpSocket::bind(client_addr) {
        let connection_config = RenetConnectionConfig::default();
        let current_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        // generate a unique client id
        let client_id = current_time.as_millis() as u64;
        // my server doesn't use authentication
        let authentication = ClientAuthentication::Unsecure {
            client_id,
            protocol_id: PROTOCOL_ID,
            server_addr,
            user_data: None,
        };
        // generates the new client and returns it
        RenetClient::new(current_time, socket, connection_config, authentication).unwrap()
    } else {
        // recursively increases the number as the port may already be being used
        // this can occur if there are multiple instances of the game running at once
        new_renet_client(number + 1, ip)
    }
}


// send the player's position and the level they are in to the server
fn client_send_input(
    mut client: ResMut<RenetClient>,
    player_position: Query<&Transform, With<Player>>,
    level: Res<CurrentLevel>,
) {
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
    mut player_map: ResMut<UserIdMap>,
    mut block_map: ResMut<BlockMap>,
) {
    // iterate over every message 
    while let Some(message) = client.receive_message(DefaultChannel::Unreliable) {
        let server_message: ServerMessageUnreliable = bincode::deserialize(&message).unwrap();

        match server_message {
            // player position
            ServerMessageUnreliable::PlayerPosition {
                id,
                position: pos,
                level,
            } => {
                // update the positions and level numbers if it exists
                if let Some((position, level_number, _)) = player_map.0.get_mut(&id) {
                    *position = pos;
                    *level_number = level;
                } else {
                    // insert the information - pos - level - has not been spawned yet
                    player_map.0.insert(id, (pos, level, false));
                }
            }
            ServerMessageUnreliable::Map { map: _, number: _ } => {
                println!("server just sent me a map even though im gaming rn")
            }

            // When the client recieves a message from the server with the wall position,
            // it adds it to the hashmap
            ServerMessageUnreliable::WallPos {
                level,
                client_id,
                wall_id,
                pos,
            } => {
                // hashmap key is the level so the client can easily find the right blocks
                // if there are some blocks in this level
                if let Some(existing_blocks) = block_map.blocks.get_mut(&level) {
                    let mut t = false;
                    // iterate over these blocks and see if they match the message recieved by the server
                    for i in &mut *existing_blocks {
                        // if the block matches
                        if i.0 == client_id && i.1 == wall_id {
                            // update position
                            i.2 = pos;
                            t = true;
                            break;
                        }
                    }
                    // if none match, create another block
                    if !t {
                        existing_blocks.push((client_id, wall_id, pos))
                    }
                } else {
                    let vec = vec![(client_id, wall_id, pos)];
                    block_map.blocks.insert(level, vec);
                }

                // println!("{:?}", block_map.blocks);
            }
        }
    }
}

fn update_players (
    mut player_map: ResMut<UserIdMap>,
    gt: Res<GameTextures>,
    cl: Res<CurrentLevel>,
    mut commands: Commands,
    mut players: Query<(Entity, &AnotherPlayer, &mut Transform)>,
) {

    // iterate over all the spawned players
    for (entity, ap, mut transform) in players.iter_mut() {
        // get the info sent by the server
        let player_info = player_map.0.get_mut(&ap.id).unwrap();
        // if the player is on the same level as the client
        if player_info.1 == cl.level_number {
            // update its position
            transform.translation = player_info.0;
        } else {
            // despawn it
            commands.entity(entity).despawn();
            // it has no longer been spawned so set this to false
            player_info.2 = false;
        }
    }

    for (id, value) in player_map.0.iter_mut() {
        // iterate over only the ones which havent been spawned
        if value.2 {
            continue;
        }
        // ignore players on different levels
        if value.1 != cl.level_number {
            continue;
        }
        // if the player is on this level and hasn't been spawned yet
        // spawn the player
        commands
            .spawn(SpriteBundle {
                texture: gt.rand_player(&id),
                sprite: Sprite {
                    custom_size: Some(FELLA_SPRITE_SIZE),
                    ..Default::default()
                },
                transform: Transform::from_translation(value.0),
                ..Default::default()
            })
            .insert(Collider::cuboid(
                FELLA_SPRITE_SIZE.x / 2.0,
                FELLA_SPRITE_SIZE.y / 2.0,
            ))
            .insert(RigidBody::Fixed)
            .insert(AnotherPlayer { id: *id });

        // it has now been spawned so set this to true
        value.2 = true;
    }
}