//! Strict 4D Causal DAG & Law Verifier (`verify_4d`).
//! Replays a `CompiledBuild` layer-by-layer (`L = 1..=T_crit`) against Mojang's 26.2 placement,
//! 4.5m reach, sightline (`traverse_blocks`), survival (`canSurvive`), fermion body exclusion
//! (`Level.isUnobstructed`), exit corridor clearance, and obligation-ledger cleanup laws.

use crate::blueprint::{
    body_intersects_block, ray_intermediate_cells, within_reach_4_5, Blueprint, Direction,
    Registry, StateId, SurvivalRule,
};
use crate::waves::CompiledBuild;
use law::{BlockPos, Vec3};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerificationReport {
    pub causal_layers: u32,
    pub placements_verified: usize,
    pub breaks_verified: usize,
    pub max_parallel_in_one_layer: usize,
    pub final_blocks_matched: usize,
    pub residual_obligations: usize,
}

pub fn verify_4d(
    bp: &Blueprint,
    reg: &Registry,
    compiled: &CompiledBuild,
) -> Result<VerificationReport, String> {
    let mut world: BTreeMap<BlockPos, (StateId, bool)> = BTreeMap::new();
    let mut placed_layer: Vec<u32> = vec![0; compiled.nodes.len()];
    let mut by_layer: BTreeMap<u32, Vec<usize>> = BTreeMap::new();

    for (idx, nd) in compiled.nodes.iter().enumerate() {
        for &u in &nd.preds {
            if u >= idx || compiled.nodes[u].earliest_tick >= nd.earliest_tick {
                return Err(format!(
                    "DAG causality violation: node {idx} ({:?} at layer {}) depends on predecessor {u} at layer {}",
                    nd.pos, nd.earliest_tick, compiled.nodes[u].earliest_tick
                ));
            }
        }
        by_layer.entry(nd.earliest_tick).or_default().push(idx);
    }

    let mut placements_verified = 0usize;
    let mut breaks_verified = 0usize;
    let mut max_parallel = 0usize;

    for (&layer, node_indices) in &by_layer {
        max_parallel = max_parallel.max(node_indices.len());

        // Verify all nodes in this causal layer against the world state and each other.
        for &idx in node_indices {
            let nd = &compiled.nodes[idx];

            // (0) All causal predecessors must already be placed in an earlier layer.
            for &u in &nd.preds {
                if placed_layer[u] == 0 || placed_layer[u] >= layer {
                    return Err(format!(
                        "layer {layer}: node {idx} ({:?}) executed before predecessor {u}",
                        nd.pos
                    ));
                }
            }

            // (a) Active placer body cells (`body_cells` at `stand_pos`) must not overlap any existing solid block.
            for &bc in &nd.body_cells {
                if let Some(&(st, is_temp)) = world.get(&bc) {
                    if !is_temp && !reg.entry(st).block.shape.is_empty() {
                        return Err(format!(
                            "layer {layer}: placer body at {:?} overlaps existing solid block at {bc:?}",
                            nd.stand_pos
                        ));
                    }
                }
            }

            // (b) Exit corridor (`exit_cells`) from `stand_pos` to exterior must remain unoccluded by solid blocks.
            for &ec in &nd.exit_cells {
                if let Some(&(st, is_temp)) = world.get(&ec) {
                    if !is_temp && !reg.entry(st).block.shape.is_empty() {
                        return Err(format!(
                            "layer {layer}: placer at {:?} has exit corridor occluded at {ec:?}",
                            nd.stand_pos
                        ));
                    }
                }
            }

            // (c) Target cell must be empty.
            if world.contains_key(&nd.pos) {
                return Err(format!(
                    "layer {layer}: tried to place at already-occupied cell {:?}",
                    nd.pos
                ));
            }

            // (d) `clicked_pos` must exist in `world` (or be ground `< bp.ground_y`).
            let clicked_exists =
                nd.clicked_pos[1] < bp.ground_y || world.contains_key(&nd.clicked_pos);
            if !clicked_exists {
                return Err(format!(
                    "layer {layer}: placed {:?} against missing clicked_pos {:?}",
                    nd.pos, nd.clicked_pos
                ));
            }

            // (e) Reach check: 4.5m from bot eye at `stand_pos`.
            let eye = Vec3::new(
                nd.stand_pos[0] as f64 + 0.5,
                nd.stand_pos[1] as f64 + if nd.crouch_crawl { 0.4 } else { 1.62 },
                nd.stand_pos[2] as f64 + 0.5,
            );
            if !nd.is_temporary && !within_reach_4_5(eye, nd.pos) {
                return Err(format!(
                    "layer {layer}: stand_pos {:?} out of 4.5m reach of {:?}",
                    nd.stand_pos, nd.pos
                ));
            }

            // (f) Sightline check (`traverse_blocks` / `ray_intermediate_cells`).
            if !nd.is_temporary {
                let ray = ray_intermediate_cells(eye, nd.pos, nd.clicked_face);
                for r in ray {
                    if r != nd.pos {
                        if let Some(&(st_r, temp_r)) = world.get(&r) {
                            if !temp_r && !reg.entry(st_r).block.shape.is_empty() {
                                return Err(format!(
                                    "layer {layer}: sightline from {:?} to {:?} occluded by {r:?}",
                                    nd.stand_pos, nd.pos
                                ));
                            }
                        }
                    }
                }
            }

            // (g) Fermion self-exclusion (`Level.isUnobstructed`): placing a solid block at `nd.pos`
            //     must not intersect the placer's own body at `nd.stand_pos`.
            let shape = &reg.entry(nd.state).block.shape;
            if !nd.is_temporary
                && !shape.is_empty()
                && body_intersects_block(nd.stand_pos, nd.crouch_crawl, nd.pos, shape)
            {
                return Err(format!(
                    "layer {layer}: fermion self-collision placing solid block at {:?} from {:?}",
                    nd.pos, nd.stand_pos
                ));
            }

            // (h) Survival rule (`canSurvive`) at placement.
            if !nd.is_temporary {
                check_can_survive(nd.pos, nd.state, bp, reg, &world, layer)?;
            }

            world.insert(nd.pos, (nd.state, nd.is_temporary));
            placed_layer[idx] = layer;
            placements_verified += 1;
        }
    }

    // Cleanup layer: break all temporary scaffolding roots (`compiled.scaffolding_roots`),
    // collapsing each vertical scaffolding column.
    for &root in &compiled.scaffolding_roots {
        let mut cur = root;
        let mut removed = 0usize;
        while let Some(&(_, is_temp)) = world.get(&cur) {
            if is_temp {
                world.remove(&cur);
                removed += 1;
                cur[1] += 1;
            } else {
                break;
            }
        }
        if removed == 0 {
            return Err(format!(
                "cleanup: tried to break missing scaffolding root at {root:?}"
            ));
        }
        breaks_verified += 1;
    }

    let residual_obligations = world.values().filter(|&&(_, is_temp)| is_temp).count();
    if residual_obligations != 0 {
        return Err(format!(
            "completion: {residual_obligations} uncleaned temporary obligations remain in world"
        ));
    }

    if world.len() != bp.target.len() {
        return Err(format!(
            "completion: world has {} blocks, blueprint has {}",
            world.len(),
            bp.target.len()
        ));
    }
    for (&pos, &expected_state) in &bp.target {
        match world.get(&pos) {
            Some(&(actual_state, false)) if actual_state == expected_state => {}
            Some(&(actual_state, _)) => {
                return Err(format!(
                    "completion: cell {pos:?} has state {:?}, expected {:?}",
                    reg.entry(actual_state).raw,
                    reg.entry(expected_state).raw
                ))
            }
            None => {
                return Err(format!(
                    "completion: missing blueprint block at {pos:?}"
                ))
            }
        }
    }

    Ok(VerificationReport {
        causal_layers: compiled.critical_path_ticks,
        placements_verified,
        breaks_verified,
        max_parallel_in_one_layer: max_parallel,
        final_blocks_matched: world.len(),
        residual_obligations,
    })
}

