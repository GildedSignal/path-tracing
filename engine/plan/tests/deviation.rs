mod replay {
    include!("../../law/tests/replay.rs");
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
                state.speed_modifiers = speed_modifiers(&row);
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

