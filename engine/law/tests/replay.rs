#[path = "support/json.rs"]
mod json;

use json::{Object, array, hex, numbers};
use law::*;
use std::{fs, path::{Path, PathBuf}, time::{Duration, Instant}};

fn recordings(root: &Path, output: &mut Vec<PathBuf>) {
    if !root.exists() { return; }
    for entry in fs::read_dir(root).expect("fixtures directory") {
        let path = entry.unwrap().path();
        if path.is_dir() {
            recordings(&path, output);
        } else if path.extension().is_some_and(|e| e == "jsonl") && !path.to_str().unwrap().ends_with(".world.jsonl") {
            output.push(path);
        }
    }
}

fn vector(object: &Object<'_>) -> Vec3 {
    Vec3::new(object.f64("x"), object.f64("y"), object.f64("z"))
}

fn boxes(value: &str) -> Vec<Aabb> {
    numbers(value).chunks(6).map(|values| {
        assert_eq!(values.len(), 6);
        Aabb::new(values[0], values[1], values[2], values[3], values[4], values[5])
    }).collect()
}

fn position(value: &str) -> BlockPos {
    let values = numbers(value);
    assert_eq!(values.len(), 3);
    std::array::from_fn(|axis| {
        assert_eq!(values[axis], values[axis] as i32 as f64);
        values[axis] as i32
    })
}

fn input(object: &Object<'_>) -> Input {
    Input {
        forward: object.boolean("forward"), back: object.boolean("back"),
        left: object.boolean("left"), right: object.boolean("right"),
        jump: object.boolean("jump"), sneak: object.boolean("sneak"), sprint: object.boolean("sprint"),
        yaw: object.f32("yaw"), pitch: object.f32("pitch"),
    }
}

fn pose(value: &str) -> Pose {
    match value {
        "STANDING" => Pose::Standing,
        "CROUCHING" => Pose::Crouching,
        "SWIMMING" => Pose::Swimming,
        value => panic!("unsupported pose {value}"),
    }
}

fn speed_modifiers(object: &Object<'_>) -> Vec<SpeedModifier> {
    let mut modifiers: Vec<SpeedModifier> = Vec::new();
    for tuple in array(object.required("speedModifiers")) {
        let fields = array(tuple);
        assert_eq!(fields.len(), 3, "speedModifiers needs [id, amount, operation]");
        let id = fields[0].strip_prefix('"').and_then(|s| s.strip_suffix('"'))
            .expect("speedModifiers id string");
        assert_ne!(id, "minecraft:sprinting", "speedModifiers must exclude sprint");
        assert!(!modifiers.iter().any(|m| m.id == id), "duplicate speedModifiers id {id}");
        let amount: f64 = fields[1].parse().expect("speedModifiers amount");
        assert!(amount.is_finite(), "non-finite speedModifiers amount");
        let operation = match fields[2].trim_matches('"') {
            "ADD_VALUE" | "add_value" | "0" => SpeedOperation::AddValue,
            "ADD_MULTIPLIED_BASE" | "add_multiplied_base" | "1" => SpeedOperation::AddMultipliedBase,
            "ADD_MULTIPLIED_TOTAL" | "add_multiplied_total" | "2" => SpeedOperation::AddMultipliedTotal,
            value => panic!("unsupported speedModifiers operation {value}"),
        };
        modifiers.push(SpeedModifier { id: id.to_owned(), amount, operation });
    }
    modifiers
}

fn load_state(object: &Object<'_>) -> PlayerState {
    let mut state = PlayerState::default();
    state.position = vector(&object.object("position"));
    state.velocity = vector(&object.object("velocity"));
    state.sprinting = object.boolean("isSprinting");
    state.movement_speed_base = object.f64("movementSpeedBase");
    state.speed_modifiers = speed_modifiers(object);
    if object.get("inputs").is_some() { state.input = input(&object.object("inputs")); }
    macro_rules! flags {
        ($($key:literal => $field:ident),*) => { $(
            if object.get($key).is_some() { state.$field = object.boolean($key); }
        )* };
    }
    macro_rules! floats {
        ($($key:literal => $field:ident),*) => { $(
            if object.get($key).is_some() || object.get(concat!($key, "Bits")).is_some() {
                state.$field = object.f32($key);
            }
        )* };
    }
    macro_rules! doubles {
        ($($key:literal => $field:ident),*) => { $(
            if object.get($key).is_some() || object.get(concat!($key, "Bits")).is_some() {
                state.$field = object.f64($key);
            }
        )* };
    }
    flags!("onGround" => on_ground, "horizontalCollision" => horizontal_collision,
        "verticalCollision" => vertical_collision, "verticalCollisionBelow" => vertical_collision_below,
        "minorHorizontalCollision" => minor_horizontal_collision,
        "isShiftKeyDown" => shift_key_down, "isInWater" => in_water, "isInLava" => in_lava,
        "onClimbable" => on_climbable, "crouching" => crouching, "swimming" => swimming,
        "underwater" => underwater, "eyeInWater" => eye_in_water, "wasEyeInWater" => was_eye_in_water,
        "leatherBoots" => leather_boots, "wasInPowderSnow" => was_in_powder_snow,
        "inPowderSnow" => in_powder_snow, "onGroundNoBlocks" => on_ground_no_blocks,
"noPhysics" => no_physics);
    floats!("yRot" => yaw, "xRot" => pitch, "health" => health, "saturationLevel" => saturation,
        "exhaustion" => exhaustion, "speed" => speed, "stepHeight" => step_height,
        "frictionModifier" => friction_modifier, "airDragModifier" => air_drag_modifier,
        "waterMovementEfficiency" => water_movement_efficiency, "movementEfficiency" => movement_efficiency,
        "sneakingSpeed" => sneaking_speed);
    doubles!("fallDistance" => fall_distance, "waterHeight" => water_height, "lavaHeight" => lava_height,
        "jumpStrength" => jump_strength, "gravity" => gravity,
        "safeFallDistance" => safe_fall_distance, "fallDamageMultiplier" => fall_damage_multiplier,
        "bounciness" => bounciness);
    if object.get("remainingFireTicks").is_some() { state.remaining_fire_ticks = object.number("remainingFireTicks") as i32; }
    if object.get("noJumpDelay").is_some() { state.no_jump_delay = object.number("noJumpDelay") as i32; }
    if object.get("ticksFrozen").is_some() { state.ticks_frozen = object.required("ticksFrozen").parse().expect("invalid ticksFrozen"); }
    if object.get("foodLevel").is_some() { state.food_level = object.number("foodLevel") as i32; }
    if object.get("foodTickTimer").is_some() { state.food_tick_timer = object.number("foodTickTimer") as u32; }
    if object.get("mainSupportingBlock").is_some() {
        state.main_supporting_block = Some(position(object.required("mainSupportingBlock")));
    }
    if object.get("stuckSpeedMultiplier").is_some() {
        state.stuck_speed_multiplier = vector(&object.object("stuckSpeedMultiplier"));
    }
    if object.get("pose").is_some() { state.pose = pose(object.string("pose")); }
    if object.get("speed").is_none() && object.get("speedBits").is_none() {
        state.speed = state.movement_speed_value() as f32;
    }
    state
}

