use law::{*, codec::{Interner, proof::{Answer, Binding, ReadCertificates, SnapshotSource}}, state_reads::{StateField, StateReads}};
use words::*;

#[test]
fn collision_separates_proposal_classes_and_timed_certificates_use_the_same_owner() {
    let mut world = BoxWorld::default(); world.difficulty = 0;
    world.fill([-3,-1,-3],[3,-1,3],Block::full());
    let start = PlayerState { on_ground: true, position: Vec3::new(0.5,0.0,0.5), ..PlayerState::default() };
    let program = Program { rows: vec![Row { when: Predicate::Always, input: Input { forward: true, ..Input::default() } }] };
    let args = Arguments::default();
    let segment = run_segment(&world, &start, &program, args).unwrap();
    assert!(!segment.timed_reads.is_empty());
    let mut table = ProposalTable::new(|s, _| class(s, StateReads(1 << StateField::leather_boots as u8)));
    *table.codec_mut() = Interner::with_digest(|_| 0);
    table.learn(&world, &start, &program, args, &segment).unwrap();
    assert_eq!(table.proposals(&start).len(), 1);
    let changed = PlayerState { leather_boots: true, ..start.clone() };
    assert!(table.proposals(&changed).is_empty(), "a colliding state must not borrow another class");
    table.learn(&world, &changed, &program, args, &segment).unwrap();
    assert_eq!(table.classes().len(), 2);
    assert_eq!(table.proposals(&start).len(), 1);
    assert_eq!(table.proposals(&changed).len(), 1);
    assert!(table.codec().digests().iter().all(|&hash| hash == 0));
    let binding = Binding::new(&start, b"forward/bare/native", 1);
    let mut certificates = ReadCertificates::default();
    let source = SnapshotSource { world: &world, ticks: (0, u64::MAX), min: [-4;3], max: [4;3] };
    let id = segment.certify_reads(table.codec_mut(), &mut certificates, binding.clone(), &source).unwrap();
    let Answer::Yes(proof) = certificates.answer(id, &binding) else { panic!("certified native read set") };
    assert_eq!(certificates.validate(&proof, id, &binding), Ok(()));
}
