use std::{collections::HashMap, path::PathBuf, time::Instant};
use plan::flight::{fly_with_budget, track_at};

struct View(PathBuf);
impl Drop for View {
    fn drop(&mut self) { if let Err(error) = std::fs::remove_dir_all(&self.0) { if error.kind() != std::io::ErrorKind::NotFound { eprintln!("Could not remove {}: {error}", self.0.display()); } } }
}

fn run() -> Result<(u32, u128), String> {
    let started = Instant::now();
    let mut args = std::env::args().skip(1);
    let name = args.next().ok_or("missing track")?;
    if name.is_empty() || !name.bytes().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_' || c == b'-') { return Err("track names use [a-z0-9_-]+".into()); }
    let (mut start, mut mode, mut out, mut cap) = (None, None, None, None);
    let (mut rig, mut world) = (None, None);
    while let Some(flag) = args.next() {
        let value = args.next().ok_or_else(|| format!("missing value for {flag}"))?;
        match flag.as_str() {
            "--start" if start.is_none() => start = Some(value),
            "--mode" if mode.is_none() => mode = Some(value),
            "--out" if out.is_none() => out = Some(PathBuf::from(value)),
            "--rig" if rig.is_none() => rig = Some(PathBuf::from(value)),
            "--world" if world.is_none() => world = Some(PathBuf::from(value)),
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
    let known = include_str!("../../../world/tests/fixtures/tracks.txt").lines().find(|line| line.split_whitespace().next() == Some(name.as_str()));
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let rig = rig.or_else(|| std::env::var_os("PATHTRACING_ROOT").map(PathBuf::from));
    let records = rig.as_ref().unwrap_or(&root).join("lab/tracks");
    let world = world.unwrap_or_else(|| rig.as_ref().map_or_else(|| root.join("engine/world/tests/fixtures/world"), |r| r.join("server/world")));
    let world = world.canonicalize().map_err(|e| format!("{}: {e}", world.display()))?;
    // Keep links on the source volume. The snapshot owns its cells before this view is removed.
    let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_err(|e| e.to_string())?.as_nanos();
    let view = View(world.parent().ok_or("world has no parent")?.join(format!(".plan-world-{}-{stamp}", std::process::id())));
    let prepared = std::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-File"])
        .arg(root.join("scripts/track-world-view.ps1"))
        .arg("-World").arg(&world).arg("-TrackFile").arg(records.join(format!("{name}.json")))
        .arg("-View").arg(&view.0).output().map_err(|e| format!("prepare dimension view: {e}"))?;
    if !prepared.status.success() { return Err(format!("track dimension: {}{}", String::from_utf8_lossy(&prepared.stdout), String::from_utf8_lossy(&prepared.stderr)).replace(['\r', '\n'], " ")); }
    let bounds = std::fs::read_to_string(view.0.join("bounds.txt")).map_err(|e| e.to_string())?;
    let track = track_at(known.unwrap_or(&bounds), &records, &view.0).map_err(|e| format!("track does not load: {e}"))?;
    drop(view);
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
