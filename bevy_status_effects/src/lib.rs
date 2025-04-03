#[cfg(feature = "bevy_butler")]
pub mod bevy_butler;

pub mod relation;
pub mod timer;

use crate::relation::{EffectedBy, Effecting};
use bevy_app::{App, Plugin, PreUpdate};
use bevy_asset::Handle;
use bevy_ecs::component::HookContext;
use bevy_ecs::prelude::*;
use bevy_ecs::world::DeferredWorld;
use bevy_image::Image;

pub use bevy_app::Startup;

pub struct StatusEffectPlugin;

impl Plugin for StatusEffectPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (timer::despawn_finished_lifetimes, timer::tick_delay).chain(),
        );
    }
}

/// The icon of a status effect.
pub struct Icon(pub Handle<Image>);

pub trait StatusEffect {
    const TYPE: EffectType = EffectType::Stack;
}

/// Describes the logic used when multiple of the same effect are applied to the same entity.
#[derive(Eq, PartialEq, Default, Debug)]
pub enum EffectType {
    #[default]
    Stack,
    Refresh,
}

pub fn init_effect_hook<T: Component + StatusEffect>(world: &mut World) {
    if T::TYPE == EffectType::Stack {
        return;
    }

    world
        .register_component_hooks::<T>()
        .on_add(effect_refresh_hook::<T>);
}

fn effect_refresh_hook<T: Component + StatusEffect>(
    mut world: DeferredWorld,
    context: HookContext,
) {
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
    struct StackDefault;

    #[test]
    fn default() {
        assert_eq!(StackDefault::TYPE, EffectType::Stack);
    }

    #[derive(StatusEffect, Component, Debug, Eq, PartialEq, Default)]
    #[effect_type(Refresh)]
    struct RefreshOverride;

    #[test]
    fn overridden() {
        assert_eq!(RefreshOverride::TYPE, EffectType::Refresh);
    }

    #[test]
    fn refresh_hook() {
        let mut world = World::new();
        world
            .register_component_hooks::<RefreshOverride>()
            .on_add(effect_refresh_hook::<RefreshOverride>);

        let target = world.spawn_empty().id();
        let first = world.spawn((RefreshOverride, Effecting(target))).id();
        let second = world.spawn((RefreshOverride, Effecting(target))).id();

        world.flush();

        assert_eq!(world.get::<RefreshOverride>(first), None);
        assert_eq!(world.get::<RefreshOverride>(second), Some(&RefreshOverride));
    }
}
