//! Owned handles to immutable phase templates. A readout contains the changing
//! coordinates/resources, not another PlayerState. Materialization is explicit.
use std::sync::Arc;
use law::{PlayerState,Vec3,BlockPos};
use crate::store::WordKernel;
#[derive(Clone,Copy,Debug)]
pub struct View<'a> {pub phase:&'a PlayerState,pub position:Vec3,pub t:u64,pub main_supporting_block:Option<BlockPos>,pub food_level:i32,pub health:f32,pub saturation:f32,pub exhaustion:f32,pub food_tick_timer:u32}
impl View<'_> {pub fn bounding_box(&self)->law::Aabb {law::Aabb::player(self.position,self.phase.pose.height())}}
pub trait StateSource {fn view(&self)->View<'_>;}
impl StateSource for PlayerState {fn view(&self)->View<'_> {View {phase:self,position:self.position,t:self.t,main_supporting_block:self.main_supporting_block,food_level:self.food_level,health:self.health,saturation:self.saturation,exhaustion:self.exhaustion,food_tick_timer:self.food_tick_timer}}}
#[derive(Clone,Debug)]
enum Template {Point(Arc<PlayerState>),Kernel {kernel:Arc<WordKernel>,index:usize,prefix:bool}}
#[derive(Clone,Debug)]
pub struct StateAt {owner:Template,pub position:Vec3,pub t:u64,pub velocity:Vec3,pub main_supporting_block:Option<BlockPos>,pub food_level:i32,pub health:f32,pub saturation:f32,pub exhaustion:f32,pub food_tick_timer:u32}
impl StateAt {
    pub fn point(state:PlayerState)->Self {Self::shared(Arc::new(state))}
    pub fn shared(state:Arc<PlayerState>)->Self {let v=state.view();let out=Self {owner:Template::Point(state.clone()),position:v.position,t:v.t,velocity:v.phase.velocity,main_supporting_block:v.main_supporting_block,food_level:v.food_level,health:v.health,saturation:v.saturation,exhaustion:v.exhaustion,food_tick_timer:v.food_tick_timer};out}
    pub(crate) fn kernel(kernel:Arc<WordKernel>,index:usize,prefix:bool,from:View<'_>,position:Vec3,ticks:u64)->Self {
        let phase=kernel.phase_record(prefix,index);
        let velocity=phase.velocity;
        let main_supporting_block=phase.supported.then(||[position.x.floor() as i32,(position.y-1.0).floor() as i32,position.z.floor() as i32]);
        Self {owner:Template::Kernel {kernel,index,prefix},position,t:from.t+ticks,velocity,main_supporting_block,food_level:from.food_level,health:from.health,saturation:from.saturation,exhaustion:from.exhaustion,food_tick_timer:from.food_tick_timer}
    }
    pub fn phase(&self)->&PlayerState {match &self.owner {Template::Point(s)=>s,Template::Kernel {kernel,index,prefix}=>if *prefix {&kernel.transient_states[*index]}else {&kernel.cycle_states[*index]}}}
    pub fn materialize(&self)->PlayerState {let mut s=self.phase().clone();s.position=self.position;s.t=self.t;s.main_supporting_block=self.main_supporting_block;s.food_level=self.food_level;s.health=self.health;s.saturation=self.saturation;s.exhaustion=self.exhaustion;s.food_tick_timer=self.food_tick_timer;s}
    pub fn bits(&self)->Vec<u64> {self.materialize().bits()}
    pub fn bounding_box(&self)->law::Aabb {self.view().bounding_box()}
}
impl StateSource for StateAt {fn view(&self)->View<'_> {View {phase:self.phase(),position:self.position,t:self.t,main_supporting_block:self.main_supporting_block,food_level:self.food_level,health:self.health,saturation:self.saturation,exhaustion:self.exhaustion,food_tick_timer:self.food_tick_timer}}}
#[derive(Clone)]pub(crate) enum Cursor<'a> {Observed(&'a PlayerState),Generated(StateAt)}
impl StateSource for Cursor<'_> {fn view(&self)->View<'_> {match self {Self::Observed(s)=>s.view(),Self::Generated(s)=>s.view()}}}
impl StateSource for View<'_> {fn view(&self)->View<'_> {*self}}
impl View<'_> {
    pub fn materialize(&self)->PlayerState {let mut s=self.phase.clone();s.position=self.position;s.t=self.t;s.main_supporting_block=self.main_supporting_block;s.food_level=self.food_level;s.health=self.health;s.saturation=self.saturation;s.exhaustion=self.exhaustion;s.food_tick_timer=self.food_tick_timer;s}
}

impl<T:StateSource> StateSource for Arc<T> {fn view(&self)->View<'_> {self.as_ref().view()}}
