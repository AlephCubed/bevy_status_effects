use bevy_app::{App, Plugin, PreUpdate};
use bevy_asset::Handle;
use bevy_ecs::prelude::{Commands, Component, Entity, Query, Res};
use bevy_image::Image;
use bevy_time::{Time, Timer};

pub struct StatusEffectPlugin;

impl Plugin for StatusEffectPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, despawn_finished_lifetimes);
    }
}

/// Stores the entity that is being effected by this status effect.
#[derive(Component)]
#[relationship(relationship_target = EffectedBy)]
pub struct Effects(pub Entity);

/// Stores all the status effects that are effecting this entity.
#[derive(Component)]
#[relationship_target(relationship = Effects, linked_spawn)]
pub struct EffectedBy(Vec<Entity>);

impl<'a> IntoIterator for &'a EffectedBy {
    type Item = <Self::IntoIter as Iterator>::Item;

    type IntoIter = std::slice::Iter<'a, Entity>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

/// Despawns the entity when the timer finishes.
#[derive(Component)]
pub struct Lifetime(pub Timer);

fn despawn_finished_lifetimes(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Lifetime)>,
) {
    for (entity, mut lifetime) in &mut query {
        lifetime.0.tick(time.delta());

        if lifetime.0.finished() {
            commands.entity(entity).despawn();
        }
    }
}

/// The icon of a status effect.
pub struct Icon(pub Handle<Image>);
