use std::time::SystemTime;

use bevy::prelude::*;

use bevy_renet::{
    renet::{
        RenetClient, DefaultChannel,
    },
};

use crate::{GameState, messages::{ClientMessageReliable, ServerMessageReliable}};

#[derive(Resource)]
struct PingTime {
    time: SystemTime,
}

pub struct PingPlugin;

impl Plugin for PingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .add_system(setup_pinging.in_schedule(OnEnter(GameState::CheckingConnection)))
            .add_system(listen_for_pong.in_set(OnUpdate(GameState::CheckingConnection)));
            
    }
}

fn setup_pinging (
    mut client: ResMut<RenetClient>, 
    mut commands: Commands,
) {

    commands.spawn(Camera2dBundle::default());

    let message = ClientMessageReliable::Ping;
    let message = bincode::serialize(&message).unwrap();
    client.send_message(DefaultChannel::Reliable, message);
    commands.insert_resource( PingTime { time: SystemTime::now() })

}

fn pinging_text (

) {
    
}

fn listen_for_pong (
    mut client: ResMut<RenetClient>,
    ping_time: Res<PingTime>,
) {

    while let Some(message) = client.receive_message(DefaultChannel::Reliable) {
        let server_message: ServerMessageReliable = bincode::deserialize(&message).unwrap();

        match server_message {
            ServerMessageReliable::Pong => {
                let ping = SystemTime::now().duration_since(ping_time.time).unwrap();
                println!("ping time: {}ms", ping.as_millis())
            },
            _ => (),
        }
    }

    while let Some(_) = client.receive_message(DefaultChannel::Unreliable) {

    }

}