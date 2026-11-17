use law::{Block,PlayerState,Vec3,World,codec::Interner};
use plan::{solve,Goal,Kit,Plan,PlanOutcome,Stage,source::GeneratorSuperflatSource,store::{GeneratorStore,SPRINT_JUMP,WALK,sprint_allowed}};
use std::time::Instant;
pub(super) fn setup()->(GeneratorSuperflatSource,PlayerState) {
    let source=GeneratorSuperflatSource::new(-61,Block::full(),[-3_000_000,-64,-3_000_000],[3_000_000,20,3_000_000]);
    let start=plan::settle(&plan::Still(&source),[0.5,-60.0,0.5],0).unwrap();(source,start)
}
pub(super) fn ready(result:plan::solve::SolveResult)->Plan {
    match result.outcome {PlanOutcome::Ready(p)=>p,other=>panic!("{other:?} reason={:?} stages={:?}",result.explanation,result.stages)}
}
pub(super) fn replay(source:&impl World,start:&PlayerState,p:&Plan,goal:Goal) {
    let programs=words::tiny_programs();let mut state=start.clone();let mut hit=None;
    for step in &p.words {
        assert!(step.args.count>=1);
        for t in 0..step.ticks {
            let before=state.clone();let input=programs[step.program as usize].select(source,&state,t,step.args).unwrap().1;
            law::try_tick(&mut state,source,&input).unwrap();
            if goal.touched(&before,&state) {assert!(hit.is_none(),"plan continues past first touch");hit=Some(state.t-start.t);}
        }
        assert_eq!([state.position.x.to_bits(),state.position.y.to_bits(),state.position.z.to_bits()],
            [step.end.position.x.to_bits(),step.end.position.y.to_bits(),step.end.position.z.to_bits()],"step program={} ticks={} count={}",step.program,step.ticks,step.args.count);
        assert_eq!([state.velocity.x.to_bits(),state.velocity.y.to_bits(),state.velocity.z.to_bits()],
            [step.end.velocity.x.to_bits(),step.end.velocity.y.to_bits(),step.end.velocity.z.to_bits()]);
        assert_eq!(state.main_supporting_block,step.end.main_supporting_block);
    }
    assert_eq!(hit,Some(p.ticks));assert!(goal.touched(&state,&state) || hit.is_some());
}
#[test]
fn u1_native_ten_thousand() {
    let (source,start)=setup();let codec=&mut Interner::default();let store=&GeneratorStore::default();
    for goal in [Goal {cell:[1000,-61,0]},Goal {cell:[10_000,-61,3300]}] {
        let clock=Instant::now();let result=solve(&start,goal,Kit::BareLegs,&source,store,codec,u64::MAX);
        let phases:Vec<_>=result.stages.iter().filter(|s|matches!(s,Stage::Learn {..})).collect();
        println!("PROBE goal={goal:?} time_ns={} learned={} ticks={}",clock.elapsed().as_nanos(),phases.len(),store.preparation_ticks());
        let p=ready(result);println!("PLAN T={} words={:?} end={:?}",p.ticks,p.words.iter().map(|w|(w.program,w.args.count,w.ticks)).collect::<Vec<_>>(),p.end.position);
        if goal.cell[0]<=10_000 && goal.cell[2]<=10_000 {replay(&source,&start,&p,goal);}
        let ticks=store.preparation_ticks();let clock=Instant::now();let warm=ready(solve(&start,goal,Kit::BareLegs,&source,store,codec,u64::MAX));
        println!("WARM ns={} preparation_delta={} profile={:?}",clock.elapsed().as_nanos(),store.preparation_ticks()-ticks,warm.thinking);
        assert_eq!(warm.ticks,p.ticks);assert_eq!(store.preparation_ticks(),ticks);
    }
}

