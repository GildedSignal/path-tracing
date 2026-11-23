//! Exact lattice families. Learning bisects law events using the already
//! witnessed addition maps; prediction never invokes the law.
use law::{PlayerState,Input,World,Vec3,state_reads::{StateField as F,NON_MONOTONE_FIELDS}};
use crate::{joints::{ordered,unordered,EntryTemplate}};

#[derive(Clone,Debug)]
pub struct UseInterval {pub lo:[f64;4],pub hi:[f64;4]}
impl UseInterval {
    pub fn witness(start:&PlayerState,end:&PlayerState)->Self {let values=Self::values(start,end);Self {lo:values,hi:values}}
    fn values(start:&PlayerState,end:&PlayerState)->[f64;4] {
        [(end.food_level-start.food_level) as f64,(end.saturation-start.saturation) as f64,
         (end.exhaustion-start.exhaustion) as f64,end.fall_distance-start.fall_distance]
    }
    fn from(start:&PlayerState,ends:impl Iterator<Item=PlayerState>)->Self {
        let mut out=Self {lo:[f64::INFINITY;4],hi:[f64::NEG_INFINITY;4]};
        for end in ends {let v=Self::values(start,&end);for a in 0..4 {out.lo[a]=out.lo[a].min(v[a]);out.hi[a]=out.hi[a].max(v[a]);}}
        out
    }
    pub fn contains(&self,start:&PlayerState,end:&PlayerState)->bool {
        Self::values(start,end).into_iter().enumerate().all(|(a,v)|self.lo[a]<=v && v<=self.hi[a])
    }
}
#[derive(Clone,Debug)]
pub enum AxisOp {Add(f64),Clamp {plane:f64,wanted:f64},Hold}
#[derive(Clone,Debug)]
pub struct AxisMap {pub ops:Vec<AxisOp>}
impl AxisMap {
    pub fn advance(&self,mut x:f64)->f64 {
        for op in &self.ops {match *op {
            AxisOp::Add(d)=>x+=d,
            AxisOp::Hold=>(),
            AxisOp::Clamp {plane,wanted}=>{
                let b=law::Aabb::player(Vec3::new(x,0.0,0.0),law::Pose::Standing.height());
                let delta=if wanted>0.0 {plane-b.max.x} else {plane-b.min.x};
                x+=if wanted>0.0 {wanted.min(delta)} else {wanted.max(delta)};
            },
        }}x
    }
    fn prefix(&self,tick:usize)->Self {Self {ops:self.ops[..tick].to_vec()}}
}
#[derive(Clone,Debug)]
pub struct MappedExit {pub lo:u64,pub hi:u64,pub map:AxisMap,pub axis:usize,pub end:PlayerState}
#[derive(Clone,Debug)]
pub enum Image {Map(Vec<MappedExit>),Points(Vec<PlayerState>)}
#[derive(Clone,Debug)]
pub struct FamilyPiece {
    pub lo:u64,pub hi:u64,pub signature:Vec<u128>,pub outcomes:Vec<u8>,pub pin:Option<[PlayerState;2]>,
    pub pin_outcome:Option<law::basis::sweep::Outcome>,pub image:Image,pub uses:UseInterval,pub ticks:(u32,u32),
}
impl FamilyPiece {
    pub fn predict(&self,bits:u64)->Vec<PlayerState> {
        assert!(self.lo<=bits && bits<=self.hi);
        match &self.image {
            Image::Points(points)=>points.clone(),
            Image::Map(branches)=>branches.iter().filter(|b|b.lo<=bits && bits<=b.hi).map(|branch| {
                let mut out=branch.end.clone();let old=[out.position.x,out.position.y,out.position.z];let mut p=old;
                p[branch.axis]=branch.map.advance(unordered(bits));out.position=Vec3::new(p[0],p[1],p[2]);
                if let Some(mut support)=out.main_supporting_block {support[branch.axis]+=p[branch.axis].floor() as i32-old[branch.axis].floor() as i32;out.main_supporting_block=Some(support);}
                out
            }).collect(),
        }
    }
}
#[derive(Clone,Debug)]
pub struct Family {pub entry:EntryTemplate,pub pieces:Vec<FamilyPiece>,pub law_hash:[u8;32],pub law_ticks:u64,pub map_probes:u64}
struct Replay {states:Vec<PlayerState>,reads:Vec<law::footprint::Footprint>,map:AxisMap,signature:Vec<u128>,outcomes:Vec<u8>,contacts:Vec<law::basis::sweep::AxisContact>,pin:Option<usize>}
fn mask()->u128 {NON_MONOTONE_FIELDS.iter().fold(0,|m,(f,_)|m|1u128<<*f as u8)}
fn event(before:&PlayerState,after:&PlayerState)->u128 {words::segment::joint_events(before,after).0 & !mask()}
fn coordinate(s:&PlayerState,axis:usize)->f64 {[s.position.x,s.position.y,s.position.z][axis]}
fn letter(s:&PlayerState,c:law::basis::sweep::AxisContact)->u8 {if matches!(c.outcome,law::basis::sweep::Outcome::Hold {..}) {2} else if s.horizontal_collision {1} else {0}}
fn operation(after:&PlayerState,c:law::basis::sweep::AxisContact,applied:f64)->AxisOp {
    // move_player resets this field, then writes it exactly when the native
    // vector addition runs. An unchanged rounded endpoint is not identity:
    // the neighbouring parity may move by two ulps under the same operand.
    if after.movement_this_tick.is_none() {return AxisOp::Hold;}
    match c.outcome {
        law::basis::sweep::Outcome::Free=>AxisOp::Add(applied),
        law::basis::sweep::Outcome::Clamp {plane}=>AxisOp::Clamp {plane,wanted:c.wanted},
        law::basis::sweep::Outcome::Hold {..}=>AxisOp::Add(applied),
    }
}
fn replay(w:&impl World,start:&PlayerState,axis:usize,x:f64,input:&Input,stop:&impl Fn(&PlayerState,u32)->bool,budget:u32)->Result<Replay,String> {
    let mut s=start.clone();match axis {0=>s.position.x=x,2=>s.position.z=x,_=>return Err("only horizontal free axes".into())};
    let mut states=vec![s.clone()];let mut reads=Vec::new();let recording=law::footprint::RecordingWorld::new(w);let mut ops=Vec::new();let mut signature=Vec::new();let mut outcomes=Vec::new();let mut contacts=Vec::new();let mut pin=None;
    for tick in 1..=budget {
        let before=s.clone();let Some((_,moved,passes))=law::basis::sweep::tick(&mut s,&recording,input) else {return Err("no collision-law addition map".into());};reads.push(recording.take_world());
        let contact=*passes.iter().find(|p|p.axis as usize==axis).ok_or("missing axis contact report")?;
        ops.push(operation(&s,contact,moved.get(contact.axis)));contacts.push(contact);
        signature.push(event(&before,&s));outcomes.push(letter(&s,contact));
        if s.horizontal_collision {pin=Some(tick as usize);}
        states.push(s.clone());
        if pin.is_some() || stop(&s,tick) {return Ok(Replay {states,reads,map:AxisMap {ops},signature,outcomes,contacts,pin});}
    }
    Err("word budget exhausted before exit or pin".into())
}
fn pinned_key(s:&PlayerState)->Vec<u64> {pinned_key_axis(s,None)}
fn pinned_key_axis(s:&PlayerState,free:Option<usize>)->Vec<u64> {
    let mut key=Vec::new();
    for &f in F::ALL {
        if matches!(f,F::food_level|F::exhaustion|F::saturation|F::fall_distance) || mask()&(1u128<<f as u8)!=0 {continue;}
        if f==F::main_supporting_block {
            key.push(s.main_supporting_block.is_some() as u64);
            if let Some(p)=s.main_supporting_block {let body=[s.position.x.floor() as i32,s.position.y.floor() as i32,s.position.z.floor() as i32];key.extend((0..3).map(|a|(p[a]-body[a]) as u64));}
        } else if f==F::position {
            for a in 0..3 {if Some(a)!=free {key.push(coordinate(s,a).to_bits());}}
        } else {s.push_field_bits(f,&mut key);}
    }key
}
impl Family {
    pub fn learn(w:&impl World,start:&PlayerState,axis:usize,domain:(f64,f64),input:Input,budget:u32,stop:impl Fn(&PlayerState,u32)->bool)->Result<Self,String> {
        if !domain.0.is_finite() || !domain.1.is_finite() || domain.0>domain.1 {return Err("invalid family lattice interval".into());}
        let counted=crate::source::MeteredWorld::new(w);
        let mut result=Self {entry:EntryTemplate::with_axis(start,start.position,0,Some(axis)),pieces:Vec::new(),law_hash:law::LAW_HASH,law_ticks:0,map_probes:0};
        let mut pending=vec![(ordered(domain.0),ordered(domain.1),false)];
        while let Some((lo,hi,partitioned))=pending.pop() {
            let left=replay(&counted,start,axis,unordered(lo),&input,&stop,budget)?;
            let right=replay(&counted,start,axis,unordered(hi),&input,&stop,budget)?;
            // A face can enter and leave between equal endpoint signatures.
            // Inventory the endpoint hull and pull back EVERY spatial
            // comparison, including the law's collision offsets. This is why
            // endpoint-signature bisection alone is insufficient.
            if !partitioned {
                let mut cuts=vec![lo,hi+1];
                
                for tick in 0..left.reads.len().min(right.reads.len()) {
                    let a=&left.states[tick];
                    // The law supplies the query halo. A positional margin
                    // would miss long sweeps or bake a body size into search.
                    let positions=||left.reads[tick].reads.keys().chain(right.reads[tick].reads.keys());
                    if positions().next().is_none() {continue;}
                    let low=std::array::from_fn::<_,3,_>(|i|positions().map(|p|p[i]).min().unwrap());
                    let high=std::array::from_fn::<_,3,_>(|i|positions().map(|p|p[i]).max().unwrap());
                    let mut planes=Vec::new();
                    for x in low[0]..=high[0] {for y in low[1]..=high[1] {for z in low[2]..=high[2] {
                        let p=[x,y,z];let mut q=p;q[axis]+=1;
                        if law::codec::legs_presence(w.block(p)).map_err(|e|format!("{e:?}"))?.bytes()!=law::codec::legs_presence(w.block(q)).map_err(|e|format!("{e:?}"))?.bytes() {planes.push((p[axis]+1) as f64);}
                        let shape=&w.block(p).shape;
                        for &v in &shape.coords[axis] {if v!=0.0 && v!=1.0 {planes.push(p[axis] as f64+v);}}
                    }}}
                    planes.sort_by(f64::total_cmp);planes.dedup_by(|a,b|a.to_bits()==b.to_bits());
                    let map=left.map.prefix(tick);
                    let compare=|bits:u64,plane:f64| {
                        let x=map.advance(unordered(bits));let mut pos=a.position;
                        if axis==0 {pos.x=x;} else {pos.z=x;}
                        law::face_comparisons(law::Aabb::player(pos,a.pose.height()),if axis==0 {law::Axis::X} else {law::Axis::Z},plane)
                    };
                    for plane in planes {let av=compare(lo,plane);let bv=compare(hi,plane);
                        for field in 0..av.len() {if av[field]!=bv[field] {
                            let (mut l,mut h)=(lo,hi);
                            while l<h {let m=l+(h-l)/2;result.map_probes+=1;if compare(m,plane)[field]==av[field] {l=m+1;} else {h=m;}}
                            if l>lo && l<=hi {cuts.push(l);}
                        }}
                    }
                    
                }
                cuts.sort_unstable();cuts.dedup();
                if cuts.len()>2 {
                    for pair in cuts.windows(2).rev() {pending.push((pair[0],pair[1]-1,true));}
                    continue;
                }
            }
            if left.signature!=right.signature || left.outcomes!=right.outcomes {
                // The first differing event is a monotone face predicate on
                // the fixed prefix. Binary64 addition maps evaluate the prefix
                // exactly; there is no grid and no repeated native prefix.
                let tick=left.signature.iter().zip(&right.signature).zip(left.outcomes.iter().zip(&right.outcomes)).position(|((a,b),(c,d))|a!=b || c!=d)
                    .unwrap_or(left.signature.len().min(right.signature.len()));
                let tick=tick.min(left.map.ops.len()-1).min(right.map.ops.len()-1);
                let prefix=left.map.prefix(tick);
                let expected=event(&left.states[tick],&left.states[tick+1]);
                let expected_pin=left.states[tick+1].horizontal_collision;
                let expected_stop=stop(&left.states[tick+1],tick as u32+1);
                let predicate=|bits:u64| {
                    let mut state=left.states[tick].clone();let x=prefix.advance(unordered(bits));
                    let old=if axis==0 {state.position.x} else {state.position.z};
                    if axis==0 {state.position.x=x;} else {state.position.z=x;}
                    if let Some(mut support)=state.main_supporting_block {support[axis]+=x.floor() as i32-old.floor() as i32;state.main_supporting_block=Some(support);}
                    let before=state.clone();let (_,_,passes)=law::basis::sweep::tick(&mut state,&counted,&input).ok_or("missing contact tick")?;let c=*passes.iter().find(|p|p.axis as usize==axis).ok_or("missing axis")?;
                    Ok::<_,String>(event(&before,&state)==expected && letter(&state,c)==left.outcomes[tick] && state.horizontal_collision==expected_pin && stop(&state,tick as u32+1)==expected_stop)
                };
                if !predicate(lo)? || predicate(hi)? {return Err("first event is not a monotone pullback on the witnessed prefix".into());}
                let (mut a,mut b)=(lo,hi);
                while a+1<b {let m=a+(b-a)/2;result.map_probes+=1;if predicate(m)? {a=m;} else {b=m;}}
                pending.push((b,hi,false));pending.push((lo,b-1,false));continue;
            }
            let a=left.states.last().unwrap();let b=right.states.last().unwrap();
            let pin_outcome=left.pin.map(|_|left.contacts.last().unwrap().outcome);
            let hold=matches!(pin_outcome,Some(law::basis::sweep::Outcome::Hold {..}));
            let (pin,image,ends,ticks)=if let Some(tick)=left.pin {
                if right.pin!=Some(tick) || pinned_key_axis(a,hold.then_some(axis))!=pinned_key_axis(b,hold.then_some(axis)) {return Err("pin retains undeclared state information after contact cut".into());}
                for (r,pinned) in [(&left,a),(&right,b)] {
                    if hold {
                        if coordinate(pinned,axis).to_bits()!=coordinate(&r.states[tick-1],axis).to_bits() {return Err("hold is not identity".into());}
                    } else if let law::basis::sweep::Outcome::Clamp {plane}=r.contacts.last().unwrap().outcome {
                        let body=law::Aabb::player(Vec3::default(),pinned.pose.height());let radius=body.max.x;
                        let wanted=r.contacts.last().unwrap().wanted;
                        let expected=if wanted>0.0 {plane-radius} else {plane+radius};
                        if coordinate(pinned,axis).to_bits()!=expected.to_bits() {return Err("clamp did not write its exact face coordinate".into());}
                    } else {return Err("pin has no clamp or hold".into());}
                }
                let mut points=Vec::new();let mut branches=Vec::new();let mut ends=Vec::new();let mut times=Vec::new();
                for flag in 0..1usize<<NON_MONOTONE_FIELDS.len() {
                    for (r,bits) in [(&left,lo),(&right,hi)] {
                        let mut state=r.states.last().unwrap().clone();let mut map=r.map.clone();
                        for (i,(field,_)) in NON_MONOTONE_FIELDS.iter().enumerate() {match field {
                            F::minor_horizontal_collision=>state.minor_horizontal_collision=flag&(1<<i)!=0,
                            _=>return Err("non-monotone field has no outcome producer".into()),
                        }}
                        let mut elapsed=tick as u32;
                        while !stop(&state,elapsed) {
                            if elapsed==budget {return Err("pinned tail budget exhausted".into());}
                            let (_,moved,passes)=law::basis::sweep::tick(&mut state,&counted,&input).ok_or("no tail collision update")?;
                            let c=*passes.iter().find(|p|p.axis as usize==axis).ok_or("no tail axis")?;
                            map.ops.push(operation(&state,c,moved.get(c.axis)));elapsed+=1;
                        }
                        times.push(elapsed);ends.push(state.clone());
                        if hold {
                            // Both endpoint tails must have the same carried
                            // state class. Their exact coordinate remains data
                            // in the map; flags are the declared alternatives.
                            if bits==lo {branches.push(MappedExit {lo,hi,map,axis,end:state});}
                        } else if !points.iter().any(|p:&PlayerState|pinned_key(p)==pinned_key(&state) && p.minor_horizontal_collision==state.minor_horizontal_collision) {points.push(state);}
                    }
                }
                let min=*times.iter().min().unwrap();let max=*times.iter().max().unwrap();
                (Some([a.clone(),b.clone()]),if hold {Image::Map(branches)} else {Image::Points(points)},ends,(min,max))
            } else {
                if left.map.ops.len()!=right.map.ops.len() {return Err("different word durations inside one piece".into());}
                (None,Image::Map(vec![MappedExit {lo,hi,map:left.map,axis,end:a.clone()}]),vec![a.clone(),b.clone()],((a.t-start.t) as u32,(b.t-start.t) as u32))
            };
            let uses=UseInterval::from(start,ends.into_iter());
            result.pieces.push(FamilyPiece {lo,hi,signature:left.signature,outcomes:left.outcomes,pin,pin_outcome,image,uses,ticks});
        }
        let mut merged:Vec<FamilyPiece>=Vec::new();
        for p in result.pieces.drain(..) {
            if let Some(last)=merged.last_mut() {
                if last.hi+1==p.lo && last.signature==p.signature && last.outcomes==p.outcomes {
                    last.hi=p.hi;
                    for i in 0..4 {last.uses.lo[i]=last.uses.lo[i].min(p.uses.lo[i]);last.uses.hi[i]=last.uses.hi[i].max(p.uses.hi[i]);}
                    if let (Some(a),Some(b))=(&mut last.pin,p.pin) {a[1]=b[1].clone();}
                    match (&mut last.image,p.image) {
                        (Image::Map(a),Image::Map(b))=>a.extend(b),
                        (Image::Points(a),Image::Points(b))=>for point in b {if !a.iter().any(|p|pinned_key(p)==pinned_key(&point) && p.minor_horizontal_collision==point.minor_horizontal_collision) {a.push(point);}},
                        _=>return Err("same contact word has incompatible map and point images".into()),
                    }
                    continue;
                }
            }merged.push(p);
        }
        result.pieces=merged;result.law_ticks=counted.ticks();Ok(result)
    }
}
