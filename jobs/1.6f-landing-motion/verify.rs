// Use the existing fixture reader and exact state assertions.
include!("../../engine/law/tests/replay.rs");

#[path = "../1.6g-terrain-corpus/terrain.rs"]
mod terrain;

const TICKS: usize = 420;
type Fields<'a> = Vec<&'a str>;
type Bits = [u64; 3];

fn csv(text: &str) -> Vec<Fields<'_>> {
    text.lines().map(|line| line.split(',').collect()).collect()
}

fn bits(fields: &[&str]) -> Bits {
    let [x, y, z] = fields else { panic!("missing vector: {fields:?}") };
    [csv_hex(x, 16), csv_hex(y, 16), csv_hex(z, 16)]
}

fn csv_hex(raw: &str, width: usize) -> u64 {
    assert_eq!(raw.len(), width, "malformed raw bits: {raw}");
    u64::from_str_radix(raw, 16).unwrap()
}

fn object_bits(object: &Object<'_>) -> Bits {
    ["xBits", "yBits", "zBits"].map(|field| hex(object.required(field), 16))
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Packet<'a> {
    ordinal: u64,
    recipient: &'a str,
    entity: &'a str,
    vector: Bits,
}

#[derive(Clone, Debug)]
struct Application<'a> {
    packet: Packet<'a>,
    tick: usize,
    player_tick: i64,
    in_tick: bool,
    before: Bits,
    after: Bits,
}

struct Landing<'a> {
    move_ordinal: u64,
    movement: &'a [&'a str],
    damage: (i64, i32, u32),
    consume: usize,
}

