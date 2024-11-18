use bevy::prelude::*;

#[derive(Resource)]
pub struct WallSize(pub Vec2);

#[derive(Resource)]
pub struct WallCount(pub i32);

#[derive(Resource, Deref)]
pub struct CollisionSound(pub Handle<AudioSource>);

#[derive(Resource, Deref)]
pub struct BackgroundMusic(pub Handle<AudioSource>);