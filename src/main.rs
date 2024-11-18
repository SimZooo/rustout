use bevy::{input::{mouse::MouseButtonInput, ButtonState}, prelude::*, sprite::MaterialMesh2dBundle, window::PrimaryWindow, DefaultPlugins};

mod collision;
use collision::*;
mod components;
use components::*;
mod resources;
use resources::*;

const BACKGROUND_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
const PADDLE_WIDTH: f32 = 80.0;
const PADDLE_HEIGHT: f32 = 20.0;
const PADDLE_SPEED: f32 = 300.0;
const PADDLE_Y: f32 = -150.0;

const MENU_ELEMENT_SIZE: Vec2 = Vec2::new(200.0, 55.0);
const ROWS: i32 = 4;
const COLS: i32 = 4;
const GAP: i32 = 1;

fn main() {
    let wallx = (WINDOW_SIZE.x - (GAP as f32* COLS as f32)) / COLS as f32;
    let wally = (WINDOW_SIZE.y / 2.0 - (GAP as f32 * ROWS as f32)) / ROWS as f32;
    App::new()
        .add_plugins(DefaultPlugins.set(
            WindowPlugin{
                primary_window: Some(Window {
                    title: String::from("Breakout"),
                    resolution: (WINDOW_SIZE.x, WINDOW_SIZE.y).into(),
                    enabled_buttons: bevy::window::EnabledButtons {
                        maximize: false,
                        ..default()
                    },
                    ..default()
                }),
                ..default()
            }
        ))
        .init_state::<AppState>()
        // Menu systems
        .add_systems(Startup, setup_mainmenu.run_if(in_state(AppState::Menu)))
        .add_systems(Update, (menu_input).run_if(in_state(AppState::Menu)))
        .add_systems(OnExit(AppState::Menu), cleanup_menu)
        // InGame systems
        .add_systems(OnTransition {exited: AppState::Menu, entered: AppState::InGame}, setup)
        .add_systems(OnExit(AppState::InGame), (cleanup_game, setup_mainmenu))
        .add_systems(Update, 
            (move_pedal, update_ball_position.after(on_collision), 
            on_collision.after(check_collisions), check_collisions, 
            animate_destroyed_walls)
            .run_if(in_state(AppState::InGame)))
        .add_event::<CollisionEvent>()
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(WallCount(0))
        .insert_resource(WallSize(Vec2::new(wallx, wally)))
        .run();
}

fn setup_mainmenu(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>, asset_server: Res<AssetServer>) {
    let collision_sound = asset_server.load("sounds/breakout_collision.ogg");
    let background_music = asset_server.load("sounds/bgmusic.ogg");

    commands.insert_resource(CollisionSound(collision_sound));
    commands.insert_resource(BackgroundMusic(background_music));

    commands.spawn((
        Camera2dBundle {
            ..default()
        },
        MainMenuCamera
    ));
    commands.spawn((
        MaterialMesh2dBundle {
            mesh:  meshes.add(Rectangle::default()).into(),
            material: materials.add(Color::srgb(0.3, 0.3, 0.7)),
            transform: Transform::from_xyz(0.0, 30.0, 0.0).with_scale(MENU_ELEMENT_SIZE.extend(1.0)),
            ..default()
        },
        MenuElement
    ));

    commands.spawn((
        MaterialMesh2dBundle {
            mesh:  meshes.add(Rectangle::default()).into(),
            material: materials.add(Color::srgb(0.3, 0.3, 0.7)),
            transform: Transform::from_xyz(0.0, -45.0, 0.0).with_scale(MENU_ELEMENT_SIZE.extend(1.0)),
            ..default()
        },
        MenuElement
    ));

    commands.spawn((
        MaterialMesh2dBundle {
            mesh:  meshes.add(Rectangle::default()).into(),
            material: materials.add(Color::srgb(0.3, 0.3, 0.7)),
            transform: Transform::from_xyz(0.0, -140.0, 0.0).with_scale(MENU_ELEMENT_SIZE.extend(1.0)),
            ..default()
        },
        MenuElement
    ));

    let style = TextStyle {
        font_size: 60.0,
        color: Color::WHITE,
        ..default()
    };

    let play= Text::from_section("Play", style.clone()).with_justify(JustifyText::Center);
    let quit = Text::from_section("Quit", style.clone()).with_justify(JustifyText::Center);
    let style = TextStyle {
        font_size: 50.0,
        color: Color::WHITE,
        ..default()
    };
    let settings = Text::from_section("Settings", style.clone()).with_justify(JustifyText::Center);

    commands.spawn((
        Text2dBundle {
            text: play,
            transform: Transform::from_translation(Vec3::new(0.0,30.0,1.0)),
            ..default()
        },
        MenuElement
    ));

    commands.spawn((
        Text2dBundle {
            text: settings,
            transform: Transform::from_translation(Vec3::new(0.0,-45.0,1.0)),
            ..default()
        },
        MenuElement
    ));

    commands.spawn((
        Text2dBundle {
            text: quit,
            transform: Transform::from_translation(Vec3::new(0.0,-140.0,1.0)),
            ..default()
        },
        MenuElement
    ));
}


fn menu_input(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>, 
    mut state: ResMut<NextState<AppState>>, 
    window: Query<&Window, With<PrimaryWindow>>, 
    mut mousebutton_evr: EventReader<MouseButtonInput>,
    sound: Res<CollisionSound>,
) {
    if keys.pressed(KeyCode::Enter) {
        state.set(AppState::InGame)
    }

    for event in mousebutton_evr.read() {
        if event.state == ButtonState::Pressed && event.button == MouseButton::Left {
            if let Some(position) = window.single().cursor_position() {
                if check_ui_collision(position, MENU_ELEMENT_SIZE, Vec2::new(0.0, 30.0)) {
                commands.spawn(AudioBundle {
                    source: sound.clone(),
                    settings: PlaybackSettings::DESPAWN
                });
                    state.set(AppState::InGame);
                }
            }
        }
    }
}

