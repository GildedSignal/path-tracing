//! The real thinking time per command (S223): the world is in memory, the bot's process lives on (one
//! store, one codec for the whole series), and every command is a new goal, as in play. No sample is
//! taken on a fresh store, and no sample repeats a goal already asked, except the series marked
//! "same goal again", which shows what the old "warm" number measured.
use law::{Block,codec::Interner};
use plan::{solve,Goal,Kit,PlanOutcome,source::GeneratorSuperflatSource,store::GeneratorStore};
use std::time::Instant;

// Count requested allocation bytes (including realloc's new size) within
// the public call. The instrumented build also allocates its diagnostic set.
use std::{alloc::{GlobalAlloc,Layout,System},cell::Cell};
struct Allocator;
thread_local! {static BYTES:Cell<Option<usize>>=const {Cell::new(None)};}
fn allocated(size:usize) {if let Some(old)=BYTES.get() {BYTES.set(Some(old+size));}}
unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self,l:Layout)->*mut u8 {allocated(l.size());unsafe {System.alloc(l)}}
    unsafe fn alloc_zeroed(&self,l:Layout)->*mut u8 {allocated(l.size());unsafe {System.alloc_zeroed(l)}}
    unsafe fn realloc(&self,p:*mut u8,l:Layout,n:usize)->*mut u8 {allocated(n);unsafe {System.realloc(p,l,n)}}
    unsafe fn dealloc(&self,p:*mut u8,l:Layout) {unsafe {System.dealloc(p,l)}}
}
#[global_allocator] static ALLOCATOR:Allocator=Allocator;

fn lcg(s:&mut u64)->u64 {*s=s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);*s>>33}

fn stats(name: &str, times: &mut [u128], refused: usize, floor_ns: f64) {
    times.sort_unstable();
    let n=times.len();
    let median=(times[(n-1)/2]+times[n/2]) as f64/2.0;
    println!("REAL_TIME series={name:?} n={n} refused={refused} min_ns={} median_ns={median} p90_ns={} max_ns={} floor_ns={floor_ns} median_gap_oom={:.3} samples_ns={times:?}",
        times[0],times[(9*n).div_ceil(10)-1],times[n-1],(median/floor_ns).log10());
}

fn command(from: &law::PlayerState, origin: &str, goal: Goal, source: &GeneratorSuperflatSource,
           store: &GeneratorStore, codec: &mut Interner, times: &mut Vec<u128>) -> Option<plan::solve::Plan> {
    let before=store.preparation_ticks();
    #[cfg(feature="solve-counters")]
    plan::solve_meter::begin(from.position,goal.center());
    BYTES.set(Some(0));
    let clock=Instant::now();
    let result=solve(from,goal,Kit::BareLegs,source,store,codec,u64::MAX);
    let ns=clock.elapsed().as_nanos();
    let bytes=BYTES.replace(None).unwrap();
    #[cfg(feature="solve-counters")]
    println!("WORK from={origin} goal={:?} {:?}",goal.cell,plan::solve_meter::finish());
    times.push(ns);
    let ticks=store.preparation_ticks()-before;
    let learned: u128=result.stages.iter().filter_map(|s| match s {
        plan::Stage::Learn{nanos,..}=>Some(*nanos), _=>None,
    }).sum();
    println!("CMD from={origin} goal={:?} ns={ns} allocated_bytes={bytes} law_ticks={ticks} kernel_ns={learned}",goal.cell);
    if before!=0 {assert_eq!(ticks,0,"new preparation for {:?}",goal.cell);}
    match result.outcome {
        PlanOutcome::Ready(p)=>{
            println!("END from={origin} goal={:?} T={} bits={:x?} profile={:?}",goal.cell,p.ticks,p.end.bits(),p.thinking);
            let programs=words::tiny_programs();
            let mut state=from.clone();
            let mut hit=goal.touched(from,from).then_some(0);
            for w in &p.words {
                let program=&programs[w.program as usize];
                let keys=program.held_input(w.args);
                for t in 0..w.ticks {
                    let before=state.bounding_box();
                    let input=keys.unwrap_or_else(||program.select(source,&state,t,w.args).unwrap().1);
                    law::try_tick(&mut state,source,&input).unwrap();
                    if goal.touched_boxes(before,state.bounding_box()) {hit.get_or_insert(state.t-from.t);}
                }
                assert_eq!(state.position,w.end.position,"word end at {} for {:?}",state.t,goal.cell);
                assert_eq!(state.velocity,w.end.velocity,"word velocity at {} for {:?}",state.t,goal.cell);
            }
            assert_eq!(hit,Some(p.ticks),"goal {:?}: replay {hit:?}, prediction {}",goal.cell,p.ticks);
            Some(p)
        }
        outcome=>{println!("REFUSED from={origin} goal={:?} outcome={outcome:?} reason={:?}",goal.cell,result.explanation); None}
    }
}

