//! One loop. No source type, terrain name, uniform-ground flag, or legacy planner.
use std::sync::Arc;
use law::{PlayerState,Vec3,BlockPos,World,codec::{Interner,LabelId,proof::Refusal}};
use words::Arguments;
use crate::{Goal,events::{self,Event,EventKind},lattice,source::{self,CorridorSource,CorridorResult,CorridorProof},store::{GeneratorStore,WordKernel,WordBundle,SPRINT_JUMP,WALK,sprint_allowed}};

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Kit {BareLegs}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Bracket {pub lb:u64,pub ub:Option<u64>}
#[derive(Clone,Debug,PartialEq,Eq)]
pub enum Stage {Bracket,ClosedCheck,Extend,Cut,Compose,Allocate,Learn {program:u16,yaw:u16,ticks:u64,nanos:u128}}
#[derive(Clone,Debug)]
pub struct WordStep {pub program:u16,pub args:Arguments,pub ticks:u32,pub expected_end_t:u64,pub events:Vec<Event>,pub certificate:CorridorProof,pub end:PlayerState,pub map_segments:usize}
#[derive(Clone,Debug)]
pub struct Plan {pub ticks:u64,pub words:Vec<WordStep>,pub end:PlayerState,pub bracket:Bracket,pub label:LabelId,pub deadline:u64,pub lattice_cost:lattice::LatticeCost,pub thinking:ThinkingCosts}
#[derive(Clone,Copy,Default,Debug)]
pub struct ThinkingCosts {pub entry_ns:u128,pub bearing_ns:u128,pub inverse_ns:u128,pub output_ns:u128}
#[derive(Clone,Debug)]
pub enum PlanOutcome {Ready(Plan),NeedsJoints {exception:BlockPos,exceptions:usize,bracket:Bracket},Unknown(Refusal)}
#[derive(Clone,Debug)]
pub struct SolveResult {pub outcome:PlanOutcome,pub stages:Vec<Stage>,pub explanation:Option<String>}

