//! Exact binary64 event maps, promoted from the judged P2 kernel.
use law::{PlayerState,Vec3};
use crate::Goal;
use std::{cell::RefCell,collections::HashMap,sync::Arc};
const HIDDEN: i128 = 1i128 << 52;
const LIMIT: i128 = 1i128 << 53;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Binary64 { pub negative: bool, pub exp: i32, pub mantissa: u64 }
impl Binary64 {
    pub(crate) fn of(x: f64) -> Self {
        assert!(x.is_finite(), "nonfinite outside qualified domain");
        let bits=x.to_bits(); let field=((bits>>52)&2047) as i32;
        Self {negative:bits>>63!=0, exp:if field==0 {-1074} else {field-1075},
            mantissa:(bits&((1u64<<52)-1))|if field==0 {0} else {1u64<<52}}
    }
    fn value(self) -> f64 {
        let field=if self.exp == -1074 && self.mantissa<1u64<<52 {0} else {(self.exp+1075) as u64};
        f64::from_bits(((self.negative as u64)<<63)|(field<<52)|(self.mantissa&((1u64<<52)-1)))
    }
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct Region {pub exp:i32, pub lo:i128, pub hi:i128}
impl Region {
    fn at(x:f64)->(Self,i128) {
        let b=Binary64::of(x);let k=b.mantissa as i128*if b.negative {-1} else {1};
        // Keep subnormal/zero/sign ownership explicit even when spacing agrees.
        let low=if b.mantissa<(1u64<<52) {0} else {HIDDEN};
        let high=if low==0 {HIDDEN} else {LIMIT};
        let (lo,hi)=if b.negative {(-high,-low)} else {(low,high)};
        (Self {exp:b.exp,lo,hi},k)
    }
    fn value(self,k:i128)->f64 {
        assert!(k>=self.lo && k<=self.hi);
        if k.abs()==LIMIT {return Binary64 {negative:k<0,exp:self.exp+1,mantissa:HIDDEN as u64}.value();}
        Binary64 {negative:k<0,exp:self.exp,mantissa:k.unsigned_abs() as u64}.value()
    }
}
// floor(d/U) and the EXACT comparison of the dyadic remainder with 1/2.
// None means the shift exceeds this integer domain: the next operation must
// leave the current region and is executed as an exact boundary addition.
fn quotient(d:f64,exp:i32)->Option<(i128,std::cmp::Ordering)> {
    use std::cmp::Ordering::*;
    let b=Binary64::of(d);if b.mantissa==0 {return Some((0,Less));}
    let shift=b.exp-exp;let m=b.mantissa as i128;
    if shift>=0 {
        if shift>70 {return None;}
        return Some((if b.negative {-(m<<shift)} else {m<<shift},Less));
    }
    let bits=-shift;
    if bits>54 {return Some(if b.negative {(-1,Greater)} else {(0,Less)});}
    let denominator=1i128<<bits;let signed=if b.negative {-m} else {m};
    let n=signed.div_euclid(denominator);let r=signed.rem_euclid(denominator);
    Some((n,(r*2).cmp(&denominator)))
}
fn increment(d:f64,exp:i32,parity:i128,poison:bool)->Option<i128> {
    use std::cmp::Ordering::*;
    let (n,r)=quotient(d,exp)?;
    Some(n+match r {Less=>0,Greater=>1,Equal=>((parity+n)&1)^poison as i128})
}
#[derive(Clone, Debug)]
struct Node {offset:Vec<i128>,delta:i128,next:usize,min:i128,max:i128}
#[derive(Clone, Debug)]
struct Graph {nodes:[Node;2],order:Vec<usize>,prefix:usize,period:usize,total:i128}
impl Graph {
    fn new(ds:&[f64],phase:usize,region:Region,k:i128,poison:bool)->Option<Self> {
        let mut nodes=Vec::new();
        for parity in 0..2 {
            let mut offset=vec![0];let mut v=parity as i128;
            for j in 0..ds.len() {v+=increment(ds[(phase+j)%ds.len()],region.exp,v&1,poison)?;offset.push(v-parity as i128);}
            let min=*offset[1..].iter().min().unwrap();let max=*offset[1..].iter().max().unwrap();
            nodes.push(Node {delta:v-parity as i128,next:(v&1) as usize,offset,min,max});
        }
        let nodes:[Node;2]=nodes.try_into().unwrap();let mut order=Vec::new();let mut seen=[None;2];let mut current=(k&1) as usize;
        while seen[current].is_none() {seen[current]=Some(order.len());order.push(current);current=nodes[current].next;}
        let prefix=seen[current].unwrap();let period=order.len()-prefix;
        let total=order[prefix..].iter().map(|&i|nodes[i].delta).sum();
        Some(Self {nodes,order,prefix,period,total})
    }
    fn displacement(&self,cycles:u64)->i128 {
        let n=cycles as usize;let head=n.min(self.prefix);
        let mut out:i128=self.order[..head].iter().map(|&i|self.nodes[i].delta).sum();
        if n>head {let remaining=n-head;out+=(remaining/self.period) as i128*self.total;
            out+=self.order[self.prefix..self.prefix+remaining%self.period].iter().map(|&i|self.nodes[i].delta).sum::<i128>();}
        out
    }
    fn safe_cycles(&self,region:Region,k:i128,requested:u64)->u64 {
        if requested==0 {return 0;}
        let mut answer=requested;let mut base=k;
        for (i,&index) in self.order.iter().enumerate() {
            let node=&self.nodes[index];let step=if i<self.prefix {0} else {self.total};
            for offset in [node.min,node.max] {
                let v=base+offset;
                let r=if v<=region.lo || v>=region.hi {Some(0)}
                    else if i<self.prefix || step==0 {None}
                    else if step>0 {Some((region.hi-1-v)/step+1)} else {Some((v-region.lo-1)/(-step)+1)};
                if let Some(r)=r {let n=i as i128+r*self.period as i128;if n<answer as i128 {answer=n as u64;}}
            }
            base+=node.delta;
        }
        answer
    }
}
// One affine family per phase/parity state, no tick array.
#[derive(Clone, Debug)]
pub(crate) struct Family {pub region:Region,pub base:i128,pub delta:i128,pub count:u64,pub tick:u64,pub stride:u64,pub before:i128,pub after:i128,pub phase:usize}
#[derive(Clone, Debug)]
pub(crate) enum Piece {Family(Family),Boundary {tick:u64,before:f64,after:f64,phase:usize}}
#[cfg(test)]
#[derive(Clone, Debug)]
pub(crate) struct Fold {pub value:f64,pub segments:usize,pub pieces:Vec<Piece>}
// Loops enumerate binary64 events, two parity states, or fixed word phases.
// No loop bound is the requested cycle count.
fn walk_map(map:&Map,start:f64,ticks:u64,mut emit:impl FnMut(Piece))->(f64,usize) {
    let ds=&map.ds;
    assert!(!ds.is_empty());let mut p=start;let mut t=0;let mut segments=0;
    if ds.iter().all(|d|*d==0.0) {
        // At most one word changes signed-zero ownership.
        for j in 0..ticks.min(ds.len() as u64) {p+=ds[j as usize];}
        return (p,1);
    }
    while t<ticks {
        segments+=1;let (region,k)=Region::at(p);let phase=t as usize%ds.len();
        let Some(graph)=map.graph(phase,region,k) else {
            let next=p+ds[phase];assert!(next.is_finite());emit(Piece::Boundary {tick:t+1,before:p,after:next,phase});p=next;t+=1;continue;
        };
        let cycles=graph.safe_cycles(region,k,(ticks-t)/ds.len() as u64);let mut base=k;
        for (i,&index) in graph.order.iter().enumerate() {
            if i as u64>=cycles {break;}
            let node=&graph.nodes[index];let count=if i<graph.prefix {1} else {1+(cycles-1-i as u64)/graph.period as u64};
            for j in 0..ds.len() {emit(Piece::Family(Family {region,base,
                delta:if i<graph.prefix {0} else {graph.total},count,tick:t+i as u64*ds.len() as u64+j as u64+1,
                stride:graph.period as u64*ds.len() as u64,before:node.offset[j],after:node.offset[j+1],phase:(phase+j)%ds.len()}));}
            base+=node.delta;
        }
        let current=k+graph.displacement(cycles);t+=cycles*ds.len() as u64;if cycles>0 {p=region.value(current);}
        if t==ticks {break;}
        let node=&graph.nodes[(current&1) as usize];let available=(ticks-t).min(ds.len() as u64) as usize;let mut progressed=0;
        for j in 0..available {
            let next=current+node.offset[j+1];
            if next<=region.lo || next>=region.hi {
                if progressed>0 {p=region.value(current+node.offset[j]);}
                let next=p+ds[(phase+j)%ds.len()];assert!(next.is_finite());
                emit(Piece::Boundary {tick:t+j as u64+1,before:p,after:next,phase:(phase+j)%ds.len()});p=next;progressed=j+1;break;
            }
            emit(Piece::Family(Family {region,base:current,delta:0,count:1,tick:t+j as u64+1,stride:0,
                before:node.offset[j],after:node.offset[j+1],phase:(phase+j)%ds.len()}));progressed=j+1;
            if progressed==available {p=region.value(next);}
        }
        assert!(progressed>0);t+=progressed as u64;
    }
    (p,segments)
}
#[cfg(test)]
fn fold_map(map:&Map,start:f64,ticks:u64)->Fold {
    let mut pieces=Vec::new();let (value,segments)=walk_map(map,start,ticks,|p|pieces.push(p));
    Fold {value,segments,pieces}
}
/// A generator's exact maps are shared across all coordinates, commands and
/// counts in the same binade/parity regime. No position or goal is a cache key.
#[derive(Clone,Debug)]
pub struct Map {ds:Vec<f64>,graphs:RefCell<HashMap<(i32,usize,u8),Option<Arc<Graph>>>>}
impl Map {
    pub fn new(ds:Vec<f64>)->Self {Self {ds,graphs:RefCell::default()}}
    pub fn len(&self)->usize {self.ds.len()}
    fn graph(&self,phase:usize,region:Region,k:i128)->Option<Arc<Graph>> {
        let key=(region.exp,phase,(k&1) as u8);
        if !self.graphs.borrow().contains_key(&key) {
            let graph=Graph::new(&self.ds,phase,region,k,false).map(Arc::new);self.graphs.borrow_mut().insert(key,graph);
        }
        // A streamed family's consumer can query this same map at another
        // coordinate. Hold immutable graph ownership, not a cache borrow,
        // across that callback, so a missing binade may be compiled there.
        self.graphs.borrow().get(&key).unwrap().clone()
    }
    #[cfg(test)]
    pub(crate) fn fold(&self,start:f64,ticks:u64)->Fold {fold_map(self,start,ticks)}
    #[cfg(test)]
    pub(crate) fn boundaries(&self,start:f64,ticks:u64)->(usize,Vec<u64>) {
        let mut boundaries=Vec::new();
        let (_,segments)=walk_map(self,start,ticks,|p|if let Piece::Boundary {tick,..}=p {boundaries.push(tick);});
        (segments,boundaries)
    }
    pub(crate) fn trace_boundaries(&self,start:f64,ticks:u64,mut emit:impl FnMut(u64))->(f64,usize) {walk_map(self,start,ticks,|p|if let Piece::Boundary {tick,..}=p {emit(tick);})}
    pub fn advance(&self,start:f64,ticks:u64)->f64 {advance_map(self,start,ticks)}
    /// First crossing of a scalar threshold by a sign-definite addition word.
    /// Skip complete binade runs; invert the final graph's integer cycle
    /// count, then inspect at most one word. No families or inverse probes
    /// are constructed along the travelled distance.
    pub fn first_cross(&self,start:f64,target:f64,horizon:u64)->Option<u64> {
        let sign=if self.ds.iter().all(|d|*d>=0.0) {1.0} else if self.ds.iter().all(|d|*d<=0.0) {-1.0} else {return None;};
        let reached=|p:f64|sign*p>=sign*target;
        if reached(start) {return Some(0);}
        if self.ds.is_empty() || self.ds.iter().all(|d|*d==0.0) {return None;}
        let (mut p,mut tick)=(start,0u64);let period=self.ds.len() as u64;
        while tick<horizon {
            let (region,k)=Region::at(p);let phase=(tick%period) as usize;
            if let Some(graph)=self.graph(phase,region,k) {
                let safe=graph.safe_cycles(region,k,(horizon-tick)/period);
                if safe>0 {
                    let end=region.value(k+graph.displacement(safe));
                    let cycles=if reached(end) {
                        let (mut lo,mut hi)=(1,safe);
                        while lo<hi {let m=lo+(hi-lo)/2;if reached(region.value(k+graph.displacement(m))) {hi=m;}else {lo=m+1;}}
                        lo-1
                    } else {safe};
                    if cycles>0 {p=region.value(k+graph.displacement(cycles));tick+=cycles*period;}
                    if tick==horizon {break;}
                }
            }
            for _ in 0..period.min(horizon-tick) {
                p+=self.ds[(tick%period) as usize];tick+=1;if reached(p) {return Some(tick);}
                let (next,_)=Region::at(p);if next.exp!=region.exp || next.lo!=region.lo || next.hi!=region.hi {break;}
            }
        }None
    }
}
fn ceil_div(n:i128,d:i128)->i128 {-(-n).div_euclid(d)}

/// A no-return horizon, from P2's downward-rounded step bound at the coarsest
/// visited binade. The extra lattice unit is a rounding bound, not an epsilon.
pub fn horizon(from:&impl crate::state_view::StateSource,goal:Goal,increments:impl Iterator<Item=Vec3>+Clone,velocity:Vec3)->u64 {
    let from=from.view();
    let axis=if velocity.x.abs()>=velocity.z.abs() {0} else {2};
    let get=|v:Vec3|if axis==0 {v.x} else {v.z};
    let sign=get(velocity).signum();if sign==0.0 {return words::Arguments::MAX_TICKS;}
    let biggest=increments.clone().map(|v|get(v).abs()).fold(0.0f64,f64::max);
    let zone=goal.zone();let r=from.bounding_box().max.minus(from.position);
    let edge=if sign>0.0 {get(zone.max)+get(r)+biggest} else {-get(zone.min)+get(r)+biggest};
    let far=edge.next_up();let start=sign*get(from.position);
    if start>far {return 0;}
    let exp=Binary64::of(start.abs().max(far.abs())).exp;
    let Some((base,_))=quotient(start,exp) else {return words::Arguments::MAX_TICKS;};
    let Some((end,_))=quotient(far,exp) else {return words::Arguments::MAX_TICKS;};
    let lower=increments.map(|v|quotient(sign*get(v),exp).map(|v|v.0-1).unwrap_or(0)).min().unwrap_or(0);
    if lower<=0 {return words::Arguments::MAX_TICKS;}
    ceil_div(end+1-base,lower).max(0).min(words::Arguments::MAX_TICKS as i128) as u64
}

/// Scalar form of P2: do not allocate the inverse's affine families when only
/// an endpoint is wanted. Remainders enumerate word phases, never q cycles.
pub fn advance(start:f64,ds:&[f64],ticks:u64)->f64 {
    Map::new(ds.to_vec()).advance(start,ticks)
}
fn advance_map(map:&Map,start:f64,ticks:u64)->f64 {
    if ticks==0 {return start;}
    let ds=&map.ds;
    assert!(!ds.is_empty());
    if ds.iter().all(|&d|d==0.0) {return start+ds[0];}
    let mut p=start;let mut t=0;
    while t<ticks {
        let (region,k)=Region::at(p);let phase=(t%ds.len() as u64) as usize;
        if let Some(graph)=map.graph(phase,region,k) {
            let cycles=graph.safe_cycles(region,k,(ticks-t)/ds.len() as u64);
            if cycles>0 {p=region.value(k+graph.displacement(cycles));t+=cycles*ds.len() as u64;}
        }
        if t==ticks {break;}
        let available=(ticks-t).min(ds.len() as u64);
        for _ in 0..available {
            p+=ds[(t%ds.len() as u64) as usize];t+=1;
            let (next,_)=Region::at(p);
            if next.exp!=region.exp || next.lo!=region.lo || next.hi!=region.hi {break;}
        }
    }p
}

#[derive(Clone,Debug,PartialEq,Eq)]
pub enum EventKind {BinadeCrossing,FoodGuard {sprint_allowed:bool},LoadedBoundary,ZoneLabelBoundary,YawSwitch,GoalBoundary}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Event {pub tick:u64,pub kind:EventKind}

#[derive(Clone)]
struct AxisRun {time:u64,stride:u64,count:u64,values:Values}
#[derive(Clone)]
enum Values {Integer {region:Region,base:i128,delta:i128,before:i128,after:i128},Exact(f64,f64)}
impl Values {
    fn pair(&self,r:u64)->(f64,f64) {match *self {
        Self::Integer {region,base,delta,before,after}=>{let k=base+delta*r as i128;(region.value(k+before),region.value(k+after))},
        Self::Exact(a,b)=>(a,b),
    }}
    fn shifted(&self,r:u64,scale:u64)->Self {match *self {
        Self::Integer {region,base,delta,before,after}=>Self::Integer {region,base:base+delta*r as i128,delta:delta*scale as i128,before,after},
        Self::Exact(a,b)=>Self::Exact(a,b),
    }}
}
fn runs(start:f64,map:&Map,ticks:u64,mut emit:impl FnMut(AxisRun)) {
    let ds=&map.ds;
    let p=ds.len() as u64;
    if ds.iter().all(|&d|d==0.0) {for phase in 0..p.min(ticks) {emit(AxisRun {time:phase+1,stride:p,count:1+(ticks-phase-1)/p,values:Values::Exact(start,start)});}return;}
    walk_map(map,start,ticks,|piece|emit(match piece {
        Piece::Family(f)=>{debug_assert_eq!(f.phase as u64,(f.tick-1)%p);AxisRun {time:f.tick,stride:if f.stride==0 {p} else {f.stride},count:f.count,
            values:Values::Integer {region:f.region,base:f.base,delta:f.delta,before:f.before,after:f.after}}},
        Piece::Boundary {tick,before,after,phase}=>{debug_assert_eq!(phase as u64,(tick-1)%p);AxisRun {time:tick,stride:p,count:1,values:Values::Exact(before,after)}},
    }));
}
pub trait PhaseGeometry {fn len(&self)->usize;fn body(&self,index:usize)->(f64,law::Pose);}
impl PhaseGeometry for [PlayerState] {fn len(&self)->usize {self.len()}fn body(&self,i:usize)->(f64,law::Pose) {(self[i].position.y,self[i].pose)}}
fn phase_pair(phases:&(impl PhaseGeometry+?Sized),phase:usize,x:(f64,f64),z:(f64,f64),goal:Goal,relax_y:bool)->(law::Aabb,law::Aabb) {
    let (ay,apose)=phases.body(phase);let (by,bpose)=phases.body((phase+1)%phases.len());
    let mut ap=Vec3::new(x.0,ay,z.0);let mut bp=Vec3::new(x.1,by,z.1);
    if relax_y {let zone=goal.zone();let y=(zone.min.y+zone.max.y-apose.height() as f64)/2.0;ap.y=y;bp.y=y;}
    (law::Aabb::player(ap,apose.height()),law::Aabb::player(bp,bpose.height()))
}
// Derive one coordinate's integer interval using the goal as the oracle. This
// is the sole inverse acceptance definition: no AABB detector is copied here.
fn window(values:&Values,count:u64,axis:usize,phases:&(impl PhaseGeometry+?Sized),phase:usize,goal:Goal,relax_y:bool)->Option<(u64,u64)> {
    let center=goal.center();
    let predicate=|r:u64| {
        let v=values.pair(r);
        let (x,z)=if axis==0 {(v,(center.z,center.z))} else {((center.x,center.x),v)};
        let (a,b)=phase_pair(phases,phase,x,z,goal,relax_y);goal.touched_boxes(a,b)
    };
    let seed=match *values {
        Values::Exact(_,_)=>if predicate(0) {0} else {return None;},
        Values::Integer {region,base,delta,before,after}=>{
            if delta==0 {if predicate(0) {0} else {return None;}}
            else {
                let edge=if axis==0 {center.x} else {center.z};
                let Some((target,_))=quotient(edge,region.exp) else {return None;};
                let mut seed=None;
                for offset in [before,after] {
                    let n=target-base-offset;
                    let q=if delta>0 {n.div_euclid(delta)} else {(-n).div_euclid(-delta)};
                    for candidate in [q,q+1] {
                        let r=candidate.clamp(0,count as i128-1) as u64;
                        if predicate(r) {seed=Some(r);break;}
                    }
                    if seed.is_some() {break;}
                }
                seed?
            }
        }
    };
    // Bracket from the accepted seed. A goal window is usually a few cycles
    // wide even when the requested trip spans millions of ticks. Exponential
    // bracketing is exact on this monotone affine family and uses the same
    // acceptance oracle; there is no geometric epsilon or probe grid.
    let mut hi=seed;let mut step=1u64;
    let mut lo=loop {
        let probe=seed.saturating_sub(step);
        if !predicate(probe) {break probe+1;}
        hi=probe;if probe==0 {break 0;}
        step=step.saturating_mul(2);
    };
    while lo<hi {let m=lo+(hi-lo)/2;if predicate(m) {hi=m;} else {lo=m+1;}}
    let first=lo;lo=seed;step=1;
    hi=loop {
        let probe=seed.saturating_add(step).min(count);
        if probe==count || !predicate(probe) {break probe;}
        lo=probe;step=step.saturating_mul(2);
    };
    while lo+1<hi {let m=lo+(hi-lo)/2;if predicate(m) {lo=m;} else {hi=m;}}
    Some((first,lo))
}
/// Exact preimage of a periodic word's goal event. Finite phase/parity/binade
/// families are intersected by integer time; no loop enumerates travel ticks.
pub fn first_touch(from:&PlayerState,ds:&[Vec3],phases:&[PlayerState],goal:Goal,horizon:u64,relax_y:bool)->Option<u64> {
    let dx:Vec<_>=ds.iter().map(|v|v.x).collect();let dz:Vec<_>=ds.iter().map(|v|v.z).collect();
    first_touch_maps(from,&Map::new(dx),&Map::new(dz),phases,goal,horizon,relax_y)
}
pub fn first_touch_maps(from:&PlayerState,xmap:&Map,zmap:&Map,phases:&[PlayerState],goal:Goal,horizon:u64,relax_y:bool)->Option<u64> {
    first_touch_readout(from.position,xmap,zmap,phases,goal,horizon,relax_y)
}
pub fn first_touch_readout(position:Vec3,xmap:&Map,zmap:&Map,phases:&(impl PhaseGeometry+?Sized),goal:Goal,horizon:u64,relax_y:bool)->Option<u64> {
    if horizon==0 || xmap.ds.is_empty() {return None;}
    let p=xmap.ds.len() as u64;let mut best=None;
    // Intersect the goal with X before visiting Z. Graphs are resident data;
    // the transient families are streamed, not allocated per inverse query.
    runs(position.x,xmap,horizon,|x| {
        let phase=((x.time-1)%p) as usize;
        let Some((first,last))=window(&x.values,x.count,0,phases,phase,goal,relax_y) else {return;};
        let x=AxisRun {time:x.time+first*x.stride,stride:x.stride,count:last-first+1,values:x.values.shifted(first,1)};
        runs(position.z,zmap,horizon,|z| {
        if x.time%p!=z.time%p {return;}
        let end=(x.time+(x.count-1)*x.stride).min(z.time+(z.count-1)*z.stride);
        let start=x.time.max(z.time);if start>end {return;}
        let mut time=x.time+(start-x.time).div_ceil(x.stride)*x.stride;
        if (time-z.time)%z.stride!=0 {time+=x.stride;}
        if time>end || (time-z.time)%z.stride!=0 {return;}
        let stride=x.stride.max(z.stride);let count=1+(end-time)/stride;
        let xv=x.values.shifted((time-x.time)/x.stride,stride/x.stride);
        let zv=z.values.shifted((time-z.time)/z.stride,stride/z.stride);
        let phase=((time-1)%p) as usize;
        // Every retained X sample already belongs to its accepted interval.
        let Some((r,_))=window(&zv,count,1,phases,phase,goal,relax_y) else {return;};
        let (a,b)=phase_pair(phases,phase,xv.pair(r),zv.pair(r),goal,relax_y);
        if goal.touched_boxes(a,b) {let t=time+r*stride;best=Some(best.map_or(t,|old:u64|old.min(t)));}
        });
    });best
}

#[cfg(test)]
mod fold_checks {
    use super::*;
    #[test]
    fn streamed_inverse_can_share_one_map_on_both_axes() {
        let phases=vec![PlayerState {position:Vec3::new(0.5,64.0,1023.5),..Default::default()}];
        let goal=Goal {cell:[3,64,1026]};let from=&phases[0];
        let map=Map::new(vec![0.25]);
        let predicted=first_touch_maps(from,&map,&map,&phases,goal,32,false);
        let mut p=from.position;let mut native=None;
        for tick in 1..=32 {let before=law::Aabb::player(p,from.pose.height());p.x+=0.25;p.z+=0.25;
            if goal.touched_boxes(before,law::Aabb::player(p,from.pose.height())) {native=Some(tick);break;}}
        assert_eq!(predicted,native);assert!(predicted.is_some());
    }
    #[test]
    fn folded_maps_equal_native_addition_and_endpoint() {
        let mut checks=0;
        for ds in [vec![0.0,-0.0],vec![0.25+2.0f64.powi(-47)],vec![0.3,0.2,-0.01],vec![-0.4,-0.3],vec![f64::from_bits(1)]] {
            let map=Map::new(ds.clone());
            for start in [-1000.0,-0.0,0.0,0.5,100.0,1023.75] {for ticks in [1,2,17,263,10003] {
                let folded=map.fold(start,ticks);let mut native=start;
                for t in 0..ticks {native+=ds[t as usize%ds.len()];}
                assert_eq!(folded.value.to_bits(),native.to_bits());
                assert_eq!(folded.value.to_bits(),map.advance(start,ticks).to_bits());
                let (segments,boundaries)=map.boundaries(start,ticks);
                assert_eq!(segments,folded.segments);
                assert_eq!(boundaries,folded.pieces.iter().filter_map(|p|if let Piece::Boundary {tick,..}=p {Some(*tick)} else {None}).collect::<Vec<_>>());
                checks+=1;
            }}
        }
        println!("U2_MAPS cases={checks} worst_error_ulp=0");
    }
}
