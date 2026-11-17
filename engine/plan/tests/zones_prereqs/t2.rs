//! P2 consumer: exact word maps and detector preimages for the zone kernel.
use law::{PlayerState, Vec3};
use plan::Goal;
use std::{collections::HashMap, time::Instant};
const P: usize = 12;
const HIDDEN: i128 = 1i128 << 52;
const LIMIT: i128 = 1i128 << 53;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Binary64 { pub negative: bool, pub exp: i32, pub mantissa: u64 }
impl Binary64 {
    pub(super) fn of(x: f64) -> Self {
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
pub(super) struct Region {pub exp:i32, pub lo:i128, pub hi:i128}
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
struct Node {offset:Vec<i128>,delta:i128,next:usize}
#[derive(Clone, Debug)]
struct Graph {nodes:[Node;2],order:Vec<usize>,prefix:usize,period:usize,total:i128}
impl Graph {
    fn new(ds:&[f64],phase:usize,region:Region,k:i128,poison:bool)->Option<Self> {
        let mut nodes=Vec::new();
        for parity in 0..2 {
            let mut offset=vec![0];let mut v=parity as i128;
            for j in 0..ds.len() {v+=increment(ds[(phase+j)%ds.len()],region.exp,v&1,poison)?;offset.push(v-parity as i128);}
            nodes.push(Node {delta:v-parity as i128,next:(v&1) as usize,offset});
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
        let mut answer=requested;let mut base=k;
        for (i,&index) in self.order.iter().enumerate() {
            let node=&self.nodes[index];let step=if i<self.prefix {0} else {self.total};
            for &offset in &node.offset[1..] {
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
pub(super) struct Family {pub region:Region,pub base:i128,pub delta:i128,pub count:u64,pub tick:u64,pub stride:u64,pub before:i128,pub after:i128,pub phase:usize}
#[derive(Clone, Debug)]
pub(super) enum Piece {Family(Family),Boundary {tick:u64,before:f64,after:f64,phase:usize}}
#[derive(Clone, Debug)]
pub(super) struct Fold {pub value:f64,pub segments:usize,pub pieces:Vec<Piece>}
// Loops enumerate binary64 events, two parity states, or fixed word phases.
// No loop bound is the requested cycle count.
pub(super) fn fold(start:f64,ds:&[f64],ticks:u64)->Fold {
    assert!(!ds.is_empty());let mut p=start;let mut t=0;let mut pieces=Vec::new();let mut segments=0;
    if ds.iter().all(|d|*d==0.0) {
        // At most one word changes signed-zero ownership.
        for j in 0..ticks.min(ds.len() as u64) {p+=ds[j as usize];}
        return Fold {value:p,segments:1,pieces};
    }
    while t<ticks {
        segments+=1;let (region,k)=Region::at(p);let phase=t as usize%ds.len();
        let Some(graph)=Graph::new(ds,phase,region,k,false) else {
            let next=p+ds[phase];assert!(next.is_finite());pieces.push(Piece::Boundary {tick:t+1,before:p,after:next,phase});p=next;t+=1;continue;
        };
        let cycles=graph.safe_cycles(region,k,(ticks-t)/ds.len() as u64);let mut base=k;
        for (i,&index) in graph.order.iter().enumerate() {
            if i as u64>=cycles {break;}
            let node=&graph.nodes[index];let count=if i<graph.prefix {1} else {1+(cycles-1-i as u64)/graph.period as u64};
            for j in 0..ds.len() {pieces.push(Piece::Family(Family {region,base,
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
                pieces.push(Piece::Boundary {tick:t+j as u64+1,before:p,after:next,phase:(phase+j)%ds.len()});p=next;progressed=j+1;break;
            }
            pieces.push(Piece::Family(Family {region,base:current,delta:0,count:1,tick:t+j as u64+1,stride:0,
                before:node.offset[j],after:node.offset[j+1],phase:(phase+j)%ds.len()}));progressed=j+1;
            if progressed==available {p=region.value(next);}
        }
        assert!(progressed>0);t+=progressed as u64;
    }
    Fold {value:p,segments,pieces}
}
fn ceil_div(n:i128,d:i128)->i128 {-(-n).div_euclid(d)}
// Independent integer-domain bisection oracle for the endpoint derivation.
fn first_true(mut lo:i128,mut hi:i128,predicate:impl Fn(i128)->bool)->i128 {
    while lo<hi {let mid=lo+(hi-lo)/2;if predicate(mid) {hi=mid;} else {lo=mid+1;}}lo
}
fn phase_state(phases:&[PlayerState],phase:usize,x:f64)->PlayerState {let mut s=phases[phase].clone();s.position.x=x;s}
fn phase_box(phases:&[PlayerState],phase:usize,x:f64)->law::Aabb {
    let s=&phases[phase];law::Aabb::player(Vec3::new(x,s.position.y,s.position.z),s.pose.height())
}
fn signed(b:Binary64)->i128 {if b.negative {-(b.mantissa as i128)} else {b.mantissa as i128}}
// Exact signed dyadic sum for the midpoint subtraction, across binary64's
// whole exponent range. The old i128 alignment overflowed at the zero edge
// once the goal epsilon was removed. Native addition remains the independent oracle.
const LIMBS:usize=((f64::MAX_EXP-f64::MIN_EXP+f64::MANTISSA_DIGITS as i32+1) as usize).div_ceil(64);
fn compare(a:&[u64;LIMBS],b:&[u64;LIMBS])->std::cmp::Ordering {a.iter().rev().cmp(b.iter().rev())}
fn subtract(a:&mut [u64;LIMBS],b:&[u64;LIMBS]) {
    let mut borrow=false;
    for i in 0..LIMBS {let (v,b0)=a[i].overflowing_sub(b[i]);let (v,b1)=v.overflowing_sub(borrow as u64);a[i]=v;borrow=b0||b1;}
    assert!(!borrow);
}
fn inverse_edge(exp:i32,add:f64,edge:f64)->i128 {
    let a=Binary64::of(edge);let b=Binary64::of(edge.next_up());let c=Binary64::of(add);
    let terms=[(signed(a),a.exp-1),(signed(b),b.exp-1),(-signed(c),c.exp)];
    let common=terms.iter().filter(|(n,_)|*n!=0).map(|(_,e)|*e).min().unwrap_or(0);
    let mut magnitude=[0u64;LIMBS];let mut negative=false;
    for (n,e) in terms {
        if n==0 {continue;}
        let mut term=[0u64;LIMBS];let shift=(e-common) as usize;let nabs=n.unsigned_abs();
        for bit in 0..128 {if nabs&(1u128<<bit)!=0 {let at=bit+shift;term[at/64]|=1u64<<(at%64);}}
        if negative==(n<0) {
            let mut carry=false;
            for i in 0..LIMBS {let (v,c0)=magnitude[i].overflowing_add(term[i]);let (v,c1)=v.overflowing_add(carry as u64);magnitude[i]=v;carry=c0||c1;}
            assert!(!carry);
        } else if compare(&magnitude,&term).is_ge() {subtract(&mut magnitude,&term);}
        else {subtract(&mut term,&magnitude);magnitude=term;negative=n<0;}
    }
    let highest=magnitude.iter().rposition(|&w|w!=0).map(|i|i*64+63-magnitude[i].leading_zeros() as usize);
    let (floor,ceil)=if let Some(highest)=highest {
        let shift=common-exp;
        if highest as i32+shift>125 {
            let outside=if negative {i128::MIN/2} else {i128::MAX/2};(outside,outside)
        } else {
            let bit=|i:usize|i<LIMBS*64 && magnitude[i/64]&(1u64<<(i%64))!=0;
            let mut integer=0i128;
            for i in 0..126 {let source=i as i32-shift;if source>=0 && bit(source as usize) {integer|=1i128<<i;}}
            let remainder=shift<0 && (0..(-shift) as usize).any(bit);
            if negative {(-integer-remainder as i128,-integer)} else {(integer,integer+remainder as i128)}
        }
    } else {(0,0)};
    if edge.to_bits()&1==0 {floor+1} else {ceil}
}
fn first_touch(fold:&Fold,phases:&[PlayerState],goal:Goal)->Option<u64> {
    let zone=goal.zone();let mut answer=None;
    for piece in &fold.pieces {
        let found=match piece {
            Piece::Boundary {tick,before,after,phase}=> {
                let a=phase_state(phases,*phase,*before);let b=phase_state(phases,(*phase+1)%P,*after);goal.touched(&a,&b).then_some(*tick)
            }
            Piece::Family(f)=> {
                let tube=plan::goal::hull(phases[f.phase].bounding_box(),phases[(f.phase+1)%P].bounding_box());
                if tube.min.y>zone.max.y || tube.max.y<zone.min.y || tube.min.z>zone.max.z || tube.max.z<zone.min.z {continue;}
                // Positive-axis domain; the same families feed future two-yaw work.
                assert!(f.after>=f.before && f.delta>=0);
                let last=f.base+(f.count-1) as i128*f.delta;
                if phase_box(phases,(f.phase+1)%P,f.region.value(last+f.after)).max.x<zone.min.x
                    || phase_box(phases,f.phase,f.region.value(f.base+f.before)).min.x>zone.max.x {continue;}
                // Family endpoints may lie on a binade boundary. The fold
                // already proves its generated interval; the inverse must not
                // discard the boundary's exact goal touch.
                let low=f.region.lo-f.before.min(f.after);let high=f.region.hi-f.before.max(f.after);if low>high {continue;}
                let radius=law::Aabb::player(Vec3::default(),phases[f.phase].pose.height()).max.x;
                let enter=(inverse_edge(f.region.exp,radius,zone.min.x.next_down())-f.after).max(low);
                let exit=(inverse_edge(f.region.exp,-radius,zone.max.x)-f.before-1).min(high);
                if enter>exit {continue;}
                let r=if f.delta==0 {if f.base<enter || f.base>exit {continue;}0} else {ceil_div(enter-f.base,f.delta).max(0)};
                if r>=f.count as i128 || f.base+r*f.delta>exit {continue;}Some(f.tick+r as u64*f.stride)
            }
        };
        if let Some(t)=found {answer=Some(answer.map_or(t,|old:u64|old.min(t)));}
    }answer
}
fn motor(s:&PlayerState)->Vec<u64> {
    let mut s=s.clone();s.t=0;s.position=Vec3::default();s.main_supporting_block=None;
    // P1 motion projection: retain actual resource guards, separate numerics.
    let mut b=s.bits();b[8]=(s.food_level>6) as u64;b[9]=0;b[47]=(s.health>0.0) as u64;b[48]=0;b[49]=0;b
}
struct Prepared {world:law::BoxWorld,phases:Vec<PlayerState>,ds:Vec<f64>,input:law::Input}
// Finite query horizon for the measurement, derived from the detector's
// no-return edge and a proven downward-rounded per-tick displacement bound.
fn detector_count_limit(p:&Prepared,start:f64,goal:Goal)->u64 {
    let radius=law::Aabb::player(Vec3::default(),p.phases[0].pose.height()).max.x;
    let zone=goal.zone();let (region,_)=Region::at(zone.max.x);
    let beyond=inverse_edge(region.exp,-radius,zone.max.x);
    assert!(beyond>region.lo && beyond<region.hi);
    let far=region.value(beyond)+p.ds.iter().copied().reduce(f64::max).unwrap();
    assert!(far<region.value(region.hi)); // Last addition cannot need a coarser U.
    let (anchor,remainder)=quotient(start,region.exp).unwrap();
    assert_eq!(remainder,std::cmp::Ordering::Less);
    // These measurement anchors are exactly on this coarsest lattice.
    assert_eq!((anchor as f64*2.0f64.powi(region.exp)).to_bits(),start.to_bits());
    let smallest=p.ds.iter().map(|d|quotient(*d,region.exp).unwrap().0).min().unwrap();
    // Subtract one lattice unit: even a crossing addition's rounding error is
    // at most half a unit, and every finer binade has no larger error.
    let lower=smallest-1;assert!(lower>0);
    ceil_div(beyond-anchor,lower).try_into().unwrap()
}
fn prepare()->Prepared {
    let lines=include_str!("../../../world/tests/fixtures/tracks.txt");
    let track=plan::flight::track(lines.lines().find(|l|l.starts_with("straight100 ")).unwrap()).unwrap();
    assert_eq!(law::World::difficulty(&track.world),0);
    let stone=law::World::block(&track.world,track.a).clone();
    let mut world=law::BoxWorld::default();world.difficulty=0;
    world.fill([-3_000_000,-61,-3_000_000],[3_000_000,-61,3_000_000],stone);
    let mut s=plan::settle(&plan::Still(&world),[300.5,-60.0,-479.5],0).unwrap();
    let programs=words::tiny_programs();let program=&programs[30];
    // P1 word/phase, bound to the actual native direction-table index for +x.
    let yaw=((49152.5*std::f64::consts::TAU/65536.0)/0.017453292f32 as f64) as f32;
    let args=words::Arguments {refined_yaw:Some(yaw),..Default::default()};
    let mut states=vec![s.clone()];let mut moved=Vec::new();let mut seen=HashMap::new();seen.insert(motor(&s),0);
    let (first,period)=loop {
        let input=program.select(&world,&s,(states.len()-1) as u32,args).unwrap().1;
        let (wanted,actual)=law::tick_with_collision(&mut s,&world,&input).unwrap();
        // movement_this_tick stores wanted: prove equality before using x/z.
        assert_eq!(wanted.x.to_bits(),actual.x.to_bits());assert_eq!(wanted.z.to_bits(),actual.z.to_bits());
        assert_eq!(s.movement_this_tick.unwrap().x.to_bits(),actual.x.to_bits());
        moved.push(actual);states.push(s.clone());
        if let Some(&f)=seen.get(&motor(&s)) {break(f,states.len()-1-f);}
        assert!(states.len()<4096);seen.insert(motor(&s),states.len()-1);
    };
    assert_eq!((first,period),(263,P));
    let phases=states[first..first+P].to_vec();let ds=moved[first..first+P].iter().map(|v|v.x).collect::<Vec<_>>();
    assert!(ds.iter().all(|d|*d>0.0));assert!(moved[first..first+P].iter().all(|v|v.z==0.0));
    println!("preparation transient={first} period={period} operands_bits={:?}",ds.iter().map(|v|v.to_bits()).collect::<Vec<_>>());
    let input=program.select(&world,&s,first as u32,args).unwrap().1;
    Prepared {world,phases,ds,input}
}
fn endpoint(prepared:&Prepared,start:&PlayerState,ticks:u64,fold:&Fold)->PlayerState {
    let mut s=prepared.phases[ticks as usize%P].clone();s.t=start.t+ticks;s.position.x=fold.value;s.position.z=start.position.z;
    s.main_supporting_block=if s.on_ground {Some([s.position.x.floor() as i32,-61,s.position.z.floor() as i32])} else {None};s
}
fn check_endpoint(p:&Prepared,start:&PlayerState,ticks:u64,reference:&PlayerState) {
    let folded=fold(start.position.x,&p.ds,ticks);let got=endpoint(p,start,ticks,&folded);
    assert_eq!([got.position.x.to_bits(),got.position.y.to_bits(),got.position.z.to_bits()],
        [reference.position.x.to_bits(),reference.position.y.to_bits(),reference.position.z.to_bits()],"positions tick={ticks} start={:?}",start.position);
    assert_eq!(motor(&got),motor(reference),"motor tick={ticks}");
    assert_eq!(got.main_supporting_block,reference.main_supporting_block,"support tick={ticks}");assert_eq!(got.t,reference.t);
}
#[test]
fn t2_binade() {
    let poison=std::env::var("PATHTRACING_T2_POISON").is_ok_and(|s|s=="1");
    // Independent exact half-ULP witnesses include odd floor N and negative d.
    for k in [HIDDEN+2,HIDDEN+3] {for d in [0.5,1.5,-0.5,-1.5] {
        let region=Region {exp:0,lo:HIDDEN,hi:LIMIT};let p=region.value(k);
        let actual=region.value(k+increment(d,0,k&1,poison).unwrap());
        assert_eq!(actual.to_bits(),(p+d).to_bits(),"tie parity witness k={k} d={d} poison={poison}");
    }}
    println!("ASSERT tie-even/odd and negative dyadics PASS");
    for x in [0.0,-0.0,f64::from_bits(1),-f64::from_bits(1),f64::MIN_POSITIVE,-f64::MIN_POSITIVE,1.0,-1.0,f64::MAX] {
        assert_eq!(Binary64::of(x).value().to_bits(),x.to_bits());
    }
    for (start,ds,n) in [(0.0,vec![f64::from_bits(1)],19),(-f64::from_bits(3),vec![f64::from_bits(1)],7),
        (-0.0,vec![-0.0],1001),(-0.0,vec![0.0],1001),(f64::MIN_POSITIVE,vec![-f64::from_bits(1)],17),(-1.0,vec![-0.5,-0.25],1001)] {
        let mut oracle=start;for j in 0..n {oracle+=ds[j as usize%ds.len()];}
        assert_eq!(fold(start,&ds,n).value.to_bits(),oracle.to_bits());
    }
    println!("ASSERT normal/subnormal/signed-zero/negative-step PASS");
    for (ds,k) in [(vec![0.5],HIDDEN+3),(vec![1.5],HIDDEN+3),(vec![1.0],HIDDEN+2),(vec![0.5,1.0],HIDDEN+2)] {
        let region=Region {exp:0,lo:HIDDEN,hi:LIMIT};
        let graph=Graph::new(&ds,0,region,k,false).unwrap();
        for n in [1,2,17,1009] {let mut oracle=region.value(k);for j in 0..n {oracle+=ds[j as usize%ds.len()];}
            assert_eq!(fold(region.value(k),&ds,n).value.to_bits(),oracle.to_bits());}
        println!("ASSERT parity_graph ds={ds:?} prefix={} period={} PASS",graph.prefix,graph.period);
    }
    let trillion=1_000_000_000_000u64;
    assert_eq!(fold((HIDDEN+3) as f64,&[0.5],trillion).value.to_bits(),((HIDDEN+4) as f64).to_bits());
    assert_eq!(fold((HIDDEN+2) as f64,&[1.0],trillion).value.to_bits(),((HIDDEN+2+trillion as i128) as f64).to_bits());
    assert_eq!(fold(0.0,&[f64::from_bits(1)],HIDDEN as u64).value.to_bits(),f64::MIN_POSITIVE.to_bits());
    println!("ASSERT quotient/remainder trillion-count and subnormal boundary PASS");
    let radius=law::Aabb::player(Vec3::default(),law::Pose::Standing.height()).max.x;
    let mut thresholds=0;
    for exp in [-1074,-1000,-64,-53,-52,-40,-33,-32,-31,-21,0,971] {for negative in [false,true] {
        let region=if negative {Region {exp,lo:-LIMIT,hi:-HIDDEN}} else {Region {exp,lo:HIDDEN,hi:LIMIT}};
        for cell in [i32::MIN,-1,0,1,100,10_000,1_000_000,i32::MAX] {
            let zone=Goal {cell:[cell,0,0]}.zone();
            for (add,edge) in [(radius,zone.min.x),(-radius,zone.max.x.next_down())] {
                let low=region.lo+1;let high=region.hi-1;
                let exact=inverse_edge(exp,add,edge).clamp(low,high+1);
                let native=first_true(low,high+1,|k|region.value(k)+add>edge);
                assert_eq!(exact,native,"rounded edge exp={exp} negative={negative} cell={cell} add={add}");thresholds+=1;
            }
        }
    }}
    println!("ASSERT rounded detector edge integer thresholds={thresholds} PASS");
    let p=prepare();
    let positive=(2.0f64).powi(19)-100.0;
    let positive_ticks=(((2.0f64).powi(21)+101.0-positive)/p.ds.iter().sum::<f64>()*P as f64).ceil() as u64;
    let negative=-(2.0f64).powi(20)-100.0;
    let negative_ticks=((-(2.0f64).powi(19)+101.0-negative)/p.ds.iter().sum::<f64>()*P as f64).ceil() as u64;
    let mut comparisons=0;let mut inverse_comparisons=0;
    for (name,x,n) in [("positive",positive,positive_ticks),("negative",negative,negative_ticks),("zero",-50.0,400),("minus-zero",-0.0,400),("plus-zero",0.0,400)] {
        let mut start=p.phases[0].clone();start.position.x=x;let mut reference=start.clone();
        let queries=[1,2,11,12,13,37*P as u64,1009*P as u64+7,n/2,n-1,n];
        let targets=if name=="positive" {vec![524288,1048576,2097152]} else if name=="negative" {vec![-1048576,-524288]} else {vec![0,50,100]};
        let goals=targets.into_iter().flat_map(|x|[-61,-60,-59].into_iter().map(move |y|Goal {cell:[x,y,-480]})).collect::<Vec<_>>();let mut native_touch=vec![None;goals.len()];
        let mut native_ends=vec![None;goals.len()];
        for tick in 1..=n {
            let before=reference.clone();law::try_tick(&mut reference,&p.world,&p.input).unwrap();
            for (i,goal) in goals.iter().enumerate() {if native_touch[i].is_none() && goal.touched(&before,&reference) {native_touch[i]=Some(tick);native_ends[i]=Some(reference.clone());}}
            if queries.contains(&tick) {check_endpoint(&p,&start,tick,&reference);comparisons+=1;}
        }
        let projected=fold(x,&p.ds,n);
        for (i,goal) in goals.iter().enumerate() {
            assert_eq!(first_touch(&projected,&p.phases,*goal),native_touch[i],"inverse name={name} goal={goal:?}");inverse_comparisons+=1;
            if let Some(t)=native_touch[i] {check_endpoint(&p,&start,t,native_ends[i].as_ref().unwrap());comparisons+=1;}
        }
        println!("ASSERT stream {name} ticks={n} segments={} endpoint+motor PASS inverse={native_touch:?}",projected.segments);
    }
    for sign in [-1.0,1.0] {for exp in [19,20,21] {let boundary=(2.0f64).powi(exp);for bits in [boundary.to_bits()-1,boundary.to_bits(),boundary.to_bits()+1] {
        let mut start=p.phases[0].clone();start.position.x=sign*f64::from_bits(bits);let mut reference=start.clone();
        for tick in 1..=29 {law::try_tick(&mut reference,&p.world,&p.input).unwrap();check_endpoint(&p,&start,tick,&reference);comparisons+=1;}
    }}}
    // Slab and body-contact neighbours at every vertical phase. All dimensions
    // enter through law bounding_box; no hand-typed width/height/eye dimensions.
    let goal=Goal {cell:[0,-61,-480]};let zone=goal.zone();
    let radius=law::Aabb::player(Vec3::default(),law::Pose::Standing.height()).max.x;let mut edge_checks=0;
    for phase in 0..P {for edge in [zone.min.x,zone.max.x,zone.min.x-radius,zone.max.x+radius] {for x in [edge.next_down(),edge,edge.next_up()] {
        let mut phases=p.phases.clone();phases.rotate_left(phase);let mut ds=p.ds.clone();ds.rotate_left(phase);
        let mut before=phases[0].clone();before.position.x=x;let mut after=before.clone();law::try_tick(&mut after,&p.world,&p.input).unwrap();
        let projected=fold(x,&ds,1);assert_eq!(projected.value.to_bits(),after.position.x.to_bits());
        assert_eq!(after.position.y.to_bits(),phases[1].position.y.to_bits());assert_eq!(after.position.z.to_bits(),phases[1].position.z.to_bits());
        assert_eq!(motor(&after),motor(&phases[1]));
        assert_eq!(first_touch(&projected,&phases,goal),goal.touched(&before,&after).then_some(1),"phase={phase} x={x:?} bits={:x} before={:?} after={:?} pieces={:?}",x.to_bits(),before.position,after.position,projected.pieces);edge_checks+=1;
    }}}
    let mut witness=p.phases[0].clone();witness.position=Vec3::new(zone.min.x-radius,-60.0,-479.5);assert!(goal.touched(&witness,&witness));
    println!("ASSERT detector law radius={radius:?} height={:?} edge_neighbours={edge_checks} PASS",witness.pose.height());
    println!("ASSERT comparisons positions+motor={comparisons} inverse={inverse_comparisons} PASS");
    for distance in [100,10_000,1_000_000] {
        let goal=Goal {cell:[distance,-59,-480]};let mut times=Vec::new();let mut result=None;
        for _ in 0..101 {
            let now=Instant::now();let max_ticks=detector_count_limit(&p,0.5,goal);let projected=fold(0.5,&p.ds,max_ticks);let tick=first_touch(&projected,&p.phases,goal).unwrap();
            let stopped=fold(0.5,&p.ds,tick);let end=std::hint::black_box(stopped.value);times.push(now.elapsed().as_nanos());result=Some((tick,stopped.segments,end,max_ticks));
        }
        times.sort();let (tick,segments,end,max_ticks)=result.unwrap();
        println!("MEASURE distance={distance} first_touch={tick} remainder={} segments={segments} end_bits={} ns_min={} ns_median={} ns_max={} n={} horizon_ticks={max_ticks}",
            tick%P as u64,end.to_bits(),times[0],times[times.len()/2],times[times.len()-1],times.len());
        println!("TIMINGS distance={distance} sorted_ns={times:?}");
    }
}
