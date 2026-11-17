use law::{*, codec::{self, Interner, LegsRole, Projection, UnportedVerb, FineId, ProjectionTable},
    codec::planes::{Plane, SectionRoles, Missing, WORDS}, codec::proof::{Answer, Binding, ReadCertificates, SnapshotSource, Refusal},
    footprint::{Footprint, Query, RecordingWorld}, pattern::PatternTable};

fn source(world: &BoxWorld) -> SnapshotSource<'_, BoxWorld> {
    SnapshotSource { world, ticks: (0, 10), min: [-4; 3], max: [4; 3] }
}

#[test]
fn forced_collision_separates_footprints_patterns_and_presence() {
    let mut world = BoxWorld::default();
    world.difficulty = 0;
    let start = PlayerState::default();
    let mut fp = Footprint::default();
    fp.reads.insert([0; 3], Query::Block as u8 | Query::Loaded as u8);
    let mut table = PatternTable::default();
    *table.codec_mut() = Interner::with_digest(|_| 0);
    let air = fp.answers(&world, &start, table.codec_mut()).unwrap();
    let a = table.insert(&world, &fp, &start, 1).unwrap();
    let air_presence = law::pattern::law_digest(world.block([0; 3])).unwrap();
    world.set([0; 3], Block::full());
    let stone = fp.answers(&world, &start, table.codec_mut()).unwrap();
    assert_ne!(air[0].answer, stone[0].answer);
    assert!(table.lookup(&world, &start).is_none());
    let b = table.insert(&world, &fp, &start, 2).unwrap();
    assert_ne!(a, b);
    let stone_presence = law::pattern::law_digest(world.block([0; 3])).unwrap();
    let x = table.codec_mut().intern(air_presence.bytes()).unwrap();
    let y = table.codec_mut().intern(stone_presence.bytes()).unwrap();
    assert_ne!(x, y);
    assert_ne!(air_presence, stone_presence);
    assert!(table.digests().iter().all(|&h| h == 0));
    let exact = codec::state_key(codec::StateQuestion::Exact, &[1,2]);
    let comparisons = codec::state_key(codec::StateQuestion::Comparisons, &[1,2]);
    let recurrence = codec::state_key(codec::StateQuestion::Recurrence, &[1,2]);
    let x = table.codec_mut().intern(&exact).unwrap();
    let y = table.codec_mut().intern(&comparisons).unwrap();
    let z = table.codec_mut().intern(&recurrence).unwrap();
    assert_ne!(x,y); assert_ne!(y,z); assert_ne!(x,z);
}

#[test]
fn raw_fluid_neighbor_question_retains_answers_omitted_by_the_legs_projection() {
    let mut world = BoxWorld::default(); world.difficulty = 0;
    let mut wall = Block { fluid_flow: Some(FluidFlowBlock { blocks_motion: true, solid_faces: [false;4] }), ..Block::full() };
    let original = wall.clone();
    let fluid = Fluid { kind: FluidKind::Water, raw_amount: Some(4), raw_falling: Some(true), ..Fluid::default() };
    world.set([0,0,-1], wall.clone());
    let without = fluid.current(&world, [0;3]);
    wall.fluid_flow.as_mut().unwrap().solid_faces[0] = true;
    world.set([0,0,-1], wall.clone());
    assert_ne!(without, fluid.current(&world, [0;3]));
    assert_eq!(codec::legs_bytes(&original, true, LegsRole::Inside), codec::legs_bytes(&wall, true, LegsRole::Inside));
    assert_ne!(codec::cell(&original, true, Query::Fluid as u8), codec::cell(&wall, true, Query::Fluid as u8));
}

#[test]
fn labels_kits_and_ported_mining_queries_keep_identity() {
    let mut codec = Interner::with_digest(|_| 0);
    let mut bare = ProjectionTable::new(b"26.2-registry", b"bare");
    let mut silk = ProjectionTable::new(b"26.2-registry", b"silk");
    let ground = Block::full();
    let legs = bare.legs(&mut codec, FineId(1), &ground, true, LegsRole::Support02).unwrap();
    assert_eq!(legs, silk.legs(&mut codec, FineId(1), &ground, true, LegsRole::Support02).unwrap());
    let unknown = bare.verb(&mut codec, FineId(1), UnportedVerb::Break).unwrap();
    assert!(matches!(unknown, Projection::Unknown(_)));
    assert_ne!(unknown, silk.verb(&mut codec, FineId(1), UnportedVerb::Break).unwrap());
    assert_ne!(unknown, bare.verb(&mut codec, FineId(2), UnportedVerb::Break).unwrap());
    assert_ne!(unknown, bare.verb(&mut codec, FineId(1), UnportedVerb::Place).unwrap());
    let sample = law::mining::MiningSample { target: law::mining::MiningTarget { position: [0;3], identity: 1 }, destroy_speed: 8.0, hardness: 1.5, correct_tool: true };
    assert_eq!(sample.progress().to_bits(), (8.0f32 / 1.5 / 30.0).to_bits());
    let known = codec.mining_progress(sample, b"pick").unwrap();
    assert!(matches!(known, Projection::Known(_)));
    assert_ne!(known, codec.mining_progress(law::mining::MiningSample { correct_tool: false, ..sample }, b"pick").unwrap());
    let support = [legs];
    let label = codec.label(codec::Label { law_schema: codec::SCHEMA, body_kit: b"bare", support: &support, medium: &[] }).unwrap();
    assert_eq!(label, codec.label(codec::Label { law_schema: codec::SCHEMA, body_kit: b"bare", support: &support, medium: &[] }).unwrap());
    assert_ne!(label, codec.label(codec::Label { law_schema: codec::SCHEMA, body_kit: b"silk", support: &support, medium: &[] }).unwrap());
    assert_ne!(label, codec.label(codec::Label { law_schema: b"other-law", body_kit: b"bare", support: &support, medium: &[] }).unwrap());
    assert_ne!(label, codec.label(codec::Label { law_schema: codec::SCHEMA, body_kit: b"bare", support: &[], medium: &support }).unwrap());
}

