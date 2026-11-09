use std::{env, io::{self, Write}, process::ExitCode};
use world::WorldReader;
fn run() -> Result<(),String> {
    let args = env::args_os().skip(1).collect::<Vec<_>>();
    if args.len()!=4 && args.len()!=7 { return Err("usage: world-probe <world dir> <x> <y> <z> [<x1> <y1> <z1>]".into()); }
    let coords = args[1..].iter().map(|s| s.to_str().ok_or("coordinate is not UTF-8")?.parse::<i32>().map_err(|_| "coordinate must be an i32"))
        .collect::<Result<Vec<_>,_>>()?;
    let min = [coords[0],coords[1],coords[2]];
    let max = if coords.len()==3 { min } else { [coords[3],coords[4],coords[5]] };
    if (0..3).any(|i|min[i]>max[i]) { return Err("range minimum exceeds maximum".into()); }
    let mut reader = WorldReader::open(&args[0]).map_err(|e|e.to_string())?;
    let mut out = io::BufWriter::new(io::stdout().lock());
    for z in min[2]..=max[2] { for y in min[1]..=max[1] { for x in min[0]..=max[0] {
        let state = reader.block_state([x,y,z]).map_err(|e|e.to_string())?;
        if args.len()==4 { writeln!(out,"{state}") } else { writeln!(out,"{x} {y} {z} {state}") }.map_err(|e|e.to_string())?;
    }}}
    out.flush().map_err(|e|e.to_string())
}
fn main() -> ExitCode {
    match run() { Ok(())=>ExitCode::SUCCESS, Err(e)=>{eprintln!("world-probe: {e}"); ExitCode::FAILURE} }
}
