use bevy::{
    log::{Level, LogPlugin},
    prelude::*,
    window::PresentMode,
};
use bevy_rapier2d::prelude::*;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Pipe;

#[derive(Component)]
struct Health {
    pub max: u8,
    pub current: u8,
}

const PIPES_COUNT: i8 = 1;

fn main() {
    App::new()
    .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Flappy Bevy".into(),
                resolution: (600.0, 600.0).into(),
                present_mode: PresentMode::AutoVsync,
                fit_canvas_to_parent: true,
                prevent_default_event_handling: false,
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
        .add_systems(Update, (process_player_input, display_events))
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

    // let _down_sprite: Handle<Image> =
    //     asset_server.load("sprites/yellowbird-midflap.png");

    // let _up_sprite: Handle<Image> =
    //     asset_server.load("sprites/yellowbird-midflap.png");

    commands.spawn(Camera2dBundle::default());

    commands
        .spawn(RigidBody::Dynamic)
        // .insert(LockedAxes::ROTATION_LOCKED)
        .insert(Damping {
            angular_damping: 5.0,
            ..default()
        })
        .insert(GravityScale(9.0))
        .insert(Collider::ball(20.0))
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(ActiveEvents::CONTACT_FORCE_EVENTS)
        .insert(Restitution::coefficient(0.7))
        .insert(Velocity {
            linvel: Vec2::new(0.0, 0.0),
            angvel: 0.0,
        })
        .insert(Player {})
        .insert(Health { current: 1, max: 1 })
        .insert(SpriteBundle {
            texture: default_sprite,
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        });

    // for n in 0..PIPES_COUNT {

    //     if n % 2 == 0 {
    //         // lower pipe
    //     } else {
    //     }
    // }

    commands
        .spawn(RigidBody::KinematicVelocityBased)
        .insert(Collider::cuboid(25.0, 160.0))
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(Velocity {
            angvel: 0.0,
            linvel: Vec2::new(-100.0, 0.0),
        })
        .insert(SpriteBundle {
            texture: pipe_sprite,
            transform: Transform::from_xyz(200.0, 0.0, 0.0),
            ..default()
        });
}

fn process_player_input(
    input: Res<Input<KeyCode>>,
    mut query: Query<&mut Velocity, With<Player>>,
) {
    let mut player_velocity = query.single_mut();

    if input.just_pressed(KeyCode::Space) {
        player_velocity.linvel = Vec2::new(0.0, 300.0);
    }
}

fn display_events(
    mut collision_events: EventReader<CollisionEvent>,
    mut player: Query<(&mut Health, Entity, With<Player>)>,
) {
    for collision_event in collision_events.iter() {
        info!(
            "Received collision event: {collision_event:?}"
        );
        match collision_event {
            CollisionEvent::Started(e1, e2, flags) => {
                info!("Started: {e1:?}, {e2:?}, {flags:?}");
                info!("{} {}", e1.index(), e2.index());

                let mut _player = player.single_mut();

                if e1.index() == _player.1.index()
                    || e2.index() == _player.1.index()
                {
                    info!("Player collided with something");
                    info!("Player's health before deducting was: {}", _player.0.current);

                    _player.0.current -= 1;
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
