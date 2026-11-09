// Reuse the L0 reader and assertions. This diagnostic cannot publish a fixture.
include!("../../engine/law/tests/replay.rs");

fn audit_edges(file: &Path) {
    let recording = fs::read_to_string(file).unwrap();
    let fixture = fs::read_to_string(file.with_extension("world.jsonl")).unwrap();
    let mut lines = recording.lines();
    let header = Object::parse(lines.next().unwrap());
    recording_header(&header);
    // A rejected run retains the raw export, without the driver's player row.
    let player = header.required("initialPlayer");
    let companion = format!("{fixture}\n{player}");
    let (world, mut state, _) = parse_world(&companion);
    let stem = file.file_stem().unwrap().to_str().unwrap();
    let snapshot = |boundary| {
        fs::read_to_string(file.with_file_name(format!("{stem}-{boundary}.server.json"))).unwrap()
    };
    let start = snapshot("settled");
    let start = Object::parse(&start);
    state.health = start.f32("Health");
    state.food_level = start.number("foodLevel") as i32;
    state.saturation = start.f32("foodSaturationLevel");
    state.exhaustion = start.f32("foodExhaustionLevel");
    state.food_tick_timer = start.number("foodTickTimer") as u32;
    let rows: Vec<_> = lines.map(Object::parse).collect();
    let (trailer, rows) = rows.split_last().unwrap();
    assert_eq!(trailer.string("type"), "end");
    assert!(trailer.boolean("complete"));
    assert_eq!(trailer.number("ticks"), rows.len() as f64);
    println!("{}", file.display());
    let mut applied_motion = None;
    for (index, row) in rows.iter().enumerate() {
        let t = index as u64;
        assert_eq!(row.number("t"), t as f64);
        state.speed_modifiers = speed_modifiers(row);
        state.ticks_frozen = row.required("ticksFrozen").parse().unwrap();
        let applied = input(&row.object("inputs"));
        if t == 174 {
            let packets = array(row.required("anomaly"));
            assert_eq!(packets.len(), 1);
            let packet = Object::parse(packets[0]);
            assert_eq!(packet.string("type"), "ClientboundSetEntityMotionPacket");
            let velocity = vector(&packet.object("velocity"));
            assert_eq!((velocity.x.to_bits(), velocity.z.to_bits()), (0, 0));
            // Illustrative inputs to the scale-one branch of 26.2 LpVec3.
            // The old recording cannot identify the actual server encoder input.
            let packet_y = rows[..index].iter().filter_map(|candidate| {
                let y = candidate.object("velocity").f64("y");
                if !(3.051944088384301e-5..=1.0).contains(&y.abs()) {
                    return None;
                }
                let packed = ((y * 0.5 + 0.5) * 32766.0).round();
                let decoded = packed * 2.0 / 32766.0 - 1.0;
                (decoded.to_bits() == velocity.y.to_bits()).then_some((candidate.number("t") as u64, y))
            }).next_back().expect("a compatible velocity in the earlier trajectory");
            println!("  latest compatible earlier client Y: t={} y={:?}", packet_y.0, packet_y.1);
            let steps = array(row.required("collisionSteps"));
            assert_eq!(steps.len(), 1);
            let step = Object::parse(steps[0]);
            assert_eq!(hex(step.object("requested").required("yBits"), 16), velocity.y.to_bits());
            assert_eq!(hex(step.object("result").required("yBits"), 16), 0);
            let mut without_packet = state.clone();
            tick(&mut without_packet, &world, &applied);
            let position = vector(&row.object("position"));
            assert_ne!(without_packet.position.x.to_bits(), position.x.to_bits());
            println!(
                "  counterfactual t={t} x={:?} vx={:?}, packet={velocity:?}",
                without_packet.position.x, without_packet.velocity.x
            );
            state.velocity = velocity;
            applied_motion = Some(velocity);
        } else {
            assert!(row.get("anomaly").is_none(), "unmodeled observation at {t}");
        }
        if t == 175 {
            let mut wrong_tick = state.clone();
            wrong_tick.velocity = applied_motion.unwrap();
            tick(&mut wrong_tick, &world, &applied);
            assert_ne!(wrong_tick.position.x.to_bits(), row.object("position").f64("x").to_bits());
            println!("  planted duplicate at t={t} would produce x={:?}", wrong_tick.position.x);
        }
        tick(&mut state, &world, &applied);
        assert_tick(&state, row, t);
        if (172..=175).contains(&t) {
            println!(
                "  t={t} position={:?} velocity={:?} health(client)={} health(local estimate)={}",
                state.position, state.velocity, row.f32("health"), state.health
            );
        }
    }
    println!("  {} movement ticks match in the diagnostic", rows.len());

    let end = snapshot("end");
    let end = Object::parse(&end);
    let last = rows.last().unwrap();
    let offset = end.number("gameTimeBefore") as i64 - last.number("gameTime") as i64;
    let offset_max = end.number("gameTimeAfter") as i64 - last.number("gameTime") as i64;
    assert!(state.on_ground && state.velocity.x == 0.0 && state.velocity.z == 0.0);
    println!("  resource sample offset bracket [{offset},{offset_max}]");
    if offset == offset_max && offset >= 0 {
        let idle = Input { yaw: state.yaw, pitch: state.pitch, ..Input::default() };
        for _ in 0..offset {
            tick(&mut state, &world, &idle);
        }
        println!(
            "  coupled resource estimate = ({}, {}, {:08x}, {:08x}, {})",
            state.health, state.food_level, state.saturation.to_bits(),
            state.exhaustion.to_bits(), state.food_tick_timer
        );
    }
    println!(
        "  independent RCON end = ({}, {}, {:08x}, {:08x}, {})",
        end.f32("Health"), end.number("foodLevel"), end.f32("foodSaturationLevel").to_bits(),
        end.f32("foodExhaustionLevel").to_bits(), end.number("foodTickTimer")
    );
}

#[test]
fn edges_motion_effect() {
    for variable in ["L0_EDGES_20", "L0_EDGES_200"] {
        audit_edges(Path::new(&std::env::var(variable).expect(variable)));
    }
}
