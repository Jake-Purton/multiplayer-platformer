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
    // setup the text and camera and background
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
// puts a cursor bar to indicate that it is a text box
struct BarTimer {
    timer: Timer,
    b: bool,
}

impl BarTimer {
    fn tick(&mut self, dt: Duration) -> bool {
        self.timer.tick(dt);
        // every time the timer finishes (700ms) b is flipped
        if self.timer.just_finished() {
            self.b = !self.b;
        }

        self.b
    }
    fn new() -> Self {
        // every 700 ms the timer finishes
        let timer = Timer::new(Duration::from_millis(700), TimerMode::Repeating);
        Self { timer, b: true }
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
        // tick the bar timer
        let b = timer.tick(dt);
        let mut a = ip_string.0.clone();
        if b {
            // if b is true, put a bar at the end.
            // otherwise; don't.
            a.push('|')
        }
        // update the text to show what you have typed already.
        text.sections[0].value = format!("Server IP: {}", a);
    }
}

fn text_input(mut char_evr: EventReader<ReceivedCharacter>, mut ip_string: ResMut<IPString>) {
    // takes input from the keyboard
    for ev in char_evr.iter() {
        let char = ev.char;

        if char == '\x08' {
            // if it's a backspace
            ip_string.0.pop();
            // pop the previous character
        } else if char.is_numeric() {
            // if it's a number
            ip_string.0.push(char)
            // add the character to the end
        } else if char == '.' {
            // if a . was inputted
            if let Some(a) = ip_string.0.pop() {
                // get the last characyter
                ip_string.0.push(a);
                // if the last character was a number
                if a.is_numeric() {
                    // push the .
                    ip_string.0.push(char)
                }
                // if the last character was not a number don't push the .
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

    // if there are 4 numbers split up by dots
    if split.len() == 4 {
        // parse it to an ip with the default port for a server
        let server_addr = format!("{}:{}", ip, SERVER_PORT).parse();

        // if it parsed properly
        if let Ok(server_addr) = server_addr {
            for i in 0..16 {
                // try up to 16 client ports. (a single machine can have up to 16 clients running)

                // listen to that socket
                let client_addr = SocketAddr::new("0.0.0.0".parse().unwrap(), CLIENT_PORT + i);
                // listen to the socket
                if let Ok(socket) = UdpSocket::bind(client_addr) {
                    // configure the connection with the currwent time, server address etc.
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

                    // if it worked return the client structure
                    return Ok(RenetClient::new(
                        current_time,
                        socket,
                        connection_config,
                        authentication,
                    )
                    .unwrap());
                }
            }
            // if it doesn't work return the corresponding error
            Err(BindError::Client)
        } else {
            // if it doesn't work return the corresponding error
            Err(BindError::Server)
        }
    } else {
        // if it doesn't work return the corresponding error
        Err(BindError::Format)
    }
}
