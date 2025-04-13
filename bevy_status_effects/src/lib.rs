#[cfg(feature = "bevy_butler")]
pub mod bevy_butler;

pub mod relation;
pub mod timer;

use crate::relation::{EffectedBy, Effecting};
use bevy_app::{App, Plugin, PreUpdate};
use bevy_ecs::component::HookContext;
use bevy_ecs::prelude::*;
use bevy_ecs::world::DeferredWorld;
use bevy_reflect::prelude::ReflectDefault;

use crate::timer::{Delay, Lifetime};
pub use bevy_app::Startup;
use bevy_reflect::Reflect;
pub use bevy_status_effects_macros::StatusEffect;

pub struct StatusEffectPlugin;

impl Plugin for StatusEffectPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<EffectMode>()
            .register_type::<Effecting>()
            .register_type::<EffectedBy>()
            .register_type::<Lifetime>()
            .register_type::<Delay>()
            .add_systems(
                PreUpdate,
                (timer::despawn_finished_lifetimes, timer::tick_delay).chain(),
            );
    }
}

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
        None => return,
        Some(e) => e.collection().clone(),
    };

    for effect in effected_by {
        // `EffectedBy` not updated until later.
        assert_ne!(effect, context.entity);

        let Some(other_mode) = world.get::<EffectMode>(effect) else {
            continue;
        };

        if mode != *other_mode {
            continue;
        }

        if world.get::<T>(effect).is_some() {
            world.commands().entity(effect).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate as bevy_status_effects;
    use bevy_status_effects_macros::StatusEffect;

    #[derive(StatusEffect, Component, Debug, Eq, PartialEq, Default)]
    struct RefreshOverride;

    #[test]
    fn stack() {
        let mut world = World::new();
        world
            .register_component_hooks::<RefreshOverride>()
            .on_add(effect_refresh_hook::<RefreshOverride>);

        let target = world.spawn_empty().id();
        let first = world.spawn((RefreshOverride, Effecting(target))).id();
        let second = world.spawn((RefreshOverride, Effecting(target))).id();

        world.flush();

        assert_eq!(world.get::<RefreshOverride>(first), Some(&RefreshOverride));
        assert_eq!(world.get::<RefreshOverride>(second), Some(&RefreshOverride));
    }

    #[test]
    fn refresh() {
        let mut world = World::new();
        world
            .register_component_hooks::<RefreshOverride>()
            .on_add(effect_refresh_hook::<RefreshOverride>);

        let target = world.spawn_empty().id();
        let first = world
            .spawn((RefreshOverride, Effecting(target), EffectMode::Replace))
            .id();
        let second = world
            .spawn((RefreshOverride, Effecting(target), EffectMode::Replace))
            .id();

        world.flush();

        assert_eq!(world.get::<RefreshOverride>(first), None);
        assert_eq!(world.get::<RefreshOverride>(second), Some(&RefreshOverride));
    }

    #[test]
    fn mixed() {
        let mut world = World::new();
        world
            .register_component_hooks::<RefreshOverride>()
            .on_add(effect_refresh_hook::<RefreshOverride>);

        let target = world.spawn_empty().id();

        let stack_1 = world.spawn((RefreshOverride, Effecting(target))).id();
        let stack_2 = world
            .spawn((RefreshOverride, Effecting(target), EffectMode::Stack))
            .id();

        let replace_1 = world
            .spawn((RefreshOverride, Effecting(target), EffectMode::Replace))
            .id();
        let replace_2 = world
            .spawn((RefreshOverride, Effecting(target), EffectMode::Replace))
            .id();

        world.flush();

        assert_eq!(
            world.get::<RefreshOverride>(stack_1),
            Some(&RefreshOverride)
        );
        assert_eq!(
            world.get::<RefreshOverride>(stack_2),
            Some(&RefreshOverride)
        );

        assert_eq!(world.get::<RefreshOverride>(replace_1), None);
        assert_eq!(
            world.get::<RefreshOverride>(replace_2),
            Some(&RefreshOverride)
        );
    }
}
