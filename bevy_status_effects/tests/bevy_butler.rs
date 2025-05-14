#![cfg(feature = "bevy_butler")]

use bevy_app::App;
use bevy_butler::butler_plugin;
use bevy_ecs::prelude::Component;
use bevy_status_effects::*;

#[butler_plugin]
struct ButlerPlugin;

#[derive(StatusEffect, Component, Debug, Eq, PartialEq, Default)]
#[add_component(plugin = ButlerPlugin)]
struct Derive;

#[test]
fn derive_refresh() {
    let mut app = App::new();
    app.add_plugins(ButlerPlugin);
    app.update();

    let target = app.world_mut().spawn_empty().id();
    let first = app
        .world_mut()
        .spawn((Derive, Effecting(target), EffectMode::Replace))
        .id();
    let second = app
        .world_mut()
        .spawn((Derive, Effecting(target), EffectMode::Replace))
        .id();

    app.world_mut().flush();

    assert_eq!(app.world().get::<Derive>(first), None);
    assert_eq!(app.world().get::<Derive>(second), Some(&Derive));
}

/// Same as above, but with `plugin(...)` syntax as apposed to `plugin = ...`
#[derive(StatusEffect, Component, Debug, Eq, PartialEq, Default)]
#[add_component(plugin(ButlerPlugin))]
struct AlternateSyntax;

#[test]
fn derive_refresh_alternate_syntax() {
    let mut app = App::new();
    app.add_plugins(ButlerPlugin);
    app.update();

    let target = app.world_mut().spawn_empty().id();
    let first = app
        .world_mut()
        .spawn((AlternateSyntax, Effecting(target), EffectMode::Replace))
        .id();
    let second = app
        .world_mut()
        .spawn((AlternateSyntax, Effecting(target), EffectMode::Replace))
        .id();

    app.world_mut().flush();

    assert_eq!(app.world().get::<AlternateSyntax>(first), None);
    assert_eq!(
        app.world().get::<AlternateSyntax>(second),
        Some(&AlternateSyntax)
    );
}
