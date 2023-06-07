use std::io::stdin;

use bevy::{prelude::*, app::AppExit, window::PrimaryWindow};
use local_ip_address::local_ip;

use crate::{GameState, startup_plugin::despawn_everything, MultiplayerSetting, client::new_renet_client, server::new_renet_server};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .insert_resource(IPString (String::new()))
            .add_system(setup_menu.in_schedule(OnEnter(GameState::Menu)))
            .add_system(menu_click_system.in_set(OnUpdate(GameState::Menu)))
            .add_system(despawn_everything.in_schedule(OnExit(GameState::Menu)))
            .add_system(text_input.in_set(OnUpdate(GameState::Menu)));
            
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

#[derive(Resource)]
pub struct IPString (String);

fn text_input(
    mut char_evr: EventReader<ReceivedCharacter>,
    mut ip_string: ResMut<IPString>
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

    println!("{}", ip_string.0)

}

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

                            PLAY => game_state.set(GameState::Gameplay),
                            HOST => {
                                println!("host");

                                commands.insert_resource(MultiplayerSetting(HostClient::Host));
                                commands.insert_resource(new_renet_client(0, &local_ip().unwrap().to_string()));
                                commands.insert_resource(new_renet_server());
                                game_state.set(GameState::Gameplay);
                            },
                            EXIT => exit.send(AppExit),
                            JOIN => {
                                println!("Join, input the server ip: ");
                                let mut ip = String::new();

                                stdin().read_line(&mut ip).unwrap();
                                commands.insert_resource(new_renet_client(0, ip.trim()));
                                commands.insert_resource(MultiplayerSetting(HostClient::Client));
                                game_state.set(GameState::Gameplay);
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