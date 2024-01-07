use crate::Vec3;
use bevy::prelude::*;

use serde::{Deserialize, Serialize};

// These enums are well named so I'm not commenting each individual branch. 

// message sent from a server through the unreliable 
// channel (faster but it is possible that packets can be lost)
#[derive(Debug, Serialize, Deserialize, Component)]
pub enum ServerMessageUnreliable {
    PlayerPosition {
        id: u64,
        position: Vec3,
        level: u8,
    },
    Map {
        map: Vec<Vec<u8>>,
        number: u8,
    },
    WallPos {
        client_id: u64,
        wall_id: i32,
        pos: Vec2,
        level: u8,
    },
}

// messages sent from server through the reliable channel
// slower but dropped packets are re-sent 
#[derive(Debug, Serialize, Deserialize, Component)]
pub enum ServerMessageReliable {
    PlayerConnected { id: u64 },
    PlayerDisconnected { id: u64 },
    DebugMessage(String),
    NumberOfMaps(u16),
    Pong,
}

// message sent from a client through unreliable channel
#[derive(Debug, Serialize, Deserialize, Component, Resource)]
pub enum ClientMessageUnreliable {
    PlayerPosition { pos: Vec3, level: u8 },
    WallPos { level: u8, wall_id: i32, pos: Vec2 },
}

// message sent from a client through the reliable channel
#[derive(Debug, Serialize, Deserialize, Component, Resource)]
pub enum ClientMessageReliable {
    DebugMessage(String),
    Ping,
}

#[derive(Resource)]
pub struct IpToJoin(String);