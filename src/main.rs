#![allow(clippy::type_complexity)]
mod collider;
mod debug;

use std::thread::spawn;

use bevy::sprite::Mesh2d;
use collider::detect_future_collisions;
use collider::Collider;
use collider::ColliderPlugin;
use rand::prelude::*;

use bevy::{
    math::bounding::{Aabb2d, IntersectsVolume},
    prelude::*,
    render::render_resource::ShaderType,
    sprite::MaterialMesh2dBundle,
    transform,
};
use bevy_derive::Deref;
use debug::DebugPlugin;

// use ui::setup_ui;
// use ui::update_score_text;

pub const CLEAR: Color = Color::linear_rgb(0.10, 0.10, 0.10);
const BALL_SIZE: f32 = 5.;

fn ball_is_colliding(ball: Vec2, rect: Aabb2d) -> bool {
    let ball_aabb = Aabb2d::new(ball, Vec2::new(BALL_SIZE, BALL_SIZE * 0.4));
    // Get the AABB of the rectangle
    // Check if the AABBs intersect
    ball_aabb.intersects(&rect)
}

//////////////////////////////////////////////////////////////////
// Arena Stuff
//////////////////////////////////////////////////////////////////
#[derive(Component, Reflect, Default, Deref, DerefMut, Debug)]
struct ArenaSize(Vec2);

#[derive(Component, Reflect, Default, Deref, DerefMut, Debug)]
struct ArenaCenter(Vec2);

#[derive(Bundle, Default)]
struct ArenaBundle {
    arena_size: ArenaSize,
    arena_center: ArenaCenter,
}

impl ArenaBundle {
    fn new(arena_size: Vec2, arena_center: Vec2) -> Self {
        Self {
            arena_size: ArenaSize(arena_size),
            arena_center: ArenaCenter(arena_center),
        }
    }
}

#[derive(Component, Reflect, Default, Debug)]
struct InteriorWall;

#[derive(Component, Reflect, Default, Debug, Deref, DerefMut)]
struct Gate {
    open: bool,
}

fn spawn_arena(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let arena_size = Vec2::new(1000., 1000.);
    let arena_center = Vec2::new(0., 0.);
    commands.spawn(ArenaBundle::new(arena_size, arena_center));
    let shape = Rectangle::new(arena_size.x, arena_size.y);
    let color = Color::srgb(0.9, 0.9, 0.6);
    let mesh = meshes.add(shape);
    let material = materials.add(color);
    commands.spawn((
        Name::new("Arena"),
        MaterialMesh2dBundle {
            mesh: mesh.into(),
            material,
            transform: Transform {
                translation: Vec3::new(arena_center.x, arena_center.y, 0.0),
                scale: Vec3::new(1.0, 1.0, 1.0),
                ..Default::default()
            },
            ..Default::default()
        },
    ));
}

fn spawn_walls(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let arena_size = Vec2::new(1000., 1000.);
    let arena_center = Vec2::new(0., 0.);
    let wall_length = arena_size.x * 0.4;
    let gap_length = arena_size.x * 0.2;

    let middle_wall_rect = Rectangle::new(wall_length, 10.);
    let color: Color = Color::BLACK;
    let mesh = meshes.add(middle_wall_rect);
    let material = materials.add(color);
    let right_wall_translation = Vec3::new((wall_length - gap_length / 2.), arena_center.y, 0.0);
    let collider_length = wall_length / 2.0;
    print!("Wall Translation: {:?}", right_wall_translation);
    commands.spawn((
        Name::new("Middle Wall Right"),
        InteriorWall,
        collider::Collider {
            size: Vec2::new(collider_length, 5.),
        },
        MaterialMesh2dBundle {
            mesh: mesh.clone().into(),
            material: material.clone(),
            transform: Transform {
                translation: right_wall_translation,
                ..Default::default()
            },
            ..Default::default()
        },
    ));
    commands.spawn((
        Name::new("Middle Wall Left"),
        InteriorWall,
        collider::Collider {
            size: Vec2::new(collider_length, 5.),
        },
        MaterialMesh2dBundle {
            mesh: mesh.into(),
            material,
            transform: Transform {
                translation: Vec3::new(-(wall_length - gap_length / 2.), arena_center.y, 0.0),
                ..Default::default()
            },
            ..Default::default()
        },
    ));
    // Now spawn the middle gate in between the two walls
    let gate_rect = Rectangle::new(gap_length, 10.);
    let color: Color = Color::WHITE;
    let mesh = meshes.add(gate_rect);
    let material = materials.add(color);
    let gate_translation = Vec3::new(0., arena_center.y, 0.0);
    commands.spawn((
        Name::new("Middle Wall Gate"),
        Gate { open: false },
        collider::Collider {
            size: Vec2::new(gap_length, 5.),
        },
        MaterialMesh2dBundle {
            mesh: mesh.into(),
            material,
            transform: Transform {
                translation: gate_translation,
                ..Default::default()
            },
            ..Default::default()
        },
    ));
}

