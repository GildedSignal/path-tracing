//! Presence over one worldtube. The generator is a source implementation, not a solver lane.
use std::{cell::Cell, collections::{BTreeMap,HashMap}};
use law::{basis::cell::StateId, Block, BlockPos, PlayerState, World, codec::{self, Interner, LabelId, LegsRole, Canonical, proof::{Source, Refusal}}, footprint::EnvironmentRead};

#[derive(Clone, Debug)]
pub struct CorridorProof { pub label:LabelId, pub min:BlockPos, pub max:BlockPos, pub version:[u8;32] }
#[derive(Clone, Debug)]
pub enum CorridorResult { Clear(CorridorProof), Exception { first_pos:BlockPos, exceptions:usize }, Unknown(Refusal) }
/// Discard any replay that reads a cell whose model is unavailable.
pub struct ObservedWorld<'a,W> {
    world:&'a W, missing:Cell<Option<BlockPos>>, empty:Block,
    indexed:bool, fluids:bool, inside:bool,
}
impl<'a,W:World> ObservedWorld<'a,W> {
    pub fn new(world:&'a W)->Self {
        Self { world, missing:Cell::new(None), empty:Default::default(), indexed:false, fluids:world.has_fluid(), inside:world.has_inside_effects() }
    }
    pub(crate) fn defect(world:&'a W)->Self where W:CorridorSource {
        Self {indexed:world.indexed_collisions(), ..Self::new(world)}
    }
    pub(crate) fn reset_missing(&self) {self.missing.set(None);}
    fn mark_missing(&self,p:BlockPos) {if self.missing.get().is_none() {self.missing.set(Some(p));}}
    pub fn check(&self)->Result<(),String> {match self.missing.get() {Some(p)=>Err(format!("cell {p:?}: {}",self.world.known_block(p).err().unwrap_or("unavailable"))),None=>Ok(())}}
}
impl<W:World> World for ObservedWorld<'_,W> {
    fn observe_axis_contact(&self,c:law::basis::sweep::AxisContact) {self.world.observe_axis_contact(c);}
    fn observe_tick(&self) {self.world.observe_tick();}
    #[inline]
    fn known_block(&self,p:BlockPos)->Result<&Block,&str> {self.world.known_block(p).inspect_err(|_|self.mark_missing(p))}
    #[inline]
    fn block(&self,p:BlockPos)->&Block {self.known_block(p).unwrap_or(&self.empty)}
    #[inline]
    fn loaded(&self,p:BlockPos)->bool {
        if self.indexed && self.world.loaded(p) {return true;}
        self.known_block(p).is_ok() && !self.indexed && self.world.loaded(p)
    }
    fn state_id(&self,p:BlockPos)->Option<StateId> {self.world.state_id(p)}
    fn uniform(&self,lo:BlockPos,hi:BlockPos)->Option<&Block> {self.world.uniform(lo,hi)}
    fn min_y(&self)->i32 {self.world.min_y()}
    fn difficulty(&self)->u8 {self.world.difficulty()}
    fn natural_regeneration(&self)->bool {self.world.natural_regeneration()}
    fn fast_lava(&self)->bool {self.world.fast_lava()}
    fn has_fluid(&self)->bool {self.fluids}
    fn has_inside_effects(&self)->bool {self.inside}
    fn has_fluid_in(&self,lo:BlockPos,hi:BlockPos)->bool {self.fluids && self.world.has_fluid_in(lo,hi)}
    fn has_inside_effects_in(&self,lo:BlockPos,hi:BlockPos)->bool {self.inside && self.world.has_inside_effects_in(lo,hi)}
    fn collision_cell_bounds(&self,b:law::Aabb,lo:BlockPos,hi:BlockPos)->Option<(BlockPos,BlockPos)> {
        if self.indexed {self.world.collision_cell_bounds(b,lo,hi)} else {Some((lo,hi))}
    }
    fn visit_collision_cells(&self,b:law::Aabb,lo:BlockPos,hi:BlockPos,mut visit:impl FnMut(BlockPos)->bool)->bool {
        if self.indexed {
            // The law checks loaded before requesting a candidate's shape.
            return self.world.visit_collision_cells(b,lo,hi,|p|visit(p) && self.missing.get().is_none());
        }
        for z in lo[2]..=hi[2] {for y in lo[1]..=hi[1] {for x in lo[0]..=hi[0] {
            let p=[x,y,z];
            let Ok(b)=self.known_block(p) else {return false;};
            if !b.never_collides() && self.world.loaded(p) && !visit(p) {return false;}
        }}}
        true
    }
}
pub trait CorridorSource: World + Source {
    /// Candidate traversal includes unknown cells even if their fallback is air.
    fn indexed_collisions(&self)->bool {false}
    fn support_cell(&self,state:&PlayerState)->BlockPos {
        state.main_supporting_block.unwrap_or_else(||[state.position.x.floor() as i32,state.position.y.floor() as i32-1,state.position.z.floor() as i32])
    }
    fn floor_y(&self, state: &PlayerState) -> i32 {
        self.support_cell(state)[1]
    }
    fn provenance(&self) -> [u8; 32] {
        [0; 32]
    }
    fn bounds_box(&self) -> Option<(BlockPos, BlockPos)> {
        None
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
    fn known_block(&self,p:BlockPos)->Result<&Block,&str> {self.world.known_block(p)}
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
    fn has_fluid_in(&self,lo:BlockPos,hi:BlockPos)->bool {self.world.has_fluid_in(lo,hi)}
    fn has_inside_effects_in(&self,lo:BlockPos,hi:BlockPos)->bool {self.world.has_inside_effects_in(lo,hi)}
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
    fn indexed_collisions(&self)->bool {self.world.indexed_collisions()}
    fn support_cell(&self,s:&PlayerState)->BlockPos {self.world.support_cell(s)}
    fn floor_y(&self, state: &PlayerState) -> i32 {self.world.floor_y(state)}
    fn provenance(&self) -> [u8; 32] {self.world.provenance()}
    fn bounds_box(&self) -> Option<(BlockPos, BlockPos)> {self.world.bounds_box()}
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
    let eyes=eyes_class(codec,world,feet)?;
    codec.bare_legs_label(classes,eyes).ok_or(Refusal::Schema)
}
/// The eyes' class where the law reads it to choose a mode (a fluid or a stuck cell at the eyes).
pub fn eyes_class(codec:&mut Interner,world:&impl World,feet:BlockPos)->Result<Option<law::codec::ClassId>,Refusal> {
    let eyes=[feet[0],feet[1]+1,feet[2]];
    if !world.loaded(eyes) {return Err(Refusal::Loaded);}
    let b=world.block(eyes);
    if !b.has_inside_effects() {return Ok(None);}
    codec.legs(b,true,LegsRole::Inside).map(Some).map_err(Refusal::Law)
}

// A section stores palette indices for cells that differ from the generator.
// A mask skips 4³ boxes whose cells all use the generator.
struct Section { cells:[u32;4096], altered:u64, collision_xor:[[u16;256];2], flags:u8 }
fn block_flags(block: &Block) -> u8 {
    u8::from(block.fluid.kind != law::FluidKind::Empty) | (u8::from(block.has_inside_effects()) << 1)
}
fn local(p:BlockPos)->usize { (((p[1]&15)<<8)|((p[2]&15)<<4)|(p[0]&15)) as usize }
fn x_mask(lo:i32,hi:i32)->u16 { (u16::MAX<<lo) & (u16::MAX>>(15-hi)) }
fn cube(p:BlockPos)->u64 { 1 << (((p[1]&12)<<2)|(p[2]&12)|((p[0]&12)>>2)) }
// Bit 0 includes every collision candidate, bit 1 includes candidates in the halo.
fn reach(block:&Block)->u8 {
    let mut result=u8::from(block.inside==law::Inside::PowderSnow);
    for shape in block.collision_shapes().filter(|s|!s.is_empty()) {
        if !shape.is_block() && shape.coords.iter().any(|axis|axis[0]<0.0 || axis[axis.len()-1]>1.0) {return 3;}
        result=1;
    }
    result
}
fn boxes(lo:BlockPos,hi:BlockPos,shift:u32,mut visit:impl FnMut(BlockPos,BlockPos)) {
    for z in lo[2]>>shift..=hi[2]>>shift {for y in lo[1]>>shift..=hi[1]>>shift {for x in lo[0]>>shift..=hi[0]>>shift {
        let a=[x<<shift,y<<shift,z<<shift];
        visit(std::array::from_fn(|i|lo[i].max(a[i])),std::array::from_fn(|i|hi[i].min(a[i]|((1<<shift)-1))));
    }}}
}

enum Row {
    Missing(String),
    Known { model: Option<Model>, state: Option<StateId> },
}
enum Model { Canonical(law::basis::cell::ModelId), Custom(Box<Block>) }
impl Model {
    fn at(&self,p:BlockPos)->&Block {
        match self {Self::Canonical(id)=>id.block_at(p),Self::Custom(block)=>block}
    }
}

/// Terrain with a sparse mutable journal. The canonical model table holds
/// block behaviour, and the section rows hold canonical states when available.
#[derive(Default)]
pub struct Scars {
    edits:BTreeMap<BlockPos,Block>,
    sections:Vec<Option<Box<Section>>>, origin:BlockPos, span:[usize;3],
    rows:Vec<Row>, has_model_scars:bool, has_unknown:bool,
}
impl Scars {
    pub fn insert(&mut self,p:BlockPos,b:Block)->Option<Block> {self.edits.insert(p,b)}
    pub fn is_empty(&self)->bool {!self.has_model_scars && self.edits.is_empty()}
    #[inline]
    fn section_index(&self,p:BlockPos)->Option<usize> {
        if self.sections.is_empty() {return None;}
        let [x,y,z]=[0,1,2].map(|i|((p[i]>>4)-self.origin[i]) as usize);
        let [nx,ny,nz]=self.span;
        (x<nx && y<ny && z<nz).then(||(z*ny+y)*nx+x)
    }
    #[inline]
    fn section(&self,p:BlockPos)->Option<&Section> { self.sections[self.section_index(p)?].as_deref() }
    #[inline]
    fn row(&self,p:BlockPos)->Option<&Row> {
        let row=self.section(p)?.cells[local(p)];
        self.rows.get(row.wrapping_sub(1) as usize)
    }
    fn paint(&mut self,lo:BlockPos,hi:BlockPos,row:u32,change:u8,flags:u8) {
        if row==0 {return;}
        let i=self.section_index(lo).unwrap();
        let s=self.sections[i].get_or_insert_with(||Box::new(Section {cells:[0;4096],altered:0,collision_xor:[[0;256];2],flags:0}));
        s.flags |= flags;
        let mask=x_mask(lo[0]&15,hi[0]&15);
        for z in lo[2]..=hi[2] {
            let (a,b)=(local([lo[0],lo[1],z]),local([hi[0],lo[1],z]));
            s.cells[a..=b].fill(row);
            for i in 0..2 {if change & (1<<i)!=0 {s.collision_xor[i][a>>4]|=mask;}}
        }
        boxes(lo,hi,2,|a,_|s.altered|=cube(a));
    }
}
fn cells(lo:BlockPos,hi:BlockPos,mut visit:impl FnMut(BlockPos)->bool)->bool {
    for z in lo[2]..=hi[2] {for y in lo[1]..=hi[1] {for x in lo[0]..=hi[0] {
        if !visit([x,y,z]) {return false;}
    }}}
    true
}

pub struct GeneratorSuperflatSource {
    pub floor_y:i32, pub floor_block:Block, pub air_block:Block,
    pub loaded_min:BlockPos, pub loaded_max:BlockPos,
    pub difficulty:u8, pub natural_regeneration:bool,
    pub scars:Scars, pub provenance_hash:[u8;32],
    layers:BTreeMap<i32,Block>,
    first_uncovered:Cell<Option<BlockPos>>,
    floor_state:Option<StateId>, air_state:Option<StateId>, layer_states:BTreeMap<i32,StateId>,
}
impl GeneratorSuperflatSource {
    pub fn first_uncovered(&self) -> Option<BlockPos> { self.first_uncovered.get() }
    pub fn new(floor_y:i32,floor_block:Block,loaded_min:BlockPos,loaded_max:BlockPos)->Self {
        let air_block=Block::default();let mut bytes=b"generator-superflat/1".to_vec();bytes.extend(codec::SCHEMA);
        bytes.extend(floor_y.to_le_bytes());for p in [loaded_min,loaded_max] {for v in p {bytes.extend(v.to_le_bytes());}}
        for b in [&floor_block,&air_block] {bytes.extend(codec::legs_presence(b).expect("qualified generator model").bytes());}
        let mut provenance_hash=[0;32];provenance_hash[..16].copy_from_slice(&codec::digest(&bytes).to_le_bytes());
        bytes.push(1);provenance_hash[16..].copy_from_slice(&codec::digest(&bytes).to_le_bytes());
        Self {floor_y,floor_block,air_block,loaded_min,loaded_max,difficulty:0,natural_regeneration:true,scars:Scars::default(),provenance_hash,layers:BTreeMap::new(),first_uncovered:Cell::new(None),floor_state:None,air_state:None,layer_states:BTreeMap::new()}
    }
    /// Factor the saved cells into A's supporting layers and sparse differences.
    /// Playback still reads the original snapshot.
    pub fn snapshot(world:&world::WorldSnapshot,a:BlockPos)->Result<Self,String> {
        Self::ingest(world,world.bounds(),a,|lo,hi|{
            let state=world.uniform_state(lo,hi)?;
            let model=state.model().filter(|id|id.fixed())?;
            Some((model.block_at(lo),Some(state)))
        })
    }
    /// The same factoring over any world and box, including the bot's received chunks.
    pub fn from_world(world:&impl World,bounds:(BlockPos,BlockPos),a:BlockPos)->Result<Self,String> {
        Self::ingest(world,bounds,a,|lo,hi|world.uniform(lo,hi).map(|block|(block,None)))
    }
    fn ingest<'a>(
        world:&'a impl World,(lo,hi):(BlockPos,BlockPos),a:BlockPos,
        uniform:impl Fn(BlockPos,BlockPos)->Option<(&'a Block,Option<StateId>)>,
    )->Result<Self,String> {
        let floor=world.known_block(a).map_err(|e|format!("support {a:?}: {e}"))?;
        codec::legs_presence(floor).map_err(|e|format!("support {a:?}: {e:?}"))?;
        let mut source=Self::new(a[1],floor.clone(),lo,hi);
        source.air_block=world::model(&world::BlockState {name:"minecraft:air".into(),properties:Vec::new()})?.clone();
        source.difficulty=world.difficulty();
        source.natural_regeneration=world.natural_regeneration();
        source.floor_state=world.state_id(a);
        if source.floor_state.is_some() {source.air_state=StateId::parse("minecraft:air",|_|None).ok();}
        for y in lo[1]..a[1] {
            let p=[a[0],y,a[2]];
            if let Ok(b)=world.known_block(p) {
                if codec::legs_presence(b).is_ok() {
                    source.layers.insert(y,b.clone());
                    if let Some(s)=world.state_id(p) {source.layer_states.insert(y,s);}
                }
            }
        }
        let origin=lo.map(|v|v>>4);
        let span=std::array::from_fn(|i|(((hi[i]>>4)-origin[i]+1).max(0)) as usize);
        source.scars.origin=origin;
        source.scars.span=span;
        source.scars.sections.resize_with(span.iter().product(),||None);
        let mut classified=HashMap::new();
        let mut missing=HashMap::new();
        let mut record=|lo:BlockPos,hi:BlockPos,value:Result<&'a Block,&'a str>,state:Option<StateId>| {
            let Ok(block)=value else {
                let why=value.unwrap_err();
                let row=*missing.entry(why).or_insert_with(|| {
                    source.scars.has_unknown=true; source.scars.rows.push(Row::Missing(why.to_owned()));
                    source.scars.rows.len() as u32
                });
                source.scars.paint(lo,hi,row,3^reach(source.base(lo)),3);
                return;
            };
            for y in lo[1]..=hi[1] {
                let p=[lo[0],y,lo[2]];
                let state_diff=state.filter(|s|Some(*s)!=source.base_state(p));
                let row=*classified.entry((y,block as *const Block,state_diff)).or_insert_with(|| {
                    let same=codec::legs_presence(source.base(p)).and_then(|base|
                        codec::legs_presence(block).map(|actual|actual==base));
                    let row=match same {
                        Ok(true) if state_diff.is_none()=>return 0,
                        Ok(true)=>Row::Known {model:None,state:state_diff},
                        Ok(false)=>{
                            source.scars.has_model_scars=true;
                            let model=state.and_then(StateId::model).filter(|m|std::ptr::eq(m.block_at(p),block));
                            let model=model.map_or_else(||Model::Custom(Box::new(block.clone())),Model::Canonical);
                            Row::Known {model:Some(model),state:state_diff}
                        }
                        Err(e)=>{source.scars.has_unknown=true; Row::Missing(format!("{e:?}"))},
                    };
                    source.scars.rows.push(row);
                    source.scars.rows.len() as u32
                });
                let (collision_diff, row_flags)=match source.scars.rows.get(row.wrapping_sub(1) as usize) {
                    Some(Row::Missing(_))=>(3^reach(source.base(p)), 3),
                    Some(Row::Known {model:Some(_),..})=>(reach(block)^reach(source.base(p)), block_flags(block)),
                    _=> (0, 0),
                };
                source.scars.paint(p,[hi[0],y,hi[2]],row,collision_diff,row_flags);
            }
        };
        fn visit<'a>(
            world:&'a impl World, lo:BlockPos,hi:BlockPos,shift:u32,
            uniform:&impl Fn(BlockPos,BlockPos)->Option<(&'a Block,Option<StateId>)>,
            record:&mut impl FnMut(BlockPos,BlockPos,Result<&'a Block,&'a str>,Option<StateId>),
        ) {
            if lo==hi {
                let block=world.known_block(lo);
                record(lo,hi,block,block.ok().and_then(|_|world.state_id(lo)));
            } else {
                match uniform(lo,hi) {
                    Some((block,state@Some(_)))=>record(lo,hi,Ok(block),state),
                    // A model proof alone cannot recover exact StateIds for kit verbs.
                    Some((block,None))=>{cells(lo,hi,|p|{record(p,p,Ok(block),world.state_id(p));true});}
                    None=>boxes(lo,hi,shift-2,|a,b|visit(world,a,b,shift-2,uniform,record)),
                }
            }
        }
        boxes(lo,hi,4,|a,b|visit(world,a,b,4,&uniform,&mut record));
        Ok(source)
    }
    fn base(&self,p:BlockPos)->&Block {if p[1]==self.floor_y {&self.floor_block} else {self.layers.get(&p[1]).unwrap_or(&self.air_block)}}
    fn base_state(&self,p:BlockPos)->Option<law::basis::cell::StateId> {if p[1]==self.floor_y {self.floor_state} else {self.layer_states.get(&p[1]).copied().or(self.air_state)}}
    #[inline]
    fn read(&self,p:BlockPos)->Result<&Block,&str> {
        let row=self.scars.row(p);
        if let Some(Row::Missing(reason))=row {return Err(reason);}
        if let Some(block)=self.scars.edits.get(&p) {return Ok(block);}
        match row {
            Some(Row::Known {model:Some(model),..})=>Ok(model.at(p)),
            _=>Ok(self.base(p)),
        }
    }
    fn blocks(&self)->impl Iterator<Item=&Block> {
        [&self.floor_block,&self.air_block].into_iter().chain(self.layers.values()).chain(self.scars.edits.values())
            .chain(self.scars.rows.iter().filter_map(|r|match r {Row::Known {model:Some(model),..}=>Some(model.at([0;3])),_=>None}))
    }
    fn covered(&self,p:BlockPos)->bool {(0..3).all(|a|p[a]>=self.loaded_min[a] && p[a]<=self.loaded_max[a])}
    /// A read the planner needs: the first one outside the source is kept, so the caller can grow it.
    fn reaches(&self,p:BlockPos)->bool {
        let inside=self.covered(p);
        if !inside && self.first_uncovered.get().is_none() { self.first_uncovered.set(Some(p)); }
        inside
    }
    fn flags_in(&self, lo: BlockPos, hi: BlockPos, mask: u8) -> bool {
        if (lo[1]..=hi[1]).contains(&self.floor_y) && (block_flags(&self.floor_block) & mask) != 0 {
            return true;
        }
        if self.layers.range(lo[1]..=hi[1]).any(|(_, b)| (block_flags(b) & mask) != 0) {
            return true;
        }
        if (block_flags(&self.air_block) & mask) != 0 {
            return true;
        }
        if self.scars.edits.iter().any(|(p, b)| (0..3).all(|a| lo[a] <= p[a] && p[a] <= hi[a]) && (block_flags(b) & mask) != 0) {
            return true;
        }
        if self.scars.sections.is_empty() {
            return false;
        }
        for sz in lo[2] >> 4..=hi[2] >> 4 {
            for sy in lo[1] >> 4..=hi[1] >> 4 {
                for sx in lo[0] >> 4..=hi[0] >> 4 {
                    if self.scars.section([sx << 4, sy << 4, sz << 4]).is_some_and(|s| (s.flags & mask) != 0) {
                        return true;
                    }
                }
            }
        }
        false
    }
}
impl World for GeneratorSuperflatSource {
    #[inline]
    fn known_block(&self,p:BlockPos)->Result<&Block,&str> {
        if !self.reaches(p) {return Err("outside the source");}
        self.read(p)
    }
    fn block(&self,p:BlockPos)->&Block {self.read(p).unwrap_or_else(|_|self.base(p))}
    fn state_id(&self,p:BlockPos)->Option<StateId> {
        if !self.covered(p) {return None;}
        match self.scars.row(p) {
            Some(Row::Missing(_))=>None,
            Some(Row::Known {state:Some(state),..})=>Some(*state),
            _=>self.base_state(p),
        }
    }
    fn uniform(&self,lo:BlockPos,hi:BlockPos)->Option<&Block> {
        if !self.covered(lo) || !self.covered(hi) || (0..3).any(|i|lo[i]>hi[i]) {return None;}
        let first=self.read(lo).ok()?;
        if lo==hi {return Some(first);}
        if !self.scars.has_model_scars && !self.scars.has_unknown && self.scars.edits.is_empty() {
            return (lo[1]..=hi[1]).all(|y|std::ptr::eq(first,self.base([lo[0],y,lo[2]]))).then_some(first);
        }
        let mut same=true;
        boxes(lo,hi,2,|a,b| {
            if !same {return;}
            if self.scars.edits.range(a..=b).next().is_none() &&
                self.scars.section(a).is_none_or(|s|s.altered&cube(a)==0) {
                same=(a[1]..=b[1]).all(|y|std::ptr::eq(first,self.base([a[0],y,a[2]])));
            } else {
                cells(a,b,|p| {same=self.read(p).is_ok_and(|next|std::ptr::eq(first,next));same});
            }
        });
        same.then_some(first)
    }
    fn loaded(&self,p:BlockPos)->bool {
        self.covered(p) && !matches!(self.scars.row(p),Some(Row::Missing(_)))
    }
    fn difficulty(&self)->u8 {self.difficulty}
    fn natural_regeneration(&self)->bool {self.natural_regeneration}
    fn min_y(&self)->i32 {self.loaded_min[1]}
    fn collision_cell_bounds(&self,bounds:law::Aabb,lo:BlockPos,hi:BlockPos)->Option<(BlockPos,BlockPos)> {
        let mut result=None;
        self.visit_collision_cells(bounds,lo,hi,|p| {
            let (a,b)=result.get_or_insert((p,p));
            for i in 0..3 { a[i]=a[i].min(p[i]); b[i]=b[i].max(p[i]); }
            true
        });
        result
    }
    fn visit_collision_cells(&self,bounds:law::Aabb,lo:BlockPos,hi:BlockPos,mut visit:impl FnMut(BlockPos)->bool)->bool {
        if !self.covered(lo) || !self.covered(hi) || !self.scars.edits.is_empty() {
            return cells(lo,hi,|p| self.known_block(p).is_ok_and(Block::never_collides) || visit(p));
        }
        let near_lo=[bounds.min.x,bounds.min.y,bounds.min.z].map(law::math::floor);
        let near_hi=[bounds.max.x,bounds.max.y,bounds.max.z].map(law::math::floor);
        let (floor,air)=(reach(&self.floor_block),reach(&self.air_block));
        for z in lo[2]..=hi[2] {for y in lo[1]..=hi[1] {
            let base=if y==self.floor_y {floor} else {self.layers.get(&y).map_or(air,reach)};
            for sx in lo[0]>>4..=hi[0]>>4 {
                let x=sx<<4;
                let (a,b)=(x.max(lo[0])&15,(x|15).min(hi[0])&15);
                let row=local([x,y,z])>>4;
                let section=self.scars.section([x,y,z]);
                let mask=|i:usize| {
                    let full=if base & (1<<i)!=0 {u16::MAX} else {0};
                    full ^ section.map_or(0,|s|s.collision_xor[i][row])
                };
                let mut candidates=mask(1);
                let (start,end)=(near_lo[0].max(x),near_hi[0].min(x|15));
                if start<=end && (near_lo[1]..=near_hi[1]).contains(&y) && (near_lo[2]..=near_hi[2]).contains(&z) {
                    candidates |= mask(0) & x_mask(start&15,end&15);
                }
                candidates &= x_mask(a,b);
                while candidates!=0 {
                    if !visit([x|candidates.trailing_zeros() as i32,y,z]) {return false;}
                    candidates &= candidates-1;
                }
            }
        }}
        true
    }
    fn has_fluid(&self)->bool {self.blocks().any(|b|b.fluid.kind!=law::FluidKind::Empty)}
    fn has_inside_effects(&self)->bool {self.blocks().any(Block::has_inside_effects)}
    fn has_fluid_in(&self, lo: BlockPos, hi: BlockPos) -> bool { self.flags_in(lo, hi, 1) }
    fn has_inside_effects_in(&self, lo: BlockPos, hi: BlockPos) -> bool { self.flags_in(lo, hi, 2) }
}
impl Source for GeneratorSuperflatSource {
    fn cell(&self,_tick:u64,p:BlockPos,mask:u8)->Result<Canonical,Refusal> {
        let block=self.known_block(p).map_err(|_|Refusal::Coverage)?;
        codec::cell(block,self.loaded(p),mask).map_err(Refusal::Law)
    }
    fn environment(&self,_tick:u64,q:&EnvironmentRead)->Result<Canonical,Refusal> {codec::environment(self,q).map_err(Refusal::Law)}
}
impl CorridorSource for GeneratorSuperflatSource {
    fn indexed_collisions(&self)->bool {true}
    fn support_cell(&self,s:&PlayerState)->BlockPos {s.main_supporting_block.unwrap_or([s.position.x.floor() as i32,self.floor_y,s.position.z.floor() as i32])}
    fn floor_y(&self, _: &PlayerState) -> i32 { self.floor_y }
    fn provenance(&self) -> [u8; 32] { self.provenance_hash }
    fn bounds_box(&self) -> Option<(BlockPos, BlockPos)> { Some((self.loaded_min, self.loaded_max)) }
    fn entry_label(&self,codec:&mut Interner,state:&PlayerState)->Result<LabelId,Refusal> {
        let p=state.position;
        let feet=[p.x.floor() as i32,p.y.floor() as i32,p.z.floor() as i32];
        // The generator supplies the support surface even during the airborne
        // phase. The corridor query still qualifies its entire read footprint.
        let support=state.main_supporting_block.unwrap_or([feet[0],self.floor_y,feet[2]]);
        label_roles(codec,self,feet,support)
    }
    fn corridor_query(&self,codec:&mut Interner,expected:LabelId,min:BlockPos,max:BlockPos)->CorridorResult {
        if !self.reaches(min) || !self.reaches(max) {return CorridorResult::Unknown(Refusal::Loaded);}
        // Qualify the generator's role tuple, not its name. No cells along its length.
        let support=[min[0],self.floor_y,min[2]];
        // Read the immutable generator, separately from its sparse edit journal.
        struct Base<'a>(&'a GeneratorSuperflatSource);
        impl World for Base<'_> {fn block(&self,p:BlockPos)->&Block {self.0.base(p)}}
        // A zone in another mode's class (water far from A): every column answers like the first.
        match label_roles(codec,&Base(self),[support[0],self.floor_y+1,support[2]],support) {Ok(id) if id==expected=>(),Ok(_)=>return self.uniform_corridor(codec,expected,min,max),Err(r)=>return CorridorResult::Unknown(r)}
        // An unknown shape may extend beyond its own cell into the corridor.
        let halo_min=min.map(|v|v.saturating_sub(1)); let halo_max=max.map(|v|v.saturating_add(1));
        if !self.reaches(halo_min) || !self.reaches(halo_max) {return CorridorResult::Unknown(Refusal::Coverage);}
        let inside=|p:BlockPos,lo:BlockPos,hi:BlockPos|(0..3).all(|i|lo[i]<=p[i] && p[i]<=hi[i]);
        let mut unknown=None;
        let mut first=None;
        let mut exceptions=0;
        let mut refusal=None;
        let mut inspect=|p,b:Option<&Block>| {
            let Some(b)=b else {
                unknown=Some(unknown.map_or(p,|old:BlockPos|old.min(p)));
                exceptions+=1;
                return;
            };
            if !inside(p,min,max) {
                let lo=std::array::from_fn::<_,3,_>(|i|f64::from(min[i])-f64::from(p[i]));
                let hi=std::array::from_fn::<_,3,_>(|i|f64::from(max[i])+1.0-f64::from(p[i]));
                let bounds=law::Aabb::new(lo[0],lo[1],lo[2],hi[0],hi[1],hi[2]);
                if !b.collision_shapes().chain(self.base(p).collision_shapes()).any(|s|s.intersects(bounds)) {return;}
            }
            let changed=[LegsRole::Support02,LegsRole::Support05,LegsRole::BodyFeet,LegsRole::Inside].into_iter().try_fold(false,|found,r| {
                Ok::<_,law::pattern::PatternError>(found || codec::legs_bytes(b,true,r)?!=codec::legs_bytes(self.base(p),true,r)?)
            });
            match changed {
                Ok(true)=>{exceptions+=1;first=Some(first.map_or(p,|old:BlockPos|old.min(p)));},
                Ok(false)=>(),
                Err(e)=>refusal=Some(Refusal::Law(e)),
            }
        };
        if self.scars.has_model_scars || self.scars.has_unknown {
            boxes(halo_min,halo_max,4,|a,b| {
                let Some(section)=self.scars.section(a) else {return;};
                boxes(a,b,2,|a,b| {
                    if section.altered&cube(a)==0 {return;}
                    cells(a,b,|p| {
                        let row=section.cells[local(p)];
                        if let Some(r)=self.scars.rows.get(row.wrapping_sub(1) as usize) {
                            match r {
                                Row::Missing(_)=>inspect(p,None),
                                Row::Known {model:Some(m),..} if !self.scars.edits.contains_key(&p)=>inspect(p,Some(m.at(p))),
                                _=>(),
                            }
                        }
                        true
                    });
                });
            });
        }
        for (&p,b) in self.scars.edits.range(halo_min..=halo_max) {
            if inside(p,halo_min,halo_max) {inspect(p,Some(b));}
        }
        if let Some(r)=refusal {return CorridorResult::Unknown(r);}
        if let Some(first_pos)=unknown.or(first) {CorridorResult::Exception {first_pos,exceptions}}
        else {CorridorResult::Clear(CorridorProof {label:expected,min,max,version:self.provenance_hash})}
    }
}
impl GeneratorSuperflatSource {
    /// The worldtube read cell by cell: each column's label (support at `min[1]`, feet, wet eyes)
    /// is the expected one, the cells above its eyes answer like the first column's, and nothing
    /// in the halo reaches into the box.
    fn uniform_corridor(&self,codec:&mut Interner,expected:LabelId,min:BlockPos,max:BlockPos)->CorridorResult {
        let halo_min=min.map(|v|v.saturating_sub(1)); let halo_max=max.map(|v|v.saturating_add(1));
        if !self.reaches(halo_min) || !self.reaches(halo_max) {return CorridorResult::Unknown(Refusal::Coverage);}
        let sy=min[1];
        let above=|codec:&mut Interner,x:i32,z:i32|->Result<Vec<law::codec::ClassId>,Refusal> {
            (sy+3..=max[1]).map(|y| {
                let b=self.known_block([x,y,z]).map_err(|_|Refusal::Coverage)?;
                codec.legs(b,true,LegsRole::Inside).map_err(Refusal::Law)
            }).collect()
        };
        let reference=match above(codec,min[0],min[2]) {Ok(r)=>r,Err(r)=>return CorridorResult::Unknown(r)};
        let (mut first,mut exceptions)=(None,0usize);
        for z in min[2]..=max[2] {for x in min[0]..=max[0] {
            let same=match label_roles(codec,self,[x,sy+1,z],[x,sy,z]) {Ok(id)=>id==expected,Err(r)=>return CorridorResult::Unknown(r)};
            let same=same && match above(codec,x,z) {Ok(a)=>a==reference,Err(r)=>return CorridorResult::Unknown(r)};
            if !same {exceptions+=1;first.get_or_insert([x,sy,z]);}
        }}
        // The halo: a shape outside the box that reaches into it.
        for z in halo_min[2]..=halo_max[2] {for y in halo_min[1]..=halo_max[1] {for x in halo_min[0]..=halo_max[0] {
            let p=[x,y,z];
            if (0..3).all(|i|min[i]<=p[i] && p[i]<=max[i]) {continue;}
            let Ok(b)=self.known_block(p) else {exceptions+=1;first.get_or_insert(p);continue};
            let lo=std::array::from_fn::<_,3,_>(|i|f64::from(min[i])-f64::from(p[i]));
            let hi=std::array::from_fn::<_,3,_>(|i|f64::from(max[i])+1.0-f64::from(p[i]));
            let bounds=law::Aabb::new(lo[0],lo[1],lo[2],hi[0],hi[1],hi[2]);
            if b.collision_shapes().any(|s|s.intersects(bounds)) {exceptions+=1;first.get_or_insert(p);}
        }}}
        match first {
            Some(first_pos)=>CorridorResult::Exception {first_pos,exceptions},
            None=>CorridorResult::Clear(CorridorProof {label:expected,min,max,version:self.provenance_hash}),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Partial { floor: Block, air: Block, wall: bool }
    impl World for Partial {
        fn block(&self, p: BlockPos) -> &Block { if p[1] == 0 { &self.floor } else { &self.air } }
        fn difficulty(&self) -> u8 { 0 }
        fn known_block(&self, p: BlockPos) -> Result<&Block, &str> {
            let blocked = if self.wall { p[2] == 15 && p[1] >= 1 } else { p == [0, 1, 15] };
            if blocked { Err("unmodeled non-local stair shape") } else { Ok(self.block(p)) }
        }
    }
    #[test]
    fn unknown_shape_in_corridor_is_an_exception() {
        let floor = Block::full();
        let air = world::model(&world::BlockState { name: "minecraft:air".into(), properties: Vec::new() }).unwrap().clone();
        // Isolated unknown cell at [0, 1, 15] may be steered around safely (verified by game replay) or reported.
        let world_one = Partial { floor: floor.clone(), air: air.clone(), wall: false };
        let source_one = GeneratorSuperflatSource::from_world(&world_one, ([-8, -4, -8], [8, 12, 60]), [0, 0, 0]).unwrap();
        let mut start = crate::settle(&crate::Still(&source_one), [0.5, 1.0, 0.5], 0).unwrap();
        start.saturation = 20.0;
        let goal = crate::Goal::of([0, 0, 40]);
        let result = crate::solve(&start, goal, crate::Kit::BareLegs, &source_one, &crate::store::GeneratorStore::default(), &mut Interner::default(), u64::MAX);
        match &result.outcome {
            crate::PlanOutcome::NeedsJoints { exception: [0, 1, 15], .. } => {}
            crate::PlanOutcome::Ready(plan) => {
                let programs = words::tiny_programs();
                let obs = ObservedWorld::new(&world_one);
                let mut sim = start.clone();
                let mut hit = false;
                for step in &plan.words {
                    for elapsed in 0..step.ticks {
                        let (_, inp) = programs[step.program as usize].select(&obs, &sim, elapsed, step.args).expect("row matches");
                        let before = sim.bounding_box();
                        law::try_tick(&mut sim, &obs, &inp).expect("tick fails");
                        obs.check().expect("plan touched unknown cell [0, 1, 15]");
                        if goal.touched_boxes(before, sim.bounding_box()) {
                            hit = true;
                        }
                    }
                }
                assert!(hit || goal.touched(&sim, &sim));
            }
            other => panic!("unexpected outcome on isolated unknown block: {other:?}"),
        }
        // Full unknown wall at z = 15 leaves no known bypass and must surface the z = 15 exception.
        let world_wall = Partial { floor, air, wall: true };
        let source_wall = GeneratorSuperflatSource::from_world(&world_wall, ([-8, -4, -8], [8, 12, 60]), [0, 0, 0]).unwrap();
        let res_wall = crate::solve(&start, goal, crate::Kit::BareLegs, &source_wall, &crate::store::GeneratorStore::default(), &mut Interner::default(), u64::MAX);
        assert!(matches!(res_wall.outcome, crate::PlanOutcome::NeedsJoints { exception: [_, _, 15], .. }), "{:?} {:?}", res_wall.outcome, res_wall.explanation);
    }
}
