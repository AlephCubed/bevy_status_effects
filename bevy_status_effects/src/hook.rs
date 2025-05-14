use crate::relation::{EffectedBy, Effecting};
use crate::timer::{Delay, EffectTimer, Lifetime};
use crate::{EffectMode, StatusEffect};
use bevy_ecs::component::HookContext;
use bevy_ecs::prelude::{Component, RelationshipTarget, World};
use bevy_ecs::world::DeferredWorld;

/// A system that registers the effect hook for a given type.
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