fn load_block(object: &Object<'_>) -> Block {
    let mut block = Block::boxes(&boxes(object.required("boxes")));
    macro_rules! floats {
        ($($key:literal => $field:ident),*) => { $(
            if object.get($key).is_some() || object.get(concat!($key, "Bits")).is_some() {
                block.$field = object.f32($key);
            }
        )* };
    }
    macro_rules! flags {
        ($($key:literal => $field:ident),*) => { $(
            if object.get($key).is_some() { block.$field = object.boolean($key); }
        )* };
    }
    floats!("friction" => friction, "speedFactor" => speed_factor, "jumpFactor" => jump_factor,
        "fallOnMultiplier" => fall_on_multiplier, "bounce" => bounce);
    flags!("climbable" => climbable, "scaffolding" => scaffolding, "slime" => slime,
        "fence" => fence, "wallOrGate" => wall_or_gate, "suppressesBounce" => suppresses_bounce,
        "resetsFallDistance" => resets_fall_distance, "waterBlock" => water_block, "air" => air,
        "bed" => bed, "suffocating" => suffocating, "endPortalOrGateway" => end_portal_or_gateway);
    for (key, target) in [("aboveBoxes", &mut block.above_shape), ("belowBoxes", &mut block.below_shape),
        ("sneakBoxes", &mut block.sneak_shape), ("insideBoxes", &mut block.inside_shape)] {
        if let Some(value) = object.get(key) { *target = Some(VoxelShape::boxes(&boxes(value))); }
    }
    if object.get("fluid").is_some() {
        block.fluid.kind = match object.string("fluid") {
            "water" => FluidKind::Water, "lava" => FluidKind::Lava, "empty" => FluidKind::Empty,
            value => panic!("unsupported fluid {value}"),
        };
        block.fluid.height = object.f32("fluidHeight");
    }
    if object.get("flow").is_some() { block.fluid.flow = vector(&object.object("flow")); }
    let unsupported_fall_on = object.get("unsupportedFallOn").map(|_| object.boolean("unsupportedFallOn"));
    if object.get("inside").is_some() {
        block.inside = match object.string("inside") {
            "cobweb" => Inside::Cobweb, "powderSnow" => Inside::PowderSnow,
            "honey" => Inside::Honey, "none" => Inside::None,
            "unmodeled" if object.get("block").is_some() => {
                let id = object.string("block");
                let name = if unsupported_fall_on.is_none() {
                    format!("{id} (legacy inside=unmodeled). Regenerate the world export.")
                } else {
                    id.to_owned()
                };
                Inside::Unsupported(name)
            }
            value => Inside::Unsupported(value.to_owned()),
        };
    }
    block.unsupported_fall_on = match (unsupported_fall_on, &block.inside) {
        (Some(true), _) => Some(if object.get("block").is_some() {
            object.string("block").to_owned()
        } else {
            "unmodeled".to_owned()
        }),
        // The old field collapsed two hooks into one refusal. Keep both barriers.
        (None, Inside::Unsupported(name)) => Some(name.clone()),
        _ => None,
    };
    if object.get("stuckMultiplier").is_some() {
        block.stuck_multiplier = Some(vector(&object.object("stuckMultiplier")));
    }
    if object.get("ladderFacing").is_some() { block.ladder_facing = Some(object.number("ladderFacing") as u8); }
    if object.get("openTrapdoorFacing").is_some() {
        block.open_trapdoor_facing = Some(object.number("openTrapdoorFacing") as u8);
    }
    block
}

fn parse_world(text: &str) -> (BoxWorld, PlayerState, Option<String>) {
    let mut world = BoxWorld::default();
    let mut state = None;
    let mut server_end = None;
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        let object = Object::parse(line);
        match object.string("kind") {
            "player" => {
                assert!(state.is_none(), "duplicate initial player");
                state = Some(load_state(&object));
            }
            "block" => world.set(position(object.required("at")), load_block(&object)),
            "region" => world.fill(position(object.required("min")), position(object.required("max")), load_block(&object)),
            "world" => {
                if object.get("minY").is_some() { world.minimum_y = object.number("minY") as i32; }
                if object.get("difficulty").is_some() { world.difficulty = object.number("difficulty") as u8; }
                if object.get("fastLava").is_some() { world.fast_lava = object.boolean("fastLava"); }
                if object.get("naturalRegeneration").is_some() { world.natural_regeneration = object.boolean("naturalRegeneration"); }
                if let Some(value) = object.get("unloadedChunks") {
                    world.unloaded_chunks = numbers(value).chunks(2).map(|p| {
                        assert_eq!(p.len(), 2);
                        [p[0] as i32, p[1] as i32]
                    }).collect();
                }
            }
            "collision" => world.extra_shapes.push(VoxelShape::boxes(&boxes(object.required("boxes")))),
            "serverEnd" => {
                assert!(server_end.is_none(), "duplicate server end snapshot");
                server_end = Some(line.to_owned());
            }
            value => panic!("unknown fixture kind {value}"),
        }
    }
    (world, state.expect("fixture needs a pre-tick player state"), server_end)
}

