//! CLI runner for the 4D Reverse-Demolition & Bernstein Causal Footprint DAG Compiler.

use build::{compile_4d, verify_4d, Blueprint, BuildKit, Registry};
use std::time::Instant;

fn main() {
    let mut reg = Registry::new();

    println!("=== 1. STRUCTURAL EXTREMES (4D Reverse-Demolition & Bernstein Causal DAG) ===");
    println!(
        "{:<28} {:>6} {:>6} {:>7} {:>7} {:>8} {:>10}",
        "Fixture", "Blocks", "Scaf", "Roots", "Layers", "MaxPar", "Verify"
    );

    // 1. Sky Needle (32m)
    {
        let script = "/fill 0 -60 0 0 -29 0 minecraft:stone_bricks";
        let bp = Blueprint::parse_commands("1.sky_needle_32m", -60, script, &mut reg).unwrap();
        let kit = BuildKit::default();
        let comp = compile_4d(&bp, &reg, &kit).unwrap();
        let rep = verify_4d(&bp, &reg, &comp).unwrap();
        println!(
            "{:<28} {:>6} {:>6} {:>7} {:>7} {:>8} {:>10}",
            bp.name,
            bp.target.len(),
            comp.scaffolding_cells.len(),
            comp.scaffolding_roots.len(),
            comp.critical_path_ticks,
            rep.max_parallel_in_one_layer,
            format!("OK ({})", rep.residual_obligations),
        );
    }

    // 2. Solid 8x6x8 Monolith
    {
        let script = "/fill 0 -60 0 7 -55 7 minecraft:stone_bricks";
        let bp = Blueprint::parse_commands("2.solid_monolith_8x6x8", -60, script, &mut reg).unwrap();
        let kit = BuildKit::default();
        let comp = compile_4d(&bp, &reg, &kit).unwrap();
        let rep = verify_4d(&bp, &reg, &comp).unwrap();
        println!(
            "{:<28} {:>6} {:>6} {:>7} {:>7} {:>8} {:>10}",
            bp.name,
            bp.target.len(),
            comp.scaffolding_cells.len(),
            comp.scaffolding_roots.len(),
            comp.critical_path_ticks,
            rep.max_parallel_in_one_layer,
            format!("OK ({})", rep.residual_obligations),
        );
    }

    // 3. 5-Thick Hermetically Sealed Vault + Chandelier
    {
        let script = "\
            /fill 0 -60 0 12 -54 12 minecraft:polished_blackstone_bricks\n\
            /fill 5 -59 5 7 -56 7 minecraft:air\n\
            /setblock 6 -56 6 minecraft:lantern[hanging=true,waterlogged=false]\n";
        let bp = Blueprint::parse_commands("3.sealed_vault_5thick", -60, script, &mut reg).unwrap();
        let kit = BuildKit::default();
        let comp = compile_4d(&bp, &reg, &kit).unwrap();
        let rep = verify_4d(&bp, &reg, &comp).unwrap();
        println!(
            "{:<28} {:>6} {:>6} {:>7} {:>7} {:>8} {:>10}",
            bp.name,
            bp.target.len(),
            comp.scaffolding_cells.len(),
            comp.scaffolding_roots.len(),
            comp.critical_path_ticks,
            rep.max_parallel_in_one_layer,
            format!("OK ({})", rep.residual_obligations),
        );
    }

    // 4. Floating Sky Island (20 blocks above ground)
    {
        let script = "\
            /fill 0 -40 0 4 -40 4 minecraft:stone_bricks\n\
            /fill 1 -39 1 3 -39 3 minecraft:gold_block\n\
            /setblock 2 -38 2 minecraft:lantern[hanging=false,waterlogged=false]\n";
        let bp = Blueprint::parse_commands("4.floating_sky_island", -60, script, &mut reg).unwrap();
        let kit = BuildKit::default();
        let comp = compile_4d(&bp, &reg, &kit).unwrap();
        let rep = verify_4d(&bp, &reg, &comp).unwrap();
        println!(
            "{:<28} {:>6} {:>6} {:>7} {:>7} {:>8} {:>10}",
            bp.name,
            bp.target.len(),
            comp.scaffolding_cells.len(),
            comp.scaffolding_roots.len(),
            comp.critical_path_ticks,
            rep.max_parallel_in_one_layer,
            format!("OK ({})", rep.residual_obligations),
        );
    }

    // 5. Coupled Water Basin & Hanging Lantern
    {
        let script = "\
            /fill 0 -60 0 4 -60 4 minecraft:mossy_stone_bricks\n\
            /fill 0 -59 0 4 -59 4 minecraft:mossy_stone_bricks\n\
            /fill 1 -59 1 3 -59 3 minecraft:water[level=0]\n\
            /setblock 2 -59 2 minecraft:sea_pickle[pickles=3,waterlogged=true]\n\
            /setblock 0 -58 0 minecraft:red_carpet\n\
            /setblock 4 -58 0 minecraft:potted_bamboo\n\
            /setblock 0 -58 4 minecraft:stone_brick_stairs[facing=east,half=bottom,shape=straight]\n\
            /fill 2 -58 0 2 -56 0 minecraft:stripped_crimson_hyphae\n\
            /setblock 2 -56 1 minecraft:deepslate_tile_slab[type=top,waterlogged=false]\n\
            /setblock 2 -57 1 minecraft:lantern[hanging=true,waterlogged=false]\n";
        let bp = Blueprint::parse_commands("5.coupled_water_basin", -60, script, &mut reg).unwrap();
        let kit = BuildKit::default();
        let comp = compile_4d(&bp, &reg, &kit).unwrap();
        let rep = verify_4d(&bp, &reg, &comp).unwrap();
        println!(
            "{:<28} {:>6} {:>6} {:>7} {:>7} {:>8} {:>10}",
            bp.name,
            bp.target.len(),
            comp.scaffolding_cells.len(),
            comp.scaffolding_roots.len(),
            comp.critical_path_ticks,
            rep.max_parallel_in_one_layer,
            format!("OK ({})", rep.residual_obligations),
        );
    }

    println!("\n=== 2. CLASSICAL 5-STOREY PAGODA (jobs/pagoda/build.txt) ===");
    let script = include_str!("../../../../jobs/pagoda/build.txt");
    let t0 = Instant::now();
    let bp = Blueprint::parse_commands("pagoda", -60, script, &mut reg).unwrap();
    let parse_us = t0.elapsed().as_micros();

    let kit = BuildKit::default();
    let t1 = Instant::now();
    let comp = compile_4d(&bp, &reg, &kit).unwrap();
    let compile_ms = t1.elapsed().as_secs_f64() * 1000.0;
    let rep = verify_4d(&bp, &reg, &comp).unwrap();

    let distinct_items = bp
        .target
        .values()
        .map(|&st| reg.entry(st).item)
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    println!(
        "Pagoda parsed in {} us ({} commands, {} raw placements -> {} net blocks, {} distinct items).",
        parse_us,
        bp.command_count,
        bp.raw_placements,
        bp.target.len(),
        distinct_items
    );
    println!(
        "4D Bernstein DAG compiled in {:.2} ms: {} total nodes ({} emergent spire scaffolding cells, {} roots), {} causal layers, max parallel width = {} blocks/layer, verified OK (residual obligations = {}).",
        compile_ms,
        comp.nodes.len(),
        comp.scaffolding_cells.len(),
        comp.scaffolding_roots.len(),
        comp.critical_path_ticks,
        rep.max_parallel_in_one_layer,
        rep.residual_obligations
    );
}
