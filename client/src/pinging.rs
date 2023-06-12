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

pub struct PingPlugin;

impl Plugin for PingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
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
) {
    let time = SystemTime::now().duration_since(time.time).unwrap();
    let mut pinging = "Pinging".to_string();
    let dots = time.as_millis() / 300 % 4;

    if time.as_millis() >= TIMEOUT_DURATION {
        println!("Ping timed out, returning to menu");
        game_state.set(GameState::Menu);
        commands.insert_resource(MultiplayerSetting ( HostClient::Play ));
        commands.remove_resource::<RenetClient>();
    }

    for mut text in &mut text {
        for _ in 0..dots {
            pinging.push('.')
        }
        text.sections[0].value = pinging.to_owned();
    }
}

fn listen_for_pong (
    mut client: ResMut<RenetClient>,
    mut game_state: ResMut<NextState<GameState>>,
    ping_time: Res<PingTime>,
) {

    while let Some(message) = client.receive_message(DefaultChannel::Reliable) {
        let server_message: ServerMessageReliable = bincode::deserialize(&message).unwrap();

        match server_message {
            ServerMessageReliable::Pong => {
                let ping = SystemTime::now().duration_since(ping_time.time).unwrap();
                println!("ping time: {}ms", ping.as_millis());
                game_state.set(GameState::Gameplay)
            },
            ServerMessageReliable::DebugMessage(string) => println!("recieved debug message (pinging.rs) {}", string),
            _ => (),
        }
    }

    while client.receive_message(DefaultChannel::Unreliable).is_some() {

        // draining the messages so they don't build up

    }

}