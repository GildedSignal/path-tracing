// Development-time extraction only. Cargo and the reader need no JVM or game jar.
import java.nio.file.*;
import java.io.*;
import java.nio.charset.StandardCharsets;
import com.google.gson.JsonParser;
import net.minecraft.tags.TagKey;
import net.minecraft.resources.Identifier;
import java.util.*;
import java.util.stream.Collectors;
import net.minecraft.SharedConstants;
import net.minecraft.server.Bootstrap;
import net.minecraft.core.*;
import net.minecraft.core.registries.BuiltInRegistries;
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
    // Only reviewed local shape families. In particular, no offsets, fences/walls,
    // doors, moving pistons, portals, lava hooks, or arbitrary entityInside hooks.
    static final Set<String> FAMILIES = Set.of("Block", "AirBlock", "RotatedPillarBlock",
        "GrassBlock", "SnowyDirtBlock", "SlabBlock", "StairBlock", "LadderBlock", "VineBlock",
        "TrapDoorBlock", "ScaffoldingBlock", "LiquidBlock", "IceBlock", "FrostedIceBlock",
        "SlimeBlock", "HoneyBlock", "SoulSandBlock", "WebBlock", "PowderSnowBlock",
        "HayBlock", "BedBlock", "SnowLayerBlock", "DirtPathBlock", "MagmaBlock");
    static final BlockGetter LOCAL = new BlockGetter() {
        public BlockState getBlockState(BlockPos p) { throw new AssertionError("neighbor-dependent block shape at " + p); }
        public FluidState getFluidState(BlockPos p) { throw new AssertionError("neighbor-dependent fluid at " + p); }
        public BlockEntity getBlockEntity(BlockPos p) { throw new AssertionError(p); }
        public int getHeight() { return 384; }
        public int getMinY() { return -64; }
    };
    static final class Context extends EntityCollisionContext {
        Context(boolean descending, double feet) { super(descending, false, feet, ItemStack.EMPTY, false, null); }
    }
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
    static String shape(VoxelShape s) {
        return s.toAabbs().stream().map(b -> b.minX+","+b.minY+","+b.minZ+","+b.maxX+","+b.maxY+","+b.maxZ).collect(Collectors.joining(";"));
    }
    static String fp(float f) { return Integer.toHexString(Float.floatToRawIntBits(f)); }
    @SuppressWarnings({"rawtypes", "unchecked"})
    static String property(BlockState s, Property p) { return p.getName(s.getValue(p)); }
    static String state(BlockState s) {
        String name=BuiltInRegistries.BLOCK.getKey(s.getBlock()).toString();
        var props=s.getProperties().stream().sorted(Comparator.comparing(Property::getName))
            .map(p -> p.getName()+"="+property(s,p)).collect(Collectors.joining(","));
        return name+(props.isEmpty()?"":"["+props+"]");
    }
    static String model(BlockState s) {
        Block b=s.getBlock();
        Class<?> inside=owner(b,"entityInside",BlockBehaviour.class);
        String effect=inside==BlockBehaviour.class?"none":inside==WebBlock.class?"cobweb":inside==PowderSnowBlock.class?"powderSnow":inside==HoneyBlock.class?"honey":null;
        Class<?> fall=owner(b,"fallOn",Block.class);
        if (effect==null || !(fall==Block.class || fall==HayBlock.class || fall==HoneyBlock.class || fall==BedBlock.class || fall==SlimeBlock.class || fall==PowderSnowBlock.class)) return null;
        Class<?> step=owner(b,"stepOn",Block.class);
        if (!(step==Block.class || step==SlimeBlock.class || step==MagmaBlock.class)) return null;
        var fluid=s.getFluidState();
        if (!fluid.isEmpty() && !fluid.getType().isSame(Fluids.WATER)) return null;
        boolean[] flags={s.isAir(),s.isSuffocating(LOCAL,BlockPos.ZERO),b instanceof LiquidBlock && fluid.getType().isSame(Fluids.WATER),
            tag(s,BlockTags.CLIMBABLE),b instanceof ScaffoldingBlock,b instanceof SlimeBlock,b instanceof BedBlock,
            tag(s,BlockTags.FENCES),tag(s,BlockTags.WALLS)||tag(s,BlockTags.FENCE_GATES),tag(s,BlockTags.SUPPRESSES_BOUNCE),
            tag(s,BlockTags.FALL_DAMAGE_RESETTING),false};
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
            fluid.isEmpty()?"false":fluid.getValue(FlowingFluid.FALLING).toString(),Boolean.toString(s.blocksMotion()),Integer.toString(faces),effect,
            b instanceof MagmaBlock?"hotFloor":"none",
            shape(s.getCollisionShape(LOCAL,BlockPos.ZERO,CollisionContext.empty())),
            // PowderSnowBlock returns Shapes.block when the unbooted collision shape is empty (26.2 bytecode).
            shape(b instanceof PowderSnowBlock?Shapes.block():s.getEntityInsideCollisionShape(LOCAL,BlockPos.ZERO,null)),
            shape(s.getCollisionShape(LOCAL,BlockPos.ZERO,new Context(false,1))),
            shape(s.getCollisionShape(LOCAL,BlockPos.ZERO,new Context(false,-2))),
            shape(s.getCollisionShape(LOCAL,BlockPos.ZERO,new Context(true,1))));
    }
    public static void main(String[] args) throws Exception {
        SharedConstants.tryDetectVersion(); Bootstrap.bootStrap();
        var rows=new TreeMap<String,String>();
        for(Block b:BuiltInRegistries.BLOCK) {
            if(!FAMILIES.contains(b.getClass().getSimpleName())) continue;
            for(BlockState s:b.getStateDefinition().getPossibleStates()) {
                String model=model(s);
                if(model != null) rows.put(state(s),model);
            }
        }
        // Equal physical models share one record; state identity still stays exact.
        var models=new TreeSet<String>(rows.values());
        var ids=new HashMap<String,Integer>(); var modelLines=new ArrayList<String>();
        for(String m:models) { ids.put(m,ids.size()); modelLines.add(m); }
        var stateLines=new ArrayList<String>();
        for(var entry:rows.entrySet()) stateLines.add(entry.getKey()+"\t"+ids.get(entry.getValue()));
        Files.write(Path.of(args[0],"models.tsv"),modelLines);
        var defaults=new TreeSet<String>();
        for(Block b:BuiltInRegistries.BLOCK) if(rows.containsKey(state(b.defaultBlockState()))) defaults.add(state(b.defaultBlockState()));
        Files.write(Path.of(args[0],"defaults.txt"),defaults);
        Files.writeString(Path.of(args[0],"version.txt"),Integer.toString(SharedConstants.getCurrentVersion().dataVersion().version())+"\n");
        Files.write(Path.of(args[0],"states.tsv"),stateLines);
        System.out.println(rows.size()+" states, "+models.size()+" physical models");
    }
}



