//! Owned handles to immutable phase templates. A readout contains the changing
//! coordinates/resources, not another PlayerState. Materialization is explicit.
use std::sync::Arc;
use law::{PlayerState,Vec3,BlockPos};
#[derive(Clone,Copy,Debug)]
pub struct View<'a> {pub phase:&'a PlayerState,pub position:Vec3,pub t:u64,pub main_supporting_block:Option<BlockPos>,pub food_level:i32,pub health:f32,pub saturation:f32,pub exhaustion:f32,pub food_tick_timer:u32}
impl View<'_> {pub fn bounding_box(&self)->law::Aabb {law::Aabb::player(self.position,self.phase.pose.height())}}
pub trait StateSource {fn view(&self)->View<'_>;}
impl StateSource for PlayerState {fn view(&self)->View<'_> {View {phase:self,position:self.position,t:self.t,main_supporting_block:self.main_supporting_block,food_level:self.food_level,health:self.health,saturation:self.saturation,exhaustion:self.exhaustion,food_tick_timer:self.food_tick_timer}}}
#[derive(Clone,Debug)]
pub struct StateAt {owner:Arc<PlayerState>,pub yaw:f32,pub movement:Option<Vec3>,pub position:Vec3,pub t:u64,pub velocity:Vec3,pub main_supporting_block:Option<BlockPos>,pub food_level:i32,pub health:f32,pub saturation:f32,pub exhaustion:f32,pub food_tick_timer:u32}
impl StateAt {
    pub fn point(state:PlayerState)->Self {Self::shared(Arc::new(state))}
    pub fn shared(state:Arc<PlayerState>)->Self {let v=state.view();let out=Self {owner:state.clone(),yaw:state.yaw,movement:state.movement_this_tick,position:v.position,t:v.t,velocity:v.phase.velocity,main_supporting_block:v.main_supporting_block,food_level:v.food_level,health:v.health,saturation:v.saturation,exhaustion:v.exhaustion,food_tick_timer:v.food_tick_timer};out}
    pub fn phase(&self)->&PlayerState {&self.owner}
    pub fn materialize(&self)->PlayerState {let mut s=self.phase().clone();s.position=self.position;s.velocity=self.velocity;s.yaw=self.yaw;s.input.yaw=self.yaw;s.movement_this_tick=self.movement;s.t=self.t;s.main_supporting_block=self.main_supporting_block;s.food_level=self.food_level;s.health=self.health;s.saturation=self.saturation;s.exhaustion=self.exhaustion;s.food_tick_timer=self.food_tick_timer;s}
    pub fn bits(&self)->Vec<u64> {self.materialize().bits()}
    pub fn bounding_box(&self)->law::Aabb {self.view().bounding_box()}
}
impl StateSource for StateAt {fn view(&self)->View<'_> {View {phase:self.phase(),position:self.position,t:self.t,main_supporting_block:self.main_supporting_block,food_level:self.food_level,health:self.health,saturation:self.saturation,exhaustion:self.exhaustion,food_tick_timer:self.food_tick_timer}}}
impl<T:StateSource> StateSource for Arc<T> {fn view(&self)->View<'_> {self.as_ref().view()}}
