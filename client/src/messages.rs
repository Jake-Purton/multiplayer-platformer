use bevy::prelude::*;
use crate::Vec3;

use serde::{Deserialize, Serialize};

// message sent from a server
#[derive(Debug, Serialize, Deserialize, Component)]
pub enum ServerMessageUnreliable {
    PlayerPosition {id: u64, position: Vec3},
}

#[derive(Debug, Serialize, Deserialize, Component)]
pub enum ServerMessageReliable {
    PlayerConnected { id: u64 },
    PlayerDisconnected { id: u64 },
    DebugMessage(String),
    Pong,
}

// message sent from a client
#[derive(Debug, Serialize, Deserialize, Component, Resource)]
pub enum ClientMessageUnreliable {
    PlayerPosition(Vec3),
}

#[derive(Debug, Serialize, Deserialize, Component, Resource)]
pub enum ClientMessageReliable {
    DebugMessage(String),
    Ping,
}

// a resource that can be updated by systems to send messages
// #[derive(Debug, Default, Serialize, Deserialize, Component, Resource)]
// pub struct ClientMessages {
//     pub messages: Vec<ClientMessageUnreliable>
// }