// Reduction only: replay the immutable ceilings seed 28 capture with its recorded START inputs.
// With L0_POSE_SYNC_TICK=n, the pose before tick n is STANDING, the value the server's lagging
// entity-data packet would carry (ClientPacketListener.handleSetEntityData -> Entity.onSyncedDataUpdated
// -> refreshDimensions). Without it, the law keeps its own END pose as the next START pose.
include!("../../engine/law/tests/replay.rs");

#[test]
fn recorded_pose_diagnostic() {
    let base = Path::new("D:/path-tracing-evidence/l0-close-20261109-b/captures/20261109T2123027060630Z-fixtures/l0_20261109T2123027060630Z_ceilings_4");
    let sync: Option<u64> = std::env::var("L0_POSE_SYNC_TICK").ok().map(|v| v.parse().unwrap());
    let recording = fs::read_to_string(base.with_extension("jsonl")).unwrap();
    let mut lines = recording.lines();
    let header = Object::parse(lines.next().unwrap());
    let world = fs::read_to_string(base.with_extension("world.jsonl")).unwrap();
    let (world, mut state, _) = parse_world(&format!("{world}\n{}", header.required("initialPlayer")));
    let mut count = 0;
    for line in lines {
        let row = Object::parse(line);
        if row.get("type").is_some() { continue; }
        let tick = row.number("t") as u64;
        assert_eq!(tick, count);
        state.sprinting = row.boolean("startSprinting");
        state.sprint_speed_modifier = row.boolean("startSprintSpeedModifier");
        state.speed_modifiers = speed_modifiers_from(row.required("startSpeedModifiers"));
        state.ticks_frozen = row.required("startTicksFrozen").parse().unwrap();
        if sync == Some(tick) { state.pose = Pose::Standing; }
        let collision = tick_with_collision(&mut state, &world, &input(&row.object("inputs")));
        let steps = array(row.required("collisionSteps"));
        assert_eq!(steps.len(), usize::from(collision.is_some()), "tick {tick}: collision count");
        if let Some((request, result)) = collision {
            let step = Object::parse(steps[0]);
            for (label, actual, expected) in [("request", request, step.object("requested")), ("result", result, step.object("result"))] {
                for (axis, value) in [("x", actual.x), ("y", actual.y), ("z", actual.z)] {
                    assert_eq!(value.to_bits(), expected.f64(axis).to_bits(), "tick {tick}: collision {label}.{axis}");
                }
            }
        }
        assert_tick(&state, &row, tick);
        count += 1;
    }
    println!("Ceilings seed 28: {count} exact movement/collision rows (pose sync tick {sync:?}); diagnostic reduction only.");
}
