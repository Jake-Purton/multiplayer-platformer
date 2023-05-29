use std::{net::{UdpSocket, SocketAddr}, time::SystemTime};

use bevy::{prelude::*};
use bevy_renet::{renet::{RenetError, RenetServer, DefaultChannel, RenetConnectionConfig, ServerConfig, ServerAuthentication, ServerEvent}, RenetServerPlugin};
use local_ip_address::local_ip;

use crate::{client::{ClientMessage, PROTOCOL_ID}, main_menu::HostClient, MultiplayerSetting};

// this version of the server bounces the messages but doesn't send them to itself
// would also like to send messages when a user disconnects for the player to be despawned.

pub const SERVER_PORT: u16 = 5000;
pub const SERVER_ADDR: &str = "192.168.1.235";
pub const CLIENT_PORT: u16 = 5001;

pub struct MyServerPlugin;

impl Plugin for MyServerPlugin {
    fn build(&self, app: &mut App) {

        app
            .add_plugin(RenetServerPlugin::default())
            .add_system(panic_on_error_system.run_if(run_if_host))
            .add_system(server_update_system.run_if(run_if_host));
    }
}

fn run_if_host (
    host: Res<MultiplayerSetting>
) -> bool {
    match host.0 {
        HostClient::Host => true,
        _ => false,
    }
}

pub fn new_renet_server() -> RenetServer {
    let server_addr = SocketAddr::new(local_ip().unwrap(), SERVER_PORT);
    let socket = UdpSocket::bind(server_addr).unwrap();
    let connection_config = RenetConnectionConfig::default();
    let server_config = ServerConfig::new(64, PROTOCOL_ID, server_addr, ServerAuthentication::Unsecure);
    let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    RenetServer::new(current_time, server_config, connection_config, socket).unwrap()
}

// If any error is found we just panic
// ^^ OVERRIDDEN > ;)
fn panic_on_error_system(mut renet_error: EventReader<RenetError>) {
    for e in renet_error.iter() {
        // panic!("{}", e);
        println!("{}", e);
    }
}

fn server_update_system(
    mut server: ResMut<RenetServer>,
) {

    for client_id in server.clients_id().into_iter() {
        while let Some(message) = server.receive_message(client_id, DefaultChannel::Unreliable) {
            let client_message: ClientMessage = bincode::deserialize(&message).unwrap();

            match client_message {
                // ClientMessage::PlayerPosition(vec) => {
                //     let message = ServerMessage::PlayerPosition { id: client_id, position: vec };
                //     server.broadcast_message_except(client_id, DefaultChannel::Unreliable, bincode::serialize(&message).unwrap())
                // },
                _ => (),
            }
        }
    }

    while let Some(event) = server.get_event() {
        match event {
            ServerEvent::ClientConnected ( client_id, _) => {
                println!("Client {client_id} connected");
            }
            ServerEvent::ClientDisconnected( client_id ) => {
                println!("Client {client_id} disconnected: BECAUSE");
            }
        }
    }

}