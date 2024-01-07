use std::time::SystemTime;

use bevy::prelude::*;

use bevy_renet::renet::{DefaultChannel, RenetClient};

use crate::{
    main_menu::{HostClient, Menu},
    messages::{ClientMessageReliable, ServerMessageReliable, ServerMessageUnreliable},
    platform::Maps,
    startup_plugin::despawn_everything,
    GameState, MultiplayerSetting,
};

// how long to wait while pinging (miliseconds)
const TIMEOUT_DURATION: u128 = 5000;

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
        app.insert_resource(PingThing(PingStage::Pinging))
            // initialise resource to none
            .insert_resource(NumberOfMaps(None))
            // add the systems
            .add_system(setup_pinging.in_schedule(OnEnter(GameState::CheckingConnection)))
            .add_system(listen_for_pong.in_set(OnUpdate(GameState::CheckingConnection)))
            .add_system(pinging_text.in_set(OnUpdate(GameState::CheckingConnection)))
            .add_system(despawn_everything.in_schedule(OnExit(GameState::CheckingConnection)));
    }
}

fn setup_pinging(
    mut client: ResMut<RenetClient>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    // a resource to keep track of where we are. pinging or requesting maps
    commands.insert_resource(PingThing(PingStage::Pinging));
    commands.spawn(Camera2dBundle::default());

    // send the ping
    let message = ClientMessageReliable::Ping;
    let message = bincode::serialize(&message).unwrap();
    client.send_message(DefaultChannel::Reliable, message);

    // start the ping timer
    commands.insert_resource(PingTime {
        time: SystemTime::now(),
    });

    // spawn some text that says pinging
    commands.spawn((
        TextBundle::from_section(
            "Pinging",
            TextStyle {
                font: asset_server.load("fonts/Rubik-SemiBold.ttf"),
                font_size: 60.0,
                color: Color::BLACK,
            },
        ),
        Menu,
    ));
}

fn pinging_text(
    mut text: Query<&mut Text, With<Menu>>,
    mut game_state: ResMut<NextState<GameState>>,
    time: Res<PingTime>,
    mut commands: Commands,
    ping_thing: Res<PingThing>,
) {
    // the time since the ping was sent
    let time = SystemTime::now().duration_since(time.time).unwrap();
    let mut pinging = "".to_string();
    match ping_thing.0 {
        // if still waiting for the ping
        PingStage::Pinging => {
            pinging.push_str("Pinging");

            // ping timeout
            if time.as_millis() >= TIMEOUT_DURATION {
                println!("Ping timed out, returning to menu");
                // resets to menu
                game_state.set(GameState::Menu);
                commands.insert_resource(MultiplayerSetting(HostClient::Play));
                commands.remove_resource::<RenetClient>();
            }
        }
        // if waiting for the maps
        PingStage::RequestingMaps => {
            pinging.push_str("Getting maps");

            // adds 20 seconds to the timeout duration (takes longer to timeout)
            if time.as_millis() >= TIMEOUT_DURATION + 20000 {
                // if it timed out return to the menu
                println!("Ping timed out, returning to menu");
                game_state.set(GameState::Menu);
                commands.insert_resource(MultiplayerSetting(HostClient::Play));
                commands.remove_resource::<RenetClient>();
            }
        }
    }
    // adds dots with this effect in a loop:
    // pinging
    // pinging.
    // pinging..
    // pinging...
    let dots = time.as_millis() / 300 % 4;
    for mut text in &mut text {
        for _ in 0..dots {
            pinging.push('.')
        }
        text.sections[0].value = pinging.to_owned();
    }
}

fn listen_for_pong(
    mut client: ResMut<RenetClient>,
    ping_time: Res<PingTime>,
    mut commands: Commands,
    mut num_maps: ResMut<NumberOfMaps>,
    mut game_state: ResMut<NextState<GameState>>,
    mut maps: ResMut<Maps>,
) {
    // recieve all messages
    while let Some(message) = client.receive_message(DefaultChannel::Reliable) {
        let server_message: ServerMessageReliable = bincode::deserialize(&message).unwrap();

        match server_message {
            // if its a pong
            ServerMessageReliable::Pong => {
                let ping = SystemTime::now().duration_since(ping_time.time).unwrap();
                println!("ping time: {}ms", ping.as_millis());

                println!("setting pingthing to request maps");
                commands.insert_resource(PingThing(PingStage::RequestingMaps))
            }
            // for debugging
            ServerMessageReliable::DebugMessage(string) => {
                println!("recieved debug message (pinging.rs) {}", string)
            }
            // the number of maps so we know we have them all
            ServerMessageReliable::NumberOfMaps(total) => num_maps.0 = Some(total),
            _ => (),
        }
    }

    // pops messages off the stack until the stack is empty
    while let Some(message) = client.receive_message(DefaultChannel::Unreliable) {
        let server_message: ServerMessageUnreliable = bincode::deserialize(&message).unwrap();

        // recieve messages from the server
        if let ServerMessageUnreliable::Map { map, number } = server_message {
            // read messages containing maps
            
            // if we have recieved the message for the total number of maps
            if let Some(num) = num_maps.0 {
                // print which map we just recieved
                println!("Just got sent map number {}", number);
                // add it to the hashmap of maps
                maps.maps.insert(number, map);

                // if we have all of the maps go to the gameplay state
                if maps.maps.len() == num as usize {
                    println!("got all the maps, going to gameplay");
                    game_state.set(GameState::Gameplay);
                }
            }
        }
    }
}
