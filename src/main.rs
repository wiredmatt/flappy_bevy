use bevy::{
    log::{Level, LogPlugin},
    prelude::*,
    render::camera::ScalingMode,
    window::{PresentMode, WindowMode},
};
use bevy_rapier2d::{prelude::*, rapier::prelude::CollisionEventFlags};

#[derive(PartialEq)]
enum PipeType {
    UP,
    DOWN,
}

#[derive(Component)]
struct Pipe {
    pub pipe_type: PipeType,
    pub original_x: f32,
} // tag component, with some extra meta-data

#[derive(Component)]
struct Player; // tag component

#[derive(Component)]
struct Health {
    pub current: u8,
} // state component, many entities could have it

#[derive(Component)]
struct Score {
    pub current: u8,
} // state component, many entities could have it, for example in a multiplayer game

const SCREEN_WIDTH: f32 = 400.0;
const SCREEN_HEIGHT: f32 = 600.0;

const PIPES_COUNT: i8 = 8;
const VERTICAL_SPACE_BETWEEN_PIPES: f32 = 120.0;
const PIPE_SPEED: f32 = -120.0;
const JUMP_FORCE: f32 = 300.0;

#[derive(Component)]
struct TextScore; // tag component

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
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
                })
                .set(LogPlugin {
                    level: Level::INFO,
                    ..default()
                }),
        )
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(50.0))
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                process_player_input,
                handle_collision_events,
                check_player_health,
                process_pipes,
                update_score,
            ),
        )
        .add_event::<CollisionEvent>()
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let default_sprite = asset_server.load("sprites/yellowbird-midflap.png");

    let pipe_sprite: Handle<Image> = asset_server.load("sprites/pipe-green.png");

    let bg: Handle<Image> = asset_server.load("sprites/background-day.png");

    let base: Handle<Image> = asset_server.load("sprites/base.png");

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
        .spawn(RigidBody::Fixed)
        .insert(Collider::cuboid(SCREEN_WIDTH / 2.0, 55.0))
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(SpriteBundle {
            texture: base,
            transform: Transform {
                translation: Vec3::new(0.0, -300.0, 1.0),
                scale: Vec3::new(1.5, 1.0, 1.0),
                ..default()
            },
            ..default()
        });

    commands
        .spawn(RigidBody::Dynamic)
        .insert(Damping {
            angular_damping: 0.0,
            linear_damping: 0.0,
            ..default()
        })
        .insert(GravityScale(8.0))
        .insert(Collider::ball(12.0))
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(Velocity {
            linvel: Vec2::new(0.0, 0.0),
            angvel: 0.0,
        })
        .insert(Player)
        .insert(Health { current: 1 })
        .insert(Score { current: 0 })
        .insert(SpriteBundle {
            texture: default_sprite,
            transform: Transform::from_xyz(0.0, 0.0, 2.0),
            ..default()
        });

    let mut y_offset: f32 = 0.0;

    for n in 0..PIPES_COUNT {
        if n % 2 == 0 {
            // lower pipes

            y_offset = fastrand::i32((-SCREEN_HEIGHT as i32 / 2)..-180) as f32;

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
                        SCREEN_WIDTH / 2.0 + n as f32 * 70.0,
                        y_offset as f32,
                        0.0,
                    ),
                    ..default()
                })
                .insert(Pipe {
                    pipe_type: PipeType::DOWN,
                    original_x: SCREEN_WIDTH / 2.0 + n as f32 * 70.0,
                });
        } else {
            // upper pipes
            commands
                .spawn(RigidBody::KinematicVelocityBased)
                .insert(Collider::cuboid(25.0, 160.0))
                .insert(Velocity {
                    angvel: 0.0,
                    linvel: Vec2::new(PIPE_SPEED, 0.0),
                })
                .insert(SpriteBundle {
                    texture: pipe_sprite.clone(),
                    sprite: Sprite {
                        flip_y: true,
                        ..default()
                    },
                    transform: Transform::from_xyz(
                        SCREEN_WIDTH / 2.0 + (n - 1) as f32 * 70.0,
                        y_offset as f32
                            + 320.0 // collider's half_y * 2
                            + VERTICAL_SPACE_BETWEEN_PIPES,
                        0.0,
                    ),
                    ..default()
                })
                .insert(Pipe {
                    pipe_type: PipeType::UP,
                    original_x: SCREEN_WIDTH / 2.0 + (n - 1) as f32 * 70.0,
                })
                .with_children(|children| {
                    // will be used to detect when the player passes through the pipes
                    children
                        .spawn(RigidBody::Fixed)
                        .insert(Collider::cuboid(10.0, SCREEN_HEIGHT * 2.0))
                        .insert(Sensor)
                        .insert(TransformBundle {
                            local: Transform {
                                translation: Vec3::new(50.0, 0.0, 1.0),
                                ..default()
                            },
                            ..default()
                        });
                });
        }
    }

    commands.spawn((
        // Create a TextBundle that has a Text with a single section.
        TextBundle::from_section(
            // Accepts a `String` or any type that converts into a `String`, such as `&str`
            "0",
            TextStyle {
                // This font is loaded and will be used instead of the default font.
                font: asset_server.load("fonts/flappy-font.ttf"),
                font_size: 100.0,
                color: Color::WHITE,
            },
        ) // Set the alignment of the Text
        .with_text_alignment(TextAlignment::Center)
        // Set the style of the TextBundle itself.
        .with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(5.0),
            right: Val::Px(5.0),
            ..default()
        }),
        TextScore,
    ));
}