fn gate_control_system(
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut gate_query: Query<(&mut Gate, &mut Handle<ColorMaterial>)>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    for (mut gate, color_handle) in gate_query.iter_mut() {
        gate.open = keys.pressed(KeyCode::KeyE);
        // If the gate is open, change the color of the gate to green
        if let Some(material) = materials.get_mut(&*color_handle) {
            if gate.open {
                *material = ColorMaterial::from(Color::srgb(0., 1., 0.));
            } else {
                *material = ColorMaterial::from(Color::srgb(1., 0., 0.));
            }
        }
    }
} //////////////////////////////////////////////////////////////////
  // Ball Stuff
  ///////////////////////////////////////////////////////////////////
#[derive(Component, Reflect, Default, Deref, DerefMut, Debug)]
struct Position(Vec2);

#[derive(Component, Reflect, Default, Deref, DerefMut, Debug)]
struct Velocity(Vec3);

#[derive(Component, Reflect, Default, Debug)]
struct Ball;

#[derive(Bundle, Default)]
struct BallBundle {
    ball: Ball,
    collider: collider::Collider,
    position: Position,
    velocity: Velocity,
}

impl BallBundle {
    fn new() -> Self {
        // Pick random velocities between -100 and 100
        let vx = (random::<f32>() * 400.0) - 200.0;
        let vy = (random::<f32>() * 400.0) - 200.0;

        // Start x and y at random between -400 and 400
        let x = (random::<f32>() * 800.0) - 400.0;
        // Start y randomly at either -200 or 200
        let y = if random::<f32>() > 0.5 { -200.0 } else { 200.0 };

        let random_velocity = Vec3::new(vx, vy, 0.);
        Self {
            ball: Ball,
            collider: collider::Collider {
                size: Vec2::new(BALL_SIZE / 2., BALL_SIZE / 2.),
            },
            position: Position(Vec2::new(x, y)),
            velocity: Velocity(random_velocity),
        }
    }
}

fn spawn_balls(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let shape = Circle { radius: BALL_SIZE };
    let mesh = meshes.add(shape);
    let mut lengths = Vec::new();
    for _ in 0..100 {
        let new_ball_bundle = BallBundle::new();
        let new_ball_velocity = new_ball_bundle.velocity.length();
        lengths.push(new_ball_velocity);
        let mut color_name = "";
        let ball_color = if new_ball_velocity > 155. {
            // Red if the ball is moving fast
            color_name = "red";
            Color::srgb(1.0, 0.0, 0.0)
        } else {
            // Blue if the ball is moving slow
            color_name = "blue";
            Color::srgb(0.0, 0.0, 1.0)
        };
        let material = materials.add(ball_color);
        commands.spawn((
            Name::new("Ball"),
            BallBundle::new(),
            MaterialMesh2dBundle {
                mesh: mesh.clone().into(),
                material: material.clone(),
                transform: Transform::from_xyz(0., 0., 1.),
                ..default()
            },
        ));
    }
    println!(
        "Average velocity: {}",
        lengths.iter().sum::<f32>() / lengths.len() as f32
    );
    println!(
        "All velocities: {}",
        lengths
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(", ")
    );
}

fn move_ball_system(
    mut balls: Query<(&Collider, &mut Position, &mut Velocity, &mut Transform), With<Ball>>,
    walls: Query<(&Collider, &Transform, Option<&Gate>), Without<Velocity>>,
    time: Res<Time>,
) {
    detect_future_collisions(&mut balls, &walls, &time);
    for (_collider, mut position, velocity, mut ball_transform) in balls.iter_mut() {
        position.x += velocity.x * time.delta_seconds();
        position.y += velocity.y * time.delta_seconds();
        ball_transform.translation.x = position.x;
        ball_transform.translation.y = position.y;
    }
}
fn ball_wall_collision_system(
    mut ball_query: Query<(&Position, &mut Velocity, &mut Handle<ColorMaterial>), With<Ball>>,
    arena_query: Query<(&ArenaSize, &ArenaCenter)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let (arena_size, arena_center) = arena_query.single();
    for (position, mut velocity, material_handle) in ball_query.iter_mut() {
        if position.x < arena_center.x - arena_size.x / 2. {
            velocity.x = velocity.x.abs();
        } else if position.x > arena_center.x + arena_size.x / 2. {
            velocity.x = -velocity.x.abs();
        }
        if position.y < arena_center.y - arena_size.y / 2. {
            velocity.y = velocity.y.abs();
        } else if position.y > arena_center.y + arena_size.y / 2. {
            velocity.y = -velocity.y.abs();
        }

        let color = if velocity.length() > 155. {
            Color::srgb(1.0, 0.0, 0.0)
        } else {
            Color::srgb(0.0, 0.0, 1.0)
        };
        // Update the material color
        if let Some(material) = materials.get_mut(&*material_handle) {
            material.color = color;
        }
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn_empty().insert(Camera2dBundle::default());
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ColliderPlugin)
        .add_plugins(DebugPlugin)
        .add_systems(Startup, (spawn_camera, spawn_arena, spawn_walls))
        .add_systems(PostStartup, spawn_balls)
        .add_systems(
            Update,
            (
                ball_wall_collision_system,
                move_ball_system,
                // update_score_text,
            ),
        )
        .add_systems(Update, gate_control_system)
        .run();
}
