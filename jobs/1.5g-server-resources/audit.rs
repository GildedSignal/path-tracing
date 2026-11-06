// Reuse the strict movement replay and its parsers without changing L0.
include!("../../engine/law/tests/replay.rs");

fn audit_pair(file: &Path, require_server_match: bool) {
    let fixture = fs::read_to_string(file.with_extension("world.jsonl")).unwrap();
    let recording = fs::read_to_string(file).unwrap();
    let (world, mut state, server_end) = parse_world(&fixture);
    let (ticks, _) = replay_text(&recording, &world, &mut state);
    let server_end = server_end.unwrap();
    let expected = Object::parse(&server_end);
    let offset = expected.number("serverTickOffset") as i64;
    assert!(expected.boolean("atRest") && offset >= 0);
    let idle = Input { yaw: state.yaw, pitch: state.pitch, ..Default::default() };
    for _ in 0..offset {
        tick(&mut state, &world, &idle);
    }
    println!("{}: {ticks} movement ticks matched, offset {offset}", file.display());
    for (name, predicted) in [
        ("health", state.health.to_bits()),
        ("foodLevel", state.food_level as u32),
        ("saturationLevel", state.saturation.to_bits()),
        ("exhaustion", state.exhaustion.to_bits()),
        ("foodTickTimer", state.food_tick_timer),
    ] {
        let server = match name {
            "foodLevel" | "foodTickTimer" => expected.number(name) as u32,
            _ => expected.f32(name).to_bits(),
        };
        let verdict = if predicted == server { "match" } else { "MISMATCH" };
        println!("  {name}: {predicted:08x} / {server:08x} {verdict}");
    }
    if require_server_match {
        assert_server(&state, &expected);
    }
}

#[test]
fn resource_audit() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../lab/fixtures");
    for course in ["corner", "full-blocks", "slime"] {
        audit_pair(&root.join(course).join(format!("{course}-1.jsonl")), false);
    }
    audit_pair(Path::new(&std::env::var("L0_NORMAL_RECORDING").unwrap()), true);
}

#[test]
fn peaceful_full_blocks_witness() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../lab/fixtures/full-blocks");
    let (world, start, end) = parse_world(&fs::read_to_string(root.join("full-blocks-1.world.jsonl")).unwrap());
    let recording = fs::read_to_string(root.join("full-blocks-1.jsonl")).unwrap();
    let rows: Vec<_> = recording.lines().skip(1).map(Object::parse)
        .take_while(|row| row.get("type").is_none()).collect();
    let end = end.unwrap();
    let end = Object::parse(&end);
    let clock = |row: &Object<'_>| row.number("gameTime") as i32;
    let first = clock(&rows[0]);
    let sample = clock(rows.last().unwrap()) + end.number("serverTickOffset") as i32;

    // Dry, unmounted packet path with accepted recorded positions and sprint commands.
    // A status or rotation packet without position contributes zero displacement.
    let mut position = start.position;
    let mut on_ground = start.on_ground;
    let mut reminder = 0;
    let mut cumulative_bound = [0.0_f64; 2];
    let second_origin = vector(&rows[25].object("position"));
    let charges: Vec<_> = rows.iter().enumerate().map(|(i, row)| {
        let reported = vector(&row.object("position"));
        let mut delta = reported.minus(position);
        reminder += 1;
        if delta.length_sqr() > 0.0002_f64.powi(2) || reminder == 20 {
            position = reported;
            reminder = 0;
        } else {
            delta = Vec3::default();
        }
        let grounded = row.boolean("onGround");
        let sprinting = row.boolean("isSprinting");
        let jump = if on_ground && !grounded && delta.y > 0.0 {
            if sprinting { 0.2_f32 } else { 0.05_f32 }
        } else {
            0.0
        };
        let sprint = if grounded && sprinting {
            0.1_f32 * math::round(delta.horizontal_sqr().sqrt() as f32 * 100.0) as f32 * 0.01_f32
        } else {
            0.0
        };
        on_ground = grounded;
        let (batch, origin) = if i <= 25 { (0, start.position) } else { (1, second_origin) };
        cumulative_bound[batch] = cumulative_bound[batch].max(position.minus(origin).length_sqr());
        (jump, sprint)
    }).collect();
    assert!(cumulative_bound.iter().all(|&d2| d2 < 100.0));

    let run = |batched: bool, phase: i32| {
        let mut state = start.clone();
        let mut depletion = Vec::new();
        let mut next = 0;
        for time in first..=sample {
            while next < rows.len() {
                let accepted = if batched { if next <= 25 { sample - 1 } else { sample } }
                    else { clock(&rows[next]) };
                assert!(accepted >= clock(&rows[next]));
                if accepted != time { break; }
                let (jump, sprint) = charges[next];
                state.exhaustion = (state.exhaustion + jump).min(40.0);
                state.exhaustion = (state.exhaustion + sprint).min(40.0);
                next += 1;
            }
            let player_tick = 600 + time - (sample - 1) + phase;
            if state.exhaustion > 4.0 {
                depletion.push((next - 1, time, player_tick));
            }
            tick_server_resources(&mut state, &world, player_tick);
        }
        assert_eq!(next, rows.len());
        assert_eq!(depletion.len(), 1);
        assert_eq!(state.exhaustion.to_bits(), end.f32("exhaustion").to_bits());
        assert_eq!(state.health, end.f32("health"));
        assert_eq!(state.food_level as f64, end.number("foodLevel"));
        assert_eq!(state.food_tick_timer as f64, end.number("foodTickTimer"));
        (state.saturation, depletion[0])
    };

    for phase in 0..20 {
        assert_eq!(run(false, phase).0, 20.0);
    }
    let prompt = run(false, 0);
    let on_cadence = run(true, 0);
    let before_cadence = run(true, 19);
    assert_eq!((on_cadence.0, before_cadence.0), (19.0, 20.0));
    println!("depletion (client row, world tick, player tick), sample world tick {sample}");
    println!("  prompt {prompt:?}, all 20 phases end at 20");
    println!("  same batched order, two phases: {on_cadence:?}, {before_cadence:?}");
    println!("  batch maximum squared displacements: {cumulative_bound:?}");
}
