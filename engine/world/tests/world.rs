use std::{collections::BTreeMap, path::PathBuf};
use law::{Aabb, Block, BoxWorld, CollisionContext, Input, PlayerState, Vec3, World, tick};
use world::{BlockState, WorldReader, WorldSnapshot};
fn fixture() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/world") }
fn state(raw: &str) -> BlockState {
    let (name, tail) = raw.split_once('[').unwrap_or((raw,""));
    let name = if name.contains(':') { name.to_owned() } else { format!("minecraft:{name}") };
    let mut properties = tail.trim_end_matches(']').split(',').filter(|p| !p.is_empty()).map(|p| {
        let (k,v) = p.split_once('=').unwrap(); (k.to_owned(),v.to_owned())
    }).collect::<Vec<_>>();
    properties.sort(); BlockState { name, properties }
}
fn defaults(raw: &str, defaults: &BTreeMap<String,BlockState>) -> BlockState {
    let patch = state(raw);
    let mut base = defaults.get(&patch.name).unwrap_or_else(|| panic!("missing independently extracted default for {raw}")).clone();
    for (key,value) in patch.properties {
        let prop = base.properties.iter_mut().find(|(k,_)|k==&key).unwrap(); prop.1=value;
    }
    base
}
#[test]
fn superflat_layers_from_copied_world() {
    let mut world = WorldReader::open(fixture()).unwrap();
    assert_eq!(world.data_version(), 4903);
    for [x,z] in [[0,0],[15,15],[16,16],[31,31]] {
        for (y,expected) in [(-64,"bedrock"),(-63,"dirt"),(-62,"dirt"),(-61,"grass_block[snowy=false]"),(-60,"air"),(0,"air"),(319,"air"),(320,"void_air"),(-65,"void_air")] {
            assert_eq!(*world.block_state([x,y,z]).unwrap(),state(expected),"at {x} {y} {z}");
        }
    }
}

#[test]
fn every_cell_of_four_courses_against_main_fill_commands() {
    let defaults_by_name = include_str!("../data/defaults.txt").lines().map(|s| {let s=state(s); (s.name.clone(),s)}).collect::<BTreeMap<_,_>>();
    let mut world=WorldReader::open(fixture()).unwrap();
    let commands=include_str!("fixtures/commands.txt");
    let mut checked=0;
    let mut course_count=0;
    let mut all_mismatches=Vec::new();
    for course in commands.split("# ").filter(|c|!c.trim().is_empty()) {
        let mut lines=course.lines(); let name=lines.next().unwrap().trim();
        let mut expected=BTreeMap::new();
        for line in lines.filter(|l|!l.trim().is_empty()) {
            let args=line.split_whitespace().collect::<Vec<_>>();
            let pos = |i:usize| [args[i].parse::<i32>().unwrap(),args[i+1].parse().unwrap(),args[i+2].parse().unwrap()];
            let (min,max,block)=match args[0] {
                "fill" => (pos(1),pos(4),defaults(args[7],&defaults_by_name)),
                "setblock" => (pos(1),pos(1),defaults(args[4],&defaults_by_name)),
                _ => panic!("unexpected course command {line}"),
            };
            for z in min[2]..=max[2] { for y in min[1]..=max[1] { for x in min[0]..=max[0] {
                expected.insert([x,y,z],block.clone());
            }}}
        }
        assert_eq!(expected.len(),32*20*32,"whole box for {name}");
        let mut mismatches=Vec::new();
        for (p,expected) in &expected {
            let actual=world.block_state(*p).unwrap();
            if actual != expected { mismatches.push(format!("{name} {p:?}: expected {expected}; actual {actual}")); }
        }
        println!("{name}: {} cells, {} mismatches",expected.len(),mismatches.len());
        checked+=expected.len(); course_count+=1; all_mismatches.extend(mismatches);
    }
    println!("checked {course_count} courses, {checked} cells, {} mismatches",all_mismatches.len());
    assert!(all_mismatches.is_empty(),"differences from main's construction commands:\n{}",all_mismatches.join("\n"));
    assert!(course_count>=3);
}