fn check_can_survive(
    pos: BlockPos,
    state: StateId,
    bp: &Blueprint,
    reg: &Registry,
    world: &BTreeMap<BlockPos, (StateId, bool)>,
    layer: u32,
) -> Result<(), String> {
    let ground_y = bp.ground_y;
    let has_support_at = |p: BlockPos| -> bool { p[1] < ground_y || world.contains_key(&p) };
    match reg.entry(state).survival {
        SurvivalRule::AnyNeighbor => {
            if Direction::VALUES
                .into_iter()
                .any(|d| has_support_at(d.relative(pos)))
            {
                Ok(())
            } else {
                Err(format!(
                    "layer {layer}: block {:?} at {pos:?} has no adjacent neighbor",
                    reg.entry(state).raw
                ))
            }
        }
        SurvivalRule::NeedBelow | SurvivalRule::Gravity => {
            let below = [pos[0], pos[1] - 1, pos[2]];
            if has_support_at(below) {
                Ok(())
            } else {
                Err(format!(
                    "layer {layer}: block {:?} at {pos:?} violated NeedBelow/Gravity (missing {below:?})",
                    reg.entry(state).raw
                ))
            }
        }
        SurvivalRule::NeedAbove => {
            let above = [pos[0], pos[1] + 1, pos[2]];
            if has_support_at(above) {
                Ok(())
            } else {
                Err(format!(
                    "layer {layer}: block {:?} at {pos:?} violated NeedAbove (missing {above:?})",
                    reg.entry(state).raw
                ))
            }
        }
        SurvivalRule::NeedSide(dir) => {
            let anchor = dir.relative(pos);
            if has_support_at(anchor) {
                Ok(())
            } else {
                Err(format!(
                    "layer {layer}: block {:?} at {pos:?} violated NeedSide({dir:?}) (missing {anchor:?})",
                    reg.entry(state).raw
                ))
            }
        }
        SurvivalRule::WaterBasin => {
            let below = [pos[0], pos[1] - 1, pos[2]];
            if !has_support_at(below) {
                return Err(format!(
                    "layer {layer}: water at {pos:?} missing basin floor at {below:?}"
                ));
            }
            for d in Direction::HORIZONTAL {
                let side = d.relative(pos);
                if let Some(&side_st) = bp.target.get(&side) {
                    if reg.entry(side_st).survival != SurvivalRule::WaterBasin && !has_support_at(side) {
                        return Err(format!(
                            "layer {layer}: water at {pos:?} missing lateral basin wall at {side:?}"
                        ));
                    }
                }
            }
            Ok(())
        }
    }
}