fn server<'a>(
    events: &'a [Fields<'a>], wire: &'a [Fields<'a>], run: &str, rate: &str, edges: Option<bool>,
) -> Result<(&'a str, &'a str, Vec<Packet<'a>>, Vec<Landing<'a>>), String> {
    let mut previous = (0, 0);
    for (i, row) in events.iter().enumerate() {
        let [order, world, player, _, ..] = row.as_slice() else {
            return Err(format!("incomplete server event {i}: {row:?}"));
        };
        let clock = (world.parse::<i64>().unwrap(), player.parse::<i32>().unwrap());
        if order.parse::<usize>().unwrap() != i || clock.0 < previous.0 || clock.1 < previous.1 {
            return Err(format!("server event order at {i}: {row:?}"));
        }
        previous = clock;
    }
    let [start, chain @ .., end] = events else { return Err("missing server fences".into()) };
    let [_, _, _, "start", start_run, recipient, entity, start_rate] = start.as_slice() else {
        return Err(format!("invalid server start: {start:?}"));
    };
    if *start_run != run || start_rate.parse::<f32>().unwrap() != rate.parse::<f32>().unwrap() ||
        end[3..] != ["end", run, recipient, entity, "-1"] {
        return Err(format!("server fence, rate or pending mark: {start:?} / {end:?}"));
    }
    if let Some(edges) = edges {
        if chain.len() != 4 * usize::from(edges) {
            return Err(format!("missing or competing server event: {chain:?}"));
        }
    }
    let mut landings = Vec::new();
    for group in chain.chunks(4) {
        let [damage, mark, consume, dispatch] = group else {
            return Err(format!("incomplete server landing chain: {group:?}"));
        };
        let [_, dw, dp, "damage", "minecraft:fall", amount, floor, ordinal, movement @ ..] = damage.as_slice() else {
            return Err(format!("unsupported server event or incomplete fall: {damage:?}"));
        };
        // The archive requires stone. Corpus cases use the observed base fallOn hook.
        if *floor == "null" || !floor.starts_with("minecraft:") || (edges.is_some() && *floor != "minecraft:stone") ||
            !matches!(movement, ["true", "true" | "false", "true", _, _, _]) {
            return Err(format!("missing base fallOn or accepted landing position: {damage:?}"));
        }
        let [mark_id, mw, mp, "mark", "minecraft:fall", "false", "true", "true", marked @ ..] = mark.as_slice()
        else { return Err(format!("missing unique fall transition: {mark:?}")) };
        let [consume_id, _, _, "consume", cause, "false", emitted_id, emitted @ ..] = consume.as_slice()
        else { return Err(format!("missing hurtMarked consume: {consume:?}")) };
        if dw != mw || dp != mp || cause != mark_id || emitted_id != entity ||
            bits(marked) != bits(emitted) || consume[1..3] != dispatch[1..3] ||
            dispatch[3..] != ["dispatch", consume_id, recipient, entity] {
            return Err(format!("ambiguous consume or dispatch: {group:?}"));
        }
        let raw = csv_hex(amount, 8) as u32;
        let move_ordinal = ordinal.parse().unwrap();
        if !(f32::from_bits(raw).is_finite() && f32::from_bits(raw) > 0.0) ||
            move_ordinal == 0 || landings.last().is_some_and(|last: &Landing<'_>| last.move_ordinal >= move_ordinal) {
            return Err(format!("invalid effective damage or accepted move order: {damage:?}"));
        }
        landings.push(Landing { move_ordinal, movement, damage: (dw.parse().unwrap(), dp.parse().unwrap(), raw),
            consume: consume_id.parse().unwrap() });
    }
    let [start, packets @ .., end] = wire else { return Err("missing wire fences".into()) };
    if start.as_slice() != ["0", "start", run, recipient, entity] ||
        end[1..] != ["end", run, recipient, entity] ||
        end[0].parse::<usize>().unwrap() != packets.len() + 1 {
        return Err(format!("wire fence identity or order: {start:?} / {end:?}"));
    }
    if packets.len() != landings.len() {
        return Err(format!("missing or extra own-motion wire packet: {packets:?}"));
    }
    let mut stream = Vec::new();
    for (index, (row, landing)) in packets.iter().zip(&landings).enumerate() {
        let [order, "motion", ordinal, player, own_id, decoded_id, cause, bytes, payload @ ..] = row.as_slice()
        else { return Err(format!("unproven motion on wire: {row:?}")) };
        let expected = index + 1;
        if order.parse::<usize>().ok() != Some(expected) || ordinal.parse::<usize>().ok() != Some(expected) ||
            cause.parse::<usize>().ok() != Some(landing.consume) {
            return Err(format!("unproven motion on wire: {row:?}"));
        }
        if player != recipient || own_id != entity || decoded_id != entity ||
            bytes.is_empty() || bytes.len() % 2 != 0 || !bytes.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(format!("wire recipient, entity or bytes: {row:?}"));
        }
        stream.push(Packet { ordinal: expected as u64, recipient, entity, vector: bits(payload) });
    }
    Ok((recipient, entity, stream, landings))
}

