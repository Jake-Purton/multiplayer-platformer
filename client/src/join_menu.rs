use std::{net::{SocketAddr, UdpSocket}, time::SystemTime};

use bevy::prelude::*;

use crate::{GameState, server::{SERVER_PORT, CLIENT_PORT}, client::PROTOCOL_ID, main_menu::{Menu, HostClient}, MultiplayerSetting, startup_plugin::despawn_everything};

use bevy_renet::renet::{
        ClientAuthentication, RenetClient, RenetConnectionConfig,
    };

pub enum BindError {
    Client,
    Server,
    Format,
}

pub struct JoinMenuPlugin;

impl Plugin for JoinMenuPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .insert_resource(IPString (String::new()))
            .add_system(setup_join_menu.in_schedule(OnEnter(GameState::JoinMenu)))
            .add_system(join_input_ip.in_set(OnUpdate(GameState::JoinMenu)))
            .add_system(update_text.in_set(OnUpdate(GameState::JoinMenu)))
            .add_system(despawn_everything.in_schedule(OnExit(GameState::JoinMenu)))
            .add_system(text_input.in_set(OnUpdate(GameState::JoinMenu)));
            
    }
}

#[derive(Resource)]
pub struct IPString (String);

fn setup_join_menu (
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {

    commands.insert_resource(ClearColor(Color::rgb(1.0, 0.5, 0.0)));
    commands.spawn(Camera2dBundle::default());

    commands.spawn((
        TextBundle::from_section(
            "",
            TextStyle {
                font: asset_server.load("fonts/Rubik-SemiBold.ttf"),
                font_size: 60.0,
                color: Color::BLACK,
            },
        ),
        Menu
    ));
}

fn update_text (
    ip_string: ResMut<IPString>,
    mut text: Query<&mut Text, With<Menu>>,
) {
    for mut text in &mut text {
        text.sections[0].value = ip_string.0.clone();
    }
}

fn text_input(
    mut char_evr: EventReader<ReceivedCharacter>,
    mut ip_string: ResMut<IPString>,
) {
    for ev in char_evr.iter() {
        let char = ev.char;

        if char == '\x08' {
            ip_string.0.pop();
        } else if char.is_numeric() {
            ip_string.0.push(char)
        } else if char == '.' {
            if let Some(a) = ip_string.0.pop() {
                ip_string.0.push(a);
                if a.is_numeric() {
                    ip_string.0.push(char)
                }
            }
        }
    }
}

fn join_input_ip (
    mut commands: Commands,
    ip: Res<IPString>,
    mut game_state: ResMut<NextState<GameState>>,
    keys: Res<Input<KeyCode>>,
) {

    if keys.just_pressed(KeyCode::Return) {

        println!("here");

        let client = renet_client(ip.0.trim());

        match client {
            Ok(client) => {
                commands.insert_resource(client);
                println!("yeah baby");

                // try and ping the server here

                commands.insert_resource(MultiplayerSetting(HostClient::Client));
                game_state.set(GameState::Gameplay);
            },
            Err(a) => {
                match a {
                    BindError::Client => println!("client error"),
                    BindError::Server => println!("server error"),
                    BindError::Format => println!("format error"),
                }
            }
        }
    }
}

fn renet_client(ip: &str) -> Result<RenetClient, BindError> {

    let split: Vec<_> = ip.split('.').collect();
    if split.len() == 4 {

        let mut server_addr = ip.parse();
        
        if !split.contains(&":") {
            
            server_addr = format!("{}:{}", ip, SERVER_PORT).parse();

        }

        if let Ok(server_addr) = server_addr {

            for i in 0..16 {

                let client_addr = SocketAddr::new("0.0.0.0".parse().unwrap(), CLIENT_PORT + i);
            
                if let Ok(socket) = UdpSocket::bind(client_addr) {
            
                    let connection_config = RenetConnectionConfig::default();
                    let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
                    let client_id = current_time.as_millis() as u64;
                    let authentication = ClientAuthentication::Unsecure {
                        client_id,
                        protocol_id: PROTOCOL_ID,
                        server_addr,
                        user_data: None,
                    };
                
                    return Ok(RenetClient::new(current_time, socket, connection_config, authentication).unwrap());
            
                }
            }

            Err(BindError::Client)


        } else {

            Err(BindError::Server)

        }
    } else {
        Err(BindError::Format)
    }

}