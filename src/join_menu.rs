use std::{
    net::{SocketAddr, UdpSocket},
    time::{Duration, SystemTime},
};

use bevy::prelude::*;

use crate::{
    client::PROTOCOL_ID,
    main_menu::{HostClient, Menu},
    server::{CLIENT_PORT, SERVER_PORT},
    startup_plugin::despawn_everything,
    GameState, MultiplayerSetting, BACKGROUND_COLOUR,
};

use bevy_renet::renet::{ClientAuthentication, RenetClient, RenetConnectionConfig};

pub enum BindError {
    Client,
    Server,
    Format,
}

pub struct JoinMenuPlugin;

impl Plugin for JoinMenuPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(IPString(String::new()))
            .insert_resource(BarTimer::new())
            .add_system(setup_join_menu.in_schedule(OnEnter(GameState::JoinMenu)))
            .add_system(join_input_ip.in_set(OnUpdate(GameState::JoinMenu)))
            .add_system(update_text.in_set(OnUpdate(GameState::JoinMenu)))
            .add_system(despawn_everything.in_schedule(OnExit(GameState::JoinMenu)))
            .add_system(text_input.in_set(OnUpdate(GameState::JoinMenu)));
    }
}

#[derive(Resource)]
pub struct IPString(String);

fn setup_join_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(ClearColor(BACKGROUND_COLOUR));
    commands.spawn(Camera2dBundle::default());

    commands.spawn((
        TextBundle::from_section(
            "Server IP: ",
            TextStyle {
                font: asset_server.load("fonts/Rubik-SemiBold.ttf"),
                font_size: 60.0,
                color: Color::BLACK,
            },
        ),
        Menu,
    ));
}

#[derive(Resource)]
struct BarTimer {
    timer: Timer,
    b: bool,
}

impl BarTimer {
    fn new() -> Self {
        let timer = Timer::new(Duration::from_millis(700), TimerMode::Repeating);
        Self { timer, b: true }
    }

    fn tick(&mut self, dt: Duration) -> bool {
        self.timer.tick(dt);
        if self.timer.just_finished() {
            self.b = !self.b;
        }

        self.b
    }
}

fn update_text(
    ip_string: Res<IPString>,
    mut text: Query<&mut Text, With<Menu>>,
    mut timer: ResMut<BarTimer>,
    time: Res<Time>,
) {
    for mut text in &mut text {
        let dt = time.delta();
        let b = timer.tick(dt);
        let mut a = ip_string.0.clone();
        if b {
            a.push('|')
        }

        text.sections[0].value = format!("Server IP: {}", a);
    }
}

fn text_input(mut char_evr: EventReader<ReceivedCharacter>, mut ip_string: ResMut<IPString>) {
    for ev in char_evr.iter() {
        let char = ev.char;

        if let Some(a) = ip_string.0.pop() {
            if a != '|' {
                ip_string.0.push(a);
            }
        }

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

fn join_input_ip(
    mut commands: Commands,

    // the user's input (the ip they typed)
    ip: Res<IPString>,

    mut game_state: ResMut<NextState<GameState>>,

    // keyboard input
    keys: Res<Input<KeyCode>>,
) {
    // if they press enter
    if keys.just_pressed(KeyCode::Return) {
        // connect to the ip the user input
        let client = renet_client(ip.0.trim());
        
        match client {
            // ip is ok
            Ok(client) => {
                // insert the client resource
                commands.insert_resource(client);
                // change the setting to client
                commands.insert_resource(MultiplayerSetting(HostClient::Client));
                // go to the next state
                game_state.set(GameState::CheckingConnection);
            }
            // there is an error
            Err(a) => match a {
                BindError::Client => println!("client error"),
                BindError::Server => println!("server error"),
                BindError::Format => println!("format error"),
            },
        }
    
    // escape goes back to the menu
    } else if keys.just_pressed(KeyCode::Escape) {
        // reset everything
        println!("escape pressed");
        game_state.set(GameState::Menu);
        commands.insert_resource(MultiplayerSetting(HostClient::Play));
        commands.remove_resource::<RenetClient>();
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
                    let current_time = SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap();
                    let client_id = current_time.as_millis() as u64;
                    let authentication = ClientAuthentication::Unsecure {
                        client_id,
                        protocol_id: PROTOCOL_ID,
                        server_addr,
                        user_data: None,
                    };

                    return Ok(RenetClient::new(
                        current_time,
                        socket,
                        connection_config,
                        authentication,
                    )
                    .unwrap());
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
