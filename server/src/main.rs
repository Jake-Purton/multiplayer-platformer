use bevy::prelude::*;
use bevy_renet::{
    renet::{
        RenetConnectionConfig, RenetError, RenetServer, ServerAuthentication,
        ServerConfig, DefaultChannel,
    }, RenetServerPlugin,
};

use std::time::SystemTime;
use std::net::UdpSocket;

use serde::{Deserialize, Serialize};

const PROTOCOL_ID: u64 = 7;

#[derive(Debug, Serialize, Deserialize, Component)]
enum ServerMessages {
    PlayerConnected { id: u64 },
    PlayerDisconnected { id: u64 },
    PlayerPosition {id: u64, position: Vec3},
}

#[derive(Debug, Serialize, Deserialize, Component, Resource)]
pub enum ClientMessage {
    PlayerPosition(Vec3),
}

fn new_renet_server() -> RenetServer {
    let server_addr = "127.0.0.1:5000".parse().unwrap();
    let socket = UdpSocket::bind(server_addr).unwrap();
    let connection_config = RenetConnectionConfig::default();
    let server_config = ServerConfig::new(64, PROTOCOL_ID, server_addr, ServerAuthentication::Unsecure);
    let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    RenetServer::new(current_time, server_config, connection_config, socket).unwrap()
}

fn main() {

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugin(RenetServerPlugin::default());
    app.insert_resource(new_renet_server());
    app.add_system(panic_on_error_system);
    app.add_system(server_update_system);

    app.run();
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
                ClientMessage::PlayerPosition(a) => println!("{}", a.x),
            }

        }
    }
}