#[test]
fn u1_kill_test_17_million_line_peaceful() {
    let (source,start)=setup();
    let counted=plan::source::MeteredWorld::new(&source);
    assert_eq!(source.difficulty(),0);
    let n=if cfg!(debug_assertions) {3} else {100};
    for cell in [[1_000_000,-61,0],[0,-61,1_000_000],[1_000_000,-61,330_000]] {
        let goal=Goal {cell};let mut cold=Vec::new();let mut warm=Vec::new();let mut profiles:Vec<[u128;4]>=Vec::new();
        let store=GeneratorStore::default();let mut codec=Interner::default();let mut cold_ticks=None;
        let cold_plan=ready(solve(&start,goal,Kit::BareLegs,&source,&store,&mut codec,u64::MAX));
        assert!(cold_plan.words.iter().any(|w|w.args.count>100_000));
        assert!(cold_plan.bracket.lb<=cold_plan.ticks);
        let before_last=cold_plan.words.get(cold_plan.words.len().saturating_sub(2)).map(|s|s.end.clone()).unwrap_or(start.clone());
        let last=cold_plan.words.last().unwrap();
        let mut tail=cold_plan.clone();tail.words=vec![last.clone()];tail.ticks=last.ticks as u64;
        replay(&source,&before_last,&tail,goal);
        for _ in 0..n {
            let fresh=GeneratorStore::default();let mut fresh_codec=Interner::default();source.reset_costs();
            counted.take_ticks();let clock=Instant::now();let p=ready(solve(&start,goal,Kit::BareLegs,&counted,&fresh,&mut fresh_codec,u64::MAX));
            cold.push(clock.elapsed().as_nanos());
            assert_eq!(counted.take_ticks(),fresh.preparation_ticks(),"law ticks bypassed store metering");
            if let Some(t)=cold_ticks {assert_eq!(fresh.preparation_ticks(),t);} else {cold_ticks=Some(fresh.preparation_ticks());}
            assert_eq!(p.ticks,cold_plan.ticks);assert_eq!(source.costs().corridors,1);
            let ticks=store.preparation_ticks();source.reset_costs();let clock=Instant::now();
            let result=solve(&start,goal,Kit::BareLegs,&counted,&store,&mut codec,u64::MAX);
            assert!(!result.stages.iter().any(|s|matches!(s,Stage::Learn {..})));
            let p=ready(result);warm.push(clock.elapsed().as_nanos());
            assert_eq!(counted.take_ticks(),0,"warm solve executed native ticks");
            profiles.push([p.thinking.entry_ns,p.thinking.bearing_ns,p.thinking.inverse_ns,p.thinking.output_ns]);
            assert_eq!(store.preparation_ticks(),ticks);assert_eq!(p.ticks,cold_plan.ticks);
            assert_eq!(source.costs().corridors,1);
        }
        cold.sort_unstable();warm.sort_unstable();
        let print=|mode:&str,times:&Vec<u128>|println!("U1_SOLVE cell={},{},{} mode={mode} n={} min_ns={} median_ns={} p99_ns={} max_ns={} samples={times:?}",
            cell[0],cell[1],cell[2],times.len(),times[0],(times[(times.len()-1)/2]+times[times.len()/2])/2,times[(99*times.len()).div_ceil(100)-1],times[times.len()-1]);
        print("cold",&cold);print("warm",&warm);
        for (i,name) in ["entry","bearing","inverse","output"].iter().enumerate() {
            let mut values:Vec<_>=profiles.iter().map(|p|p[i]).collect();values.sort_unstable();
            println!("U1_PROFILE cell={},{},{} stage={name} min_ns={} median_ns={} p99_ns={} max_ns={}",cell[0],cell[1],cell[2],values[0],values[values.len()/2],values[(99*values.len()).div_ceil(100)-1],values[values.len()-1]);
        }
        println!("U1_RESULT cell={},{},{} ticks={} cold_law_ticks={} warm_law_ticks=0 reads={:?} end_bits={:?}",
            cell[0],cell[1],cell[2],cold_plan.ticks,cold_ticks.unwrap(),source.costs(),[cold_plan.end.position.x.to_bits(),cold_plan.end.position.y.to_bits(),cold_plan.end.position.z.to_bits()]);
        // Keep the bearing's native pair covered: an axis tie crossed by a
        // lateral shift is a NEW bearing and correctly invokes learning.
        let shift=Vec3::new(if cell[0]==0 {0.0} else {0.25},0.0,if cell[2]==0 {0.0} else {0.25});
        let mut shifted=start.clone();shifted.position=shifted.position.plus(shift);shifted.t=77;shifted.food_level=17;shifted.exhaustion=0.25;
        let ticks=store.preparation_ticks();let shifted_plan=ready(solve(&shifted,goal,Kit::BareLegs,&source,&store,&mut codec,u64::MAX));
        assert_eq!(shifted_plan.label,cold_plan.label);assert_eq!(store.preparation_ticks(),ticks);
        println!("U1_SHIFT cell={},{},{} label_same=true preparation_delta=0",cell[0],cell[1],cell[2]);
    }
}