fn prepare(store:&GeneratorStore,label:LabelId,source:&impl World,from:&PlayerState,program:u16,yaw:u16,stages:&mut Vec<Stage>)->Result<Arc<WordKernel>,String> {
    if let Some(k)=store.get(label,from,program,yaw) {return Ok(k);}
    let tick=store.preparation_ticks();let clock=std::time::Instant::now();
    let answer=store.prepare(label,source,from,program,yaw);
    stages.push(Stage::Learn {program,yaw,ticks:store.preparation_ticks()-tick,nanos:clock.elapsed().as_nanos()});answer
}
fn kernel_touch(k:&WordKernel,from:&PlayerState,goal:Goal,horizon:u64)->Option<u64> {
    let horizon=horizon.min(events::horizon(from,goal,k.transient_increments.iter().chain(&k.cycle_increments).copied(),k.cycle_displacement));
    let head=horizon.min(k.transient_ticks as u64);
    if head>0 && word_lower_bound(k,from,goal)<=head {
        if let Some(t)=events::first_touch_maps(from,&k.transient_maps[0],&k.transient_maps[1],&k.prefix_phases(from),goal,head,false) {return Some(t);}
    }
    if horizon<=head {return None;}
    let cycle=k.cycle_start(from);
    events::first_touch_maps(&cycle,&k.cycle_maps[0],&k.cycle_maps[1],&k.periodic_phases(&cycle),goal,horizon-head,false).map(|t|head+t)
}
#[derive(Clone)]
struct Leg {kernel:Arc<WordKernel>,from:PlayerState,ticks:u64,cycle_only:bool}
fn cycle_at(k:&WordKernel,from:&PlayerState,ticks:u64)->PlayerState {
    k.cycle_at(from,ticks)
}
fn leg_end(leg:&Leg)->PlayerState {if leg.cycle_only {cycle_at(&leg.kernel,&leg.from,leg.ticks)} else {leg.kernel.at(&leg.from,leg.ticks)}}
fn word_lower_bound(k:&WordKernel,from:&PlayerState,goal:Goal)->u64 {
    let step=k.max_body_step;
    if step==0.0 {return u64::MAX;}
    let zone=goal.zone();let p=from.position;
    let extent=[p.x.abs(),p.y.abs(),p.z.abs(),zone.min.x.abs(),zone.min.y.abs(),zone.min.z.abs(),zone.max.x.abs(),zone.max.y.abs(),zone.max.z.abs()].into_iter().fold(0.0f64,f64::max)+step+from.pose.height() as f64;
    let ulp=2.0f64.powi(events::Binary64::of(extent.next_up()).exp);
    let upper=(step+2.0*ulp*3.0f64.sqrt()).next_up();
    (goal.gap(from)/upper).next_down().ceil().max(0.0) as u64
}
fn append_steps(out:&mut Vec<WordStep>,leg:&Leg,proof:&CorridorProof) {
    let k=&leg.kernel;let head=if leg.cycle_only {0} else {leg.ticks.min(k.transient_ticks as u64)};
    let rest=leg.ticks-head;let cycles=rest/k.cycle_period as u64;let tail=rest%k.cycle_period as u64;
    let mut offset=0;
    for (ticks,count) in [(head,1),(cycles*k.cycle_period as u64,cycles),(tail,1)] {
        if ticks==0 {continue;}
        offset+=ticks;
        let end=if leg.cycle_only {cycle_at(k,&leg.from,offset)} else {k.at(&leg.from,offset)};
        let mut events=Vec::new();
        let mut map_segments=0;
        let before=if leg.cycle_only {cycle_at(k,&leg.from,offset-ticks)} else {k.at(&leg.from,offset-ticks)};
        let maps=if offset<=head {&k.transient_maps} else {&k.cycle_maps};
        for (coordinate,map,expected) in [(before.position.x,&maps[0],end.position.x),(before.position.z,&maps[1],end.position.z)] {
            let fold=map.fold(coordinate,ticks);
            assert_eq!(fold.value.to_bits(),expected.to_bits(),"event map and endpoint map disagree");map_segments+=fold.segments;
            for piece in fold.pieces {if let events::Piece::Boundary {tick,..}=piece {events.push(Event {tick:before.t+tick,kind:EventKind::BinadeCrossing});}}
        }
        events.sort_by_key(|e|e.tick);events.dedup();
        out.push(WordStep {program:k.program,args:Arguments {count:count as u32,..k.arguments},ticks:ticks.try_into().expect("word tick width"),expected_end_t:end.t,events,certificate:proof.clone(),end,map_segments});
    }
}

