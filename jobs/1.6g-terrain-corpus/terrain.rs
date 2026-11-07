use super::*;
use std::collections::BTreeMap;

fn block_pos(p: Vec3) -> BlockPos {
    [p.x, p.y, p.z].map(|coordinate| coordinate.floor() as i32)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Bounds(pub BlockPos, pub BlockPos);

impl Bounds {
    pub(super) fn contains(self, pos: BlockPos) -> bool {
        (0..3).all(|axis| (self.0[axis]..=self.1[axis]).contains(&pos[axis]))
    }

    pub(super) fn contains_chunk(self, x: i32, z: i32) -> bool {
        ((self.0[0] >> 4)..=(self.1[0] >> 4)).contains(&x) &&
            ((self.0[2] >> 4)..=(self.1[2] >> 4)).contains(&z)
    }
}

pub(super) struct Terrain {
    pub pre_world: PathBuf,
    pub envelope: Bounds,
    pub mixed: bool,
}

struct Export<'a> {
    bounds: Bounds,
    world: (i32, u8, bool, bool),
    blocks: BTreeMap<BlockPos, &'a str>,
    payload: &'a str,
}

impl<'a> Export<'a> {
    fn parse(text: &'a str) -> Self {
        let (payload, end) = text.trim_end().rsplit_once('\n').expect("missing world export trailer");
        let end = Object::parse(end);
        assert_eq!(end.string("kind"), "worldEnd", "incomplete world export");
        let mut lines = payload.lines();
        let world = Object::parse(lines.next().expect("missing world header"));
        assert_eq!(world.string("kind"), "world");
        let bounds = Bounds(position(world.required("min")), position(world.required("max")));
        let mut blocks = BTreeMap::new();
        for line in lines {
            let row = Object::parse(line);
            assert_eq!(row.string("kind"), "block");
            let pos = position(row.required("at"));
            assert!(bounds.contains(pos) && blocks.insert(pos, line).is_none(), "world block outside bounds or repeated: {pos:?}");
        }
        assert_eq!(end.required("blocks").parse::<usize>().unwrap(), blocks.len(), "world export block count");
        Self { bounds, world: (world.required("minY").parse().unwrap(), world.required("difficulty").parse().unwrap(),
            world.boolean("naturalRegeneration"), world.boolean("fastLava")), blocks, payload }
    }
}

impl Terrain {
    pub fn world<'a>(&self, after: &'a str, initial: &Object<'_>, rows: &[Object<'_>]) -> (&'a str, Bounds) {
        let before = fs::read_to_string(&self.pre_world)
            .unwrap_or_else(|err| panic!("{}: {err}", self.pre_world.display()));
        let before = Export::parse(&before);
        let after = Export::parse(after);
        assert_eq!(before.bounds, self.envelope, "pre-run world envelope");
        assert_eq!(before.world, after.world, "world settings changed");

        let mut min = [i32::MAX; 3];
        let mut max = [i32::MIN; 3];
        for row in std::iter::once(initial).chain(rows) {
            let p = vector(&row.object("position"));
            assert!(p.x.is_finite() && p.y.is_finite() && p.z.is_finite(), "nonfinite observed position {p:?}");
            let pos = block_pos(p);
            for axis in 0..3 {
                min[axis] = min[axis].min(pos[axis] - 3);
                max[axis] = max[axis].max(pos[axis] + 3);
            }
        }
        min[1] = min[1].max(before.world.0);
        let witness = Bounds(min, max);
        assert_eq!(after.bounds, witness, "post-run world does not cover the replay footprint");
        assert!(before.bounds.contains(min) && before.bounds.contains(max), "replay footprint left the pre-run world: {witness:?}");
        let overlap = before.blocks.iter().filter(|(pos, _)| witness.contains(**pos));
        let mut post = after.blocks.iter();
        for (pos, row) in overlap {
            assert_eq!(post.next(), Some((pos, row)), "pre/post world diverged at {pos:?}");
        }
        assert!(post.next().is_none(), "post-run world has extra blocks");
        (after.payload, witness)
    }
}

