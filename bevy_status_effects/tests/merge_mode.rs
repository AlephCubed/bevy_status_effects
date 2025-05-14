use bevy_ecs::prelude::*;
use bevy_status_effects::*;
use bevy_time::*;
use std::time::Duration;

#[derive(StatusEffect, Component, Debug, Eq, PartialEq, Default)]
struct MyEffect;

#[test]
fn stack() {
    let mut world = World::new();
    init_effect_hook::<MyEffect>(&mut world);

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
    init_effect_hook::<MyEffect>(&mut world);

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
    init_effect_hook::<MyEffect>(&mut world);

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
    init_effect_hook::<MyEffect>(&mut world);

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
    init_effect_hook::<MyEffect>(&mut world);

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
    init_effect_hook::<MyEffect>(&mut world);

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
    init_effect_hook::<MyEffect>(&mut world);

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