pub fn solve<S:CorridorSource>(from:&PlayerState,to:Goal,kit:Kit,source:&S,store:&GeneratorStore,codec:&mut Interner,deadline_ticks:u64)->SolveResult {
    let Kit::BareLegs=kit; // A new kit must supply its own bound and word data.
    let mut stages=vec![Stage::Bracket,Stage::ClosedCheck];
    let bracket=Bracket {lb:to.lower_bound(from) as u64,ub:None};
    if !store.bind_codec(codec) {return SolveResult {outcome:PlanOutcome::Unknown(Refusal::Schema),stages,explanation:Some("generator store belongs to another codec".into())};}
    if to.touched(from,from) {
        let label=match source::label(codec,source,from) {Ok(label)=>label,Err(r)=>return SolveResult {outcome:PlanOutcome::Unknown(r),stages,explanation:None}};
        return SolveResult {outcome:PlanOutcome::Ready(Plan {ticks:0,words:Vec::new(),end:from.clone(),bracket:Bracket {lb:0,ub:Some(0)},label,deadline:deadline_ticks,lattice_cost:Default::default(),thinking:Default::default()}),stages,explanation:None};
    }
    if source.difficulty()!=0 {return SolveResult {outcome:PlanOutcome::Unknown(Refusal::StateOrContext),stages,explanation:Some("U1 requires invariant resource guards; server food crossings are not predicted".into())};}
    stages.push(Stage::Extend);
    let mut thinking=ThinkingCosts::default();let entry_clock=std::time::Instant::now();
    let mut work=||->Result<(Vec<Leg>,LabelId,lattice::LatticeCost),String> {
        let label=source::label(codec,source,from).map_err(|e|format!("entry label: {e:?}"))?;
        let delta=to.center().minus(from.position);let pair=lattice::bracket(delta.x,delta.z);
        let program=if sprint_allowed(from) {SPRINT_JUMP} else {WALK};
        let bundle=if let Some(bundle)=store.bundle(label,from,program,pair) {bundle} else {
            let first=prepare(store,label,source,from,program,pair[0],&mut stages)?;
            let c1=first.cycle_start(from);
            let second=prepare(store,label,source,&c1,program,pair[1],&mut stages)?;
            let mut incumbents=Vec::new();let mut terminals=Vec::new();
            for word in store.words() {if sprint_allowed(from) || word==WALK {incumbents.push(prepare(store,label,source,from,word,pair[0],&mut stages)?);}}
            for phase in 0..second.cycle_period {
                let state=second.phase(phase as usize);let mut rows=Vec::new();
                for word in store.words() {if sprint_allowed(&state) || word==WALK {rows.push(prepare(store,label,source,&state,word,pair[1],&mut stages)?);}}
                terminals.push(rows);
            }
            store.file_bundle(label,from,program,pair,WordBundle {first,second,incumbents,terminals})
        };
        let first=&bundle.first;
        let c1=first.cycle_start(from);
        let second=&bundle.second;
        let c2=second.cycle_start(&c1);
        thinking.entry_ns=entry_clock.elapsed().as_nanos();let bearing_clock=std::time::Instant::now();
        let e=c2.position.minus(from.position);
        let target=delta.minus(e);
        let d1=first.bearing_displacement(to.center());let d2=second.bearing_displacement(to.center());let d=d1.plus(d2);
        let radius=from.bounding_box().max.minus(from.position);
        // A fundamental-cell rectangle for the bearing. It retains the whole
        // Euclidean remainder; it never asserts that these counts touch B.
        let lo=Vec3::new(target.x-d.x.max(0.0)-radius.x,0.0,target.z-d.z.max(0.0)-radius.z);
        let hi=Vec3::new(target.x-d.x.min(0.0)+radius.x,0.0,target.z-d.z.min(0.0)+radius.z);
        let cap=u32::MAX/first.cycle_period.max(second.cycle_period);
        let bearing=lattice::counts(d1,d2,lo,hi,cap);
        thinking.bearing_ns=bearing_clock.elapsed().as_nanos();let inverse_clock=std::time::Instant::now();
        let mut candidates:Vec<(u64,Vec<Leg>)>=Vec::new();
        // The same store words also supply an incumbent from the observed
        // state, including displacements shorter than their entry transient.
        for k in &bundle.incumbents {
            if let Some(t)=kernel_touch(k,from,to,u32::MAX as u64) {candidates.push((t,vec![Leg {kernel:k.clone(),from:from.clone(),ticks:t,cycle_only:false}]));}
        }
        let mut lattice_cost=lattice::LatticeCost::default();
        let mut bearings=Vec::new();
        if let Some((q,cost))=bearing {lattice_cost=cost;bearings.push(q);}
        if let Some(q)=lattice::bearing_cell(d1,d2,target) {if !bearings.contains(&q) {bearings.push(q);}}
        for q in bearings {
            let t1=first.transient_ticks as u64+q[0] as u64*first.cycle_period as u64;
            let end1=first.at(from,t1);
            let t2=second.transient_ticks as u64+q[1] as u64*second.cycle_period as u64;
            let base=second.at(&end1,t2);
            // Extend to the goal event, relaxing only vertical eligibility to
            // locate its approach. The terminal preimage restores all phases.
            let horizon=events::horizon(&base,to,second.cycle_increments.iter().copied(),second.cycle_displacement);
            if let Some(touch)=events::first_touch_maps(&base,&second.cycle_maps[0],&second.cycle_maps[1],&second.periodic_phases(&base),to,horizon,true) {
                let p=second.cycle_period as u64;
                let block=(touch-1)/p*p;
                let mut jobs=Vec::new();
                for phase in 0..p {
                    let mut cut=block+phase;
                    if cut>=touch {if cut<p {continue;}cut-=p;}
                    let carried=cycle_at(&second,&base,cut);
                    for terminal in &bundle.terminals[phase as usize] {
                        jobs.push((t1+t2+cut+word_lower_bound(terminal,&carried,to),cut,carried.clone(),terminal));
                    }
                }
                jobs.sort_by_key(|job|job.0);
                for (lower,cut,carried,terminal) in jobs {
                    let best=candidates.iter().map(|c|c.0).min().unwrap_or(u64::MAX);
                    if lower>=best {continue;}
                    let horizon=best.saturating_sub(t1+t2+cut+1).min(u32::MAX as u64);
                        if let Some(t)=kernel_touch(terminal,&carried,to,horizon) {
                            let legs=vec![Leg {kernel:first.clone(),from:from.clone(),ticks:t1,cycle_only:false},
                                Leg {kernel:second.clone(),from:end1.clone(),ticks:t2,cycle_only:false},
                                Leg {kernel:second.clone(),from:base.clone(),ticks:cut,cycle_only:true},
                                Leg {kernel:terminal.clone(),from:carried.clone(),ticks:t,cycle_only:false}];
                            candidates.push((t1+t2+cut+t,legs));
                        }
                }
            }
        }
        let (_,legs)=candidates.into_iter().min_by_key(|(ticks,_)|*ticks).ok_or("no word preimage reaches the goal")?;
        thinking.inverse_ns=inverse_clock.elapsed().as_nanos();
        Ok((legs,label,lattice_cost))
    };
    let (legs,label,lattice_cost)=match work() {Ok(v)=>v,Err(explanation)=>return SolveResult {outcome:PlanOutcome::Unknown(Refusal::StateOrContext),stages,explanation:Some(explanation)}};
    let output_clock=std::time::Instant::now();
    let mut min=from.position;let mut max=from.position;
    let mut halo=[[0,0];3];
    for leg in &legs {
        let (lo,hi)=leg.kernel.envelope(&leg.from,leg.ticks,leg.cycle_only);
        min=Vec3::new(min.x.min(lo.x),min.y.min(lo.y),min.z.min(lo.z));max=Vec3::new(max.x.max(hi.x),max.y.max(hi.y),max.z.max(hi.z));
        for (a,h) in halo.iter_mut().enumerate() {h[0]=h[0].min(leg.kernel.cycle_reads_relative[a][0]);h[1]=h[1].max(leg.kernel.cycle_reads_relative[a][1]);}
    }
    let low=[min.x.floor() as i32+halo[0][0],min.y.floor() as i32+halo[1][0],min.z.floor() as i32+halo[2][0]];
    let high=[max.x.floor() as i32+halo[0][1],max.y.floor() as i32+halo[1][1],max.z.floor() as i32+halo[2][1]];
    let proof=match source.corridor_query(codec,label,low,high) {
        CorridorResult::Clear(p)=>p,
        CorridorResult::Exception {first_pos,exceptions}=>{stages.push(Stage::Cut);return SolveResult {outcome:PlanOutcome::NeedsJoints {exception:first_pos,exceptions,bracket},stages,explanation:None};},
        CorridorResult::Unknown(r)=>return SolveResult {outcome:PlanOutcome::Unknown(r),stages,explanation:None},
    };
    // A goal boundary and a terrain boundary go through the same cut/extend.
    stages.extend([Stage::Cut,Stage::Extend,Stage::Compose]);
    let mut words=Vec::new();for leg in &legs {append_steps(&mut words,leg,&proof);}
    let end=legs.last().map(leg_end).unwrap_or_else(||from.clone());let ticks=end.t-from.t;
    if let Some(step)=words.first_mut() {step.events.push(Event {tick:from.t,kind:EventKind::FoodGuard {sprint_allowed:sprint_allowed(from)}});}
    for i in 1..words.len() {if words[i].args.refined_yaw!=words[i-1].args.refined_yaw {let tick=words[i-1].expected_end_t;words[i].events.push(Event {tick,kind:EventKind::YawSwitch});}}
    if let Some(step)=words.last_mut() {step.events.push(Event {tick:end.t,kind:EventKind::GoalBoundary});}
    for step in &mut words {step.events.sort_by_key(|e|e.tick);}
    stages.extend([Stage::ClosedCheck,Stage::Allocate]);
    thinking.output_ns=output_clock.elapsed().as_nanos();
    SolveResult {outcome:PlanOutcome::Ready(Plan {ticks,words,end,bracket:Bracket {lb:bracket.lb,ub:Some(ticks)},label,deadline:deadline_ticks,lattice_cost,thinking}),stages,explanation:None}
}
