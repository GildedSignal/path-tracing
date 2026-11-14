//! Points forward, then the slide (job 3.2b step 5; Julien, S94: "some elements have wiggle
//! room more than others so can we wiggle the path as we go"). T in ticks is a step function,
//! so this is no gradient: each joint of a certified sentence is slid inside its set, by one
//! tick of its step or one aim of the grid, and the whole sentence is replayed. A slide is
//! kept only when the sentence's T falls. Tightest joints first: a joint is tight when few of
//! its slides still reach the goal.
use law::{BlockPos, PlayerState};
use words::Program;
use crate::{Certificate, Goal, Step, Timeline, certify};

#[derive(Clone, Debug, Default)]
pub struct Slide {
    pub replays: u64,
    pub replayed_ticks: u64,
    pub kept: u32,
    pub passes: u32,
    pub before: u32,
    pub after: u32,
    /// Per joint, at the first pass: how many of its slides still reached the goal.
    pub room: Vec<u8>,
    pub seconds: f64,
}
fn slides(step: Step) -> Vec<Option<Step>> {
    let aim = |by: u8| Some(Step { args: words::Arguments { yaw_index: (step.args.yaw_index + by) % 64, ..step.args }, ..step });
    vec![if step.ticks > 1 { Some(Step { ticks: step.ticks - 1, ..step }) } else { None }, Some(Step { ticks: step.ticks + 1, ..step }), aim(1), aim(63)]
}
pub fn slide<T: Timeline>(timeline: &T, programs: &[Program], start: &PlayerState, goal: &Goal, a: Option<BlockPos>, sentence: (Vec<Step>, Certificate),
    stop: &mut dyn FnMut() -> bool) -> ((Vec<Step>, Certificate), Slide) {
    let clock = std::time::Instant::now();
    let (mut steps, mut best) = sentence;
    let mut report = Slide { before: best.ticks, ..Slide::default() };
    let mut order: Option<Vec<usize>> = None;
    'passes: loop {
        report.passes += 1;
        // The room of each joint, and with it the order: tightest first.
        let mut tried: Vec<(usize, Vec<(Vec<Step>, Certificate)>)> = Vec::new();
        for joint in 0..steps.len() {
            let mut reached = Vec::new();
            for slid in slides(steps[joint]) {
                if stop() { break 'passes; }
                let mut candidate = steps.clone();
                match slid { Some(step) => candidate[joint] = step, None => { candidate.remove(joint); } }
                report.replays += 1;
                if let Ok(certificate) = certify(timeline, programs, start, &candidate, goal, a, false) {
                    report.replayed_ticks += certificate.ticks as u64;
                    reached.push((certificate.steps.clone(), certificate));
                }
            }
            tried.push((joint, reached));
        }
        if order.is_none() { report.room = tried.iter().map(|joint| joint.1.len() as u8).collect(); }
        tried.sort_by_key(|joint| joint.1.len());
        order = Some(tried.iter().map(|joint| joint.0).collect());
        // The tightest joint whose slide lowers T is slid; then every joint is looked at again.
        let Some((slid, certificate)) = tried.into_iter().flat_map(|joint| joint.1).filter(|slid| slid.1.ticks < best.ticks).next() else { break };
        (steps, best) = (slid, certificate);
        report.kept += 1;
    }
    // The returned sentence carries its tube.
    if let Ok(certificate) = certify(timeline, programs, start, &steps, goal, a, true) { best = certificate; }
    report.after = best.ticks;
    report.seconds = clock.elapsed().as_secs_f64();
    ((steps, best), report)
}
