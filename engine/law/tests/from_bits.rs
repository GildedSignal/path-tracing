use law::{PlayerState, Pose, SpeedModifier, SpeedOperation, Vec3};
#[test]
fn raw_bits_and_options_round_trip() {
    for state in [PlayerState::default(), PlayerState {
        pose: Pose::Swimming, yaw: f32::from_bits(0x7fc01234),
        position: Vec3::new(f64::from_bits(0x7ff8000000001234), -0.0, 3.0),
        movement_this_tick: Some(Vec3::new(1.0, 2.0, 3.0)),
        main_supporting_block: Some([-1, i32::MIN, i32::MAX]),
        use_item_speed_multiplier: Some(f32::from_bits(0xffc01234)), use_item_can_sprint: Some(false),
        speed_modifiers: vec![SpeedModifier { id: "minecraft:é".into(), amount: f64::NAN, operation: SpeedOperation::AddMultipliedTotal }],
        ..PlayerState::default()
    }] { let bits = state.bits(); assert_eq!(PlayerState::from_bits(&bits).unwrap().bits(), bits); }
}
#[test]
fn refuses_invalid_values_and_lengths() {
    let base = PlayerState::default().bits();
    for (at, value, reason) in [(1, u64::MAX, "f32"), (5, 3, "pose"), (6, 0x80000000, "i32"), (9, u64::MAX, "u32"), (13, 2, "boolean")] {
        let mut bits = base.clone(); bits[at] = value;
        let err = PlayerState::from_bits(&bits).unwrap_err(); assert_eq!((err.position, err.reason), (at, reason));
    }
    let err = PlayerState::from_bits(&base[..5]).unwrap_err(); assert_eq!((err.position, err.reason), (5, "missing word"));
    let mut bits = base.clone(); bits.push(0); let err = PlayerState::from_bits(&bits).unwrap_err(); assert_eq!((err.position, err.reason), (base.len(), "extra word"));
    let mut bits = base; *bits.last_mut().unwrap() = 1; assert_eq!(PlayerState::from_bits(&bits).unwrap_err().reason, "modifier count");
    let mut bits = PlayerState { speed_modifiers: vec![SpeedModifier { id: "a".into(), amount: 1.0, operation: SpeedOperation::AddValue }], ..PlayerState::default() }.bits();
    let n = bits.len(); bits[n-4] = 100; assert_eq!(PlayerState::from_bits(&bits).unwrap_err().reason, "id length");
    bits[n-4] = 1; bits[n-3] = 256; assert_eq!(PlayerState::from_bits(&bits).unwrap_err().reason, "id byte");
    bits[n-3] = 255; assert_eq!(PlayerState::from_bits(&bits).unwrap_err().reason, "id utf8");
    bits[n-3] = 97; bits[n-1] = 3; assert_eq!(PlayerState::from_bits(&bits).unwrap_err().reason, "operation");
}
