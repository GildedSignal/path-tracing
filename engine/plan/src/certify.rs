//! Single-winner route certification. Exploration runs raw; only the best route
//! pays one `RecordingWorld` pass to record read cells, worldtube bounds, and `NoRegen` guards.
use std::collections::BTreeMap;
use law::{
    BlockPos, PlayerState, World,
    codec::{self, LabelId, proof::Refusal},
    footprint::RecordingWorld,
    sim::WorldDelta,
};
use words::Program;
use crate::{
    Goal, KitOp,
    events::{Event, EventKind},
    inline::List,
    solve::WordStep,
    source::{CorridorProof, ObservedWorld},
    state_view::StateAt,
    store::sprint_allowed,
    word_store::{CachedRoute, Guard, NoRegen, Route, motion_key},
};

fn apply_op<W: World>(delta: &mut WorldDelta<'_, W>, op: KitOp) {
    match op {
        KitOp::Interact { pos, after, partner } => {
            delta.set(pos, after);
            if let Some((pp, pa)) = partner {
                delta.set(pp, pa);
            }
        }
        KitOp::Break { pos, after, .. }
        | KitOp::Place { pos, after }
        | KitOp::BucketEmpty { pos, after }
        | KitOp::BucketPickup { pos, after } => {
            delta.set(pos, after);
        }
    }
}

