//! A second, instrumented run uses the same solve. The normal build contains
//! no phase set, counter increments, thread-local lookup or profiling clock.
use std::{cell::RefCell,collections::HashSet};

#[derive(Default,Debug)]
pub struct Work {
    pub walks: [u64;2],
    pub full_walks: [u64;2],
    pub probes: u64,
    pub distinct_phases: usize,
}
struct Meter {work:Work, phases:HashSet<usize>, ends:[[u64;2];2]}
thread_local! {static METER:RefCell<Option<Meter>>=const {RefCell::new(None)};}

fn chart(p:f64)->u64 {p.to_bits()>>52}
pub fn begin(start:law::Vec3,goal:law::Vec3) {
    METER.with_borrow_mut(|m|*m=Some(Meter {work:Work::default(),phases:HashSet::new(),
        ends:[[chart(start.x),chart(goal.x)],[chart(start.z),chart(goal.z)]]}));
    law::solve_meter::reset();
}
pub fn finish()->Work {
    let mut m=METER.with_borrow_mut(Option::take).unwrap();
    m.work.probes=law::solve_meter::probes();
    m.work.distinct_phases=m.phases.len();
    m.work
}
pub(crate) fn phase<T>(state:&T) {
    METER.with_borrow_mut(|m|if let Some(m)=m {m.phases.insert(std::ptr::from_ref(state) as usize);});
}
pub(crate) fn walk(axis:Option<usize>,start:f64,end:f64) {
    let Some(a)=axis else{return};
    let ends=[chart(start),chart(end)];
    if ends[0]==ends[1] {return;}
    METER.with_borrow_mut(|m|if let Some(m)=m {
        m.work.walks[a]+=1;
        if ends==m.ends[a] {m.work.full_walks[a]+=1;}
    });
}
