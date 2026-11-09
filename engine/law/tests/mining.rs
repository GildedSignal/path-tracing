mod mining {
    include!("../src/mining.rs");

    // Reach int-to-float and overflow boundaries without replaying billions of ticks.
    pub fn seek(clock: &mut MiningClock, game_ticks: i32) {
        clock.game_ticks = game_ticks;
    }
}

use mining::{
    MiningAction, MiningClock, MiningRefusal, MiningSample, MiningStep, MiningTarget, MiningWork,
};
use std::{collections::BTreeMap, env, fs};

fn position(x: i32) -> [i32; 3] {
    [x, 64, -x]
}

fn target_text(target: MiningTarget) -> String {
    assert_eq!(target.position, position(target.position[0]));
    format!("{}:{}", target.position[0], target.identity)
}

fn sample(field: &str) -> Option<MiningSample> {
    if field == "-" {
        return None;
    }
    let fields: Vec<_> = field.split(':').collect();
    assert_eq!(fields.len(), 5, "{field}");
    let float = |bits| f32::from_bits(u32::from_str_radix(bits, 16).unwrap());
    Some(MiningSample {
        target: MiningTarget {
            position: position(fields[0].parse().unwrap()),
            identity: fields[1].parse().unwrap(),
        },
        destroy_speed: float(fields[2]),
        hardness: float(fields[3]),
        correct_tool: fields[4] == "1",
    })
}

fn work_text(work: Option<MiningWork>) -> String {
    work.map_or_else(
        || "-".into(),
        |work| format!("{}@{}", target_text(work.target), work.start_tick),
    )
}

fn clock_text(clock: MiningClock) -> String {
    format!(
        "{}/{}/{}",
        clock.game_ticks(),
        work_text(clock.active()),
        work_text(clock.delayed())
    )
}

fn step_text(step: Result<MiningStep, MiningRefusal>) -> String {
    match step {
        Ok(MiningStep::Idle) => "idle".into(),
        Ok(MiningStep::TargetGone(target)) => format!("gone:{}", target_text(target)),
        Ok(MiningStep::Evaluated {
            target,
            progress,
            destroy,
            ..
        }) => {
            let event = match destroy {
                Some(intent) => {
                    assert_eq!(target, intent.target);
                    "destroy"
                }
                None => "progress",
            };
            let bits = if progress.is_nan() {
                f32::NAN.to_bits()
            } else {
                progress.to_bits()
            };
            format!("{event}:{}:{bits:x}", target_text(target))
        }
        Err(MiningRefusal::RejectedInput) => "refuse:rejected".into(),
        Err(MiningRefusal::StartWhileActive) => "refuse:start-active".into(),
        Err(MiningRefusal::StartAtDelayedPosition) => "refuse:start-delayed".into(),
        Err(MiningRefusal::StopWithoutActive) => "refuse:stop-inactive".into(),
        Err(MiningRefusal::StopAtDifferentPosition) => "refuse:stop-position".into(),
        Err(MiningRefusal::TargetChanged(target)) => {
            format!("refuse:changed:{}", target_text(target))
        }
    }
}

#[test]
#[ignore = "run oracles/1.9g-mining-progress/run.ps1 against the pinned JVM oracle"]
fn mining_progress_matches_jvm() {
    let path = env::var_os("MINING_ORACLE").expect("MINING_ORACLE must name the JVM rows");
    let rows = fs::read_to_string(path).unwrap();
    let mut clock = MiningClock::default();
    let mut counts = BTreeMap::<&str, (usize, usize)>::new();
    let mut first = None;
    for (index, row) in rows.lines().enumerate() {
        let fields: Vec<_> = row.split('|').collect();
        let [_, action, arg, current, accepted, expected_state, expected_lookup, expected_step] =
            fields.as_slice()
        else {
            panic!("bad oracle row: {row}");
        };
        let action = *action;
        let arg = arg.parse().unwrap();
        let current = sample(current);
        let accepted = *accepted == "1";
        let before = clock_text(clock);
        let mut lookup = None;
        let step = match action {
            "reset" => {
                clock = MiningClock::new(arg);
                Ok(MiningStep::Idle)
            }
            "seek" => {
                mining::seek(&mut clock, arg);
                Ok(MiningStep::Idle)
            }
            "start" => {
                // Retain the frozen oracle's formerly unqualified branch.
                if accepted && clock.active().is_some() {
                    Err(MiningRefusal::StartWhileActive)
                } else {
                    clock.apply(MiningAction::Start(current.unwrap()), accepted)
                }
            },
            "stop" => clock.apply(
                MiningAction::Stop {
                    position: position(arg),
                    current,
                },
                accepted,
            ),
            "tick" => clock.tick(|target| {
                lookup = Some(target);
                current
            }),
            _ => panic!("bad oracle action: {action}"),
        };
        let state = clock_text(clock);
        let lookup = lookup.map_or_else(|| "-".into(), target_text);
        let step = step_text(step);
        let count = counts.entry(action).or_default();
        count.0 += 1;
        if (state.as_str(), lookup.as_str(), step.as_str())
            != (*expected_state, *expected_lookup, *expected_step)
        {
            count.1 += 1;
            first.get_or_insert_with(|| {
                format!(
                    "row {}\nprior Rust state: {before}\nJVM row: {row}\nRust state|lookup|step: {state}|{lookup}|{step}",
                    index + 1
                )
            });
        }
    }
    for action in ["reset", "seek", "start", "stop", "tick"] {
        let (rows, differences) = counts[action];
        assert!(rows > 0);
        println!("{action}: {rows} rows, {differences} differences");
    }
    if let Some(first) = first {
        println!("FIRST DIFFERENCE\n{first}");
        panic!("mining differential disagreed");
    }
}

#[test]
fn target_switch_emits_old_reset_before_new_progress() {
    fn sample_at(x: i32, speed: f32) -> MiningSample {
        MiningSample { target: MiningTarget { position: position(x), identity: x as u64 }, destroy_speed: speed, hardness: 1.0, correct_tool: true }
    }
    // Pinned vanilla START arm: write start tick, reset old active position,
    // install the new position, then broadcast the new stage.
    let a = sample_at(1, 3.0);
    let b = sample_at(2, 6.0);
    let mut clock = MiningClock::new(10);
    assert!(matches!(clock.apply(MiningAction::Start(a), true), Ok(MiningStep::Evaluated { reset: None, .. })));
    clock.tick(|_| Some(a)).unwrap();
    let step = clock.apply(MiningAction::Start(b), true).unwrap();
    let events = match step {
        MiningStep::Evaluated { target, reset, destroy: None, .. } => vec![reset.unwrap(), target],
        _ => panic!("expected reset and progress"),
    };
    assert_eq!(events, vec![a.target, b.target]);
    assert_eq!(clock.active(), Some(MiningWork { target: b.target, start_tick: 11 }));
    // These checks reject both missing reset and reset-after-progress mutants.
    assert_ne!(events, vec![b.target]);
    assert_ne!(events, vec![b.target, a.target]);
    let same = clock.apply(MiningAction::Start(b), true).unwrap();
    assert!(matches!(same, MiningStep::Evaluated { reset: Some(t), .. } if t == b.target));
    let instant = sample_at(3, 30.0);
    assert!(matches!(clock.apply(MiningAction::Start(instant), true), Ok(MiningStep::Evaluated { reset: None, destroy: Some(_), .. })));
    assert_eq!(clock.active().unwrap().target, b.target);
}
