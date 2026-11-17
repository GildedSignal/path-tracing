use law::{Input, PlayerState, World};
use law::state_reads::StateField as F;

fn ordered(x: f64) -> u64 { let b = x.to_bits(); if b >> 63 != 0 { !b } else { b ^ (1 << 63) } }
fn unordered(b: u64) -> f64 { f64::from_bits(if b >> 63 != 0 { b ^ (1 << 63) } else { !b }) }

fn phase(yaw:f32)->PlayerState {
    let source=plan::source::GeneratorSuperflatSource::new(63,law::Block::full(),[80,60,-10],[400,100,10]);
    let held=Input {forward:true,sprint:true,jump:true,yaw,..Default::default()};
    let mut warm=plan::settle(&plan::Still(&source),[90.5,64.0,0.5],0).unwrap();
    let mut phase=warm.clone();
    for _ in 0..600 {law::try_tick(&mut warm,&source,&held).unwrap();if warm.on_ground {phase=warm.clone();}}
    phase
}
fn fixture(step:bool,yaw:f32,z0:f64)->(law::BoxWorld,PlayerState,Input) {
    let held=Input {forward:true,sprint:true,jump:true,yaw,..Default::default()};
    let mut phase=phase(yaw);
    let mut w=law::BoxWorld::default();w.difficulty=0;
    w.fill([90,60,-5],[120,63,5],law::Block::full());
    if step {w.fill([100,64,-5],[100,64,5],law::Block::full());}
    else {w.fill([100,63,-5],[100,63,5],law::Block::default());}
    if std::env::var("WALL2").is_ok() {w.fill([115,64,-5],[115,66,5],law::Block::full());}
    phase.position.x=95.5;phase.position.z=z0;phase.main_supporting_block=Some([95,63,z0.floor() as i32]);
    (w,phase,held)
}
// Branch of the x-axis collision at a tick: 0 free, 1 clamped (moved.x != 0 and != wanted.x), 2 snapped (moved.x == 0 != wanted.x)
fn branch(wa:law::Vec3,mo:law::Vec3)->u8 { if wa.x==mo.x {0} else if mo.x==0.0 {2} else {1} }
// Law outcome letter on the free axis: 0 free, 1 clamp (writes plane-r; includes the silent clamp and the delta==0 tie), 2 hold (the 1e-7 rule zeroed a nonzero remainder)
fn letter(bx:f64,wa:law::Vec3,mo:law::Vec3)->u8 { if wa.x==mo.x {0} else if mo.x!=0.0 {1} else { let e=bx+(0.6f32/2.0) as f64; if e==e.round() {1} else {2} } }

#[derive(Clone,PartialEq,Eq,Debug)]
struct Key { sig: Vec<u128>, br: Vec<u8> }
struct Run { key: Key, pin: Option<PlayerState>, pin_tick: u32, before_pin_x: f64, exit: PlayerState, t: u32 }

