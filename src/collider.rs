use bevy::{math::bounding::{Aabb2d, IntersectsVolume}, prelude::*};
use crate::{Ball, Position, Velocity};

pub struct ColliderPlugin;

impl Plugin for ColliderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, detect_collisions);;
    }
}

#[derive(Component, Debug, Reflect, Default, Deref, DerefMut)]
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

pub fn detect_future_collisions(
    // mut balls: Query<(&Collider, &mut Velocity, &mut Transform), With<Ball>>,
    balls: &mut Query<(&Collider, &mut Position, &mut Velocity, &mut Transform), With<Ball>>,
    walls: &Query<(&Collider, &Transform), Without<Velocity>>,
    time: &Res<Time>,
) {
    for (wall_collider, wall_transform) in walls.iter() {
        let wall_aabb = Aabb2d::new(
            wall_transform.translation.xy(),
            wall_collider.size
        );
        for (collider, _pos, mut velocity, transform) in balls.iter_mut() {
            // Apply x velocity to the ball
            let future_position =
                transform.translation + Vec3::new(velocity.x * time.delta_seconds(), 0.0, 0.0);
            let future_collider_aabb = Aabb2d::new(future_position.xy(), collider.size);
            if future_collider_aabb.intersects(&wall_aabb) {
                velocity.x = -velocity.x;
            }
            // Apply y velocity to the ball 
            let future_position =
                transform.translation + Vec3::new(0.0, velocity.y * time.delta_seconds(), 0.0);
            let future_collider_aabb = Aabb2d::new(future_position.xy(), collider.size);
            if future_collider_aabb.intersects(&wall_aabb) {
                velocity.y = -velocity.y;
            }
        }
    }
}