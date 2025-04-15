use crate::ReflectComponent;
use bevy_ecs::prelude::{Commands, Component, Entity, Query, Res};
use bevy_reflect::Reflect;
use bevy_time::{Time, Timer, TimerMode};
use std::time::Duration;

pub trait EffectTimer: Sized {
    fn new(duration: Duration) -> Self;

    fn from_seconds(seconds: f32) -> Self {
        Self::new(Duration::from_secs_f32(seconds))
    }

    fn with_mode(self, mode: TimerMergeMode) -> Self;
}

/// Despawns the entity when the timer finishes.
#[derive(Component, Reflect, Eq, PartialEq, Debug, Clone)]
#[reflect(Component, PartialEq, Debug, Clone)]
pub struct Lifetime {
    pub timer: Timer,
    pub mode: TimerMergeMode,
}

impl EffectTimer for Lifetime {
    fn new(duration: Duration) -> Self {
        Self {
            timer: Timer::new(duration, TimerMode::Once),
            ..Self::default()
        }
    }

    fn with_mode(mut self, mode: TimerMergeMode) -> Self {
        self.mode = mode;
        self
    }
}

impl Default for Lifetime {
    fn default() -> Self {
        Self {
            timer: Timer::default(),
            mode: TimerMergeMode::Max,
        }
    }
}

/// Repeating timer used for the delay between effect applications.  
#[derive(Component, Reflect, Eq, PartialEq, Debug, Clone)]
#[reflect(Component, PartialEq, Debug, Clone)]
pub struct Delay {
    pub timer: Timer,
    pub mode: TimerMergeMode,
}

impl EffectTimer for Delay {
    fn new(duration: Duration) -> Self {
        Self {
            timer: Timer::new(duration, TimerMode::Repeating),
            ..Self::default()
        }
    }

    fn with_mode(mut self, mode: TimerMergeMode) -> Self {
        self.mode = mode;
        self
    }
}

impl Default for Delay {
    fn default() -> Self {
        Self {
            timer: Timer::default(),
            mode: TimerMergeMode::Fraction,
        }
    }
}

#[derive(Reflect, Eq, PartialEq, Debug, Copy, Clone)]
#[reflect(PartialEq, Debug, Clone)]
pub enum TimerMergeMode {
    /// The new effect's time will be used, ignoring the old one.
    Replace,
    /// The old effect's time will be used, ignoring the new one.
    Inherit,
    /// The new timer is used, but with the same fraction of the old timer's elapsed time.
    Fraction,
    /// The timer with the larger time remaining will be used.
    Max,
}

pub(super) fn despawn_finished_lifetimes(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Lifetime)>,
) {
    for (entity, mut lifetime) in &mut query {
        lifetime.timer.tick(time.delta());

        if lifetime.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub(super) fn tick_delay(time: Res<Time>, mut query: Query<&mut Delay>) {
    for mut delay in &mut query {
        delay.timer.tick(time.delta());
    }
}
