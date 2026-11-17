//! One loop. No source type, terrain name, uniform-ground flag, or legacy planner.
use std::sync::Arc;
use crate::{inline::List,state_view::{StateSource,StateAt,Cursor}};
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
pub struct WordStep {pub program:u16,pub args:Arguments,pub ticks:u32,pub expected_end_t:u64,pub events:List<Event,48>,pub certificate:CorridorProof,pub end:StateAt,pub map_segments:usize}
#[derive(Clone,Debug)]
pub struct Plan {pub ticks:u64,pub words:List<WordStep,12>,pub end:StateAt,pub bracket:Bracket,pub label:LabelId,pub deadline:u64,pub lattice_cost:lattice::LatticeCost,pub thinking:ThinkingCosts}
#[derive(Clone,Copy,Default,Debug)]
pub struct ThinkingCosts {pub entry_ns:u128,pub bearing_ns:u128,pub inverse_ns:u128,pub output_ns:u128}
#[derive(Clone,Debug)]
pub enum PlanOutcome {Ready(Plan),NeedsJoints {exception:BlockPos,exceptions:usize,bracket:Bracket},Unknown(Refusal)}
#[derive(Clone,Debug)]
pub struct SolveResult {pub outcome:PlanOutcome,pub stages:List<Stage,16>,pub explanation:Option<String>}

fn prepare(store:&GeneratorStore,label:LabelId,source:&impl World,from:&impl StateSource,program:u16,yaw:u16,stages:&mut List<Stage,16>)->Result<Arc<WordKernel>,String> {
    if let Some(k)=store.get(label,from,program,yaw) {return Ok(k);}
    let tick=store.preparation_ticks();let clock=std::time::Instant::now();
    let observed=from.view().materialize();let answer=store.prepare(label,source,&observed,program,yaw);
    stages.push(Stage::Learn {program,yaw,ticks:store.preparation_ticks()-tick,nanos:clock.elapsed().as_nanos()});answer
}
#[derive(Clone)]
struct Leg<'a> {kernel:Arc<WordKernel>,from:Cursor<'a>,ticks:u64,cycle_only:bool}
fn cycle_at(k:&Arc<WordKernel>,from:&impl StateSource,ticks:u64)->StateAt {k.readout(from,ticks,true)}
fn word_lower_bound(k:&WordKernel,from:&impl StateSource,goal:Goal)->u64 {
    let step=k.max_body_step;
    if step==0.0 {return u64::MAX;}
    let zone=goal.zone();let p=from.view().position;
    let extent=[p.x.abs(),p.y.abs(),p.z.abs(),zone.min.x.abs(),zone.min.y.abs(),zone.min.z.abs(),zone.max.x.abs(),zone.max.y.abs(),zone.max.z.abs()].into_iter().fold(0.0f64,f64::max)+step+from.view().phase.pose.height() as f64;
    let ulp=2.0f64.powi(events::Binary64::of(extent.next_up()).exp);
    let upper=(step+2.0*ulp*3.0f64.sqrt()).next_up();
    (goal.gap(from)/upper).next_down().ceil().max(0.0) as u64
}
fn append_steps(out:&mut List<WordStep,12>,leg:&Leg<'_>,proof:&CorridorProof) {
    let k=&leg.kernel;let head=if leg.cycle_only {0} else {leg.ticks.min(k.transient_ticks as u64)};
    let rest=leg.ticks-head;let cycles=rest/k.cycle_period as u64;let tail=rest%k.cycle_period as u64;
    let mut offset=0;let mut previous=leg.from.clone();
    for (ticks,count) in [(head,1),(cycles*k.cycle_period as u64,cycles),(tail,1)] {
        if ticks==0 {continue;}
        offset+=ticks;

        let mut events=List::new();
        let mut map_segments=0;
        let before=previous.view();let mut xy=[before.position.x,before.position.z];
        let maps=if offset<=head {&k.transient_maps} else {&k.cycle_maps};
        for a in 0..2 {let (at,segments)=maps[a].trace_boundaries(xy[a],ticks,|tick|events.push(Event {tick:before.t+tick,kind:EventKind::BinadeCrossing}));xy[a]=at;map_segments+=segments;}
        let end=k.readout_position(&leg.from,offset,leg.cycle_only,xy[0],xy[1]);previous=Cursor::Generated(end.clone());
        events.sort_unstable_by_key(|e|e.tick);events.dedup();
        out.push(WordStep {program:k.program,args:Arguments {count:count as u32,..k.arguments},ticks:ticks.try_into().expect("word tick width"),expected_end_t:end.t,events,certificate:proof.clone(),end,map_segments});
    }
}

