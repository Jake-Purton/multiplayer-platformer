use bevy::{prelude::*, sprite::collide_aabb::collide, app::AppExit, window::PrimaryWindow};

use crate::{GameState, startup_plugin::{despawn_everything, GameTextures}};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .add_system(setup_menu.in_schedule(OnEnter(GameState::Menu)))
            .add_system(menu_click_system.in_set(OnUpdate(GameState::Menu)))
            .add_system(despawn_everything.in_schedule(OnExit(GameState::Menu)));
    }
}

pub enum MenuAction {
    Exit,
    Start,
    Online,
    Host,
    Join,
}

#[derive(Component)]
pub struct MenuItem {
    pub size: Vec2,
    pub action: MenuAction,
}

fn setup_menu (
    mut commands: Commands,
    game_textures: Res<GameTextures>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {

    let window = windows.get_single().unwrap();

    commands.insert_resource(ClearColor(Color::rgb(1.0, 0.5, 0.0)));
    commands.spawn(Camera2dBundle::default());

    commands.spawn(SpriteBundle {
        texture: game_textures.play.clone(),
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, 20.0),
            scale: Vec3::new(1.0, 1.0, 1.0),
            ..Default::default()
        },
        ..Default::default()
    }).insert(MenuItem { size: Vec2::new(500.0, 150.0), action: MenuAction::Start });

    commands.spawn(SpriteBundle {
        texture: game_textures.exit.clone(),
        transform: Transform {
            translation: Vec3::new(0.0, window.height() / 4.0, 20.0),
            scale: Vec3::new(1.0, 1.0, 1.0),
            ..Default::default()
        },
        ..Default::default()
    }).insert(MenuItem { size: Vec2::new(500.0, 150.0), action: MenuAction::Exit });

    commands.spawn(SpriteBundle {
        texture: game_textures.online.clone(),
        transform: Transform {
            translation: Vec3::new(0.0, -(window.height() / 4.0) -200.0, 20.0),
            scale: Vec3::new(1.0, 1.0, 1.0),
            ..Default::default()
        },
        ..Default::default()
    }).insert(MenuItem { size: Vec2::new(500.0, 150.0), action: MenuAction::Online });
}

pub fn menu_click_system (
    buttons: Res<Input<MouseButton>>, 
    windows: Query<&Window, With<PrimaryWindow>>,
    menu_item: Query<(&MenuItem, &Transform)>,
    mut game_state: ResMut<NextState<GameState>>,
    mut exit: EventWriter<AppExit>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        let window = windows.get_single().unwrap();

        if let Some(position) = window.cursor_position() {
            let position = Vec3::new(position.x - window.width() / 2.0, position.y - window.height() / 2.0, 0.0);

            for (item, transform) in menu_item.iter() {
                if collide(position, Vec2::new(2.0, 2.0), transform.translation, item.size).is_some() {

                    match item.action {
                        MenuAction::Exit => exit.send(AppExit),
                        MenuAction::Start => {
                            game_state.set(GameState::Gameplay)
                        }
                        MenuAction::Online => () /* Host or join? */,
                    }
                }
            }
        }
    }
}