fn assert_vector(actual: Vec3, expected: &Object<'_>, name: &str, tick: u64) {
    for (key, value) in [("x", actual.x), ("y", actual.y), ("z", actual.z)] {
        let bits = hex(expected.required(&format!("{key}Bits")), 16);
        assert_eq!(value.to_bits(), bits,
            "tick {tick} {name}.{key}: actual {value:?} ({:016x}), expected {:?} ({bits:016x})",
            value.to_bits(), f64::from_bits(bits));
    }
}

fn assert_tick(state: &PlayerState, expected: &Object<'_>, tick: u64) {
    assert_vector(state.position, &expected.object("position"), "position", tick);
    assert_vector(state.velocity, &expected.object("velocity"), "velocity", tick);
    assert_eq!(state.yaw.to_bits(), hex(expected.required("yRotBits"), 8) as u32, "tick {tick} yaw");
    assert_eq!(state.pitch.to_bits(), hex(expected.required("xRotBits"), 8) as u32, "tick {tick} pitch");
    for (key, actual) in [("onGround", state.on_ground), ("horizontalCollision", state.horizontal_collision),
        ("verticalCollision", state.vertical_collision), ("isSprinting", state.sprinting),
        ("isShiftKeyDown", state.shift_key_down), ("isInWater", state.in_water), ("onClimbable", state.on_climbable)] {
        assert_eq!(actual, expected.boolean(key), "tick {tick} {key}");
    }
    for (key, actual) in [("minorHorizontalCollision", state.minor_horizontal_collision),
        ("verticalCollisionBelow", state.vertical_collision_below),
        ("swimming", state.swimming), ("crouching", state.crouching)] {
        if expected.get(key).is_some() {
            assert_eq!(actual, expected.boolean(key), "tick {tick} {key}");
        }
    }
    if expected.get("speed").is_some() || expected.get("speedBits").is_some() {
        assert_eq!(state.speed.to_bits(), expected.f32("speed").to_bits(), "tick {tick} speed");
    }
    assert_eq!(state.pose, pose(expected.string("pose")), "tick {tick} pose");
    assert_eq!(state.fall_distance.to_bits(), hex(expected.required("fallDistanceBits"), 16), "tick {tick} fall distance");
}

fn recording_header(object: &Object<'_>) {
    assert_eq!(object.string("type"), "recording");
    for option in ["autoJump", "toggleSprint", "toggleCrouch"] {
        assert!(!object.boolean(option), "unsupported recorder option {option}");
    }
    assert_eq!(object.number("sprintWindow"), 0.0, "unsupported double-tap sprint window");
}

fn replay_text(text: &str, world: &BoxWorld, state: &mut PlayerState) -> (u64, Duration) {
    let mut expected_tick = 0;
    let mut previous_tick_count = None;
    let mut elapsed = Duration::ZERO;
    let mut ended = false;
    let mut lines = text.lines().filter(|line| !line.trim().is_empty()).enumerate().peekable();
    while let Some((line_index, line)) = lines.next() {
        let object = Object::parse(line);
        if object.get("type").is_some() {
            if line_index == 0 && object.string("type") == "recording" {
                recording_header(&object);
                continue;
            }
            assert_eq!(object.string("type"), "end", "unexpected recording row type");
            assert!(lines.peek().is_none(), "end trailer must be last");
            assert!(object.boolean("complete"), "incomplete recording");
            assert_eq!(object.number("ticks"), expected_tick as f64, "end trailer tick count");
            ended = true;
            continue;
        }
        assert!(object.get("anomaly").is_none(), "recording has a correction/push");
        let tick_index = object.number("t");
        assert_eq!(tick_index, expected_tick as f64, "non-contiguous recording");
        if object.get("tickCount").is_some() {
            let count = object.number("tickCount") as i64;
            if let Some(previous) = previous_tick_count { assert_eq!(count, previous + 1, "client tick gap"); }
            previous_tick_count = Some(count);
        }
        state.speed_modifiers = speed_modifiers(&object);
        state.ticks_frozen = object.required("ticksFrozen").parse().expect("invalid ticksFrozen");
        let applied = input(&object.object("inputs"));
        let start = Instant::now();
        tick(state, world, &applied);
        elapsed += start.elapsed();
        assert_tick(state, &object, expected_tick);
        expected_tick += 1;
    }
    assert!(ended, "missing end trailer");
    assert!(expected_tick > 0, "empty recording");
    (expected_tick, elapsed)
}

fn assert_server(state: &PlayerState, expected: &Object<'_>) {
    assert_eq!(state.health.to_bits(), expected.f32("health").to_bits(), "server health");
    assert_eq!(state.food_level as f64, expected.number("foodLevel"), "server food");
    assert_eq!(state.saturation.to_bits(), expected.f32("saturationLevel").to_bits(), "server saturation");
    assert_eq!(state.exhaustion.to_bits(), expected.f32("exhaustion").to_bits(), "server exhaustion");
    assert_eq!(state.food_tick_timer as f64, expected.number("foodTickTimer"), "server foodTickTimer");
}

