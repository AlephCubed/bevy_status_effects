use bevy_app::{App, Plugin, PreUpdate};
use bevy_asset::Handle;
use bevy_ecs::prelude::*;
use bevy_image::Image;
use bevy_time::{Time, Timer, TimerMode};
use std::time::Duration;

pub struct StatusEffectPlugin;

impl Plugin for StatusEffectPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, (despawn_finished_lifetimes, tick_delay).chain());
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

impl Lifetime {
    pub fn new(duration: Duration) -> Self {
        Self(Timer::new(duration, TimerMode::Once))
    }

    pub fn from_seconds(seconds: f32) -> Self {
        Self::new(Duration::from_secs_f32(seconds))
    }
}

/// Repeating timer used for the delay between effect applications.  
#[derive(Component)]
pub struct Delay(pub Timer);

impl Delay {
    pub fn new(duration: Duration) -> Self {
        Self(Timer::new(duration, TimerMode::Repeating))
    }

    pub fn from_seconds(seconds: f32) -> Self {
        Self::new(Duration::from_secs_f32(seconds))
    }
}

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

fn tick_delay(time: Res<Time>, mut query: Query<&mut Delay>) {
    for mut delay in &mut query {
        delay.0.tick(time.delta());
    }
}

/// The icon of a status effect.
pub struct Icon(pub Handle<Image>);
