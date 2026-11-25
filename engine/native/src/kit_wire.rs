//! The carried kit, read from the bot's own `PlayerSnapshot.context` JSON (`meta[7..]`: the
//! dimension, the game mode and `TrackInventoryCodec`'s 41 SNBT slots), and each `KitOp` as the
//! one client click or hold that performs it: hotbar slot, clicked block and face, and the look.
use law::{BlockPos, PlayerState, Vec3, World, basis::cell::StateId};
use plan::{Kit, KitOp, KitSpec, kit::{self, BodyMovementKey, ToolKind, ToolProfileKey}};

pub const OP_INTERACT: u64 = 1;
pub const OP_PLACE: u64 = 2;
pub const OP_BREAK_HOLD: u64 = 3;
pub const OP_BUCKET_EMPTY: u64 = 4;
pub const OP_BUCKET_PICKUP: u64 = 5;
/// Words per action record: tick, slot, opcode, x, y, z, face, yaw bits, pitch bits, hold ticks.
pub const ACTION_WORDS: usize = 10;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Item { pub id: String, pub count: u32, pub enchantments: Vec<(String, u32)> }
impl Item {
    fn level(&self, name: &str) -> u8 {
        self.enchantments.iter().find(|(n, _)| n == name).map_or(0, |e| e.1.min(255) as u8)
    }
}

/// The kit and the nine hotbar stacks it is bound to.
#[derive(Clone, Debug)]
pub struct Carried { pub kit: Kit, pub hotbar: [Option<Item>; 9], pub selected: u8 }

/// One JSON string literal at the start of `s`: its value and the text after it.
fn literal(s: &str) -> Option<(String, &str)> {
    let quote = s.chars().next().filter(|c| *c == '"' || *c == '\'')?;
    let mut out = String::new();
    let mut chars = s[1..].char_indices();
    while let Some((i, c)) = chars.next() {
        match c {
            c if c == quote => return Some((out, &s[1 + i + 1..])),
            '\\' => match chars.next()?.1 {
                'n' => out.push('\n'),
                't' => out.push('\t'),
                'r' => out.push('\r'),
                'u' => {
                    let hex: String = (0..4).filter_map(|_| chars.next().map(|c| c.1)).collect();
                    out.push(u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32).unwrap_or('\u{fffd}'));
                }
                c => out.push(c),
            },
            c => out.push(c),
        }
    }
    None
}
fn after<'a>(json: &'a str, key: &str) -> Option<&'a str> {
    let k = format!("\"{key}\":");
    json.find(&k).map(|i| json[i + k.len()..].trim_start())
}
fn number(s: &str) -> Option<u32> {
    s.trim_start().chars().take_while(char::is_ascii_digit).collect::<String>().parse().ok()
}

/// The top-level `key: raw value` pairs of one SNBT compound.
fn compound(s: &str) -> Vec<(String, &str)> {
    let Some(body) = s.trim().strip_prefix('{').and_then(|b| b.strip_suffix('}')) else { return Vec::new() };
    let mut pieces = Vec::new();
    let (mut depth, mut quote, mut escaped, mut start) = (0i32, None, false, 0);
    for (i, c) in body.char_indices() {
        match quote {
            Some(q) => { if escaped { escaped = false } else if c == '\\' { escaped = true } else if c == q { quote = None } }
            None => match c {
                '"' | '\'' => quote = Some(c),
                '{' | '[' => depth += 1,
                '}' | ']' => depth -= 1,
                ',' if depth == 0 => { pieces.push(&body[start..i]); start = i + 1; }
                _ => {}
            },
        }
    }
    pieces.push(&body[start..]);
    pieces.into_iter().filter_map(|p| {
        let p = p.trim();
        if p.starts_with('"') || p.starts_with('\'') {
            let (key, rest) = literal(p)?;
            Some((key, rest.trim_start().strip_prefix(':')?.trim()))
        } else {
            let (key, value) = p.split_once(':')?;
            Some((key.trim().to_owned(), value.trim()))
        }
    }).collect()
}
fn field<'a>(pairs: &[(String, &'a str)], key: &str) -> Option<&'a str> {
    pairs.iter().find(|(k, _)| k == key).map(|p| p.1)
}

