use std::time::Duration;

use bevy::prelude::*;
use bevy_rapier2d::{
    control::KinematicCharacterControllerOutput,
    prelude::{ActiveCollisionTypes, Collider, KinematicCharacterController, RigidBody, Toi},
};
use rand::Rng;

use crate::{Money, Player};

pub struct PigPlugin;

impl Plugin for PigPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_pig_parent)
            .add_systems(
                Update,
                (spawn_pig, pig_lifetime, bumped_by_player, pig_wander),
            )
            .register_type::<Pig>();
    }
}

#[derive(Debug, Component, Default, Reflect)]
#[reflect(Component)]
pub struct Pig {
    pub lifetime: Timer,
    pub movement_speed: f32,
    pub home: Vec2,
    pub direction: Vec2,
    new_direction_timer: Timer,
    pub wander_range: f32,
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
                    movement_speed: 25.0,
                    home: pig_transform.translation.truncate(),
                    direction: Vec2::new(1.0, 0.0),
                    new_direction_timer: Timer::from_seconds(1.0, TimerMode::Repeating),
                    wander_range: 150.,
                },
                Name::new("Pig"),
                KinematicCharacterController::default(),
                RigidBody::KinematicPositionBased,
                Collider::cuboid(24.0 / 2.0, 16.0 / 2.0),
                ActiveCollisionTypes::default()
                    | ActiveCollisionTypes::KINEMATIC_KINEMATIC
                    | ActiveCollisionTypes::KINEMATIC_STATIC,
            ));
        });
    }
}

fn bumped_by_player(
    player: Query<Option<&KinematicCharacterControllerOutput>, With<Player>>,
    mut pigs: Query<(Entity, &mut Pig)>,
    time: Res<Time>,
) {
    let output_option = player.get_single().expect("1 Player");
    if let Some(output) = output_option {
        for collision in output.collisions.iter() {
            if let Some((_, mut pig)) = pigs
                .iter_mut()
                .find(|(pig_entity, _)| *pig_entity == collision.entity)
            {
                debug!("Pig bumped by player: {:?}", collision.entity);
                pig.direction = determine_evasion_translation(&collision.toi, pig.as_ref(), &time);
                pig.new_direction_timer
                    .set_duration(Duration::from_secs_f32(1.0));
            }
        }
    }
}

fn determine_evasion_translation(toi: &Toi, pig: &Pig, time: &Res<Time>) -> Vec2 {
    let movement_amount = pig.movement_speed * time.delta_seconds();
    let evasion_speed_factor = 6.0;
    let pig_contact = toi.normal1;
    let evasion_amount = evasion_speed_factor * movement_amount;

    if pig_contact.x < 0.0 {
        Vec2::new(
            evasion_amount,
            evasion_amount,
        )
    } else if pig_contact.x > 0.0 {
        Vec2::new(
            -evasion_amount,
            -evasion_amount,
        )
    } else if pig_contact.y < 0.0 {
        Vec2::new(
            evasion_amount,
            evasion_amount,
        )
    } else if pig_contact.y > 0.0 {
        Vec2::new(
            -evasion_amount,
            -evasion_amount,
        )
    } else {
        Vec2::ZERO
    }
}

fn pig_wander(mut pigs: Query<(&mut Pig, &mut KinematicCharacterController)>, time: Res<Time>) {
    let mut rng = rand::thread_rng();
    for (mut pig, mut controller) in &mut pigs {
        pig.new_direction_timer.tick(time.delta());
        if pig.new_direction_timer.just_finished() {
            let wander_time = rng.gen_range(1.5..3.0);

            pig.new_direction_timer
                .set_duration(Duration::from_secs_f32(wander_time));

            let x = rng.gen_range(-1.0..1.0);
            let y = rng.gen_range(-1.0..1.0);
            pig.direction = Vec2::new(x, y).normalize();
        }

        // if (transform.translation.truncate() - pig.home).length() > pig.wander_range
        // {     pig.direction = -(transform.translation.truncate() -
        // pig.home).normalize(); }

        controller.translation = Some(pig.direction * pig.movement_speed * time.delta_seconds());
    }
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
