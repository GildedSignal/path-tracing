use std::{collections::BTreeMap, env, path::PathBuf, process::ExitCode};
use world::datapack::{DatapackGraph, DatapackSnapshot};

fn run() -> Result<(), String> {
    let mut args = env::args_os().skip(1);
    let root = PathBuf::from(args.next().ok_or("usage: datapack-graph WORLD [MIN_X MIN_Y MIN_Z MAX_X MAX_Y MAX_Z]")?);
    let bounds: Vec<i32> = args.map(|arg| {
        arg.to_str().and_then(|arg| arg.parse().ok()).ok_or("bounds must be integer cells")
    }).collect::<Result<_, _>>()?;
    let snapshot;
    let graph_only;
    let (graph, snapshot) = match bounds.as_slice() {
        [] => {
            graph_only = DatapackGraph::load(&root).map_err(|error| error.to_string())?;
            (&graph_only, None)
        }
        &[ax, ay, az, bx, by, bz] => {
            snapshot = DatapackSnapshot::load(&root, [ax, ay, az], [bx, by, bz]).map_err(|error| error.to_string())?;
            (&snapshot.graph, Some(&snapshot))
        }
        _ => return Err("expected six bounds coordinates".into()),
    };
    let tags = ["#minecraft:load".to_owned(), "#minecraft:tick".to_owned()];
    for tag in &tags {
        println!("{tag} {:?}", graph.tags.get(&tag[1..]));
    }
    println!("external packs {:?}", graph.external_packs);
    println!("saved game tick {}", graph.game_tick);
    for event in &graph.scheduled {
        println!("scheduled {} {} {}", event.trigger_tick, event.id, event.target);
    }
    println!("file functions {}, function tags {}, other resources {}", graph.functions.len(), graph.tags.len(), graph.data_resources.len());
    let tick_roots = tags.into_iter().chain(graph.scheduled.iter().map(|event| event.target.clone()));
    let (reachable, missing) = graph.reachable(tick_roots.clone());
    let actions = |functions: &std::collections::BTreeSet<String>| {
        let mut counts = BTreeMap::new();
        for command in functions.iter().flat_map(|name| &graph.functions[name]) {
            if let Some(verb) = command.action().first() {
                *counts.entry(verb.trim_start_matches('$')).or_insert(0) += 1;
            }
        }
        counts
    };
    println!("load/tick functions {}, unresolved {missing:?}, action counts {:?}", reachable.len(), actions(&reachable));
    if let Some(snapshot) = snapshot {
        println!("saved terrain {:?}", snapshot.saved_terrain().bounds());
        println!("{} saved block ticks in {bounds:?}", snapshot.pending_block_ticks.len());
        for tick in &snapshot.pending_block_ticks {
            println!("saved block tick {:?} {} delay {} priority {}", tick.cell, tick.block, tick.delay, tick.priority);
        }
        let blocks = &snapshot.command_blocks;
        println!("{} command blocks in {bounds:?}", blocks.len());
        for block in blocks {
            println!("command block {:?} auto={} powered={} action={:?} calls={:?}",
                block.cell, block.automatic, block.powered, block.command.action().first(),
                block.command.calls().collect::<Vec<_>>());
        }
        let roots = tick_roots.chain(blocks.iter().flat_map(|block| block.command.calls().map(str::to_owned)));
        let (reachable, missing) = graph.reachable(roots);
        println!("functions including command block roots {}, unresolved {missing:?}, action counts {:?}", reachable.len(), actions(&reachable));
        for id in &reachable {
            for command in &graph.functions[id] {
                if let Some((lo, hi)) = command.direct_block_writes(None) {
                    println!("{id}:{} direct block write {lo:?}..{hi:?}", command.line);
                }
            }
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("datapack-graph: {error}");
            ExitCode::FAILURE
        }
    }
}
