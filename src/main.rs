use bevy::{
    log::{Level, LogPlugin},
    prelude::*,
    render::camera::ScalingMode,
    window::{PresentMode, WindowMode},
};
use bevy_rapier2d::prelude::*;
use rand::Rng;

#[derive(Component)]
struct Player;

#[derive(PartialEq)]
enum PipeType {
    UP,
    DOWN,
}

#[derive(Component)]
struct Pipe {
    pub pipe_type: PipeType,
    pub original_x: f32,
}

#[derive(Component)]
struct Health {
    pub current: u8,
}

const SCREEN_WIDTH: f32 = 400.0;
const SCREEN_HEIGHT: f32 = 600.0;

const PIPES_COUNT: i8 = 8;
const VERTICAL_SPACE_BETWEEN_PIPES: f32 = 120.0;
const PIPE_SPEED: f32 = -120.0;

fn main() {
    App::new()
    .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Flappy Bevy".into(),
                resolution: (SCREEN_WIDTH, SCREEN_HEIGHT).into(),
                present_mode: PresentMode::AutoVsync,
                fit_canvas_to_parent: true,
                prevent_default_event_handling: false,
                mode: WindowMode::Windowed,
                ..default()
            }),
            ..default()
        }).set(LogPlugin {
            level: Level::INFO,
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(50.0))
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (process_player_input, handle_collision_events, check_player_health, process_pipes))
        .add_event::<CollisionEvent>()
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let default_sprite =
        asset_server.load("sprites/yellowbird-midflap.png");

    let pipe_sprite: Handle<Image> =
        asset_server.load("sprites/pipe-green.png");

    let bg: Handle<Image> =
        asset_server.load("sprites/background-day.png");

    // let _down_sprite: Handle<Image> =
    //     asset_server.load("sprites/yellowbird-midflap.png");

    // let _up_sprite: Handle<Image> =
    //     asset_server.load("sprites/yellowbird-midflap.png");

    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            scaling_mode: ScalingMode::AutoMax {
                // use automax to try to scale the camera to the window size. Not very responsive but it works for the most part. It would be great to achieve lib-gdx's combined mode effect.
                max_width: SCREEN_WIDTH,
                max_height: SCREEN_HEIGHT,
            },
            far: 1000.0,
            near: -1000.0,
            ..default()
        },
        ..default()
    });

    commands.spawn(SpriteBundle {
        texture: bg,
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, -10.0),
            scale: Vec3::new(1.5, 1.5, 1.0),
            ..default()
        },
        ..default()
    });

    commands
        .spawn(RigidBody::Dynamic)
        // .insert(LockedAxes::ROTATION_LOCKED)
        .insert(Damping {
            angular_damping: 5.0,
            ..default()
        })
        .insert(GravityScale(8.0))
        .insert(Collider::ball(12.0))
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(Restitution::coefficient(0.7))
        .insert(Velocity {
            linvel: Vec2::new(0.0, 0.0),
            angvel: 0.0,
        })
        .insert(Player {})
        .insert(Health { current: 1 })
        .insert(SpriteBundle {
            texture: default_sprite,
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        });

    let mut rng = rand::thread_rng();
    let mut y_offset: f64 = 0.0;

    for n in 0..PIPES_COUNT {
        if n % 2 == 0 {
            // lower pipes

            y_offset = rng.gen_range(-300..-160) as f64;

            commands
                .spawn(RigidBody::KinematicVelocityBased)
                .insert(Collider::cuboid(25.0, 160.0))
                .insert(ActiveEvents::COLLISION_EVENTS)
                .insert(Velocity {
                    angvel: 0.0,
                    linvel: Vec2::new(PIPE_SPEED, 0.0),
                })
                .insert(SpriteBundle {
                    texture: pipe_sprite.clone(),
                    transform: Transform::from_xyz(
                        200.0 + n as f32 * 70.0,
                        y_offset as f32,
                        0.0,
                    ),
                    ..default()
                })
                .insert(Pipe {
                    pipe_type: PipeType::DOWN,
                    original_x: 200.0 + n as f32 * 70.0,
                });
        } else {
            // upper pipes
            commands
                .spawn(RigidBody::KinematicVelocityBased)
                .insert(Collider::cuboid(25.0, 160.0))
                .insert(ActiveEvents::COLLISION_EVENTS)
                .insert(Velocity {
                    angvel: 0.0,
                    linvel: Vec2::new(PIPE_SPEED, 0.0),
                })
                .insert(SpriteBundle {
                    texture: pipe_sprite.clone(),
                    transform: Transform::from_xyz(
                        200.0 + (n - 1) as f32 * 70.0,
                        y_offset as f32
                            + 320.0 // collider's half_y * 2
                            + VERTICAL_SPACE_BETWEEN_PIPES,
                        0.0,
                    ),
                    sprite: Sprite {
                        flip_y: true,
                        ..Default::default()
                    },
                    ..default()
                })
                .insert(Pipe {
                    pipe_type: PipeType::UP,
                    original_x: 200.0
                        + (n - 1) as f32 * 70.0,
                });
        }
    }
}

