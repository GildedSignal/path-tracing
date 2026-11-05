#[path = "support/json.rs"]
mod json;

use json::{Object, hex, numbers};
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

fn load_state(object: &Object<'_>) -> PlayerState {
    let mut state = PlayerState::default();
    state.position = vector(&object.object("position"));
    state.velocity = vector(&object.object("velocity"));
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
        "minorHorizontalCollision" => minor_horizontal_collision, "isSprinting" => sprinting,
        "isShiftKeyDown" => shift_key_down, "isInWater" => in_water, "isInLava" => in_lava,
        "onClimbable" => on_climbable, "crouching" => crouching, "swimming" => swimming,
        "underwater" => underwater, "eyeInWater" => eye_in_water, "wasEyeInWater" => was_eye_in_water,
        "leatherBoots" => leather_boots, "wasInPowderSnow" => was_in_powder_snow,
        "inPowderSnow" => in_powder_snow, "onGroundNoBlocks" => on_ground_no_blocks);
    floats!("yRot" => yaw, "xRot" => pitch, "health" => health, "saturationLevel" => saturation,
        "exhaustion" => exhaustion, "speed" => speed, "stepHeight" => step_height,
        "frictionModifier" => friction_modifier, "airDragModifier" => air_drag_modifier,
        "waterMovementEfficiency" => water_movement_efficiency, "movementEfficiency" => movement_efficiency,
        "sneakingSpeed" => sneaking_speed);
    doubles!("fallDistance" => fall_distance, "waterHeight" => water_height, "lavaHeight" => lava_height,
        "movementSpeed" => movement_speed, "jumpStrength" => jump_strength, "gravity" => gravity,
        "safeFallDistance" => safe_fall_distance, "fallDamageMultiplier" => fall_damage_multiplier,
        "bounciness" => bounciness);
    if object.get("noJumpDelay").is_some() { state.no_jump_delay = object.number("noJumpDelay") as i32; }
    if object.get("foodLevel").is_some() { state.food_level = object.number("foodLevel") as i32; }
    if object.get("foodTickTimer").is_some() { state.food_tick_timer = object.number("foodTickTimer") as u32; }
    if object.get("mainSupportingBlock").is_some() {
        state.main_supporting_block = Some(position(object.required("mainSupportingBlock")));
    }
    if object.get("stuckSpeedMultiplier").is_some() {
        state.stuck_speed_multiplier = vector(&object.object("stuckSpeedMultiplier"));
    }
    if object.get("pose").is_some() { state.pose = pose(object.string("pose")); }
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
        "resetsFallDistance" => resets_fall_distance, "waterBlock" => water_block, "air" => air);
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
    if object.get("inside").is_some() {
        block.inside = match object.string("inside") {
            "cobweb" => Inside::Cobweb, "powderSnow" => Inside::PowderSnow,
            "honey" => Inside::Honey, "none" => Inside::None,
            value => panic!("unsupported inside hook {value}"),
        };
    }
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
    for (line_index, line) in text.lines().filter(|line| !line.trim().is_empty()).enumerate() {
        let object = Object::parse(line);
        if object.get("type").is_some() {
            assert_eq!(line_index, 0, "recording header must be first");
            recording_header(&object);
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
        let applied = input(&object.object("inputs"));
        let start = Instant::now();
        tick(state, world, &applied);
        elapsed += start.elapsed();
        assert_tick(state, &object, expected_tick);
        expected_tick += 1;
    }
    assert!(expected_tick > 0, "empty recording");
    (expected_tick, elapsed)
}

fn assert_server(state: &PlayerState, expected: &Object<'_>) {
    assert_eq!(state.health.to_bits(), expected.f32("health").to_bits(), "server health");
    assert_eq!(state.food_level as f64, expected.number("foodLevel"), "server food");
    assert_eq!(state.saturation.to_bits(), expected.f32("saturationLevel").to_bits(), "server saturation");
    assert_eq!(state.exhaustion.to_bits(), expected.f32("exhaustion").to_bits(), "server exhaustion");
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
    for file in files {
        let fixture = fs::read_to_string(file.with_extension("world.jsonl")).expect("fixture world description");
        let (world, mut state, server_end) = parse_world(&fixture);
        let text = fs::read_to_string(&file).unwrap();
        let (ticks, elapsed) = replay_text(&text, &world, &mut state);
        if let Some(snapshot) = server_end {
            assert_server(&state, &Object::parse(&snapshot));
            server_checks += 1;
        }
        let rate = ticks as f64 / elapsed.as_secs_f64();
        rates.push(rate);
        println!("{} {ticks} ticks, {rate:.0} ticks/s", file.display());
    }
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
        "{\"kind\":\"player\",\"position\":{\"x\":0,\"y\":0,\"z\":0},\"velocity\":{\"x\":0,\"y\":0,\"z\":0},\"speed\":0.13,\"noJumpDelay\":7,\"onGroundNoBlocks\":true}\n",
        "{\"kind\":\"world\",\"difficulty\":3,\"naturalRegeneration\":false,\"fastLava\":true,\"unloadedChunks\":[[1,-1]]}\n",
        "{\"kind\":\"region\",\"min\":[-1,-1,-1],\"max\":[1,-1,1],\"boxes\":[[0,0,0,1,1,1]],\"friction\":0.98}\n",
        "{\"kind\":\"block\",\"at\":[0,-1,0],\"boxes\":[[0,0,0,1,0.5,1]],\"speedFactor\":0.4}\n",
        "{\"kind\":\"serverEnd\",\"health\":20,\"foodLevel\":20,\"saturationLevel\":5,\"exhaustion\":0}\n"
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
fn stationary_recording() -> String {
    let vertical_bits = (-0.08 * 0.98f32 as f64).to_bits();
    format!(concat!(
        "{{\"t\":0,\"inputs\":{{\"forward\":false,\"back\":false,\"left\":false,\"right\":false,\"jump\":false,\"sneak\":false,\"sprint\":false,\"yaw\":0,\"pitch\":0}},",
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
    let (ticks, _) = replay_text(&stationary_recording(), &world, &mut state);
    assert_eq!(ticks, 1);
}

#[test]
#[should_panic(expected = "position.x")]
fn replay_rejects_single_bit_difference() {
    let (world, mut state) = stationary_fixture();
    let recording = stationary_recording().replacen("\"xBits\":\"0000000000000000\"", "\"xBits\":\"0000000000000001\"", 1);
    replay_text(&recording, &world, &mut state);
}

#[test]
#[should_panic(expected = "correction/push")]
fn replay_rejects_anomaly() {
    let (world, mut state) = stationary_fixture();
    let recording = stationary_recording();
    let recording = format!("{},\"anomaly\":[{{\"type\":\"ClientboundPlayerPositionPacket\"}}]}}", &recording[..recording.len()-1]);
    replay_text(&recording, &world, &mut state);
}