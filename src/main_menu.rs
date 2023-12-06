use std::{fs::File, io::Read};

use bevy::{app::AppExit, prelude::*, window::PrimaryWindow};
use bevy_renet::renet::{RenetClient, RenetServer};

use crate::{
    client::new_renet_client,
    platform::{level_directory, Maps},
    server::new_renet_server,
    startup_plugin::despawn_everything,
    CurrentLevel, GameState, MultiplayerSetting, BACKGROUND_COLOUR,
};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(setup_menu.in_schedule(OnEnter(GameState::Menu)))
            .add_system(menu_click_system.in_set(OnUpdate(GameState::Menu)))
            .add_system(despawn_everything.in_schedule(OnExit(GameState::Menu)))
            .add_system(go_back_to_menu.in_set(OnUpdate(GameState::Gameplay)))
            .add_system(go_back_to_menu.in_set(OnUpdate(GameState::Death)))
            // .add_system(go_back_to_menu.in_set(OnUpdate(GameState::Win)))
            .add_system(go_back_to_menu.in_set(OnUpdate(GameState::Win)));
    }
}

const PLAY: &str = "Singleplayer";
const HOST: &str = "Host";
const JOIN: &str = "Join";
const EXIT: &str = "Exit";

fn go_back_to_menu(
    keys: Res<Input<KeyCode>>,
    mut game_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
    mut cl: ResMut<CurrentLevel>,
    mut setting: ResMut<MultiplayerSetting>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        println!("escape pressed {:?}", game_state.0);
        game_state.set(GameState::Menu);
        cl.level_number = 1;

        match setting.0 {
            HostClient::Host => {
                setting.0 = HostClient::Play;
                commands.remove_resource::<RenetClient>();
                commands.remove_resource::<RenetServer>();
            }
            HostClient::Client => {
                setting.0 = HostClient::Play;
                commands.remove_resource::<RenetClient>();
            }
            HostClient::Play => (),
        }
    }
}

pub enum HostClient {
    Host,
    Client,
    Play,
}

#[derive(Component)]
pub struct Menu;

fn setup_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(ClearColor(BACKGROUND_COLOUR));
    commands.spawn(Camera2dBundle::default());

    commands.spawn((
        TextBundle::from_sections([
            TextSection::new(
                format!("{}\n", PLAY),
                TextStyle {
                    font: asset_server.load("fonts/Rubik-SemiBold.ttf"),
                    font_size: 60.0,
                    color: Color::BLACK,
                },
            ),
            TextSection::new(
                format!("{}\n", HOST),
                TextStyle {
                    font: asset_server.load("fonts/Rubik-SemiBold.ttf"),
                    font_size: 60.0,
                    color: Color::BLACK,
                },
            ),
            TextSection::new(
                format!("{}\n", JOIN),
                TextStyle {
                    font: asset_server.load("fonts/Rubik-SemiBold.ttf"),
                    font_size: 60.0,
                    color: Color::BLACK,
                },
            ),
            TextSection::new(
                format!("{}\n", EXIT),
                TextStyle {
                    font: asset_server.load("fonts/Rubik-SemiBold.ttf"),
                    font_size: 60.0,
                    color: Color::BLACK,
                },
            ),
        ]),
        Menu,
    ));
}

pub fn menu_click_system(
    buttons: Res<Input<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut menu_items: Query<(&mut Text, &CalculatedSize), With<Menu>>,
    mut game_state: ResMut<NextState<GameState>>,
    mut exit: EventWriter<AppExit>,
    mut commands: Commands,
    mut maps: ResMut<Maps>,
) {
    let window = windows.get_single().unwrap();

    if let Some(position) = window.cursor_position() {
        for (mut items, size) in menu_items.iter_mut() {
            for (i, section) in items.sections.iter_mut().enumerate() {
                // to find the position of text: i * size / total_items is the highest y value. lowest is highest + 70
                let highest = window.height() - (i as f32 * size.size.y / 4.0);
                let lowest = highest - 60.0;

                if position.y < highest && position.y > lowest {
                    section.style.color = Color::WHITE;
                    if buttons.just_pressed(MouseButton::Left) {
                        match section.value.trim() {
                            PLAY => {
                                let mut cl = 1;

                                while let Ok(mut file) =
                                    File::open(level_directory(cl, &HostClient::Play))
                                {
                                    let mut contents = String::new();
                                    file.read_to_string(&mut contents).unwrap();

                                    let mut map: Vec<Vec<u8>> = Vec::new();

                                    for line in contents.lines() {
                                        map.push(
                                            line.split_whitespace()
                                                .map(|a| a.parse::<u8>().unwrap())
                                                .collect(),
                                        );
                                    }

                                    map.reverse();

                                    maps.maps.insert(cl, map);

                                    cl += 1;
                                }

                                game_state.set(GameState::Gameplay)
                            }
                            HOST => {
                                println!("host");

                                commands.insert_resource(MultiplayerSetting(HostClient::Host));

                                // public ip
                                let rt = tokio::runtime::Runtime::new().unwrap();
                                let public_ip = rt.block_on(public_ip::addr()).unwrap();
                                commands.insert_resource(new_renet_client(0, public_ip));
                                commands.insert_resource(new_renet_server(public_ip));

                                // local ip
                                // let local_ip = local_ip().unwrap();
                                // commands.insert_resource(new_renet_client(0, local_ip));
                                // commands.insert_resource(new_renet_server(local_ip));

                                let mut cl = 1;

                                while let Ok(mut file) =
                                    File::open(level_directory(cl, &HostClient::Host))
                                {
                                    let mut contents = String::new();
                                    file.read_to_string(&mut contents).unwrap();

                                    let mut map: Vec<Vec<u8>> = Vec::new();

                                    for line in contents.lines() {
                                        map.push(
                                            line.split_whitespace()
                                                .map(|a| a.parse::<u8>().unwrap())
                                                .collect(),
                                        );
                                    }

                                    map.reverse();
                                    maps.maps.insert(cl, map);

                                    println!("{:?}", maps.maps.keys());

                                    cl += 1;
                                }

                                game_state.set(GameState::Gameplay);
                            }
                            EXIT => exit.send(AppExit),
                            JOIN => {
                                game_state.set(GameState::JoinMenu);
                            }
                            _ => println!("What?"),
                        }
                    }
                } else {
                    section.style.color = Color::BLACK;
                }
            }
        }
    }
}