fn update_score(
    mut text: Query<(&mut Text, With<TextScore>)>,
    score: Query<(&Score, With<Score>, With<Player>)>,
) {
    let mut score_text = text.single_mut();

    score_text.0.sections[0].value = score.single().0.current.to_string();
}

fn process_player_input(
    input: Res<Input<KeyCode>>,
    mut _player: Query<(
        &mut Velocity,
        &mut Transform,
        &mut Health,
        With<Player>,
        Without<Pipe>,
    )>,
    mut pipes: Query<(&mut Transform, &Pipe, With<Pipe>, Without<Player>)>,
    mut rapier_config: ResMut<RapierConfiguration>, // we access the rapier config to resume the physics pipeline
) {
    let mut player = _player.single_mut();

    if input.just_pressed(KeyCode::Space)
        && player.1.translation.y < SCREEN_HEIGHT / 2.0
        && player.2.current > 0
    {
        player.0.linvel = Vec2::new(0.0, JUMP_FORCE);
    }

    // would be best to use a Variable Curve, or a Tween here
    // https://bevyengine.org/examples/Animation/animated-transform/
    player.1.rotation = Quat::from_rotation_z(player.0.linvel.y / JUMP_FORCE * 0.5);

    if input.just_pressed(KeyCode::R) {
        info!("Restarting game");

        for (mut pipe_transform, pipe, _, _) in pipes.iter_mut() {
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
        player.1.rotation = Quat::from_rotation_z(0.0);

        rapier_config.physics_pipeline_active = true; // resume the physics pipeline
    }
}

fn process_pipes(mut query: Query<(&mut Transform, &Pipe, With<Pipe>)>) {
    let mut y_offset: f32 = 0.0;

    for (mut transform, pipe, _) in query.iter_mut() {
        if transform.translation.x <= -(SCREEN_WIDTH / 2.0) - (50.0 * 2.0)
        // 50 is the pipe's width
        {
            if pipe.pipe_type == PipeType::DOWN {
                y_offset = fastrand::i32((-SCREEN_HEIGHT as i32 / 2)..-180) as f32;

                transform.translation.y = y_offset;
            } else {
                if y_offset != 0.0 {
                    transform.translation.y = y_offset
                    + 320.0 // collider's half_y * 2
                    + VERTICAL_SPACE_BETWEEN_PIPES;
                }
            }

            transform.translation.x = SCREEN_WIDTH / 2.0 + 70.0;
        }
    }
}

fn check_player_health(
    _player: Query<(&Health, Entity, With<Player>)>,
    mut rapier_config: ResMut<RapierConfiguration>, // we access the rapier config to stop the physics pipeline
) {
    let player = _player.single();

    if player.0.current <= 0 {
        info!("Player is dead");
        rapier_config.physics_pipeline_active = false; // stop the physics pipeline
    }
}

fn handle_collision_events(
    mut collision_events: EventReader<CollisionEvent>,
    mut player: Query<(&mut Health, Entity, &mut Score, With<Player>)>,
) {
    for collision_event in collision_events.iter() {
        match collision_event {
            CollisionEvent::Started(e1, e2, flags) => {
                let mut _player = player.single_mut();

                // use the flags to check if it's collision or sensor
                if flags.contains(CollisionEventFlags::SENSOR)
                    && (e1.index() == _player.1.index() || e2.index() == _player.1.index())
                {
                    info!("Player passed through pipes");

                    _player.2.current += 1;
                    info!("Player score now is: {}", _player.2.current);
                } else {
                    if e1.index() == _player.1.index() || e2.index() == _player.1.index() {
                        info!("Player collided with something");
                        _player.0.current = 0;
                        info!("Player health now is: {}", _player.0.current);
                    }
                }
            }
            CollisionEvent::Stopped(_, _, _) => (),
        }
    }
}
