use bevy::prelude::*;

#[derive(Component)]
pub struct MenuElement;

#[derive(Component)]
pub struct MainMenuCamera;

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
pub struct Collider;

#[derive(Component)]
pub struct Collided;

#[derive(Component)]
pub struct Ball;

#[derive(Component)]
pub struct Paddle;

#[derive(Component)]
pub struct Destroyed(pub f32);

#[derive(Component)]
pub struct Velocity(pub Vec2);

#[derive(Component)]
pub struct Wall;

#[derive(Component)]
pub struct GameElement;