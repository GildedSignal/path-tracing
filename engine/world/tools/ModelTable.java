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
import net.minecraft.world.level.block.state.properties.Property;
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
        "entityInside", Map.of(BlockBehaviour.class,"none", WebBlock.class,"cobweb", PowderSnowBlock.class,"powderSnow",
            HoneyBlock.class,"honey", BasePressurePlateBlock.class,"world", TripWireBlock.class,"world", ButtonBlock.class,"world"),
        "stepOn", Map.of(Block.class,"none", SlimeBlock.class,"none", MagmaBlock.class,"hotFloor"),
        "fallOn", Map.of(Block.class,"none", HayBlock.class,"none", HoneyBlock.class,"none", BedBlock.class,"none",
            SlimeBlock.class,"none", PowderSnowBlock.class,"none"));
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
    // The collision contexts the law uses: none, feet above, feet below, descending above.
    static final CollisionContext[] CONTEXTS = {CollisionContext.empty(), new Context(false,1), new Context(false,-2), new Context(true,1)};
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
    static String model(BlockState s) {
        Block b=s.getBlock();
        String inside=hook(b,"entityInside"), step=hook(b,"stepOn"), fall=hook(b,"fallOn");
        boolean world=inside.equals("world")||step.equals("world")||fall.equals("world");
        var fluid=s.getFluidState();
        if (!fluid.isEmpty() && !fluid.getType().isSame(Fluids.WATER)) throw new Refused("fluid other than water");
        var collision=Arrays.stream(CONTEXTS).map(c -> s.getCollisionShape(LOCAL,BlockPos.ZERO,c)).toList();
        // An offset follows the position, not the state: it may only move shapes nothing collides with.
        if (s.hasOffsetFunction() && collision.stream().anyMatch(c -> !c.isEmpty())) throw new Refused("offset collision");
        // Entity.getOnPos tests the FENCES and WALLS tags and FenceGateBlock.
        boolean[] flags={s.isAir(),s.isSuffocating(LOCAL,BlockPos.ZERO),b instanceof LiquidBlock && fluid.getType().isSame(Fluids.WATER),
            tag(s,BlockTags.CLIMBABLE),b instanceof ScaffoldingBlock,b instanceof SlimeBlock,b instanceof BedBlock,
            tag(s,BlockTags.FENCES),tag(s,BlockTags.WALLS)||b instanceof FenceGateBlock,tag(s,BlockTags.SUPPRESSES_BOUNCE),
            tag(s,BlockTags.FALL_DAMAGE_RESETTING),false,world};
        int bits=0; for(int i=0;i<flags.length;i++) if(flags[i]) bits|=1<<i;
        int faces=0; int i=0;
        for(var d:new Direction[]{Direction.NORTH,Direction.EAST,Direction.SOUTH,Direction.WEST}) {
            if (!(b instanceof IceBlock) && s.isFaceSturdy(LOCAL,BlockPos.ZERO,d)) faces|=1<<i; i++;
        }
        int ladder=b instanceof LadderBlock?s.getValue(LadderBlock.FACING).get3DDataValue():-1;
        int trap=b instanceof TrapDoorBlock && s.getValue(TrapDoorBlock.OPEN)?s.getValue(TrapDoorBlock.FACING).get3DDataValue():-1;
        return String.join("\t",fp(b.getFriction()),fp(b.getSpeedFactor()),fp(b.getJumpFactor()),fp(b.getBounceRestitution()),
            fp(b instanceof HayBlock || b instanceof HoneyBlock?0.2f:1f),Integer.toString(bits),Integer.toString(ladder),Integer.toString(trap),
            fluid.isEmpty()?"0":"1",fluid.isEmpty()?"0":Integer.toString(fluid.getAmount()),
            fluid.isEmpty()?"false":fluid.getValue(FlowingFluid.FALLING).toString(),Boolean.toString(s.blocksMotion()),Integer.toString(faces),
            inside.equals("world")?"none":inside, step.equals("hotFloor")?"hotFloor":"none",
            shape(collision.get(0)),
            // PowderSnowBlock returns Shapes.block when the unbooted collision shape is empty (26.2 bytecode).
            shape(b instanceof PowderSnowBlock?Shapes.block():s.getEntityInsideCollisionShape(LOCAL,BlockPos.ZERO,null)),
            shape(collision.get(1)),shape(collision.get(2)),shape(collision.get(3)));
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
    public static void main(String[] args) throws Exception {
        SharedConstants.tryDetectVersion(); Bootstrap.bootStrap();
        if (args[0].equals("--shapes")) { shapes(Path.of(args[1]),Arrays.copyOfRange(args,2,args.length)); return; }
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
        System.out.println(rows.size()+" of "+total+" states modeled, "+models.size()+" physical models");
    }
}
