use law::PlayerState;
#[test]
fn frozen_java_rust_schema_witness() {
    let raw = include_str!("../../../bot/tests/fixtures/native-state.hex");
    let bits: Vec<_> = raw.split_whitespace().map(|w|u64::from_str_radix(w,16).unwrap()).collect();
    let known = vec![true;bits.len()];
    let state = PlayerState::from_observed_bits(&bits,&known).unwrap();
    assert_eq!(state.bits(),bits);
    assert_eq!(state.t,123); assert_eq!(state.hurt_cooldown.remaining_ticks,-7);
    assert_eq!(state.speed_modifiers[0].id,"minecraft:é");
    for i in 0..bits.len() { let mut mask = known.clone(); mask[i]=false; assert_eq!(PlayerState::from_observed_bits(&bits,&mask).unwrap_err().position,i); }
}

#[test]
fn recorded_observation_preserves_unknown_fields_without_defaults() {
    let raw = include_str!("../../../bot/tests/fixtures/recorded-state.hex");
    let lines: Vec<Vec<u64>> = raw.lines().filter(|line|!line.trim().is_empty()).map(|line|line.split_whitespace().map(|w|u64::from_str_radix(w,16).unwrap()).collect()).collect();
    assert_eq!(lines.len(),3); assert_eq!(lines[0].len(),lines[1].len()); assert_eq!(lines[0].len(),lines[2].len());
    let state = PlayerState::from_bits(&lines[0]).unwrap();
    assert_eq!(state.bits(),lines[0]);
    assert_eq!(state.t,361); assert_eq!(state.position.x.to_bits(),0x40b0108000000000);
    let known: Vec<_> = lines[1].iter().map(|&v|v==1).collect();
    assert!(PlayerState::from_observed_bits(&lines[0],&known).is_err());
    for (known,provenance) in lines[1].iter().zip(&lines[2]) { assert_eq!(*known==0,*provenance==0); }
}
