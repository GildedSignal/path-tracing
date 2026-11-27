import java.lang.reflect.InvocationHandler;
import java.lang.reflect.Proxy;
import java.util.*;
import net.minecraft.SharedConstants;
import net.minecraft.core.*;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.resources.Identifier;
import net.minecraft.server.Bootstrap;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.packs.*;
import net.minecraft.server.packs.repository.ServerPacksSource;
import net.minecraft.server.packs.resources.MultiPackResourceManager;
import net.minecraft.tags.TagLoader;
import net.minecraft.util.BlockUtil;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.entity.LivingEntity;
import net.minecraft.world.entity.decoration.ArmorStand;
import net.minecraft.world.level.*;
import net.minecraft.world.level.block.*;
import net.minecraft.world.level.block.entity.BlockEntity;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.border.WorldBorder;
import net.minecraft.world.level.material.*;
import net.minecraft.world.level.portal.PortalForcer;
import net.minecraft.world.phys.shapes.CollisionContext;
import sun.misc.Unsafe;

// Run against each vanilla jar. Probe real methods using only in-memory block and portal views.
class JarBoundaries {
    static class ExitProbe extends PortalForcer {
        int creationAttempts;
        ExitProbe() { super(null); }
        @Override public Optional<BlockPos> findClosestPortalPosition(BlockPos pos, boolean nether, WorldBorder border) {
            return Optional.empty();
        }
        @Override public Optional<BlockUtil.FoundRectangle> createPortal(BlockPos pos, Direction.Axis axis) {
            creationAttempts++;
            return Optional.empty();
        }
    }
    static class PortalLevel extends ServerLevel {
        ExitProbe forcer;
        PortalLevel() { super(null, null, null, null, null, null, false, 0, List.of(), false); }
        @Override public PortalForcer getPortalForcer() { return forcer; }
        @Override public BlockState getBlockState(BlockPos pos) { return Blocks.NETHER_PORTAL.defaultBlockState(); }
    }
    static class Traveller extends ArmorStand {
        Level source;
        boolean spectator;
        Traveller() { super(null, 0, 0, 0); }
        @Override public boolean isSpectator() { return spectator; }
        @Override public Level level() { return source; }
    }
    record FluidCourse(BlockState origin, BlockState neighbor) implements BlockGetter {
        public BlockState getBlockState(BlockPos p) {
            if (p.equals(BlockPos.ZERO)) return origin;
            if (p.equals(new BlockPos(1, 0, 0))) return neighbor;
            if (p.equals(new BlockPos(1, -1, 0))) return Blocks.WATER.defaultBlockState();
            return Blocks.STONE.defaultBlockState();
        }
        public FluidState getFluidState(BlockPos pos) { return getBlockState(pos).getFluidState(); }
        public BlockEntity getBlockEntity(BlockPos pos) { return null; }
        public int getHeight() { return 384; }
        public int getMinY() { return -64; }
    }

