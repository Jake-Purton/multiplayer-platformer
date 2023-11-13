use crate::Vec3;
use bevy::{prelude::*, utils::HashMap};

use serde::{Deserialize, Serialize};

// message sent from a server
#[derive(Debug, Serialize, Deserialize, Component)]
pub enum ServerMessageUnreliable {
    PlayerPosition { id: u64, position: Vec3, level: u8 },
    Map { map: Vec<Vec<u8>>, number: u8 },
    MovingWallVel { wall_pos: HashMap<i32, Vec2> },
}

#[derive(Debug, Serialize, Deserialize, Component)]
pub enum ServerMessageReliable {
    PlayerConnected { id: u64 },
    PlayerDisconnected { id: u64 },
    DebugMessage(String),
    NumberOfMaps(u16),
    Pong,
}

// message sent from a client
#[derive(Debug, Serialize, Deserialize, Component, Resource)]
pub enum ClientMessageUnreliable {
    PlayerPosition { pos: Vec3, level: u8 },
    MovingWallVelocity { wall_id: i32, velocity: Vec2 },
}

#[derive(Debug, Serialize, Deserialize, Component, Resource)]
pub enum ClientMessageReliable {
    DebugMessage(String),
    Ping,
}

#[derive(Resource)]
pub struct IpToJoin(String);

// a resource that can be updated by systems to send messages
// #[derive(Debug, Default, Serialize, Deserialize, Component, Resource)]
// pub struct ClientMessages {
//     pub messages: Vec<ClientMessageUnreliable>
// }
