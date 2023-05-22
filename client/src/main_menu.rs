use bevy::{prelude::*, sprite::collide_aabb::collide, app::AppExit, window::PrimaryWindow};

use crate::{GameState, startup_plugin::{despawn_everything}};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .add_system(setup_menu.in_schedule(OnEnter(GameState::Menu)))
            .add_system(menu_click_system.in_set(OnUpdate(GameState::Menu)))
            .add_system(despawn_everything.in_schedule(OnExit(GameState::Menu)));
    }
}

const TEXT_SIZE: Vec2 = Vec2 { x: 300.0, y: 150.0 };

pub enum MenuAction {
    Exit,
    Start,
    Online,
    Host,
    Join,
    Back,
}

#[derive(Component)]
pub struct MenuItem {
    pub size: Vec2,
    pub action: MenuAction,
}

fn setup_menu (
    mut commands: Commands,
    windows: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>
) {

    let window = windows.get_single().unwrap();

    commands.insert_resource(ClearColor(Color::rgb(1.0, 0.5, 0.0)));
    commands.spawn(Camera2dBundle::default());

    commands.spawn((
        TextBundle {
            text: 
                Text::from_section("Play",             
                TextStyle {
                    font_size: 100.0,
                    color: Color::WHITE,
                    font: asset_server.load("fonts/Rubik-SemiBold.ttf")
                },),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..Default::default()
        }        
        .with_style(Style {
            align_items: AlignItems::Center,
            align_self: AlignSelf::Center,
            align_content: AlignContent::Center,
            justify_content: JustifyContent::Center,
            ..Default::default()
        })        
        .with_text_alignment(TextAlignment::Right),
        MenuItem { size: TEXT_SIZE, action: MenuAction::Start },
    ));
}

pub fn menu_click_system (
    buttons: Res<Input<MouseButton>>, 
    windows: Query<&Window, With<PrimaryWindow>>,
    menu_item: Query<(Entity, &MenuItem, &Transform)>,
    mut game_state: ResMut<NextState<GameState>>,
    mut exit: EventWriter<AppExit>,
    mut commands: Commands,
) {
    if buttons.just_pressed(MouseButton::Left) {
        let window = windows.get_single().unwrap();
        let mut online_pressed = false;

        if let Some(position) = window.cursor_position() {
            let position = Vec3::new(position.x - window.width() / 2.0, position.y - window.height() / 2.0, 0.0);

            for (_, item, transform) in menu_item.iter() {
                if collide(position, Vec2::new(2.0, 2.0), transform.translation, item.size).is_some() {

                    match item.action {
                        MenuAction::Exit => exit.send(AppExit),
                        MenuAction::Start => {
                            game_state.set(GameState::Gameplay)
                        }
                        MenuAction::Online => online_pressed = true,
                        _ => (),
                    }
                }
            }
        }

        if online_pressed {

            for (entity, _, _) in menu_item.iter() {

                commands.entity(entity).despawn();
                println!("despawned")

            }


        }
    }
}