fn check_server_end(state: &mut PlayerState, world: &BoxWorld, expected: &Object<'_>) -> bool {
    if expected.get("atRest") != Some("true") {
        println!("server resources skipped: serverEnd atRest is false or unknown");
        return false;
    }
    if expected.get("serverTickOffset").is_none_or(|value| value == "null") {
        println!("server resources skipped: serverEnd serverTickOffset is unknown");
        return false;
    }
    let offset: i64 = expected.required("serverTickOffset").parse().expect("invalid serverTickOffset");
    if offset < 0 {
        println!("server resources skipped: serverEnd serverTickOffset is negative ({offset})");
        return false;
    }
    let input = Input { yaw: state.yaw, pitch: state.pitch, ..Input::default() };
    for _ in 0..offset { tick(state, world, &input); }
    assert_server(state, expected);
    true
}

#[test]
#[cfg_attr(not(has_recordings), ignore = "No recorder recordings in lab/fixtures; L0 has not run")]
fn replay() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../lab/fixtures");
    let mut files = Vec::new();
    recordings(&root, &mut files);
    files.sort();
    assert!(!files.is_empty(), "No recorder recordings in lab/fixtures; replay must be ignored until fixtures exist");
    let mut rates = Vec::new();
    let mut server_checks = 0;
    let mut server_skips = 0;
    for file in files {
        println!("Replaying {}", file.display());
        let fixture = fs::read_to_string(file.with_extension("world.jsonl")).expect("fixture world description");
        let (world, mut state, server_end) = parse_world(&fixture);
        let text = fs::read_to_string(&file).unwrap();
        let (ticks, elapsed) = replay_text(&text, &world, &mut state);
        if let Some(snapshot) = server_end {
            println!("{} serverEnd", file.display());
            if check_server_end(&mut state, &world, &Object::parse(&snapshot)) {
                server_checks += 1;
            } else {
                server_skips += 1;
            }
        } else {
            println!("{} server resources skipped: no serverEnd snapshot", file.display());
            server_skips += 1;
        }
        let rate = ticks as f64 / elapsed.as_secs_f64();
        rates.push(rate);
        println!("{} {ticks} ticks, {rate:.0} ticks/s", file.display());
    }
    println!("server resources: {server_checks} fixtures compared, {server_skips} fixtures skipped");
    rates.sort_by(f64::total_cmp);
    let quantile = |p: f64| rates[((rates.len() - 1) as f64 * p).ceil() as usize];
    println!("fixture ticks/s n={}, min={:.0}, median={:.0}, p90={:.0}, p99={:.0}, max={:.0}, server snapshots={server_checks}",
        rates.len(), rates[0], quantile(0.5), quantile(0.9), quantile(0.99), rates[rates.len() - 1]);
}

