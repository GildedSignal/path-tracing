//! Independent single-yaw exclusion certificate for P3 (release reference gate).
//! From rest, positive drag/input/jump factors preserve each heading component's
//! sign. Only the positive quadrant can reach the positive x/z goal. For those
//! headings, native ticks provide the complete prefix and one exact recurrence.
//! Every future increment is in that finite record. A coarsest-binade dyadic
//! enclosure includes every subsequent RN position addition. Summing vectors
//! preserves its slope cone. If that cone misses even the enlarged detector
//! rectangle, no tick/count of this single yaw can touch the goal.
use law::{PlayerState,Vec3,World,codec, state_reads::{StateField as F,StateReads}};
use words::{Arguments,Program};

fn identity(s:&PlayerState)->Vec<u64> {
    let mut out=vec![s.position.y.to_bits()];
    for &f in F::ALL {match f {
        F::t|F::position|F::main_supporting_block|F::food_level|F::health|F::saturation|F::exhaustion|F::food_tick_timer=>(),
        _=>s.push_field_bits(f,&mut out),
    }}
    out.extend_from_slice(&codec::state_answers(s,StateReads((1u128<<F::food_level as u8)|(1u128<<F::health as u8))));out
}
fn units(x:f64,exp:i32)->i128 {
    assert!(x>=0.0);let b=super::t2::Binary64::of(x);if b.mantissa==0 {return 0;}
    let shift=b.exp-exp;
    if shift>=0 {(b.mantissa as i128)<<shift}
    else if -shift>=128 {0} else {(b.mantissa as i128)>>(-shift)}
}
pub(super) fn all_native_yaws_miss(world:&impl World,start:&PlayerState,program:&Program,period:usize) {
    assert!(!program.timed());assert_eq!(world.difficulty(),0);
    let goal=plan::Goal {cell:[1_000_000,-61,330_000]};let zone=goal.zone();
    let r=start.bounding_box().max.minus(start.position);let c=plan::goal::C;
    let max=Vec3::new(((zone.max.x+r.x).next_up()+c).next_up(),0.0,((zone.max.z+r.z).next_up()+c).next_up());
    let ex=super::t2::Binary64::of(max.x).exp;let ez=super::t2::Binary64::of(max.z).exp;
    let xmin=units(((zone.min.x-r.x).next_down()-start.position.x).next_down(),ex)-1;
    let zmin=units(((zone.min.z-r.z).next_down()-start.position.z).next_down(),ez)-1;
    let xmax=units((max.x-start.position.x).next_up(),ex)+2;
    let zmax=units((max.z-start.position.z).next_up(),ez)+2;
    let entries=include_bytes!("../../../law/src/sin.bin").len()/std::mem::size_of::<f32>();
    let mut probed=0;let mut excluded_sign=0;let mut overlaps=0;let mut total_ticks=0u64;let mut durations=Vec::new();let mut worst:Option<(i128,i128)>=None;
    for index in 0..entries {
        let yaw=(((index as f64+0.5)*std::f64::consts::TAU/entries as f64)/0.017453292f32 as f64) as f32;
        let radians=(yaw*0.017453292f32) as f64;
        if -law::math::sin(radians)<=0.0 || law::math::cos(radians)<=0.0 {excluded_sign+=1;continue;}
        probed+=1;let args=Arguments {refined_yaw:Some(yaw),..Default::default()};
        let mut state=start.clone();let mut last=vec![None;period];last[0]=Some(identity(start));
        let mut n=0u64;let mut min_slope:Option<(i128,i128)>=None;let mut max_slope:Option<(i128,i128)>=None;
        loop {
            let input=program.select(world,&state,n as u32,args).unwrap().1;
            let (_,moved)=law::tick_with_collision(&mut state,world,&input).unwrap();n+=1;
            assert!(moved.x>=0.0 && moved.z>=0.0);assert!(moved.horizontal_sqr()<=c*c);
            // One full coarsest lattice unit on either side encloses RN at
            // every finer binade as well, including a boundary addition.
            let lx=(units(moved.x,ex)-1).max(0);let hx=units(moved.x,ex)+2;
            let lz=(units(moved.z,ez)-1).max(0);let hz=units(moved.z,ez)+2;
            assert!(lx>0 && lz>0);
            if min_slope.is_none_or(|(a,b)|lz*b<a*hx) {min_slope=Some((lz,hx));}
            if max_slope.is_none_or(|(a,b)|hz*b>a*lx) {max_slope=Some((hz,lx));}
            let key=identity(&state);let slot=n as usize%period;
            if last[slot].as_ref()==Some(&key) {break;}
            last[slot]=Some(key);
        }
        total_ticks+=n;durations.push(n);
        let (a,b)=min_slope.unwrap();let (u,v)=max_slope.unwrap();
        let above=a*xmin-zmax*b;let below=zmin*v-u*xmax;
        let gap=if above>0 {Some((above,b))} else if below>0 {Some((below,v))} else {None};
        if let Some((n,d))=gap {
            if worst.is_none_or(|(a,b)|n*b<a*d) {worst=Some((n,d));}
        } else {overlaps+=1;}
    }
    assert_eq!(probed+excluded_sign,entries);assert_eq!(overlaps,0,"single-yaw horizontal cone can reach the goal");
    durations.sort_unstable();let (num,den)=worst.unwrap();
    println!("T3_SINGLE_YAW domain={entries} native_yaws={probed} sign_excluded={excluded_sign} cone_overlaps={overlaps} law_ticks={total_ticks} ticks_min={} ticks_median={} ticks_p99={} ticks_max={} gap_num={num} gap_den={den} gap_unit_exp={ez}",
        durations[0],durations[durations.len()/2],durations[(99*durations.len()).div_ceil(100)-1],durations[durations.len()-1]);
}
