use law::*;
use law::footprint::{EnvironmentRead, Footprint, Query, RecordingWorld};
use law::pattern::{FRAMES, Frame, PatternError, PatternTable, model_digest};

fn floor(world: &mut BoxWorld, [x, y, z]: BlockPos, block: Block) { world.fill([x-30, y-1, z-30], [x+30, y-1, z+30], block); }

/// Runs `ticks` from `start` on the snapshot, recording, and returns the footprint and the states.
fn word(world: &BoxWorld, start: &PlayerState, inputs: &[Input]) -> (Footprint, Vec<PlayerState>) {
    let recorder = RecordingWorld::new(world);
    let mut state = start.clone();
    let states = inputs.iter().map(|input| { recorder.tick(&mut state, input); state.clone() }).collect();
    (recorder.take(), states)
}

fn standing(p: BlockPos) -> PlayerState {
    PlayerState { position: Vec3::new(p[0] as f64+0.5, p[1] as f64, p[2] as f64+0.5), on_ground: true, ..PlayerState::default() }
}

// Does a hit need the same contents, as the law reads them, and the same start bits?
#[test]
fn translated_copies_hit_and_one_block_or_one_bit_misses() {
    let forward = [Input { forward: true, ..Input::default() }];
    let mut near = BoxWorld::default();
    floor(&mut near, [0, 0, 0], Block::full());
    let (footprint, states) = word(&near, &standing([0, 0, 0]), &forward);
    let mut table = PatternTable::default();
    table.insert(&near, &footprint, &standing([0, 0, 0]), states).unwrap();
    let far = [1_000_000, 70, -1_000_000];
    // The same physics written differently: the shapes the law resolves are equal.
    let full = Block::full();
    let mut spelled = BoxWorld::default();
    floor(&mut spelled, far, Block { above_shape: Some(full.shape.clone()), below_shape: Some(full.shape.clone()), ..full });
    assert!(table.lookup(&spelled, &standing(far)).is_some());
    let mut ice = spelled.clone();
    ice.set([far[0], far[1]-1, far[2]], Block { friction: 0.98, ..Block::full() });
    assert!(table.lookup(&ice, &standing(far)).is_none());
    let mut moving = standing(far);
    moving.velocity.x = f64::from_bits(1);
    assert!(table.lookup(&spelled, &moving).is_none());
    println!("{} ids, {} cells", table.digests().len(), table.shapes()[0].cells.len());
}

// Is an integer shift bit-exact wherever the entry says so, across 2^n boundaries and y = 64?
#[test]
fn shifts_inside_the_binades_replay_bit_exact() {
    let cycle: Vec<Input> = (0..20).map(|t| Input { forward: true, sprint: true, jump: t % 6 == 0, yaw: 37.3, ..Input::default() }).collect();
    let origin = [1030, 70, 1030];
    let mut base = BoxWorld::default();
    floor(&mut base, origin, Block::full());
    let (footprint, states) = word(&base, &standing(origin), &cycle);
    let mut table = PatternTable::default();
    table.insert(&base, &footprint, &standing(origin), states).unwrap();
    let (mut exact, mut replayed, mut differed) = (0, 0, 0);
    for shift in [[1, 0, 0], [7, 0, -3], [900, 0, 0], [0, 5, 0], [0, 40, 0], [5, 0, 900],
        [1100, 0, 0], [-10, 0, 0], [0, 58, 0], [0, -6, 0], [0, -7, 0], [0, -40, 0], [0, 0, -7], [1 << 20, 0, 0]] {
        let at = [0, 1, 2].map(|a| origin[a] + shift[a]);
        let mut world = BoxWorld::default();
        floor(&mut world, at, Block::full());
        let hit = table.lookup(&world, &standing(at)).expect("the same contents from the same start");
        assert_eq!(hit.shift, shift);
        let (_, states) = word(&world, &standing(at), &cycle);
        let same = states.iter().zip(&hit.entry.value).all(|(actual, recorded)| {
            let mut moved = recorded.clone();
            moved.position = moved.position.plus(Vec3::new(shift[0] as f64, shift[1] as f64, shift[2] as f64));
            moved.main_supporting_block = moved.main_supporting_block.map(|p| [0, 1, 2].map(|a| p[a] + shift[a]));
            moved.bits() == actual.bits()
        });
        println!("{shift:?}: in scope {}, same bits {same}", hit.exact);
        if hit.exact { assert!(same, "in scope but not bit-exact: {shift:?}"); exact += 1; }
        else if same { replayed += 1; } else { differed += 1; }
    }
    assert!(exact >= 4 && replayed + differed >= 4);
    println!("in scope and exact {exact}; out of scope: {replayed} replayed the same, {differed} differed");
}

