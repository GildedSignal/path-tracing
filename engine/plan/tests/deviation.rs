mod replay {
    include!("../../law/tests/replay.rs");
    #[test]
    fn solve_scripts_reach_the_saved_track_finish() {
        for line in include_str!("../../world/tests/fixtures/tracks.txt").lines().filter(|s|
            matches!(s.split_whitespace().next(),Some("straight100"|"straight300"|"straight800"|"stonesimple"))) {
            let track=plan::flight::track(line).unwrap();
            let file=std::env::temp_dir().join(format!("solve-{}-{}.jsonl",std::process::id(),track.name));
            let (predicted,_)=plan::flight::solve_track(&track,&file).unwrap();
            let script=std::fs::read_to_string(&file).unwrap();
            std::fs::remove_file(file).unwrap();
            let mut lines=script.lines();
            let header=Object::parse(lines.next().unwrap());
            let bits:Vec<u64>=array(header.required("start_bits")).iter().map(|s|s.parse().unwrap()).collect();
            let mut state=PlayerState::from_bits(&bits).unwrap();
            assert_eq!(state.saturation,20.0,"settled bot saturation");
            let goal=plan::Goal {cell:track.b};
            let mut first=None;let mut last_on_a=state.t;
            for (index,line) in lines.enumerate() {
                let row=Object::parse(line);
                assert_eq!(row.required("t").parse::<usize>().unwrap(),index);
                if state.on_ground && state.main_supporting_block==Some(track.a) {last_on_a=state.t;}
                let before=state.bounding_box();
                law::try_tick(&mut state,&track.world,&input(&row)).unwrap();
                if goal.touched_boxes(before,state.bounding_box()) {first.get_or_insert(state.t);}
            }
            println!("SCRIPT {} predicted={predicted} first_touch={first:?} last_on_a={last_on_a}",track.name);
            assert_eq!(first,Some(u64::from(predicted)));
            assert_eq!(state.t-last_on_a,u64::from(predicted));
        }
    }
    #[test]
    fn l0_deviation_matches_law_and_catches_edited_input() {
        use std::fmt::Write;
        let root = std::path::Path::new(r"D:\path-tracing-evidence\l0-veto-20261111\audit\clients\bot\pathtracing");
        if !root.exists() { eprintln!("L0 evidence unavailable"); return; }
        let name = "l0_20261111T1451167890697Z_mixed-seam_1";
        let recording = std::fs::read_to_string(root.join("recordings").join(format!("{name}.jsonl"))).unwrap();
        let fixture = std::fs::read_to_string(root.join("fixtures").join(format!("{name}.world.jsonl"))).unwrap();
        let make_plan = |fault: bool| {
            let header = Object::parse(recording.lines().next().unwrap()); let full_fixture = format!("{}\n{}", fixture, header.required("initialPlayer"));
            let (world, mut state, _) = parse_world(&full_fixture);
            let mut plan = format!("{{\"start\":[{},{},{},{},{}],\"start_bits\":{:?}}}\n", state.position.x, state.position.y, state.position.z, state.yaw, state.pitch, state.bits());
            for line in recording.lines().skip(1) {
                let row = Object::parse(line); if row.get("type").is_some() { break; }
                state.profile.speed_modifiers = speed_modifiers(&row);
                if row.get("startSprinting").is_some() { state.sprinting = row.boolean("startSprinting"); state.sprint_speed_modifier = row.boolean("startSprintSpeedModifier"); }
                if row.get("startPose").is_some() { state.pose = pose(row.string("startPose")); }
                if row.get("startSwimming").is_some() { state.swimming = row.boolean("startSwimming"); }
                state.ticks_frozen = row.required("ticksFrozen").parse().unwrap();
                let t: usize = row.required("t").parse().unwrap(); let mut applied = input(&row.object("inputs"));
                if fault && t == 3 { applied.forward = !applied.forward; }
                law::tick(&mut state, &world, &applied);
                writeln!(plan, "{{\"t\":{t},\"forward\":{},\"back\":{},\"left\":{},\"right\":{},\"jump\":{},\"sneak\":{},\"sprint\":{},\"yaw\":{},\"pitch\":{},\"yaw_bits\":{},\"pitch_bits\":{},\"position_bits\":[{},{},{}],\"velocity_bits\":[{},{},{}],\"onGround\":{},\"state_bits\":{:?}}}", applied.forward, applied.back, applied.left, applied.right, applied.jump, applied.sneak, applied.sprint, applied.yaw, applied.pitch, applied.yaw.to_bits(), applied.pitch.to_bits(), state.position.x.to_bits(), state.position.y.to_bits(), state.position.z.to_bits(), state.velocity.x.to_bits(), state.velocity.y.to_bits(), state.velocity.z.to_bits(), state.on_ground, state.bits()).unwrap();
            }
            plan
        };
        let ok = plan::deviation::compare(&make_plan(false), &recording).unwrap();
        assert_eq!(ok.first_deviation, None);
        let bad = plan::deviation::compare(&make_plan(true), &recording).unwrap().first_deviation.unwrap();
        assert_eq!(bad.tick, 3);
        assert!(bad.fields.iter().any(|f| f.field.starts_with("position.") || f.field.starts_with("velocity.")), "{bad:?}");
        let plan = make_plan(false);
        let mut lines: Vec<String> = recording.lines().map(str::to_owned).collect();
        lines[1] = lines[1].replacen("\"onGround\":true", "\"onGround\":false", 1)
            .replacen("\"horizontalCollision\":false", "\"horizontalCollision\":true", 1);
        lines[1].pop();
        lines[1].push_str(",\"anomaly\":[{\"type\":\"ClientboundPlayerPositionPacket\"},{\"type\":\"ClientboundSetEntityMotionPacket\"},{\"type\":\"ClientboundExplodePacket\"}]}");
        let report = plan::deviation::compare(&plan, &lines.join("\n")).unwrap();
        let fields = &report.first_deviation.unwrap().fields;
        assert!(fields.iter().any(|f| f.field == "on_ground"));
        assert!(fields.iter().any(|f| f.field == "horizontal_collision"));
        assert_eq!((report.corrections, report.pushes), (1, 2));
        assert!(plan::deviation::compare(&plan, &recording.replace("\"complete\":true", "\"complete\":false")).is_err());
    }
}

