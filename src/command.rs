use crate::{Delay, EffectMode, EffectTimer, EffectedBy, Effecting, Lifetime};
use bevy_ecs::prelude::*;

/// Applies an effect to a target entity.
/// This *might* spawn a new entity, depending on what effects are already applied to the target.
pub struct AddEffectCommand<B: Bundle> {
    /// The entity to apply the effect to.
    pub target: Entity,
    /// The effect to apply.
    pub bundle: EffectBundle<B>,
}

/// A "bundle" of components/settings used when applying an effect.
///
/// Not that this doesn't implement [`Bundle`] due to technical limitations.
#[derive(Default)]
pub struct EffectBundle<B: Bundle> {
    /// The name/ID of the effect. Effects with different IDs have no effect on one another.
    pub name: Name,
    /// Describes the logic used when new effect collides with an existing one.
    pub mode: EffectMode,
    /// The duration of the effect.
    #[doc(alias = "duration")]
    pub lifetime: Option<Lifetime>,
    /// Repeating timer used for the delay between effect applications.
    pub delay: Option<Delay>,
    /// Components that will be added to the effect. This is where the actual effect components get added.
    pub bundle: B,
}

fn insert_effect<B: Bundle>(mut entity: EntityWorldMut, effect: AddEffectCommand<B>) {
    entity.insert((
        Effecting(effect.target),
        effect.bundle.name,
        effect.bundle.mode,
        effect.bundle.bundle,
    ));

    if let Some(lifetime) = effect.bundle.lifetime {
        entity.insert(lifetime);
    }

    if let Some(delay) = effect.bundle.delay {
        entity.insert(delay);
    }
}

fn spawn_effect<B: Bundle>(world: &mut World, effect: AddEffectCommand<B>) {
    insert_effect(world.spawn(()), effect);
}

impl<B: Bundle> Command for AddEffectCommand<B> {
    fn apply(mut self, world: &mut World) -> () {
        if self.bundle.mode == EffectMode::Stack {
            spawn_effect(world, self);
            return;
        }

        let Some(effected_by) = world
            .get::<EffectedBy>(self.target)
            .map(|e| e.collection().clone())
        else {
            spawn_effect(world, self);
            return;
        };

        // Find previous entity that is:
        // 1. effecting the same target,
        // 2. and has the same name (ID).
        let old_entity = effected_by.iter().find_map(|entity| {
            let Some(other_mode) = world.get::<EffectMode>(*entity) else {
                return None;
            };

            // Todo Think more about.
            if self.bundle.mode != *other_mode {
                return None;
            }

            if let Some(name) = world.get::<Name>(*entity) {
                if name == &self.bundle.name {
                    return Some(*entity);
                }
            }

            None
        });

        let Some(old_entity) = old_entity else {
            spawn_effect(world, self);
            return;
        };

        if self.bundle.mode == EffectMode::Merge {
            if let Some(lifetime) = &mut self.bundle.lifetime {
                if let Some(old_lifetime) = world.get::<Lifetime>(old_entity).cloned() {
                    lifetime.merge(&old_lifetime)
                }
            }

            if let Some(delay) = &mut self.bundle.delay {
                if let Some(old_delay) = world.get::<Delay>(old_entity).cloned() {
                    delay.merge(&old_delay)
                }
            }
        }

        insert_effect(world.entity_mut(old_entity), self);
    }
}

/// An extension trait for adding effect methods to [`Commands`].
pub trait AddEffectExt {
    /// Applies an effect to a target entity.
    /// This *might* spawn a new entity, depending on what effects are already applied to the target.
    fn add_effect<B: Bundle>(&mut self, target: Entity, bundle: EffectBundle<B>) -> &mut Self;
}

impl AddEffectExt for Commands<'_, '_> {
    fn add_effect<B: Bundle>(&mut self, target: Entity, bundle: EffectBundle<B>) -> &mut Self {
        self.queue(AddEffectCommand { target, bundle });
        self
    }
}
