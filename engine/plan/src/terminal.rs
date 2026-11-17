//! Goal-face preimages, shared by floor label, gait phase and floating chart.
//! These are entry families with constant T and next word. A query reads their
//! intervals; it never searches controls or bisects an inverse.
use std::sync::Arc;
use law::codec::LabelId;
use crate::{Goal,store::WordKernel,state_view::{StateSource,View}};
#[derive(Clone,Copy,Debug,PartialEq,Eq,Hash)]
pub struct Chart {pub exponent:u64,pub edge:i32,pub interior:bool}
fn chart(cell:i32,pad:f64)->(Chart,i32) {
    let lo=cell as f64-pad;let hi=cell as f64+1.0+pad;
    let tag=|x:f64|x.to_bits()>>52;
    if tag(lo)==tag(hi) && lo!=0.0 && hi!=0.0 {
        let e=((((cell as f64).to_bits()>>52)&2047) as i32-1023).max(0);
        let middle=(1.5*2.0f64.powi(e))*if cell<0 {-1.0} else {1.0};
        (Chart {exponent:tag(lo),edge:0,interior:true},middle as i32)
    } else {(Chart {exponent:tag(cell as f64),edge:cell,interior:false},cell)}
}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Hash)]
pub struct Key {pub label:u32,pub gait:u64,pub phase:u32,pub charts:[Chart;2],pub y:u64,pub target_y:i32}
#[derive(Clone,Debug)]
pub struct Row {pub intervals:[(i128,i128);2],pub ticks:u32,pub word:usize}
#[derive(Clone,Debug)]
pub struct Set {pub key:Key,pub words:Vec<Arc<WordKernel>>,pub rows:Vec<Row>,pub canonical:[i32;2],pub domain:[(u64,u64);2],pub law_hash:[u8;32]}
pub struct Request {pub key:Key,pub canonical:Goal,pub shift:[i128;2],pub pad:f64}
impl Request {
    pub fn new(label:LabelId,gait:&WordKernel,phase:u32,words:&[Arc<WordKernel>],from:&impl StateSource,goal:Goal)->Self {
        let v=from.view();let pad=gait.cycle_period as f64*gait.max_body_step+v.bounding_box().max.x-v.position.x;
        let future=words.iter().map(|k|k.max_body_step*(k.transient_ticks+k.cycle_period) as f64).fold(0.0f64,f64::max);
        let xs=chart(goal.cell[0],pad+future);let zs=chart(goal.cell[2],pad+future);
        let canonical=Goal {cell:[xs.1,goal.cell[1],zs.1]};
        Self {key:Key {label:label.index(),gait:gait.id,phase,charts:[xs.0,zs.0],y:v.position.y.to_bits(),target_y:goal.cell[1]},canonical,
            shift:[crate::joints::ordered(goal.cell[0] as f64) as i128-crate::joints::ordered(xs.1 as f64) as i128,crate::joints::ordered(goal.cell[2] as f64) as i128-crate::joints::ordered(zs.1 as f64) as i128],pad}
    }
}
fn replace(mut view:View<'_>,axis:usize,x:f64)->View<'_> {if axis==0 {view.position.x=x;}else {view.position.z=x;}view}
fn axis_preimage(k:&Arc<WordKernel>,v:View<'_>,goal:Goal,t:u64,axis:usize,domain:(u64,u64))->Option<(u64,u64)> {
    let sides=|bits| {
        let at=replace(v,axis,crate::joints::unordered(bits));
        let before=k.readout(&at,t-1,false);let after=k.readout(&at,t,false);
        let b=before.bounding_box();let a=after.bounding_box();goal.axis_sides(b,a,axis)
    };
    let (mut lo,mut hi)=domain;
    if !sides(hi).0 || !sides(lo).1 {return None;}
    while lo<hi {let m=lo+(hi-lo)/2;if sides(m).0 {hi=m;}else {lo=m+1;}}
    let first=lo;hi=domain.1;
    while lo<hi {let m=lo+(hi-lo).div_ceil(2);if sides(m).1 {lo=m;}else {hi=m-1;}}
    sides(first).1.then_some((first,lo))
}
impl Set {
    pub fn build(request:&Request,words:&[Arc<WordKernel>],from:&impl StateSource)->Result<Self,String> {
        let goal=request.canonical;let mut v=from.view();v.position.x=goal.center().x;v.position.z=goal.center().z;
        let domain=[0usize,2].map(|a| {
            let min=if a==0 {goal.cell[0]}else {goal.cell[2]} as f64;
            (crate::joints::ordered(min-request.pad),crate::joints::ordered(min+1.0+request.pad))
        });
        let mut rows=Vec::new();
        for (word,k) in words.iter().enumerate() {
            let a=replace(replace(v,0,crate::joints::unordered(domain[0].0)),2,crate::joints::unordered(domain[1].0));
            let b=replace(replace(v,0,crate::joints::unordered(domain[0].1)),2,crate::joints::unordered(domain[1].1));
            let horizon=crate::events::horizon(&a,goal,k.transient_increments.iter().chain(&k.cycle_increments).copied(),k.cycle_displacement)
                .max(crate::events::horizon(&b,goal,k.transient_increments.iter().chain(&k.cycle_increments).copied(),k.cycle_displacement));
            if horizon==words::Arguments::MAX_TICKS {return Err("terminal face has no finite no-return proof".into());}
            for t in 1..=horizon {
                let before=k.readout(&v,t-1,false);let after=k.readout(&v,t,false);
                let vertical=goal.axis_sides(before.bounding_box(),after.bounding_box(),1);if !vertical.0 || !vertical.1 {continue;}
                let Some(x)=axis_preimage(k,v,goal,t,0,domain[0]) else {continue;};
                let Some(z)=axis_preimage(k,v,goal,t,2,domain[1]) else {continue;};
                rows.push(Row {intervals:[(x.0 as i128-crate::joints::ordered(goal.cell[0] as f64) as i128,x.1 as i128-crate::joints::ordered(goal.cell[0] as f64) as i128),
                    (z.0 as i128-crate::joints::ordered(goal.cell[2] as f64) as i128,z.1 as i128-crate::joints::ordered(goal.cell[2] as f64) as i128)],ticks:t.try_into().map_err(|_|"terminal tick width")?,word});
            }
        }
        rows.sort_by_key(|r|(r.ticks,r.word));
        Ok(Self {key:request.key,words:words.to_vec(),rows,canonical:[goal.cell[0],goal.cell[2]],domain,law_hash:law::LAW_HASH})
    }
    pub fn lookup(&self,from:&impl StateSource,goal:Goal)->Option<(Arc<WordKernel>,u64)> {
        if self.law_hash!=law::LAW_HASH {return None;}
        let p=from.view().position;
        let entry=[crate::joints::ordered(p.x) as i128-crate::joints::ordered(goal.cell[0] as f64) as i128,
                   crate::joints::ordered(p.z) as i128-crate::joints::ordered(goal.cell[2] as f64) as i128];
        self.rows.iter().find(|r|(0..2).all(|a|r.intervals[a].0<=entry[a] && entry[a]<=r.intervals[a].1)).map(|r|(self.words[r.word].clone(),r.ticks as u64))
    }
}
