//! Stage 3 Minton anti-regression, Pareto dominance, and effect-index tests.
use law::{PlayerState, World, codec::Interner};
use plan::{
    Kit, PlanOutcome, Stage, flight::track, solve,
    source::{GeneratorSuperflatSource, ObservedWorld},
    store::GeneratorStore,
    word_store::{AimRule, DefectSig, EntryClass, ObservedStep, WordStore},
};

fn learn_ticks(res: &plan::solve::SolveResult) -> u64 {
    res.stages.iter().filter_map(|s| match s {
        Stage::Defect { learn_ticks, .. } => Some(*learn_ticks),
        _ => None,
    }).sum()
}

#[test]
fn effect_index_pareto_and_minton_suppression() {
    let mut ws = WordStore::default();
    let base = PlayerState::default();
    let entry = EntryClass::of(&base);

    // Promote a fast word and a strictly dominated slower word with identical (entry, exit, delta_y).
    let slow_step = ObservedStep {
        program: 2,
        aim: AimRule::GoalOffset(4096),
        ticks: 14,
        entry,
        exit: entry,
        defect_fine: 777,
        defect_coarse: 888,
        delta_along: 3.0,
        delta_y: 0,
    };
    let fast_step = ObservedStep {
        program: 1,
        aim: AimRule::GoalOffset(0),
        ticks: 10,
        entry,
        exit: entry,
        defect_fine: 777,
        defect_coarse: 888,
        delta_along: 3.5,
        delta_y: 0,
    };
    ws.promote_route(&[slow_step]);
    ws.promote_route(&[fast_step]);

    let props = ws.propose_for_effect(DefectSig { fine: 777, coarse: 888, rot: 0 }, entry, Some((0, 3.0)), false);
    assert_eq!(props.len(), 1, "Pareto pruning must drop strictly dominated slower word");
    let fast_id = props[0].0;
    assert_eq!(ws.word(fast_id).ticks_max, 10);

    // Promote an unrelated word and fail it 6 times via record_outcome.
    let bad_step = ObservedStep {
        program: 3,
        aim: AimRule::GoalOffset(-4096),
        ticks: 8,
        entry,
        exit: entry,
        defect_fine: 111,
        defect_coarse: 222,
        delta_along: 2.2,
        delta_y: 0,
    };
    ws.promote_route(&[bad_step]);
    let bad_id = ws
        .propose_for_effect(DefectSig { fine: 111, coarse: 222, rot: 0 }, entry, Some((0, 2.2)), false)
        .into_iter()
        .find(|&(w, known)| known && w != fast_id)
        .expect("bad word present on exact fine match")
        .0;

    for _ in 0..6 {
        ws.record_outcome(bad_id, false, 40);
    }

    // Suppressed as a coarse/effect guess on unseen defects, retained on exact fine matches.
    let guess_props = ws.propose_for_effect(DefectSig { fine: 999, coarse: 222, rot: 0 }, entry, Some((0, 2.2)), false);
    assert!(!guess_props.iter().any(|&(w, _)| w == bad_id), "Minton gate must suppress failed guess");

    let fine_props = ws.propose_for_effect(DefectSig { fine: 111, coarse: 222, rot: 0 }, entry, Some((0, 2.2)), false);
    assert!(fine_props.iter().any(|&(w, known)| known && w == bad_id), "exact fine match must stay available");

    let pruned = ws.consolidate();
    assert!(pruned >= 1, "consolidate must prune Minton-suppressed guess buckets");
}

#[test]
fn minton_warm_store_never_regresses_cold_across_tracks() {
    let tracks_txt = include_str!("../../world/tests/fixtures/tracks.txt");
    let names = [
        "grassjumpeasy",
        "fencemaze",
        "trapdoorparkour",
        "butoonpressureplates",
        "doorparkourblue",
        "birchparkourmedium",
    ];
    let loaded: Vec<_> = names
        .iter()
        .map(|&name| {
            let line = tracks_txt.lines().find(|l| l.split_whitespace().next() == Some(name)).expect("track in tracks.txt");
            let t = track(line).expect("load track");
            let source = GeneratorSuperflatSource::snapshot(&t.world, t.a).unwrap();
            let obs0 = ObservedWorld::new(&t.world);
            let top = obs0.block(t.a).shape.coords[1].last().copied().unwrap() + f64::from(t.a[1]);
            let mut start = plan::settle(&plan::Still(&t.world), [f64::from(t.a[0]) + 0.5, top, f64::from(t.a[2]) + 0.5], 0).unwrap();
            start.saturation = 20.0;
            (name, t, source, start)
        })
        .collect();

    let cold_baselines: Vec<(u64, u64)> = loaded
        .iter()
        .map(|(name, t, source, start)| {
            let store = GeneratorStore::default();
            let mut codec = Interner::default();
            let res = solve(start, t.goal(), Kit::BareLegs, source, &store, &mut codec, u64::MAX);
            let PlanOutcome::Ready(plan) = res.outcome else {
                panic!("cold solve failed on {name}: {:?}", res.outcome);
            };
            let law_ticks = store.words.borrow().work.law_ticks;
            (plan.ticks, law_ticks)
        })
        .collect();

    for (i, (name, t, source, start)) in loaded.iter().enumerate() {
        let (cold_t, cold_law) = cold_baselines[i];
        let store = GeneratorStore::default();
        let mut codec = Interner::default();
        for (j, (other_name, ot, osource, ostart)) in loaded.iter().enumerate() {
            if i == j {
                continue;
            }
            let r = solve(ostart, ot.goal(), Kit::BareLegs, osource, &store, &mut codec, u64::MAX);
            assert!(matches!(r.outcome, PlanOutcome::Ready(_)), "warming track {other_name} failed");
        }
        store.consolidate_idle();

        let law_before = store.words.borrow().work.law_ticks;
        let res_warm = solve(start, t.goal(), Kit::BareLegs, source, &store, &mut codec, u64::MAX);
        let warm_law = store.words.borrow().work.law_ticks - law_before;
        let PlanOutcome::Ready(plan_warm) = res_warm.outcome else {
            panic!("warm_other solve failed on {name}: {:?}", res_warm.outcome);
        };
        assert!(
            plan_warm.ticks <= cold_t,
            "{name}: warm_other T={} exceeded cold T={}",
            plan_warm.ticks,
            cold_t
        );
        assert!(
            warm_law <= cold_law + 1_500,
            "{name}: Minton gate violated: warm_other law_ticks={warm_law} exceeded cold={cold_law} + 1500"
        );

        let res_repeat = solve(start, t.goal(), Kit::BareLegs, source, &store, &mut codec, u64::MAX);
        let PlanOutcome::Ready(plan_repeat) = &res_repeat.outcome else {
            panic!("warm_same repeat failed on {name}: {:?}", res_repeat.outcome);
        };
        assert_eq!(learn_ticks(&res_repeat), 0, "{name}: repeat solve must have 0 learn_ticks");
        assert!(plan_repeat.ticks <= cold_t, "{name}: repeat T={} exceeded cold T={}", plan_repeat.ticks, cold_t);
    }
}