#[test]
fn local_role_planes_ignore_distant_cobweb_and_refuse_missing_bits() {
    let mut codec = Interner::default();
    let air = Block::default();
    let web = Block { inside: Inside::Cobweb, air: false, ..air.clone() };
    let plain = codec.legs(&air, true, LegsRole::Inside).unwrap();
    assert_ne!(plain, codec.legs(&web, true, LegsRole::Inside).unwrap());
    let mut section = SectionRoles { complete: Plane::AllOne, loaded: Plane::AllOne, roles: std::array::from_fn(|_| Plane::AllZero) };
    section.roles[LegsRole::Inside as usize].set(4000, true);
    assert_eq!(section.query(LegsRole::Inside, &[(0, 1)]), Ok(vec![]));
    section.roles[LegsRole::Inside as usize].set(0, true);
    assert_eq!(section.query(LegsRole::Inside, &[(0, 1)]), Ok(vec![(0, 1)]));
    section.complete.set(1, false);
    assert_eq!(section.query(LegsRole::Inside, &[(0, 2)]), Err(Missing::Coverage));
    section.loaded.set(2, false);
    assert_eq!(section.query(LegsRole::Inside, &[(0, 4)]), Err(Missing::Loaded));
    assert_eq!(section.query(LegsRole::Inside, &[(WORDS, 0)]), Err(Missing::Address));
    assert_eq!(Plane::from_words([0; WORDS]), Plane::AllZero);
    assert_eq!(Plane::from_words([u64::MAX; WORDS]), Plane::AllOne);
    let mut world = BoxWorld::default(); world.difficulty = 0;
    world.fill([-3,-1,-3], [3,-1,3], Block::full());
    let state = PlayerState { position: Vec3::new(0.5,0.0,0.5), on_ground: true, ..PlayerState::default() };
    let mut original = state.clone();
    let fp = RecordingWorld::new(&world).tick(&mut original, &Input::default()).footprint;
    let binding = Binding::new(&state, b"local-inside-tube", law::LAW_HASH);
    let mut certs = ReadCertificates::default();
    let id = certs.capture(&mut codec, binding.clone(), &[(0,&fp)], &source(&world)).unwrap();
    world.set([20,0,20], web.clone());
    assert!(certs.edit(&codec, [20,0,20], (0,10), &source(&world)).is_empty());
    let mut far = state.clone(); tick(&mut far, &world, &Input::default());
    assert_eq!(original.bits(), far.bits());
    assert!(matches!(certs.answer(id,&binding), Answer::Yes(_)));
    world.set([0,0,0], web);
    assert_eq!(certs.edit(&codec, [0,0,0], (0,10), &source(&world)), vec![id]);
    assert!(matches!(certs.answer(id,&binding), Answer::No(_)));
}

