fn main() {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 3 { eprintln!("usage: deviation <plan.jsonl> <recording.jsonl>"); std::process::exit(2); }
    let run = || -> Result<(), String> {
        let plan = std::fs::read_to_string(&args[1]).map_err(|e| e.to_string())?;
        let recording = std::fs::read_to_string(&args[2]).map_err(|e| e.to_string())?;
        println!("{}", plan::deviation::compare(&plan, &recording)?.to_json()); Ok(())
    };
    if let Err(e) = run() { eprintln!("deviation: {e}"); std::process::exit(1); }
}