/// One `ItemStack.OPTIONAL_CODEC` SNBT slot; the empty stack is `{}`.
pub fn item(snbt: &str) -> Option<Item> {
    let top = compound(snbt);
    let id = literal(field(&top, "id")?)?.0;
    let count = field(&top, "count").and_then(number).unwrap_or(1);
    let mut enchantments = Vec::new();
    if let Some(e) = field(&top, "components").map(compound).and_then(|c| field(&c, "minecraft:enchantments").map(compound)) {
        // Older codecs nest the levels under `levels`.
        let e = field(&e, "levels").map(compound).unwrap_or(e);
        enchantments = e.into_iter().filter_map(|(k, v)| Some((k, number(v)?))).collect();
    }
    Some(Item { id, count, enchantments })
}

fn tier(id: &str, tool: &str) -> u8 {
    let Some(material) = id.strip_prefix("minecraft:").and_then(|n| n.strip_suffix(tool)) else { return 0 };
    // Gold mines at its own speed with wood's level; copper sits under iron: both are given the
    // lower tier, so a planned break never ends before the real one.
    match material { "wooden_" | "golden_" => 1, "stone_" | "copper_" => 2, "iron_" => 3, "diamond_" => 4, "netherite_" => 5, _ => 0 }
}
fn tool_kind(id: &str) -> Option<(ToolKind, u8)> {
    [(ToolKind::Pickaxe, "pickaxe"), (ToolKind::Axe, "axe"), (ToolKind::Shovel, "shovel")]
        .into_iter().map(|(k, t)| (k, tier(id, t))).find(|p| p.1 > 0)
}
/// A block item the planner may place: a full sturdy cube that is not a fluid, not a door or
/// switch, and does not fall.
fn placeable(id: &str) -> Option<StateId> {
    let name = id.strip_prefix("minecraft:")?;
    if name.contains("sand") && !name.contains("sandstone") || name.contains("gravel") || name.contains("concrete_powder") || name.starts_with("suspicious_") {
        return None;
    }
    let sid = StateId::parse(id, |_| None).ok()?;
    let st = kit::state_static(sid);
    (st.sturdy_faces() == 0x3f && !st.has_fluid() && st.interact_kind() == kit::InteractKind::None).then_some(sid)
}

/// The meta body: `{"dimension":..,"mode":..,"kit":{"selected":n,"slots":[41 SNBT]}}`. Anything
/// unreadable is bare legs, so an old or minimal body plans exactly as before.
pub fn read(body: &[u64]) -> Carried {
    let bare = Carried { kit: Kit::BareLegs, hotbar: Default::default(), selected: 0 };
    let Ok(text) = String::from_utf8(body.iter().map(|&b| b as u8).collect()) else { return bare };
    let (Some(mode), Some(slots)) = (after(&text, "mode").and_then(literal), after(&text, "slots")) else { return bare };
    let mut items = Vec::with_capacity(41);
    let mut rest = slots.strip_prefix('[').unwrap_or("");
    while let Some((snbt, tail)) = literal(rest.trim_start()) {
        items.push(item(&snbt));
        rest = tail.trim_start().strip_prefix(',').unwrap_or(tail);
    }
    let selected = after(&text, "selected").and_then(number).unwrap_or(0);
    if items.len() != 41 || selected > 8 || mode.0 == "spectator" { return bare; }
    let hotbar: [Option<Item>; 9] = std::array::from_fn(|k| items[31 + k].clone());
    let nether = after(&text, "dimension").and_then(literal).is_some_and(|d| d.0.ends_with(":the_nether"));
    let mut spec = KitSpec { interact: true, dimension_flags: if nether { kit::DIM_WATER_EVAPORATES | kit::DIM_FAST_LAVA } else { 0 }, ..KitSpec::empty() };
    // Adventure cannot edit blocks; creative breaks instantly and never runs out, which the
    // law does not model: both keep only the hand verb.
    if mode.0 == "survival" {
        let boots = items[3].as_ref();
        spec.body = BodyMovementKey {
            boots_depth_strider: boots.map_or(0, |b| b.level("minecraft:depth_strider")),
            soul_speed: boots.map_or(0, |b| b.level("minecraft:soul_speed")),
            leather_boots: boots.is_some_and(|b| b.id == "minecraft:leather_boots"),
        };
        let tools: Vec<_> = hotbar.iter().flatten().filter_map(|i| tool_kind(&i.id).map(|t| (t, i))).collect();
        let best = |k: ToolKind| tools.iter().filter(|t| t.0.0 == k).map(|t| t.0.1).max().unwrap_or(0);
        // One efficiency and one silk flag stand for every tool: the smallest, so no planned
        // break ends before the real one and no restore counts on a drop that does not come.
        spec.tool = ToolProfileKey {
            pickaxe_tier: best(ToolKind::Pickaxe),
            axe_tier: best(ToolKind::Axe),
            shovel_tier: best(ToolKind::Shovel),
            efficiency: tools.iter().map(|t| t.1.level("minecraft:efficiency")).min().unwrap_or(0),
            silk_touch: !tools.is_empty() && tools.iter().all(|t| t.1.level("minecraft:silk_touch") > 0),
            haste: 0,
        };
        spec.can_break = true;
        if let Some((sid, n)) = hotbar.iter().flatten().filter_map(|i| Some((placeable(&i.id)?, i.count))).max_by_key(|p| p.1) {
            spec.place_block = sid;
            spec.blocks = n.min(u32::from(u16::MAX)) as u16;
        }
        let count = |id: &str| hotbar.iter().flatten().filter(|i| i.id == id).map(|i| i.count).sum::<u32>().min(255) as u8;
        spec.water_buckets = count("minecraft:water_bucket");
        spec.powder_snow_buckets = count("minecraft:powder_snow_bucket");
        spec.pearls = count("minecraft:ender_pearl");
        spec.boats = hotbar.iter().flatten().filter(|i| i.id.ends_with("_boat") || i.id.ends_with("_raft")).count().min(255) as u8;
        spec.elytra = items[1].as_ref().is_some_and(|i| i.id == "minecraft:elytra");
    }
    Carried { kit: Kit::Carried(spec), hotbar, selected: selected as u8 }
}

