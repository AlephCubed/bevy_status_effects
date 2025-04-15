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

#[cfg(test)]
mod tests {
    use super::*;
    use crate as bevy_status_effects;
    use bevy_status_effects_macros::StatusEffect;
    use bevy_time::{Timer, TimerMode};
    use std::time::Duration;

    #[derive(StatusEffect, Component, Debug, Eq, PartialEq, Default)]
    struct MyEffect;

    #[test]
    fn stack() {
        let mut world = World::new();
        world
            .register_component_hooks::<MyEffect>()
            .on_add(effect_refresh_hook::<MyEffect>);

        let target = world.spawn_empty().id();
        let first = world.spawn((MyEffect, Effecting(target))).id();
        let second = world.spawn((MyEffect, Effecting(target))).id();

        world.flush();

        assert_eq!(world.get::<MyEffect>(first), Some(&MyEffect));
        assert_eq!(world.get::<MyEffect>(second), Some(&MyEffect));
    }

    #[test]
    fn refresh() {
        let mut world = World::new();
        world
            .register_component_hooks::<MyEffect>()
            .on_add(effect_refresh_hook::<MyEffect>);

        let target = world.spawn_empty().id();
        let first = world
            .spawn((MyEffect, Effecting(target), EffectMode::Replace))
            .id();
        let second = world
            .spawn((MyEffect, Effecting(target), EffectMode::Replace))
            .id();

        world.flush();

        assert_eq!(world.get::<MyEffect>(first), None);
        assert_eq!(world.get::<MyEffect>(second), Some(&MyEffect));
    }

    #[test]
    fn mixed() {
        let mut world = World::new();
        world
            .register_component_hooks::<MyEffect>()
            .on_add(effect_refresh_hook::<MyEffect>);

        let target = world.spawn_empty().id();

        let stack_1 = world.spawn((MyEffect, Effecting(target))).id();
        let stack_2 = world
            .spawn((MyEffect, Effecting(target), EffectMode::Stack))
            .id();

        let replace_1 = world
            .spawn((MyEffect, Effecting(target), EffectMode::Replace))
            .id();
        let replace_2 = world
            .spawn((MyEffect, Effecting(target), EffectMode::Replace))
            .id();

        world.flush();

        assert_eq!(world.get::<MyEffect>(stack_1), Some(&MyEffect));
        assert_eq!(world.get::<MyEffect>(stack_2), Some(&MyEffect));

        assert_eq!(world.get::<MyEffect>(replace_1), None);
        assert_eq!(world.get::<MyEffect>(replace_2), Some(&MyEffect));
    }

    #[test]
    fn timer_replace() {
        let mut world = World::new();
        world
            .register_component_hooks::<MyEffect>()
            .on_add(effect_refresh_hook::<MyEffect>);

        let target = world.spawn_empty().id();
        let second_lifetime = Lifetime::from_seconds(2.0).with_mode(TimerMergeMode::Replace);
        world.spawn((
            MyEffect,
            Effecting(target),
            EffectMode::Replace,
            Lifetime::from_seconds(1.0).with_mode(TimerMergeMode::Replace),
        ));
        let second = world
            .spawn((
                MyEffect,
                Effecting(target),
                EffectMode::Replace,
                second_lifetime.clone(),
            ))
            .id();

        world.flush();

        assert_eq!(world.get::<Lifetime>(second), Some(&second_lifetime));
    }

    #[test]
    fn timer_inherit() {
        let mut world = World::new();
        world
            .register_component_hooks::<MyEffect>()
            .on_add(effect_refresh_hook::<MyEffect>);

        let target = world.spawn_empty().id();
        let first_delay = Delay::from_seconds(1.0).with_mode(TimerMergeMode::Inherit);

        world.spawn((
            MyEffect,
            Effecting(target),
            EffectMode::Replace,
            first_delay.clone(),
        ));
        let second = world
            .spawn((
                MyEffect,
                Effecting(target),
                EffectMode::Replace,
                Delay::from_seconds(2.0).with_mode(TimerMergeMode::Inherit),
            ))
            .id();

        world.flush();

        assert_eq!(world.get::<Delay>(second), Some(&first_delay));
    }

    #[test]
    fn timer_fraction() {
        let mut world = World::new();
        world
            .register_component_hooks::<MyEffect>()
            .on_add(effect_refresh_hook::<MyEffect>);

        let target = world.spawn_empty().id();

        let mut first_timer = Timer::from_seconds(2.0, TimerMode::Once);
        first_timer.tick(Duration::from_secs_f32(1.0));

        world.spawn((
            MyEffect,
            Effecting(target),
            EffectMode::Replace,
            Delay {
                timer: first_timer,
                mode: TimerMergeMode::Fraction,
            },
        ));
        let second = world
            .spawn((
                MyEffect,
                Effecting(target),
                EffectMode::Replace,
                Delay::from_seconds(10.0).with_mode(TimerMergeMode::Fraction),
            ))
            .id();

        world.flush();

        let mut expected_timer = Timer::from_seconds(10.0, TimerMode::Repeating);
        expected_timer.tick(Duration::from_secs_f32(5.0));

        assert_eq!(
            world.get::<Delay>(second),
            Some(&Delay {
                timer: expected_timer,
                mode: TimerMergeMode::Fraction,
            })
        );
    }

    #[test]
    fn timer_max() {
        let mut world = World::new();
        world
            .register_component_hooks::<MyEffect>()
            .on_add(effect_refresh_hook::<MyEffect>);

        let target = world.spawn_empty().id();
        let max = Delay::from_seconds(3.0).with_mode(TimerMergeMode::Max);

        world.spawn((
            MyEffect,
            Effecting(target),
            EffectMode::Replace,
            Delay::from_seconds(1.0).with_mode(TimerMergeMode::Max),
        ));
        world.spawn((
            MyEffect,
            Effecting(target),
            EffectMode::Replace,
            max.clone(),
        ));
        let third = world
            .spawn((
                MyEffect,
                Effecting(target),
                EffectMode::Replace,
                Delay::from_seconds(2.0).with_mode(TimerMergeMode::Max),
            ))
            .id();

        world.flush();

        assert_eq!(world.get::<Delay>(third), Some(&max));
    }
}
