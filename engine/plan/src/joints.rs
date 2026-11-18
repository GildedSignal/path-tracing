//! Goal-free joint facts and a goal-bound backward wave. Native replay belongs
//! only to learn(); acting consumes certificates and never chooses inputs.
use std::{cmp::Reverse,collections::{BinaryHeap,VecDeque}};
use law::solve_meter::HashMap;
use law::{BlockPos,PlayerState,Vec3,World,codec::{self,Interner,StateClassId,JointPatternId,
    proof::{Answer,Binding,ReadCertificates,Refusal,Source}},footprint::{Footprint,RecordingWorld}};
use words::{Arguments,Program};
use crate::{Goal,solve::WordStep,inline::List,state_view::StateAt};
#[path="family_wave.rs"] mod family_wave;
pub use family_wave::{FamilyInterval,ExitImage,Consumption};

#[derive(Clone,Debug)]
pub struct EntryTemplate {
    pub key:Vec<u64>,base:PlayerState,origin:Vec3,pub axis:Option<usize>,
    /// Absolute ordered binary64 lattice coordinate; no slope/interpolation.
    pub lattice:u64,pub s:f64,pub phase:u32,pub resources:Resources,
}
fn xyz(v:Vec3)->[f64;3] {[v.x,v.y,v.z]}
pub fn ordered(x:f64)->u64 {let b=x.to_bits();if b>>63!=0 {!b} else {b^(1<<63)}}
pub fn unordered(b:u64)->f64 {f64::from_bits(if b>>63!=0 {b^(1<<63)} else {!b})}
#[derive(Clone,Copy,Debug)]
pub struct Resources {pub food:i32,pub saturation:f32,pub exhaustion:f32,pub fall_distance:f64}
impl Resources {
    fn of(s:&PlayerState)->Self {Self {food:s.food_level,saturation:s.saturation,exhaustion:s.exhaustion,fall_distance:s.fall_distance}}
    fn same_bits(self,other:Self)->bool {self.food==other.food && self.saturation.to_bits()==other.saturation.to_bits() && self.exhaustion.to_bits()==other.exhaustion.to_bits() && self.fall_distance.to_bits()==other.fall_distance.to_bits()}
}
impl EntryTemplate {
    pub fn new(state:&PlayerState,origin:Vec3,phase:u32)->Self {
        let v=xyz(state.velocity);
        let axis=if v[0]==0.0 && v[2]==0.0 {None} else {Some(if v[0].abs()>=v[2].abs() {0} else {2})};
        Self::with_axis(state,origin,phase,axis)
    }
    pub fn with_axis(state:&PlayerState,origin:Vec3,phase:u32,axis:Option<usize>)->Self {
        let s=axis.map_or(0.0,|a|xyz(state.position)[a]);let mut key=Vec::new();
        Self::write_key(state,phase,axis,&mut key);
        Self {key,base:state.clone(),origin,axis,lattice:ordered(s),s,phase,resources:Resources::of(state)}
    }
    fn write_key(state:&PlayerState,phase:u32,axis:Option<usize>,key:&mut Vec<u64>) {
        use law::state_reads::StateField as F;let p=xyz(state.position);
        key.clear();
        for &field in F::ALL {
            match field {
                F::exhaustion|F::saturation|F::food_level|F::fall_distance|F::yaw|F::pitch|F::input=>(),
                F::position=>for a in 0..3 {key.push(if Some(a)==axis {p[a].to_bits()>>52} else {p[a].to_bits()});},
                F::main_supporting_block=>{
                    key.push(state.main_supporting_block.is_some() as u64);
                    if let Some(support)=state.main_supporting_block {key.extend((0..3).map(|a|(support[a] as i64-p[a].floor() as i64) as u64));}
                },
                _=>state.push_field_bits(field,key),
            }
        }
        key.extend([phase as u64,axis.map_or(3,|a|a) as u64]);
    }
    pub fn state(&self,s:f64)->Option<PlayerState> {
        if !s.is_finite() {return None;}
        let mut out=self.base.clone();let old=xyz(out.position);
        if let Some(a)=self.axis {
            let mut p=old;p[a]=s;out.position=Vec3::new(p[0],p[1],p[2]);
            if let Some(mut support)=out.main_supporting_block {support[a]+=p[a].floor() as i32-old[a].floor() as i32;out.main_supporting_block=Some(support);}
        } else if s.to_bits()!=self.s.to_bits() {return None;}
        let check=Self::with_axis(&out,self.origin,self.phase,self.axis);
        (check.key==self.key && check.s.to_bits()==s.to_bits()).then_some(out)
    }
}
/// No producer exists until a kit-wide envelope is derived. A missed learning
/// proposal cannot construct this value or turn a searched gap into Forbidden.
#[derive(Clone,Debug)]
pub struct EnvelopeProof {pub premises:Vec<u8>,pub separating_cells:Vec<BlockPos>,pub law_hash:[u8;32],certificate:usize,binding:Binding}
#[derive(Clone,Debug)]
pub enum Status {Found,Forbidden(EnvelopeProof), Searched {work:u64},Refused {reason:String}}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Hash)]
pub struct EdgeId(pub usize);
#[derive(Clone,Debug)]
pub struct Witness {pub s_bits:u64,pub end_digest:u128}
#[derive(Clone,Debug)]
pub struct Piece {
    pub lo:u64,pub hi:u64,pub status:Status,pub ticks:u32,pub tick_range:(u32,u32),pub exit_lo:[u64;3],pub exit_hi:[u64;3],
    pub witness:Witness,pub signature:Vec<u128>,pub grounded:bool,
    /// The legacy point facts have only their witnessed resource guard. A
    /// broader interval must be proved before it can consume other numerics.
    pub uses:crate::families::UseInterval,resource_guard:Resources,pub exit:ExitImage,
    binding:Binding,certificate:usize,from:usize,to:usize,step:WordStep,last_before:PlayerState,
}
pub struct JointPattern {pub id:JointPatternId,pub entry:StateClassId,pub law_hash:[u8;32],pub origin:Vec3,pub pieces:Vec<Piece>}
#[derive(Clone,Debug)]
pub struct Demand {pub entry:Vec<u64>,pub resource_guard:Option<Resources>,pub face:Option<BlockPos>,pub lower:u64,pub reason:String}
#[derive(Clone,Debug)]
pub struct Attempt {pub entry:EntryTemplate,pub program:u16,pub args:Arguments,pub status:Status,pub witness:Witness,pub reads:Vec<(u64,Footprint)>}
#[derive(Default,Clone,Copy,Debug)]
pub struct Work {pub waves:u64,pub edge_reads:u64,pub settled:u64,pub law_ticks:u64,pub terminal_builds:u64,pub terminal_lookups:u64,#[cfg(feature="solve-counters")] pub terminal_build_ns:u128}
#[derive(Clone,Debug)]
pub struct Route {pub words:List<WordStep,12>,pub ticks:u64,pub end:StateAt}
/// The field's partition is separate from the goal-free pattern partition.
#[derive(Clone,Debug)]
pub struct InstanceInterval {pub lo:f64,pub hi:f64,pub t:Option<u64>,pub via:Option<EdgeId>}
#[derive(Clone,Debug)]
pub struct Field {pub goal:Goal,pub t:Vec<Option<u64>>,pub via:Vec<Option<EdgeId>>,pub alternate:Vec<Option<EdgeId>>,pub partitions:Vec<Vec<InstanceInterval>>,pub families:Vec<FamilyInterval>,pub prices:HashMap<Vec<u8>,i128>}
impl Field {
    pub fn set_price(&mut self,resource:Vec<u8>,without:Option<u64>,with:Option<u64>,lower_without:u64,lower_with:u64) {
        self.prices.insert(resource,price(without,with,lower_without,lower_with));
    }
    /// The upstream caller supplies its certified reachable interval. With
    /// singleton facts there is no assertion about an unproved sliding move.
    pub fn slide_want(&self,pattern:usize,at:f64,reachable:(f64,f64))->bool {
        let Some(partition)=self.partitions.get(pattern) else {return false;};
        let Some(current)=partition.iter().find(|p|p.lo<=at && at<=p.hi).and_then(|p|p.t) else {return false;};
        partition.iter().any(|p|p.lo<=reachable.1 && reachable.0<=p.hi && p.t.is_some_and(|t|t<current))
    }
}

fn backward(n:usize,edges:&[(usize,usize,u64,EdgeId)],terminals:impl Iterator<Item=usize>,goal:Goal,work:&mut Work)->Field {
    work.waves+=1;
    let mut t=vec![None;n];let mut via=vec![None;n];let mut alternate=vec![None;n];
    let mut incoming=vec![Vec::new();n];let mut heap=BinaryHeap::new();
    for node in terminals {if t[node].is_none() {t[node]=Some(0);heap.push(Reverse((0u64,node)));}}
    for &(from,to,cost,id) in edges {work.edge_reads+=1;incoming[to].push((from,cost,id));}
    let mut settled=vec![false;n];let mut count=0;
    while let Some(Reverse((cost,node)))=heap.pop() {
        if settled[node] || t[node]!=Some(cost) {continue;}
        settled[node]=true;count+=1;
        for &(from,step,id) in &incoming[node] {
            let candidate=cost.checked_add(step).expect("tick cost overflow");
            if t[from].is_none_or(|old|candidate<old) {
                alternate[from]=via[from];t[from]=Some(candidate);via[from]=Some(id);heap.push(Reverse((candidate,from)));
            } else if via[from]!=Some(id) && alternate[from].is_none() {alternate[from]=Some(id);}
        }
    }
    work.settled+=count;
    Field {goal,t,via,alternate,partitions:Vec::new(),families:Vec::new(),prices:HashMap::default()}
}

/// Uniform generator chains and learned joint edges use this same min-plus
/// operation. The caller has already certified every WordStep.
pub(crate) fn compose_chain(from:&PlayerState,_goal:Goal,words:List<WordStep,12>,work:&mut Work)->Route {
    // Exact min-plus recurrence on a certified linear chain. Branching joint
    // graphs use the same recurrence with min over their incoming edges.
    work.waves+=1;work.edge_reads+=words.len() as u64;work.settled+=words.len() as u64+1;
    let ticks=words.iter().rev().fold(0u64,|tail,w|tail.checked_add(w.ticks as u64).expect("tick cost overflow"));
    let end=words.last().map(|w|w.end.clone()).unwrap_or_else(||StateAt::point(from.clone()));Route {ticks,words,end}
}
#[derive(Default)]
pub struct JointStore {
    pub patterns:Vec<JointPattern>,pub attempts:Vec<Attempt>,pub queue:VecDeque<Demand>,pub work:Work,pub inventory:Option<crate::faces::FaceInventory>,
    templates:HashMap<Vec<u64>,StateClassId>,template_bytes:usize,
    states:Vec<std::sync::Arc<PlayerState>>,state_ids:HashMap<Vec<u64>,usize>,edges:Vec<(usize,usize)>,
    point_entries:HashMap<Vec<u64>,Vec<usize>>,point_scratch:Vec<u64>,
    certificates:ReadCertificates,field:Option<Field>,family_entries:HashMap<usize,EntryTemplate>,
}
impl JointStore {
    pub fn template_bytes(&self)->usize {self.template_bytes}
    pub fn cold_state_bytes(&self)->usize {self.states.iter().map(|s|s.bits().len()*std::mem::size_of::<u64>()).sum()}
    pub fn edge_count(&self)->usize {self.edges.len()}
    /// Resolve an interned point image without copying its player state.
    pub fn point(&self,id:usize)->&PlayerState {&self.states[id]}
    pub fn end(&self,id:EdgeId)->&PlayerState {&self.states[self.edge(id).to]}
    fn node(&mut self,s:&PlayerState)->usize {
        let bits=s.bits();if let Some(&id)=self.state_ids.get(&bits) {return id;}
        let id=self.states.len();self.states.push(std::sync::Arc::new(s.clone()));self.state_ids.insert(bits,id);
        let mut key=Vec::new();EntryTemplate::write_key(s,0,None,&mut key);self.point_entries.entry(key).or_default().push(id);self.field=None;id
    }
    fn equivalent_points(&self,state:&PlayerState)->impl Iterator<Item=usize>+'_ {
        let mut key=Vec::new();EntryTemplate::write_key(state,0,None,&mut key);let resources=Resources::of(state);
        self.point_entries.get(&key).into_iter().flatten().copied().filter(move |&node|Resources::of(&self.states[node]).same_bits(resources))
    }
    pub fn enqueue(&mut self,s:&PlayerState,lower:u64,reason:impl Into<String>) {
        let mut entry=Vec::new();EntryTemplate::write_key(s,0,None,&mut entry);let resources=Resources::of(s);
        if self.queue.iter().any(|d|d.entry==entry && d.resource_guard.is_some_and(|r|r.same_bits(resources))) {return;}
        self.queue.push_back(Demand {entry,resource_guard:Some(resources),face:None,lower,reason:reason.into()});
    }
    pub fn inventory(&mut self,w:&(impl World+Source),codec:&mut Interner,lo:BlockPos,hi:BlockPos)->Result<(),Refusal> {
        self.inventory=Some(crate::faces::FaceInventory::build(w,codec,lo,hi)?);Ok(())
    }
    pub fn world_edit(&mut self,w:&(impl World+Source),codec:&mut Interner,p:BlockPos,ticks:(u64,u64))->Result<usize,Refusal> {
        let broken=self.edit(codec,p,ticks,w);
        if let Some(inventory)=&mut self.inventory {
            inventory.created.clear();inventory.edit(w,codec,p)?;
            for face in &inventory.created {
                if !self.queue.iter().any(|d|d.face==Some(face.cell)) {self.queue.push_back(Demand {entry:Vec::new(),resource_guard:None,face:Some(face.cell),lower:0,reason:"inventory edit created a joint candidate".into()});}
            }
        }
        Ok(broken)
    }
    /// The law witnesses a transition once. Budget exhaustion is Searched;
    /// it cannot produce a negative reach proof. The stop predicate belongs to
    /// the learning task; it is absent from acting and from the stored key.
    pub fn learn<W:World+Source>(&mut self,world:&W,codec:&mut Interner,start:&PlayerState,
        origin:Vec3,phase:u32,program_id:u16,program:&Program,args:Arguments,
        mut stop:impl FnMut(&PlayerState,&PlayerState,u32)->bool)->Result<EdgeId,Status> {
        let entry=EntryTemplate::new(start,origin,phase);
        let toolkit=words::tiny_programs();
        let Some(canonical)=toolkit.get(program_id as usize) else {return Err(Status::Refused {reason:"program ID outside toolkit".into()});};
        let metered=crate::source::MeteredWorld::new(world);
        let recorder=RecordingWorld::new(&metered);let mut state=start.clone();let mut reads=Vec::new();let mut signature=Vec::new();
        for elapsed in 0..args.max_ticks {
            let Some((_,input))=program.select(&recorder,&state,elapsed,args) else {
                let status=Status::Refused {reason:"no matching row".into()};
                self.attempt(entry,program_id,args,status.clone(),&state,reads);return Err(status);
            };
            let before=state.clone();
            if canonical.select(&recorder,&state,elapsed,args).map(|(_,i)|i)!=Some(input) {
                let status=Status::Refused {reason:"program does not match its toolkit ID".into()};
                self.attempt(entry,program_id,args,status.clone(),&state,reads);return Err(status);
            }
            let result=law::try_tick(&mut state,&recorder,&input);self.work.law_ticks+=metered.take_ticks();
            if let Err(e)=result {
                reads.push((before.t,recorder.take_world()));
                let status=Status::Refused {reason:e.0};self.attempt(entry,program_id,args,status.clone(),&state,reads);return Err(status);
            }
            signature.push(words::segment::joint_events(&before,&state).0);
            reads.push((before.t,recorder.take_world()));
            if stop(&before,&state,elapsed+1) {
                return self.file(codec,world,&entry,start,state,before,program_id,args,reads,signature);
            }
        }
        self.enqueue(start,0,"word budget spent without an exit");
        let status=Status::Searched {work:args.max_ticks as u64};
        self.attempt(entry,program_id,args,status.clone(),&state,reads);Err(status)
    }
    fn attempt(&mut self,entry:EntryTemplate,program:u16,args:Arguments,status:Status,state:&PlayerState,reads:Vec<(u64,Footprint)>) {
        let witness=Witness {s_bits:entry.s.to_bits(),end_digest:codec::digest(&codec::state_key(codec::StateQuestion::Exact,&state.bits()))};
        self.attempts.push(Attempt {entry,program,args,status,witness,reads});
    }
    fn file(&mut self,codec:&mut Interner,source:&(impl Source+World),entry:&EntryTemplate,start:&PlayerState,
        end:PlayerState,last_before:PlayerState,program:u16,args:Arguments,reads:Vec<(u64,Footprint)>,signature:Vec<u128>)->Result<EdgeId,Status> {
        let error=|r:Refusal|Status::Refused {reason:format!("certificate: {r:?}")};
        let entry_id=if let Some(&id)=self.templates.get(&entry.key) {id} else {
            let bytes=codec::state_key(codec::StateQuestion::Exact,&entry.key);
            let id=codec.intern_state(&bytes).ok_or_else(||error(Refusal::Schema))?;
            self.template_bytes+=bytes.len();self.templates.insert(entry.key.clone(),id);id
        };
        // Identity-frame pattern content includes every timed cell answer.
        // A pattern is only published after capture has qualified all reads.
        let pattern=JointPatternId(u32::try_from(self.patterns.len()).map_err(|_|error(Refusal::Schema))?);
        let binding=Binding::plateau(pattern,0,law::LAW_HASH);
        let refs:Vec<_>=reads.iter().map(|(t,f)|(*t,f)).collect();
        let certificate=self.certificates.capture(codec,binding.clone(),&refs,source).map_err(error)?;
        let from=self.node(start);let to=self.node(&end);let ticks=(end.t-start.t).try_into().unwrap();
        let positions=xyz(end.position).map(f64::to_bits);
        let label=crate::source::label(codec,source,start).map_err(error)?;
        // WordStep's compatibility corridor handle is never the joint proof;
        // the ReadCertificates record above is the sole authority for this edge.
        let cells=self.certificates.cells(certificate).unwrap();
        let mut min=[i32::MAX;3];let mut max=[i32::MIN;3];
        for c in cells {for a in 0..3 {min[a]=min[a].min(c.position[a]);max[a]=max[a].max(c.position[a]);}}
        let step=WordStep {program,args,ticks,expected_end_t:end.t,events:Default::default(),certificate:crate::source::CorridorProof {label,min,max,version:law::LAW_HASH},end:StateAt::shared(self.states[to].clone())};
        let witness=Witness {s_bits:entry.s.to_bits(),end_digest:codec::digest(&codec::state_key(codec::StateQuestion::Exact,&end.bits()))};
        let uses=crate::families::UseInterval::witness(start,&end);
        let plateau=Piece {lo:entry.lattice,hi:entry.lattice,status:Status::Found,ticks,tick_range:(ticks,ticks),exit_lo:positions,exit_hi:positions,witness,signature,grounded:start.on_ground,uses,resource_guard:entry.resources,exit:ExitImage::Points(vec![to]),binding,certificate,from,to,step,last_before};
        let id=EdgeId(self.edges.len());self.edges.push((self.patterns.len(),0));
        self.patterns.push(JointPattern {id:pattern,entry:entry_id,law_hash:law::LAW_HASH,origin:entry.origin,pieces:vec![plateau]});self.field=None;Ok(id)
    }
    fn edge(&self,id:EdgeId)->&Piece {let (p,i)=self.edges[id.0];&self.patterns[p].pieces[i]}
    fn valid(&self,id:EdgeId)->bool {
        let (p,_)=self.edges[id.0];let e=self.edge(id);
        self.patterns[p].law_hash==law::LAW_HASH && matches!(e.status,Status::Found) &&
            matches!(self.certificates.answer(e.certificate,&e.binding),Answer::Yes(_))
    }
    pub fn proven_forbidden(&self,status:&Status)->bool {
        let Status::Forbidden(p)=status else {return false;};
        p.law_hash==law::LAW_HASH && !p.premises.is_empty() && !p.separating_cells.is_empty() &&
            matches!(self.certificates.answer(p.certificate,&p.binding),Answer::Yes(_))
    }
    pub fn consume(&self,id:EdgeId,state:&PlayerState,phase:u32)->Result<Consumption<'_>,Refusal> {
        let (p,_)=self.edges[id.0];let pattern=&self.patterns[p];let e=self.edge(id);
        let entry=if let Some(template)=self.family_entries.get(&p) {EntryTemplate::with_axis(state,pattern.origin,phase,template.axis)}else {EntryTemplate::new(state,pattern.origin,phase)};
        if self.templates.get(&entry.key)!=Some(&pattern.entry) || entry.lattice<e.lo || entry.lattice>e.hi || !entry.resources.same_bits(e.resource_guard) {return Err(Refusal::StateOrContext);}
        if !self.valid(id) {return Err(Refusal::Changed);}
        Ok(Consumption {program:e.step.program,args:e.step.args,ticks:e.tick_range,exit:&e.exit,coordinate:entry.lattice})
    }
    pub fn edit(&mut self,codec:&Interner,p:BlockPos,ticks:(u64,u64),source:&impl Source)->usize {
        let broken=self.certificates.edit(codec,p,ticks,source);self.invalidate(&broken)
    }
    pub fn environment_event(&mut self,codec:&Interner,ticks:(u64,u64),source:&impl Source)->usize {
        let broken=self.certificates.environment_event(codec,ticks,source);self.invalidate(&broken)
    }
    fn invalidate(&mut self,broken:&[usize])->usize {
        let mut count=0;let mut demands=Vec::new();
        for pattern in &mut self.patterns {for e in &mut pattern.pieces {
            if broken.contains(&e.certificate) {e.status=Status::Searched {work:0};demands.push(e.from);count+=1;}
        }}
        for from in demands {let state=self.states[from].clone();self.enqueue(&state,0,"read certificate invalidated");}
        if count>0 {self.field=None;}
        count
    }
    /// A single Dijkstra pass on certified joint edges. It never calls the law,
    /// queries candidate inputs, or deduplicates by a cell's address.
    pub fn wave(&mut self,goal:Goal)->&Field {
        if self.field.as_ref().is_some_and(|f|f.goal.cell==goal.cell) {return self.field.as_ref().unwrap();}
        let mut terminals:Vec<_>=self.states.iter().enumerate().filter_map(|(i,s)|goal.touched(s,s).then_some(i)).collect();
        // A swept touch belongs to an edge, not to its overshooting endpoint.
        // A later command from that endpoint must not inherit a zero-time goal.
        let swept_terminal=self.states.len();terminals.push(swept_terminal);
        let mut edges=Vec::new();
        for i in 0..self.edges.len() {
            let id=EdgeId(i);if !self.valid(id) || self.family_entries.contains_key(&self.edges[i].0) {continue;}
            let e=self.edge(id);let to=if goal.touched(&e.last_before,&self.states[e.to]) {swept_terminal} else {e.to};
            edges.push((e.from,to,e.ticks as u64,id));
        }
        let mut field=backward(self.states.len()+1,&edges,terminals.into_iter(),goal,&mut self.work);
        field.partitions=self.patterns.iter().map(|p|p.pieces.iter().map(|e|InstanceInterval {lo:unordered(e.lo),hi:unordered(e.hi),t:field.t[e.from],via:field.via[e.from]}).collect()).collect();
        family_wave::extend(self,&mut field);
        self.field=Some(field);self.field.as_ref().unwrap()
    }
    pub fn route(&mut self,start:&PlayerState,goal:Goal)->Option<Route> {
        if self.state_ids.is_empty() {return None;}self.wave(goal);
        EntryTemplate::write_key(start,0,None,&mut self.point_scratch);
        let field=self.field.as_ref().unwrap();let resources=Resources::of(start);
        let mut node=self.point_entries.get(&self.point_scratch)?.iter().copied().filter(|&n|Resources::of(&self.states[n]).same_bits(resources)).filter_map(|n|field.t[n].map(|t|(t,n))).min()?.1;
        let ticks=field.t[node]?;let mut words=List::new();
        while let Some(id)=field.via[node] {
            let e=self.edge(id);if !self.valid(id) || self.family_entries.contains_key(&self.edges[id.0].0) {return None;}words.push(e.step.clone());node=e.to;
            if goal.touched(&e.last_before,&self.states[node]) {break;}
        }
        Some(Route {words,ticks,end:StateAt::shared(self.states[node].clone())})
    }
}