    public static void main(String[] args) throws Exception {
        SharedConstants.tryDetectVersion();
        Bootstrap.bootStrap();
        int version = SharedConstants.getCurrentVersion().dataVersion().version();
        var packs = ServerPacksSource.createVanillaTrustedRepository();
        packs.reload();
        packs.setSelected(List.of("vanilla"));
        try (var resources = new MultiPackResourceManager(PackType.SERVER_DATA, packs.openAllSelected())) {
            TagLoader.loadTagsForExistingRegistries(resources, RegistryAccess.fromRegistryOfRegistries(BuiltInRegistries.REGISTRY))
                .forEach(Registry.PendingTags::apply);

            var hold = FlowingFluid.class.getDeclaredMethod("canHoldAnyFluid", BlockState.class);
            hold.setAccessible(true);
            for (var block : List.of(Blocks.COBWEB, Blocks.OAK_SIGN, Blocks.OAK_SLAB, Blocks.STRUCTURE_VOID, Blocks.STONE, Blocks.AIR)) {
                var state = block.defaultBlockState();
                System.out.printf("ROW\t%d\tadmission\t%s\t%s\t%s%n", version,
                    BuiltInRegistries.BLOCK.getKey(block), hold.invoke(null, state), state.isSolid());
            }
            var water = Blocks.WATER.defaultBlockState().setValue(LiquidBlock.LEVEL, 1);
            var airCourse = new FluidCourse(water, Blocks.AIR.defaultBlockState());
            for (var neighbor : List.of(Blocks.OAK_SIGN, Blocks.AIR)) {
                var course = new FluidCourse(water, neighbor.defaultBlockState());
                var flow = water.getFluidState().getFlow(course, BlockPos.ZERO);
                System.out.printf("ROW\t%d\tcurrent\t%s\t%016x\t%016x\t%016x%n", version,
                    BuiltInRegistries.BLOCK.getKey(neighbor), Double.doubleToRawLongBits(flow.x),
                    Double.doubleToRawLongBits(flow.y), Double.doubleToRawLongBits(flow.z));
            }

            var bedNames = version == 5023 ? List.of("white_bed", "straw_bed") : List.of("white_bed");
            var reductionGetter = version == 5023 ? Block.class.getMethod("getFallDistanceReduction") : null;
            for (var name : bedNames) {
                var block = BuiltInRegistries.BLOCK.getValue(Identifier.withDefaultNamespace(name));
                var state = block.defaultBlockState();
                var bounds = state.getCollisionShape(airCourse, BlockPos.ZERO, CollisionContext.empty()).bounds();
                var fallOn = block.getClass().getMethod("fallOn", Level.class, BlockState.class, BlockPos.class, Entity.class, double.class);
                float reduction = reductionGetter == null ? 0.5f : (float) reductionGetter.invoke(block);
                System.out.printf("ROW\t%d\tbed\t%s\t%08x\t%016x\t%s\t%08x%n", version, name,
                    Float.floatToRawIntBits(block.getBounceRestitution()), Double.doubleToRawLongBits(bounds.maxY),
                    fallOn.getDeclaringClass().getSimpleName(), Float.floatToRawIntBits(reduction));
            }

            if (version == 5023) {
                var writer = (LevelWriter) Proxy.newProxyInstance(LevelWriter.class.getClassLoader(), new Class<?>[]{LevelWriter.class},
                    (proxy, method, inputs) -> {
                        if (method.isDefault()) return InvocationHandler.invokeDefault(proxy, method, inputs);
                        if (!method.getName().equals("setBlock")) throw new AssertionError(method);
                        System.out.printf("ROW\t%d\tsetBlockAndUpdate\t%d\t%d%n", version, inputs[2], inputs[3]);
                        return true;
                    });
                LevelWriter.class.getMethod("setBlockAndUpdate", BlockPos.class, BlockState.class)
                    .invoke(writer, BlockPos.ZERO, Blocks.AIR.defaultBlockState());
            }
        }

        var field = Unsafe.class.getDeclaredField("theUnsafe");
        field.setAccessible(true);
        var unsafe = (Unsafe) field.get(null);
        var level = (PortalLevel) unsafe.allocateInstance(PortalLevel.class);
        level.forcer = new ExitProbe();
        var traveller = (Traveller) unsafe.allocateInstance(Traveller.class);
        traveller.source = level;
        var exit = NetherPortalBlock.class.getDeclaredMethod("getExitPortal", ServerLevel.class,
            Entity.class, BlockPos.class, BlockPos.class, boolean.class, WorldBorder.class);
        exit.setAccessible(true);
        for (boolean spectator : new boolean[]{false, true}) {
            traveller.spectator = spectator;
            level.forcer.creationAttempts = 0;
            var transition = exit.invoke(Blocks.NETHER_PORTAL, level, traveller, BlockPos.ZERO, BlockPos.ZERO, true, null);
            System.out.printf("ROW\t%d\tportal_missing\t%s\t%d\t%s%n", version, spectator,
                level.forcer.creationAttempts, transition == null);
        }
        if (version == 5023) {
            LivingEntity.class.getField("damageCooldownTime").setInt(traveller, 20);
            var invulnerable = Entity.class.getMethod("isTemporarilyInvulnerable");
            System.out.printf("ROW\t%d\tdamage_clock\t20\t0\t%s%n", version, invulnerable.invoke(traveller));
            Entity.class.getMethod("setInvulnerableTime", int.class).invoke(traveller, 4);
            LivingEntity.class.getField("damageCooldownTime").setInt(traveller, 0);
            System.out.printf("ROW\t%d\tdamage_clock\t0\t4\t%s%n", version, invulnerable.invoke(traveller));
        }
    }
}
