use crate::rapier2d::RapierPipelineActive;
use bevy::app::AppExit;

use super::components::*;
use super::laser::*;
use super::state::*;
use bevy::prelude::*;
use bevy_rapier2d::{
    na::Vector2,
    physics::RigidBodyHandleComponent,
    rapier::{
        dynamics::{RigidBodyBuilder, RigidBodySet},
        geometry::ColliderBuilder,
        //        math::Point,
    },
};

pub fn spawn_player(
    mut commands: Commands,
    mut runstate: ResMut<RunState>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let texture_handle = asset_server.load("assets/playerShip2_red.png").unwrap();
    let body = RigidBodyBuilder::new_dynamic();
    let collider = ColliderBuilder::ball(1.0);
    // The triangle Collider does not compute mass
    //let collider = ColliderBuilder::triangle(
    //    Point::new(1.0, -0.5),
    //    Point::new(0.0, 0.8),
    //    Point::new(-1.0, -0.5),
    //);
    commands
        .spawn(SpriteComponents {
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, -5.0))
                .with_scale(1.0 / 37.0),
            material: materials.add(texture_handle.into()),
            ..Default::default()
        })
        .with(Ship {
            rotation_speed: 0.3,
            thrust: 60.0,
            life: 1,
        })
        .with(body)
        .with(collider)
        .with(ForStates {
            states: vec![GameState::Game, GameState::Pause, GameState::GameOver],
        });
    let player_entity = commands.current_entity().unwrap();
    runstate.player = Some(player_entity);

    // Helper points to visualize some points in space for Collider
    //commands
    //    .spawn(SpriteComponents {
    //        translation: Translation::new(1.2, -1.0, 2.0),
    //        material: materials.add(texture_handle.into()),
    //        scale: Scale(0.001),
    //        ..Default::default()
    //    })
    //    .spawn(SpriteComponents {
    //        translation: Translation::new(0.0, 1.0, 2.0),
    //        material: materials.add(texture_handle.into()),
    //        scale: Scale(0.001),
    //        ..Default::default()
    //    })
    //    .spawn(SpriteComponents {
    //        translation: Translation::new(-1.2, -1.0, 2.0),
    //        material: materials.add(texture_handle.into()),
    //        scale: Scale(0.001),
    //        ..Default::default()
    //    });
}

pub fn player_dampening_system(
    time: Res<Time>,
    runstate: Res<RunState>,
    mut bodies: ResMut<RigidBodySet>,
    query: Query<&RigidBodyHandleComponent>,
) {
    if runstate.current == Some(GameState::Game) {
        if let Ok(body_handle) = query.get::<RigidBodyHandleComponent>(runstate.player.unwrap()) {
            let elapsed = time.delta_seconds;
            let mut body = bodies.get_mut(body_handle.handle()).unwrap();
            body.angvel = body.angvel * 0.1f32.powf(elapsed);
            body.linvel = body.linvel * 0.8f32.powf(elapsed);
        }
    }
}

pub fn user_input_system(
    commands: Commands,
    asset_server: Res<AssetServer>,
    materials: ResMut<Assets<ColorMaterial>>,
    audio_output: Res<AudioOutput>,
    mut runstate: ResMut<RunState>,
    input: Res<Input<KeyCode>>,
    mut rapier_active: ResMut<RapierPipelineActive>,
    mut bodies: ResMut<RigidBodySet>,
    mut app_exit_events: ResMut<Events<AppExit>>,
    query: Query<(&RigidBodyHandleComponent, &Ship)>,
) {
    if runstate.current == Some(GameState::Game) {
        let player = runstate.player.unwrap();
        let mut rotation = 0;
        let mut thrust = 0;
        if input.pressed(KeyCode::W) {
            thrust += 1
        }
        if input.pressed(KeyCode::A) {
            rotation += 1
        }
        if input.pressed(KeyCode::D) {
            rotation -= 1
        }
        if rotation != 0 || thrust != 0 {
            if let Ok(body_handle) = query.get::<RigidBodyHandleComponent>(player) {
                let mut body = bodies.get_mut(body_handle.handle()).unwrap();
                let ship = query.get::<Ship>(player).unwrap();
                if rotation != 0 {
                    let rotation = rotation as f32 * ship.rotation_speed;
                    body.wake_up();
                    body.apply_torque_impulse(rotation);
                }
                if thrust != 0 {
                    let force = body.position.rotation.transform_vector(&Vector2::y())
                        * thrust as f32
                        * ship.thrust;
                    body.wake_up();
                    body.apply_force(force);
                }
            }
        }
        if input.just_pressed(KeyCode::Space) {
            if let Ok(body_handle) = query.get::<RigidBodyHandleComponent>(player) {
                let body = bodies.get(body_handle.handle()).unwrap();
                spawn_laser(commands, body, asset_server, materials, audio_output);
            }
        }
        if input.just_pressed(KeyCode::Escape) {
            runstate.next = Some(GameState::Pause);
            rapier_active.0 = false;
        }
    } else if runstate.current == Some(GameState::StartMenu) {
        if input.just_pressed(KeyCode::Return) {
            runstate.next = Some(GameState::Game);
        }
        if input.just_pressed(KeyCode::Escape) {
            app_exit_events.send(AppExit);
        }
    } else if runstate.current == Some(GameState::GameOver) {
        if input.just_pressed(KeyCode::Return) {
            runstate.next = Some(GameState::StartMenu);
        }
        if input.just_pressed(KeyCode::Escape) {
            app_exit_events.send(AppExit);
        }
    } else if runstate.current == Some(GameState::Pause) {
        if input.just_pressed(KeyCode::Escape) {
            runstate.next = Some(GameState::Game);
            rapier_active.0 = true;
        }
    }
}