#[test]
fn fixed_json_format() {
    let object = Object::parse(r#"{"inputs":{"forward":true},"position":{"x":1,"xBits":"3ff0000000000000"},"t":0}"#);
    assert!(object.object("inputs").boolean("forward"));
    assert_eq!(object.object("position").f64("x"), 1.0);
    assert_eq!(numbers("[[0,0,0,1,1,1]]"), [0., 0., 0., 1., 1., 1.]);
}

#[test]
#[should_panic(expected = "duplicate JSON key")]
fn duplicate_json_key() {
    Object::parse(r#"{"t":0,"t":1}"#);
}

#[test]
fn recorder_header() {
    recording_header(&Object::parse(r#"{"type":"recording","autoJump":false,"toggleSprint":false,"toggleCrouch":false,"sprintWindow":0}"#));
}

#[test]
#[should_panic(expected = "unsupported recorder option")]
fn reject_option_dependent_recording() {
    recording_header(&Object::parse(r#"{"type":"recording","autoJump":true,"toggleSprint":false,"toggleCrouch":false,"sprintWindow":0}"#));
}

#[test]
fn fixture_initialization() {
    let (world, state, server_end) = parse_world(concat!(
        "{\"kind\":\"player\",\"movementSpeedBase\":0.10000000149011612,\"speedModifiers\":[],\"position\":{\"x\":0,\"y\":0,\"z\":0},\"velocity\":{\"x\":0,\"y\":0,\"z\":0},\"isSprinting\":false,\"speed\":0.13,\"noJumpDelay\":7,\"onGroundNoBlocks\":true}\n",
        "{\"kind\":\"world\",\"difficulty\":3,\"naturalRegeneration\":false,\"fastLava\":true,\"unloadedChunks\":[[1,-1]]}\n",
        "{\"kind\":\"region\",\"min\":[-1,-1,-1],\"max\":[1,-1,1],\"boxes\":[[0,0,0,1,1,1]],\"friction\":0.98}\n",
        "{\"kind\":\"block\",\"at\":[0,-1,0],\"boxes\":[[0,0,0,1,0.5,1]],\"speedFactor\":0.4}\n",
        "{\"kind\":\"serverEnd\",\"health\":20,\"foodLevel\":20,\"saturationLevel\":5,\"exhaustion\":0,\"foodTickTimer\":0}\n"
    ));
    assert_eq!(state.speed, 0.13f32);
    assert_eq!(state.no_jump_delay, 7);
    assert!(state.on_ground_no_blocks);
    assert_eq!(world.block([1,-1,0]).friction, 0.98f32);
    assert_eq!(world.block([0,-1,0]).speed_factor, 0.4f32);
    assert!(!world.loaded([16,0,-1]));
    assert!(world.fast_lava());
    assert!(!world.natural_regeneration());
    assert_eq!(world.difficulty(), 3);
    assert_server(&state, &Object::parse(&server_end.unwrap()));
}

#[test]
#[should_panic(expected = "missing isSprinting")]
fn fixture_requires_initial_sprint_latch() {
    parse_world(r#"{"kind":"player","movementSpeedBase":0.10000000149011612,"speedModifiers":[],"position":{"x":0,"y":0,"z":0},"velocity":{"x":0,"y":0,"z":0}}"#);
}

fn stationary_tick() -> String {
    let vertical_bits = (-0.08 * 0.98f32 as f64).to_bits();
    format!(concat!(
        "{{\"t\":0,\"speedModifiers\":[],\"ticksFrozen\":0,\"inputs\":{{\"forward\":false,\"back\":false,\"left\":false,\"right\":false,\"jump\":false,\"sneak\":false,\"sprint\":false,\"yaw\":0,\"pitch\":0}},",
        "\"position\":{{\"x\":0,\"xBits\":\"0000000000000000\",\"y\":0,\"yBits\":\"0000000000000000\",\"z\":0,\"zBits\":\"0000000000000000\"}},",
        "\"velocity\":{{\"x\":0,\"xBits\":\"0000000000000000\",\"y\":-0.0784,\"yBits\":\"{vertical_bits:016x}\",\"z\":0,\"zBits\":\"0000000000000000\"}},",
        "\"yRotBits\":\"00000000\",\"xRotBits\":\"00000000\",\"onGround\":true,\"horizontalCollision\":false,\"verticalCollision\":true,",
        "\"isSprinting\":false,\"isShiftKeyDown\":false,\"isInWater\":false,\"onClimbable\":false,\"pose\":\"STANDING\",\"fallDistanceBits\":\"0000000000000000\"}}"
    ), vertical_bits = vertical_bits)
}

fn stationary_fixture() -> (BoxWorld, PlayerState) {
    let mut world = BoxWorld::default();
    world.fill([-1,-1,-1], [1,-1,1], Block::full());
    let state = PlayerState {
        on_ground: true,
        velocity: Vec3::new(0.0, -0.08 * 0.98f32 as f64, 0.0),
        ..PlayerState::default()
    };
    (world, state)
}

#[test]
fn replay_stationary_tick() {
    let (world, mut state) = stationary_fixture();
    let (ticks, _) = replay_text(&complete_recording(&stationary_tick(), 1), &world, &mut state);
    assert_eq!(ticks, 1);
}

#[test]
#[should_panic(expected = "position.x")]
fn replay_rejects_single_bit_difference() {
    let (world, mut state) = stationary_fixture();
    let recording = stationary_tick().replacen("\"xBits\":\"0000000000000000\"", "\"xBits\":\"0000000000000001\"", 1);
    replay_text(&complete_recording(&recording, 1), &world, &mut state);
}

#[test]
#[should_panic(expected = "correction/push")]
fn replay_rejects_anomaly() {
    let (world, mut state) = stationary_fixture();
    let recording = stationary_tick();
    let recording = format!("{},\"anomaly\":[{{\"type\":\"ClientboundPlayerPositionPacket\"}}]}}", &recording[..recording.len()-1]);
    replay_text(&complete_recording(&recording, 1), &world, &mut state);
}
#[test]
fn replay_compares_optional_fields() {
    let (world, mut state) = stationary_fixture();
    let recording = stationary_tick();
    let record = format!("{},\"minorHorizontalCollision\":false,\"verticalCollisionBelow\":true,\"speedBits\":\"3dcccccd\",\"swimming\":false,\"crouching\":false}}",
        &recording[..recording.len()-1]);
    replay_text(&complete_recording(&record, 1), &world, &mut state);
    for (key, old, new) in [
        ("minorHorizontalCollision", "false", "true"),
        ("verticalCollisionBelow", "true", "false"),
        ("speedBits", "\"3dcccccd\"", "\"3dcccccc\""),
        ("swimming", "false", "true"),
        ("crouching", "false", "true"),
    ] {
        let corrupt = record.replace(&format!("\"{key}\":{old}"), &format!("\"{key}\":{new}"));
        let (world, mut state) = stationary_fixture();
        let failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            replay_text(&complete_recording(&corrupt, 1), &world, &mut state);
        }));
        assert!(failure.is_err(), "{key} corruption was accepted");
    }
}

#[test]
fn replay_seeds_sprint_from_player() {
    let fixture = concat!(
        "{\"kind\":\"player\",\"movementSpeedBase\":0.10000000149011612,\"speedModifiers\":[],\"position\":{\"x\":0,\"y\":0,\"z\":0},\"velocity\":{\"x\":0,\"y\":-0.0784000015258789,\"z\":0},\"onGround\":true,\"isSprinting\":true}\n",
        "{\"kind\":\"region\",\"min\":[-1,-1,-1],\"max\":[1,-1,1],\"boxes\":[[0,0,0,1,1,1]]}\n"
    );
    let (world, mut initial, _) = parse_world(fixture);
    assert!(initial.sprinting);
    let mut expected = initial.clone();
    tick(&mut expected, &world, &Input { forward: true, ..Input::default() });
    assert!(expected.sprinting);
    let mut recording = stationary_tick().replace("\"forward\":false", "\"forward\":true")
        .replace("\"isSprinting\":false", "\"isSprinting\":true");
    for value in [expected.position.z, expected.velocity.z] {
        recording = recording.replacen("\"z\":0,\"zBits\":\"0000000000000000\"",
            &format!("\"z\":{value},\"zBits\":\"{:016x}\"", value.to_bits()), 1);
    }
    assert!(!Object::parse(&recording).object("inputs").boolean("sprint"));
    let (ticks, _) = replay_text(&complete_recording(&recording, 1), &world, &mut initial);
    assert_eq!(ticks, 1);
    assert!(initial.sprinting);

    let (world, mut initial, _) = parse_world(&fixture.replace("\"isSprinting\":true", "\"isSprinting\":false"));
    let failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        replay_text(&complete_recording(&recording, 1), &world, &mut initial);
    }));
    assert!(failure.is_err(), "post-tick isSprinting was accepted as the initial latch");
}

#[test]
fn fixture_keeps_unsupported_hooks_explicit() {
    for hook in ["bubbleColumn", "sweetBerryBush", "unmodeled"] {
        let block = load_block(&Object::parse(&format!("{{\"boxes\":[],\"inside\":\"{hook}\"}}")));
        assert_eq!(block.inside, Inside::Unsupported(hook.to_owned()));
        assert_eq!(block.unsupported_fall_on.as_deref(), Some(hook));
    }
}

fn complete_recording(rows: &str, ticks: u64) -> String {
    format!("{rows}\n{{\"type\":\"end\",\"ticks\":{ticks},\"complete\":true}}")
}

#[test]
fn replay_accepts_recorder_trailer() {
    let header = r#"{"type":"recording","autoJump":false,"toggleSprint":false,"toggleCrouch":false,"sprintWindow":0}"#;
    let row = stationary_tick();
    for rows in [row.clone(), format!("{header}\n{row}"), format!("{header}\n{row}\n{}", row.replace("\"t\":0", "\"t\":1"))] {
        let count = if rows.contains("\"t\":1") { 2 } else { 1 };
        let (world, mut state) = stationary_fixture();
        let (ticks, _) = replay_text(&complete_recording(&rows, count), &world, &mut state);
        assert_eq!(ticks, count);
    }
}

#[test]
fn replay_rejects_invalid_trailers_and_row_types() {
    let row = stationary_tick();
    let end = r#"{"type":"end","ticks":1,"complete":true}"#;
    for (name, text, message) in [
        ("missing", row.clone(), "missing end trailer"),
        ("incomplete", format!("{row}\n{}", end.replace("true", "false")), "incomplete recording"),
        ("wrong count", format!("{row}\n{}", end.replace(":1", ":2")), "end trailer tick count"),
        ("fractional count", format!("{row}\n{}", end.replace(":1", ":1.5")), "end trailer tick count"),
        ("tick after end", format!("{row}\n{end}\n{row}"), "end trailer must be last"),
        ("duplicate end", format!("{row}\n{end}\n{end}"), "end trailer must be last"),
        ("late header", format!("{row}\n{{\"type\":\"recording\"}}\n{end}"), "unexpected recording row type"),
        ("unknown type", format!("{row}\n{{\"type\":\"other\"}}\n{end}"), "unexpected recording row type"),
    ] {
        let (world, mut state) = stationary_fixture();
        let failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            replay_text(&text, &world, &mut state);
        })).expect_err(name);
        let diagnostic = failure.downcast_ref::<String>().map(String::as_str)
            .or_else(|| failure.downcast_ref::<&str>().copied()).expect("panic message");
        assert!(diagnostic.contains(message), "{name}: {diagnostic}");
    }
}

