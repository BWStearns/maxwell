#![allow(clippy::type_complexity)]
mod debug;

use std::thread::spawn;

use rand::prelude::*;

use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use bevy_derive::Deref;
use debug::DebugPlugin;

pub const CLEAR: Color = Color::linear_rgb(0.10, 0.10, 0.10);

#[derive(Component, Reflect, Default, Deref, DerefMut, Debug)]
struct Position(Vec2);

#[derive(Component, Reflect, Default, Deref, DerefMut, Debug)]
struct Velocity(Vec3);

#[derive(Component, Reflect, Default)]
struct Ball;

#[derive(Bundle, Default)]
struct BallBundle {
    ball: Ball,
    position: Position,
    velocity: Velocity,
}

impl BallBundle {
    fn new() -> Self {
        // Pick random velocities between 1 and 5 or -5 and -1
        let x_neg = if random::<f32>() > 0.5 { 1.0 } else { -1.0 };
        let y_neg = if random::<f32>() > 0.5 { 1.0 } else { -1.0 };
        let vx = (random::<f32>() * 100.0) * x_neg;
        let vy = (random::<f32>() * 100.0) * y_neg;
        let random_velocity = Vec3::new(vx, vy, 0.);
        Self {
            ball: Ball,
            position: Position(Vec2::new(0., 0.)),
            velocity: Velocity(random_velocity),
        }
    }
}

const BALL_SIZE: f32 = 5.;
fn spawn_ball(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let shape = Circle { radius: BALL_SIZE };
    let color = Color::srgb(1.0, 0.0, 0.0);
    let mesh = meshes.add(shape);
    let material = materials.add(color);
    for _ in 0..100 {
        commands.spawn((
            Name::new("Ball"),
            BallBundle::new(),
            MaterialMesh2dBundle {
                mesh: mesh.clone().into(),
                material: material.clone(),
                transform: Transform::from_xyz(0., 0., 0.),
                ..default()
            },
        ));
    }
}

fn move_ball_system(
    mut ball_query: Query<(&mut Position, &Velocity, &mut Transform), With<Ball>>,
    time: Res<Time>,
) {
    for (mut position, velocity, mut ball_transform) in ball_query.iter_mut() {
        position.x += velocity.x * time.delta_seconds();
        position.y += velocity.y * time.delta_seconds();
        ball_transform.translation.x = position.x;
        ball_transform.translation.y = position.y;
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn_empty().insert(Camera2dBundle::default());
}

fn main() {
    App::new()
        .add_systems(Startup, (spawn_camera, spawn_ball))
        .add_systems(Update, move_ball_system)
        .add_plugins(DefaultPlugins)
        .add_plugins(DebugPlugin)
        .run();
}
