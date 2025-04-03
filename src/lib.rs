pub mod relation;
pub mod timer;

use bevy_app::{App, Plugin, PreUpdate};
use bevy_asset::Handle;
use bevy_ecs::prelude::*;
use bevy_image::Image;

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
