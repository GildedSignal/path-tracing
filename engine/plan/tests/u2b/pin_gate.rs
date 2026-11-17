use super::*;
use law::contact::{AxisContact,Outcome};
use super::u2b_premises::{fixture};
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct NativeKey {pub events:Vec<u128>,pub outcomes:Vec<u8>}
pub struct NativeRun {pub key:NativeKey,pub pin:Option<(PlayerState,PlayerState,AxisContact)>,pub end:PlayerState}
pub fn run(w:&impl World,start:&PlayerState,x:f64,held:&Input,kind:u8,raw:bool)->NativeRun {
    let mask=law::state_reads::NON_MONOTONE_FIELDS.iter().fold(0u128,|m,(f,_)|m|1u128<<*f as u8);
    let mut state=start.clone();state.position.x=x;let mut events=Vec::new();let mut outcomes=Vec::new();let mut pin=None;
    for tick in 1..=80 {
        let before=state.clone();let (_,_,passes)=law::contact::tick(&mut state,w,held).unwrap();
        let contact=*passes.iter().find(|p|p.axis==law::Axis::X).unwrap();
        if pin.is_none() {
            events.push(words::segment::joint_events(&before,&state).0 & !mask);
            // The proposal's MINMAP object treats a silent clamp as min(P2,W)
            // inside a free-motion piece. Also expose the literal three-letter
            // trace so this projection is explicit and can be reviewed.
            outcomes.push(if raw {contact.outcome.kind()} else {if matches!(contact.outcome,Outcome::Hold {..}) {2} else if state.horizontal_collision {1} else {0}});
            if state.horizontal_collision {pin=Some((before,state.clone(),contact));}
        }
        let stop=match kind {
            0=>state.on_ground && state.position.x>100.0,
            1=>state.on_ground && tick>1 && (state.position.x>100.0 || pin.is_some()),
            2=>state.main_supporting_block.is_some_and(|p|p[0]>=100 && p[1]==63),
            _=>state.on_ground && tick>1,
        };
        if stop {return NativeRun {key:NativeKey {events,outcomes},pin,end:state};}
    }
    panic!("native contact oracle did not reach exit")
}
fn all_cuts(f:&mut impl FnMut(u64)->NativeKey,a:u64,ka:NativeKey,b:u64,kb:NativeKey,out:&mut Vec<u64>) {
    if ka==kb {return;}if b-a==1 {out.push(b);return;}
    let m=a+(b-a)/2;let km=f(m);all_cuts(f,a,ka,m,km.clone(),out);all_cuts(f,m,km,b,kb,out);
}
pub fn oracle(w:&impl World,start:&PlayerState,input:&Input,kind:u8,lo:f64,hi:f64,power:u32,raw:bool)->Vec<u64> {
    let mut f=|bits|run(w,start,unordered(bits),input,kind,raw).key;
    let n=1u64<<power;let mut cuts=Vec::new();let mut a=ordered(lo);let mut ka=f(a);
    for i in 1..=n {
        let b=ordered(lo+(hi-lo)*i as f64/n as f64);let kb=f(b);
        all_cuts(&mut f,a,ka,b,kb.clone(),&mut cuts);a=b;ka=kb;
    }cuts
}
pub fn fixtures()->Vec<(&'static str,Scene,PlayerState,Input,f64,f64,usize,u8)> {
    let mut cases=Vec::new();
    for (step,yaw,z,expected) in [(false,270.0,0.5,5),(false,275.625,0.5,7),(false,270.0,1.0,7),(true,270.0,0.5,8),(true,275.625,0.5,12)] {
        let (w,mut start,input)=fixture(step,yaw);start.position.z=z;start.main_supporting_block=Some([95,63,z.floor() as i32]);
        cases.push((if step {"step"} else {"gap"},Scene(w),start,input,95.5,99.5,expected,if step {1} else {0}));
    }
    let (mut ice,start,input)=fixture(false,270.0);ice.fill([100,63,-5],[120,63,5],block("ice"));
    cases.push(("glm-ice",Scene(ice),start,input,95.5,99.5,2,2));
    let w=scene();let mut start=plan::settle(&plan::Still(&w),[-86.5,-28.0,-101.5],0).unwrap();start.velocity.x=0.24;start.velocity.z=0.06;
    cases.push(("hardtrack-lane-wall",w,start,Input {forward:true,sprint:true,yaw:0.0,..Default::default()},-86.75,-86.25,2,3));cases
}
#[test]
fn u2_contact_oracle_before_freeze() {
    for (name,w,start,input,lo,hi,_,kind) in fixtures() {
        for raw in [false,true] {for power in [6,10,18] {
            let cuts=oracle(&w,&start,&input,kind,lo,hi,power,raw);
            println!("S203_ORACLE pattern={name} yaw={} z={} raw={raw} power={power} pieces={} cuts={cuts:x?}",input.yaw,start.position.z,cuts.len()+1);
        }}
        for bits in [ordered(lo),ordered(hi)] {
            let mut a=start.clone();a.position.x=unordered(bits);let mut b=a.clone();
            for _ in 0..24 {law::try_tick(&mut a,&w,&input).unwrap();law::contact::tick(&mut b,&w,&input).unwrap();assert_eq!(a.bits(),b.bits());}
        }
    }
}
