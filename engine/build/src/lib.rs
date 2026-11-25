//! 4D Reverse-Demolition & Bernstein-Footprint Causal DAG Compiler.

pub mod blueprint;
pub mod verify;
pub mod waves;

pub use blueprint::{Blueprint, Direction, Registry, StateId, SurvivalRule};
pub use verify::{verify_4d, VerificationReport};
pub use waves::{compile_4d, BuildKit, BuildNode, CompiledBuild};

#[cfg(test)]
mod tests {
    use super::*;

    /// Einstein Extreme 1: 1x1x32 Vertical Sky Needle.
    /// Verifies emergent temporary scaffolding spine synthesis (`has_scaffolding = true`)
    /// and 1-root-break collapse at completion (`residual_obligations == 0`).
    #[test]
    fn einstein_1_sky_needle_emergent_scaffolding() {
        let mut reg = Registry::new();
        let script = "/fill 0 -60 0 0 -29 0 minecraft:stone_bricks";
        let bp = Blueprint::parse_commands("sky_needle_32", -60, script, &mut reg).unwrap();
        assert_eq!(bp.target.len(), 32);

        let kit_bare = BuildKit {
            has_scaffolding: false,
            has_ender_pearl: false,
            has_out_of_vault_teleport: false,
        };
        let comp_bare = compile_4d(&bp, &reg, &kit_bare).unwrap();
        let rep_bare = verify_4d(&bp, &reg, &comp_bare).unwrap();
        assert_eq!(comp_bare.scaffolding_cells.len(), 0);
        assert_eq!(rep_bare.residual_obligations, 0);

        let kit_scaf = BuildKit::default();
        let comp_scaf = compile_4d(&bp, &reg, &kit_scaf).unwrap();
        assert!(
            !comp_scaf.scaffolding_cells.is_empty(),
            "emergent scaffolding spine must be synthesized for tall needle"
        );
        let rep_scaf = verify_4d(&bp, &reg, &comp_scaf).unwrap();
        assert_eq!(rep_scaf.final_blocks_matched, 32);
        assert_eq!(rep_scaf.residual_obligations, 0);
    }

    /// Einstein Extreme 2: Solid 8x6x8 Monolith (384 solid blocks, zero interior air).
    /// Verifies that the backward 4D demolition wave peels the monolith from outside in and the
    /// forward causal wave retreats outward like a 3D printer head with zero fermion collisions.
    #[test]
    fn einstein_2_solid_monolith_3d_printer_wave() {
        let mut reg = Registry::new();
        let script = "/fill 0 -60 0 7 -55 7 minecraft:stone_bricks";
        let bp = Blueprint::parse_commands("solid_monolith", -60, script, &mut reg).unwrap();
        assert_eq!(bp.target.len(), 384);

        let kit = BuildKit::default();
        let comp = compile_4d(&bp, &reg, &kit).unwrap();
        let rep = verify_4d(&bp, &reg, &comp).unwrap();
        assert_eq!(rep.final_blocks_matched, 384);
        assert_eq!(rep.residual_obligations, 0);
        assert!(
            rep.max_parallel_in_one_layer >= 16,
            "causal wave must expose parallel commuting placements per layer"
        );
    }

