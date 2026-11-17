//! Certificate publication and the same backward min-plus recurrence over
//! (family,piece) intervals and exact points. Unknown branches stay open.
use super::*;
use crate::families::{Family,Image};
#[derive(Clone,Debug)]
pub struct FamilyInterval {pub pattern:usize,pub piece:usize,pub lo:u64,pub hi:u64,pub lower:u64,pub upper:Option<u64>,pub via:EdgeId,pub next:Option<EdgeId>}
#[derive(Clone,Debug)]
pub enum ExitImage {Points(Vec<usize>),Map(std::sync::Arc<crate::families::FamilyPiece>)}
pub struct Consumption<'a> {pub program:u16,pub args:Arguments,pub ticks:(u32,u32),pub exit:&'a ExitImage,pub coordinate:u64}
impl Consumption<'_> {
    /// A set readout, never an arbitrary choice of a non-monotone flag.
    pub fn mapped_exits(&self)->Option<Vec<PlayerState>> {match self.exit {ExitImage::Map(image)=>Some(image.predict(self.coordinate)),_=>None}}
}
impl JointStore {
    pub fn learn_family<W:World+Source>(&mut self,world:&W,codec:&mut Interner,start:&PlayerState,axis:usize,domain:(f64,f64),phase:u32,program:u16,args:Arguments,stop:impl Fn(&PlayerState,u32)->bool)->Result<Vec<EdgeId>,Status> {
        let toolkit=words::tiny_programs();let canonical=toolkit.get(program as usize).ok_or_else(||Status::Refused {reason:"program ID outside toolkit".into()})?;
        let input=canonical.held_input(args).ok_or_else(||Status::Refused {reason:"family word requires a held-input producer".into()})?;
        let family=Family::learn(world,start,axis,domain,input,args.max_ticks,&stop).map_err(|reason|Status::Refused {reason})?;
        self.work.law_ticks+=family.law_ticks;
        let entry=EntryTemplate::with_axis(start,start.position,phase,Some(axis));
        let entry_id=if let Some(&id)=self.templates.get(&entry.key) {id}else {let bytes=codec::state_key(codec::StateQuestion::Exact,&entry.key);let id=codec.intern_state(&bytes).ok_or_else(||Status::Refused {reason:"state arena full".into()})?;self.template_bytes+=bytes.len();self.templates.insert(entry.key.clone(),id);id};
        let index=self.patterns.len();let pattern=JointPatternId(index.try_into().map_err(|_|Status::Refused {reason:"pattern arena full".into()})?);
        let label=crate::source::label(codec,world,start).map_err(|r|Status::Refused {reason:format!("entry label {r:?}")})?;
        let mut pieces=Vec::new();let mut ids=Vec::new();
        for (piece_index,piece) in family.pieces.into_iter().enumerate() {
            let mut timed=std::collections::BTreeMap::<u64,Footprint>::new();
            for bits in [piece.lo,piece.hi] {for flag in 0..1usize<<law::state_reads::NON_MONOTONE_FIELDS.len() {
                let mut state=entry.state(unordered(bits)).ok_or_else(||Status::Refused {reason:"family left its entry chart".into()})?;
                let counted=crate::source::MeteredWorld::new(world);let recording=RecordingWorld::new(&counted);let mut pinned=false;
                for tick in 1..=args.max_ticks {
                    let before=state.clone();law::try_tick(&mut state,&recording,&input).map_err(|e|Status::Refused {reason:e.0})?;
                    timed.entry(before.t).or_default().extend(&recording.take_world());
                    if !pinned && state.horizontal_collision {state.minor_horizontal_collision=flag&1!=0;pinned=true;}
                    if stop(&state,tick) {break;}
                    if tick==args.max_ticks {return Err(Status::Searched {work:tick as u64});}
                }
                self.work.law_ticks+=counted.ticks();
            }}
            // H2: include the interior of the endpoint read hull, not only its
            // two borders. The native query's own halo determines that hull.
            for fp in timed.values_mut() {
                if !fp.reads.is_empty() {
                    let lo:[i32;3]=std::array::from_fn(|a|fp.reads.keys().map(|p|p[a]).min().unwrap());
                    let hi:[i32;3]=std::array::from_fn(|a|fp.reads.keys().map(|p|p[a]).max().unwrap());
                    let masks=fp.reads.values().fold(0,|m,q|m|q);
                    for x in lo[0]..=hi[0] {for y in lo[1]..=hi[1] {for z in lo[2]..=hi[2] {*fp.reads.entry([x,y,z]).or_default()|=masks;}}}
                }
            }
            let binding=Binding::plateau(pattern,piece_index as u32,law::LAW_HASH);let reads:Vec<_>=timed.iter().map(|(t,f)|(*t,f)).collect();
            let certificate=self.certificates.capture(codec,binding.clone(),&reads,world).map_err(|e|Status::Refused {reason:format!("family certificate {e:?}")})?;
            let initial=entry.state(unordered(piece.lo)).unwrap();let from=self.node(&initial);
            let image=std::sync::Arc::new(piece);let predicted=image.predict(image.lo);let end=&predicted[0];let to=self.node(end);
            let mut points=Vec::new();if let Image::Points(p)=&image.image {for state in p {points.push(self.node(state));}}
            let exit=if points.is_empty() {ExitImage::Map(image.clone())}else {ExitImage::Points(points)};
            let cells=self.certificates.cells(certificate).unwrap();let min=std::array::from_fn(|a|cells.iter().map(|c|c.position[a]).min().unwrap());let max=std::array::from_fn(|a|cells.iter().map(|c|c.position[a]).max().unwrap());
            let step=WordStep {program,args,ticks:(end.t-start.t).try_into().unwrap(),expected_end_t:end.t,events:List::new(),certificate:crate::source::CorridorProof {label,min,max,version:law::LAW_HASH},end:StateAt::shared(self.states[to].clone()),map_segments:0};
            let positions=xyz(end.position).map(f64::to_bits);
            let item=Piece {lo:image.lo,hi:image.hi,status:Status::Found,ticks:step.ticks,tick_range:image.ticks,exit_lo:positions,exit_hi:xyz(image.predict(image.hi)[0].position).map(f64::to_bits),witness:Witness {s_bits:unordered(image.lo).to_bits(),end_digest:codec::digest(&end.bits().iter().flat_map(|w|w.to_le_bytes()).collect::<Vec<_>>())},signature:image.signature.clone(),grounded:start.on_ground,uses:image.uses.clone(),resource_guard:entry.resources,exit,binding,certificate,from,to,step,last_before:end.clone()};
            let id=EdgeId(self.edges.len()+pieces.len());ids.push(id);pieces.push(item);
        }
        for i in 0..pieces.len() {self.edges.push((index,i));}
        self.family_entries.insert(index,entry);
        self.patterns.push(JointPattern {id:pattern,entry:entry_id,law_hash:law::LAW_HASH,origin:start.position,pieces});self.field=None;Ok(ids)
    }
}
#[derive(Clone)]struct Variant {lo:u64,hi:u64,map:Option<crate::families::AxisMap>,axis:usize,end:std::sync::Arc<PlayerState>,ticks:u64}
#[derive(Clone)]struct Candidate {variant:usize,lo:u64,hi:u64,lower:u64,upper:u64,next:Option<EdgeId>}
fn position(v:&Variant,bits:u64)->Vec3 {let mut p=v.end.position;if let Some(map)=&v.map {if v.axis==0 {p.x=map.advance(unordered(bits));}else {p.z=map.advance(unordered(bits));}}p}
fn goal_preimage(v:&Variant,goal:Goal)->Option<(u64,u64)> {
    let sides=|bits| {let b=law::Aabb::player(position(v,bits),v.end.pose.height());goal.axis_sides(b,b,v.axis)};
    let p=position(v,v.lo);let b=law::Aabb::player(p,v.end.pose.height());
    for a in 0..3 {if a!=v.axis {let s=goal.axis_sides(b,b,a);if !s.0 || !s.1 {return None;}}}
    if !sides(v.hi).0 || !sides(v.lo).1 {return None;}
    let (mut lo,mut hi)=(v.lo,v.hi);while lo<hi {let m=lo+(hi-lo)/2;if sides(m).0 {hi=m;}else {lo=m+1;}}let first=lo;hi=v.hi;
    while lo<hi {let m=lo+(hi-lo).div_ceil(2);if sides(m).1 {lo=m;}else {hi=m-1;}}sides(first).1.then_some((first,lo))
}
pub(super) fn extend(store:&mut JointStore,field:&mut Field) {
    let mut old=Vec::<FamilyInterval>::new();
    for _ in 0..=store.edges.len()+store.states.len() {
        let mut points_changed=false;
        // Point identity retains the complete witness. Entry equivalence has
        // no aim memory, so all aliases with the same numeric guard share T.
        for nodes in store.point_entries.values() {for &node in nodes {if let Some(t)=field.t[node] {for &other in nodes {
            if Resources::of(&store.states[node]).same_bits(Resources::of(&store.states[other])) && field.t[other].is_none_or(|old|t<old) {
                field.t[other]=Some(t);field.via[other]=field.via[node];points_changed=true;
            }
        }}}}
        let mut next=Vec::new();
        for (&pattern,entry) in &store.family_entries {
            for (piece_index,piece) in store.patterns[pattern].pieces.iter().enumerate() {
                let id=EdgeId(store.edges.iter().position(|p|*p==(pattern,piece_index)).unwrap());if !store.valid(id) {continue;}
                let variants:Vec<_>=match &piece.exit {
                    ExitImage::Points(ids)=>ids.iter().map(|&node|Variant {lo:piece.lo,hi:piece.hi,map:None,axis:entry.axis.unwrap(),end:store.states[node].clone(),ticks:store.states[node].t-entry.base.t}).collect(),
                    ExitImage::Map(image)=>match &image.image {Image::Map(rows)=>rows.iter().map(|r|Variant {lo:r.lo,hi:r.hi,map:Some(r.map.clone()),axis:r.axis,end:std::sync::Arc::new(r.end.clone()),ticks:r.end.t-entry.base.t}).collect(),_=>unreachable!()},
                };
                let mut candidates=Vec::new();let mut cuts=vec![piece.lo,piece.hi+1];
                for (v_index,v) in variants.iter().enumerate() {
                    cuts.extend([v.lo,v.hi+1]);
                    if let Some((lo,hi))=goal_preimage(v,field.goal) {candidates.push(Candidate {variant:v_index,lo,hi,lower:v.ticks,upper:v.ticks,next:None});cuts.extend([lo,hi+1]);}
                    if v.map.is_none() && piece.uses.lo==piece.uses.hi {for node in store.equivalent_points(&v.end) {if let Some(t)=field.t[node] {candidates.push(Candidate {variant:v_index,lo:v.lo,hi:v.hi,lower:v.ticks+t,upper:v.ticks+t,next:field.via[node]});}}}
                    for target in &old {
                        // Exact numeric guards cannot consume a varying use
                        // interval without an additional downstream proof.
                        if piece.uses.lo!=piece.uses.hi {continue;}
                        let Some(upper)=target.upper else {continue;};let target_entry=&store.family_entries[&target.pattern];
                        let mut at=(*v.end).clone();
                        let domain=if entry.axis==target_entry.axis {(v.lo,v.hi)} else {
                            let target_s=target_entry.axis.map(|a|ordered(xyz(v.end.position)[a])).unwrap();
                            if target_s<target.lo || target_s>target.hi {continue;}
                            let fixed=xyz(target_entry.base.position)[v.axis];
                            let map=|bits|ordered(xyz(position(v,bits))[v.axis]);
                            let Some(domain)=pullback(v.lo,v.hi,(ordered(fixed),ordered(fixed)),map) else {continue;};
                            let p=position(v,domain.0);at.position=p;
                            if let Some(mut support)=at.main_supporting_block {support[v.axis]+=xyz(p)[v.axis].floor() as i32-xyz(v.end.position)[v.axis].floor() as i32;at.main_supporting_block=Some(support);}
                            domain
                        };
                        let end_entry=EntryTemplate::with_axis(&at,target_entry.origin,target_entry.phase,target_entry.axis);
                        if end_entry.key!=target_entry.key || !end_entry.resources.same_bits(target_entry.resources) {continue;}
                        let map=|bits|ordered(if v.axis==0 {position(v,bits).x}else {position(v,bits).z});
                        let interval=if entry.axis==target_entry.axis {pullback(domain.0,domain.1,(target.lo,target.hi),map)}else {Some(domain)};
                        if let Some((lo,hi))=interval {
                            candidates.push(Candidate {variant:v_index,lo,hi,lower:v.ticks+target.lower,upper:v.ticks+upper,next:Some(target.via)});cuts.extend([lo,hi+1]);
                        }
                    }
                }
                cuts.sort_unstable();cuts.dedup();
                for range in cuts.windows(2) {
                    let lo=range[0];let hi=range[1]-1;let mut lower=u64::MAX;let mut upper=Some(0u64);let mut via=None;let mut choices=Vec::new();
                    for (index,v) in variants.iter().enumerate() {if v.lo>lo || v.hi<hi {continue;}
                        if let Some(best)=candidates.iter().filter(|c|c.variant==index && c.lo<=lo && hi<=c.hi).min_by_key(|c|c.upper) {lower=lower.min(best.lower);upper=upper.map(|u|u.max(best.upper));choices.push(best.next);}else {lower=0;upper=None;}
                    }
                    if !choices.is_empty() && choices.iter().all(|v|*v==choices[0]) {via=choices[0];}
                    if lower==u64::MAX {lower=0;upper=None;}
                    next.push(FamilyInterval {pattern,piece:piece_index,lo,hi,lower,upper,via:id,next:via});
                }
            }
        }
        next.sort_by_key(|v|(v.pattern,v.piece,v.lo));
        store.work.edge_reads+=next.len() as u64;
        // A point may enter a family, and that family's image may in turn
        // contain points. Close both parts of the same min-plus relation.
        for (node,state) in store.states.iter().enumerate() {for row in &next {
            if row.upper!=Some(row.lower) {continue;}
            let template=&store.family_entries[&row.pattern];
            let entry=EntryTemplate::with_axis(state,template.origin,template.phase,template.axis);
            if entry.key==template.key && entry.resources.same_bits(template.resources) && row.lo<=entry.lattice && entry.lattice<=row.hi && field.t[node].is_none_or(|t|row.lower<t) {
                field.t[node]=Some(row.lower);field.via[node]=Some(row.via);points_changed=true;
            }
        }}
        for i in 0..store.edges.len() {let id=EdgeId(i);if !store.valid(id) || store.family_entries.contains_key(&store.edges[i].0) {continue;}
            let e=store.edge(id);let tail=if field.goal.touched(&e.last_before,&store.states[e.to]) {Some(0)}else {field.t[e.to]};
            if let Some(t)=tail {let cost=t+e.ticks as u64;if field.t[e.from].is_none_or(|old|cost<old) {field.t[e.from]=Some(cost);field.via[e.from]=Some(id);points_changed=true;}}
        }
        let stable=!points_changed && next.len()==old.len() && next.iter().zip(&old).all(|(a,b)|(a.pattern,a.piece,a.lo,a.hi,a.lower,a.upper,a.next)==(b.pattern,b.piece,b.lo,b.hi,b.lower,b.upper,b.next));
        old=next;if stable {break;}
    }
    for &pattern in store.family_entries.keys() {field.partitions[pattern].clear();}
    for (pattern,p) in store.patterns.iter().enumerate() {if !store.family_entries.contains_key(&pattern) {
        field.partitions[pattern]=p.pieces.iter().map(|e|InstanceInterval {lo:unordered(e.lo),hi:unordered(e.hi),t:field.t[e.from],via:field.via[e.from]}).collect();
    }}
    for row in &old {field.partitions[row.pattern].push(InstanceInterval {lo:unordered(row.lo),hi:unordered(row.hi),t:row.upper,via:Some(row.via)});}
    field.families=old;
}