fn run(w:&impl World,start:&PlayerState,x:f64,input:&Input,step:bool,use_branch:bool)->Run {
    let mask=1u128<<(F::minor_horizontal_collision as u8);
    let mut s=start.clone(); s.position.x=x; let mut sig=Vec::new(); let mut br=Vec::new();
    let mut pin=None; let mut pin_tick=0; let mut bpx=0.0;
    for t in 1..=80u32 {
        let before=s.clone();
        let c=law::tick_with_collision(&mut s,w,input).unwrap_or_default();
        if pin.is_none() {
            sig.push(words::segment::joint_events(&before,&s).0 & !mask);
            br.push(if use_branch { if std::env::var("MINMAP").is_ok() { law_hold(w,&before,c.0,c.1) as u8 * 2 } else { letter(before.position.x,c.0,c.1) } } else {0});
            if s.horizontal_collision { pin=Some(s.clone()); pin_tick=t; bpx=before.position.x; }
        }
        let stop = if step { s.on_ground && t>1 && (s.position.x>100.0 || pin.is_some()) } else { s.on_ground && s.position.x>100.0 };
        if stop { return Run{key:Key{sig,br},pin,pin_tick,before_pin_x:bpx,exit:s,t}; }
    }
    Run{key:Key{sig,br},pin,pin_tick,before_pin_x:bpx,exit:s,t:80}
}
fn all_cuts<K: PartialEq + Clone>(f: &mut impl FnMut(u64) -> K, a: u64, ka: K, b: u64, kb: K, out: &mut Vec<u64>) {
    if ka == kb { return; }
    if b - a <= 1 { out.push(b); return; }
    let m = a + (b - a) / 2; let km = f(m);
    all_cuts(f, a, ka, m, km.clone(), out); all_cuts(f, m, km, b, kb, out);
}
fn pieces(w:&impl World,start:&PlayerState,input:&Input,step:bool,lo:f64,hi:f64,k:u32,ub:bool)->Vec<(u64,u64)> {
    let n=1u64<<k; let pts:Vec<u64>=(0..=n).map(|i|ordered(lo+(hi-lo)*i as f64/n as f64)).collect();
    let mut f=|p:u64| run(w,start,unordered(p),input,step,ub).key;
    let keys:Vec<Key>=pts.iter().map(|&p|f(p)).collect(); let mut cuts=Vec::new();
    for i in 0..n as usize { all_cuts(&mut f,pts[i],keys[i].clone(),pts[i+1],keys[i+1].clone(),&mut cuts); }
    let mut b=vec![pts[0]]; b.extend(cuts.iter().copied()); b.push(pts[n as usize]+1);
    (0..b.len()-1).map(|j|(b[j],b[j+1]-1)).collect()
}
fn hash(v:&[u64])->u64 { let mut h:u64=1469598103934665603; for x in v {h=(h^x).wrapping_mul(1099511628211);} h }