use kit::FACE_NORMALS as NORMALS;
fn face_point(clicked: BlockPos, face: usize) -> Vec3 {
    let n = NORMALS[face];
    Vec3::new(f64::from(clicked[0]) + 0.5 + 0.5 * f64::from(n[0]), f64::from(clicked[1]) + 0.5 + 0.5 * f64::from(n[1]), f64::from(clicked[2]) + 0.5 + 0.5 * f64::from(n[2]))
}
/// The face of `cell` turned most toward the eye.
fn facing(eye: Vec3, cell: BlockPos) -> usize {
    let d = [eye.x - f64::from(cell[0]) - 0.5, eye.y - f64::from(cell[1]) - 0.5, eye.z - f64::from(cell[2]) - 0.5];
    let a = (0..3).max_by(|&i, &j| d[i].abs().total_cmp(&d[j].abs())).unwrap_or(1);
    [[4, 5], [0, 1], [2, 3]][a][usize::from(d[a] > 0.0)]
}
/// `Entity.lookAt`: the yaw and pitch that look from `eye` at `point`.
pub fn look(eye: Vec3, point: Vec3) -> (f32, f32) {
    let (dx, dy, dz) = (point.x - eye.x, point.y - eye.y, point.z - eye.z);
    let yaw = (dz.atan2(dx).to_degrees() as f32) - 90.0;
    let pitch = -(dy.atan2((dx * dx + dz * dz).sqrt()).to_degrees() as f32);
    (yaw, pitch)
}

impl Carried {
    fn slot_of(&self, want: impl Fn(&Item) -> bool) -> Option<u8> {
        self.hotbar.iter().position(|i| i.as_ref().is_some_and(&want)).map(|k| k as u8)
    }
    /// An empty hand: a door opens to an empty hand even while sneaking.
    fn hand(&self) -> u8 {
        self.hotbar.iter().position(Option::is_none).map_or(self.selected, |k| k as u8)
    }
    fn tool(&self, before: StateId) -> u8 {
        let kind = kit::tool_for(before);
        let best = (0..9).filter_map(|k| Some((k as u8, tool_kind(&self.hotbar[k].as_ref()?.id)?)))
            .filter(|(_, t)| t.0 == kind).max_by_key(|(_, t)| t.1);
        best.map_or_else(|| self.hand(), |b| b.0)
    }

