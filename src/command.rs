use crate::{Delay, EffectMode, EffectedBy, Effecting, Lifetime};
use bevy_ecs::prelude::*;

pub struct AddEffect<B: Bundle> {
    pub target: Entity,
    pub bundle: EffectBundle<B>,
}

#[derive(Default)]
pub struct EffectBundle<B: Bundle> {
    pub name: Name,
    pub mode: EffectMode,
    pub lifetime: Option<Lifetime>,
    pub delay: Option<Delay>,
    pub bundle: B,
}

fn insert_effect<B: Bundle>(mut world: EntityWorldMut, effect: AddEffect<B>) {
    world.insert((
        Effecting(effect.target),
        effect.bundle.name,
        effect.bundle.mode,
        effect.bundle.bundle,
    ));
}

fn spawn_effect<B: Bundle>(world: &mut World, effect: AddEffect<B>) {
    insert_effect(world.spawn(()), effect);
}

impl<B: Bundle> Command for AddEffect<B> {
    fn apply(self, world: &mut World) -> () {
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

        // Todo
        // if self.mode == EffectMode::Merge {
        //     if let Some(old_lifetime) = world.get::<Lifetime>(old_entity).cloned() {
        //         if let Some(mut lifetime) = world.get_mut::<Lifetime>(self.target) {
        //             lifetime.merge(&old_lifetime)
        //         }
        //     }
        //
        //     if let Some(old_delay) = world.get::<Delay>(old_entity).cloned() {
        //         if let Some(mut delay) = world.get_mut::<Delay>(self.target) {
        //             delay.merge(&old_delay)
        //         }
        //     }
        // }

        insert_effect(world.entity_mut(old_entity), self);
    }
}

pub trait AddEffectExt {
    fn add_effect<B: Bundle>(&mut self, target: Entity, bundle: EffectBundle<B>) -> &mut Self;
}

impl AddEffectExt for Commands<'_, '_> {
    fn add_effect<B: Bundle>(&mut self, target: Entity, bundle: EffectBundle<B>) -> &mut Self {
        self.queue(AddEffect { target, bundle });
        self
    }
}