// Can a shift that is not a multiple of 16 hit while the law's chunk check sees an unloaded chunk?
#[test]
fn a_shift_across_a_chunk_edge_cannot_hit_when_the_law_sees_it() {
    let water = Block { air: false, fluid: Fluid { kind: FluidKind::Water, height: 0.8, ..Fluid::default() }, ..Block::default() };
    let pool = |at: BlockPos| {
        let mut world = BoxWorld::default();
        floor(&mut world, at, Block::full());
        world.set(at, water.clone());
        world
    };
    // In the middle of chunk 64: every column the fluid check asks about is in it.
    let origin = [1032, 70, 1032];
    let (footprint, states) = word(&pool(origin), &standing(origin), &[Input::default()]);
    assert!(states[0].in_water);
    let mut table = PatternTable::default();
    table.insert(&pool(origin), &footprint, &standing(origin), states).unwrap();
    // Seven east, the body's east column is in chunk 65; unload it.
    let at = [1039, 70, 1032];
    let mut edge = pool(at);
    edge.unloaded_chunks.push([65, 64]);
    let (_, states) = word(&edge, &standing(at), &[Input::default()]);
    assert!(!states[0].in_water, "the law skips fluids beside an unloaded chunk");
    assert!(table.lookup(&edge, &standing(at)).is_none());
    assert!(table.lookup(&pool(at), &standing(at)).is_some_and(|hit| hit.exact), "loaded, the same shift hits");
}

// Independent signed-permutation table for the eight horizontal frames.
fn transform(x: f64, z: f64, frame: Frame) -> (f64, f64) {
    let x = if frame.mirrored { -x } else { x };
    match frame.quarter_turns { 0 => (x, z), 1 => (-z, x), 2 => (-x, -z), _ => (z, -x) }
}

/// An asymmetric directional block, built already turned by `frame`.
fn oriented(frame: Frame) -> Block {
    let (a, b) = transform(-0.5, -0.25, frame);
    let (c, d) = transform(0.25, 0.5, frame);
    let mut block = Block::boxes(&[Aabb::new(a.min(c)+0.5, 0.0, b.min(d)+0.5, a.max(c)+0.5, 0.5, b.max(d)+0.5)]);
    let (flow_x, flow_z) = transform(0.25, -0.5, frame);
    block.fluid = Fluid { kind: FluidKind::Water, height: 0.5, flow: Vec3::new(flow_x, 0.0, flow_z), ..Fluid::default() };
    let (north_x, north_z) = transform(0.0, -1.0, frame);
    let facing = if north_x < 0.0 { 4 } else if north_x > 0.0 { 5 } else if north_z < 0.0 { 2 } else { 3 };
    block.ladder_facing = Some(facing);
    let mut solid_faces = [false; 4];
    solid_faces[[2, 5, 3, 4].iter().position(|&f| f == facing).unwrap()] = true;
    block.fluid_flow = Some(FluidFlowBlock { blocks_motion: true, solid_faces });
    block.stuck_multiplier = Some(if frame.quarter_turns % 2 == 0 { Vec3::new(0.25, 0.5, 0.75) } else { Vec3::new(0.75, 0.5, 0.25) });
    block
}

// Are a model's turns and mirror found by hashing its content, never declared?
#[test]
fn turned_models_are_found_by_hashing() {
    let original = oriented(FRAMES[0]);
    for frame in FRAMES {
        let turned = model_digest(&oriented(frame), FRAMES[0]).unwrap();
        assert_eq!(model_digest(&original, frame).unwrap(), turned, "{frame:?}");
        assert_eq!(FRAMES.iter().filter(|&&f| model_digest(&original, f).unwrap() == turned).count(), 1, "asymmetric {frame:?}");
    }
}

// Is what the key cannot name refused, rather than approximated?
#[test]
fn unknown_physics_is_refused() {
    let mut world = BoxWorld::default();
    let mut footprint = Footprint::default();
    footprint.reads.insert([0, 0, 0], Query::Hooks as u8);
    world.set([0, 0, 0], Block { inside: Inside::Unsupported("x".into()), ..Block::default() });
    let start = standing([0, 0, 0]);
    assert_eq!(PatternTable::default().insert(&world, &footprint, &start, ()).unwrap_err(), PatternError::UnsupportedHook);
    world.set([0, 0, 0], Block::full());
    footprint.environment.push(EnvironmentRead::ExtraCollisions { bounds: Aabb::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
        context: CollisionContext::of(&start), shapes: vec![Block::full().shape] });
    assert_eq!(PatternTable::default().insert(&world, &footprint, &start, ()).unwrap_err(), PatternError::ExternalCollider);
}
