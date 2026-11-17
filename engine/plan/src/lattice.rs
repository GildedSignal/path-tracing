//! Native-yaw bearing inversion and integer interval lattices. No yaw or q sweep.
use std::cmp::Ordering;
use law::Vec3;

pub fn yaw(index:u16)->f32 {(((index as f64+0.5)*std::f64::consts::TAU/65536.0)/0.017453292f32 as f64) as f32}
pub fn direction(index:u16)->(f64,f64) {let r=(yaw(index)*0.017453292f32) as f64;(-law::math::sin(r) as f64,law::math::cos(r) as f64)}
#[derive(Clone,Copy,Debug)]
struct Dyadic {n:i128,e:i32}
impl Dyadic {
    fn of(x:f64)->Self {
        assert!(x.is_finite());let b=x.to_bits();let field=((b>>52)&2047) as i32;
        let mut n=((b&((1u64<<52)-1))|if field==0 {0} else {1u64<<52}) as i128;
        let mut e=if field==0 {-1074} else {field-1075};
        if n!=0 {let z=n.trailing_zeros();n>>=z;e+=z as i32;} else {e=0;}
        Self {n:if b>>63!=0 {-n} else {n},e}
    }
    fn product(self,other:Self)->Self {Self {n:self.n*other.n,e:self.e+other.e}.normal()}
    fn minus(self,other:Self)->Self {
        if self.n==0 {return Self {n:-other.n,e:other.e};}
        if other.n==0 {return self;}
        let e=self.e.min(other.e);
        Self {n:(self.n<<(self.e-e))-(other.n<<(other.e-e)),e}.normal()
    }
    fn normal(mut self)->Self {if self.n!=0 {let z=self.n.trailing_zeros();self.n>>=z;self.e+=z as i32;} else {self.e=0;}self}
}
fn shifted_wide(n:u128,shift:u32)->[u64;4] {
    let mut out=[0;4];
    for i in 0..128 {if n&(1u128<<i)!=0 {let bit=i+shift;assert!(bit<256);out[(bit/64) as usize]|=1u64<<(bit%64);}}
    out
}
fn ratio_floor(a:Dyadic,b:Dyadic)->i128 {
    assert!(b.n!=0);
    let negative=(a.n<0)!=(b.n<0);let a=Dyadic {n:a.n.abs(),e:a.e};let b=Dyadic {n:b.n.abs(),e:b.e};
    let e=a.e.min(b.e);let shift=(a.e-e) as u32;let other=(b.e-e) as u32;
    if shift<127 && other<127 {
        if let (Some(n),Some(d))=(a.n.checked_mul(1i128<<shift),b.n.checked_mul(1i128<<other)) {
            return if negative {(-n).div_euclid(d)} else {n/d};
        }
    }
    let n=shifted_wide(a.n as u128,shift);
    let denominator=|q:u32| {
        let p=wide_product(b.n as u128,q as u128);let mut d=[0;4];
        for i in 0..256u32 {if p[(i/64) as usize]&(1u64<<(i%64))!=0 {assert!(i+other<256);d[((i+other)/64) as usize]|=1u64<<((i+other)%64);}}
        d
    };
    let (mut lo,mut hi)=(0u32,u32::MAX);
    while lo<hi {let m=lo+((hi as u64-lo as u64+1)/2) as u32;
        if denominator(m).iter().rev().cmp(n.iter().rev()).is_le() {lo=m;} else {hi=m-1;}}
    if negative {-(lo as i128)-i128::from(denominator(lo)!=n)} else {lo as i128}
}
/// Euclidean remainders in the two-vector basis retain a complete phase cell
/// before the goal. The terminal preimage, rather than these counts, stops it.
pub fn bearing_cell(d1:Vec3,d2:Vec3,target:Vec3)->Option<[u32;2]> {
    let cross=|a:Vec3,b:Vec3|Dyadic::of(a.x).product(Dyadic::of(b.z)).minus(Dyadic::of(a.z).product(Dyadic::of(b.x)));
    let det=cross(d1,d2);if det.n==0 {return None;}
    let a=ratio_floor(cross(target,d2),det).max(0) as u32;
    let b=ratio_floor(cross(d1,target),det).max(0) as u32;
    // Keep the preceding phase cell, including an event at the first phase.
    Some([a.saturating_sub(u32::from(b==0)),b.saturating_sub(1)])
}
fn wide_product(a:u128,b:u128)->[u64;4] {
    let aa=[a as u64,(a>>64) as u64];let bb=[b as u64,(b>>64) as u64];let mut out=[0;4];
    for i in 0..2 {let mut carry=0;for j in 0..2 {let v=aa[i] as u128*bb[j] as u128+out[i+j] as u128+carry;out[i+j]=v as u64;carry=v>>64;}out[i+2]=carry as u64;}
    out
}
fn product_cmp(a:i128,b:i128,c:i128,d:i128)->Ordering {
    let sa=a.signum()*b.signum();let sc=c.signum()*d.signum();
    if sa!=sc {return sa.cmp(&sc);}
    let x=wide_product(a.unsigned_abs(),b.unsigned_abs());let y=wide_product(c.unsigned_abs(),d.unsigned_abs());
    let cmp=x.iter().rev().cmp(y.iter().rev());if sa<0 {cmp.reverse()} else {cmp}
}
fn cross_sign(a:(f64,f64),b:(f64,f64))->Ordering {
    let x=Dyadic::of(a.0);let z=Dyadic::of(a.1);let u=Dyadic::of(b.0);let v=Dyadic::of(b.1);
    let n=x.n*v.n;let m=z.n*u.n;let e=x.e+v.e;let f=z.e+u.e;
    if n.signum()!=m.signum() {return n.signum().cmp(&m.signum());}
    if n==0 {return Ordering::Equal;}
    let hn=128-n.unsigned_abs().leading_zeros() as i32+e;let hm=128-m.unsigned_abs().leading_zeros() as i32+f;
    let cmp=if hn!=hm {hn.cmp(&hm)} else if e>=f {(n.unsigned_abs()<<(e-f)).cmp(&m.unsigned_abs())} else {n.unsigned_abs().cmp(&(m.unsigned_abs()<<(f-e)))};
    if n<0 {cmp.reverse()} else {cmp}
}
fn half(v:(f64,f64))->bool {v.0>0.0 || v.0==0.0 && v.1<0.0}
fn angle_cmp(a:(f64,f64),b:(f64,f64))->Ordering {
    if half(a)!=half(b) {return half(a).cmp(&half(b));}
    cross_sign(a,b).reverse()
}
pub fn bracket(dx:f64,dz:f64)->[u16;2] {
    if dx==0.0 && dz==0.0 {return [0,1];}
    let target=(dx,dz);let mut lo=0u32;let mut hi=65536;
    while lo<hi {let mid=(lo+hi)/2;if angle_cmp(direction(mid as u16),target)==Ordering::Less {lo=mid+1;} else {hi=mid;}}
    let next=lo as u16;
    if lo<65536 && angle_cmp(direction(next),target)==Ordering::Equal {[next,next.wrapping_add(1)]}
    else {[next.wrapping_sub(1),next]}
}

