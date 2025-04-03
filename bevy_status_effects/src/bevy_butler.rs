#[cfg(test)]
mod tests {
    use crate as bevy_status_effects;
    use crate::relation::Effecting;
    use bevy_app::App;
    use bevy_butler::butler_plugin;
    use bevy_ecs::prelude::Component;
    use bevy_status_effects_macros::StatusEffect;

    #[butler_plugin]
    struct ButlerPlugin;

    #[derive(StatusEffect, Component, Debug, Eq, PartialEq, Default)]
    #[add_component(plugin = ButlerPlugin)]
    struct Derive;
    
    #[derive(StatusEffect, Component, Debug, Eq, PartialEq, Default)]
    #[effect_type(Refresh)]
    #[add_component(plugin = ButlerPlugin)]
    struct DeriveRefresh;

    #[test]
    fn derive_refresh() {
        let mut app = App::new();
        app.add_plugins(ButlerPlugin);
        app.update();

        let target = app.world_mut().spawn_empty().id();
        let first = app
            .world_mut()
            .spawn((DeriveRefresh, Effecting(target)))
            .id();
        let second = app
            .world_mut()
            .spawn((DeriveRefresh, Effecting(target)))
            .id();

        app.world_mut().flush();

        assert_eq!(app.world().get::<DeriveRefresh>(first), None);
        assert_eq!(
            app.world().get::<DeriveRefresh>(second),
            Some(&DeriveRefresh)
        );
    }
}
