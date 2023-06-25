use std::time::SystemTime;

use bevy::prelude::*;

use bevy_renet::{
    renet::{
        RenetClient, DefaultChannel,
    },
};

// how long to wait while pinging (miliseconds)
const TIMEOUT_DURATION: u128 = 5000;

use crate::{GameState, messages::{ClientMessageReliable, ServerMessageReliable}, main_menu::{Menu, HostClient}, startup_plugin::despawn_everything, MultiplayerSetting};

#[derive(Resource)]
struct PingTime {
    time: SystemTime,
}


enum PingStage {
    Pinging,
    RequestingMaps,
}

#[derive(Resource)]
struct NumberOfMaps(Option<u16>);

#[derive(Resource)]
struct PingThing(PingStage);

pub struct PingPlugin;

impl Plugin for PingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .insert_resource(PingThing(PingStage::Pinging))
            .add_system(setup_pinging.in_schedule(OnEnter(GameState::CheckingConnection)))
            .add_system(listen_for_pong.in_set(OnUpdate(GameState::CheckingConnection)))
            .add_system(pinging_text.in_set(OnUpdate(GameState::CheckingConnection)))
            .add_system(despawn_everything.in_schedule(OnExit(GameState::CheckingConnection)));
            
    }
}

fn setup_pinging (
    mut client: ResMut<RenetClient>, 
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {

    commands.insert_resource(PingThing(PingStage::Pinging));
    commands.spawn(Camera2dBundle::default());

    let message = ClientMessageReliable::Ping;
    let message = bincode::serialize(&message).unwrap();
    client.send_message(DefaultChannel::Reliable, message);
    commands.insert_resource( PingTime { time: SystemTime::now() });

    commands.spawn((
        TextBundle::from_section(
            "Pinging",
            TextStyle {
                font: asset_server.load("fonts/Rubik-SemiBold.ttf"),
                font_size: 60.0,
                color: Color::BLACK,
            },
        ),
        Menu
    ));

}

fn pinging_text (
    mut text: Query<&mut Text, With<Menu>>,
    mut game_state: ResMut<NextState<GameState>>,
    time: Res<PingTime>,  
    mut commands: Commands, 
    ping_thing: Res<PingThing>,
) {
    let time = SystemTime::now().duration_since(time.time).unwrap();

    let mut pinging = "".to_string();

    match ping_thing.0 {
        PingStage::Pinging => {

            pinging.push_str("Pinging");

            if time.as_millis() >= TIMEOUT_DURATION {
                println!("Ping timed out, returning to menu");
                game_state.set(GameState::Menu);
                commands.insert_resource(MultiplayerSetting ( HostClient::Play ));
                commands.remove_resource::<RenetClient>();
            }
        
        },
        PingStage::RequestingMaps => {
            pinging.push_str("Getting maps");

            // adds 20 seconds to the timer
            if time.as_millis() >= TIMEOUT_DURATION + 20000 {
                println!("Ping timed out, returning to menu");
                game_state.set(GameState::Menu);
                commands.insert_resource(MultiplayerSetting ( HostClient::Play ));
                commands.remove_resource::<RenetClient>();
            }
        
        },
    }

    let dots = time.as_millis() / 300 % 4;

    
    for mut text in &mut text {
        for _ in 0..dots {
            pinging.push('.')
        }
        text.sections[0].value = pinging.to_owned();
    }
}

fn listen_for_pong (
    mut client: ResMut<RenetClient>,
    ping_time: Res<PingTime>,
    mut commands: Commands,
) {

    while let Some(message) = client.receive_message(DefaultChannel::Reliable) {
        let server_message: ServerMessageReliable = bincode::deserialize(&message).unwrap();

        match server_message {
            ServerMessageReliable::Pong => {
                let ping = SystemTime::now().duration_since(ping_time.time).unwrap();
                println!("ping time: {}ms", ping.as_millis());
                
                println!("setting pingthing to request maps");
                commands.insert_resource(PingThing(PingStage::RequestingMaps))

                
            },
            ServerMessageReliable::DebugMessage(string) => println!("recieved debug message (pinging.rs) {}", string),
            ServerMessageReliable::NumberOfMaps(total) => {
                commands.insert_resource(NumberOfMaps(Some(total)))
            },
            _ => (),
        }
    }

    while client.receive_message(DefaultChannel::Unreliable).is_some() {

        // if number of maps is some - then look for maps ig

    }

}