#[test]
fn real_time_per_command() {
    let source=GeneratorSuperflatSource::new(-61,Block::full(),[-3_000_000,-64,-3_000_000],[3_000_000,20,3_000_000]);
    let start=plan::settle(&plan::Still(&source),[0.5,-60.0,0.5],0).unwrap();
    let store=GeneratorStore::default();let mut codec=Interner::default();
    let mut seed=0x5eed_u64;let n=if cfg!(debug_assertions) {4} else {60};
    // Random goals: bearing uniform over the circle, distance log-uniform from 10^2 to 10^6 blocks.
    let goal=|s:&mut u64|{
        let angle=(lcg(s) as f64/(1u64<<31) as f64)*std::f64::consts::TAU;
        let d=10f64.powf(2.0+4.0*(lcg(s) as f64/(1u64<<31) as f64));
        Goal {cell:[(d*angle.cos()).round() as i32,-61,(d*angle.sin()).round() as i32]}
    };
    let mut warmup=Vec::new();
    command(&start,"first",Goal {cell:[0,-61,100]},&source,&store,&mut codec,&mut warmup).unwrap();
    // 1. New goals from standing, one after another, on the same living store.
    let mut times=Vec::new();let mut refused=0;let mut moving_starts=Vec::new();
    for _ in 0..n {
        let g=goal(&mut seed);
        if let Some(p)=command(&start,"standing",g,&source,&store,&mut codec,&mut times) {
            moving_starts.push(p.words.first().expect("distant goal").end.materialize());
        } else {refused+=1;}
    }
    stats("new goal, from standing",&mut times,refused,10_000.0);
    assert_eq!(moving_starts.len(),n,"each moving retarget needs its actual predecessor");
    let mut times=Vec::new();
    let mut refused=0;
    for s in &moving_starts {
        let g=goal(&mut seed);
        if command(s,"moving",g,&source,&store,&mut codec,&mut times).is_none() {refused+=1;}
    }
    stats("new goal, while moving",&mut times,refused,10_000.0);
    assert_eq!(refused,0);
    // The original moving regression starts 265 ticks into the preceding jump word.
    let mut retained=start.clone();
    let input=law::Input {forward:true,jump:true,sprint:true,yaw:f32::from_bits(1135073362),..Default::default()};
    for _ in 0..265 {law::try_tick(&mut retained,&source,&input).unwrap();}
    let p=command(&retained,"retained",Goal {cell:[-388,-61,-317]},&source,&store,&mut codec,&mut Vec::new()).unwrap();
    assert!(p.ticks<=1666,"retained moving command T={}",p.ticks);
    let mut times=Vec::new();
    for _ in 0..n {command(&start,"repeat",Goal {cell:[0,-61,100]},&source,&store,&mut codec,&mut times).unwrap();}
    stats("same goal again (repeat)",&mut times,0,1_000.0);
}
