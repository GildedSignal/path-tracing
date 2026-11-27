// Development-time extraction only. Cargo and the reader need no JVM or game jar.
import java.nio.file.*;
import java.io.*;
import java.nio.charset.StandardCharsets;
import com.google.gson.JsonParser;
import net.minecraft.tags.TagKey;
import net.minecraft.resources.Identifier;
import java.util.*;
import java.util.stream.Collectors;
import java.util.stream.DoubleStream;
import net.minecraft.SharedConstants;
import net.minecraft.server.Bootstrap;
import net.minecraft.core.*;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.commands.arguments.blocks.BlockStateParser;
import net.minecraft.tags.BlockTags;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.level.*;
import net.minecraft.world.level.block.*;
import net.minecraft.world.level.block.entity.BlockEntity;
import net.minecraft.world.level.block.state.*;
import net.minecraft.world.level.block.state.properties.*;
import net.minecraft.world.level.material.*;
import net.minecraft.world.phys.shapes.*;

public final class ModelTable {
    /** A state the law does not model, and why. */
    static final class Refused extends RuntimeException {
        Refused(String reason) { super(reason, null, false, false); }
    }
    // The player-movement hooks the tick calls on a block (Entity.move: stepOn; Entity.checkFallDamage:
    // fallOn; Entity.checkInsideBlocks: entityInside), keyed by the class that implements each, with
    // what the law makes of it. Friction, speed, jump and bounce are getters read below; climbing is
    // the CLIMBABLE tag. "world": the implementation only changes the world, never the player, this
    // tick (26.2 bytecode). An implementation missing here refuses every state of its blocks.
    static final Map<String,Map<Class<?>,String>> HOOKS = Map.of(
        "entityInside", Map.ofEntries(
            Map.entry(BlockBehaviour.class, "none"),
            Map.entry(WebBlock.class, "cobweb"),
            Map.entry(PowderSnowBlock.class, "powderSnow"),
            Map.entry(HoneyBlock.class, "honey"),
            Map.entry(BubbleColumnBlock.class, "bubbleColumn"),
            Map.entry(CactusBlock.class, "cactus"),
            Map.entry(CampfireBlock.class, "campfire"),
            Map.entry(BaseFireBlock.class, "fire"),
            Map.entry(SweetBerryBushBlock.class, "berryBush"),
            Map.entry(LayeredCauldronBlock.class, "extinguish"),
            Map.entry(LavaCauldronBlock.class, "lava"),
            Map.entry(EndPortalBlock.class, "endPortal"),
            Map.entry(EndGatewayBlock.class, "endPortal"),
            Map.entry(NetherPortalBlock.class, "netherPortal"),
            Map.entry(WitherRoseBlock.class, "witherRose"),
            Map.entry(BigDripleafBlock.class, "world"),
            Map.entry(BasePressurePlateBlock.class, "world"),
            Map.entry(TripWireBlock.class, "world"),
            Map.entry(ButtonBlock.class, "world"),
            Map.entry(DetectorRailBlock.class, "world"),
            Map.entry(CropBlock.class, "world"),
            Map.entry(PitcherCropBlock.class, "world"),
            Map.entry(EyeblossomBlock.class, "world"),
            Map.entry(LilyPadBlock.class, "world"),
            Map.entry(FrogspawnBlock.class, "world"),
            Map.entry(HopperBlock.class, "world")),
        "stepOn", Map.of(
            Block.class, "none",
            SlimeBlock.class, "none",
            MagmaBlock.class, "hotFloor",
            SculkSensorBlock.class, "world",
            SculkShriekerBlock.class, "world",
            RedStoneOreBlock.class, "world",
            TurtleEggBlock.class, "world"),
        "fallOn", Map.of(
            Block.class, "none",
            HayBlock.class, "none",
            HoneyBlock.class, "none",
            BedBlock.class, "none",
            SlimeBlock.class, "none",
            PowderSnowBlock.class, "none",
            FarmlandBlock.class, "world",
            TurtleEggBlock.class, "world",
            PointedDripstoneBlock.class, "dripstone"));
    static final Map<String,Class<?>> HOOK_BASE = Map.of("entityInside",BlockBehaviour.class, "stepOn",Block.class, "fallOn",Block.class);
    static final BlockGetter LOCAL = new BlockGetter() {
        public BlockState getBlockState(BlockPos p) { throw new Refused("non-local shape"); }
        public FluidState getFluidState(BlockPos p) { throw new Refused("non-local shape"); }
        public BlockEntity getBlockEntity(BlockPos p) { throw new Refused("non-local shape"); }
        public int getHeight() { return 384; }
        public int getMinY() { return -64; }
    };
    static final class Context extends EntityCollisionContext {
        Context(boolean descending, double feet) { super(descending, false, feet, ItemStack.EMPTY, false, null); }
    }
    // Empty, above, below, descending above, and feet at the cell's bottom plane.
    static final CollisionContext[] CONTEXTS = {CollisionContext.empty(), new Context(false,1), new Context(false,-2), new Context(true,1), new Context(false,0)};
    // Bootstrap does not bind datapack tags. Resolve the vanilla jar resources
    // explicitly instead of treating its initially empty tag sets as answers.
    static final Map<String,Set<String>> TAGS = new HashMap<>();
    static final Set<String> RESOLVING = new HashSet<>();
    static Set<String> loadTag(String name) {
        if(TAGS.containsKey(name)) return TAGS.get(name);
        if(!RESOLVING.add(name)) throw new AssertionError("cyclic tag "+name);
        var id=Identifier.parse(name);
        var values=new TreeSet<String>();
        try(var input=ModelTable.class.getResourceAsStream("/data/"+id.getNamespace()+"/tags/block/"+id.getPath()+".json")) {
            if(input==null) throw new AssertionError("missing tag "+name);
            var json=JsonParser.parseReader(new InputStreamReader(input,StandardCharsets.UTF_8)).getAsJsonObject();
            for(var entry:json.getAsJsonArray("values")) {
                String value=entry.isJsonPrimitive()?entry.getAsString():entry.getAsJsonObject().get("id").getAsString();
                if(value.startsWith("#")) values.addAll(loadTag(value.substring(1)));
                else values.add(Identifier.parse(value).toString());
            }
        } catch(IOException e) { throw new UncheckedIOException(e); }
        RESOLVING.remove(name); TAGS.put(name,values); return values;
    }
    static boolean tag(BlockState state, TagKey<Block> key) {
        return loadTag(key.location().toString()).contains(BuiltInRegistries.BLOCK.getKey(state.getBlock()).toString());
    }
    static Class<?> owner(Block b, String method, Class<?> base) {
        for (Class<?> c=b.getClass(); c != base; c=c.getSuperclass()) {
            for (var m:c.getDeclaredMethods()) if (m.getName().equals(method)) return c;
        }
        return base;
    }
    static String hook(Block b, String name) {
        Class<?> c=owner(b,name,HOOK_BASE.get(name));
        String effect=HOOKS.get(name).get(c);
        if (effect==null) throw new Refused("hook "+name+" "+c.getSimpleName());
        return effect;
    }
    static String shape(VoxelShape s) {
        return s.toAabbs().stream().map(b -> b.minX+","+b.minY+","+b.minZ+","+b.maxX+","+b.maxY+","+b.maxZ).collect(Collectors.joining(";"));
    }
    static String fp(float f) { return Integer.toHexString(Float.floatToRawIntBits(f)); }
    @SuppressWarnings({"rawtypes", "unchecked"})
    static String property(BlockState s, Property p) { return p.getName(s.getValue(p)); }
    @SuppressWarnings({"rawtypes", "unchecked"})
    static BlockState with(BlockState s, Property p, Comparable v) { return s.setValue(p,v); }
    static String state(BlockState s) {
        String name=BuiltInRegistries.BLOCK.getKey(s.getBlock()).toString();
        var props=s.getProperties().stream().sorted(Comparator.comparing(Property::getName))
            .map(p -> p.getName()+"="+property(s,p)).collect(Collectors.joining(","));
        return name+(props.isEmpty()?"":"["+props+"]");
    }
    static String resolveEntityInside(BlockState s, Block b, String base) {
        return switch (base) {
            case "bubbleColumn" -> s.getValue(BubbleColumnBlock.DRAG_DOWN) ? "bubbleColumnDown" : "bubbleColumnUp";
            case "campfire" -> !s.getValue(CampfireBlock.LIT) ? "none" : (b == Blocks.SOUL_CAMPFIRE ? "soulCampfire" : "campfire");
            case "fire" -> {
                if (b instanceof SoulFireBlock) yield "soulFire";
                if (b instanceof FireBlock) yield "fire";
                throw new Refused("hook entityInside " + b.getClass().getSimpleName());
            }
            case "berryBush" -> s.getValue(SweetBerryBushBlock.AGE) > 0 ? "berryBushHurt" : "berryBush";
            default -> base;
        };
    }
    static String model(BlockState s) {
        Block b=s.getBlock();
        String inside=resolveEntityInside(s, b, hook(b,"entityInside")), step=hook(b,"stepOn"), fall=hook(b,"fallOn");
        boolean world=inside.equals("world")||step.equals("world")||fall.equals("world");
        var fluid=s.getFluidState();
        boolean isLava = fluid.getType().isSame(Fluids.LAVA);
        boolean isWater = fluid.getType().isSame(Fluids.WATER);
        if (!fluid.isEmpty() && !isWater && !isLava) throw new Refused("fluid other than water or lava");
        var collision=Arrays.stream(CONTEXTS).map(c -> s.getCollisionShape(LOCAL,BlockPos.ZERO,c)).toList();
        String offset="none";
        if (s.hasOffsetFunction() && collision.stream().anyMatch(c -> !c.isEmpty())) {
            if (b instanceof BambooStalkBlock) {
                offset = "bamboo";
            } else if (b instanceof SpeleothemBlock) {
                offset = "speleothem:" + s.getValue(SpeleothemBlock.THICKNESS).getSerializedName()
                    + ":" + s.getValue(SpeleothemBlock.TIP_DIRECTION).getSerializedName();
            } else {
                throw new Refused("offset collision");
            }
        }
        // Entity.getOnPos tests the FENCES and WALLS tags and FenceGateBlock.
        boolean[] flags={s.isAir(),s.isSuffocating(LOCAL,BlockPos.ZERO),b instanceof LiquidBlock && isWater,
            tag(s,BlockTags.CLIMBABLE),b instanceof ScaffoldingBlock,b instanceof SlimeBlock,JarRules.halvesFallDistance(b),
            tag(s,BlockTags.FENCES),tag(s,BlockTags.WALLS)||b instanceof FenceGateBlock,tag(s,BlockTags.SUPPRESSES_BOUNCE),
            tag(s,BlockTags.FALL_DAMAGE_RESETTING),b instanceof EndPortalBlock || b instanceof EndGatewayBlock,world};
        int bits=0; for(int i=0;i<flags.length;i++) if(flags[i]) bits|=1<<i;
        int faces=0; int i=0;
        for(var d:new Direction[]{Direction.NORTH,Direction.EAST,Direction.SOUTH,Direction.WEST}) {
            if (!(b instanceof IceBlock) && s.isFaceSturdy(LOCAL,BlockPos.ZERO,d)) faces|=1<<i; i++;
        }
        int ladder=b instanceof LadderBlock?s.getValue(LadderBlock.FACING).get3DDataValue():-1;
        int trap=b instanceof TrapDoorBlock && s.getValue(TrapDoorBlock.OPEN)?s.getValue(TrapDoorBlock.FACING).get3DDataValue():-1;
        float fallMult = (b instanceof HayBlock || b instanceof HoneyBlock) ? 0.2f
            : (fall.equals("dripstone") && s.getValue(PointedDripstoneBlock.TIP_DIRECTION) == Direction.UP
                && s.getValue(PointedDripstoneBlock.THICKNESS) == SpeleothemThickness.TIP ? 2.0f : 1f);
        String fluidKind = fluid.isEmpty() ? "0" : (isLava ? "2" : "1");
        return String.join("\t",fp(b.getFriction()),fp(b.getSpeedFactor()),fp(b.getJumpFactor()),fp(b.getBounceRestitution()),
            fp(fallMult),Integer.toString(bits),Integer.toString(ladder),Integer.toString(trap),
            fluidKind,fluid.isEmpty()?"0":Integer.toString(fluid.getAmount()),
            fluid.isEmpty()?"false":fluid.getValue(FlowingFluid.FALLING).toString(),Boolean.toString(JarRules.blocksFluidFlow(s)),Integer.toString(faces),
            inside.equals("world")?"none":inside, step.equals("hotFloor")?"hotFloor":"none",
            shape(collision.get(0)),
            // PowderSnowBlock returns Shapes.block when the unbooted collision shape is empty (26.2 bytecode).
            shape(b instanceof PowderSnowBlock?Shapes.block():s.getEntityInsideCollisionShape(LOCAL,BlockPos.ZERO,null)),
            shape(collision.get(1)),shape(collision.get(2)),shape(collision.get(3)),shape(collision.get(4)),offset);
    }
    // The jar's collision boxes for named states, in each law context, as exact double bits.
    static void shapes(Path out, String[] states) throws Exception {
        var lines=new ArrayList<String>();
        for (String raw:states) {
            BlockState s=BlockStateParser.parseForBlock(BuiltInRegistries.BLOCK,raw,false).blockState();
            for (int c=0;c<CONTEXTS.length;c++) {
                lines.add(raw+"\t"+c+"\t"+s.getCollisionShape(LOCAL,BlockPos.ZERO,CONTEXTS[c]).toAabbs().stream()
                    .map(b -> DoubleStream.of(b.minX,b.minY,b.minZ,b.maxX,b.maxY,b.maxZ).mapToObj(v -> Long.toHexString(Double.doubleToRawLongBits(v)))
                    .collect(Collectors.joining(","))).collect(Collectors.joining(";")));
            }
        }
        Files.write(out,lines);
    }
    // states.tsv: every block in registry order (its row is its id), its default state's index, then
    // each property with its values, in the StateDefinition's order. A state's id is the block's first
    // id plus the mixed-radix index of its values, the first property the most significant.
    static void states(Path out) throws Exception {
        var lines=new ArrayList<String>(); int base=0;
        for(Block b:BuiltInRegistries.BLOCK) {
            if (BuiltInRegistries.BLOCK.getId(b)!=lines.size()) throw new AssertionError("registry order");
            var props=new ArrayList<>(b.getStateDefinition().getProperties());
            var states=b.getStateDefinition().getPossibleStates();
            for (BlockState s:states) {
                int local=0;
                for (Property p:props) local=local*p.getPossibleValues().size()+p.getPossibleValues().indexOf(s.getValue(p));
                if (Block.BLOCK_STATE_REGISTRY.getId(s)!=base+local) throw new AssertionError(state(s)+" has id "+Block.BLOCK_STATE_REGISTRY.getId(s)+", not "+(base+local));
            }
            int dflt=Block.BLOCK_STATE_REGISTRY.getId(b.defaultBlockState())-base;
            var line=new StringBuilder(BuiltInRegistries.BLOCK.getKey(b)+"\t"+dflt);
            for (Property p:props) {
                var values=new ArrayList<String>();
                for (Object v:p.getPossibleValues()) values.add(p.getName((Comparable)v));
                line.append("\t"+p.getName()+":"+String.join(",",values));
            }
            lines.add(line.toString());
            base+=states.size();
        }
        if (base!=Block.BLOCK_STATE_REGISTRY.size()) throw new AssertionError(base+" states");
        Files.write(out,lines);
    }