fn process_player_input(
    input: Res<Input<KeyCode>>,
    mut query: Query<(
        &mut Velocity,
        &mut Transform,
        &mut Health,
        With<Player>,
        Without<Pipe>,
    )>,
    mut pipes: Query<(
        &mut Transform,
        &Pipe,
        With<Pipe>,
        Without<Player>,
    )>,
    mut rapier_config: ResMut<RapierConfiguration>, // we access the rapier config to resume the physics pipeline
) {
    let mut player = query.single_mut();

    if input.just_pressed(KeyCode::Space) {
        player.0.linvel = Vec2::new(0.0, 300.0);
    }

    if input.just_pressed(KeyCode::R) {
        info!("Restarting game");

        for (mut pipe_transform, pipe, _, _) in
            pipes.iter_mut()
        {
            pipe_transform.translation = Vec3::new(
                pipe.original_x,
                pipe_transform.translation.y,
                pipe_transform.translation.z,
            );
        }

        player.0.linvel = Vec2::new(0.0, 0.0);
        player.0.angvel = 0.0;
        player.2.current = 1;
        player.1.translation = Vec3::new(0.0, 0.0, 0.0);

        rapier_config.physics_pipeline_active = true; // resume the physics pipeline
    }
}

fn process_pipes(
    mut query: Query<(&mut Transform, &Pipe, With<Pipe>)>,
) {
    let mut y_offset: f64 = 0.0;

    for (mut transform, pipe, _) in query.iter_mut() {
        if transform.translation.x < -300.0 {
            if pipe.pipe_type == PipeType::DOWN {
                y_offset = rand::thread_rng()
                    .gen_range(-300..-160)
                    as f64;

                transform.translation.y = y_offset as f32;
            } else {
                transform.translation.y = y_offset as f32
                    + 320.0 // collider's half_y * 2
                    + VERTICAL_SPACE_BETWEEN_PIPES;
            }

            transform.translation.x = 200.0 + 70.0;
        }
    }
}

fn check_player_health(
    mut player: Query<(&mut Health, Entity, With<Player>)>,
    mut rapier_config: ResMut<RapierConfiguration>, // we access the rapier config to stop the physics pipeline
) {
    let mut _player = player.single_mut();

    if _player.0.current <= 0 {
        info!("Player is dead");
        rapier_config.physics_pipeline_active = false; // stop the physics pipeline
    }
}

fn handle_collision_events(
    mut collision_events: EventReader<CollisionEvent>,
    mut player: Query<(&mut Health, Entity, With<Player>)>,
) {
    for collision_event in collision_events.iter() {
        match collision_event {
            CollisionEvent::Started(e1, e2, _) => {
                let mut _player = player.single_mut();

                if e1.index() == _player.1.index()
                    || e2.index() == _player.1.index()
                {
                    info!("Player collided with something");
                    info!("Player's health before deducting was: {}", _player.0.current);

                    _player.0.current = 0;
                    info!(
                        "Player health now is: {}",
                        _player.0.current
                    );
                }
            }
            CollisionEvent::Stopped(_, _, _) => (),
        }
    }
}