pub fn solve<S:CorridorSource>(from:&PlayerState,to:Goal,kit:Kit,source:&S,store:&GeneratorStore,codec:&mut Interner,deadline_ticks:u64)->SolveResult {
    let Kit::BareLegs=kit; // A new kit must supply its own bound and word data.
    let mut stages=crate::list![Stage::Bracket,Stage::ClosedCheck];
    let bracket=Bracket {lb:to.lower_bound(from) as u64,ub:None};
    if !store.bind_codec(codec) {return SolveResult {outcome:PlanOutcome::Unknown(Refusal::Schema),stages,explanation:Some("generator store belongs to another codec".into())};}
    if to.touched(from,from) {
        let label=match source::label(codec,source,from) {Ok(label)=>label,Err(r)=>return SolveResult {outcome:PlanOutcome::Unknown(r),stages,explanation:None}};
        return SolveResult {outcome:PlanOutcome::Ready(Plan {ticks:0,words:List::new(),end:StateAt::point(from.clone()),bracket:Bracket {lb:0,ub:Some(0)},label,deadline:deadline_ticks,lattice_cost:Default::default(),thinking:Default::default()}),stages,explanation:None};
    }
    if source.difficulty()!=0 {return SolveResult {outcome:PlanOutcome::Unknown(Refusal::StateOrContext),stages,explanation:Some("U1 requires invariant resource guards; server food crossings are not predicted".into())};}
    // Read the certified joint relation for EVERY query. A known chain closes
    // the bracket; an absent chain supplies no negative reach claim.
    if let Some(route)=store.joints.borrow_mut().route(from,to) {
        let label=route.words.first().unwrap().certificate.label;
        stages.extend([Stage::Extend,Stage::Cut,Stage::Compose,Stage::ClosedCheck,Stage::Allocate]);
        return SolveResult {outcome:PlanOutcome::Ready(Plan {ticks:route.ticks,words:route.words,end:route.end,
            bracket:Bracket {lb:bracket.lb,ub:Some(route.ticks)},label,deadline:deadline_ticks,lattice_cost:Default::default(),thinking:Default::default()}),stages,explanation:None};
    }
    stages.push(Stage::Extend);
    let mut thinking=ThinkingCosts::default();let entry_clock=std::time::Instant::now();
    let mut work=||->Result<(List<Leg<'_>,4>,LabelId,lattice::LatticeCost),String> {
        let label=source::label(codec,source,from).map_err(|e|format!("entry label: {e:?}"))?;
        let delta=to.center().minus(from.position);let pair=lattice::bracket(delta.x,delta.z);
        let program=if sprint_allowed(from) {SPRINT_JUMP} else {WALK};
        let bundle=if let Some(bundle)=store.bundle(label,from,program,pair) {bundle} else {
            let first=prepare(store,label,source,from,program,pair[0],&mut stages)?;
            let c1=first.readout(from,first.transient_ticks as u64,false);
            let second=prepare(store,label,source,&c1,program,pair[1],&mut stages)?;
            let mut incumbents=Vec::new();let mut terminals=Vec::new();
            for word in store.words() {if sprint_allowed(from) || word==WALK {incumbents.push(prepare(store,label,source,from,word,pair[0],&mut stages)?);}}
            for phase in 0..second.cycle_period {
                let state=second.phase(phase as usize);let mut rows=Vec::new();
                for word in store.words() {if sprint_allowed(&state) || word==WALK {rows.push(prepare(store,label,source,&state,word,pair[1],&mut stages)?);}}
                terminals.push(rows);
            }
            let mut incumbent_terminals=Vec::new();
            for gait in &incumbents {
                let mut phases=Vec::new();
                for phase in 0..gait.cycle_period {
                    let state=gait.phase(phase as usize);let mut rows=Vec::new();
                    for word in store.words() {if sprint_allowed(&state) || word==WALK {rows.push(prepare(store,label,source,&state,word,gait.yaw_index,&mut stages)?);}}
                    phases.push(rows);
                }incumbent_terminals.push(phases);
            }
            store.file_bundle(label,from,program,pair,WordBundle {first,second,incumbents,terminals,incumbent_terminals})
        };
        let first=&bundle.first;
        let c1=first.readout(from,first.transient_ticks as u64,false);
        let second=&bundle.second;
        let c2=second.readout(&c1,second.transient_ticks as u64,false);
        thinking.entry_ns=entry_clock.elapsed().as_nanos();let bearing_clock=std::time::Instant::now();
        let e=c2.position.minus(from.position);
        let target=delta.minus(e);
        let d1=first.bearing_displacement(to.center());let d2=second.bearing_displacement(to.center());let d=d1.plus(d2);
        let radius=from.bounding_box().max.minus(from.position);
        // A fundamental-cell rectangle for the bearing. It retains the whole
        // Euclidean remainder; it never asserts that these counts touch B.
        let lo=Vec3::new(target.x-d.x.max(0.0)-radius.x,0.0,target.z-d.z.max(0.0)-radius.z);
        let hi=Vec3::new(target.x-d.x.min(0.0)+radius.x,0.0,target.z-d.z.min(0.0)+radius.z);
        let cap=Arguments::MAX_COUNT/first.cycle_period.max(second.cycle_period);
        let bearing=lattice::counts(d1,d2,lo,hi,cap);
        thinking.bearing_ns=bearing_clock.elapsed().as_nanos();let inverse_clock=std::time::Instant::now();
        let mut candidates:List<(u64,List<Leg<'_>,4>),32>=List::new();
        // The same store words also supply an incumbent from the observed
        // state, including displacements shorter than their entry transient.
        let mut incumbents:List<_,4>=bundle.incumbents.iter().enumerate().map(|(i,k)|(word_lower_bound(k,from,to),i,k)).collect();
        incumbents.sort_unstable_by_key(|&(lower,_,_)|lower);
        for (lower,index,k) in incumbents {
            let best=candidates.iter().map(|c|c.0).min().unwrap_or(words::Arguments::MAX_TICKS+1);
            if lower>=best {continue;}
            let head=k.transient_ticks as u64;
            if lower<=head {
                if let Some(t)=events::first_touch_readout(from.position,&k.transient_maps[0],&k.transient_maps[1],&k.prefix_geometry(from),to,head.min(best-1),false) {
                    candidates.push((t,crate::list![Leg {kernel:k.clone(),from:Cursor::Observed(from),ticks:t,cycle_only:false}]));
                }
            }
            if head>=best {continue;}
            let base=k.readout(from,head,false);
            let axis=if k.cycle_displacement.x.abs()>=k.cycle_displacement.z.abs() {0} else {2};
            let positive=if axis==0 {k.cycle_displacement.x>=0.0} else {k.cycle_displacement.z>=0.0};
            let zone=to.zone();let body=law::Aabb::player(Vec3::default(),from.pose.height());
            let target=if positive {[zone.min.x-body.max.x,zone.min.z-body.max.z][axis/2]}else {[zone.max.x-body.min.x,zone.max.z-body.min.z][axis/2]};
            let position=if axis==0 {base.position.x}else {base.position.z};
            let Some(touch)=k.cycle_maps[axis/2].first_cross(position,target,words::Arguments::MAX_TICKS) else {continue;};
            let touch=touch.max(1);let p=k.cycle_period as u64;let block=(touch-1)/p*p;
            let anchor_cut=block.saturating_sub(p);let anchor=k.readout(&base,anchor_cut,true);
            for phase in 0..p {
                let mut cut=block+phase;if cut>=touch {if cut<p {continue;}cut-=p;}
                let carried=k.readout(&anchor,cut-anchor_cut,true);
                let Some((terminal,t))=store.joints.borrow_mut().terminal(label,k,phase as u32,&bundle.incumbent_terminals[index][phase as usize],&carried,to)? else {continue;};
                let total=head+cut+t;let best=candidates.iter().map(|c|c.0).min().unwrap_or(u64::MAX);
                if total>=best {continue;}
                candidates.push((total,crate::list![Leg {kernel:k.clone(),from:Cursor::Observed(from),ticks:head+cut,cycle_only:false},Leg {kernel:terminal,from:Cursor::Generated(carried),ticks:t,cycle_only:false}]));
            }
        }
        let mut lattice_cost=lattice::LatticeCost::default();
        let mut bearings:List<_,2>=List::new();
        if let Some((q,cost))=bearing {lattice_cost=cost;bearings.push(q);}
        if let Some(q)=lattice::bearing_cell(d1,d2,target) {if !bearings.contains(&q) {bearings.push(q);}}
        for q in bearings {
            let t1=first.transient_ticks as u64+q[0] as u64*first.cycle_period as u64;
            let end1=first.readout(from,t1,false);
            let t2=second.transient_ticks as u64+q[1] as u64*second.cycle_period as u64;
            let base=second.readout(&end1,t2,false);
            // Extend to the goal event, relaxing only vertical eligibility to
            // locate its approach. The terminal preimage restores all phases.
            let horizon=events::horizon(&base,to,second.cycle_increments.iter().copied(),second.cycle_displacement);
            if let Some(touch)=events::first_touch_readout(base.position,&second.cycle_maps[0],&second.cycle_maps[1],&second.cycle_geometry(&base),to,horizon,true) {
                let p=second.cycle_period as u64;
                let block=(touch-1)/p*p;
                for phase in 0..p {
                    let mut cut=block+phase;
                    if cut>=touch {if cut<p {continue;}cut-=p;}
                    let carried=cycle_at(&second,&base,cut);
                    let Some((terminal,t))=store.joints.borrow_mut().terminal(label,second,phase as u32,&bundle.terminals[phase as usize],&carried,to)? else {continue;};
                    let total=t1+t2+cut+t;
                    let best=candidates.iter().map(|c|c.0).min().unwrap_or(u64::MAX);
                    if total>=best {continue;}
                    let legs=crate::list![Leg {kernel:first.clone(),from:Cursor::Observed(from),ticks:t1,cycle_only:false},
                        Leg {kernel:second.clone(),from:Cursor::Generated(end1.clone()),ticks:t2,cycle_only:false},
                        Leg {kernel:second.clone(),from:Cursor::Generated(base.clone()),ticks:cut,cycle_only:true},
                        Leg {kernel:terminal,from:Cursor::Generated(carried),ticks:t,cycle_only:false}];
                    candidates.push((total,legs));
                }            }
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
        let (lo,hi)=leg.kernel.envelope_view(&leg.from,leg.ticks,leg.cycle_only);
        min=Vec3::new(min.x.min(lo.x),min.y.min(lo.y),min.z.min(lo.z));max=Vec3::new(max.x.max(hi.x),max.y.max(hi.y),max.z.max(hi.z));
        for (a,h) in halo.iter_mut().enumerate() {h[0]=h[0].min(leg.kernel.cycle_reads_relative[a][0]);h[1]=h[1].max(leg.kernel.cycle_reads_relative[a][1]);}
    }
    let low=[min.x.floor() as i32+halo[0][0],min.y.floor() as i32+halo[1][0],min.z.floor() as i32+halo[2][0]];
    let high=[max.x.floor() as i32+halo[0][1],max.y.floor() as i32+halo[1][1],max.z.floor() as i32+halo[2][1]];
    let proof=match source.corridor_query(codec,label,low,high) {
        CorridorResult::Clear(p)=>p,
        CorridorResult::Exception {first_pos,exceptions}=>{stages.push(Stage::Cut);store.joints.borrow_mut().enqueue(from,bracket.lb,format!("corridor exception {first_pos:?}"));return SolveResult {outcome:PlanOutcome::NeedsJoints {exception:first_pos,exceptions,bracket},stages,explanation:None};},
        CorridorResult::Unknown(r)=>return SolveResult {outcome:PlanOutcome::Unknown(r),stages,explanation:None},
    };
    // A goal boundary and a terrain boundary go through the same cut/extend.
    stages.extend([Stage::Cut,Stage::Extend,Stage::Compose]);
    let mut words=List::new();for leg in &legs {append_steps(&mut words,leg,&proof);}
    let end=words.last().map(|w|w.end.clone()).unwrap_or_else(||StateAt::point(from.clone()));
    if let Some(step)=words.first_mut() {step.events.push(Event {tick:from.t,kind:EventKind::FoodGuard {sprint_allowed:sprint_allowed(from)}});}
    for i in 1..words.len() {if words[i].args.refined_yaw!=words[i-1].args.refined_yaw {let tick=words[i-1].expected_end_t;words[i].events.push(Event {tick,kind:EventKind::YawSwitch});}}
    if let Some(step)=words.last_mut() {step.events.push(Event {tick:end.t,kind:EventKind::GoalBoundary});}
    for step in &mut words {step.events.sort_unstable_by_key(|e|e.tick);}
    let route=crate::joints::compose_chain(from,to,words,&mut store.joints.borrow_mut().work);let words=route.words;let ticks=route.ticks;let end=route.end;
    stages.extend([Stage::ClosedCheck,Stage::Allocate]);
    thinking.output_ns=output_clock.elapsed().as_nanos();
    SolveResult {outcome:PlanOutcome::Ready(Plan {ticks,words,end,bracket:Bracket {lb:bracket.lb,ub:Some(ticks)},label,deadline:deadline_ticks,lattice_cost,thinking}),stages,explanation:None}
}
