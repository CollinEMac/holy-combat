use bevy::math::vec3;
use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};

/// Player movement speed factor.
const PLAYER_SPEED: f32 = 200.;

/// Camera lerp factor.
const CAM_LERP_FACTOR: f32 = 2.;

/// Collision radius for both player and opponent
const COLLISION_RADIUS: f32 = 25.;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Opponent;

#[derive(Component)]
struct Collidable {
    radius: f32,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (setup_scene, setup_camera))
        .add_systems(Update, (move_player, update_camera).chain())
        .run();
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // World where we move the player
    commands.spawn(MaterialMesh2dBundle {
        mesh: Mesh2dHandle(meshes.add(Rectangle::new(2000., 1400.))),
        material: materials.add(Color::srgb(0.2, 0.2, 0.3)),
        ..default()
    });

    // Player
    commands.spawn((
        Player,
        Collidable { radius: COLLISION_RADIUS },
        MaterialMesh2dBundle {
            mesh: meshes.add(Circle::new(COLLISION_RADIUS)).into(),
            material: materials.add(Color::srgb(0.0, 1.0, 0.0)),
            transform: Transform {
                translation: vec3(0., 0., 2.),
                ..default()
            },
            ..default()
        },
    ));
    
    // Opponent
    commands.spawn((
        Opponent,
        Collidable { radius: COLLISION_RADIUS },
        MaterialMesh2dBundle {
            mesh: meshes.add(Circle::new(COLLISION_RADIUS)).into(),
            material: materials.add(Color::srgb(1.0, 0.0, 0.0)),
            transform: Transform {
                translation: vec3(150., 0., 1.),
                ..default()
            },
            ..default()
        },
    ));
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                ..default()
            },
            ..default()
        },
    ));
}

/// Update the camera position by tracking the player.
fn update_camera(
    mut camera: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
    player: Query<&Transform, (With<Player>, Without<Camera2d>)>,
    time: Res<Time>,
) {
    let Ok(mut camera) = camera.get_single_mut() else {
        return;
    };

    let Ok(player) = player.get_single() else {
        return;
    };

    let Vec3 { x, y, .. } = player.translation;
    // Add 150 offset the camera with the player a little
    let direction = Vec3::new(x, y + 150., camera.translation.z);

    camera.translation = camera
        .translation
        .lerp(direction, time.delta_seconds() * CAM_LERP_FACTOR);
}

/// Update the player position with keyboard inputs, considering collisions.
fn move_player(
    mut player: Query<(&mut Transform, &Collidable), With<Player>>,
    opponent: Query<(&Transform, &Collidable), (With<Opponent>, Without<Player>)>,
    time: Res<Time>,
    kb_input: Res<ButtonInput<KeyCode>>,
) {
    let Ok((mut player_transform, player_collidable)) = player.get_single_mut() else {
        return;
    };

    let Ok((opponent_transform, opponent_collidable)) = opponent.get_single() else {
        return;
    };

    let mut direction = Vec2::ZERO;

    if kb_input.pressed(KeyCode::KeyA) {
        direction.x -= 1.;
    }

    if kb_input.pressed(KeyCode::KeyD) {
        direction.x += 1.;
    }

    let move_delta = direction.normalize_or_zero() * PLAYER_SPEED * time.delta_seconds();
    let new_position = player_transform.translation + move_delta.extend(0.);

    // Check if the new position would cause a collision
    let distance = new_position.distance(opponent_transform.translation);
    let min_distance = player_collidable.radius + opponent_collidable.radius;

    if distance >= min_distance {
        // No collision, apply the movement
        player_transform.translation = new_position;
    } else {
        // Collision detected, move as close as possible without overlapping
        let direction_to_opponent = (opponent_transform.translation - player_transform.translation).normalize();
        let max_movement = (distance - min_distance).max(0.0);
        let safe_move = move_delta.extend(0.).reject_from(direction_to_opponent) * (max_movement / move_delta.length());
        player_transform.translation += safe_move;
    }
}