    /// Einstein Extreme 3: 5-Block-Thick Hermetically Sealed Vault with Interior Hanging Chandelier.
    /// Reach is 4.5 blocks, wall thickness is 5 blocks: a bot cannot reach the interior from outside.
    /// Verifies that the 4D reverse-demolition wave leaves a breach tunnel open, hangs the chandelier
    /// under the ceiling inside, and plugs the breach tunnel from the inside out on retreat.
    #[test]
    fn einstein_3_thick_sealed_vault_with_interior_chandelier() {
        let mut reg = Registry::new();
        let script = "\
            /fill 0 -60 0 12 -54 12 minecraft:polished_blackstone_bricks\n\
            /fill 5 -59 5 7 -56 7 minecraft:air\n\
            /setblock 6 -56 6 minecraft:lantern[hanging=true,waterlogged=false]\n";
        let bp = Blueprint::parse_commands("thick_sealed_vault", -60, script, &mut reg).unwrap();

        let kit = BuildKit::default();
        let comp = compile_4d(&bp, &reg, &kit).unwrap();
        let rep = verify_4d(&bp, &reg, &comp).unwrap();
        assert_eq!(rep.final_blocks_matched, bp.target.len());
        assert_eq!(rep.residual_obligations, 0);

        // Verify causal order: ceiling `[6, -55, 6]` placed BEFORE chandelier `[6, -56, 6]`,
        // and chandelier placed BEFORE the final outer breach plug!
        let mut l_ceiling = 0;
        let mut l_chandelier = 0;
        let mut l_last_wall = 0;
        for nd in &comp.nodes {
            if nd.pos == [6, -55, 6] {
                l_ceiling = nd.earliest_tick;
            } else if nd.pos == [6, -56, 6] {
                l_chandelier = nd.earliest_tick;
            }
            l_last_wall = l_last_wall.max(nd.earliest_tick);
        }
        assert!(
            l_ceiling < l_chandelier && l_chandelier < l_last_wall,
            "4D vault causal order violated: ceiling={l_ceiling}, chandelier={l_chandelier}, last_wall={l_last_wall}"
        );
    }

    /// Einstein Extreme 4: Floating Sky Island disconnected from ground (`y = -40..-38`, ground at `y = -60`).
    /// Verifies that the forward wave automatically synthesizes a temporary 20-block scaffolding spine
    /// from `y = -60..-41`, builds the island at `y = -40..-38`, and breaks the scaffolding root at `-60`
    /// so the island floats in empty air with 0 residual debt.
    #[test]
    fn einstein_4_floating_sky_island_emergent_scaffolding_and_cleanup() {
        let mut reg = Registry::new();
        let script = "\
            /fill 0 -40 0 4 -40 4 minecraft:stone_bricks\n\
            /fill 1 -39 1 3 -39 3 minecraft:gold_block\n\
            /setblock 2 -38 2 minecraft:lantern[hanging=false,waterlogged=false]\n";
        let bp = Blueprint::parse_commands("floating_island", -60, script, &mut reg).unwrap();
        assert_eq!(bp.target.len(), 25 + 9 + 1);

        let kit = BuildKit::default();
        let comp = compile_4d(&bp, &reg, &kit).unwrap();
        assert_eq!(
            comp.scaffolding_cells.len(),
            20,
            "must erect exact 20-block temporary scaffolding spine from y=-60 to -41"
        );
        assert_eq!(comp.scaffolding_roots.len(), 1);

        let rep = verify_4d(&bp, &reg, &comp).unwrap();
        assert_eq!(rep.final_blocks_matched, 35);
        assert_eq!(
            rep.residual_obligations, 0,
            "scaffolding spine must be 100% collapsed at completion"
        );
        assert_eq!(rep.breaks_verified, 1);
    }

    /// Einstein Extreme 5: Coupled / Unstable World (Water basin, Hanging lantern, Carpet, Potted plant, Stairs).
    #[test]
    fn einstein_5_coupled_survival_and_water_basin() {
        let mut reg = Registry::new();
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
        let bp = Blueprint::parse_commands("coupled_basin", -60, script, &mut reg).unwrap();

        let kit = BuildKit::default();
        let comp = compile_4d(&bp, &reg, &kit).unwrap();
        let rep = verify_4d(&bp, &reg, &comp).unwrap();
        assert_eq!(rep.final_blocks_matched, bp.target.len());
        assert_eq!(rep.residual_obligations, 0);
    }

    /// Full 5-Storey Pagoda (`jobs/pagoda/build.txt`, 797 commands -> 4,324 net non-air blocks).
    #[test]
    fn full_pagoda_4d_compilation_and_law_verification() {
        let mut reg = Registry::new();
        let script = include_str!("../../../jobs/pagoda/build.txt");
        let bp = Blueprint::parse_commands("pagoda", -60, script, &mut reg).unwrap();
        assert_eq!(bp.command_count, 797);
        assert_eq!(bp.raw_placements, 7951);

        let kit = BuildKit::default();
        let comp = compile_4d(&bp, &reg, &kit).unwrap();
        let rep = verify_4d(&bp, &reg, &comp).unwrap();
        assert_eq!(rep.final_blocks_matched, bp.target.len());
        assert_eq!(rep.residual_obligations, 0);
    }
}