    // A rule row contains only facts visible without a world. A dash marks a getter
    // that reads a neighbour or block entity. The index is in exact StateId order.
    static List<String> cellRules(BlockState state) {
        Block block=state.getBlock();
        String conductor="-", sturdy="-", waterContainer="no";
        try { conductor=Boolean.toString(state.isRedstoneConductor(LOCAL,BlockPos.ZERO)); }
        catch (Refused ignored) {}
        try {
            int mask=0;
            for (Direction dir:Direction.values()) {
                if (state.isFaceSturdy(LOCAL,BlockPos.ZERO,dir)) mask|=1<<dir.get3DDataValue();
            }
            sturdy=Integer.toString(mask);
        } catch (Refused ignored) {}
        if (block instanceof LiquidBlockContainer container) {
            boolean accepts=container.canPlaceLiquid(null,LOCAL,BlockPos.ZERO,state,Fluids.WATER);
            if (container.canPlaceLiquid(null,LOCAL,BlockPos.ZERO,state,Fluids.LAVA))
                throw new AssertionError("unmodeled lava container: "+state);
            if (block instanceof SimpleWaterloggedBlock) {
                waterContainer=accepts ? "accepts" : "rejects";
            } else {
                if (accepts) throw new AssertionError("unmodeled liquid container: "+state);
                waterContainer="other";
            }
        }
        return List.of(
            Integer.toHexString(Float.floatToRawIntBits(state.getDestroySpeed(LOCAL,BlockPos.ZERO))),
            Boolean.toString(state.requiresCorrectToolForDrops()), conductor, sturdy,
            state.getPistonPushReaction().name(), Boolean.toString(JarRules.blocksMotion(state)),
            Boolean.toString(state.canBeReplaced(Fluids.WATER)), Boolean.toString(state.canBeReplaced(Fluids.LAVA)),
            Boolean.toString(state.canBeReplaced()), Boolean.toString(state.isSignalSource()),
            Boolean.toString(state.hasBlockEntity()), Boolean.toString(state.isAir()), waterContainer,
            JarRules.hasFluidAdmissionTag() ? JarRules.washedAway(state)+":"+state.isSolid() : "");
    }