#[test]
fn snapshot_bridge_shapes_and_flat_tick() {
    let snapshot=WorldSnapshot::load(fixture(),[4096,-64,4096],[4127,-45,4127]).unwrap();
    assert_eq!(snapshot.difficulty(),0);
    assert!(snapshot.natural_regeneration());
    assert!(!snapshot.has_fluid());
    assert!(!snapshot.has_inside_effects());
    assert!(snapshot.block([4104,-64,4104]).shape.is_block());
    assert!(snapshot.block([4104,-60,4104]).shape.is_empty());
    let mut reference=BoxWorld::default();
    reference.difficulty=0;
    reference.fill([4096,-64,4096],[4127,-61,4127],Block::full());
    let mut a=PlayerState {position:Vec3::new(4112.5,-60.0,4100.5),on_ground:true,..PlayerState::default()};
    let mut b=a.clone();
    let input=Input {forward:true,sprint:true,..Input::default()};
    for _ in 0..16 {
        tick(&mut a,&snapshot,&input); tick(&mut b,&reference,&input);
        assert_eq!(a.position,b.position); assert_eq!(a.velocity,b.velocity);
    }
    assert!(std::panic::catch_unwind(|| snapshot.block([4128,-60,4104])).is_err());
    assert!(WorldSnapshot::load_with_limit(fixture(),[0,-64,0],[31,-45,31],1).is_err());
}
#[test]
fn snapshot_bridge_stairs_and_slabs_from_course() {
    let snapshot=WorldSnapshot::load(fixture(),[4224,-64,4096],[4255,-45,4127]).unwrap();
    let context=CollisionContext {feet_y:-59.0,descending:false,fall_distance:0.0,leather_boots:false};
    let slab=snapshot.collision_shape([4225,-60,4102],context);
    assert!(slab.intersects(Aabb::new(0.1,0.1,0.1,0.9,0.4,0.9)));
    assert!(!slab.intersects(Aabb::new(0.1,0.6,0.1,0.9,0.9,0.9)));
    let stair=snapshot.collision_shape([4225,-60,4104],context);
    assert!(stair.intersects(Aabb::new(0.1,0.6,0.6,0.9,0.9,0.9)));
    assert!(!stair.intersects(Aabb::new(0.1,0.6,0.1,0.9,0.9,0.4)));
}

#[test]
fn fixed_cache_bound_evicts_and_reloads() {
    let points=[[0,-64,0],[16,-64,0],[32,-64,0]];
    let mut largest=0;
    for p in points {
        let mut reader=WorldReader::open(fixture()).unwrap();
        reader.block_state(p).unwrap(); largest=largest.max(reader.cache_bytes());
    }
    let mut reader=WorldReader::with_cache_limit(fixture(),largest).unwrap();
    for p in points.into_iter().chain(points.into_iter().rev()) {
        assert_eq!(reader.block_state(p).unwrap().name,"minecraft:bedrock");
        assert!(reader.cache_bytes()<=reader.cache_limit());
        assert_eq!(reader.cached_chunks(),1,"one chunk fits the measured one-chunk budget");
    }
    assert!(WorldReader::with_cache_limit(fixture(),0).is_err());
}
#[test]
fn snapshot_refuses_an_unmodeled_real_state() {
    let error=WorldSnapshot::load(fixture(),[10,-58,2],[10,-58,2]).err().expect("fences are outside the supported model families");
    assert!(matches!(error,world::LoadError::Unmodeled {position:[10,-58,2],..}));
    let message=error.to_string();
    assert!(message.contains("minecraft:cherry_fence"));
    assert!(message.contains("waterlogged=false"));
}
#[test]
fn probe_point_range_and_failure() {
    let point=std::process::Command::new(env!("CARGO_BIN_EXE_world-probe")).arg(fixture()).args(["0","-64","0"]).output().unwrap();
    assert!(point.status.success());
    assert_eq!(String::from_utf8(point.stdout).unwrap().trim(),"minecraft:bedrock");
    let range=std::process::Command::new(env!("CARGO_BIN_EXE_world-probe")).arg(fixture()).args(["0","-64","0","0","-60","0"]).output().unwrap();
    assert!(range.status.success());
    let lines=String::from_utf8(range.stdout).unwrap();
    assert_eq!(lines.lines().count(),5);
    assert!(lines.contains("0 -61 0 minecraft:grass_block[snowy=false]"));
    let missing=std::process::Command::new(env!("CARGO_BIN_EXE_world-probe")).arg(fixture()).args(["1000000","0","1000000"]).output().unwrap();
    assert!(!missing.status.success());
    assert!(String::from_utf8(missing.stderr).unwrap().contains("missing generated chunk"));
}
