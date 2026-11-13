//! Job 2.1b: the generated rule's census, Julien's tracks, and the new families' shapes against the jar.
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use law::{Aabb, BoxWorld, CollisionContext, Input, PlayerState, Vec3, VoxelShape, World, tick_with_collision};
use world::{BlockState, WorldReader, WorldSnapshot, model};
fn fixture() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/world") }
fn state(raw: &str) -> BlockState {
    let (name, tail) = raw.split_once('[').unwrap_or((raw,""));
    let name = if name.contains(':') { name.to_owned() } else { format!("minecraft:{name}") };
    let mut properties = tail.trim_end_matches(']').split(',').filter(|p| !p.is_empty())
        .map(|p| { let (k,v) = p.split_once('=').unwrap(); (k.to_owned(),v.to_owned()) }).collect::<Vec<_>>();
    properties.sort(); BlockState { name, properties }
}

#[test]
fn inside_cell_collision_matches_jar_bottom_cap() {
    let block = model(&state("scaffolding[bottom=true,distance=1,waterlogged=false]")).unwrap();
    let mut world = BoxWorld::default();
    world.set([0,0,0], block.clone());
    for descending in [false, true] {
        let actual = world.collision_shape([0,0,0], CollisionContext { feet_y: 0.125, descending, fall_distance: 0.0, leather_boots: false });
        let expected = VoxelShape::boxes(&[Aabb::new(0.0,0.0,0.0,1.0,0.125,1.0)]);
        for axis in 0..3 { assert_eq!(actual.coords[axis], expected.coords[axis]); }
    }
}

#[test]
fn descending_climbable_caps_motion_and_bottom_plate_stops_it() {
    let mut world = BoxWorld::default();
    let input = Input { sneak: true, ..Default::default() };
    let mut player = PlayerState { position: Vec3::new(0.5, 0.5, 0.5), velocity: Vec3::new(0.0,-0.2,0.0), ..Default::default() };
    world.set([0,0,0], model(&state("scaffolding[bottom=false,distance=7,waterlogged=false]")).unwrap().clone());
    let (requested, _) = tick_with_collision(&mut player, &world, &input).unwrap();
    assert_eq!(requested.y.to_bits(), (-0.15f32 as f64).to_bits());
    world.set([0,0,0], model(&state("scaffolding[bottom=true,distance=1,waterlogged=false]")).unwrap().clone());
    player.position.y = 0.2;
    player.velocity.y = -0.2;
    tick_with_collision(&mut player, &world, &input);
    assert_eq!(player.position.y.to_bits(), 0.125f64.to_bits());
}

