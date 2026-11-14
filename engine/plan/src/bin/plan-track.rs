use std::{collections::HashMap, path::PathBuf, time::Instant};
use plan::flight::{fly_with_budget, track};

fn run() -> Result<(u32, u128), String> {
    let started = Instant::now();
    let mut args = std::env::args().skip(1);
    let name = args.next().ok_or("missing track")?;
    let (mut start, mut mode, mut out, mut cap) = (None, None, None, None);
    while let Some(flag) = args.next() {
        let value = args.next().ok_or_else(|| format!("missing value for {flag}"))?;
        match flag.as_str() {
            "--start" if start.is_none() => start = Some(value),
            "--mode" if mode.is_none() => mode = Some(value),
            "--out" if out.is_none() => out = Some(PathBuf::from(value)),
            "--cap" if cap.is_none() => {
                let seconds: f64 = value.parse().map_err(|_| "invalid cap")?;
                if !seconds.is_finite() || seconds <= 0.0 { return Err("invalid cap".into()); }
                cap = Some(seconds);
            }
            _ => return Err(format!("unknown or repeated option {flag}")),
        }
    }
    if start.as_deref() != Some("settled") { return Err("only --start settled is supported; PlayerState::from_bits is unavailable".into()); }
    let mode = match mode.as_deref() { Some("3") => '3', Some("5") => '5', Some("b") => 'b', _ => return Err("mode must be 3, 5, or b".into()) };
    let out = out.ok_or("missing --out")?;
    let line = include_str!("../../../world/tests/fixtures/tracks.txt").lines().find(|line| line.split_whitespace().next() == Some(name.as_str())).ok_or("unknown track")?;
    let track = track(line).map_err(|e| format!("track does not load: {e}"))?;
    let mut log = Vec::new();
    let row = fly_with_budget(&track, mode, &words::tiny_programs(), &mut HashMap::new(), 20.0, false, &mut log, cap, Some(&out));
    if !row.found {
        let log = String::from_utf8_lossy(&log);
        return Err(log.lines().find(|line| line.contains("NO SENTENCE:")).unwrap_or("search found no sentence").replace('\n', " "));
    }
    Ok((row.ticks, started.elapsed().as_millis()))
}

fn main() {
    let result = std::panic::catch_unwind(run);
    match result {
        Ok(Ok((ticks, ms))) => println!("SENTENCE {ticks} thinking {ms}"),
        Ok(Err(reason)) => { println!("NO SENTENCE {reason}"); std::process::exit(3); }
        Err(_) => { println!("NO SENTENCE planner or export failed"); std::process::exit(3); }
    }
}
