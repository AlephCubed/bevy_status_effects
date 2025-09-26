use bevy_ecs::prelude::*;
use bevy_status_effects::*;
use bevy_time::*;
use std::time::Duration;

#[derive(Component, Debug, Eq, PartialEq, Default)]
struct MyEffect(u8);

#[test]
fn stack() {
    let mut world = World::new();

    let target = world.spawn_empty().id();

    world
        .commands()
        .add_effect(target, "MyEffect".into(), MyEffect(0));
    world
        .commands()
        .add_effect(target, "MyEffect".into(), MyEffect(1));

    world.flush();

    let effects: Vec<u8> = world
        .query::<&MyEffect>()
        .iter(&mut world)
        .map(|c| c.0)
        .collect();

    assert!(effects.contains(&0));
    assert!(effects.contains(&1));
}

#[test]
fn replace() {
    let mut world = World::new();

    let target = world.spawn_empty().id();

    world.commands().add_effect(
        target,
        "MyEffect".into(),
        (MyEffect(0), EffectMode::Replace),
    );
    world.commands().add_effect(
        target,
        "MyEffect".into(),
        (MyEffect(1), EffectMode::Replace),
    );

    world.flush();

    let effects: Vec<u8> = world
        .query::<&MyEffect>()
        .iter(&mut world)
        .map(|c| c.0)
        .collect();

    assert!(!effects.contains(&0));
    assert!(effects.contains(&1));
}

#[test]
fn mixed() {
    let mut world = World::new();

    let target = world.spawn_empty().id();

    world
        .commands()
        .add_effect(target, "MyEffect".into(), MyEffect(0));
    world
        .commands()
        .add_effect(target, "MyEffect".into(), (MyEffect(1), EffectMode::Stack));

    world.commands().add_effect(
        target,
        "MyEffect".into(),
        (MyEffect(2), EffectMode::Replace),
    );
    world.commands().add_effect(
        target,
        "MyEffect".into(),
        (MyEffect(3), EffectMode::Replace),
    );

    world.flush();

    let effects: Vec<u8> = world
        .query::<&MyEffect>()
        .iter(&mut world)
        .map(|c| c.0)
        .collect();

    assert!(effects.contains(&0));
    assert!(effects.contains(&1));
    assert!(!effects.contains(&2));
    assert!(effects.contains(&3));
}

#[test]
fn timer_merge_replace() {
    let mut world = World::new();

    let target = world.spawn_empty().id();
    let second_lifetime = Lifetime::from_seconds(2.0).with_mode(TimerMergeMode::Replace);
    world.commands().add_effect(
        target,
        "MyEffect".into(),
        (
            MyEffect(0),
            EffectMode::Merge,
            Lifetime::from_seconds(1.0).with_mode(TimerMergeMode::Replace),
        ),
    );
    world.commands().add_effect(
        target,
        "MyEffect".into(),
        (MyEffect(1), EffectMode::Merge, second_lifetime.clone()),
    );

    world.flush();

    assert_eq!(
        world.query::<&Lifetime>().single(&world).unwrap(),
        &second_lifetime
    );
}

#[test]
fn timer_merge_inherit() {
    let mut world = World::new();

    let target = world.spawn_empty().id();
    let first_delay = Delay::from_seconds(1.0).with_mode(TimerMergeMode::Inherit);
    world.commands().add_effect(
        target,
        "MyEffect".into(),
        (MyEffect(0), EffectMode::Merge, first_delay.clone()),
    );
    world.commands().add_effect(
        target,
        "MyEffect".into(),
        (
            MyEffect(1),
            EffectMode::Merge,
            Delay::from_seconds(2.0).with_mode(TimerMergeMode::Inherit),
        ),
    );

    world.flush();

    assert_eq!(
        world.query::<&Delay>().single(&world).unwrap(),
        &first_delay
    );
}

#[test]
fn timer_merge_fraction() {
    let mut world = World::new();

    let target = world.spawn_empty().id();
    let mut first_timer = Timer::from_seconds(2.0, TimerMode::Once);
    first_timer.tick(Duration::from_secs_f32(1.0));

    world.commands().add_effect(
        target,
        "MyEffect".into(),
        (
            MyEffect(0),
            EffectMode::Merge,
            Delay {
                timer: first_timer,
                mode: TimerMergeMode::Fraction,
            },
        ),
    );
    world.commands().add_effect(
        target,
        "MyEffect".into(),
        (
            MyEffect(1),
            EffectMode::Merge,
            Delay::from_seconds(10.0).with_mode(TimerMergeMode::Fraction),
        ),
    );

    world.flush();

    let mut expected_timer = Timer::from_seconds(10.0, TimerMode::Repeating);
    expected_timer.tick(Duration::from_secs_f32(5.0));

    assert_eq!(
        world.query::<&Delay>().single(&world).unwrap(),
        &Delay {
            timer: expected_timer,
            mode: TimerMergeMode::Fraction,
        }
    );
}

#[test]
fn timer_merge_max() {
    let mut world = World::new();

    let target = world.spawn_empty().id();
    let max = Delay::from_seconds(3.0).with_mode(TimerMergeMode::Max);

    world.commands().add_effect(
        target,
        "MyEffect".into(),
        (
            MyEffect(0),
            EffectMode::Merge,
            Delay::from_seconds(1.0).with_mode(TimerMergeMode::Max),
        ),
    );
    world.commands().add_effect(
        target,
        "MyEffect".into(),
        (MyEffect(1), EffectMode::Merge, max.clone()),
    );
    world.commands().add_effect(
        target,
        "MyEffect".into(),
        (
            MyEffect(2),
            EffectMode::Merge,
            Delay::from_seconds(2.0).with_mode(TimerMergeMode::Max),
        ),
    );

    world.flush();

    assert_eq!(world.query::<&Delay>().single(&world).unwrap(), &max);
}
