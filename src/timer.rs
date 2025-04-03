use bevy_ecs::prelude::{Commands, Component, Entity, Query, Res};
use bevy_time::{Time, Timer, TimerMode};
use std::time::Duration;

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

pub(super) fn despawn_finished_lifetimes(
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

pub(super) fn tick_delay(time: Res<Time>, mut query: Query<&mut Delay>) {
    for mut delay in &mut query {
        delay.0.tick(time.delta());
    }
}
