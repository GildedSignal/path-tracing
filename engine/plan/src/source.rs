//! Presence over one worldtube. The generator is a source implementation, not a solver lane.
use std::{cell::Cell, collections::BTreeMap};
use law::{Block, BlockPos, PlayerState, World, codec::{self, Interner, LabelId, LegsRole, Canonical, proof::{Source, Refusal}}, footprint::EnvironmentRead};

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct ReadCost { pub blocks:u64, pub loaded:u64, pub corridors:u64, pub scars:u64 }
#[derive(Clone, Debug)]
pub struct CorridorProof { pub label:LabelId, pub min:BlockPos, pub max:BlockPos, pub version:[u8;32] }
#[derive(Clone, Debug)]
pub enum CorridorResult { Clear(CorridorProof), Exception { first_pos:BlockPos, exceptions:usize }, Unknown(Refusal) }
pub trait CorridorSource: World + Source {
    fn support_cell(&self,state:&PlayerState)->BlockPos {
        state.main_supporting_block.unwrap_or_else(||[state.position.x.floor() as i32,state.position.y.floor() as i32-1,state.position.z.floor() as i32])
    }
    fn entry_label(&self,codec:&mut Interner,state:&PlayerState)->Result<LabelId,Refusal> where Self:Sized {
        label(codec,self,state)
    }
    fn corridor_query(&self, codec:&mut Interner, label:LabelId, min:BlockPos, max:BlockPos)->CorridorResult;
}

/// Counts the shared native tick entry, including refused calls.
/// Delegation preserves the source's collision acceleration and all observations.
pub struct MeteredWorld<'a,W> {pub world:&'a W,ticks:Cell<u64>}
impl<'a,W> MeteredWorld<'a,W> {
    pub fn new(world:&'a W)->Self {Self {world,ticks:Cell::new(0)}}
    pub fn ticks(&self)->u64 {self.ticks.get()}
    pub fn take_ticks(&self)->u64 {self.ticks.replace(0)}
}
impl<W:World> World for MeteredWorld<'_,W> {
    fn observe_axis_contact(&self,c:law::basis::sweep::AxisContact) {self.world.observe_axis_contact(c);}
    fn observe_tick(&self) {self.ticks.set(self.ticks.get()+1);self.world.observe_tick();}
    fn block(&self,p:BlockPos)->&Block {self.world.block(p)}
    fn hook_block(&self,p:BlockPos)->&Block {self.world.hook_block(p)}
    fn fluid_block(&self,p:BlockPos)->&Block {self.world.fluid_block(p)}
    fn raw_fluid_block(&self,p:BlockPos)->Option<&Block> {self.world.raw_fluid_block(p)}
    fn state_id(&self,p:BlockPos)->Option<law::basis::cell::StateId> {self.world.state_id(p)}
    fn uniform(&self,lo:BlockPos,hi:BlockPos)->Option<&Block> {self.world.uniform(lo,hi)}
    fn loaded(&self,p:BlockPos)->bool {self.world.loaded(p)}
    fn difficulty(&self)->u8 {self.world.difficulty()}
    fn natural_regeneration(&self)->bool {self.world.natural_regeneration()}
    fn min_y(&self)->i32 {self.world.min_y()}
    fn fast_lava(&self)->bool {self.world.fast_lava()}
    fn has_fluid(&self)->bool {self.world.has_fluid()}
    fn has_inside_effects(&self)->bool {self.world.has_inside_effects()}
    fn state_footprint(&self,r:law::state_reads::StateReads,w:law::state_reads::StateReads) {self.world.state_footprint(r,w)}
    fn observe_region(&self,q:law::footprint::Query,lo:BlockPos,hi:BlockPos) {self.world.observe_region(q,lo,hi)}
    fn observe_sweep(&self,a:law::Aabb,b:law::Aabb) {self.world.observe_sweep(a,b)}
    fn collision_cell_bounds(&self,b:law::Aabb,lo:BlockPos,hi:BlockPos)->Option<(BlockPos,BlockPos)> {self.world.collision_cell_bounds(b,lo,hi)}
    fn visit_collision_cells(&self,b:law::Aabb,lo:BlockPos,hi:BlockPos,visit:impl FnMut(BlockPos)->bool)->bool {self.world.visit_collision_cells(b,lo,hi,visit)}
    fn collision_shape(&self,p:BlockPos,c:law::CollisionContext)->&law::VoxelShape {self.world.collision_shape(p,c)}
    fn extra_collisions(&self,b:law::Aabb,c:law::CollisionContext)->Vec<law::VoxelShape> {self.world.extra_collisions(b,c)}
    fn visit_extra_collisions(&self,b:law::Aabb,c:law::CollisionContext,v:impl FnMut(law::shapes::ShapeView<'_>)->bool)->bool {self.world.visit_extra_collisions(b,c,v)}
}
impl<W:Source> Source for MeteredWorld<'_,W> {
    fn cell(&self,t:u64,p:BlockPos,m:u8)->Result<Canonical,Refusal> {self.world.cell(t,p,m)}
    fn environment(&self,t:u64,q:&EnvironmentRead)->Result<Canonical,Refusal> {self.world.environment(t,q)}
}
impl<W:CorridorSource> CorridorSource for MeteredWorld<'_,W> {
    fn support_cell(&self,s:&PlayerState)->BlockPos {self.world.support_cell(s)}
    fn entry_label(&self,c:&mut Interner,s:&PlayerState)->Result<LabelId,Refusal> {self.world.entry_label(c,s)}
    fn corridor_query(&self,c:&mut Interner,l:LabelId,lo:BlockPos,hi:BlockPos)->CorridorResult {self.world.corridor_query(c,l,lo,hi)}
}

