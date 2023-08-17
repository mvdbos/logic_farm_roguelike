use std::f32::consts::FRAC_1_SQRT_2;
use bevy::{prelude::*, render::camera::ScalingMode};
use bevy::input::common_conditions::input_toggle_active;
use bevy_inspector_egui::InspectorOptions;
use bevy_inspector_egui::prelude::ReflectInspectorOptions;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use pig::PigPlugin;
use ui::GameUI;

const DIAGONAL_MOVEMENT_NORMALIZATION_FACTOR: f32 = FRAC_1_SQRT_2;

#[derive(Component, InspectorOptions, Default, Reflect)]
#[reflect(Component, InspectorOptions)]
pub struct Player {
    #[inspector(min = 0.0)]
    pub speed: f32,
}

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
        .insert_resource(Money(100.0))
        .register_type::<Money>()
        .register_type::<Player>()
        .add_plugins((PigPlugin, GameUI))
        .add_systems(Startup, setup)
        .add_systems(Update, character_movement)
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
}

fn character_movement(
    mut characters: Query<(&mut Transform, &Player)>,
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    for (mut transform, player) in &mut characters {
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

        transform.translation.x += target_x_movement;
        transform.translation.y += target_y_movement;
    }
}

fn normalize_diagonal_movement(target_y_movement: &mut f32, target_x_movement: &mut f32) {
    if target_x_movement != &mut 0f32 && target_y_movement != &mut 0f32 {
        *target_x_movement = *target_x_movement * DIAGONAL_MOVEMENT_NORMALIZATION_FACTOR;
        *target_y_movement = *target_y_movement * DIAGONAL_MOVEMENT_NORMALIZATION_FACTOR;
    }
}
