//! Relationship-based status effects for bevy.

mod command;
mod relation;
mod timer;

use bevy_app::{App, Plugin, PreUpdate};
use bevy_ecs::prelude::*;
use bevy_reflect::prelude::ReflectDefault;
use bevy_reflect::{reflect_trait, Reflect};

pub use command::*;
pub use relation::*;
pub use timer::*;

pub struct StatusEffectPlugin;

impl Plugin for StatusEffectPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<EffectMode>()
            .register_type::<Effecting>()
            .register_type::<EffectedBy>()
            .register_type::<Lifetime>()
            .register_type::<Delay>()
            .register_type::<TimerMergeMode>()
            .add_systems(PreUpdate, (despawn_finished_lifetimes, tick_delay).chain());
    }
}

#[reflect_trait]
pub trait StatusEffect {}

/// Describes the logic used when multiple of the same effect are applied to the same entity.
#[derive(Component, Reflect, Eq, PartialEq, Debug, Default, Copy, Clone)]
#[reflect(Component, PartialEq, Debug, Default, Clone)]
pub enum EffectMode {
    /// Multiple of the same effect can exist at once.
    #[default]
    Stack,
    /// When an effect is added, any matching effects are removed.
    Replace,
    Merge,
}
