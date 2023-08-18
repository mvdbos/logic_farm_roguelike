use bevy::prelude::*;
use bevy_rapier2d::control::KinematicCharacterControllerOutput;
use bevy_rapier2d::prelude::{ActiveCollisionTypes, Collider, KinematicCharacterController, RapierContext, RigidBody};

use crate::{Money, Player};

pub struct PigPlugin;

impl Plugin for PigPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_pig_parent)
            .add_systems(Update, (spawn_pig, pig_lifetime, detect_player_collisions))
            .register_type::<Pig>();
    }
}

#[derive(Debug, Component, Default, Reflect)]
#[reflect(Component)]
pub struct Pig {
    pub lifetime: Timer,
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
                },
                Name::new("Pig"),
                KinematicCharacterController {
                    translation: Some(Vec2::new(0.01, 0.0)),
                    up: Vec2::X,
                    ..default()
                },
                RigidBody::KinematicPositionBased,
                ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_KINEMATIC,
                Collider::cuboid(24.0 / 2.0, 16.0 / 2.0),
            ));
        });
    }
}

fn detect_player_collisions(
    // mut commands: Commands,
    mut pigs: Query<(Entity, &mut Pig, &mut KinematicCharacterController, &KinematicCharacterControllerOutput)>,
    player: Query<Entity, With<Player>>,
    rapier_context: Res<RapierContext>,
) {
    let player = player.get_single().expect("1 Player");
    for (pig_entity, mut pig, mut controller, output) in &mut pigs {
        // controller.translation = display_contact_info(player, pig_entity, &rapier_context);
    }
}

fn display_contact_info(player: Entity, pig: Entity, rapier_context: &Res<RapierContext>) -> Option<Vec2> {
    if let Some(contact_pair) = rapier_context.contact_pair(player, pig) {
        // The contact pair exists meaning that the broad-phase identified a potential contact.
        if contact_pair.has_any_active_contacts() {
            // The contact pair has active contacts, meaning that it
            // contains contacts for which contact forces were computed.
            info!("The contact pair has active contacts.");

            for manifold in contact_pair.manifolds() {
                // println!("Local-space contact player: {}", manifold.local_n1());
                // println!("Local-space contact pig: {}", manifold.local_n2());
                // Read the geometric contacts.
                for contact_point in manifold.points() {
                    // Keep in mind that all the geometric contact data are expressed in the local-space of the colliders.
                    println!("Found local contact point on player: {:?}", contact_point.local_p1());
                    println!("Found local contact point on pig: {:?}", contact_point.local_p2());
                }
                // let evasion_translation = if manifold.local_n1().x == 0.0 {
                //     Vec2::new(4.0, 2.0)
                // } else {
                //     Vec2::new(2.0, 4.0)
                // };
                // return Some(evasion_translation);

            }
        }
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
