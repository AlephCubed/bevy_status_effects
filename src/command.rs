use crate::{Delay, EffectMode, EffectTimer, EffectedBy, Effecting, Lifetime};
use bevy_ecs::prelude::*;

pub struct AddEffect<B: Bundle> {
    pub entity: Entity,
    pub name: Name,
    pub bundle: B,
}

impl<B: Bundle> Command for AddEffect<B> {
    fn apply(self, world: &mut World) -> () {
        let Some(mode) = world.get::<EffectMode>(self.entity).copied() else {
            return;
        };

        if mode == EffectMode::Stack {
            return;
        }

        let Some(target) = world.get::<Effecting>(self.entity) else {
            return;
        };

        let effected_by = match world.get::<EffectedBy>(target.0) {
            Some(e) => e.collection().clone(),
            None => return,
        };

        if mode == EffectMode::Stack {
            world.spawn((Effecting(self.entity), self.name, self.bundle));
            return;
        }

        let old_entity = effected_by.iter().find_map(|entity| {
            // `EffectedBy` not updated until later.
            assert_ne!(*entity, self.entity);

            let Some(other_mode) = world.get::<EffectMode>(*entity) else {
                return None;
            };

            if mode != *other_mode {
                return None;
            }

            if let Some(name) = world.get::<Name>(*entity) {
                if name == &self.name {
                    return Some(*entity);
                }
            }

            None
        });

        let Some(old_entity) = old_entity else {
            world.spawn((Effecting(self.entity), self.name, self.bundle));
            return;
        };

        if mode == EffectMode::Replace {
            world.commands().entity(old_entity).despawn();
        };

        if let Some(old_lifetime) = world.get::<Lifetime>(old_entity).cloned() {
            if let Some(mut lifetime) = world.get_mut::<Lifetime>(self.entity) {
                lifetime.merge(&old_lifetime)
            }
        }

        if let Some(old_delay) = world.get::<Delay>(old_entity).cloned() {
            if let Some(mut delay) = world.get_mut::<Delay>(self.entity) {
                delay.merge(&old_delay)
            }
        }

        world.entity_mut(old_entity).insert(self.bundle);
        return;
    }
}

pub trait AddEffectExt {
    fn add_effect<B: Bundle>(&mut self, entity: Entity, name: Name, bundle: B) -> &mut Self;
}

impl AddEffectExt for Commands<'_, '_> {
    fn add_effect<B: Bundle>(&mut self, entity: Entity, name: Name, bundle: B) -> &mut Self {
        self.queue(AddEffect {
            entity,
            name,
            bundle,
        });
        self
    }
}
