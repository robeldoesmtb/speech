// src/ecs/components.rs
use nalgebra as na;

// Position component
pub struct Position(pub na::Vector2<f32>);

// Velocity component
pub struct Velocity(pub na::Vector2<f32>);

// Sprite component
pub struct Sprite {
    pub texture_id: usize,
    pub width: f32,
    pub height: f32,
}

// Player component (marker for player entity)
pub struct Player;

// Collider component
pub struct Collider {
    pub width: f32,
    pub height: f32,
    pub collision_type: CollisionType,
}

pub enum CollisionType {
    Solid,
    Trigger,
    Evidence,
}