/// Monotone integer preimage of one downstream interval; used to refine an
/// instance partition, never to claim an unknown pattern interval is certified.
pub fn pullback(lo:u64,hi:u64,target:(u64,u64),map:impl Fn(u64)->u64)->Option<(u64,u64)> {
    let increasing=map(lo)<=map(hi);
    let value=|s|if increasing {map(s)} else {u64::MAX-map(s)};
    let (a,b)=if increasing {target} else {(u64::MAX-target.1,u64::MAX-target.0)};
    let lower=|threshold:u64| {let mut l=lo;let mut h=hi;while l<h {let m=l+(h-l)/2;if value(m)<threshold {l=m+1;} else {h=m;}}l};
    let first=lower(a);if value(first)<a || value(first)>b {return None;}
    let mut l=first;let mut h=hi;while l<h {let m=l+(h-l).div_ceil(2);if value(m)>b {h=m-1;} else {l=m;}}
    Some((first,l))
}

pub fn price(without:Option<u64>,with:Option<u64>,lower_without:u64,lower_with:u64)->i128 {
    match (without,with) {(Some(a),Some(b))=>a as i128-b as i128,_=>lower_without as i128-lower_with as i128}
}

#[derive(Clone,Debug)]
pub struct ReachArc {pub program:u16,pub args:Arguments,pub ticks:u32,pub end:PlayerState,pub delta:Vec3}
/// A finite learning result, keyed by the exact arriving template. Each row is
/// a program and an event exit. No per-cell or speed-band deduplication.
pub struct MirroredReach {pub entry:EntryTemplate,pub arcs:Vec<ReachArc>,pub searched:usize,pub refused:Vec<String>,pub ticks:u64}
impl MirroredReach {
    pub fn compile(world:&impl World,start:&PlayerState,origin:Vec3,phase:u32,words:&[(u16,Program,Arguments)])->Self {
        let counted=crate::source::MeteredWorld::new(world);
        let mut result=Self {entry:EntryTemplate::new(start,origin,phase),arcs:Vec::new(),searched:0,refused:Vec::new(),ticks:0};
        for (id,program,args) in words {
            let mut s=start.clone();let mut air=!s.on_ground;let mut finished=false;
            for elapsed in 0..args.max_ticks {
                let Some((_,input))=program.select(&counted,&s,elapsed,*args) else {result.refused.push("no matching row".into());finished=true;break;};
                if let Err(e)=law::try_tick(&mut s,&counted,&input) {result.refused.push(e.0);finished=true;break;}
                if (air && s.on_ground) || (air && s.on_climbable) {
                    result.arcs.push(ReachArc {program:*id,args:*args,ticks:elapsed+1,delta:s.position.minus(start.position),end:s});finished=true;break;
                }
                air|=!s.on_ground;
            }
            if !finished {result.searched+=1;}
        }
        result.ticks=counted.ticks();result
    }
    /// Mirror the target by each table row. This is a necessary positional
    /// filter, never a motion certificate at another entry coordinate.
    pub fn preimages<'a>(&'a self,face:&crate::faces::Face)->impl Iterator<Item=&'a ReachArc> {
        let target=face.bounds;
        self.arcs.iter().filter(move |arc| {
            let b=arc.end.bounding_box();let r=(b.max.x-b.min.x)/2.0;
            let mirrored=law::Aabb::new(target.min.x-r-arc.delta.x,target.min.y-arc.delta.y,target.min.z-r-arc.delta.z,
                target.max.x+r-arc.delta.x,target.max.y-arc.delta.y,target.max.z+r-arc.delta.z);
            let p=self.entry.base.position;
            p.x>=mirrored.min.x && p.x<=mirrored.max.x && p.y>=mirrored.min.y && p.y<=mirrored.max.y && p.z>=mirrored.min.z && p.z<=mirrored.max.z
        })
    }
}
