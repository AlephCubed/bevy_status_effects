use bevy_ecs::prelude::{Component, Entity};

/// Stores the entity that is being effected by this status effect.
#[derive(Component)]
#[relationship(relationship_target = EffectedBy)]
pub struct Effecting(pub Entity);

/// Stores all the status effects that are effecting this entity.
#[derive(Component)]
#[relationship_target(relationship = Effecting, linked_spawn)]
pub struct EffectedBy(Vec<Entity>);

impl<'a> IntoIterator for &'a EffectedBy {
    type Item = <Self::IntoIter as Iterator>::Item;

    type IntoIter = std::slice::Iter<'a, Entity>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}