    /// One action record: `[tick, slot, opcode, x, y, z, face, yaw, pitch, hold]`, `(x, y, z)` the
    /// edited cell. Place and bucket-empty click the neighbour `cell - normal(face)`; the others
    /// click the cell itself. `emptied` is the slot the last bucket was emptied from.
    pub fn action(&self, world: &impl World, s: &PlayerState, op: KitOp, tick: u64, ticks: u32, emptied: &mut Option<u8>) -> Result<[u64; ACTION_WORDS], String> {
        let eye = law::interact::get_eye_position(s.position, s.pose.eye_height());
        let pos = op.pos();
        let before = world.state_id(pos).ok_or("edited cell has no state")?;
        let name = |sid: StateId| format!("minecraft:{}", sid.block().name());
        let (slot, opcode, face) = match op {
            KitOp::Interact { .. } => (self.hand(), OP_INTERACT, facing(eye, pos)),
            KitOp::Break { .. } => (self.tool(before), OP_BREAK_HOLD, facing(eye, pos)),
            KitOp::BucketPickup { .. } => {
                let slot = emptied.take().or_else(|| self.slot_of(|i| i.id == "minecraft:bucket")).ok_or("no empty bucket in the hotbar")?;
                (slot, OP_BUCKET_PICKUP, 1)
            }
            KitOp::BucketEmpty { after, .. } => {
                let id = if after == kit::water_source_state() { "minecraft:water_bucket" } else { "minecraft:powder_snow_bucket" };
                let slot = self.slot_of(|i| i.id == id).ok_or_else(|| format!("no {id} in the hotbar"))?;
                *emptied = Some(slot);
                (slot, OP_BUCKET_EMPTY, 1)
            }
            KitOp::Place { after, .. } => {
                let id = name(after);
                // Item drops are not modelled: a restore places from what the hotbar holds.
                let slot = self.slot_of(|i| i.id == id).ok_or_else(|| format!("placing {id} needs it in the hotbar (item drops are not modelled)"))?;
                let support = kit::place_faces(world, pos).min_by(|&a, &b| {
                    let key = |f: usize| {
                        let n = NORMALS[f];
                        let p = face_point([pos[0] - n[0], pos[1] - n[1], pos[2] - n[2]], f);
                        let to_eye = [eye.x - p.x, eye.y - p.y, eye.z - p.z];
                        let hidden = (0..3).map(|i| f64::from(n[i]) * to_eye[i]).sum::<f64>() <= 0.0;
                        (hidden, to_eye.iter().map(|v| v * v).sum::<f64>())
                    };
                    key(a).partial_cmp(&key(b)).unwrap_or(std::cmp::Ordering::Equal)
                }).ok_or("no face to place against")?;
                (slot, OP_PLACE, support)
            }
        };
        let n = NORMALS[face];
        let clicked = if matches!(opcode, OP_PLACE | OP_BUCKET_EMPTY) {
            [pos[0] - n[0], pos[1] - n[1], pos[2] - n[2]]
        } else {
            pos
        };
        let (yaw, pitch) = look(eye, face_point(clicked, face));
        let hold = if op.holds_still() { u64::from(ticks) } else { 1 };
        Ok([tick, u64::from(slot), opcode, pos[0] as i64 as u64, pos[1] as i64 as u64, pos[2] as i64 as u64, face as u64,
            u64::from(yaw.to_bits()), u64::from(pitch.to_bits()), hold])
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    /// `PlayerSnapshot.context`'s body as Gson writes it: 41 SNBT slots, hotbar at 31..=39.
    pub(crate) fn body(dimension: &str, mode: &str, hotbar: [&str; 9], feet: &str, selected: u8) -> Vec<u64> {
        let mut slots = vec!["{}".to_owned(); 41];
        slots[3] = feet.to_owned();
        for (k, s) in hotbar.iter().enumerate() { slots[31 + k] = (*s).to_owned(); }
        let slots: Vec<String> = slots.iter().map(|s| format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))).collect();
        let json = format!(r#"{{"dimension":"{dimension}","player":"6b1c","mode":"{mode}","pose":"STANDING","kit":{{"version":1,"selected":{selected},"slots":[{}]}}}}"#, slots.join(","));
        json.bytes().map(u64::from).collect()
    }
    pub(crate) const PICK: &str = r#"{components:{"minecraft:enchantments":{"minecraft:efficiency":5,"minecraft:silk_touch":1}},count:1,id:"minecraft:diamond_pickaxe"}"#;
    pub(crate) const BOOTS: &str = r#"{components:{"minecraft:enchantments":{"minecraft:depth_strider":3}},count:1,id:"minecraft:leather_boots"}"#;

