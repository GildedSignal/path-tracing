//! Executable kill test & live-flight regression gates for `kit-edges-in-field`.
use law::{PlayerState, World, codec::Interner};
use plan::{
    Kit, KitSpec, PlanOutcome, Stage,
    flight::track,
    solve,
    source::{GeneratorSuperflatSource, ObservedWorld},
    store::GeneratorStore,
};

fn settle_at_a(t: &plan::flight::Track) -> (GeneratorSuperflatSource, PlayerState) {
    let source = GeneratorSuperflatSource::snapshot(&t.world, t.a).unwrap();
    let obs = ObservedWorld::new(&t.world);
    let top = obs.block(t.a).shape.coords[1].last().copied().unwrap() + f64::from(t.a[1]);
    let mut s = plan::settle(
        &plan::Still(&t.world),
        [f64::from(t.a[0]) + 0.5, top, f64::from(t.a[2]) + 0.5],
        0,
    )
    .unwrap();
    s.saturation = 20.0;
    (source, s)
}

fn strip_line_comments(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    for line in src.lines() {
        let code = match line.find("//") {
            Some(i) => &line[..i],
            None => line,
        };
        out.push_str(code);
        out.push('\n');
    }
    out
}

fn live_survival_kit() -> Kit {
    Kit::Carried(KitSpec { interact: true, can_break: true, ..KitSpec::empty() })
}

#[test]
fn kill_test_no_tool_verbs_in_search_code() {
    let forbidden = [
        "KitOp::Interact",
        "KitOp::Break",
        "KitOp::Place",
        "KitOp::BucketEmpty",
        "KitOp::BucketPickup",
        "VerbKind::",
        "KitVerb::",
        "WRITE_SHAPE_",
        "WRITE_FALL_RESET",
        "is_hand_toggleable",
        "water_source_state",
        "powder_snow_state",
        "toggle_open_state",
        "can_place_at",
    ];
    let files = [
        ("solve.rs", include_str!("../src/solve.rs")),
        ("cut.rs", include_str!("../src/cut.rs")),
        ("defect.rs", include_str!("../src/defect.rs")),
    ];
    for (name, raw) in files {
        let code = strip_line_comments(raw);
        for pat in forbidden {
            assert!(!code.contains(pat), "kill test failed: `{name}` references `{pat}` outside field edge interface");
        }
    }
}

#[test]
fn doorparkourblue_survival_zero_edits() {
    let txt = include_str!("../../world/tests/fixtures/tracks.txt");
    let line = txt.lines().find(|l| l.split_whitespace().next() == Some("doorparkourblue")).unwrap();
    let t = track(line).unwrap();
    let (src, s) = settle_at_a(&t);
    let mut codec = Interner::default();
    let r = solve(&s, t.goal(), live_survival_kit(), &src, &GeneratorStore::default(), &mut codec, u64::MAX);
    let PlanOutcome::Ready(plan) = r.outcome else {
        panic!("doorparkourblue survival failed: {:?}", r.outcome);
    };
    let edits = plan.words.iter().filter(|w| w.edit.is_some()).count();
    assert_eq!(edits, 0, "doorparkourblue must take 0 edits");
    assert!(plan.ticks <= 90, "doorparkourblue T={} exceeded 90", plan.ticks);
}

#[test]
fn doorcorridor_survival_under_cap() {
    let txt = include_str!("../../world/tests/fixtures/tracks.txt");
    let line = txt.lines().find(|l| l.split_whitespace().next() == Some("doorcorridor")).unwrap();
    let t = track(line).unwrap();
    let (src, s) = settle_at_a(&t);
    let mut codec = Interner::default();
    let r = solve(&s, t.goal(), live_survival_kit(), &src, &GeneratorStore::default(), &mut codec, u64::MAX);
    let learn: u64 = r.stages.iter().filter_map(|st| match st { Stage::Defect { learn_ticks, .. } => Some(*learn_ticks), _ => None }).sum();
    let PlanOutcome::Ready(plan) = r.outcome else {
        panic!("doorcorridor survival failed: {:?}", r.outcome);
    };
    assert!(learn < 132_000, "doorcorridor learn={learn} exceeded 132_000");
    assert!(plan.ticks <= 36, "doorcorridor T={} regressed", plan.ticks);
}

#[test]
fn warm_store_does_not_degrade_grassjumpeasy() {
    let txt = include_str!("../../world/tests/fixtures/tracks.txt");
    let line_of = |name: &str| txt.lines().find(|l| l.split_whitespace().next() == Some(name)).unwrap();
    let mut codec = Interner::default();
    let g_track = track(line_of("grassjumpeasy")).unwrap();
    let (g_src, g_start) = settle_at_a(&g_track);
    let cold = match solve(&g_start, g_track.goal(), live_survival_kit(), &g_src, &GeneratorStore::default(), &mut codec, u64::MAX).outcome {
        PlanOutcome::Ready(p) => p.ticks,
        other => panic!("grassjumpeasy cold failed: {other:?}"),
    };
    let store = GeneratorStore::default();
    for _round in 0..2 {
        for name in ["flowers", "stonesimple", "fencemaze", "trapdoorparkour", "doorparkourblue", "grassjumpeasy", "stonestaircase", "doorcorridor"] {
            let t = track(line_of(name)).unwrap();
            let (src, s) = settle_at_a(&t);
            let _ = solve(&s, t.goal(), live_survival_kit(), &src, &store, &mut codec, u64::MAX);
        }
    }
    let warm = match solve(&g_start, g_track.goal(), live_survival_kit(), &g_src, &store, &mut codec, u64::MAX).outcome {
        PlanOutcome::Ready(p) => p.ticks,
        other => panic!("grassjumpeasy warm failed: {other:?}"),
    };
    assert!(warm <= cold && warm <= 40, "grassjumpeasy warm T={warm} degraded beyond cold T={cold}");
}