#[test]
fn u1_work_tracks_events_and_scars() {
    let (mut source,start)=setup();let store=GeneratorStore::default();let mut codec=Interner::default();
    let mut cold_cost=None;let mut warm_cost=None;let mut prep=None;
    for distance in [1000,10_000,1_000_000] {
        source.reset_costs();let fresh=GeneratorStore::default();let mut fresh_codec=Interner::default();
        ready(solve(&start,Goal {cell:[distance,-61,0]},Kit::BareLegs,&source,&fresh,&mut fresh_codec,u64::MAX));
        if let Some(c)=cold_cost {assert_eq!(source.costs(),c);} else {cold_cost=Some(source.costs());}
        if let Some(t)=prep {assert_eq!(fresh.preparation_ticks(),t);} else {prep=Some(fresh.preparation_ticks());}
        ready(solve(&start,Goal {cell:[distance,-61,0]},Kit::BareLegs,&source,&store,&mut codec,u64::MAX));
        source.reset_costs();let ticks=store.preparation_ticks();
        let p=ready(solve(&start,Goal {cell:[distance,-61,0]},Kit::BareLegs,&source,&store,&mut codec,u64::MAX));
        if let Some(c)=warm_cost {assert_eq!(source.costs(),c);} else {warm_cost=Some(source.costs());}
        assert_eq!(store.preparation_ticks(),ticks);
        println!("U1_WORK distance={distance} cold={:?} warm={:?} cold_ticks={} warm_ticks=0 lattice={:?}",cold_cost.unwrap(),source.costs(),prep.unwrap(),p.lattice_cost);
    }
    for k in [1,8,32] {
        source.scars.clear();for i in 0..k {source.scars.insert([500+10*i,-60,0],Block::full());}
        source.reset_costs();let ticks=store.preparation_ticks();
        let result=solve(&start,Goal {cell:[1000,-61,0]},Kit::BareLegs,&source,&store,&mut codec,u64::MAX);
        assert!(matches!(result.outcome,PlanOutcome::NeedsJoints {exception:[500,-60,0],exceptions,..} if exceptions==k as usize));
        assert_eq!(source.costs().scars,k as u64);assert_eq!(store.preparation_ticks(),ticks);
        println!("U1_SCARS k={k} costs={:?} warm_ticks=0",source.costs());
    }
}

#[test]
fn u1_kill_test_8_one_mechanism_and_scar_cut() {
    let (mut source,start)=setup();let store=GeneratorStore::default();let mut codec=Interner::default();let goal=Goal {cell:[1000,-61,0]};
    ready(solve(&start,goal,Kit::BareLegs,&source,&store,&mut codec,u64::MAX));
    source.reset_costs();store.reset_reads();let metered=plan::source::MeteredWorld::new(&source);
    let clean=solve(&start,goal,Kit::BareLegs,&metered,&store,&mut codec,u64::MAX);
    assert_eq!(metered.ticks(),0,"warm solve executed native law ticks");
    let clean_reads=store.reads();
    assert!(clean_reads.kernels>0,"solve bypassed the generator store");
    assert_eq!(clean_reads.bundle_hits,1,"warm solve did not consume its bundle");
    assert_eq!(&clean.stages[..3],&[Stage::Bracket,Stage::ClosedCheck,Stage::Extend]);
    for s in [Stage::Cut,Stage::Compose,Stage::Allocate] {assert!(clean.stages.contains(&s));}
    let p=match &clean.outcome {PlanOutcome::Ready(p)=>p,_=>panic!("not ready")};
    assert_eq!(source.costs().corridors,1,"solve bypassed the corridor query");
    assert_eq!(p.words.iter().map(|w|w.ticks as u64).sum::<u64>(),p.ticks);
    assert!(p.words.iter().any(|w|w.args.count>1));
    replay(&source,&start,p,goal);
    source.scars.insert([500,-60,0],Block::full());source.reset_costs();store.reset_reads();
    let metered=plan::source::MeteredWorld::new(&source);
    let scar=solve(&start,goal,Kit::BareLegs,&metered,&store,&mut codec,u64::MAX);
    assert_eq!(metered.ticks(),0,"scar solve executed native law ticks");
    assert_eq!(store.reads(),clean_reads,"clean and scarred solves read different generators");
    assert_eq!(&scar.stages[..3],&[Stage::Bracket,Stage::ClosedCheck,Stage::Extend]);
    assert_eq!(scar.stages.last(),Some(&Stage::Cut));assert_eq!(source.costs().corridors,1);
    assert!(matches!(scar.outcome,PlanOutcome::NeedsJoints {exception:[500,-60,0],..}));
    println!("U1_ONE_MECHANISM clean={:?} scar={:?}",clean.stages,scar.stages);
}