pub fn label(codec:&mut Interner, world:&impl World, state:&PlayerState)->Result<LabelId,Refusal> {
    let feet=[state.position.x.floor() as i32,state.position.y.floor() as i32,state.position.z.floor() as i32];
    let support=state.main_supporting_block.unwrap_or([feet[0],feet[1]-1,feet[2]]);
    label_roles(codec,world,feet,support)
}
fn label_roles(codec:&mut Interner,world:&impl World,feet:BlockPos,support:BlockPos)->Result<LabelId,Refusal> {
    let mut classes=[None;4];
    for (i,(p,role)) in [(support,LegsRole::Support02),(support,LegsRole::Support05),(feet,LegsRole::BodyFeet),(feet,LegsRole::Inside)].into_iter().enumerate() {
        let loaded=world.loaded(p);
        if !loaded {return Err(Refusal::Loaded);}
        classes[i]=Some(codec.legs(world.block(p),loaded,role).map_err(Refusal::Law)?);
    }
    let classes=classes.map(Option::unwrap);
    codec.bare_legs_label(classes).ok_or(Refusal::Schema)
}

pub struct GeneratorSuperflatSource {
    pub floor_y:i32, pub floor_block:Block, pub air_block:Block,
    pub loaded_min:BlockPos, pub loaded_max:BlockPos,
    pub difficulty:u8, pub natural_regeneration:bool,
    pub scars:BTreeMap<BlockPos,Block>, pub provenance_hash:[u8;32],
    layers:BTreeMap<i32,Block>, costs:Cell<ReadCost>,
}
impl GeneratorSuperflatSource {
    pub fn new(floor_y:i32,floor_block:Block,loaded_min:BlockPos,loaded_max:BlockPos)->Self {
        let air_block=Block::default();let mut bytes=b"generator-superflat/1".to_vec();bytes.extend(codec::SCHEMA);
        bytes.extend(floor_y.to_le_bytes());for p in [loaded_min,loaded_max] {for v in p {bytes.extend(v.to_le_bytes());}}
        for b in [&floor_block,&air_block] {bytes.extend(codec::legs_presence(b).expect("qualified generator model").bytes());}
        let mut provenance_hash=[0;32];provenance_hash[..16].copy_from_slice(&codec::digest(&bytes).to_le_bytes());
        bytes.push(1);provenance_hash[16..].copy_from_slice(&codec::digest(&bytes).to_le_bytes());
        Self {floor_y,floor_block,air_block,loaded_min,loaded_max,difficulty:0,natural_regeneration:true,scars:BTreeMap::new(),provenance_hash,layers:BTreeMap::new(),costs:Cell::default()}
    }
    /// Factor the saved cells into A's supporting layers and sparse differences.
    /// This is ingestion, before the command clock. Playback reads the original snapshot.
    pub fn snapshot(world:&world::WorldSnapshot,a:BlockPos)->Result<Self,String> {Self::from_world(world,world.bounds(),a)}
    /// The same factoring over any world and box: the saved track, or the chunks the bot has received.
    pub fn from_world(world:&impl World,(lo,hi):(BlockPos,BlockPos),a:BlockPos)->Result<Self,String> {
        let mut source=Self::new(a[1],world.block(a).clone(),lo,hi);
        source.air_block=world::model(&world::BlockState {name:"minecraft:air".into(),properties:Vec::new()})?.clone();
        source.difficulty=world.difficulty();
        source.natural_regeneration=world.natural_regeneration();
        for y in lo[1]..a[1] {source.layers.insert(y,world.block([a[0],y,a[2]]).clone());}
        let mut differences=std::collections::HashMap::new();
        for y in lo[1]..=hi[1] {
            for z in lo[2]..=hi[2] {for x in lo[0]..=hi[0] {
                let p=[x,y,z];
                // A model's id where the world has one, else its address: equal keys, equal blocks.
                let id=world.model_id(p).map_or(world.block(p) as *const law::Block as usize,|m| m.0 as usize);
                let differs=match differences.entry((y,id)) {
                    std::collections::hash_map::Entry::Occupied(e)=>*e.get(),
                    std::collections::hash_map::Entry::Vacant(e)=>{
                        let same=codec::legs_presence(source.base(p)).and_then(|base|
                            codec::legs_presence(world.block(p)).map(|actual|actual==base));
                        *e.insert(!same.map_err(|e|format!("snapshot cell {p:?}: {e:?}"))?)
                    }
                };
                if differs {source.scars.insert(p,world.block(p).clone());}
            }}
        }
        Ok(source)
    }
    pub fn costs(&self)->ReadCost {self.costs.get()}
    pub fn reset_costs(&self) {self.costs.set(ReadCost::default());}
    fn base(&self,p:BlockPos)->&Block {if p[1]==self.floor_y {&self.floor_block} else {self.layers.get(&p[1]).unwrap_or(&self.air_block)}}
    fn covered(&self,p:BlockPos)->bool {(0..3).all(|a|p[a]>=self.loaded_min[a] && p[a]<=self.loaded_max[a])}
}
impl World for GeneratorSuperflatSource {
    fn block(&self,p:BlockPos)->&Block {
        let mut c=self.costs.get();c.blocks+=1;self.costs.set(c);
        self.scars.get(&p).unwrap_or_else(||self.base(p))
    }
    fn loaded(&self,p:BlockPos)->bool {let mut c=self.costs.get();c.loaded+=1;self.costs.set(c);self.covered(p)}
    fn difficulty(&self)->u8 {self.difficulty}
    fn natural_regeneration(&self)->bool {self.natural_regeneration}
    fn min_y(&self)->i32 {self.loaded_min[1]}
    fn collision_cell_bounds(&self,_bounds:law::Aabb,lo:BlockPos,hi:BlockPos)->Option<(BlockPos,BlockPos)> {
        // Occupancy is part of the generator: air proves emptiness, while the
        // floor and journal give conservative cell bounds. Keep the caller's
        // halo intact so shapes extending outside their cell remain visible.
        if !self.air_block.never_collides() {return Some((lo,hi));}
        let mut result:Option<(BlockPos,BlockPos)>=None;
        let mut include=|mut min:BlockPos,mut max:BlockPos| {
            for a in 0..3 {min[a]=min[a].max(lo[a]);max[a]=max[a].min(hi[a]);}
            if (0..3).any(|a|min[a]>max[a]) {return;}
            let (rlo,rhi)=result.get_or_insert((min,max));
            for a in 0..3 {rlo[a]=rlo[a].min(min[a]);rhi[a]=rhi[a].max(max[a]);}
        };
        if !self.floor_block.never_collides() {include([lo[0],self.floor_y,lo[2]],[hi[0],self.floor_y,hi[2]]);}
        for (&y,b) in self.layers.range(lo[1]..=hi[1]) {
            if !b.never_collides() {include([lo[0],y,lo[2]],[hi[0],y,hi[2]]);}
        }
        for (&p,b) in self.scars.range(lo..=hi) {if !b.never_collides() {include(p,p);}}
        result
    }
    fn visit_collision_cells(&self,bounds:law::Aabb,lo:BlockPos,hi:BlockPos,mut visit:impl FnMut(BlockPos)->bool)->bool {
        let Some((lo,hi))=self.collision_cell_bounds(bounds,lo,hi) else {return true;};
        for z in lo[2]..=hi[2] {for y in lo[1]..=hi[1] {for x in lo[0]..=hi[0] {if !visit([x,y,z]) {return false;}}}}
        true
    }
    fn has_fluid(&self)->bool {self.floor_block.fluid.kind!=law::FluidKind::Empty || self.air_block.fluid.kind!=law::FluidKind::Empty || self.scars.values().any(|b|b.fluid.kind!=law::FluidKind::Empty)}
    fn has_inside_effects(&self)->bool {std::iter::once(&self.floor_block).chain(std::iter::once(&self.air_block)).chain(self.scars.values()).any(|b|b.inside!=law::Inside::None || b.stuck_multiplier.is_some())}
}
impl Source for GeneratorSuperflatSource {
    fn cell(&self,_tick:u64,p:BlockPos,mask:u8)->Result<Canonical,Refusal> {
        if !self.covered(p) {return Err(Refusal::Coverage);}
        codec::cell(self.block(p),self.loaded(p),mask).map_err(Refusal::Law)
    }
    fn environment(&self,_tick:u64,q:&EnvironmentRead)->Result<Canonical,Refusal> {codec::environment(self,q).map_err(Refusal::Law)}
}
impl CorridorSource for GeneratorSuperflatSource {
    fn support_cell(&self,s:&PlayerState)->BlockPos {[s.position.x.floor() as i32,self.floor_y,s.position.z.floor() as i32]}
    fn entry_label(&self,codec:&mut Interner,state:&PlayerState)->Result<LabelId,Refusal> {
        let p=state.position;
        let feet=[p.x.floor() as i32,p.y.floor() as i32,p.z.floor() as i32];
        // The generator supplies the support surface even during the airborne
        // phase. The corridor query still qualifies its entire read footprint.
        let support=state.main_supporting_block.unwrap_or([feet[0],self.floor_y,feet[2]]);
        label_roles(codec,self,feet,support)
    }
    fn corridor_query(&self,codec:&mut Interner,expected:LabelId,min:BlockPos,max:BlockPos)->CorridorResult {
        let mut c=self.costs.get();c.corridors+=1;self.costs.set(c);
        if !self.covered(min) || !self.covered(max) {return CorridorResult::Unknown(Refusal::Loaded);}
        // Qualify the generator's role tuple, not its name. No cells along its length.
        let support=[min[0],self.floor_y,min[2]];
        let seed=PlayerState {position:law::Vec3::new(support[0] as f64+0.5,self.floor_y as f64+1.0,support[2] as f64+0.5),main_supporting_block:Some(support),..Default::default()};
        // Read the immutable generator, separately from its sparse edit journal.
        struct Base<'a>(&'a GeneratorSuperflatSource);
        impl World for Base<'_> {fn block(&self,p:BlockPos)->&Block {self.0.base(p)}}
        match label(codec,&Base(self),&seed) {Ok(id) if id==expected=>(),Ok(_)=>return CorridorResult::Exception {first_pos:support,exceptions:1},Err(r)=>return CorridorResult::Unknown(r)}
        let mut first=None;let mut exceptions=0;
        // Lexicographic range, then all three coordinate bounds: never mistake
        // BTreeMap ordering for a three-dimensional box.
        for (&p,b) in self.scars.range(min..=max) {
            let mut c=self.costs.get();c.scars+=1;self.costs.set(c);
            if !(0..3).all(|a|min[a]<=p[a] && p[a]<=max[a]) {continue;}
            let changed=[LegsRole::Support02,LegsRole::Support05,LegsRole::BodyFeet,LegsRole::Inside].into_iter().try_fold(false,|found,r| {
                Ok::<_,law::pattern::PatternError>(found || codec::legs_bytes(b,true,r)?!=codec::legs_bytes(self.base(p),true,r)?)
            });
            match changed {Ok(true)=>{exceptions+=1;first.get_or_insert(p);},Ok(false)=>(),Err(e)=>return CorridorResult::Unknown(Refusal::Law(e))}
        }
        if let Some(first_pos)=first {CorridorResult::Exception {first_pos,exceptions}}
        else {CorridorResult::Clear(CorridorProof {label:expected,min,max,version:self.provenance_hash})}
    }
}