    #[test]
    fn live_meta_reads_the_carried_kit() {
        let hotbar = [PICK, "{count:64,id:\"minecraft:cobblestone\"}", "{count:1,id:\"minecraft:water_bucket\"}", "{}", "{count:12,id:\"minecraft:sand\"}",
            "{count:16,id:\"minecraft:ender_pearl\"}", "{count:1,id:\"minecraft:powder_snow_bucket\"}", "{count:1,id:\"minecraft:stick\"}", "{count:1,id:\"minecraft:oak_boat\"}"];
        let c = read(&body("minecraft:overworld", "survival", hotbar, BOOTS, 7));
        let Kit::Carried(k) = c.kit else { panic!("survival must carry a kit") };
        assert_eq!(k.tool, ToolProfileKey { pickaxe_tier: 4, efficiency: 5, silk_touch: true, ..Default::default() });
        assert_eq!(k.body, BodyMovementKey { boots_depth_strider: 3, soul_speed: 0, leather_boots: true });
        assert_eq!((k.blocks, k.place_block), (64, StateId::parse("minecraft:cobblestone", |_| None).unwrap()), "falling sand is never the bridge block");
        assert_eq!((k.water_buckets, k.powder_snow_buckets, k.pearls, k.boats), (1, 1, 16, 1));
        assert!(k.interact && k.can_break && k.restore_required && k.dimension_flags == 0 && !k.elytra);
        assert_eq!((c.selected, c.hand()), (7, 3), "the first empty hotbar slot is the hand");
        let stone = StateId::parse("minecraft:stone", |_| None).unwrap();
        let oak = StateId::parse("minecraft:oak_planks", |_| None).unwrap();
        assert_eq!((c.tool(stone), c.tool(oak)), (0, 3), "stone takes the pickaxe; planks without an axe take the hand");

        let Kit::Carried(n) = read(&body("minecraft:the_nether", "survival", hotbar, "{}", 0)).kit else { panic!() };
        assert_eq!(n.dimension_flags & kit::DIM_WATER_EVAPORATES, kit::DIM_WATER_EVAPORATES);
        assert!(!plan::KitView::for_spec(n).verb_effects.iter().any(|e| e.kind == kit::VerbKind::BucketWater), "nether water evaporates");
        let Kit::Carried(a) = read(&body("minecraft:overworld", "adventure", hotbar, BOOTS, 0)).kit else { panic!() };
        assert!(a.interact && !a.can_break && a.blocks == 0 && a.water_buckets == 0, "adventure keeps only the hand");
        assert_eq!(read(&body("minecraft:overworld", "spectator", hotbar, BOOTS, 0)).kit, Kit::BareLegs);
        assert_eq!(read(&[123, 125]).kit, Kit::BareLegs, "a minimal body plans as bare legs");
    }

    #[test]
    fn snbt_items_keep_counts_ids_and_levels() {
        assert_eq!(item("{}"), None);
        let i = item(r#"{count:3,components:{"minecraft:custom_name":'{"text":"id:\"x\",count:9"}',"minecraft:enchantments":{levels:{"minecraft:efficiency":4}}},id:"minecraft:iron_shovel"}"#).unwrap();
        assert_eq!((i.id.as_str(), i.count, i.level("minecraft:efficiency")), ("minecraft:iron_shovel", 3, 4));
        assert_eq!(tool_kind(&i.id), Some((ToolKind::Shovel, 3)));
        let (yaw, pitch) = look(Vec3::new(0.5, 1.62, 0.5), Vec3::new(0.5, 0.0, 1.5));
        assert!((yaw - 0.0).abs() < 1e-4 && pitch > 0.0, "south and down is yaw 0, pitch positive: {yaw} {pitch}");
    }
}
