//! The shared generator store. Learning and acting use exactly the same preparation.
use std::{cell::{Cell,RefCell},collections::{HashMap,HashSet},sync::{Arc,atomic::{AtomicU64,Ordering}}};
use law::{PlayerState,Vec3,World,codec::{self,LabelId,StateQuestion},footprint::{Footprint,RecordingWorld},state_reads::{StateField as F,StateReads}};
use words::Arguments;
use crate::{events,lattice};
use std::borrow::Cow;

use words::program::{tiny_program_id,HeldPredicate::{Never,Always}};
pub const SPRINT_JUMP:u16=tiny_program_id(0,true,Always,Never);
pub const SPRINT:u16=tiny_program_id(0,true,Never,Never);
pub const WALK:u16=tiny_program_id(0,false,Never,Never);
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
pub struct StoreReads {pub kernels:u64,pub bundles:u64,pub bundle_hits:u64}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Hash)]
pub struct MotorKeyId(pub u32);
pub fn sprint_allowed(s:&PlayerState)->bool {
    codec::state_answers(s,StateReads(1u128<<F::food_level as u8))[2]!=0
}
fn guards(s:&PlayerState)->Vec<u64> {codec::state_answers(s,StateReads((1u128<<F::food_level as u8)|(1u128<<F::health as u8))).into_vec()}
pub fn motor_words(s:&PlayerState,reads:StateReads,anchor_y:f64)->Vec<u64> {
    let mut out=vec![reads.0 as u64,(reads.0>>64) as u64,(s.position.y-anchor_y).to_bits()];
    for f in reads.fields() {match f {
        F::t|F::position|F::food_level|F::health|F::saturation|F::exhaustion|F::food_tick_timer=>(),
        F::main_supporting_block=>out.extend_from_slice(&codec::state_answers(s,StateReads(1u128<<F::main_supporting_block as u8))),
        _=>s.push_field_bits(f,&mut out),
    }}
    out.extend(guards(s));out
}
fn cycle_words(s:&PlayerState,anchor:f64)->Vec<u64> {
    let mask=F::ALL.iter().fold(0u128,|bits,f|bits|(1u128<<*f as u8));
    motor_words(s,StateReads(mask),anchor)
}
#[derive(Clone,Debug)]
struct VerticalWord {prefix:Vec<(Option<f64>,events::Map)>,whole:events::Map,has_clamp:bool}
impl VerticalWord {
    fn new(ops:&[Vec3],clamped:&[[bool;3]],states:&[PlayerState],anchor:f64)->Self {
        let mut prefix=vec![(None,events::Map::new(Vec::new()))];let mut level=None;let mut tail=Vec::new();let mut has_clamp=false;
        for (i,op) in ops.iter().enumerate() {
            if clamped[i][1] {level=Some(states[i+1].position.y-anchor);tail.clear();has_clamp=true;} else {tail.push(op.y);}
            prefix.push((level,events::Map::new(tail.clone())));
        }
        Self {prefix,whole:events::Map::new(ops.iter().map(|op|op.y).collect()),has_clamp}
    }
    fn at(&self,start:f64,n:usize,anchor:f64)->f64 {
        let (level,map)=&self.prefix[n];map.advance(level.map_or(start,|y|anchor+y),map.len() as u64)
    }
    fn repeated(&self,start:f64,n:u64,anchor:f64)->f64 {
        let period=self.prefix.len()-1;
        if !self.has_clamp {return self.whole.advance(start,n);}
        let p=if n>=period as u64 {self.at(start,period,anchor)} else {start};
        self.at(p,(n%period as u64) as usize,anchor)
    }
}
#[derive(Clone,Debug)]
pub struct WordKernel {
    pub program:u16,pub yaw_index:u16,pub arguments:Arguments,
    pub start:PlayerState,pub anchor_y:f64,pub reads:StateReads,
    pub transient_ticks:u32,pub transient_states:Vec<PlayerState>,pub transient_endpoint:PlayerState,pub transient_reads:Footprint,
    pub cycle_period:u32,pub cycle_increments:Vec<Vec3>,pub cycle_clamped:Vec<[bool;3]>,pub cycle_level:Vec<f64>,
    pub cycle_displacement:Vec3,pub cycle_states:Vec<PlayerState>,pub cycle_reads_relative:[[i32;2];3],
    pub(crate) transient_increments:Vec<Vec3>,
    pub(crate) cycle_maps:[events::Map;2],pub(crate) transient_maps:[events::Map;2],
    transient_y:VerticalWord,cycle_y:VerticalWord,pub max_body_step:f64,
    transient_turns:Vec<u64>,cycle_turns:Vec<u64>,
}
impl WordKernel {
    /// Exact extrema: scalar addition maps are monotone. For any fixed phase,
    /// extrema over repeated cycles occur in the first or last cycle. Prefix
    /// extrema occur where a stored horizontal operand changes sign.
    pub fn envelope(&self,from:&PlayerState,ticks:u64,cycle_only:bool)->(Vec3,Vec3) {
        let read=|t|if cycle_only {self.cycle_at(from,t)} else {self.at(from,t)};
        let mut lo=from.position;let mut hi=lo;
        let mut include=|p:Vec3| {lo=Vec3::new(lo.x.min(p.x),lo.y.min(p.y),lo.z.min(p.z));hi=Vec3::new(hi.x.max(p.x),hi.y.max(p.y),hi.z.max(p.z));};
        let head=if cycle_only {0} else {ticks.min(self.transient_ticks as u64)};
        include(read(ticks).position);include(read(head).position);
        if head>0 {
            for &t in &self.transient_turns {if t<head {include(read(t).position);}}
            for s in self.prefix_phases(from).iter() {include(Vec3::new(from.position.x,s.position.y,from.position.z));}
        }
        let rest=ticks-head;
        if rest>0 {
            for &phase in &self.cycle_turns {
                if phase<=rest {include(read(head+phase).position);include(read(head+(rest-phase)/self.cycle_period as u64*self.cycle_period as u64+phase).position);}
            }
            let base=read(head);for s in self.periodic_phases(&base).iter() {include(Vec3::new(from.position.x,s.position.y,from.position.z));}
        }
        (lo,hi)
    }
    pub fn bearing_displacement(&self,at:Vec3)->Vec3 {
        // Read out the generator in the requested binade. A residual velocity
        // can be subnormal while its EXACT position map has zero displacement.
        Vec3::new(self.cycle_maps[0].advance(at.x,self.cycle_period as u64)-at.x,0.0,
            self.cycle_maps[1].advance(at.z,self.cycle_period as u64)-at.z)
    }
    pub fn cycle_at(&self,from:&PlayerState,ticks:u64)->PlayerState {
        let mut s=self.cycle_states[(ticks%self.cycle_period as u64) as usize].clone();
        s.position.x=self.cycle_maps[0].advance(from.position.x,ticks);
        s.position.z=self.cycle_maps[1].advance(from.position.z,ticks);
        let anchor=from.position.y.floor()+self.anchor_y-self.cycle_states[0].position.y.floor();
        s.position.y=self.cycle_y.repeated(from.position.y,ticks,anchor);
        Self::carry(s,from,ticks)
    }
    fn carry(mut s:PlayerState,from:&PlayerState,ticks:u64)->PlayerState {
        s.t=from.t+ticks;
        if s.main_supporting_block.is_some() {s.main_supporting_block=Some([s.position.x.floor() as i32,(s.position.y-1.0).floor() as i32,s.position.z.floor() as i32]);}
        s.food_level=from.food_level;s.health=from.health;s.saturation=from.saturation;s.exhaustion=from.exhaustion;s.food_tick_timer=from.food_tick_timer;s
    }
    /// Exact position addition operands, never a rounded displacement times q.
    pub fn at(&self,from:&PlayerState,ticks:u64)->PlayerState {
        let head=ticks.min(self.transient_ticks as u64);
        let rest=ticks-head;
        let mut s=if ticks<self.transient_ticks as u64 {self.transient_states[ticks as usize].clone()}
            else {self.cycle_states[(rest%self.cycle_period as u64) as usize].clone()};
        let x=self.transient_maps[0].advance(from.position.x,head);
        let z=self.transient_maps[1].advance(from.position.z,head);
        s.position.x=self.cycle_maps[0].advance(x,rest);
        s.position.z=self.cycle_maps[1].advance(z,rest);
        let anchor=from.position.y.floor();
        let y=self.transient_y.at(from.position.y,head as usize,anchor);
        s.position.y=self.cycle_y.repeated(y,rest,anchor);
        // The server owns resource numerics. The kernel predicts their guards;
        // it must never fabricate full bars on a cache hit.
        Self::carry(s,from,ticks)
    }
    pub fn cycle_start(&self,from:&PlayerState)->PlayerState {self.at(from,self.transient_ticks as u64)}
    pub fn phase(&self,phase:usize)->PlayerState {self.cycle_states[phase%self.cycle_states.len()].clone()}
    pub(crate) fn prefix_phases(&self,from:&PlayerState)->Cow<'_,[PlayerState]> {
        let anchor=from.position.y.floor();if anchor.to_bits()==self.anchor_y.to_bits() {return Cow::Borrowed(&self.transient_states);}
        Cow::Owned(self.transient_states.iter().enumerate().map(|(i,s)|{let mut s=s.clone();s.position.y=self.transient_y.at(from.position.y,i,anchor);s}).collect())
    }
    pub(crate) fn periodic_phases(&self,from:&PlayerState)->Cow<'_,[PlayerState]> {
        let anchor=from.position.y.floor()+self.anchor_y-self.cycle_states[0].position.y.floor();
        if anchor.to_bits()==self.anchor_y.to_bits() {return Cow::Borrowed(&self.cycle_states);}
        Cow::Owned(self.cycle_states.iter().enumerate().map(|(i,s)|{let mut s=s.clone();s.position.y=self.cycle_y.repeated(from.position.y,i as u64,anchor);s}).collect())
    }
}
pub struct WordBundle {
    pub first:Arc<WordKernel>,pub second:Arc<WordKernel>,
    pub incumbents:Vec<Arc<WordKernel>>,pub terminals:Vec<Vec<Arc<WordKernel>>>,
}
type Bucket=(LabelId,u16,u16,u32);
#[derive(Default)]
struct Inner {motors:HashMap<Vec<u8>,MotorKeyId>,entries:HashMap<Bucket,Vec<(MotorKeyId,Arc<WordKernel>)>>,prepared:HashSet<(LabelId,u16,u16)>,bundles:HashMap<(LabelId,MotorKeyId,u16,u16,u16),Arc<WordBundle>>}
pub struct GeneratorStore {pub joints:RefCell<crate::joints::JointStore>,inner:RefCell<Inner>,ticks:Cell<u64>,reads:Cell<StoreReads>,codec_key:Vec<u8>,codec_binding:Cell<Option<codec::StateClassId>>}
impl Default for GeneratorStore {
    fn default()->Self {
        static NEXT:AtomicU64=AtomicU64::new(0);
        let mut key=b"generator-store-owner/1".to_vec();key.extend(NEXT.fetch_add(1,Ordering::Relaxed).to_le_bytes());
        Self {joints:RefCell::default(),inner:RefCell::default(),ticks:Cell::new(0),reads:Cell::default(),codec_key:key,codec_binding:Cell::new(None)}
    }
}
impl GeneratorStore {
    /// LabelId and ClassId are local to one codec. One owner marker (not one
    /// class per motor key) prevents accidental reuse in another ID namespace.
    pub fn bind_codec(&self,codec:&mut codec::Interner)->bool {
        if let Some(id)=self.codec_binding.get() {return codec.find_state(&self.codec_key)==Some(id);}
        let Some(id)=codec.intern_state(&self.codec_key) else {return false;};self.codec_binding.set(Some(id));true
    }
    pub fn bundle(&self,label:LabelId,from:&PlayerState,program:u16,pair:[u16;2])->Option<Arc<WordBundle>> {
        let mut reads=self.reads.get();reads.bundles+=1;self.reads.set(reads);
        let first=self.get(label,from,program,pair[0])?;
        let bytes=codec::state_key(StateQuestion::Recurrence,&motor_words(from,first.reads,from.position.y.floor()));
        let inner=self.inner.borrow();let motor=*inner.motors.get(&bytes)?;
        let result=inner.bundles.get(&(label,motor,program,pair[0],pair[1])).cloned();
        if result.is_some() {let mut reads=self.reads.get();reads.bundle_hits+=1;self.reads.set(reads);}
        result
    }
    pub fn file_bundle(&self,label:LabelId,from:&PlayerState,program:u16,pair:[u16;2],bundle:WordBundle)->Arc<WordBundle> {
        let motor=self.intern_motor(&motor_words(from,bundle.first.reads,from.position.y.floor()));let bundle=Arc::new(bundle);
        self.inner.borrow_mut().bundles.insert((label,motor,program,pair[0],pair[1]),bundle.clone());bundle
    }
    pub fn preparation_ticks(&self)->u64 {self.ticks.get()}
    pub fn reads(&self)->StoreReads {self.reads.get()}
    pub fn reset_reads(&self) {self.reads.set(StoreReads::default());}
    pub fn motor_count(&self)->usize {self.inner.borrow().motors.len()}
    pub fn intern_motor(&self,words:&[u64])->MotorKeyId {
        let bytes=codec::state_key(StateQuestion::Recurrence,words);let mut inner=self.inner.borrow_mut();
        if let Some(&id)=inner.motors.get(&bytes) {return id;}
        let id=MotorKeyId(u32::try_from(inner.motors.len()).expect("motor ID capacity"));inner.motors.insert(bytes,id);id
    }
    pub fn prepared(&self,label:LabelId,program:u16,yaw:u16)->bool {self.inner.borrow().prepared.contains(&(label,program,yaw))}
    pub fn get(&self,label:LabelId,from:&PlayerState,program:u16,yaw:u16)->Option<Arc<WordKernel>> {
        let mut reads=self.reads.get();reads.kernels+=1;self.reads.set(reads);
        let inner=self.inner.borrow();
        let bucket=inner.entries.get(&(label,program,yaw,Arguments::default().pitch.to_bits()))?;
        for (id,k) in bucket {
            let key=codec::state_key(StateQuestion::Recurrence,&motor_words(from,k.reads,from.position.y.floor()));
            if inner.motors.get(&key)==Some(id) {return Some(k.clone());}
        }None
    }
    pub fn prepare(&self,label:LabelId,world:&impl World,from:&PlayerState,program:u16,yaw:u16)->Result<Arc<WordKernel>,String> {
        if world.difficulty()!=0 {return Err("U1 requires invariant resource guards; server food crossings are not predicted".into());}
        if from.effects || from.boat || from.riding || from.fall_flying || from.flying {return Err("unported movement regime".into());}
        if let Some(k)=self.get(label,from,program,yaw) {return Ok(k);}
        let programs=words::tiny_programs();let p=programs.get(program as usize).ok_or("unknown program")?;
        if p.timed() {return Err("clock-dependent program needs a clock recurrence".into());}
        let arguments=Arguments {refined_yaw:Some(lattice::yaw(yaw)),..Default::default()};
        let anchor=from.position.y.floor();
        let mut state=from.clone();let mut states=vec![state.clone()];let mut moved=Vec::new();let mut clamps=Vec::new();
        let mut seen=HashMap::new();seen.insert(cycle_words(&state,anchor),0);
        let metered=crate::source::MeteredWorld::new(world);
        let recording=RecordingWorld::new(&metered);let mut footprint=Footprint::default();
        let mut bounds=[[0,0];3];
        let (first,period)=loop {
            let origin=[state.position.x.floor() as i32,state.position.y.floor() as i32,state.position.z.floor() as i32];
            let input=p.select(&recording,&state,(states.len()-1) as u32,arguments).ok_or("no matching row")?.1;
            let (wanted,actual)=law::tick_with_collision(&mut state,&recording,&input).ok_or("word has no collision-law position update")?;
            self.ticks.set(self.ticks.get()+metered.take_ticks());
            // The law can clamp a tiny horizontal operand even on a flat
            // floor. Store the actual addition operand and the clamp bit on
            // EVERY axis; wanted movement is not a translation certificate.
            moved.push(actual);clamps.push([wanted.x.to_bits()!=actual.x.to_bits(),wanted.y.to_bits()!=actual.y.to_bits(),wanted.z.to_bits()!=actual.z.to_bits()]);
            let reads=recording.take_world();
            if reads.reads.keys().any(|&p|!world.loaded(p)) {return Err("loaded boundary during word preparation".into());}
            for cell in reads.reads.keys() {for a in 0..3 {bounds[a][0]=bounds[a][0].min(cell[a]-origin[a]);bounds[a][1]=bounds[a][1].max(cell[a]-origin[a]);}}
            footprint.extend(&reads);states.push(state.clone());
            let key=cycle_words(&state,anchor);
            if let Some(&f)=seen.get(&key) {break(f,states.len()-1-f);}
            seen.insert(key,states.len()-1);
        };
        let reads=footprint.state_reads;
        let increments=moved[first..first+period].to_vec();
        let displacement=increments.iter().fold(Vec3::default(),|s,v|s.plus(*v));
        let max_body_step=moved.iter().enumerate().flat_map(|(i,&v)|[v,v.plus(Vec3::new(0.0,states[i+1].pose.height() as f64-states[i].pose.height() as f64,0.0))])
            .map(|v|v.length_sqr().sqrt().next_up()).fold(0.0f64,f64::max);
        let turns=|ops:&[Vec3]| {
            let mut out=vec![0,ops.len() as u64];let mut last=[0.0;2];
            for (i,v) in ops.iter().enumerate() {for (a,d) in [v.x,v.z].into_iter().enumerate() {let sign=d.signum();if d!=0.0 {if last[a]!=0.0 && sign!=last[a] {out.push(i as u64);}last[a]=sign;}}}
            out.sort_unstable();out.dedup();out
        };
        let k=Arc::new(WordKernel {program,yaw_index:yaw,arguments,start:from.clone(),anchor_y:anchor,reads,
            transient_ticks:first as u32,transient_states:states[..=first].to_vec(),transient_endpoint:states[first].clone(),transient_reads:footprint,
            cycle_period:period as u32,cycle_increments:increments.clone(),cycle_clamped:clamps[first..first+period].to_vec(),
            cycle_level:states[first..first+period].iter().map(|s|s.position.y-anchor).collect(),cycle_displacement:displacement,
            cycle_states:states[first..first+period].to_vec(),cycle_reads_relative:bounds,
            transient_increments:moved[..first].to_vec(),
            cycle_maps:[events::Map::new(increments.iter().map(|v|v.x).collect()),events::Map::new(increments.iter().map(|v|v.z).collect())],
            transient_maps:[events::Map::new(moved[..first].iter().map(|v|v.x).collect()),events::Map::new(moved[..first].iter().map(|v|v.z).collect())],
            transient_y:VerticalWord::new(&moved[..first],&clamps[..first],&states[..=first],anchor),
            cycle_y:VerticalWord::new(&moved[first..first+period],&clamps[first..first+period],&states[first..=first+period],anchor),max_body_step,
            transient_turns:turns(&moved[..first]),cycle_turns:turns(&increments),
            });
        let id=self.intern_motor(&motor_words(from,reads,anchor));
        let mut inner=self.inner.borrow_mut();inner.entries.entry((label,program,yaw,arguments.pitch.to_bits())).or_default().push((id,k.clone()));
        inner.prepared.insert((label,program,yaw));Ok(k)
    }
    pub fn words(&self)->[u16;3] {[SPRINT_JUMP,SPRINT,WALK]}
}
