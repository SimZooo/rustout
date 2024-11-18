use bevy::{
    math::bounding::{Aabb2d, BoundingCircle, IntersectsVolume},
    prelude::*,
};

use crate::components::*;
use crate::resources::*;

pub const BALL_RADIUS: f32 = 10.0;
pub const WINDOW_SIZE: Vec2 = Vec2::new(600.0, 400.0);

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    Menu,
    InGame,
    //Settings
}

#[derive(PartialEq, Eq)]
pub enum CollisionType {
    Paddle,
    Wall,
}

#[derive(PartialEq, Eq)]
pub enum CollisionSide {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Event)]
pub struct CollisionEvent {
    pub collisiontype: CollisionType,
    pub side: CollisionSide,
    pub entity: Option<Entity>,
}

pub fn check_boundary_collide(ball: BoundingCircle, bounding_box: Aabb2d) -> Option<CollisionSide> {
    if !ball.intersects(&bounding_box) {
        return None;
    }

    let closest = bounding_box.closest_point(ball.center);
    let offset = ball.center - closest;
    let side = if offset.x.abs() > offset.y.abs() {
        if offset.x < 0.0 {
            CollisionSide::Left
        } else {
            CollisionSide::Right
        }
    } else if offset.y > 0.0 {
        CollisionSide::Top
    } else {
        CollisionSide::Bottom
    };
    Some(side)
}

pub fn check_collisions(
    mut ball: Query<&mut Transform, (With<Ball>, Without<Paddle>, Without<Wall>)>,
    mut walls: Query<(Entity, &Transform, Option<&Wall>), (With<Collider>, Without<Ball>)>,
    mut evw: EventWriter<CollisionEvent>,
    sound: Res<CollisionSound>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<AppState>>,
    count: Res<WallCount>,
) {
    if let Ok(ball) = ball.get_single_mut() {
        for (entity, collider_transform, is_wall) in walls.iter_mut() {
            let side = check_boundary_collide(
                BoundingCircle::new(ball.translation.truncate(), BALL_RADIUS),
                Aabb2d::new(
                    collider_transform.translation.truncate(),
                    collider_transform.scale.truncate() / 2.0,
                ),
            );
            if let Some(side) = side {
                evw.send(CollisionEvent {
                    collisiontype: if is_wall.is_some() {
                        CollisionType::Wall
                    } else {
                        CollisionType::Paddle
                    },
                    side,
                    entity: if is_wall.is_some() {
                        Some(entity)
                    } else {
                        None
                    },
                });
                commands.spawn(AudioBundle {
                    source: sound.clone(),
                    settings: PlaybackSettings::DESPAWN,
                });
            }
        }
    }
    if count.0 == 0 {
        next_state.set(AppState::Menu);
    }
}

pub fn on_collision(
    mut commands: Commands,
    mut ball: Query<(Entity, &mut Velocity), (With<Ball>, Without<Paddle>, Without<Wall>)>,
    mut evr: EventReader<CollisionEvent>,
    mut count: ResMut<WallCount>,
) {
    if let Ok((entity, mut vel)) = ball.get_single_mut() {
        if evr.is_empty() {
            commands.entity(entity).remove::<Collided>();
            return;
        }
        for event in evr.read() {
            commands.entity(entity).insert(Collided);
            match event.side {
                CollisionSide::Bottom | CollisionSide::Top => {
                    vel.0.y *= -1.0;
                }
                CollisionSide::Left | CollisionSide::Right => vel.0.x *= -1.0,
            }
            if let Some(entity) = event.entity {
                if event.collisiontype == CollisionType::Wall {
                    commands
                        .entity(entity)
                        .insert(Destroyed(1.0))
                        .remove::<Collider>();
                    count.0 -= 1;
                }
            }
        }
    }
}

pub fn check_ui_collision(position: Vec2, element_size: Vec2, element_center: Vec2) -> bool {
    let position = Vec2::new(
        position.x - WINDOW_SIZE.x / 2.0,
        position.y - WINDOW_SIZE.y / 2.0,
    );
    if position.x > element_center.x - element_size.x / 2.0
        && position.x < element_center.x + element_size.x / 2.0
        && position.y > -element_size.y / 2.0 - 30.0
        && position.y < element_size.y / 2.0 - element_center.y
    {
        return true;
    } else {
        return false;
    }
}
