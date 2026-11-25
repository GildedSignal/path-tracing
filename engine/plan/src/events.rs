//! Exact binary64 advance for a repeated gait.
/// A sign-definite cycle stays within its binade whenever both ends do.
/// Sample its two mantissa parities with native additions, then translate in
/// same mantissa chart. A tie fixes parity; without a
/// tie the cycle translates both parities equally.
pub(crate) fn advance_cycles(axis:usize,start:f64,ds:&[f64],mut count:u64)->f64 {
    let cycle=|p|ds.iter().fold(p,|p,d|p+d);
    let mut p=start;
    while count>0 {
        let first=cycle(p);
        count-=1;
        if count==0 {p=first;break;}
        let second=cycle(first);
        count-=1;
        let bits=second.to_bits();
        let step=[first,p].into_iter().enumerate().find_map(|(i,prior)| {
            let prior=prior.to_bits();
            ((prior^bits)>>52==0 && (prior^bits)&1==0).then(|| (i as u64+1,bits as i64-prior as i64))
        });
        p=second;
        if let Some((period,delta))=step {
            if delta==0 {if period==2 && count%2!=0 {p=cycle(p);}break;}
            const MANTISSA: u64 = (1<<52)-1;
            let k=bits&MANTISSA;
            let room=if delta>0 {MANTISSA-k} else {k.saturating_sub(1)};
            let n=(count/period).min(room/delta.unsigned_abs());
            p=f64::from_bits(bits.wrapping_add_signed(n as i64*delta));
            count-=n*period;
        }
    }
    #[cfg(feature="solve-counters")] crate::solve_meter::walk(Some(axis),start,p);
    let _=axis;
    p
}
#[derive(Clone,Debug,PartialEq,Eq)]
pub enum EventKind {BinadeCrossing,FoodGuard {sprint_allowed:bool},LoadedBoundary,ZoneLabelBoundary,YawSwitch,GoalBoundary}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Event {pub tick:u64,pub kind:EventKind}


#[cfg(test)]
mod tests {
    use super::advance_cycles;

    #[test]
    fn advance_cycles_matches_native_additions() {
        let tiny = f64::from_bits(1);
        let normal = f64::MIN_POSITIVE;
        let half_ulp = f64::EPSILON / 2.0;
        let mut checks = 0;
        for (starts, increments) in [
            (&[-1024.25, -0.5, -0.0, 0.0, 1023.75][..], &[0.25, 0.3][..]),
            (&[-1023.75, -0.0, 0.0, 0.5, 1024.25][..], &[-0.4, -0.3][..]),
            (&[-0.0, 0.0][..], &[0.0, -0.0][..]),
            (&[-0.0, 0.0][..], &[-0.0][..]),
            (&[-normal, -tiny, 0.0, normal - tiny][..], &[tiny, 2.0 * tiny][..]),
            (&[-normal + tiny, -0.0, tiny, normal][..], &[-tiny][..]),
            (&[1.0, 1.0 + f64::EPSILON, 2.0 - f64::EPSILON][..], &[half_ulp][..]),
            (&[1.0, 1.0 + f64::EPSILON, 2.0 - f64::EPSILON][..], &[3.0 * half_ulp][..]),
            (&[-2.0 + f64::EPSILON, -1.0 - f64::EPSILON, -1.0][..], &[-3.0 * half_ulp][..]),
            (&[1_048_575.75, -1_048_576.25][..], &[0.25][..]),
        ] {
            for &start in starts {
                for count in [0, 1, 2, 3, 17, 263, 10_003] {
                    let mut native = start;
                    for _ in 0..count {
                        for &increment in increments {
                            native += increment;
                        }
                    }
                    assert_eq!(
                        advance_cycles(0, start, increments, count).to_bits(),
                        native.to_bits(),
                        "start={start:?}, increments={increments:?}, count={count}"
                    );
                    checks += 1;
                }
            }
        }
        println!("CYCLE_FOLD cases={checks} worst_error_ulp=0");
    }
}