#[test]
fn fixture_server_end_replays_rest_ticks_and_checks_every_resource() {
    let fixture = concat!(
        "{\"kind\":\"player\",\"movementSpeedBase\":0.10000000149011612,\"speedModifiers\":[],\"position\":{\"x\":0,\"y\":0,\"z\":0},\"velocity\":{\"x\":0,\"y\":-0.0784000015258789,\"z\":0},\"onGround\":true,\"isSprinting\":false,\"health\":19,\"saturationLevel\":6,\"foodTickTimer\":8}\n",
        "{\"kind\":\"region\",\"min\":[-1,-1,-1],\"max\":[1,-1,1],\"boxes\":[[0,0,0,1,1,1]]}"
    );
    for (offset, health, saturation, exhaustion, timer) in [
        (0, 19, 6, 0, 9), (1, 20, 6, 6, 0), (2, 20, 5, 2, 0), (20, 20, 5, 2, 0)
    ] {
        let snapshot = format!("{{\"kind\":\"serverEnd\",\"atRest\":true,\"serverTickOffset\":{offset},\"health\":{health},\"foodLevel\":20,\"saturationLevel\":{saturation},\"exhaustion\":{exhaustion},\"foodTickTimer\":{timer}}}");
        let (world, mut state, end) = parse_world(&format!("{fixture}\n{snapshot}"));
        replay_text(&complete_recording(&stationary_tick(), 1), &world, &mut state);
        assert!(check_server_end(&mut state, &world, &Object::parse(&end.unwrap())));
        assert_eq!(state.t, 1 + offset as u64);
        for (key, correct, wrong) in [
            ("health", health, health-1), ("foodLevel", 20, 19),
            ("saturationLevel", saturation, saturation+1), ("exhaustion", exhaustion, exhaustion+1),
            ("foodTickTimer", timer, timer+1)
        ] {
            let corrupt = snapshot.replace(&format!("\"{key}\":{correct}"), &format!("\"{key}\":{wrong}"));
            let (world, mut state, _) = parse_world(fixture);
            replay_text(&complete_recording(&stationary_tick(), 1), &world, &mut state);
            let failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                check_server_end(&mut state, &world, &Object::parse(&corrupt));
            }));
            assert!(failure.is_err(), "server {key} corruption accepted at offset {offset}");
        }
    }
}

#[test]
fn fixture_server_end_skips_unaligned_or_moving_snapshots() {
    let (world, mut state) = stationary_fixture();
    for metadata in [
        r#""atRest":false,"serverTickOffset":0"#,
        r#""serverTickOffset":0"#,
        r#""atRest":true,"serverTickOffset":-1"#,
        r#""atRest":true,"serverTickOffset":-20"#,
        r#""atRest":true"#,
        r#""atRest":true,"serverTickOffset":null"#,
        r#""atRest":true,"ticksAfterEnd":0"#,
    ] {
        assert!(!check_server_end(&mut state, &world, &Object::parse(&format!("{{{metadata}}}"))));
        assert_eq!(state.t, 0, "skipped snapshot advanced simulation");
    }
}