#[test]
fn u1_peaceful_food_guard_and_no_fed() {
    let (source,start)=setup();let store=GeneratorStore::default();let mut codec=Interner::default();
    for food in [6,7] {
        let mut from=start.clone();from.food_level=food;from.saturation=0.0;from.exhaustion=3.99;
        let bits=from.bits();let from=PlayerState::from_observed_bits(&bits,&vec![true;bits.len()]).unwrap();
        let expected=if sprint_allowed(&from) {SPRINT_JUMP} else {WALK};
        let result=solve(&from,Goal {cell:[1000,-61,0]},Kit::BareLegs,&source,&store,&mut codec,u64::MAX);
        assert_eq!(&result.stages[..3],&[Stage::Bracket,Stage::ClosedCheck,Stage::Extend]);
        let p=ready(result);assert_eq!(p.words[0].program,expected);assert_eq!(p.end.food_level,food);
        assert_eq!(p.end.saturation.to_bits(),from.saturation.to_bits());
        replay(&source,&from,&p,Goal {cell:[1000,-61,0]});
        println!("U1_FOOD input={food} program={} ticks={}",p.words[0].program,p.ticks);
    }
}

#[test]
fn u1_class_id_table_not_exhausted() {
    let store=GeneratorStore::default();let mut codec=Interner::default();let (source,start)=setup();
    plan::source::label(&mut codec,&source,&start).unwrap();let classes=codec.digests().len();
    for i in 0..100_000u32 {assert_eq!(store.intern_motor(&[i as u64]).0,i);}
    assert_eq!(store.intern_motor(&[99_999]).0,99_999);assert_eq!(store.motor_count(),100_000);
    assert_eq!(codec.digests().len(),classes);
    println!("U1_MOTOR keys={} codec_classes={classes}",store.motor_count());
    assert!(store.bind_codec(&mut codec));let mut other=Interner::default();assert!(!store.bind_codec(&mut other));
}

#[test]
fn u1_new_bearing_learns_subnormal_switch() {
    let (source,start)=setup();let store=GeneratorStore::default();let mut codec=Interner::default();let goal=Goal {cell:[1000,-61,0]};
    ready(solve(&start,goal,Kit::BareLegs,&source,&store,&mut codec,u64::MAX));
    let mut shifted=start.clone();shifted.position=shifted.position.plus(Vec3::new(0.25,0.0,0.25));
    let result=solve(&shifted,goal,Kit::BareLegs,&source,&store,&mut codec,u64::MAX);
    assert!(result.stages.iter().any(|s|matches!(s,Stage::Learn {ticks,..} if *ticks>0)));
    let p=ready(result);replay(&source,&shifted,&p,goal);
    let ticks=store.preparation_ticks();ready(solve(&shifted,goal,Kit::BareLegs,&source,&store,&mut codec,u64::MAX));
    assert_eq!(store.preparation_ticks(),ticks);
    println!("U1_NEW_BEARING learned=true warm_ticks=0 native_T={}",p.ticks);
}

#[test]
fn u1_already_touched_closes_without_learning() {
    let (source,start)=setup();let store=GeneratorStore::default();let mut codec=Interner::default();
    let result=solve(&start,Goal {cell:[0,-61,0]},Kit::BareLegs,&source,&store,&mut codec,u64::MAX);
    assert_eq!(result.stages,vec![Stage::Bracket,Stage::ClosedCheck]);let p=ready(result);
    assert_eq!(p.ticks,0);assert!(p.words.is_empty());assert_eq!(store.preparation_ticks(),0);
}

#[test]
fn u1_vertical_frame_and_reversal_envelope() {
    let (mut source,start)=setup();let store=GeneratorStore::default();let mut codec=Interner::default();
    let goal=Goal {cell:[1000,-61,0]};ready(solve(&start,goal,Kit::BareLegs,&source,&store,&mut codec,u64::MAX));
    let high=GeneratorSuperflatSource::new(63,Block::full(),[-3_000_000,-64,-3_000_000],[3_000_000,128,3_000_000]);
    let at=plan::settle(&plan::Still(&high),[0.5,64.0,0.5],0).unwrap();let ticks=store.preparation_ticks();
    let p=ready(solve(&at,Goal {cell:[1000,63,0]},Kit::BareLegs,&high,&store,&mut codec,u64::MAX));
    assert_eq!(store.preparation_ticks(),ticks);replay(&high,&at,&p,Goal {cell:[1000,63,0]});
    let mut reverse=start.clone();reverse.velocity.x=-2.0;
    let p=ready(solve(&reverse,goal,Kit::BareLegs,&source,&store,&mut codec,u64::MAX));replay(&source,&reverse,&p,goal);
    assert!(p.words[0].certificate.min[0]<0);
    source.scars.insert([-1,-60,0],Block::full());
    let result=solve(&reverse,goal,Kit::BareLegs,&source,&store,&mut codec,u64::MAX);
    assert!(matches!(result.outcome,PlanOutcome::NeedsJoints {exception:[-1,-60,0],..}));
    println!("U1_FRAMES vertical_native_equal=true reverse_tube_detects_joint=true");
}

