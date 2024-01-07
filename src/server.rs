use std::{
    net::{IpAddr, SocketAddr, UdpSocket},
    time::SystemTime,
};

use bevy::prelude::*;
use bevy_renet::{
    renet::{
        DefaultChannel, RenetConnectionConfig, RenetError, RenetServer, ServerAuthentication,
        ServerConfig, ServerEvent,
    },
    RenetServerPlugin,
};
use local_ip_address::local_ip;

use crate::{
    client::PROTOCOL_ID,
    messages::{
        ClientMessageReliable, ClientMessageUnreliable, ServerMessageReliable,
        ServerMessageUnreliable,
    },
    platform::Maps, run_if::run_if_host,
};

// the default ports for the client and server
pub const SERVER_PORT: u16 = 42069;
pub const CLIENT_PORT: u16 = 5001;

pub struct MyServerPlugin;

impl Plugin for MyServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(RenetServerPlugin::default())
            .add_system(panic_on_error_system.run_if(run_if_host))
            .add_system(server_update_system.run_if(run_if_host));
    }
}



pub fn new_renet_server(public_ip: IpAddr) -> RenetServer {
    // sets up the binding to the public ip address
    let inbound_server_addr = SocketAddr::new(local_ip().unwrap(), SERVER_PORT);
    let socket = UdpSocket::bind(inbound_server_addr).unwrap();

    // Public hosting, requires port forwarding on your router
    let server_addr = SocketAddr::new(public_ip, SERVER_PORT);
    // sets up the server
    let connection_config = RenetConnectionConfig::default();
    let server_config =
        ServerConfig::new(64, PROTOCOL_ID, server_addr, ServerAuthentication::Unsecure);
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    // returns the server
    RenetServer::new(current_time, server_config, connection_config, socket).unwrap()
}

fn panic_on_error_system(mut renet_error: EventReader<RenetError>) {
    // if there is an error, it crashes and prints the error
    // this is for development purposes
    // I have not seen a crash in the final version
    for e in renet_error.iter() {
        panic!("{}", e);
    }
}

fn server_update_system(mut server: ResMut<RenetServer>, maps: Res<Maps>) {
    for client_id in server.clients_id().into_iter() {
        while let Some(message) = server.receive_message(client_id, DefaultChannel::Unreliable) {
            // recieve all the messages

            // deserialise the messages to be understood (they are sent as bytes and then parsed back to data structures)
            let client_message: ClientMessageUnreliable = bincode::deserialize(&message).unwrap();

            // find out what type of message it is
            match client_message {
                ClientMessageUnreliable::PlayerPosition { level, pos } => {
                    // send the position to all clients except the one that told us
                    let message = ServerMessageUnreliable::PlayerPosition {
                        id: client_id,
                        position: pos,
                        level,
                    };
                    // broadcasts a message to all clients except one
                    server.broadcast_message_except(
                        // the id of the client rhat doesn't get sent the message
                        client_id,
                        DefaultChannel::Unreliable,
                        // serialise the message into 1s and 0s
                        bincode::serialize(&message).unwrap(),
                    )
                }

                // when the wall position message is recieved by the server, the server
                // broadcasts a message to all of the other clients, with cliend_id,
                // level, position and wall_id.
                ClientMessageUnreliable::WallPos {
                    level,
                    wall_id,
                    pos,
                } => {
                    // send the wall positions to all clients except the one that sent it to us
                    let message = ServerMessageUnreliable::WallPos {
                        client_id,
                        pos,
                        wall_id,
                        level,
                    };
                    server.broadcast_message_except(
                        client_id,
                        DefaultChannel::Unreliable,
                        bincode::serialize(&message).unwrap(),
                    )
                }
            }
        }

        while let Some(message) = server.receive_message(client_id, DefaultChannel::Reliable) {
            let client_message: ClientMessageReliable = bincode::deserialize(&message).unwrap();

            match client_message {
                ClientMessageReliable::DebugMessage(string) => {
                    println!("server recieved message: {}", string)
                }

                ClientMessageReliable::Ping => {
                    // if a ping is recieved
                    println!("ping recieved from {}", client_id);

                    // send a pong
                    let message = ServerMessageReliable::Pong;
                    server.send_message(
                        client_id,
                        DefaultChannel::Reliable,
                        bincode::serialize(&message).unwrap(),
                    );

                    // send the number of maps
                    let message = ServerMessageReliable::NumberOfMaps(maps.maps.len() as u16);
                    server.send_message(
                        client_id,
                        DefaultChannel::Reliable,
                        bincode::serialize(&message).unwrap(),
                    );

                    // send the maps to the client
                    for (i, a) in &maps.maps {
                        let message = ServerMessageUnreliable::Map {
                            map: a.clone(),
                            number: *i,
                        };
                        server.send_message(
                            client_id,
                            DefaultChannel::Unreliable,
                            bincode::serialize(&message).unwrap(),
                        );
                    }
                }
            }
        }
    }

    while let Some(event) = server.get_event() {
        match event {
            // server tells us when a client has connected
            ServerEvent::ClientConnected(client_id, _) => {
                println!("Client {client_id} connected");
            }
            // server tells us when a client has disconnected
            ServerEvent::ClientDisconnected(client_id) => {
                println!("Client {client_id} disconnected: BECAUSE");
                let message = ServerMessageReliable::PlayerDisconnected { id: client_id };
                server.broadcast_message(
                    DefaultChannel::Reliable,
                    bincode::serialize(&message).unwrap(),
                )
            }
        }
    }
}
