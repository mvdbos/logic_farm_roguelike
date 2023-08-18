use bevy::prelude::*;
use bevy_rapier2d::{
    control::KinematicCharacterControllerOutput,
    prelude::{Collider, KinematicCharacterController, RigidBody, Toi},
};

use crate::{Money, Player};

pub struct PigPlugin;

impl Plugin for PigPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_pig_parent)
            .add_systems(Update, (spawn_pig, pig_lifetime, bumped_by_player))
            .register_type::<Pig>();
    }
}

#[derive(Debug, Component, Default, Reflect)]
#[reflect(Component)]
pub struct Pig {
    pub lifetime: Timer,
    pub speed: f32,
}

#[derive(Component)]
pub struct PigParent;

fn spawn_pig_parent(mut commands: Commands) {
    commands.spawn((SpatialBundle::default(), PigParent, Name::new("Pig Parent")));
}

fn spawn_pig(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    input: Res<Input<KeyCode>>,
    mut money: ResMut<Money>,
    player: Query<&Transform, With<Player>>,
    parent: Query<Entity, With<PigParent>>,
) {
    if !input.just_pressed(KeyCode::Space) {
        return;
    }

    let player_transform = player.single();
    let parent = parent.single();

    let mut pig_transform = *player_transform;
    pig_transform.translation.x += 32.0;

    if money.0 >= 10.0 {
        money.0 -= 10.0;
        info!("Spent $10 on a pig, remaining money: ${:?}", money.0);

        let texture = asset_server.load("pig.png");

        commands.entity(parent).with_children(|commands| {
            commands.spawn((
                SpriteBundle {
                    texture,
                    transform: pig_transform,
                    ..default()
                },
                Pig {
                    lifetime: Timer::from_seconds(30.0, TimerMode::Once),
                    speed: 100.0,
                },
                Name::new("Pig"),
                KinematicCharacterController {
                    translation: Some(Vec2::new(0.01, 0.0)),
                    up: Vec2::X,
                    ..default()
                },
                RigidBody::KinematicPositionBased,
                Collider::cuboid(24.0 / 2.0, 16.0 / 2.0),
            ));
        });
    }
}

fn bumped_by_player(
    player: Query<Option<&KinematicCharacterControllerOutput>, With<Player>>,
    mut pigs: Query<(Entity, &Pig, &mut KinematicCharacterController)>,
    time: Res<Time>,
) {
    let output_option = player.get_single().expect("1 Player");
    if let Some(output) = output_option {
        for collision in output.collisions.iter() {
            if let Some((_, pig, mut pig_controller)) = pigs
                .iter_mut()
                .find(|(pig_entity, _, _)| *pig_entity == collision.entity)
            {
                debug!("Pig bumped by player: {:?}", collision.entity);
                let movement_amount = pig.speed * time.delta_seconds();
                pig_controller.translation =
                    determine_evasion_translation(&collision.toi, movement_amount);
            }
        }
    }
}

fn determine_evasion_translation(toi: &Toi, movement_amount: f32) -> Option<Vec2> {
    let movement_factor = 12.0;
    let pig_contact = toi.normal1;
    if pig_contact.x < 0.0 {
        return Some(Vec2::new(
            movement_factor * movement_amount,
            movement_factor * movement_amount,
        ));
    } else if pig_contact.x > 0.0 {
        return Some(Vec2::new(
            -movement_factor * movement_amount,
            -movement_factor * movement_amount,
        ));
    } else if pig_contact.y < 0.0 {
        return Some(Vec2::new(
            movement_factor * movement_amount,
            movement_factor * movement_amount,
        ));
    } else if pig_contact.y > 0.0 {
        return Some(Vec2::new(
            -movement_factor * movement_amount,
            -movement_factor * movement_amount,
        ));
    }
    return None;
}

fn pig_lifetime(
    mut commands: Commands,
    time: Res<Time>,
    mut pigs: Query<(Entity, &mut Pig)>,
    parent: Query<Entity, With<PigParent>>,
    mut money: ResMut<Money>,
) {
    let parent = parent.single();

    for (pig_entity, mut pig) in &mut pigs {
        pig.lifetime.tick(time.delta());

        if pig.lifetime.finished() {
            money.0 += 15.0;

            commands.entity(parent).remove_children(&[pig_entity]);
            commands.entity(pig_entity).despawn();

            info!("Pig sold for $15! Current Money: ${:?}", money.0);
        }
    }
}
