//! Observations of the existing collision recurrence. These values never
//! substitute a movement or change collider visitation order.
use crate::{Aabb,Axis,Vec3};
pub const CONTACT_ZERO:f64=1.0e-7;
pub const CONTACT_HIT:f64=1.0e-5f32 as f64;
pub const CONTACT_CONSTANTS:&[(&str,f64,&str)]=&[
    ("contact remainder / overlap",CONTACT_ZERO,"shapes.rs:ShapeView::collide; movement.rs:collide_in_world"),
    ("horizontal hit",CONTACT_HIT,"math.rs:equal"),
    ("position update squared gate",1.0e-7,"movement.rs:move_player"),
];
#[derive(Clone,Copy,Debug,PartialEq)]
pub enum Outcome {Free,Clamp {plane:f64},Hold {plane:Option<f64>,remainder:f64}}
impl Outcome {
    pub fn kind(self)->u8 {match self {Self::Free=>0,Self::Clamp {..}=>1,Self::Hold {..}=>2}}
    pub fn plane(self)->Option<f64> {match self {Self::Free=>None,Self::Clamp {plane}=>Some(plane),Self::Hold {plane,..}=>plane}}
}
#[derive(Clone,Copy,Debug)]
pub struct AxisContact {pub axis:Axis,pub bounds:Aabb,pub body:Aabb,pub wanted:f64,pub moved:f64,pub outcome:Outcome}
impl AxisContact {
    pub(crate) fn free(axis:Axis,bounds:Aabb,body:Aabb,wanted:f64)->Self {Self {axis,bounds,body,wanted,moved:wanted,outcome:Outcome::Free}}
}
#[derive(Clone,Copy,Debug)]
pub(crate) struct Pass {pub moved:Vec3,pub axes:[AxisContact;3]}
impl Pass {
    pub(crate) fn free(bounds:Aabb,body:Aabb,wanted:Vec3)->Self {
        Self {moved:wanted,axes:[Axis::X,Axis::Y,Axis::Z].map(|a|AxisContact::free(a,bounds,body,wanted.get(a)))}
    }
    pub(crate) fn report(&self,world:&impl crate::World) {for axis in crate::shapes::axis_step_order(Vec3::new(self.axes[0].wanted,self.axes[1].wanted,self.axes[2].wanted)) {world.observe_axis_contact(self.axes[axis as usize]);}}
}

use crate::{Block,BlockPos,PlayerState,Input,World};
use std::cell::RefCell;
struct ContactRecorder<'a,W> {world:&'a W,passes:RefCell<Vec<AxisContact>>}
impl<W:World> World for ContactRecorder<'_,W> {
    fn observe_axis_contact(&self,c:AxisContact) {self.passes.borrow_mut().push(c);self.world.observe_axis_contact(c);}
    fn observe_tick(&self) {self.world.observe_tick();}
    fn block(&self,p:BlockPos)->&Block {self.world.block(p)}
    fn hook_block(&self,p:BlockPos)->&Block {self.world.hook_block(p)}
    fn fluid_block(&self,p:BlockPos)->&Block {self.world.fluid_block(p)}
    fn raw_fluid_block(&self,p:BlockPos)->Option<&Block> {self.world.raw_fluid_block(p)}
    fn model_id(&self,p:BlockPos)->Option<u16> {self.world.model_id(p)}
    fn loaded(&self,p:BlockPos)->bool {self.world.loaded(p)}
    fn difficulty(&self)->u8 {self.world.difficulty()}
    fn natural_regeneration(&self)->bool {self.world.natural_regeneration()}
    fn min_y(&self)->i32 {self.world.min_y()}
    fn fast_lava(&self)->bool {self.world.fast_lava()}
    fn has_fluid(&self)->bool {self.world.has_fluid()}
    fn has_inside_effects(&self)->bool {self.world.has_inside_effects()}
    fn read_state(&self,f:crate::state_reads::StateField) {
        self.world.read_state(f);
    }
    fn write_state(&self,f:crate::state_reads::StateField) {self.world.write_state(f)}
    fn observe_region(&self,q:crate::footprint::Query,lo:BlockPos,hi:BlockPos) {self.world.observe_region(q,lo,hi)}
    fn observe_sweep(&self,a:crate::Aabb,b:crate::Aabb) {self.world.observe_sweep(a,b)}
    fn collision_cell_bounds(&self,b:crate::Aabb,lo:BlockPos,hi:BlockPos)->Option<(BlockPos,BlockPos)> {self.world.collision_cell_bounds(b,lo,hi)}
    fn visit_collision_cells(&self,b:crate::Aabb,lo:BlockPos,hi:BlockPos,visit:impl FnMut(BlockPos)->bool)->bool {self.world.visit_collision_cells(b,lo,hi,visit)}
    fn collision_shape(&self,p:BlockPos,c:crate::CollisionContext)->&crate::VoxelShape {self.world.collision_shape(p,c)}
    fn extra_collisions(&self,b:crate::Aabb,c:crate::CollisionContext)->Vec<crate::VoxelShape> {self.world.extra_collisions(b,c)}
    fn visit_extra_collisions(&self,b:crate::Aabb,c:crate::CollisionContext,v:impl FnMut(crate::shapes::ShapeView<'_>)->bool)->bool {self.world.visit_extra_collisions(b,c,v)}
}
/// The native tick with opt-in contact observations, in selected axis-pass order.
pub fn tick(state:&mut PlayerState,world:&impl World,input:&Input)->Option<(Vec3,Vec3,Vec<AxisContact>)> {
    let recorder=ContactRecorder {world,passes:RefCell::new(Vec::new())};
    let (wanted,moved)=crate::tick_with_collision(state,&recorder,input)?;
    Some((wanted,moved,recorder.passes.into_inner()))
}