fn main() {
    let mode=std::env::args().nth(1).unwrap_or_default();
    let yaw:f32=std::env::var("YAW").ok().and_then(|v|v.parse().ok()).unwrap_or(275.625);
    let z0:f64=std::env::var("Z0").ok().and_then(|v|v.parse().ok()).unwrap_or(0.5);
    let step=std::env::var("WORLD").as_deref()==Ok("step");
    let k:u32=std::env::var("K").ok().and_then(|v|v.parse().ok()).unwrap_or(12);
    let ub=std::env::var("NOBRANCH").is_err();
    if mode=="disc" {
        let source=plan::source::GeneratorSuperflatSource::new(63,law::Block::full(),[-2000,60,-2000],[2000,100,2000]);
        let mut rows=Vec::new();
        for i in 0..64 { let yaw=5.625f32*i as f32; let r=(yaw*0.017453292f32) as f64; let s=law::math::sin(r) as f64; let c=law::math::cos(r) as f64;
            let held=Input {forward:true,sprint:true,jump:true,yaw,..Default::default()};
            let mut st=plan::settle(&plan::Still(&source),[0.5,64.0,0.5],0).unwrap(); let mut lands=Vec::new();
            for t in 0..2400u32 { let g=st.on_ground; law::try_tick(&mut st,&source,&held).unwrap(); if st.on_ground && !g { lands.push((t,st.position)); } }
            let n=lands.len(); let (t0,p0)=lands[n-41]; let (t1,p1)=lands[n-1]; let dx=p1.x-p0.x; let dz=p1.z-p0.z; let sp=(dx*dx+dz*dz).sqrt()/(t1-t0) as f64;
            rows.push((yaw,s*s+c*c-1.0,sp,(t1-t0)/40)); }
        let base=rows[0].2; for (yaw,e,sp,per) in &rows { println!("DISC yaw={yaw} table_norm2_minus_1={e:+.3e} speed_blocks_per_tick={sp:.12} rel_to_yaw0={:+.3e} cycle_ticks={per}", sp/base-1.0); }
        return;
    }
    if mode=="res" {
        for (step,yaw,z0) in [(false,270.0f32,0.5),(false,275.625,0.5),(false,270.0,1.0),(true,270.0,0.5),(true,275.625,0.5)] { let (w,start,input)=fixture(step,yaw,z0);
            let v:Vec<usize>=[6u32,8,10,12,14,16].iter().map(|&k| pieces(&w,&start,&input,step,95.5,99.5,k,ub).len()).collect();
            let ps=pieces(&w,&start,&input,step,95.5,99.5,16,ub); let mut img=std::collections::BTreeSet::new(); let mut fam=0;
            for &(a,b) in &ps { let ra=run(&w,&start,unordered(a),&input,step,ub); let rb=run(&w,&start,unordered(b),&input,step,ub);
                if let (Some(p),Some(q))=(&ra.pin,&rb.pin) { if p.position.x==q.position.x { img.insert((ra.pin_tick,p.position.x.to_bits(),p.minor_horizontal_collision,q.minor_horizontal_collision)); } else { fam+=1; } } }
            println!("RES world={} yaw={yaw} z0={z0} pieces_at_2^6..2^16={v:?} pin_point_pieces={} pin_hold_families={fam}", if step {"step"} else {"gap"}, img.len()); }
        return;
    }
    if mode=="sweep" {
        let t0=std::time::Instant::now();
        for i in 0..=12 { let yaw=236.25f32+5.625*i as f32;
            for step in [false,true] {
            let (w,start,input)=fixture(step,yaw,z0);
            let ps=pieces(&w,&start,&input,step,95.5,99.5,k,ub);
            let mut snaps=0; let mut shell_vals=Vec::new(); let mut clamp_bad=0;
            for &(a,b) in &ps { let ra=run(&w,&start,unordered(a),&input,step,ub); let rb=run(&w,&start,unordered(b),&input,step,ub);
                if let (Some(pa),Some(pb))=(&ra.pin,&rb.pin) { let last=*ra.key.br.last().unwrap();
                    if last==2 { snaps+=1; shell_vals.push(ordered(pb.position.x)-ordered(pa.position.x)+1); }
                    else if pa.position.x!=pb.position.x { clamp_bad+=1; } } }
            println!("SWEEP world={} yaw={yaw} z0={z0} samples=2^{k} pieces={} snap_pieces={snaps} shell_lattice_values={shell_vals:?} other_pin_pieces_with_unequal_pin_x={clamp_bad}", if step {"step"} else {"gap"}, ps.len());
        }}
        println!("elapsed {:?}",t0.elapsed());
        return;
    }
    let (w,start,input)=fixture(step,yaw,z0);
    let ps=pieces(&w,&start,&input,step,95.5,99.5,k,ub);
    println!("FIXTURE world={} yaw={yaw} z0={z0} samples=2^{k} branch_in_key={ub} pieces={}", if step {"step"} else {"gap"}, ps.len());
    let wall=101.0-(0.6f32/2.0) as f64;
    for (j,&(a,b)) in ps.iter().enumerate() {
        let ra=run(&w,&start,unordered(a),&input,step,ub); let rb=run(&w,&start,unordered(b),&input,step,ub);
        let desc = match (&ra.pin,&rb.pin) { (Some(p),Some(q))=> {
            format!("pin_tick={} branch={:?} pin_x_lo={:x} pin_x_hi={:x} pin_lattice_span={} before_pin_x_lo={:.17} hi={:.17} W={:.17} W-before_lo={:e} W-before_hi={:e} minor={},{}", ra.pin_tick, ra.key.br.last().unwrap(), p.position.x.to_bits(), q.position.x.to_bits(), ordered(q.position.x) as i64-ordered(p.position.x) as i64+1, ra.before_pin_x, rb.before_pin_x, wall, wall-ra.before_pin_x, wall-rb.before_pin_x, p.minor_horizontal_collision, q.minor_horizontal_collision) }, _=>"free".to_string() };
        println!("P{j:02} lo={:x} hi={:x} entry_ulps={} T={} {desc}", unordered(a).to_bits(), unordered(b).to_bits(), b-a+1, ra.t);
        if mode=="mono" {
            let mut xs=Vec::new(); let n=2048u64;
            for i in 0..=n { xs.push(a+(b-a)/n*i); }
            for d in 0..256 { if a+d<=b {xs.push(a+d);} if b>=d && b-d>=a {xs.push(b-d);} }
            xs.sort(); xs.dedup(); let mut prev:Option<u64>=None; let mut dec=0; let mut keybad=0; let mut ident_bad=0;
            let mut distinct=std::collections::BTreeSet::new(); let mut distinct_pin=std::collections::BTreeSet::new();
            for &p in &xs { let r=run(&w,&start,unordered(p),&input,step,ub); if r.key!=ra.key {keybad+=1;}
                let ex=ordered(r.exit.position.x); distinct.insert(ex);
                if let Some(q)=&r.pin { distinct_pin.insert(q.position.x.to_bits()); }
                if let Some(pp)=prev { if ex<pp {dec+=1;} } prev=Some(ex);
                if *ra.key.br.last().unwrap()==2 && r.pin.as_ref().map(|q|q.position.x)!=Some(r.before_pin_x) { ident_bad+=1; } }
            println!("    MONO samples={} key_mismatch={keybad} exit_x_decreases={dec} distinct_exit_x={} distinct_pin_x={} snap_identity_violations={ident_bad}", xs.len(), distinct.len(), distinct_pin.len());
        }
    }
    if mode=="merge" {
        let mut all=Vec::new();
        for &(a,b) in &ps { let r=run(&w,&start,unordered(a),&input,step,ub); if r.pin.is_none() {continue;} let n=64u64; for i in 0..=n { all.push(a+(b-a)/n*i); } }
        for k in [0u32,10,20,40,60,80] { let mut set=std::collections::HashSet::new(); let mut setx=std::collections::HashSet::new();
            for &p in &all { let r=run(&w,&start,unordered(p),&input,step,ub); let mut s=r.exit.clone(); s.t=0; for _ in 0..k { law::try_tick(&mut s,&w,&input).unwrap(); }
                let mut q=s.clone(); q.t=0; q.exhaustion=0.0; q.saturation=0.0; q.food_level=0; q.fall_distance=0.0; setx.insert(q.position.x.to_bits()); set.insert(q.bits()); }
            println!("MERGE after_exit_ticks={k} pinned_samples={} distinct_states={} distinct_x={}", all.len(), set.len(), setx.len()); }
        return;
    }
    if mode=="down" {
        let mut reps=Vec::new();
        for &(a,b) in &ps { let r=run(&w,&start,unordered(a),&input,step,ub); if r.pin.is_some() {
            for p in [a,a+(b-a)/2,b] { reps.push((p,run(&w,&start,unordered(p),&input,step,ub))); } } }
        let key_minus_x=|s:&PlayerState|{ let mut q=s.clone(); q.position.x=0.0; q.exhaustion=0.0; q.saturation=0.0; q.food_level=0; q.fall_distance=0.0; q.main_supporting_block=None; q.t=0; q.bits() };
        for (p,r) in &reps {
            let mut s=r.exit.clone(); let mut evs=Vec::new(); let mut land=Vec::new(); let mut snaps=Vec::new();
            for t in 1..=120u32 { let b=s.clone(); let c=law::tick_with_collision(&mut s,&w,&input).unwrap_or_default(); if s.on_ground && !b.on_ground { land.push(t);} if s.horizontal_collision {evs.push((t,branch(c.0,c.1)));} if branch(c.0,c.1)==2 {snaps.push(t);} }
            println!("DOWN entry={:x} pin_tick={} br={} exit_T={} exit_x={:.17} after120_x={:.17} after120_minus_W={:e} hc={:?} landings={:?} state_minus_x={:x}", unordered(*p).to_bits(), r.pin_tick, r.key.br.last().unwrap(), r.t, r.exit.position.x, s.position.x, s.position.x-wall, evs, land, hash(&key_minus_x(&s)));
        }
    }
}

// The law's own selector for the hold: on the x pass, a shape clamps the remainder below 1e-7
// and at least one more shape is visited after it (Shapes.collide zero rule).
fn law_hold(w:&impl World,before:&PlayerState,wa:law::Vec3,mo:law::Vec3)->bool {
    if wa.x==mo.x || mo.x!=0.0 { return false; }
    let b=before.bounding_box(); let ctx=law::CollisionContext::of(before);
    let moved=b.moved(law::Vec3::new(0.0,mo.y,0.0));
    let mut d=wa.x; let mut clamped=false; let mut later=false;
    w.visit_collisions(b.expand(wa),ctx,|shape| { if clamped { later=true; return false; } let nd=shape.collide(law::Axis::X,moved,d); if nd.abs()<1.0e-7 { clamped=true; } d=nd; true });
    clamped && later
}
