use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_xpbd_3d::{math::*, prelude::*};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PhysicsPlugins::default(),
            PhysicsDebugPlugin::default(),
            WorldInspectorPlugin::default(),
        ))
        .register_type::<Character>()
        .add_systems(Startup, setup)
        .add_systems(Update, (restart, floating_capsule))
        .run();
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct Character {
    pub ride_height: f32,
    pub spring_strength: f32,
    pub spring_damper: f32,
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        projection: Projection::Orthographic(OrthographicProjection {
            scale: 0.02,
            ..default()
        }),
        ..default()
    });

    commands.spawn((
        TransformBundle::from_transform(Transform::from_translation(Vec3::Y * 10.0)),
        RigidBody::Dynamic,
        LockedAxes::ROTATION_LOCKED,
        Collider::capsule(1.0, 0.5),
        ExternalForce::new(Vec3::ZERO).with_persistence(false),
        Restitution {
            coefficient: 0.0,
            combine_rule: CoefficientCombine::Min,
        },
        Character {
            ride_height: 0.1,
            spring_strength: 11.5,
            spring_damper: 5.0,
        },
        ShapeCaster::new(
            Collider::capsule(1.0, 0.35),
            Vec3::NEG_Y * 0.05,
            Quaternion::default(),
            Direction3d::NEG_Y,
        )
        .with_max_time_of_impact(0.5)
        .with_max_hits(1)
        .with_ignore_self(true),
        Name::from("Player"),
    ));
    commands.spawn((
        TransformBundle::from_transform(Transform::from_translation(Vec3::NEG_Y * 2.0)),
        RigidBody::Static,
        Collider::cuboid(5.0, 0.5, 5.0),
    ));
}

fn restart(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    player_query: Query<Entity, With<Character>>,
) {
    if let Ok(entity) = player_query.get_single() {
        if input.just_pressed(KeyCode::KeyR) {
            commands.entity(entity).despawn_recursive();

            commands.spawn((
                TransformBundle::from_transform(Transform::from_translation(Vec3::Y * 10.0)),
                RigidBody::Dynamic,
                LockedAxes::ROTATION_LOCKED,
                Collider::capsule(1.0, 0.5),
                Character {
                    ride_height: 0.5,
                    spring_strength: 0.5,
                    spring_damper: 0.5,
                },
                ShapeCaster::new(
                    Collider::capsule(1.0, 0.35),
                    Vec3::NEG_Y * 0.05,
                    Quaternion::default(),
                    Direction3d::NEG_Y,
                )
                .with_max_time_of_impact(5.0)
                .with_max_hits(1)
                .with_ignore_self(true),
                Name::from("Player"),
            ));
        }
    }
}

fn floating_capsule(
    mut character_query: Query<
        (&mut ExternalForce, &LinearVelocity, &ShapeHits, &Character),
        With<Character>,
    >,
    velocity_query: Query<&LinearVelocity, Without<Character>>,
) {
    for (mut force, velocity, ground_hits, character) in &mut character_query {
        if !ground_hits.is_empty() {
            let ray_dir = Vec3::Y;
            let mut other_velocity = LinearVelocity::default();
            let mut distance = 0.0_f32;

            if let Some(shape_hit_data) = ground_hits.iter().next() {
                distance = shape_hit_data.point1.distance(shape_hit_data.point2);
                if let Ok(lin_vel) = velocity_query.get(shape_hit_data.entity) {
                    other_velocity.0 = lin_vel.0;
                }
            }

            let self_downward_force = ray_dir.dot(velocity.0);
            let other_downard_force = ray_dir.dot(other_velocity.0);

            let relative_force = self_downward_force - other_downard_force;
            let x = distance + character.ride_height;

            let spring_force =
                (x * character.spring_strength) - (relative_force * character.spring_damper);

            let applied_force = ray_dir * spring_force;
            force.apply_force(applied_force);
        }
    }
}
