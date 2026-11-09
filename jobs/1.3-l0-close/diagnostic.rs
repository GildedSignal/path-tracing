// Reduction only: inject the independently logged START state into the immutable
// step-ups recording, and compare every original output plus logged END speed.
include!("../../engine/law/tests/replay.rs");

fn observation(log: &str, phase: &str, tick: u64, player_tick: u64) -> (bool, bool, u32) {
    let key = format!("{phase} t={tick} playerTick={player_tick} ");
    let matches: Vec<_> = log.lines().filter_map(|line| line.split_once(&key).map(|(_, suffix)| suffix)).collect();
    let [suffix] = matches.as_slice() else { panic!("missing/ambiguous diagnostic {key}") };
    let field = |name: &str| suffix.split_whitespace().find_map(|part| part.strip_prefix(name)).unwrap();
    (field("sprint=") == "true", field("modifier=") == "true", u32::from_str_radix(field("speedBits="), 16).unwrap())
}

#[test]
fn recorded_sprint_diagnostic() {
    let base = Path::new("D:/path-tracing-target/sprint-diagnostic-source-20261107T231317/lab/gates/results/20261107T2213532561408Z-fixtures/l0_20261107T2213532561408Z_step-ups_1");
    let log = fs::read_to_string(base.parent().unwrap().join("bot-latest.log")).unwrap();
    let recording = fs::read_to_string(base.with_extension("jsonl")).unwrap();
    let mut lines = recording.lines();
    let header = Object::parse(lines.next().unwrap());
    let world = fs::read_to_string(base.with_extension("world.jsonl")).unwrap();
    let (world, mut state, _) = parse_world(&format!("{world}\n{}", header.required("initialPlayer")));
    let legacy = std::env::var_os("L0_DIAGNOSTIC_DERIVE_MODIFIER").is_some();
    let mut count = 0;
    for line in lines {
        let row = Object::parse(line);
        if row.get("type").is_some() {
            assert!(row.boolean("complete"));
            assert_eq!(row.number("ticks"), count as f64);
            continue;
        }
        let tick = row.number("t") as u64;
        assert_eq!(tick, count);
        assert!(row.get("anomaly").is_none());
        let player_tick = row.number("tickCount") as u64;
        let (flag, modifier, _) = observation(&log, "recorder-start", tick, player_tick - 1);
        state.sprinting = flag;
        state.sprint_speed_modifier = if legacy { flag } else { modifier };
        state.speed_modifiers = speed_modifiers(&row);
        state.ticks_frozen = row.required("ticksFrozen").parse().unwrap();
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
        let (flag, modifier, speed) = observation(&log, "recorder-end", tick, player_tick);
        assert_eq!(state.sprinting, flag, "tick {tick}: logged sprint flag");
        assert_eq!(state.speed.to_bits(), speed, "tick {tick}: logged speedBits");
        assert_eq!(state.sprint_speed_modifier, modifier, "tick {tick}: logged sprint modifier");
        count += 1;
    }
    assert_eq!(count, 420);
    println!("Step-ups seed 17: {count} exact movement/collision rows and logged END speed/modifier observations; diagnostic reduction only.");
}
