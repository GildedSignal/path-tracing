use law::*;
#[test]
fn apple_use_witness() {
    let mut world = BoxWorld::default();
    world.fill([-2, -1, -2], [2, -1, 5], Block::full());
    let mut state = PlayerState { using_item: true, use_item_speed_multiplier: Some(f32::from_bits(0x3e4ccccd)), use_item_can_sprint: Some(false), on_ground: true, velocity: Vec3::new(0.0, -0.0784000015258789, 0.0), ..Default::default() };
    for _ in 0..20 { tick(&mut state, &world, &Input { forward: true, ..Default::default() }); }
    assert_eq!(state.position.z.to_bits(), 0x3fe9f7f1008b3824);
}