#[test]
fn exact_edits_state_version_scope_and_owner_are_enforced() {
    let mut codec = Interner::with_digest(|_| 0);
    let mut world = BoxWorld::default();
    world.difficulty = 0;
    let state = PlayerState::default();
    let binding = Binding::new(&state, b"word/kit/context", law::LAW_HASH);
    let mut fp = Footprint::default();
    fp.reads.insert([0;3], Query::Shape as u8);
    let mut certificates = ReadCertificates::default();
    let id = certificates.capture(&mut codec, binding.clone(), &[(2, &fp)], &source(&world)).unwrap();
    let Answer::Yes(proof) = certificates.answer(id, &binding) else { panic!("read certificate") };
    assert_eq!(certificates.validate(&proof, id, &binding), Ok(()));
    let mut changed = state.clone(); changed.velocity.x = 1.0;
    assert!(matches!(certificates.answer(id, &Binding::new(&changed, b"word/kit/context", law::LAW_HASH)), Answer::Unknown(Refusal::StateOrContext)));
    assert!(matches!(certificates.answer(id, &Binding::new(&state, b"word/kit/context", [0;32])), Answer::Unknown(Refusal::Version)));
    assert!(matches!(certificates.answer(id, &Binding::new(&state, b"different-word", law::LAW_HASH)), Answer::Unknown(Refusal::StateOrContext)));
    let mut other = ReadCertificates::default();
    other.capture(&mut codec, binding.clone(), &[(2, &fp)], &source(&world)).unwrap();
    assert_eq!(other.validate(&proof, id, &binding), Err(Refusal::UnknownCertificate));
    let another = certificates.capture(&mut codec, binding.clone(), &[(4, &fp)], &source(&world)).unwrap();
    assert_eq!(certificates.validate(&proof, another, &binding), Err(Refusal::Question));
    assert!(certificates.edit(&codec, [1;3], (0,10), &source(&world)).is_empty());
    world.set([0;3], Block { world_hook: true, ..Block::default() });
    assert!(certificates.edit(&codec, [0;3], (0,10), &source(&world)).is_empty());
    world.set([0;3], Block::full());
    assert!(certificates.edit(&codec, [0;3], (3,4), &source(&world)).is_empty());
    assert_eq!(certificates.validate(&proof, id, &binding), Ok(()));
    assert_eq!(certificates.edit(&codec, [0;3], (2,3), &source(&world)), vec![id]);
    assert!(matches!(certificates.answer(id, &binding), Answer::No(_)));
    assert_eq!(certificates.validate(&proof, id, &binding), Err(Refusal::Changed));
    let outside = SnapshotSource { min: [1;3], ..source(&world) };
    assert_eq!(other.capture(&mut codec, binding, &[(2, &fp)], &outside), Err(Refusal::Coverage));
}

#[test]
fn environment_schedule_and_lost_coverage_are_exact_and_unknown_never_becomes_no() {
    let mut codec = Interner::default();
    let mut world = BoxWorld::default(); world.difficulty = 0;
    let binding = Binding::new(&PlayerState::default(), b"environment-read", law::LAW_HASH);
    let mut fp = Footprint::default();
    fp.environment.push(law::footprint::EnvironmentRead::MinY(world.minimum_y));
    let mut certs = ReadCertificates::default();
    let id = certs.capture(&mut codec, binding.clone(), &[(2,&fp)], &source(&world)).unwrap();
    assert!(certs.environment_event(&codec, (2,3), &source(&world)).is_empty());
    world.minimum_y += 1;
    assert!(certs.environment_event(&codec, (3,4), &source(&world)).is_empty());
    assert_eq!(certs.environment_event(&codec, (2,3), &source(&world)), vec![id]);
    assert!(matches!(certs.answer(id,&binding), Answer::No(_)));
    let mut cell = Footprint::default(); cell.reads.insert([0;3], Query::Shape as u8);
    let id = certs.capture(&mut codec, binding.clone(), &[(2,&cell)], &source(&world)).unwrap();
    let missing = SnapshotSource { min: [1;3], ..source(&world) };
    assert_eq!(certs.edit(&codec, [0;3], (2,3), &missing), vec![id]);
    assert!(matches!(certs.answer(id,&binding), Answer::Unknown(Refusal::Coverage)));
}

#[test]
fn native_wall_release_and_sneak_backoff_reads_invalidate() {
    for sneak in [false, true] {
        let mut world = BoxWorld::default(); world.difficulty = 0;
        world.set([0,-1,0], Block::full());
        if !sneak {
            world.fill([1,0,-2], [1,0,2], Block::full());
            world.fill([-2,0,1], [2,0,1], Block::full());
        }
        let state = PlayerState { position: Vec3::new(0.6,0.0,0.6), velocity: Vec3::new(if sneak {0.9} else {0.4},-0.0784, if sneak {0.0} else {0.4}), on_ground: true,
            main_supporting_block: Some([0,-1,0]), ..PlayerState::default() };
        let input = Input { sneak, ..Input::default() };
        let mut before = state.clone();
        let fp = RecordingWorld::new(&world).tick(&mut before, &input).footprint;
        let position = if sneak { [1,-1,0] } else { [1,0,0] };
        assert!(fp.reads[&position] & Query::Shape as u8 != 0, "sneak={sneak}: {position:?}");
        let binding = Binding::new(&state, b"native-read-witness", law::LAW_HASH);
        let mut codec = Interner::default();
        let mut certs = ReadCertificates::default();
        let id = certs.capture(&mut codec, binding.clone(), &[(0,&fp)], &source(&world)).unwrap();
        if sneak { world.set(position, Block::full()); }
        else { world.set(position, Block::default()); }
        let mut after = state.clone(); tick(&mut after, &world, &input);
        assert_ne!(before.position.x.to_bits(), after.position.x.to_bits(), "positive control sneak={sneak}");
        if !sneak { assert_eq!(before.horizontal_collision, after.horizontal_collision); }
        assert_eq!(certs.edit(&codec, position, (0,1), &source(&world)), vec![id]);
        assert!(matches!(certs.answer(id,&binding), Answer::No(_)));
    }
}
