pub mod relation;
pub mod timer;

use crate::relation::{EffectedBy, Effecting};
use bevy_app::{App, Plugin, PreUpdate};
use bevy_ecs::component::HookContext;
use bevy_ecs::prelude::*;
use bevy_ecs::world::DeferredWorld;
use bevy_reflect::prelude::ReflectDefault;

use crate::timer::{Delay, EffectTimer, Lifetime, TimerMergeMode};
pub use bevy_app::Startup;
use bevy_reflect::{Reflect, reflect_trait};
pub use bevy_status_effects_macros::StatusEffect;

pub struct StatusEffectPlugin;

impl Plugin for StatusEffectPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<EffectMode>()
            .register_type::<Effecting>()
            .register_type::<EffectedBy>()
            .register_type::<Lifetime>()
            .register_type::<Delay>()
            .register_type::<TimerMergeMode>()
            .add_systems(
                PreUpdate,
                (timer::despawn_finished_lifetimes, timer::tick_delay).chain(),
            );
    }
}

#[reflect_trait]
pub trait StatusEffect {}

/// Describes the logic used when multiple of the same effect are applied to the same entity.
#[derive(Component, Reflect, Eq, PartialEq, Debug, Default, Copy, Clone)]
#[reflect(Component, PartialEq, Debug, Default, Clone)]
pub enum EffectMode {
    #[default]
    Stack,
    Replace,
    // Todo
    // Merge,
}

pub fn init_effect_hook<T: Component + StatusEffect>(world: &mut World) {
    world
        .register_component_hooks::<T>()
        .on_add(effect_refresh_hook::<T>);
}

fn effect_refresh_hook<T: Component + StatusEffect>(
    mut world: DeferredWorld,
    context: HookContext,
) {
    let Some(mode) = world.get::<EffectMode>(context.entity).copied() else {
        return;
    };

    if mode == EffectMode::Stack {
        return;
    }

    let Some(target) = world.get::<Effecting>(context.entity) else {
        return;
    };

    let effected_by = match world.get::<EffectedBy>(target.0) {
        Some(e) => e.collection().clone(),
        None => return,
    };

    let old = effected_by.iter().find_map(|entity| {
        // `EffectedBy` not updated until later.
        assert_ne!(*entity, context.entity);

        let Some(other_mode) = world.get::<EffectMode>(*entity) else {
            return None;
        };

        if mode != *other_mode {
            return None;
        }

        world
            .get::<T>(*entity)
            .and_then(|effect| Some((*entity, effect)))
    });

    if let Some((old_entity, _old_effect)) = old {
        match mode {
            EffectMode::Stack => return,
            EffectMode::Replace => world.commands().entity(old_entity).despawn(),
        }

        if let Some(old_lifetime) = world.get::<Lifetime>(old_entity).cloned() {
            if let Some(mut lifetime) = world.get_mut::<Lifetime>(context.entity) {
                lifetime.merge(&old_lifetime)
            }
        }

        if let Some(old_delay) = world.get::<Delay>(old_entity).cloned() {
            if let Some(mut delay) = world.get_mut::<Delay>(context.entity) {
                delay.merge(&old_delay)
            }
        }
    }
}