#[test]
fn census_puts_every_registry_state_in_one_bucket() {
    let census = include_str!("../data/census.tsv");
    let total: usize = census.lines().next().unwrap().strip_prefix("# ").unwrap().split(' ').next().unwrap().parse().unwrap();
    let mut modeled = include_str!("../data/blocks.tsv").lines().map(|l| l.split('\t').next().unwrap()).collect::<BTreeSet<_>>();
    let (mut buckets, mut seen, mut sum) = (BTreeMap::<&str,usize>::new(), BTreeSet::new(), 0);
    for line in census.lines().skip(1) {
        let [block, bucket, count] = line.split('\t').collect::<Vec<_>>()[..] else { panic!("census row {line}") };
        let count: usize = count.parse().unwrap();
        assert!(seen.insert((block,bucket)), "repeated census row {line}");
        if bucket == "modeled" { assert!(modeled.remove(block), "census-modeled {block} has no generator row"); }
        *buckets.entry(bucket).or_default() += count; sum += count;
    }
    assert!(modeled.is_empty(), "modeled blocks outside the census: {modeled:?}");
    assert_eq!(sum, total, "every registry state lands in exactly one bucket");
    let mut rows = buckets.into_iter().collect::<Vec<_>>(); rows.sort_by(|a,b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    for (bucket, count) in rows { println!("{count:6}  {bucket}"); }
}

#[test]
fn rule_flags_and_reasons() {
    let plate = model(&state("stone_pressure_plate[powered=false]")).unwrap();
    assert!(plate.world_hook && plate.shape.is_empty());
    assert!(!model(&state("stone[]")).unwrap().world_hook);
    assert!(model(&state("oak_fence[east=false,north=false,south=false,waterlogged=false,west=false]")).unwrap().fence);
    assert!(model(&state("oak_fence_gate[facing=north,in_wall=false,open=false,powered=false]")).unwrap().wall_or_gate);
    assert!(model(&state("cobblestone_wall[east=none,north=none,south=none,up=true,waterlogged=false,west=none]")).unwrap().wall_or_gate);
    assert_eq!(model(&state("bamboo[age=0,leaves=none,stage=0]")).err().as_deref(), Some("offset collision"));
    assert_eq!(model(&state("sweet_berry_bush[age=0]")).err().as_deref(), Some("hook entityInside SweetBerryBushBlock"));
    assert_eq!(model(&state("lava[level=0]")).err().as_deref(), Some("fluid other than water"));
    assert_eq!(model(&state("moving_piston[facing=north,type=normal]")).err().as_deref(), Some("non-local shape"));
    // A button's powered and face never change its model; an unknown value still names the block.
    assert!(std::ptr::eq(model(&state("stone_button[face=wall,facing=east,powered=true]")).unwrap(), model(&state("stone_button[face=floor,facing=north,powered=false]")).unwrap()));
    assert_eq!(model(&state("oak_fence[east=maybe,north=false,south=false,waterlogged=false,west=false]")).err().as_deref(), Some("state unknown to the 26.2 table of minecraft:oak_fence"));
}

#[test]
fn julien_tracks_load() {
    let mut reader = WorldReader::open(fixture()).unwrap();
    let mut refused_tracks = Vec::new();
    for line in include_str!("fixtures/tracks.txt").lines() {
        let f = line.split_whitespace().collect::<Vec<_>>();
        let v = f[1..].iter().map(|s| s.parse::<i32>().unwrap()).collect::<Vec<_>>();
        let (min, max) = ([v[0],v[1],v[2]], [v[3],v[4],v[5]]);
        if WorldSnapshot::load(fixture(), min, max).is_ok() { println!("{}: loaded {min:?}..{max:?}", f[0]); continue; }
        let mut refused = BTreeMap::new();
        for z in min[2]..=max[2] { for y in min[1]..=max[1] { for x in min[0]..=max[0] {
            let s = reader.block_state([x,y,z]).unwrap();
            if let Err(reason) = model(s) { *refused.entry((s.to_string(), reason)).or_insert(0) += 1; }
        }}}
        println!("{}: refused {refused:?}", f[0]);
        refused_tracks.push(f[0]);
    }
    assert!(refused_tracks.is_empty(), "tracks with refused states: {refused_tracks:?}");
}

#[test]
fn new_family_shapes_equal_the_jar_bit_for_bit() {
    let mut checked = 0;
    for line in include_str!("fixtures/shapes.txt").lines() {
        let [raw, context, boxes] = line.split('\t').collect::<Vec<_>>()[..] else { panic!("shape row {line}") };
        let jar = boxes.split(';').filter(|b| !b.is_empty()).map(|b| {
            let v = b.split(',').map(|h| f64::from_bits(u64::from_str_radix(h,16).unwrap())).collect::<Vec<_>>();
            Aabb::new(v[0],v[1],v[2],v[3],v[4],v[5])
        }).collect::<Vec<_>>();
        let block = model(&state(raw)).unwrap();
        // ModelTable's contexts: empty, feet above, feet below, descending with feet above.
        let law = match context {
            "0" => block.shape.clone(),
            c => {
                let (descending, feet_y) = [(false,1.0),(false,-2.0),(true,1.0),(false,0.0)][c.parse::<usize>().unwrap()-1];
                let mut world = BoxWorld::default(); world.set([0,0,0], block.clone());
                world.collision_shape([0,0,0], CollisionContext { feet_y, descending, fall_distance: 0.0, leather_boots: false }).clone()
            }
        };
        let expected = VoxelShape::boxes(&jar);
        for axis in 0..3 {
            let bits = |s: &VoxelShape| s.coords[axis].iter().map(|v| v.to_bits()).collect::<Vec<_>>();
            assert_eq!(bits(&law), bits(&expected), "{raw} context {context} axis {axis}");
        }
        let n = |a: usize| expected.coords[a].len().saturating_sub(1);
        for x in 0..n(0) { for y in 0..n(1) { for z in 0..n(2) {
            assert_eq!(law.is_full([x,y,z]), expected.is_full([x,y,z]), "{raw} context {context} cell {x} {y} {z}");
        }}}
        checked += 1;
    }
    assert_eq!(checked, 10*5);
}
