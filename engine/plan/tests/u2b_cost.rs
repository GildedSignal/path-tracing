//! Frozen U2b cost gate. The measured boundary is the public solve call.
use std::{alloc::{GlobalAlloc,Layout,System},cell::Cell,time::Instant};
use law::{Block,PlayerState,codec::Interner};
use plan::{Goal,Kit,PlanOutcome,source::{GeneratorSuperflatSource,MeteredWorld},store::GeneratorStore};
struct Counting;
std::thread_local! {
    static ENABLED: Cell<bool> = const { Cell::new(false) };
    static ALLOCATIONS: Cell<u64> = const { Cell::new(0) };
}
fn allocated() {if ENABLED.with(Cell::get) {ALLOCATIONS.with(|n|n.set(n.get()+1));}}
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self,l:Layout)->*mut u8 {allocated();unsafe {System.alloc(l)}}
    unsafe fn alloc_zeroed(&self,l:Layout)->*mut u8 {allocated();unsafe {System.alloc_zeroed(l)}}
    unsafe fn realloc(&self,p:*mut u8,l:Layout,n:usize)->*mut u8 {allocated();unsafe {System.realloc(p,l,n)}}
    unsafe fn dealloc(&self,p:*mut u8,l:Layout) {unsafe {System.dealloc(p,l)}}
}
#[global_allocator] static ALLOCATOR:Counting=Counting;
fn measure<T>(f:impl FnOnce()->T)->(T,u64,u64,u128) {
    let before=law::player_state_copies();ALLOCATIONS.with(|n|n.set(0));
    let clock=Instant::now();ENABLED.with(|on|on.set(true));let value=f();ENABLED.with(|on|on.set(false));
    let elapsed=clock.elapsed().as_nanos();
    (value,law::player_state_copies()-before,ALLOCATIONS.with(Cell::get),elapsed)
}
#[test]
fn u2b_counters_detect_work() {
    let state=PlayerState::default();
    let (copy,copies,_,_)=measure(||std::hint::black_box(state.clone()));
    assert_eq!(copies,1);assert_eq!(copy.bits(),state.bits());
    let (allocation,_,allocations,_)=measure(||std::hint::black_box(Box::new([0u8;1024])));
    assert_eq!(allocations,1);drop(allocation);
    println!("U2B_COUNTERS clone_detected=1 allocation_detected=1");
}
#[test]
fn u2_warm_zero_copies_and_allocations() {
    let source=GeneratorSuperflatSource::new(-61,Block::full(),[-3_000_000,-64,-3_000_000],[3_000_000,20,3_000_000]);
    let start=plan::settle(&plan::Still(&source),[0.5,-60.0,0.5],0).unwrap();
    let mut failures=Vec::new();
    for cell in [[1_000_000,-61,0],[0,-61,1_000_000],[1_000_000,-61,330_000]] {
        let goal=Goal {cell};let store=GeneratorStore::default();let mut codec=Interner::default();
        let cold=plan::solve(&start,goal,Kit::BareLegs,&source,&store,&mut codec,u64::MAX);
        let PlanOutcome::Ready(cold)=cold.outcome else {panic!("cold solve refused")};
        let counted=MeteredWorld::new(&source);
        let mut times=Vec::with_capacity(100);let mut profile=Vec::with_capacity(100);
        let mut worst=(0,0);
        for _ in 0..100 {
            let (answer,copies,allocations,ns)=measure(||std::hint::black_box(plan::solve(&start,goal,Kit::BareLegs,&counted,&store,&mut codec,u64::MAX)));
            let PlanOutcome::Ready(answer)=answer.outcome else {panic!("warm solve refused")};
            assert_eq!(answer.ticks,cold.ticks);assert_eq!(answer.end.bits(),cold.end.bits());
            assert_eq!(counted.take_ticks(),0);
            times.push(ns);profile.push([answer.thinking.entry_ns,answer.thinking.bearing_ns,answer.thinking.inverse_ns,answer.thinking.output_ns]);
            worst.0=worst.0.max(copies);worst.1=worst.1.max(allocations);
        }
        times.sort_unstable();
        println!("U2B_WARM cell={cell:?} n=100 ns_min={} ns_median={} ns_p99={} ns_max={} copies_max={} allocations_max={} samples_ns={times:?}",times[0],(times[49]+times[50])/2,times[98],times[99],worst.0,worst.1);
        for (column,name) in ["entry","bearing","inverse","output"].into_iter().enumerate() {
            let mut values:Vec<_>=profile.iter().map(|v|v[column]).collect();values.sort_unstable();
            println!("U2B_PROFILE cell={cell:?} stage={name} ns_min={} ns_median={} ns_p99={} ns_max={}",values[0],(values[49]+values[50])/2,values[98],values[99]);
        }
        if worst!=(0,0) {failures.push((cell,worst));}
    }
    assert!(failures.is_empty(),"warm solve must copy zero PlayerStates and allocate zero times: {failures:?}");
}

