use bevy::{math::bounding::{Aabb2d, IntersectsVolume}, prelude::*};
use crate::Velocity;

pub struct ColliderPlugin;

impl Plugin for ColliderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, detect_collisions);;
    }
}

#[derive(Component, Debug, Reflect, Default)]
pub struct Collider {
    pub size: Vec2,
}

fn detect_collisions(
    mut balls: Query<(&Collider, &Transform, &mut Velocity)>,
    mut walls: Query<(&Collider, &Transform), Without<Velocity>>,
) {
    for (collider, transform, mut velocity) in balls.iter_mut() {
        let collider_aabb = Aabb2d::new(
            transform.translation.xy(),
            collider.size
        );
        for (wall_collider, wall_transform) in walls.iter_mut() {
            let wall_aabb = Aabb2d::new(
                wall_transform.translation.xy(),
                wall_collider.size
            );
            if collider_aabb.intersects(&wall_aabb) {
                // Collision detected!
                // Handle the collision here
                // For example, you can reverse the ball's velocity
                // velocity.x = -velocity.x;
                velocity.y = -velocity.y;
            }
        }
    }
}