    // Keep the columns read by the kit projection in place. The same rows serve
    // its block defaults and the simulator's full state index.
    static String cellRuleLine(String name, List<String> row) {
        float hardness=Float.intBitsToFloat(Integer.parseUnsignedInt(row.get(0),16));
        String line=String.join("\t", name, Float.toString(hardness), row.get(1).equals("true")?"y":"n",
            row.get(2), row.get(3), row.get(4), row.get(5), row.get(6), row.get(7), row.get(8),
            row.get(9), row.get(10).equals("true")?"y":"n", row.get(11)+":"+row.get(12));
        return row.get(13).isEmpty() ? line : line+"\t"+row.get(13);
    }

    static void writeCellRules(Path out, Path rules) throws IOException {
        var ids=new LinkedHashMap<List<String>,Integer>();
        var index=new ArrayList<String>();
        var rows=new ArrayList<String>();
        int blocks=0, states=0, previous=-1;
        for (Block block:BuiltInRegistries.BLOCK) {
            if (BuiltInRegistries.BLOCK.getId(block)!=blocks++) throw new AssertionError("registry order");
            var defaults=cellRules(block.defaultBlockState());
            ids.putIfAbsent(defaults,rows.size());
            rows.add(cellRuleLine(BuiltInRegistries.BLOCK.getKey(block).getPath(),defaults));
        }
        for (Block block:BuiltInRegistries.BLOCK) {
            for (BlockState state:block.getStateDefinition().getPossibleStates()) {
                if (Block.getId(state)!=states) throw new AssertionError("state order: "+state);
                int id=ids.computeIfAbsent(cellRules(state), row -> {
                    int next=rows.size();
                    rows.add(cellRuleLine("@"+next,row));
                    return next;
                });
                if (id!=previous && previous!=-1) index.add(states+"\t"+previous);
                previous=id;
                states++;
            }
        }
        if (blocks!=BuiltInRegistries.BLOCK.size() || states!=Block.BLOCK_STATE_REGISTRY.size())
            throw new AssertionError(blocks+" blocks, "+states+" states");
        index.add(states+"\t"+previous);
        index.addFirst("# state id (exclusive end)\trule row");
        rows.addFirst("# block default or variant\thardness\tcorrect tool\tconductor\tsturdy faces\tpush reaction\tblocks motion\treplaceable by water\treplaceable by lava\treplaceable\tsignal source\tblock entity\tair:water container"
            + (JarRules.hasFluidAdmissionTag() ? "\twashed away by fluids:solid" : ""));
        Files.write(rules,rows);
        Files.write(out.resolve("cell_rule_index.tsv"),index);
    }

