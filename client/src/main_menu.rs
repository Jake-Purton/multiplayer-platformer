use std::{fs::File, io::Read};

use bevy::{prelude::*, app::AppExit, window::PrimaryWindow};
use local_ip_address::local_ip;

use crate::{GameState, startup_plugin::despawn_everything, MultiplayerSetting, client::new_renet_client, server::new_renet_server, platform::{level_directory, Maps}, CurrentLevel};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .add_system(setup_menu.in_schedule(OnEnter(GameState::Menu)))
            .add_system(menu_click_system.in_set(OnUpdate(GameState::Menu)))
            .add_system(despawn_everything.in_schedule(OnExit(GameState::Menu)));
            
    }
}


const PLAY: &str = "Play";
const HOST: &str = "Host";
const JOIN: &str = "Join";
const EXIT: &str = "Exit";

pub enum HostClient {
    Host,
    Client,
    Play,
}

#[derive(Component)]
pub struct Menu;

fn setup_menu (
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {

    commands.insert_resource(ClearColor(Color::rgb(1.0, 0.5, 0.0)));
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
            }),
            TextSection::new(
                format!("{}\n", JOIN),
                TextStyle {
                font: asset_server.load("fonts/Rubik-SemiBold.ttf"),
                font_size: 60.0,
                color: Color::BLACK,
            }),
            TextSection::new(
                format!("{}\n", EXIT),
                TextStyle {
                font: asset_server.load("fonts/Rubik-SemiBold.ttf"),
                font_size: 60.0,
                color: Color::BLACK,
            }),
        ]),
        Menu,
    ));

}

pub fn menu_click_system (
    buttons: Res<Input<MouseButton>>, 
    windows: Query<&Window, With<PrimaryWindow>>,
    mut menu_items: Query<(&mut Text, &CalculatedSize), With<Menu>>,
    mut game_state: ResMut<NextState<GameState>>,
    mut exit: EventWriter<AppExit>,
    mut commands: Commands,
    current_level: Res<CurrentLevel>,
) {
    let window = windows.get_single().unwrap();
    
    if let Some(position) = window.cursor_position() {
        
        for (mut items, size) in menu_items.iter_mut() {
            
            for (i, mut section) in items.sections.iter_mut().enumerate() {
                
                // to find the position of text: i * size / total_items is the highest y value. lowest is highest + 70
                let highest = window.height() - (i as f32 * size.size.y / 4.0);
                let lowest = highest - 60.0;
                
                if position.y < highest && position.y > lowest {
                    section.style.color = Color::WHITE;
                    if buttons.just_pressed(MouseButton::Left) {

                        match section.value.trim() {

                            PLAY => {
                                let mut cl = current_level.level_number;
                                let mut maps: Vec<Vec<Vec<u8>>> = Vec::new();

                                while let Ok(mut file) = File::open(level_directory(cl, &HostClient::Play)) {
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
                                    maps.push(map);
                                    cl += 1;
                                }

                                commands.insert_resource(Maps { maps });

                                game_state.set(GameState::Gameplay)
                            },
                            HOST => {
                                println!("host");

                                commands.insert_resource(MultiplayerSetting(HostClient::Host));

                                // public ip
                                // let rt = tokio::runtime::Runtime::new().unwrap();
                                // let public_ip = rt.block_on(public_ip::addr()).unwrap();
                                // commands.insert_resource(new_renet_client(0, public_ip));
                                // commands.insert_resource(new_renet_server(public_ip));

                                // local ip
                                let local_ip = local_ip().unwrap();
                                commands.insert_resource(new_renet_client(0, local_ip));
                                commands.insert_resource(new_renet_server(local_ip));

                                // load maps into ram so that they can be sent and used
                                // load the maps as hashmaps where the key is "level-<level_number>" and the value is the string

                                let mut cl = current_level.level_number;
                                let mut maps: Vec<Vec<Vec<u8>>> = Vec::new();

                                while let Ok(mut file) = File::open(level_directory(cl, &HostClient::Host)) {
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
                                    maps.push(map);
                                    cl += 1;
                                }
                                
                                commands.insert_resource(Maps { maps });
                                game_state.set(GameState::Gameplay);
                            },
                            EXIT => exit.send(AppExit),
                            JOIN => {
                                game_state.set(GameState::JoinMenu);
                            },
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