#[test]
fn u2_probe_borrows_phase_without_copy_or_allocation() {
    let source=GeneratorSuperflatSource::new(-61,Block::full(),[-3_000_000,-64,-3_000_000],[3_000_000,20,3_000_000]);
    let start=plan::settle(&plan::Still(&source),[0.5,-60.0,0.5],0).unwrap();
    let store=GeneratorStore::default();let mut codec=Interner::default();
    let label=plan::source::label(&mut codec,&source,&start).unwrap();
    for cell in [[1_000_000,-61,0],[1_000_000,-61,330_000]] {
        let pair={let d=Goal {cell}.center().minus(start.position);plan::lattice::bracket(d.x,d.z)};
        let kernel=store.prepare(label,&source,&start,plan::store::SPRINT_JUMP,pair[0]).unwrap();
        for cycle_only in [false,true] {
            for ticks in [0,1,12,263,2806325,2955181] {
                let expected=if cycle_only {kernel.cycle_at(&start,ticks)} else {kernel.at(&start,ticks)};
                let (probe,copies,allocations,_)=measure(||std::hint::black_box(kernel.probe(&start,ticks,cycle_only)));
                assert_eq!(copies,0);assert_eq!(allocations,0);
                assert_eq!([probe.position.x.to_bits(),probe.position.y.to_bits(),probe.position.z.to_bits()],
                    [expected.position.x.to_bits(),expected.position.y.to_bits(),expected.position.z.to_bits()]);
                assert_eq!(probe.t,expected.t);assert_eq!(probe.phase.velocity,expected.velocity);
                assert!(kernel.cycle_states.iter().chain(&kernel.transient_states).any(|s|std::ptr::eq(s,probe.phase)));
            }
        }
    }
    println!("U2B_PROBE copies=0 allocations=0 worst_position_error_ulp=0 borrowed_template=true");
}
#[test]
fn u2_terminal_joint_reuses_face_preimages() {
    let source=GeneratorSuperflatSource::new(-61,Block::full(),[-3_000_000,-64,-3_000_000],[3_000_000,20,3_000_000]);
    let start=plan::settle(&plan::Still(&source),[0.5,-60.0,0.5],0).unwrap();
    let store=GeneratorStore::default();let mut codec=Interner::default();
    let first=Goal {cell:[1_000_000,-61,0]};let next=Goal {cell:[1_000_016,-61,0]};
    assert!(matches!(plan::solve(&start,first,Kit::BareLegs,&source,&store,&mut codec,u64::MAX).outcome,PlanOutcome::Ready(_)));
    let first_work=store.joints.borrow().work;assert!(first_work.terminal_builds>0);
    // The first different goal reuses the face table, but may compile new
    // parity/binade graphs. It is not the warm arithmetic solve measured below.
    assert!(matches!(plan::solve(&start,next,Kit::BareLegs,&source,&store,&mut codec,u64::MAX).outcome,PlanOutcome::Ready(_)));
    assert_eq!(store.joints.borrow().work.terminal_builds,first_work.terminal_builds);
    let before=store.joints.borrow().work;let count=store.joints.borrow().terminal_count();
    let (answer,copies,allocations,_)=measure(||plan::solve(&start,next,Kit::BareLegs,&source,&store,&mut codec,u64::MAX));
    let PlanOutcome::Ready(answer)=answer.outcome else {panic!("face reuse refused")};
    let after=store.joints.borrow().work;assert_eq!(after.terminal_builds,before.terminal_builds);assert_eq!(store.joints.borrow().terminal_count(),count);
    assert!(after.terminal_lookups>before.terminal_lookups);assert_eq!((copies,allocations),(0,0));
    let fresh=GeneratorStore::default();let mut codec=Interner::default();
    let PlanOutcome::Ready(expected)=plan::solve(&start,next,Kit::BareLegs,&source,&fresh,&mut codec,u64::MAX).outcome else {panic!("fresh solve refused")};
    assert_eq!(answer.ticks,expected.ticks);assert_eq!(answer.end.bits(),expected.end.bits());
    let mut changed=start.clone();changed.yaw=11.0;changed.pitch=80.0;changed.input=law::Input {back:true,..Default::default()};
    let label=plan::source::label(&mut codec,&source,&changed).unwrap();let pair=plan::lattice::bracket(next.center().x-changed.position.x,0.0);
    assert!(fresh.bundle(label,&changed,plan::store::SPRINT_JUMP,pair).is_some(),"dead aim must not change the motor key");
    println!("U2B_TERMINAL goals=2 shared_face_sets={count} warm_builds=0 warm_lookups={} copies=0 allocations=0 exact_fresh=true dead_aim_motor_reused=true",after.terminal_lookups-before.terminal_lookups);
}