pub(crate) fn certify_route<W: World>(
    from: &PlayerState,
    goal: Goal,
    label: LabelId,
    provenance_hash: [u8; 32],
    world: &W,
    programs: &[Program],
    raw_steps: &[WordStep],
    kit_key: u64,
    restore_required: bool,
    work_certify_ticks: &mut u64,
) -> Result<(Route, CachedRoute), (Refusal, &'static str)> {
    let obs = ObservedWorld::new(world);
    let has_edits = raw_steps.iter().any(|s| s.edit.is_some());
    let mut delta = has_edits.then(|| WorldDelta::new(&obs));
    let mut edited_positions: Vec<BlockPos> = Vec::new();
    let mut applied_ops: Vec<(usize, KitOp, bool)> = Vec::new();
    let mut state = from.clone();
    let mut certified_steps: List<WordStep, 12> = List::new();
    let mut touch: Option<u64> = (!has_edits && goal.touched(from, from)).then_some(0);
    let mut reads_map: BTreeMap<(BlockPos, u8), u128> = BTreeMap::new();
    let mut lo = [i32::MAX; 3];
    let mut hi = [i32::MIN; 3];
    let mut total_damage_no_regen = 0.0f64;

    for (idx, step) in raw_steps.iter().enumerate() {
        let step_start = state.clone();
        let program = &programs[step.program as usize];
        let held = program.held_input(step.args);
        let is_break = matches!(step.edit, Some(KitOp::Break { .. }));
        if let (Some(op), Some(d)) = (step.edit, delta.as_mut()) {
            match op {
                KitOp::Interact { pos, partner, .. } => {
                    edited_positions.push(pos);
                    if let Some((pp, _)) = partner { edited_positions.push(pp); }
                }
                KitOp::Break { pos, .. }
                | KitOp::Place { pos, .. }
                | KitOp::BucketEmpty { pos, .. }
                | KitOp::BucketPickup { pos, .. } => edited_positions.push(pos),
            }
            if !is_break {
                apply_op(d, op);
                applied_ops.push((idx, op, true));
            }
        }
        let is_last_step = idx + 1 == raw_steps.len();
        let mut actual_ticks = 0u32;
        let mut touched_during_step = false;
        let fp = if let Some(d) = delta.as_ref() {
            let rec = RecordingWorld::new(d);
            for elapsed in 0..step.ticks {
                let before_box = state.bounding_box();
                let input = if is_break {
                    Default::default()
                } else {
                    match held {
                        Some(inp) => inp,
                        None => program
                            .select(&rec, &state, elapsed, step.args)
                            .ok_or((Refusal::StateOrContext, "select-delta"))?
                            .1,
                    }
                };
                if law::try_tick(&mut state, &rec, &input).is_err() || obs.check().is_err() {
                    return Err((Refusal::Coverage, "tick-delta"));
                }
                *work_certify_ticks += 1;
                actual_ticks += 1;
                if state.health <= 0.0 {
                    return Err((Refusal::StateOrContext, "dead-delta"));
                }
                if goal.touched_boxes(before_box, state.bounding_box()) {
                    touched_during_step = true;
                    if is_last_step && !is_break {
                        touch.get_or_insert(state.t - from.t);
                        break;
                    }
                }
            }
            rec.take()
        } else {
            let rec = RecordingWorld::new(&obs);
            for elapsed in 0..step.ticks {
                let before_box = state.bounding_box();
                let input = match held {
                    Some(inp) => inp,
                    None => program
                        .select(&rec, &state, elapsed, step.args)
                        .ok_or((Refusal::StateOrContext, "select-obs"))?
                        .1,
                };
                if law::try_tick(&mut state, &rec, &input).is_err() || obs.check().is_err() {
                    return Err((Refusal::Coverage, "tick-obs"));
                }
                *work_certify_ticks += 1;
                actual_ticks += 1;
                if state.health <= 0.0 {
                    return Err((Refusal::StateOrContext, "dead-obs"));
                }
                if goal.touched_boxes(before_box, state.bounding_box()) {
                    touch.get_or_insert(state.t - from.t);
                    break;
                }
            }
            rec.take()
        };
        if is_break {
            if let (Some(op), Some(d)) = (step.edit, delta.as_mut()) {
                apply_op(d, op);
                applied_ops.push((idx, op, false));
                if is_last_step && (touched_during_step || goal.touched(&state, &state)) {
                    touch.get_or_insert(state.t - from.t);
                }
            }
        }

        let mut step_lo = [i32::MAX; 3];
        let mut step_hi = [i32::MIN; 3];
        for p in fp.reads.keys().chain(fp.swept.iter()).copied() {
            for a in 0..3 {
                step_lo[a] = step_lo[a].min(p[a]);
                step_hi[a] = step_hi[a].max(p[a]);
                lo[a] = lo[a].min(p[a]);
                hi[a] = hi[a].max(p[a]);
            }
        }
        if step_lo[0] > step_hi[0] {
            let p = [state.position.x.floor() as i32, state.position.y.floor() as i32, state.position.z.floor() as i32];
            step_lo = p;
            step_hi = p;
        }
        for (&p, &mask) in &fp.reads {
            let b = world.known_block(p).map_err(|_| (Refusal::Coverage, "known-read"))?;
            let cell = codec::cell(b, world.loaded(p), mask).map_err(|e| (Refusal::Law(e), "codec-read"))?;
            reads_map.entry((p, mask)).or_insert_with(|| codec::digest(cell.bytes()));
        }

        // Measure unmitigated damage when any tick altered health (zero cost on non-damaging words).
        if state.health != step_start.health {
            let nr_obs = ObservedWorld::new(world);
            let nr = NoRegen(&nr_obs);
            let mut probe = step_start.clone();
            let start_h = probe.health.max(20.0);
            probe.health = start_h;
            if has_edits {
                let mut nr_delta = WorldDelta::new(&nr);
                for &(prev_idx, prev_op, _) in &applied_ops {
                    if prev_idx < idx || (prev_idx == idx && !is_break) {
                        apply_op(&mut nr_delta, prev_op);
                    }
                }
                for elapsed in 0..actual_ticks {
                    let input = if is_break {
                        Default::default()
                    } else {
                        match held {
                            Some(inp) => inp,
                            None => program
                                .select(&nr_delta, &probe, elapsed, step.args)
                                .ok_or((Refusal::StateOrContext, "select-nr-delta"))?
                                .1,
                        }
                    };
                    if law::try_tick(&mut probe, &nr_delta, &input).is_err() || nr_obs.check().is_err() {
                        return Err((Refusal::Coverage, "tick-nr-delta"));
                    }
                    *work_certify_ticks += 1;
                }
            } else {
                for elapsed in 0..actual_ticks {
                    let input = match held {
                        Some(inp) => inp,
                        None => program
                            .select(&nr, &probe, elapsed, step.args)
                            .ok_or((Refusal::StateOrContext, "select-nr-obs"))?
                            .1,
                    };
                    if law::try_tick(&mut probe, &nr, &input).is_err() || nr_obs.check().is_err() {
                        return Err((Refusal::Coverage, "tick-nr-obs"));
                    }
                    *work_certify_ticks += 1;
                }
            }
            total_damage_no_regen += f64::from(start_h - probe.health).max(0.0);
        }

        let mut events = step.events.clone();
        if idx == 0 {
            events.push_mark_unique(Event {
                tick: step_start.t,
                kind: EventKind::FoodGuard { sprint_allowed: sprint_allowed(from) },
            });
        }
        if let Some(prev) = certified_steps.last() {
            if prev.args.refined_yaw != step.args.refined_yaw {
                events.push_mark_unique(Event {
                    tick: step_start.t,
                    kind: EventKind::YawSwitch,
                });
            }
        }
        let mut s_out = WordStep {
            program: step.program,
            args: step.args,
            ticks: actual_ticks,
            expected_end_t: state.t,
            events,
            certificate: CorridorProof {
                label,
                min: step_lo,
                max: step_hi,
                version: provenance_hash,
            },
            end: StateAt::point(state.clone()),
            edit: step.edit,
        };
        if touch.is_some() {
            s_out.events.push_mark_unique(Event {
                tick: state.t,
                kind: EventKind::GoalBoundary,
            });
            certified_steps.push(s_out);
            break;
        }
        certified_steps.push(s_out);
    }

    if has_edits && restore_required {
        if let Some(d) = delta.as_ref() {
            for p in edited_positions {
                if d.state_id(p) != world.state_id(p) {
                    return Err((Refusal::StateOrContext, "restore-mismatch"));
                }
            }
        }
    }

    let predicted = state.t - from.t;
    if touch != Some(predicted) || state.health <= 0.0 {
        return Err((Refusal::StateOrContext, "touch-mismatch"));
    }
    if lo[0] > hi[0] {
        let p = [from.position.x.floor() as i32, from.position.y.floor() as i32, from.position.z.floor() as i32];
        lo = p;
        hi = p;
    }

    let route = Route {
        words: certified_steps,
        ticks: predicted,
        end: StateAt::point(state),
    };
    let reads = reads_map.into_iter().map(|((p, m), d)| (p, m, d)).collect();
    let cached = CachedRoute {
        kit_key,
        motion_key: motion_key(from),
        guard: Guard::of(from),
        damage_no_regen: total_damage_no_regen,
        route: route.clone(),
        lo,
        hi,
        reads,
        env: (world.difficulty(), world.natural_regeneration(), world.min_y(), world.fast_lava()),
    };
    Ok((route, cached))
}