fn cleanup_menu(mut commands: Commands, camera: Query<Entity, With<MainMenuCamera>>, menuelements: Query<Entity, With<MenuElement>>) {
    if let Ok(cam_entity) = camera.get_single() {
        commands.entity(cam_entity).despawn();
    }
    for entity in menuelements.iter() {
        commands.entity(entity).despawn();
    }
}

fn setup(
    mut commands: Commands, 
    mut meshes: ResMut<Assets<Mesh>>, 
    mut materials: ResMut<Assets<ColorMaterial>>, 
    mut count: ResMut<WallCount>,
    wall_size: Res<WallSize>,
    bmusic: Res<BackgroundMusic>
) {
    // Create 2D Ortho camera
    commands.spawn((
        Camera2dBundle {
            ..default()
        },
        MainCamera,
        GameElement
    ));

    commands.spawn(AudioBundle {
        source: bmusic.clone(),
        settings: PlaybackSettings::LOOP
    });

    // Spawn Paddle
    commands.spawn((
        MaterialMesh2dBundle {
            mesh:  meshes.add(Rectangle::default()).into(),
            material: materials.add(Color::srgb(0.3, 0.3, 0.7)),
            transform: Transform::from_xyz(0.0, PADDLE_Y, 0.0).with_scale(Vec3::new(PADDLE_WIDTH, PADDLE_HEIGHT, 1.0)),
            ..default()
        },
        Paddle,
        Collider,
        GameElement
    ));
    // Spawn walls
    for y in 0..ROWS {
        for x in 0..COLS {
            count.0 += 1;
            commands.spawn((
                MaterialMesh2dBundle {
                    mesh: meshes.add(Rectangle::default()).into(),
                    material: materials.add(Color::srgb(0.3, 0.3, 0.7)),
                    transform: Transform::from_xyz(
                        -(WINDOW_SIZE.x - wall_size.0.x) / 2.0 + (wall_size.0.x + 1.0) * x as f32, 
                        WINDOW_SIZE.y / 2.0 - wall_size.0.y / 2.0 - (wall_size.0.y + 1.0) * y as f32, 0.0,
                    ).with_scale(wall_size.0.extend(1.0)),
                    ..default()
                },
                Wall,
                Collider,
                GameElement
            ));
        }
    }

    // Boundary
    commands.spawn((
        Transform::from_xyz(-WINDOW_SIZE.x / 2.0, 0.0, 0.0).with_scale(Vec3::new(1.0, WINDOW_SIZE.y, 1.0)),
        Collider,
        GameElement
    ));
    commands.spawn((
    Transform::from_xyz(WINDOW_SIZE.x / 2.0, 0.0, 0.0).with_scale(Vec3::new(1.0, WINDOW_SIZE.y, 1.0)),
        Collider,
        GameElement
    ));
    commands.spawn((
        Transform::from_xyz(0.0, WINDOW_SIZE.y / 2.0, 0.0).with_scale(Vec3::new(WINDOW_SIZE.x, 1.0, 1.0)),
        Collider,
        GameElement
    ));
    commands.spawn((
        Transform::from_xyz(0.0, -WINDOW_SIZE.y / 2.0, 0.0).with_scale(Vec3::new(WINDOW_SIZE.x, 1.0, 1.0)),
        Collider,
        GameElement
    ));

    // Spawn Ball
    commands.spawn((
        MaterialMesh2dBundle {
            mesh:  meshes.add(Circle::default()).into(),
            material: materials.add(Color::srgb(0.3, 0.3, 0.7)),
            transform: Transform::from_xyz(0.0, PADDLE_Y + 50.0, 0.0).with_scale(Vec2::splat(BALL_RADIUS*2.0).extend(1.0)),
            ..default()
        },
        Velocity(Vec2::new(-100.0, 100.0)),
        Ball,
        GameElement
    ));
}

fn cleanup_game(mut commands: Commands, camera: Query<Entity, With<MainCamera>>, elements: Query<Entity, With<GameElement>>) {
    if let Ok(cam_entity) = camera.get_single() {
        commands.entity(cam_entity).despawn();
    }
    for entity in elements.iter() {
        commands.entity(entity).despawn();
    }
}

fn move_pedal(mut paddle: Query<&mut Transform, With<Paddle>>, keys: Res<ButtonInput<KeyCode>>, time: Res<Time>) {
    let mut transform= paddle.single_mut();
    let mut translation = Vec3::new(0.0,0.0,0.0);
    
    if keys.pressed(KeyCode::KeyA) {
        translation.x -= PADDLE_SPEED;
    }
    if keys.pressed(KeyCode::KeyD) {
        translation.x += PADDLE_SPEED;
    }

    transform.translation.x += translation.x * time.delta_seconds();
}

fn update_ball_position(mut ball: Query<(&Velocity, &mut Transform), With<Ball>>, time: Res<Time>) {
    if let Ok((vel, mut ball)) = ball.get_single_mut() {
        ball.translation += vel.0.extend(0.0) * time.delta_seconds()
    }
}

fn animate_destroyed_walls(mut commands: Commands, 
    mut walls: Query<(Entity, &mut Transform, &mut Destroyed), With<Wall>>,
    wall_size: Res<WallSize>
) {
    for (entity, mut transform, mut destroyed) in walls.iter_mut() {
        if transform.scale.length() < 0.1 {
            commands.entity(entity).despawn();
        } else {
            transform.scale = wall_size.0.extend(1.0) * (destroyed.0);
            destroyed.0 -= 0.01;
        }
    }
}