use law::{Block, BoxWorld, Input, PlayerState, Vec3, tick};
use world::use_effects;

#[test]
fn dumped_items_drive_witnesses_and_sprint_gate() {
    let mut world = BoxWorld::default();
    world.fill([-2, -1, -2], [2, -1, 8], Block::full());
    for (item, expected) in [("minecraft:apple", 0x3fe9f7f1008b3824), ("minecraft:iron_spear", 0x40103af6980f2207)] {
        let fx = use_effects(item).unwrap();
        let mut s = PlayerState { using_item: true, use_item_speed_multiplier: Some(fx.speed_multiplier), use_item_can_sprint: Some(fx.can_sprint), on_ground: true, velocity: Vec3::new(0.0, -0.0784000015258789, 0.0), ..Default::default() };
        for _ in 0..20 { tick(&mut s, &world, &Input { forward: true, ..Default::default() }); }
        assert_eq!(s.position.z.to_bits(), expected, "{item}");
        let mut s = PlayerState { using_item: true, use_item_speed_multiplier: Some(fx.speed_multiplier), use_item_can_sprint: Some(fx.can_sprint), on_ground: true, ..Default::default() };
        tick(&mut s, &world, &Input { forward: true, sprint: true, ..Default::default() });
        assert_eq!(s.sprinting, fx.can_sprint, "{item}");
    }
    assert!(use_effects("minecraft:not_an_item").is_none());
    let fx = use_effects("minecraft:apple").unwrap();
    let mut s = PlayerState { using_item: true, use_item_speed_multiplier: Some(fx.speed_multiplier), use_item_can_sprint: Some(fx.can_sprint), on_ground: true, ..Default::default() };
    s.set_sprinting(true);
    tick(&mut s, &world, &Input { forward: true, sprint: true, ..Default::default() });
    assert!(s.sprinting);
}