#[test]
fn u1_refusals_are_explicit() {
    let (mut source,start)=setup();let store=GeneratorStore::default();let mut codec=Interner::default();let goal=Goal {cell:[1000,-61,0]};
    source.difficulty=1;
    let result=solve(&start,goal,Kit::BareLegs,&source,&store,&mut codec,u64::MAX);
    assert!(matches!(result.outcome,PlanOutcome::Unknown(_)));assert_eq!(store.preparation_ticks(),0);
    println!("U1_REFUSAL source=Normal reason={:?}",result.explanation);
    source.difficulty=0;source.loaded_max[0]=20;
    let result=solve(&start,goal,Kit::BareLegs,&source,&store,&mut codec,u64::MAX);
    assert!(matches!(result.outcome,PlanOutcome::Unknown(_)));
    assert!(result.explanation.as_ref().is_some_and(|s|s.contains("loaded boundary")));
    println!("U1_REFUSAL source=LoadedBoundary reason={:?}",result.explanation);
    let mut other=Interner::default();let result=solve(&start,goal,Kit::BareLegs,&source,&store,&mut other,u64::MAX);
    assert!(matches!(result.outcome,PlanOutcome::Unknown(law::codec::proof::Refusal::Schema)));
    println!("U1_REFUSAL source=OtherCodec reason=Schema");
}

#[test]
fn u1_generator_occupancy_equals_full_scan() {
    struct Full<'a>(&'a GeneratorSuperflatSource);
    impl World for Full<'_> {
        fn block(&self,p:law::BlockPos)->&Block {self.0.block(p)}
        fn loaded(&self,p:law::BlockPos)->bool {self.0.loaded(p)}
        fn difficulty(&self)->u8 {self.0.difficulty()}
        fn min_y(&self)->i32 {self.0.min_y()}
        fn has_fluid(&self)->bool {self.0.has_fluid()}
        fn has_inside_effects(&self)->bool {self.0.has_inside_effects()}
    }
    fn planes(s:law::shapes::ShapeView<'_>)->[Vec<u64>;3] {
        std::array::from_fn(|a|(0..s.shape.coords[a].len()).map(|i|s.coord(a,i).to_bits()).collect())
    }
    let (mut source,start)=setup();let mut queries=0;
    for scene in 0..4 {
        source.floor_block=if scene==1 {Block::default()} else {Block::full()};
        source.air_block=if scene==2 {Block::full()} else {Block::default()};
        source.scars.insert([0,-60,0],Block {suffocating:true,..Block::boxes(&[law::Aabb::new(-0.5,0.0,0.0,1.0,1.5,1.0)])});
        source.scars.insert([1,-61,0],Block::default());
        for x in -2..=2 {for y in -62..=-58 {for z in -2..=2 {
            let bounds=law::Aabb::player(Vec3::new(x as f64+0.75,y as f64+0.25,z as f64+0.75),1.8);
            let context=law::CollisionContext {feet_y:bounds.min.y,descending:true,fall_distance:0.0,leather_boots:false};
            let expected:Vec<_>=Full(&source).collisions(bounds,context).iter().map(|s|planes(s.view())).collect();
            let mut actual=Vec::new();source.visit_collisions(bounds,context,|s| {actual.push(planes(s));true});
            assert_eq!(actual,expected,"scene={scene} cell={x},{y},{z}");
            assert_eq!(source.collides_with_suffocating_block(bounds,context),Full(&source).collides_with_suffocating_block(bounds,context));
            queries+=1;
        }}}
        if scene==3 {
            let mut fast=start.clone();let mut slow=fast.clone();let program=words::tiny_programs().remove(SPRINT_JUMP as usize);
            for n in 0..200 {
                let input=program.select(&source,&fast,n,words::Arguments::default()).unwrap().1;
                law::try_tick(&mut fast,&source,&input).unwrap();law::try_tick(&mut slow,&Full(&source),&input).unwrap();
                assert_eq!(fast.bits(),slow.bits(),"native tick={n}");
            }
        }
    }
    println!("U1_OCCUPANCY full_scan_equal=true queries={queries} native_ticks=200");
}
