#[path = "support/footprints.rs"]
mod support;
use law::*;
use law::footprint::{EnvironmentRead, Query, RecordingWorld};
use support::{Random, state_bits};

#[test]
fn outside_footprint_changes_cannot_alter_tick() {
    let mut outside_checks = 0;
    let mut changed_inside = 0;
    for seed in 0..24 {
        let mut random = Random(seed);
        let mut world = if seed % 4 == 0 { BoxWorld::default() } else { support::world(&mut random, seed) };
        world.fill([-4, -1, -4], [4, -1, 4], Block::full());
        let initial = PlayerState { position: Vec3::new(0.5, 0.0, 0.5), on_ground: true,
            velocity: Vec3::new(random.coordinate(), -0.0784, random.coordinate()), ..PlayerState::default() };
        let input = random.input();
        let mut expected = initial.clone();
        let record = RecordingWorld::new(&world).tick(&mut expected, &input);
        // Randomized finite neighborhood, plus a guaranteed remote edit. Include
        // shape, hook and fluid replacements, even when they enable a global flag.
        let mut candidates = vec![[20, 0, 20], [0, 0, 0], [0, -1, 0]];
        for _ in 0..32 { candidates.push([random.int(15)-7, random.int(9)-3, random.int(15)-7]); }
        for p in candidates {
            for replacement in [Block::full(), Block { air: false, inside: Inside::Cobweb, ..Block::default() },
                Block { air: false, fluid: Fluid { kind: FluidKind::Water, height: 1.0, ..Fluid::default() }, ..Block::default() }] {
                let mut edited = world.clone(); edited.set(p, replacement);
                let mut actual = initial.clone(); tick(&mut actual, &edited, &input);
                if !record.footprint.reads.contains_key(&p) {
                    outside_checks += 1;
                    assert_eq!(state_bits(&actual), state_bits(&expected), "outside seed {seed} cell {p:?}");
                } else if state_bits(&actual) != state_bits(&expected) { changed_inside += 1; }
            }
        }
    }
    assert!(outside_checks > 0); assert!(changed_inside > 0, "positive control must change a tick");
    println!("outside edits {outside_checks}, influential inside edits {changed_inside}");
}

/// `other`, carrying `original`'s answers to the queries in `mask`.
fn keep(original: &Block, other: Block, mask: u8) -> Block {
    let q = |query: Query| mask & query as u8 != 0;
    if q(Query::Block) { return original.clone(); }
    let mut block = if q(Query::Hooks) { Block { shape: other.shape, above_shape: other.above_shape,
        below_shape: other.below_shape, sneak_shape: other.sneak_shape, ..original.clone() } } else { other };
    if q(Query::Fluid) { block.fluid = original.fluid; block.fluid_flow = original.fluid_flow; }
    if q(Query::Shape) {
        block.shape = original.shape.clone(); block.above_shape = original.above_shape.clone();
        block.below_shape = original.below_shape.clone(); block.sneak_shape = original.sneak_shape.clone();
        if (block.inside == Inside::PowderSnow) != (original.inside == Inside::PowderSnow) { block.inside = original.inside.clone(); }
    }
    block
}

// The key's induction: a cell edit that keeps the answers to the queries that read it cannot alter the tick.
#[test]
fn edits_that_keep_a_cells_answers_cannot_alter_tick() {
    let (mut kept, mut varied) = (0, 0);
    for seed in 0..36 {
        let mut random = Random(seed + 100);
        let mut world = if seed < 24 { support::world(&mut random, seed) } else {
            // Raw water: its current reads the neighbours' fluid only (Fluid and Loaded).
            let mut world = BoxWorld::default();
            world.fill([-3, 0, -3], [3, 2, 3], Block { fluid_flow: Some(FluidFlowBlock::default()), ..Block::default() });
            for x in -1..=1 { world.set([x, 0, random.int(3)-1], Block { air: false, fluid: Fluid { kind: FluidKind::Water,
                raw_amount: Some(1 + random.int(8) as u8), raw_falling: Some(false), ..Fluid::default() },
                fluid_flow: Some(FluidFlowBlock::default()), ..Block::default() }); }
            world
        };
        world.fill([-4, -1, -4], [4, -1, 4], Block { fluid_flow: Some(FluidFlowBlock { blocks_motion: true, solid_faces: [true; 4] }), ..Block::full() });
        let initial = PlayerState { position: Vec3::new(0.5, 0.0, 0.5), on_ground: true,
            velocity: Vec3::new(random.coordinate(), -0.0784, random.coordinate()), ..PlayerState::default() };
        let input = random.input();
        let mut expected = initial.clone();
        let record = RecordingWorld::new(&world).tick(&mut expected, &input);
        for (&p, &mask) in &record.footprint.reads {
            let original = world.block(p).clone();
            let mut other = support::block(&mut random);
            other.friction = 0.5 + random.int(8) as f32 / 16.0;
            other.jump_factor = 0.5 + random.int(2) as f32 / 2.0;
            let mut edit = keep(&original, other, mask);
            // Then one field at random: the digest, not this test, decides whether the answers held.
            let water = Fluid { kind: FluidKind::Water, raw_amount: Some(1 + random.int(8) as u8), raw_falling: Some(random.int(2) == 0), ..Fluid::default() };
            match random.int(13) {
                0 => edit.friction = 0.98, 1 => edit.speed_factor = 0.4, 2 => edit.fluid = water,
                3 => edit.fluid_flow = Some(FluidFlowBlock { blocks_motion: true, solid_faces: [true; 4] }),
                4 => edit.shape = Block::full().shape, 5 => edit.inside = Inside::Cobweb, 6 => edit.climbable = true,
                7 => edit.suffocating = !edit.suffocating, 8 => edit.air = !edit.air, 9 => edit.bounce = 1.0,
                10 => edit.sneak_shape = Some(VoxelShape::default()), 11 => edit.above_shape = Some(VoxelShape::default()),
                _ => edit.inside = Inside::PowderSnow,
            }
            let loaded = world.loaded(p);
            if law::pattern::cell_digest(&edit, loaded, mask) != law::pattern::cell_digest(&original, loaded, mask) { continue; }
            varied += (law::pattern::model_digest(&edit, law::pattern::FRAMES[0]) != law::pattern::model_digest(&original, law::pattern::FRAMES[0])) as u32;
            let mut edited = world.clone(); edited.set(p, edit);
            let mut actual = initial.clone(); tick(&mut actual, &edited, &input);
            assert_eq!(state_bits(&actual), state_bits(&expected), "seed {seed} cell {p:?} mask {mask}");
            kept += 1;
        }
    }
    assert!(varied > 0, "the edits must change what the queries did not read");
    println!("{kept} edits kept their cell's answers, {varied} of them changed the rest of the model");
}

