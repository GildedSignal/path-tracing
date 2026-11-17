//! Face candidates are inventory, not standability. Only the law can put an
//! arriving state in their grounded takeoff domain. Edits repair local records.
use std::collections::HashMap;
use law::{Aabb,BlockPos,Vec3,World,codec::{Interner,JointPatternId,proof::{Binding,ReadCertificates,Source,Refusal}},footprint::RecordingWorld};

#[derive(Clone,Debug)]
pub struct Face {pub cell:BlockPos,pub bounds:Aabb,pub climbable:bool}
#[derive(Default)]
pub struct FaceInventory {pub faces:HashMap<BlockPos,Vec<Face>>,certificates:ReadCertificates,owners:HashMap<usize,BlockPos>,next_id:u32,pub created:Vec<Face>,pub reads:u64}
impl FaceInventory {
    fn capture(&mut self,w:&(impl World+Source),codec:&mut Interner,p:BlockPos)->Result<(),Refusal> {
        let recording=RecordingWorld::new(w);let block=recording.block(p);recording.loaded(p);
        let above=recording.block([p[0],p[1]+1,p[2]]);recording.loaded([p[0],p[1]+1,p[2]]);
        let mut faces=Vec::new();let s=&block.shape;let offset=Vec3::new(p[0] as f64,p[1] as f64,p[2] as f64);
        let sizes=s.coords.each_ref().map(|v|v.len().saturating_sub(1));
        for x in 0..sizes[0] {for y in 0..sizes[1] {for z in 0..sizes[2] {
            if s.is_full([x,y,z]) && !s.is_full([x,y+1,z]) {
                let top=s.coords[1][y+1];
                // A full neighbouring cube seals this whole face. Partial
                // shapes remain candidates; no centre-only clearance rule.
                if top==1.0 && above.shape.is_block() {continue;}
                faces.push(Face {cell:p,bounds:Aabb::new(s.coords[0][x],top,s.coords[2][z],s.coords[0][x+1],top,s.coords[2][z+1]).moved(offset),climbable:false});
            }
        }}}
        if block.climbable {faces.push(Face {cell:p,bounds:Aabb::new(p[0] as f64,p[1] as f64,p[2] as f64,p[0] as f64+1.0,p[1] as f64+1.0,p[2] as f64+1.0),climbable:true});}
        let fp=recording.take_world();self.reads+=fp.reads.len() as u64;
        let context=JointPatternId(self.next_id);self.next_id=self.next_id.checked_add(1).ok_or(Refusal::Schema)?;
        let id=self.certificates.capture(codec,Binding::plateau(context,0,law::LAW_HASH),&[(0,&fp)],w)?;
        self.owners.insert(id,p);self.created.extend(faces.iter().cloned());self.faces.insert(p,faces);Ok(())
    }
    pub fn build(w:&(impl World+Source),codec:&mut Interner,lo:BlockPos,hi:BlockPos)->Result<Self,Refusal> {
        let mut result=Self::default();for x in lo[0]..=hi[0] {for y in lo[1]..=hi[1] {for z in lo[2]..=hi[2] {result.capture(w,codec,[x,y,z])?;}}}result.created.clear();Ok(result)
    }
    pub fn edit(&mut self,w:&(impl World+Source),codec:&mut Interner,p:BlockPos)->Result<usize,Refusal> {
        let ids=self.certificates.edit(codec,p,(0,u64::MAX),w);let mut cells:Vec<_>=ids.iter().filter_map(|id|self.owners.remove(id)).collect();cells.sort();cells.dedup();
        for &cell in &cells {self.capture(w,codec,cell)?;}
        Ok(cells.len())
    }
    pub fn iter(&self)->impl Iterator<Item=&Face> {self.faces.values().flatten()}
}