#[test]
fn fixture_reads_exported_world_fields() {
    let fixture = concat!(
        "{\"kind\":\"player\",\"movementSpeedBase\":0.10000000149011612,\"speedModifiers\":[],\"position\":{\"x\":0,\"y\":0,\"z\":0},\"velocity\":{\"x\":0,\"y\":0,\"z\":0},\"isSprinting\":false,\"ticksFrozen\":73}\n",
        "{\"kind\":\"block\",\"at\":[0,-1,0],\"block\":\"minecraft:red_bed\",\"boxes\":[[0,0,0,1,0.5,1]],\"bed\":true}\n",
        "{\"kind\":\"block\",\"at\":[1,-1,0],\"boxes\":[[0,0,0,1,1,1]],\"suffocating\":true}\n",
        "{\"kind\":\"block\",\"at\":[2,-1,0],\"boxes\":[],\"endPortalOrGateway\":true}\n"
    );
    let (world, state, _) = parse_world(fixture);
    assert_eq!(state.ticks_frozen, 73);
    assert!(world.block([0,-1,0]).bed);
    assert!(world.block([1,-1,0]).suffocating);
    assert!(world.block([2,-1,0]).end_portal_or_gateway);
}

#[test]
fn replay_rejects_touched_unmodeled_block_by_name() {
    let (mut world, mut state) = stationary_fixture();
    let block = load_block(&Object::parse(r#"{"boxes":[],"inside":"unmodeled","block":"minecraft:sweet_berry_bush"}"#));
    world.set([4,0,0], block.clone());
    replay_text(&complete_recording(&stationary_tick(), 1), &world, &mut state);
    world.set([0,0,0], block);
    let failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        replay_text(&complete_recording(&stationary_tick(), 1), &world, &mut state);
    })).expect_err("unmodeled contact was accepted");
    let diagnostic = failure.downcast_ref::<String>().expect("panic message");
    assert!(diagnostic.contains("BlockState.entityInside: minecraft:sweet_berry_bush"), "{diagnostic}");
    assert!(diagnostic.contains("Regenerate the world export"), "{diagnostic}");
}

#[test]
fn fixture_reads_server_driven_modifiers_and_frozen_counter_each_tick() {
    let fixture = concat!(
        "{\"kind\":\"player\",\"movementSpeedBase\":0.10000000149011612,\"speedModifiers\":[[\"minecraft:powder_snow\",-0.025,\"ADD_VALUE\"]],\"position\":{\"x\":0,\"y\":0,\"z\":0},\"velocity\":{\"x\":0,\"y\":-0.0784000015258789,\"z\":0},\"onGround\":true,\"isSprinting\":false,\"ticksFrozen\":73}\n",
        "{\"kind\":\"region\",\"min\":[-1,-1,-1],\"max\":[1,-1,1],\"boxes\":[[0,0,0,1,1,1]]}"
    );
    let (world, mut state, _) = parse_world(fixture);
    assert_eq!(state.speed.to_bits(), ((0.10000000149011612f64-0.025) as f32).to_bits());
    let mut rows = Vec::new();
    for (index, modifiers, frozen, speed) in [
        (0, r#"[["minecraft:powder_snow",-0.05000000074505806,"ADD_VALUE"]]"#, 140, 0.05f32),
        (1, "[]", 0, 0.1f32),
        (2, r#"[["test:total",1,"ADD_MULTIPLIED_TOTAL"],["test:base",0.5,"ADD_MULTIPLIED_BASE"],["test:add",0.125,"ADD_VALUE"]]"#, 73, 0.675f32),
    ] {
        let row = stationary_tick().replace("\"speedModifiers\":[]",
            &format!("\"speedModifiers\":{modifiers}"))
            .replace("\"ticksFrozen\":0", &format!("\"ticksFrozen\":{frozen}"));
        let row = format!("{},\"speedBits\":\"{:08x}\"}}", &row[..row.len()-1], speed.to_bits());
        rows.push(row.replace("\"t\":0", &format!("\"t\":{index}")));
    }
    replay_text(&complete_recording(&rows.join("\n"), 3), &world, &mut state);
    assert_eq!(state.ticks_frozen, 73, "server observation replaces the counter");
    assert_eq!(state.speed_modifiers.len(), 3);
}

#[test]
#[should_panic(expected = "missing movementSpeedBase")]
fn fixture_requires_movement_speed_base() {
    parse_world(r#"{"kind":"player","position":{"x":0,"y":0,"z":0},"velocity":{"x":0,"y":0,"z":0},"isSprinting":false,"movementSpeed":0.13,"speedModifiers":[]}"#);
}

#[test]
fn fixture_base_and_modifiers_seed_sprint_exactly_once() {
    let fixture = r#"{"kind":"player","position":{"x":0,"y":0,"z":0},"velocity":{"x":0,"y":0,"z":0},"isSprinting":true,"movementSpeedBase":0.125,"speedModifiers":[["test:add",0.125,"ADD_VALUE"],["test:base",0.5,"ADD_MULTIPLIED_BASE"],["test:total",1,"ADD_MULTIPLIED_TOTAL"]]}"#;
    let (_, state, _) = parse_world(fixture);
    let expected = (0.25f64 + 0.25*0.5) * 2.0 * (1.0 + 0.3f32 as f64);
    assert_eq!(state.speed.to_bits(), (expected as f32).to_bits());
    let explicit_speed = fixture.replace("\"movementSpeedBase\":", "\"speed\":0.0625,\"movementSpeedBase\":");
    let (_, state, _) = parse_world(&explicit_speed);
    assert_eq!(state.speed.to_bits(), 0.0625f32.to_bits(), "preserve recorded initial getSpeed");
}

#[test]
#[should_panic(expected = "BlockState.fallOn: minecraft:slime_block (legacy inside=unmodeled). Regenerate")]
fn replay_refuses_ambiguous_legacy_landing() {
    let (mut world, mut state) = stationary_fixture();
    state.fall_distance = 13.0;
    world.fill([-1,-1,-1], [1,-1,1], load_block(&Object::parse(
        r#"{"boxes":[[0,0,0,1,1,1]],"inside":"unmodeled","block":"minecraft:slime_block","slime":true,"bounce":1}"#
    )));
    replay_text(&complete_recording(&stationary_tick(), 1), &world, &mut state);
}

#[test]
#[ignore = "requires the actual 26.2 registry rows from test-exporter.ps1"]
fn registry_export_contacts() {
    let path = std::env::var_os("PATHTRACING_HOOK_ROWS").expect("run test-exporter.ps1");
    let rows = fs::read_to_string(path).expect("read exporter check rows");
    let mut contacts = 0;
    for line in rows.lines() {
        let row = Object::parse(line);
        let id = row.string("block");
        let mut world = BoxWorld::default();
        world.natural_regeneration = false;
        world.set([0,0,0], load_block(&row));
        let mut player = PlayerState::default();
        let inside = id == "minecraft:sweet_berry_bush";
        let top = boxes(row.required("boxes")).iter().map(|b| b.max.y).fold(0.0, f64::max);
        if inside {
            player.position = Vec3::new(0.5, 0.5, 0.5);
        } else {
            player.position = Vec3::new(0.5, top + 0.125, 0.5);
            player.velocity.y = -0.5;
            player.fall_distance = 13.0;
        }
        let refusal = match id {
            "minecraft:sweet_berry_bush" => Some("BlockState.entityInside"),
            "minecraft:pointed_dripstone" => Some("BlockState.fallOn"),
            _ => None,
        };
        if let Some(hook) = refusal {
            let failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                tick(&mut player, &world, &Input::default());
            })).expect_err("unmodeled contact was accepted");
            let diagnostic = failure.downcast_ref::<String>().expect("panic message");
            assert!(diagnostic.contains(&format!("{hook}: {id}")), "{diagnostic}");
        } else {
            let (health, bounce) = match id {
                "minecraft:slime_block" => (20.0, 1.0),
                "minecraft:hay_block" => (18.0, 0.0),
                id if id.ends_with("_bed") => (17.0, 0.75),
                _ => panic!("unexpected registry witness {id}"),
            };
            tick(&mut player, &world, &Input::default());
            assert!(player.on_ground, "{id} did not land");
            assert_eq!(player.position.y, top, "{id} contact height");
            assert_eq!(player.fall_distance, 0.0, "{id} fall distance");
            assert_eq!(player.health, health, "{id} landing damage");
            let drag = 0.98f32 as f64;
            let restitution = (0.25 * 0.08 + 0.5) * (1.0 + 0.25 * (drag - 1.0)) * bounce;
            assert_eq!(player.velocity.y, (restitution - 0.08) * drag, "{id} landing velocity");
        }
        contacts += 1;
    }
    assert_eq!(contacts, 20, "slime, hay, all 16 beds and both unsupported hooks");
}