#[test]
fn recording_replays_bits_and_accumulates_tick_worldtubes() {
    for seed in 0..16 {
        let mut random = Random(seed);
        let world = support::world(&mut random, seed);
        let recorder = RecordingWorld::new(&world);
        let mut plain = PlayerState { position: Vec3::new(0.5, 2.0, 0.5), ..PlayerState::default() };
        let mut recorded = plain.clone();
        let mut union = law::footprint::Footprint::default();
        for _ in 0..8 {
            let input = random.input();
            tick(&mut plain, &world, &input);
            let record = recorder.tick(&mut recorded, &input);
            let mut replay = record.initial.clone();
            tick(&mut replay, &support::FullScan(&world), &record.input);
            assert_eq!(state_bits(&plain), state_bits(&recorded), "seed {seed}");
            assert_eq!(state_bits(&replay), state_bits(&recorded), "replay seed {seed}");
            assert!(!record.footprint.swept.is_empty());
            union.extend(&record.footprint);
        }
        assert_eq!(union.reads, recorder.footprint().reads);
        assert_eq!(union.swept, recorder.take().swept);
        assert!(recorder.footprint().reads.is_empty());
    }
}

#[test]
fn raw_fluid_neighbors_and_query_kinds_are_visible() {
    let mut world = BoxWorld::default();
    world.fill([-2, -2, -2], [2, 2, 2], Block { fluid_flow: Some(FluidFlowBlock::default()), ..Block::default() });
    world.set([0, 0, 0], Block { air: false, fluid: Fluid { kind: FluidKind::Water,
        raw_amount: Some(4), raw_falling: Some(true), ..Fluid::default() },
        fluid_flow: Some(FluidFlowBlock::default()), ..Block::default() });
    let recorder = RecordingWorld::new(&world);
    let mut state = PlayerState { position: Vec3::new(0.5, 0.0, 0.5), ..PlayerState::default() };
    let record = recorder.tick(&mut state, &Input::default());
    for p in [[0, 1, 0], [0, 0, -1], [0, -1, -1], [0, 1, -1], [1, 0, 0]] {
        assert_ne!(record.footprint.reads[&p] & Query::Fluid as u8, 0, "{p:?}");
    }
    assert!(record.footprint.reads.values().any(|q| q & Query::Shape as u8 != 0));
    assert!(record.footprint.reads.values().any(|q| q & Query::Hooks as u8 != 0));
    assert!(record.footprint.reads.values().any(|q| q & Query::Loaded as u8 != 0));
    recorder.block([9, 9, 9]);
    assert_eq!(recorder.footprint().reads[&[9, 9, 9]], Query::Block as u8);
}

#[test]
fn explicit_server_time_is_recorded_and_replayed() {
    let mut world = BoxWorld::default();
    world.difficulty = 0;
    let recorder = RecordingWorld::new(&world);
    let initial = PlayerState { health: 10.0, food_level: 10, saturation: 0.0, ..PlayerState::default() };
    let mut state = initial.clone();
    let record = recorder.server_event(&mut state, ServerResourceEvent::Regeneration(20));
    assert!(matches!(record.event, ServerResourceEvent::Regeneration(20)));
    let mut replay = initial.clone();
    apply_server_resource_event(&mut replay, &world, record.event);
    assert_eq!(state_bits(&state), state_bits(&replay));
    assert!(record.footprint.environment.iter().any(|r| matches!(r, EnvironmentRead::Difficulty(0))));
    let mut other_time = initial;
    apply_server_resource_event(&mut other_time, &world, ServerResourceEvent::Regeneration(19));
    assert_ne!(state_bits(&state), state_bits(&other_time));
}

#[test]
fn extra_colliders_preserve_bits_and_keep_query_dependencies() {
    let mut world = BoxWorld::default();
    world.extra_shapes.push(VoxelShape::boxes(&[Aabb::new(0.75, 0.0, 0.0, 1.0, 2.0, 1.0)]));
    let recorder = RecordingWorld::new(&world);
    let mut state = PlayerState { position: Vec3::new(0.5, 0.0, 0.5), velocity: Vec3::new(0.5, 0.0, 0.0), ..PlayerState::default() };
    let mut plain = state.clone();
    recorder.tick(&mut state, &Input::default()); tick(&mut plain, &world, &Input::default());
    assert_eq!(state_bits(&state), state_bits(&plain));
    assert!(recorder.footprint().environment.iter().any(|r|
        matches!(r, EnvironmentRead::ExtraCollisions { shapes, .. } if !shapes.is_empty())));
}