fn authorize(
    wire: &[Packet<'_>], received: &[Application<'_>], rows: &[Object<'_>],
    initial_tick: i64, landing_ticks: &[usize],
) -> Result<(), String> {
    if wire.len() != received.len() {
        return Err(format!("wire/receive count: {} sends, {} receives", wire.len(), received.len()));
    }
    if wire.len() != landing_ticks.len() {
        return Err("accepted move ordinal missing".into());
    }
    let mut previous = None;
    for ((sent, receipt), &landing) in wire.iter().zip(received).zip(landing_ticks) {
        if *sent != receipt.packet {
            return Err(format!("wire/receive identity, order or payload: {sent:?} / {receipt:?}"));
        }
        if receipt.in_tick || receipt.tick >= rows.len() || receipt.player_tick != initial_tick + receipt.tick as i64 ||
            landing >= receipt.tick || previous.is_some_and(|tick| tick >= receipt.tick) {
            return Err(format!("unrepresentable application boundary: {receipt:?}; landing {landing}"));
        }
        previous = Some(receipt.tick);
        if receipt.after != receipt.packet.vector {
            return Err(format!("lerpMotion did not install decoded payload: {receipt:?}"));
        }
    }
    for (tick, row) in rows.iter().enumerate() {
        let raw = row.get("anomaly");
        match (raw, received.iter().find(|event| event.tick == tick)) {
            (None, None) => {}
            (Some(raw), Some(event)) => {
                let packets = array(raw);
                let [packet] = packets.as_slice() else { return Err(format!("tick {tick}: competing anomalies {raw}")) };
                let packet = Object::parse(packet);
                if packet.string("type") != "ClientboundSetEntityMotionPacket" ||
                    packet.required("id") != event.packet.entity ||
                    object_bits(&packet.object("velocity")) != event.packet.vector {
                    return Err(format!("tick {tick}: unknown correction {raw}"));
                }
            }
            _ => return Err(format!("tick {tick}: missing or unknown correction {raw:?}")),
        }
    }
    Ok(())
}

fn verify(file: &Path, rate: &str, ticks: usize, archive: Option<(bool, &[Object<'_>])>, prefix: usize,
    terrain: Option<&terrain::Terrain>) -> f64 {
    let edges = archive.map(|(edges, _)| edges);
    let run = file.file_stem().unwrap().to_str().unwrap();
    println!("Verifying {run} at configured {rate} TPS");
    let read = |extension| fs::read_to_string(file.with_extension(extension))
        .unwrap_or_else(|e| panic!("{run}: {extension}: {e}"));
    let text = read("jsonl");
    let mut lines = text.lines();
    let header = Object::parse(lines.next().unwrap());
    recording_header(&header);
    let rows: Vec<_> = lines.map(Object::parse).collect();
    let (trailer, rows) = rows.split_last().unwrap();
    assert_eq!(trailer.string("type"), "end");
    assert!(trailer.boolean("complete"));
    assert_eq!(trailer.number("ticks"), ticks as f64);
    assert_eq!(rows.len(), ticks);
    let input_text = read("inputs.jsonl");
    let mut controls = input_text.lines();
    let start = Object::parse(controls.next().expect("missing input start"));
    let controls: Vec<_> = controls.map(Object::parse).collect();
    assert_eq!(controls.len(), ticks);
    if let Some((_, archive)) = archive { assert_eq!(archive.len(), ticks); }
    for (t, (row, expected)) in rows.iter().zip(&controls).enumerate() {
        let observed = input(&row.object("inputs"));
        assert_eq!(row.required("t").parse::<usize>().unwrap(), t);
        assert_eq!(expected.required("t").parse::<usize>().unwrap(), t);
        assert_eq!(observed, input(expected), "applied input tick {t}");
        if let Some((true, archive)) = archive {
            assert_eq!(observed, input(&archive[t].object("inputs")), "original edges input tick {t}");
        }
        if edges.is_none() {
            let expected = input(expected);
            let applied = row.object("inputs");
            assert_eq!(applied.f32("yaw").to_bits(), expected.yaw.to_bits(), "applied yaw tick {t}");
            assert_eq!(applied.f32("pitch").to_bits(), expected.pitch.to_bits(), "applied pitch tick {t}");
        }
    }
    let initial = header.object("initialPlayer");
    let initial_tick: i64 = initial.required("tickCount").parse().unwrap();
    let after = read("world.jsonl");
    let (world_text, bounds) = if let Some(terrain) = terrain {
        let (world, bounds) = terrain.world(&after, &initial, rows);
        (world, Some(bounds))
    } else { (after.as_str(), None) };
    let (world, mut state, _) = parse_world(&format!("{world_text}\n{}", header.required("initialPlayer")));
    let client_start = state.clone();
    if terrain.is_some() {
        let expected = numbers(start.required("start"));
        assert_eq!(expected.len(), 5);
        assert_eq!([state.position.x.to_bits(), state.position.y.to_bits(), state.position.z.to_bits()],
            [expected[0].to_bits(), expected[1].to_bits(), expected[2].to_bits()], "settled start position");
        assert_eq!([state.yaw.to_bits(), state.pitch.to_bits()], [(expected[3] as f32).to_bits(), (expected[4] as f32).to_bits()],
            "settled start angles");
        assert_eq!(state.ticks_frozen, 0, "frozen start");
    }
    assert_eq!(world.difficulty, 2, "normal difficulty");
    assert!(world.natural_regeneration);
    assert!(state.on_ground && state.velocity.horizontal_sqr() == 0.0, "unsettled start");
    if edges == Some(true) { assert_eq!(state.position, Vec3::new(5648.5, -57.0, 4100.5)); }

    let trace = read("server-motion.csv");
    let (events, wire) = trace.strip_prefix("landing-v1\n").unwrap().split_once("wire\n").unwrap();
    let events = csv(events);
    let wire = csv(wire);
    let (recipient, entity, packets, landings) = server(&events, &wire, run, rate, edges)
        .unwrap_or_else(|failure| panic!("{run}: {failure}"));

    let client = read("client-motion.csv");
    let client = csv(&client);
    let [start, stream @ .., end] = client.as_slice() else { panic!("missing client fences") };
    assert_eq!(start.as_slice(), ["start", run, recipient, entity]);
    assert_eq!(end.as_slice(), ["end", run, recipient, entity]);
    let stream: Vec<_> = stream.iter().filter(|row| {
        if let Some(bounds) = bounds {
            match row.as_slice() {
                ["block-update", tick, x, y, z] => {
                    assert!(!bounds.contains([x, y, z].map(|v| v.parse().unwrap())),
                        "tick {tick}: terrain changed in the replay footprint: {row:?}");
                    return false;
                }
                ["chunk-update", tick, x, z] => {
                    assert!(!bounds.contains_chunk(x.parse().unwrap(), z.parse().unwrap()),
                        "tick {tick}: replay chunk changed: {row:?}");
                    return false;
                }
                ["unsupported", tick, kind] => panic!("tick {tick}: unsupported client event {kind}"),
                _ => {}
            }
        }
        true
    }).collect();
    let [begin, middle @ .., finished] = stream.as_slice() else { panic!("missing client replay fences") };
    let fast = if rate != "20" { "true" } else { "false" };
    let ["begin", begin_run, clock, begin_rate, follow, full_demand] = begin.as_slice()
        else { panic!("missing client replay start: {begin:?}") };
    assert_eq!((*begin_run, clock.parse::<i64>().unwrap(), begin_rate.parse::<f32>().unwrap(), *follow, *full_demand),
        (run, initial_tick, rate.parse::<f32>().unwrap(), fast, fast));
    let ["finished", name, count, "true", elapsed_ns] = finished.as_slice()
        else { panic!("missing complete client replay: {finished:?}") };
    assert_eq!(*name, run);
    assert_eq!(count.parse::<usize>().unwrap(), ticks);
    let elapsed_ns = elapsed_ns.parse::<u64>().unwrap();
    assert!(elapsed_ns > 0);
    let measured = ticks as f64 * 1e9 / elapsed_ns as f64;
    if rate == "200" { assert!(measured > 100.0, "{run}: lab clock inactive, {measured} client TPS"); }
    if rate == "2000" { assert!(measured > 1000.0, "{run}: 100x lab clock did not execute, {measured} client TPS"); }

    let mut previous_move = (0, None);
    let mut last_application = None;
    let mut landing_ticks = Vec::new();
    let mut received = Vec::new();
    for row in middle {
        match row.as_slice() {
            ["move", ordinal, tick, player_tick, movement @ ..] => {
                let [ground, horizontal, position, _, _, _] = movement else { panic!("incomplete move: {row:?}") };
                let ordinal = ordinal.parse::<u64>().unwrap();
                let tick = tick.parse::<usize>().unwrap();
                assert!(ordinal > previous_move.0 && previous_move.1.is_none_or(|last| tick > last), "move order: {row:?}");
                assert!(last_application.is_none_or(|application| tick >= application), "move precedes application: {row:?}");
                previous_move = (ordinal, Some(tick));
                let observed = &rows[tick];
                assert_eq!(observed.required("tickCount"), *player_tick, "client move clock");
                assert_eq!(observed.required("onGround"), *ground, "client move ground");
                assert_eq!(observed.required("horizontalCollision"), *horizontal, "client move collision");
                if *position == "true" {
                    assert_eq!(bits(&movement[3..]), object_bits(&observed.object("position")), "client move position");
                }
                if let Some(landing) = landings.get(landing_ticks.len()) {
                    if ordinal == landing.move_ordinal {
                        assert_eq!(movement, landing.movement, "accepted landing payload");
                        assert!(tick > 0 && !rows[tick - 1].boolean("onGround") && observed.boolean("onGround"),
                            "accepted move did not follow an airborne client tick");
                        landing_ticks.push(tick);
                    }
                }
            }
            ["motion", ordinal, recipient, entity, tick, clock, in_tick, vectors @ ..] => {
                assert_eq!(vectors.len(), 9, "missing lerpMotion invocation: {row:?}");
                let tick = tick.parse::<usize>().unwrap();
                assert!(previous_move.1.is_none_or(|movement| movement < tick), "application precedes move: {row:?}");
                last_application = Some(tick);
                received.push(Application {
                    packet: Packet { ordinal: ordinal.parse().unwrap(), recipient, entity, vector: bits(&vectors[..3]) },
                    tick, player_tick: clock.parse().unwrap(), in_tick: in_tick.parse().unwrap(),
                    before: bits(&vectors[3..6]), after: bits(&vectors[6..]),
                });
            }
            _ => panic!("unexpected client witness row {row:?}"),
        }
    }
    assert_eq!(landing_ticks.len(), landings.len(), "accepted move ordinal missing");
    authorize(&packets, &received, rows, initial_tick, &landing_ticks).unwrap_or_else(|e| panic!("{run}: {e}"));

    let movement_clock = Instant::now();
    for (t, row) in rows.iter().take(prefix).enumerate() {
        let tick = t as u64;
        assert_eq!(row.required("tickCount").parse::<i64>().unwrap(), initial_tick + t as i64 + 1);
        if let Some(event) = received.iter().find(|event| event.tick == t) {
            let Vec3 { x, y, z } = state.velocity;
            assert_eq!([x.to_bits(), y.to_bits(), z.to_bits()], event.before, "tick {tick}: pre-packet velocity");
            let [x, y, z] = event.packet.vector.map(f64::from_bits);
            state.velocity = Vec3::new(x, y, z);
        }
        if terrain.is_some() {
            assert!(state.health > 0.0 && state.food_level > 6, "tick {tick}: unsupported replay resource eligibility");
            let modifiers = row.get("startSpeedModifiers").unwrap_or_else(|| panic!("tick {tick}: missing START speed modifiers"));
            assert_eq!(modifiers, row.required("speedModifiers"), "tick {tick}: unsupported in-tick speed modifier change");
            state.speed_modifiers = speed_modifiers_from(modifiers);
            state.ticks_frozen = row.get("startTicksFrozen").unwrap_or_else(|| panic!("tick {tick}: missing START freeze counter"))
                .parse().unwrap();
        } else {
            state.speed_modifiers = speed_modifiers(row);
            state.ticks_frozen = row.required("ticksFrozen").parse().unwrap();
        }
        let input = input(&row.object("inputs"));
        let collision = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| tick_with_collision(&mut state, &world, &input)))
            .unwrap_or_else(|failure| {
                eprintln!("{run}: first unsupported movement boundary at tick {tick}, input {input:?}");
                std::panic::resume_unwind(failure)
            });
        let steps = array(row.required("collisionSteps"));
        assert_eq!(steps.len(), usize::from(collision.is_some()), "tick {tick}: collision step count");
        if let Some((requested, resolved)) = collision {
            let step = Object::parse(steps[0]);
            assert_vector(requested, &step.object("requested"), "collision request", tick);
            assert_vector(resolved, &step.object("result"), "collision result", tick);
        }
        for flag in ["minorHorizontalCollision", "verticalCollisionBelow", "swimming", "crouching"] {
            assert!(row.get(flag).is_some(), "tick {tick}: missing {flag}");
        }
        assert_tick(&state, row, tick);
        if terrain.is_some() {
            assert_eq!(state.ticks_frozen, row.required("ticksFrozen").parse::<i32>().unwrap(), "tick {tick}: END freeze counter");
            assert!(row.get("speedBits").is_some(), "tick {tick}: missing END speed");
            for (flag, predicted) in [("underwater", state.underwater), ("eyeInWater", state.eye_in_water),
                ("inPowderSnow", state.in_powder_snow), ("wasInPowderSnow", state.was_in_powder_snow)] {
                assert!(row.get(flag).is_some(), "tick {tick}: missing {flag}");
                assert_eq!(predicted, row.boolean(flag), "tick {tick}: {flag}");
            }
        }
    }
    if terrain.is_some() && prefix == ticks {
        println!("{run}: offline movement TPS {:.2};", ticks as f64 / movement_clock.elapsed().as_secs_f64());
    }
    if prefix < ticks {
        println!("{run}: reduction only, {prefix} exact movement rows");
        return measured;
    }
    let damage: Vec<_> = landings.iter().map(|landing| landing.damage).collect();
    let resources = read("resources.csv");
    if edges.is_some() {
        server_resources::verify_landing_resources(&resources, run, &world, damage.first().copied());
    } else {
        let clock = |row: &[&str]| (row[1].parse().unwrap(), row[2].parse().unwrap());
        server_resources::verify_corpus_resources(&resources, run, (recipient, entity), &world, &client_start,
            &damage, [clock(&events[0]), clock(events.last().unwrap())]);
        if terrain.is_some_and(|terrain| terrain.mixed) {
            terrain::mixed_contacts(&after, &initial, rows);
        }
    }
    println!("{run}: {ticks} exact movement rows; client TPS {measured:.2}; landings {landing_ticks:?}; motion {received:?}");
    if edges == Some(true) {
        let mut extra = received.clone();
        let mut duplicate = extra[0].clone();
        duplicate.packet.ordinal += 1;
        extra.push(duplicate);
        let failure = authorize(&packets, &extra, rows, initial_tick, &landing_ticks).unwrap_err();
        assert!(failure.starts_with("wire/receive count"), "{failure}");
        println!("identical extra receive refused: {failure}; planted {:?}", extra.last().unwrap());

        let mut shifted = received.clone();
        shifted[0].tick += 1;
        let failure = authorize(&packets, &shifted, rows, initial_tick, &landing_ticks).unwrap_err();
        assert!(failure.starts_with("unrepresentable application boundary"), "{failure}");
        println!("shifted application refused: {failure}");

        let mut reordered = events.clone();
        reordered.swap(2, 3);
        reordered[2][0] = "2";
        reordered[3][0] = "3";
        let Err(failure) = server(&reordered, &wire, run, rate, Some(true)) else { panic!("reordered witness survived") };
        assert!(failure.starts_with("missing unique fall transition") || failure.starts_with("server event order"), "{failure}");
        println!("reordered server witness refused: {failure}");
    }
    measured
}

#[test]
fn live_landing_motion() {
    let archive = fs::read_to_string(std::env::var_os("L0_EDGES_20").expect("original edges archive")).unwrap();
    let archive: Vec<_> = archive.lines().skip(1).take(TICKS).map(Object::parse).collect();
    for (var, rate) in [("L0_LANDING_20", "20"), ("L0_LANDING_200", "200")] {
        let directory = PathBuf::from(std::env::var_os(var).expect(var));
        let mut tps: Vec<_> = ["edges", "full-blocks", "ice"].into_iter().map(|course| {
            let suffix = format!("_{course}_1.jsonl");
            let candidates: Vec<_> = fs::read_dir(&directory).unwrap().map(|entry| entry.unwrap().path())
                .filter(|path| path.file_name().unwrap().to_string_lossy().ends_with(&suffix)).collect();
            let [file] = candidates.as_slice() else { panic!("{var}: expected one {course} capture: {candidates:?}") };
            verify(file, rate, TICKS, Some((course == "edges", &archive)), TICKS, None)
        }).collect();
        tps.sort_by(f64::total_cmp);
        println!("{rate} configured TPS, client TPS n=3 min={:.2} median={:.2} p90/p99/max={:.2}", tps[0], tps[1], tps[2]);
    }
}

#[test]
fn offline_landing_refusals() {
    let zero = "0000000000000000";
    let trace = format!(concat!(
        "0,10,10,start,run,player,7,20.0\n",
        "1,11,10,damage,minecraft:fall,3f800000,minecraft:stone,9,true,false,true,{0},{0},{0}\n",
        "2,11,10,mark,minecraft:fall,false,true,true,{0},{0},{0}\n",
        "3,11,10,consume,2,false,7,{0},{0},{0}\n",
        "4,11,10,dispatch,3,player,7\n",
        "5,12,12,end,run,player,7,-1\n",
    ), zero);
    let encoded = format!("0,start,run,player,7\n1,motion,1,player,7,7,3,0700,{zero},{zero},{zero}\n2,end,run,player,7\n");
    let events = csv(&trace);
    let wire = csv(&encoded);
    let (_, _, packets, _) = server(&events, &wire, "run", "20", Some(true)).unwrap();
    let application = Application {
        packet: packets[0].clone(), tick: 1, player_tick: 101, in_tick: false, before: [0; 3], after: [0; 3],
    };
    let raw = format!(
        r#"{{"anomaly":[{{"type":"ClientboundSetEntityMotionPacket","id":7,"velocity":{{"xBits":"{zero}","yBits":"{zero}","zBits":"{zero}"}}}}]}}"#
    );
    let rows = [Object::parse("{}"), Object::parse(&raw)];
    authorize(&packets, std::slice::from_ref(&application), &rows, 100, &[0]).unwrap();

    let mut extra = application.clone();
    extra.packet.ordinal = 2;
    assert!(authorize(&packets, &[application.clone(), extra], &rows, 100, &[0])
        .unwrap_err().starts_with("wire/receive count"));
    let shifted = Application { tick: 2, ..application.clone() };
    let extended = [Object::parse("{}"), Object::parse(&raw), Object::parse("{}")];
    assert!(authorize(&packets, &[shifted], &extended, 100, &[0])
        .unwrap_err().starts_with("unrepresentable application boundary"));
    let teleport = Object::parse(r#"{"anomaly":[{"type":"ClientboundPlayerPositionPacket"}]}"#);
    assert!(authorize(&packets, &[application], &[Object::parse("{}"), teleport], 100, &[0])
        .unwrap_err().contains("unknown correction"));

    let mut events = events;
    events.swap(2, 3);
    events[2][0] = "2";
    events[3][0] = "3";
    assert!(matches!(server(&events, &wire, "run", "20", Some(true)), Err(e) if e.starts_with("missing unique fall transition")));
    let mut wire = wire;
    wire[1][6] = "-1";
    assert!(matches!(server(&csv(&trace), &wire, "run", "20", Some(true)), Err(e) if e.starts_with("unproven motion")));
}

#[test]
fn corpus_requires_a_distinct_chain_for_each_motion() {
    let zero = "0000000000000000";
    let trace = format!(concat!(
        "0,10,10,start,run,player,7,20.0\n",
        "1,11,10,damage,minecraft:fall,3f800000,minecraft:stone,9,true,false,true,{0},{0},{0}\n",
        "2,11,10,mark,minecraft:fall,false,true,true,{0},{0},{0}\n",
        "3,11,10,consume,2,false,7,{0},{0},{0}\n",
        "4,11,10,dispatch,3,player,7\n",
        "5,12,11,damage,minecraft:fall,40000000,minecraft:stone_slab,19,true,false,true,{0},{0},{0}\n",
        "6,12,11,mark,minecraft:fall,false,true,true,{0},{0},{0}\n",
        "7,12,11,consume,6,false,7,{0},{0},{0}\n",
        "8,12,11,dispatch,7,player,7\n",
        "9,13,13,end,run,player,7,-1\n",
    ), zero);
    let encoded = format!(concat!("0,start,run,player,7\n",
        "1,motion,1,player,7,7,3,0700,{0},{0},{0}\n",
        "2,motion,2,player,7,7,7,0700,{0},{0},{0}\n",
        "3,end,run,player,7\n",
    ), zero);
    let events = csv(&trace);
    let mut wire = csv(&encoded);
    let (_, _, packets, landings) = server(&events, &wire, "run", "20", None).unwrap();
    assert_eq!(packets.iter().map(|p| p.ordinal).collect::<Vec<_>>(), [1, 2]);
    assert_eq!(landings.iter().map(|p| p.move_ordinal).collect::<Vec<_>>(), [9, 19]);
    wire[2][6] = "3";
    assert!(matches!(server(&events, &wire, "run", "20", None), Err(e) if e.starts_with("unproven motion")));
}