// The mixed course is one block wide. An observed floor, clipped upward
// displacement, or water flag can be assigned to the corresponding route cell.
pub(super) fn mixed_contacts(world_text: &str, initial: &Object<'_>, rows: &[Object<'_>]) {
    let world = Export::parse(world_text);
    let block = |pos| world.blocks.get(&pos).map(|line| Object::parse(line));
    let mut ice = None;
    let mut step = None;
    let mut seam = None;
    let mut bonk = None;
    let mut ice_lip = None;
    let mut flow = None;
    let mut previous = vector(&initial.object("position"));
    let half_width = f64::from(0.6f32) / 2.0;
    for (tick, row) in rows.iter().enumerate() {
        let here = vector(&row.object("position"));
        let [x, y, z] = block_pos(here);
        let ground = row.boolean("onGround");
        let floor = block([x, y - i32::from(here.y == f64::from(y)), z]).map(|floor| floor.string("block"));
        if ground && ice.is_none() && floor == Some("minecraft:ice") {
            ice = Some(tick);
        }
        if bonk.is_some() && ice_lip.is_none() && ground && (4114..=4115).contains(&z) &&
            floor == Some("minecraft:ice") && !row.boolean("isInWater") {
            ice_lip = Some(tick);
        }
        if let [collision] = array(row.required("collisionSteps")).as_slice() {
            let collision = Object::parse(collision);
            let requested = collision.object("requested").f64("y");
            let resolved = collision.object("result").f64("y");
            if step.is_none() && ground && here.y == -59.5 && requested <= 0.0 && resolved == 0.5 &&
                previous.z + half_width <= 4112.0 && here.z + half_width > 4112.0 &&
                block([x, -60, 4112]).is_some_and(|slab| slab.string("block") == "minecraft:stone_slab") {
                step = Some(tick);
            }
            if seam.is_some() && bonk.is_none() && requested > 0.0 && (0.0..requested).contains(&resolved) &&
                row.boolean("verticalCollision") && block([x, -58, z]).is_some_and(|roof|
                    roof.string("block") == "minecraft:stone_slab" && roof.string("state").contains("type=top")) {
                bonk = Some(tick);
            }
        }
        if seam.is_none() && previous.z < 4112.0 && here.z >= 4112.0 && here.y >= -59.5 &&
            floor == Some("minecraft:stone_slab") {
            seam = Some(tick);
        }
        if flow.is_none() && row.boolean("isInWater") && block(block_pos(previous)).is_some_and(|cell|
            cell.string("fluid") == "water" && vector(&cell.object("flow")) != Vec3::default()) {
            flow = Some(tick);
        }
        previous = here;
    }
    let contacts = format!("ice {ice:?}, step {step:?}, seam {seam:?}, ceiling {bonk:?}, ice lip {ice_lip:?}, flowing water {flow:?}");
    assert!(matches!((ice, step, seam, bonk, ice_lip, flow), (Some(i), Some(s), Some(c), Some(b), Some(l), Some(w))
        if i < s && s <= c && c < b && b < l && l < w), "mixed contact refusal: {contacts}");
    println!("Mixed contacts: {contacts}");
}

#[test]
fn authoritative_terrain_case() {
    let env = |key| std::env::var(key).unwrap_or_else(|_| panic!("missing {key}"));
    let file = PathBuf::from(env("L0_TERRAIN_RECORDING"));
    let ticks = env("L0_TERRAIN_TICKS").parse().unwrap();
    let prefix = env("L0_TERRAIN_PREFIX").parse().unwrap();
    assert!(prefix <= ticks);
    let bounds = env("L0_TERRAIN_ENVELOPE");
    let (min, max) = bounds.split_once(';').unwrap();
    let terrain = Terrain { pre_world: env("L0_TERRAIN_PRE_WORLD").into(),
        envelope: Bounds(position(min), position(max)), mixed: env("L0_TERRAIN_MIXED") == "true" };
    verify(&file, &env("L0_TERRAIN_RATE"), ticks, None, prefix, Some(&terrain));
}