#[test]
fn replay_modifier_changes_apply_to_travel_on_the_observed_tick() {
    let (world, mut state) = stationary_fixture();
    let acceleration = |speed: f32| speed * (0.21600002f32 / (0.6f32*0.6f32*0.6f32));
    let drag = (0.6f32 * 0.91f32) as f64;
    let first_move = 0.98f32 as f64 * acceleration(0.05) as f64;
    let second_move = first_move*drag + 0.98f32 as f64 * acceleration(0.1) as f64;
    let mut rows = Vec::new();
    for (index, z, velocity, frozen, modifiers, speed) in [
        (0, first_move, first_move*drag, 140,
            r#"[["minecraft:powder_snow",-0.05000000074505806,"ADD_VALUE"]]"#, 0.05f32),
        (1, first_move+second_move, second_move*drag, 0, "[]", 0.1f32),
    ] {
        let mut row = stationary_tick().replace("\"forward\":false", "\"forward\":true")
            .replace("\"t\":0", &format!("\"t\":{index}"))
            .replace("\"ticksFrozen\":0", &format!("\"ticksFrozen\":{frozen}"))
            .replace("\"speedModifiers\":[]", &format!("\"speedModifiers\":{modifiers}"));
        for value in [z, velocity] {
            row = row.replacen("\"z\":0,\"zBits\":\"0000000000000000\"",
                &format!("\"z\":{value},\"zBits\":\"{:016x}\"", value.to_bits()), 1);
        }
        rows.push(format!("{},\"speedBits\":\"{:08x}\"}}", &row[..row.len()-1], speed.to_bits()));
    }
    replay_text(&complete_recording(&rows.join("\n"), 2), &world, &mut state);
    assert_eq!(state.position.z.to_bits(), (first_move+second_move).to_bits());
}

#[test]
fn server_end_rejects_one_bit_resource_errors() {
    let (world, state) = stationary_fixture();
    for (field, value) in [("health", state.health), ("saturationLevel", state.saturation), ("exhaustion", state.exhaustion)] {
        let snapshot = format!(concat!(
            "{{\"atRest\":true,\"serverTickOffset\":0,\"health\":20,\"foodLevel\":20,",
            "\"saturationLevel\":5,\"exhaustion\":0,\"foodTickTimer\":0,\"{field}Bits\":\"{bits:08x}\"}}"
        ), field = field, bits = value.to_bits() ^ 1);
        let mut actual = state.clone();
        let failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            check_server_end(&mut actual, &world, &Object::parse(&snapshot));
        }));
        assert!(failure.is_err(), "one-bit {field} mismatch was accepted");
    }
}

#[test]
fn replay_requires_server_driven_fields() {
    for key in ["speedModifiers", "ticksFrozen"] {
        let (world, mut state) = stationary_fixture();
        let row = stationary_tick().replace(&format!("\"{key}\":{},", if key == "speedModifiers" { "[]" } else { "0" }), "");
        let failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            replay_text(&complete_recording(&row, 1), &world, &mut state);
        })).expect_err("missing input field was accepted");
        let diagnostic = failure.downcast_ref::<String>().expect("panic message");
        assert!(diagnostic.contains(&format!("missing {key}")), "{diagnostic}");
    }
}