    public static void main(String[] args) throws Exception {
        SharedConstants.tryDetectVersion(); Bootstrap.bootStrap();
        if (args[0].equals("--shapes")) { shapes(Path.of(args[1]),Arrays.copyOfRange(args,2,args.length)); return; }
        if (args[0].equals("--states")) { states(Path.of(args[1])); return; }
        if (args[0].equals("--cell-rules")) { writeCellRules(Path.of(args[1]),Path.of(args[2])); return; }
        var rows=new TreeMap<String,String>();
        // Each registry state lands in one bucket of its block: "modeled", or the reason it is refused.
        var census=new TreeMap<String,Integer>();
        int total=0;
        for(Block b:BuiltInRegistries.BLOCK) {
            String name=BuiltInRegistries.BLOCK.getKey(b).toString();
            for(BlockState s:b.getStateDefinition().getPossibleStates()) {
                String bucket="modeled";
                try { rows.put(state(s),model(s)); } catch (Refused r) { bucket=r.getMessage(); }
                census.merge(name+"\t"+bucket,1,Integer::sum); total++;
            }
        }
        if (total!=Block.BLOCK_STATE_REGISTRY.size()) throw new AssertionError(total+" states; registry has "+Block.BLOCK_STATE_REGISTRY.size());
        // Equal physical models share one record; state identity still stays exact.
        var models=new TreeSet<String>(rows.values());
        var ids=new HashMap<String,Integer>(); var modelLines=new ArrayList<String>();
        for(String m:models) { ids.put(m,ids.size()); modelLines.add(m); }
        // Store the generator: per block, the properties that change its model, and their values' model ids.
        // maps.tsv: relevant properties, then "values:slot" entries. blocks.tsv: block, map, model id per slot.
        var blockLines=new ArrayList<String>();
        var maps=new HashMap<String,Integer>();
        int checked=0;
        for(Block b:BuiltInRegistries.BLOCK) {
            var states=b.getStateDefinition().getPossibleStates();
            long modeled=states.stream().filter(s -> rows.containsKey(state(s))).count();
            if (modeled==0) continue;
            if (modeled!=states.size()) throw new AssertionError(BuiltInRegistries.BLOCK.getKey(b)+" is partly modeled");
            // A property counts when changing it alone, from some state, changes the model row.
            var relevant=b.getStateDefinition().getProperties().stream().sorted(Comparator.comparing(Property::getName))
                .filter(p -> states.stream().anyMatch(s -> p.getPossibleValues().stream()
                    .anyMatch(v -> !rows.get(state(s)).equals(rows.get(state(with(s,p,(Comparable)v)))))))
                .toList();
            var entries=new TreeMap<String,Integer>();
            for (BlockState s:states) entries.putIfAbsent(relevant.stream().map(p -> property(s,p)).collect(Collectors.joining(",")),ids.get(rows.get(state(s))));
            // Blocks share a map from values to local slots; each block lists its model id per slot.
            var slots=new ArrayList<Integer>(new LinkedHashSet<>(entries.values()));
            String map=relevant.stream().map(Property::getName).collect(Collectors.joining(","))+"\t"
                +entries.entrySet().stream().map(e -> e.getKey()+":"+slots.indexOf(e.getValue())).collect(Collectors.joining(" "));
            int index=maps.computeIfAbsent(map,m -> maps.size());
            // The stored lookup must reproduce every state's model id.
            var local=new HashMap<String,Integer>();
            for (String e:map.split("\t",-1)[1].split(" ")) local.put(e.substring(0,e.lastIndexOf(':')),Integer.parseInt(e.substring(e.lastIndexOf(':')+1)));
            for (BlockState s:states) {
                int id=slots.get(local.get(relevant.stream().map(p -> property(s,p)).collect(Collectors.joining(","))));
                if (id!=ids.get(rows.get(state(s)))) throw new AssertionError(state(s)+" looks up model "+id+", not "+ids.get(rows.get(state(s))));
                checked++;
            }
            blockLines.add(BuiltInRegistries.BLOCK.getKey(b)+"\t"+index+"\t"+slots.stream().map(String::valueOf).collect(Collectors.joining(",")));
        }
        if (checked!=rows.size()) throw new AssertionError(checked+" states checked of "+rows.size());
        var mapLines=new ArrayList<String>(Collections.nCopies(maps.size(),""));
        maps.forEach((m,i) -> mapLines.set(i,m));
        Files.write(Path.of(args[0],"maps.tsv"),mapLines);
        Files.write(Path.of(args[0],"models.tsv"),modelLines);
        var defaults=new TreeSet<String>();
        for(Block b:BuiltInRegistries.BLOCK) if(rows.containsKey(state(b.defaultBlockState()))) defaults.add(state(b.defaultBlockState()));
        Files.write(Path.of(args[0],"defaults.txt"),defaults);
        Files.writeString(Path.of(args[0],"version.txt"),Integer.toString(SharedConstants.getCurrentVersion().dataVersion().version())+"\n");
        Files.write(Path.of(args[0],"blocks.tsv"),blockLines);
        var censusLines=new ArrayList<String>(List.of("# "+Block.BLOCK_STATE_REGISTRY.size()+" registry states"));
        for(var entry:census.entrySet()) censusLines.add(entry.getKey()+"\t"+entry.getValue());
        Files.write(Path.of(args[0],"census.tsv"),censusLines);
        Path out=Path.of(args[0]).normalize();
        writeCellRules(out,out.resolve("rules.tsv"));
        System.out.println(rows.size()+" of "+total+" states modeled, "+models.size()+" physical models");
    }
}
