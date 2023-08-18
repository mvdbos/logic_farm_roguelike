use std::f32::consts::FRAC_1_SQRT_2;

use bevy::{
    input::common_conditions::input_toggle_active, math::vec2, prelude::*,
    render::camera::ScalingMode,
};
use bevy_inspector_egui::{
    InspectorOptions, prelude::ReflectInspectorOptions, quick::WorldInspectorPlugin,
};
use bevy_rapier2d::prelude::*;

use pig::PigPlugin;
use ui::GameUI;

const DIAGONAL_MOVEMENT_NORMALIZATION_FACTOR: f32 = FRAC_1_SQRT_2;

#[derive(Component, InspectorOptions, Default, Reflect)]
#[reflect(Component, InspectorOptions)]
pub struct Player {
    #[inspector(min = 0.0)]
    pub speed: f32,
}

#[derive(Component, InspectorOptions, Default, Reflect)]
#[reflect(Component, InspectorOptions)]
pub struct Wall;

#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct Money(pub f32);

mod pig;
mod ui;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Logic Farming Roguelike".into(),
                        resolution: (640.0, 480.0).into(),
                        resizable: true,
                        ..default()
                    }),
                    ..default()
                })
                .build(),
        )
        .add_plugins(
            WorldInspectorPlugin::default().run_if(input_toggle_active(true, KeyCode::Escape)),
        )
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(RapierDebugRenderPlugin::default())
        .insert_resource(Money(100.0))
        .register_type::<Money>()
        .register_type::<Player>()
        .add_plugins((PigPlugin, GameUI))
        .add_systems(Startup, setup)
        .add_systems(PostStartup, setup_physics)
        .add_systems(Update, (player_movement, player_hit_wall, camera_follow))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut camera = Camera2dBundle::default();

    camera.projection.scaling_mode = ScalingMode::AutoMin {
        min_width: 256.0,
        min_height: 144.0,
    };

    commands.spawn(camera);

    let texture = asset_server.load("character.png");

    commands.spawn((
        SpriteBundle {
            texture,
            ..default()
        },
        Player { speed: 100.0 },
        Name::new("Player"),
    ));

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::DARK_GREEN,
                custom_size: Some(Vec2::splat(256.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, -999.0),
            ..default()
        },
        Name::new("Ground"),
    ));

    let wall_size = vec2(4.0, 200.0);
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::YELLOW,
                custom_size: Some(wall_size),
                ..default()
            },
            transform: Transform::from_xyz(-200.0, 0.0, 0.0),
            ..default()
        },
        Name::new("Wall"),
        Collider::cuboid(wall_size.x / 2.0, wall_size.y / 2.0),
        RigidBody::Fixed,
        Wall,
    ));
}

fn setup_physics(mut commands: Commands, player: Query<Entity, With<Player>>) {
    let player_entity = player.get_single().expect("1 Player");
    info!("Adding physics to player: {:?}", player_entity);
    commands
        .entity(player_entity)
        .insert(KinematicCharacterController::default())
        .insert(RigidBody::KinematicPositionBased)
        .insert(
            ActiveCollisionTypes::default()
                | ActiveCollisionTypes::KINEMATIC_KINEMATIC
                | ActiveCollisionTypes::KINEMATIC_STATIC,
        )
        .insert(Collider::cuboid(16.0 / 2.0, 16.0 / 2.0));
}

fn camera_follow(
    mut camera: Query<&mut Transform, With<Camera>>,
    player: Query<&Transform, (With<Player>, Without<Camera>)>,
) {
    let mut camera = camera.single_mut();
    let player = player.single();
    camera.translation = player.translation.truncate().extend(999.0);
}

fn player_movement(
    mut player: Query<(&Player, &mut KinematicCharacterController), With<RigidBody>>,
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let (player, mut controller) = player.get_single_mut().expect("1 Player");
    let movement_amount = player.speed * time.delta_seconds();

    let mut target_y_movement = if input.pressed(KeyCode::W) {
        movement_amount
    } else if input.pressed(KeyCode::S) {
        -movement_amount
    } else {
        0f32
    };

    let mut target_x_movement = if input.pressed(KeyCode::D) {
        movement_amount
    } else if input.pressed(KeyCode::A) {
        -movement_amount
    } else {
        0f32
    };

    normalize_diagonal_movement(&mut target_y_movement, &mut target_x_movement);

    controller.translation = Some(Vec2::new(target_x_movement, target_y_movement));
}

fn player_hit_wall(
    player: Query<Option<&KinematicCharacterControllerOutput>, With<Player>>,
    walls: Query<Entity, With<Wall>>,
) {
    let output_option = player.get_single().expect("1 Player");
    if let Some(output) = output_option {
        for collision in output.collisions.iter() {
            if walls
                .iter()
                .any(|wall_entity| wall_entity == collision.entity)
            {
                info!("Player hit wall: {:?}", collision.entity);
                // TODO: play wall_collision sound
            }
        }
    }
}

fn normalize_diagonal_movement(target_y_movement: &mut f32, target_x_movement: &mut f32) {
    if target_x_movement != &mut 0f32 && target_y_movement != &mut 0f32 {
        *target_x_movement *= DIAGONAL_MOVEMENT_NORMALIZATION_FACTOR;
        *target_y_movement *= DIAGONAL_MOVEMENT_NORMALIZATION_FACTOR;
    }
}