/// Sum floor((a*i+b)/m), Euclidean quotient descent, signed a and b.
pub fn floor_sum(n:i128,m:i128,a:i128,b:i128)->i128 {
    assert!(n>=0 && m>0);
    let mut out=n*(n-1)/2*a.div_euclid(m)+n*b.div_euclid(m);
    let (mut n,mut m,mut a,mut b)=(n,m,a.rem_euclid(m),b.rem_euclid(m));
    loop {
        if a>=m {out+=(n-1)*n*(a/m)/2;a%=m;}
        if b>=m {out+=n*(b/m);b%=m;}
        let y=a*n+b;if y<m {break;}
        n=y/m;b=y%m;std::mem::swap(&mut m,&mut a);
    }out
}
#[derive(Clone,Copy,Debug)]
struct Line {a:i128,b:i128,d:i128}
impl Line {
    fn cmp(self,other:Self,q:i128)->Ordering {product_cmp(self.a*q+self.b,other.d,other.a*q+other.b,self.d)}
    fn ceil(self,q:i128)->i128 {-(-(self.a*q+self.b)).div_euclid(self.d)}
    fn sum_floor(self,lo:i128,hi:i128)->i128 {floor_sum(hi-lo+1,self.d,self.a,self.a*lo+self.b)}
    fn sum_ceil(self,lo:i128,hi:i128)->i128 {-floor_sum(hi-lo+1,self.d,-self.a,-self.a*lo-self.b)}
}
fn row(d1:f64,d2:f64,lo:f64,hi:f64)->[i128;4] {
    let v=[d1,d2,lo,hi].map(Dyadic::of);
    let e=v.iter().filter(|v|v.n!=0).map(|v|v.e).min().unwrap_or(0);
    v.map(|v|if v.n==0 {0} else {v.n.checked_shl((v.e-e) as u32).expect("native coordinate integer width")})
}
#[derive(Default,Clone,Copy,Debug)]
pub struct LatticeCost {pub envelope_intervals:u64,pub quotient_queries:u64}
fn feasible(rows:&[[i128;4];2],cap:u32,cost:&mut LatticeCost,witness:bool)->Option<(u32,u32)> {
    cost.quotient_queries+=1;
    let mut lower=vec![Line {a:0,b:0,d:1}];let mut upper=vec![Line {a:-1,b:cap as i128,d:1}];
    let (mut start,mut end)=(0i128,cap as i128);
    for &[a,b,lo,hi] in rows {
        if a==0 {
            if b==0 {if lo>0 || hi<0 {return None;}}
            else if b>0 {start=start.max(-(-lo).div_euclid(b));end=end.min(hi.div_euclid(b));}
            else {start=start.max(-hi.div_euclid(-b));end=end.min((-lo).div_euclid(-b));}
        } else if a>0 {lower.push(Line {a:-b,b:lo,d:a});upper.push(Line {a:-b,b:hi,d:a});}
        else {lower.push(Line {a:b,b:-hi,d:-a});upper.push(Line {a:b,b:-lo,d:-a});}
    }
    if start>end {return None;}
    let all:Vec<_>=lower.iter().chain(&upper).copied().collect();let mut cuts=vec![start,end+1];
    for (i,&a) in all.iter().enumerate() {for &b in &all[i+1..] {
        let numerator=b.b.checked_mul(a.d).and_then(|v|a.b.checked_mul(b.d).and_then(|w|v.checked_sub(w)));
        let denominator=a.a.checked_mul(b.d).and_then(|v|b.a.checked_mul(a.d).and_then(|w|v.checked_sub(w)));
        if let (Some(n),Some(d))=(numerator,denominator) {
            if d!=0 {let q=if d>0 {n.div_euclid(d)} else {(-n).div_euclid(-d)};
                for cut in [q,q+1] {if cut>start && cut<=end {cuts.push(cut);}}}
            continue;
        }
        for strict in [false,true] {
            let predicate=|q|if strict {a.cmp(b,q).is_lt()} else {a.cmp(b,q).is_le()};
            let initial=predicate(start);if initial==predicate(end) {continue;}
            let (mut lo,mut hi)=(start,end);
            while lo<hi {let mid=lo+(hi-lo)/2;if predicate(mid)==initial {lo=mid+1;} else {hi=mid;}}
            cuts.push(lo);
        }
    }}
    cuts.sort_unstable();cuts.dedup();
    // At the minimum cap all points have q1+q2=cap. Greatest q2 is canonical
    // smallest q1; selecting it uses a suffix floor-sum, never q enumeration.
    for pair in cuts.windows(2).rev() {
        cost.envelope_intervals+=1;
        let (lo,hi)=(pair[0],pair[1]-1);
        let low=*lower.iter().max_by(|&&a,&&b|a.cmp(b,lo)).unwrap();
        let high=*upper.iter().min_by(|&&a,&&b|a.cmp(b,lo)).unwrap();
        if high.cmp(low,lo).is_lt() {continue;}
        let count=|l,h|high.sum_floor(l,h)-low.sum_ceil(l,h)+(h-l+1);
        if count(lo,hi)==0 {continue;}
        if !witness {return Some((0,0));}
        let (mut l,mut h)=(lo,hi);
        while l<h {let mid=l+(h-l+1)/2;if count(mid,hi)>0 {l=mid;} else {h=mid-1;}}
        return Some((low.ceil(l).try_into().unwrap(),l.try_into().unwrap()));
    }None
}
/// Minimum total cycle count inside an exact dyadic rectangle, then (q1,q2).
/// The runtime depends on operand bits and envelope intersections, not distance.
pub fn counts(d1:Vec3,d2:Vec3,lo:Vec3,hi:Vec3,cap:u32)->Option<([u32;2],LatticeCost)> {
    let rows=[row(d1.x,d2.x,lo.x,hi.x),row(d1.z,d2.z,lo.z,hi.z)];
    let mut cost=LatticeCost::default();feasible(&rows,cap,&mut cost,false)?;
    let (mut l,mut h)=(0,cap);
    while l<h {let mid=l+(h-l)/2;if feasible(&rows,mid,&mut cost,false).is_some() {h=mid;} else {l=mid+1;}}
    let (a,b)=feasible(&rows,l,&mut cost,true)?;Some(([a,b],cost))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn integer_rectangle_matches_enumeration() {
        for d in [(3.0,1.0,2.0,3.0),(4.0,0.0,4.0,1.0),(-3.0,1.0,-2.0,3.0)] {
            for x in -8..=12 {for z in 0..=12 {
                let a=Vec3::new(d.0,0.0,d.1);let b=Vec3::new(d.2,0.0,d.3);
                let lo=Vec3::new(x as f64,0.0,z as f64);let hi=lo.plus(Vec3::new(2.0,0.0,2.0));
                let oracle=(0u32..=12).flat_map(|q1|(0u32..=12-q1).map(move |q2|(q1,q2))).filter(|&(q1,q2)| {
                    let p=Vec3::new(q1 as f64*d.0+q2 as f64*d.2,0.0,q1 as f64*d.1+q2 as f64*d.3);
                    p.x>=lo.x && p.x<=hi.x && p.z>=lo.z && p.z<=hi.z
                }).min_by_key(|&(a,b)|(a+b,a,b));
                assert_eq!(counts(a,b,lo,hi,12).map(|(q,_)|(q[0],q[1])),oracle,"d={d:?} x={x} z={z}");
            }}
        }
    }
}
