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

    // spawns the menu
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

    // if cursor is in the window
    if let Some(position) = window.cursor_position() {
        // get the items and the size, there is only one of these so i could have used single_mut()
        for (mut items, size) in menu_items.iter_mut() {
            // iterate over the sections of texy
            for (i, section) in items.sections.iter_mut().enumerate() {
                // to find the position of text: i * size / total_items is the top y value. bottom is top - 60
                let top = window.height() - (i as f32 * size.size.y / 4.0);
                let bottom = top - 60.0;

                // if cursor is hovering over that text
                if position.y < top && position.y > bottom {
                    //turn the text white
                    section.style.color = Color::WHITE;

                    // if the user clicks on the text
                    if buttons.just_pressed(MouseButton::Left) {
                        // match the menu's action
                        match section.value.trim() {
                            PLAY => {
                                // for debugging
                                println!("play");
                                // while there are contiguous numbered map files
                                read_and_parse_files(1, &mut maps, HostClient::Play);

                                // sets the gamestate to gameplay
                                game_state.set(GameState::Gameplay)
                            }
                            HOST => {
                                println!("host");

                                // tells the systems that we are the host
                                commands.insert_resource(MultiplayerSetting(HostClient::Host));

                                // gets the public ip and starts a renet server
                                let rt = tokio::runtime::Runtime::new().unwrap();
                                let public_ip = rt.block_on(public_ip::addr()).unwrap();
                                commands.insert_resource(new_renet_client(0, public_ip));
                                commands.insert_resource(new_renet_server(public_ip));

                                // does the same as the singleplayer button but uses the levels
                                // in another directiory to let the player change which ones
                                // they are playing with
                                read_and_parse_files(1, &mut maps, HostClient::Host);

                                game_state.set(GameState::Gameplay);
                            }
                            //exits the game
                            EXIT => exit.send(AppExit),
                            // sends us to the join menu
                            JOIN => {
                                game_state.set(GameState::JoinMenu);
                            }
                            // if the item pressed doesn't exist it does nothing
                            _ => (),
                        }
                    }
                } else {
                    // makes the text black if you dont hover over it
                    section.style.color = Color::BLACK;
                }
            }
        }
    }
}

fn turn_file_into_map(contents: String) -> Vec<Vec<u8>> {
    let mut map: Vec<Vec<u8>> = Vec::new();

    // iterate over it and parse it to a map
    for line in contents.lines() {
        map.push(
            line.split_whitespace()
                .map(|a| a.parse::<u8>().unwrap())
                .collect(),
        );
    }

    map.reverse();

    map
}

fn read_and_parse_files(mut cl: u8, maps: &mut ResMut<Maps>, hc: HostClient) {
    // while there is a file
    while let Ok(mut file) =
        // level_directory() function returns the directory for a specific level number and gamemode
        File::open(level_directory(cl, &hc))
    {
        // read the contents of the file
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        // turn the string into a map
        let map = turn_file_into_map(contents);

        // insert the map to the maps resource to be used
        maps.maps.insert(cl, map);
        println!("map {cl}");
        cl += 1;
    }
}
