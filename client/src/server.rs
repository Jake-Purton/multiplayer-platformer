use std::{net::UdpSocket, time::SystemTime};

use bevy::prelude::*;
use bevy_renet::{renet::{RenetError, RenetServer, DefaultChannel, RenetConnectionConfig, ServerConfig, ServerAuthentication}, RenetServerPlugin};

use crate::client::{ClientMessage, ServerMessage, PROTOCOL_ID};

// this version of the server bounces the messages but doesn't send them to itself
// would also like to send messages when a user disconnects for the player to be despawned.

pub struct MyServerPlugin;

impl Plugin for MyServerPlugin {
    fn build(&self, app: &mut App) {

        app
            .add_plugins(MinimalPlugins)
            .add_plugin(RenetServerPlugin::default())
            .insert_resource(new_renet_server())
            .add_system(panic_on_error_system)
            .add_system(server_update_system);

    }
}

fn new_renet_server() -> RenetServer {
    let server_addr = "127.0.0.1:5000".parse().unwrap();
    let socket = UdpSocket::bind(server_addr).unwrap();
    let connection_config = RenetConnectionConfig::default();
    let server_config = ServerConfig::new(64, PROTOCOL_ID, server_addr, ServerAuthentication::Unsecure);
    let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    RenetServer::new(current_time, server_config, connection_config, socket).unwrap()
}

// If any error is found we just panic
fn panic_on_error_system(mut renet_error: EventReader<RenetError>) {
    for e in renet_error.iter() {
        panic!("{}", e);
    }
}

fn server_update_system(
    mut server: ResMut<RenetServer>,
) {

    for client_id in server.clients_id().into_iter() {
        while let Some(message) = server.receive_message(client_id, DefaultChannel::Unreliable) {
            let client_message: ClientMessage = bincode::deserialize(&message).unwrap();

            match client_message {
                ClientMessage::PlayerPosition(vec) => {
                    let message = ServerMessage::PlayerPosition { id: client_id, position: vec };
                    server.broadcast_message_except(client_id, DefaultChannel::Unreliable, bincode::serialize(&message).unwrap())
                },
            }
        }
    }
}