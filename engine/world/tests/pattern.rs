//! Job 1.2: the pattern key over the real model table (docs/footprints.md).
use std::collections::{BTreeMap, BTreeSet};
use law::{Block, pattern::{FRAMES, model_digest}};
use world::{BlockState, model, model_table};

fn state(raw: &str) -> &'static Block {
    let (name, tail) = raw.split_once('[').unwrap_or((raw, ""));
    let mut properties = tail.trim_end_matches(']').split(',').filter(|p| !p.is_empty())
        .map(|p| { let (k, v) = p.split_once('=').unwrap(); (k.to_owned(), v.to_owned()) }).collect::<Vec<_>>();
    properties.sort();
    model(&BlockState { name: format!("minecraft:{name}"), properties }).unwrap()
}

// Does the key separate every model of the table, and find its turns and mirror by hashing alone?
#[test]
fn the_key_separates_models_and_finds_their_symmetry() {
    let table = model_table();
    let digest = |block: &Block, frame| model_digest(block, frame).unwrap().expect("every frame carries every model exactly");
    let identity: BTreeMap<u128, usize> = table.iter().enumerate().map(|(id, block)| (digest(block, FRAMES[0]), id)).collect();
    assert_eq!(identity.len(), table.len(), "one physics per model");
    // A turned model is found when its digest is some model's own.
    let stairs = |facing: &str, shape: &str| state(&format!("oak_stairs[facing={facing},half=bottom,shape={shape},waterlogged=false]"));
    assert_eq!(digest(stairs("north", "straight"), FRAMES[1]), digest(stairs("east", "straight"), FRAMES[0]));
    assert_eq!(digest(stairs("north", "inner_left"), FRAMES[4]), digest(stairs("north", "inner_right"), FRAMES[0]));
    let open: BTreeSet<usize> = table.iter().enumerate()
        .filter(|(_, block)| FRAMES.iter().any(|&frame| !identity.contains_key(&digest(block, frame))))
        .map(|(id, _)| id).collect();
    // Mojang's own asymmetric shapes: bites from one side, candles in a fixed arrangement, the egg.
    let id = |block: &Block| table.iter().position(|b| std::ptr::eq(b, block)).unwrap();
    let mut expected: BTreeSet<usize> = (1..=6).map(|b| id(state(&format!("cake[bites={b}]")))).collect();
    for n in 2..=4 { for lit in [false, true] { for wet in [false, true] {
        expected.insert(id(state(&format!("candle[candles={n},lit={lit},waterlogged={wet}]"))));
    } } }
    expected.insert(id(state("sniffer_egg[hatch=0]")));
    assert_eq!(open, expected);
    println!("{} models, {} closed under the 8 frames, {} not", table.len(), table.len()-open.len(), open.len());
}
