package pathtracing;

import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import java.io.BufferedWriter;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import net.minecraft.client.Minecraft;
import net.minecraft.core.BlockPos;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.util.Mth;
import net.minecraft.world.InteractionHand;
import net.minecraft.world.entity.Pose;
import net.minecraft.world.entity.player.Input;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.item.Items;
import net.minecraft.world.level.BlockGetter;
import net.minecraft.world.level.EmptyBlockGetter;
import net.minecraft.world.level.block.entity.BlockEntity;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.material.FluidState;
import net.minecraft.world.phys.AABB;
import net.minecraft.world.phys.Vec3;
import net.minecraft.world.phys.shapes.CollisionContext;
import pathtracing.mixin.BlockBehaviourAccess;

/** Samples the registry's law, with only the block under test present in the getter. */
final class BlockDumper {
    private static final BlockPos[] OFFSET_POSITIONS = {
        BlockPos.ZERO, new BlockPos(1, 0, 0), new BlockPos(0, 1, 0), new BlockPos(0, 0, 1),
        new BlockPos(-1, -1, -1), new BlockPos(16, 64, 16), new BlockPos(-17, -64, 31),
        new BlockPos(30000000, 255, -30000000)
    };
    private static final String[] CONTEXTS = {"empty", "above", "below", "sneakingAbove", "holdingScaffolding"};

    static String dump(Minecraft mc) throws IOException {
        if (Recorder.active()) throw new IllegalStateException("A recording or replay is running; cannot dump blocks.");
        var player = mc.player;
        Vec3 position = player.position();
        AABB bounds = player.getBoundingBox();
        Pose pose = player.getPose();
        Input input = player.input.keyPresses;
        double fallDistance = player.fallDistance;
        ItemStack mainHand = player.getMainHandItem();
        Path path = Recorder.file(mc, "dumps", "blocks");
        Files.createDirectories(path.getParent());
        Path temporary = path.resolveSibling(path.getFileName() + ".tmp");
        int registryCount = 0;
        for (var block : BuiltInRegistries.BLOCK) registryCount += block.getStateDefinition().getPossibleStates().size();
        int count = 0;
        try (BufferedWriter writer = Files.newBufferedWriter(temporary, StandardCharsets.UTF_8)) {
            JsonObject header = new JsonObject();
            header.addProperty("type", "dump");
            header.add("mainHand", item(mainHand));
            header.add("offHand", item(player.getOffhandItem()));
            writer.write(header.toString());
            writer.write('\n');
            for (var block : BuiltInRegistries.BLOCK) {
                for (BlockState state : block.getStateDefinition().getPossibleStates()) {
                    BlockGetter getter = new StateGetter(state);
                    JsonObject out = new JsonObject();
                    out.addProperty("block", BuiltInRegistries.BLOCK.getKey(block).toString());
                    JsonObject properties = new JsonObject();
                    state.getValues().sorted(Comparator.comparing(v -> v.property().getName()))
                        .forEach(v -> properties.addProperty(v.property().getName(), v.valueName()));
                    out.add("properties", properties);
                    TraceJson.number(out, "friction", block.getFriction());
                    TraceJson.number(out, "speedFactor", block.getSpeedFactor());
                    TraceJson.number(out, "jumpFactor", block.getJumpFactor());
                    TraceJson.number(out, "defaultDestroyTime", block.defaultDestroyTime());
                    out.addProperty("requiresCorrectToolForDrops", state.requiresCorrectToolForDrops());
                    JsonArray tags = new JsonArray();
                    state.tags().map(tag -> tag.location().toString()).sorted().forEach(tags::add);
                    out.add("tags", tags);
                    out.addProperty("isCollisionShapeFullBlock", state.isCollisionShapeFullBlock(getter, BlockPos.ZERO));
                    JsonArray shapes = new JsonArray();
                    List<JsonArray> distinct = new ArrayList<>();
                    List<JsonArray> names = new ArrayList<>();
                    for (int i = 0; i < CONTEXTS.length; i++) {
                        player.setItemInHand(InteractionHand.MAIN_HAND, i == 4 ? new ItemStack(Items.SCAFFOLDING) : mainHand);
                        CollisionContext context;
                        if (i == 0) context = CollisionContext.empty();
                        else {
                            player.setPose(i == 3 ? Pose.CROUCHING : Pose.STANDING);
                            // LocalPlayer reads shift from its input, rather than Entity's shared flag.
                            player.input.keyPresses = new Input(false, false, false, false, false, i == 3, false);
                            player.fallDistance = 0;
                            player.setPos(0.5, i == 2 ? -2 : 1, 0.5);
                            context = CollisionContext.of(player);
                        }
                        JsonArray boxes = boxes(state.getCollisionShape(getter, BlockPos.ZERO, context).toAabbs());
                        int index = distinct.indexOf(boxes);
                        if (index == -1) {
                            index = distinct.size();
                            distinct.add(boxes);
                            JsonArray contexts = new JsonArray();
                            names.add(contexts);
                            JsonObject shape = new JsonObject();
                            shape.add("contexts", contexts);
                            shape.add("aabbs", boxes);
                            shapes.add(shape);
                        }
                        names.get(index).add(CONTEXTS[i]);
                    }
                    out.add("collisionShapes", shapes);
                    if (state.hasOffsetFunction()) out.add("offset", offset(state));
                    writer.write(out.toString());
                    writer.write('\n');
                    count++;
                }
            }
            JsonObject summary = new JsonObject();
            summary.addProperty("type", "summary");
            summary.addProperty("writtenCount", count);
            summary.addProperty("registryCount", registryCount);
            writer.write(summary.toString());
            writer.write('\n');
            if (count != registryCount) {
                throw new IllegalStateException("Dump count mismatch: written " + count + ", registry " + registryCount + ".");
            }
        } finally {
            player.setItemInHand(InteractionHand.MAIN_HAND, mainHand);
            player.input.keyPresses = input;
            player.setPose(pose);
            player.fallDistance = fallDistance;
            player.setPos(position);
            player.setBoundingBox(bounds);
        }
        Files.move(temporary, path, StandardCopyOption.ATOMIC_MOVE, StandardCopyOption.REPLACE_EXISTING);
        return "Dumped " + count + " block states; independent registry count " + registryCount + ".";
    }

    private static JsonObject item(ItemStack stack) {
        JsonObject out = new JsonObject();
        out.addProperty("item", BuiltInRegistries.ITEM.getKey(stack.getItem()).toString());
        out.addProperty("count", stack.getCount());
        return out;
    }

    private static JsonArray boxes(List<AABB> boxes) {
        JsonArray out = new JsonArray();
        for (AABB box : boxes) {
            JsonObject value = new JsonObject();
            TraceJson.number(value, "minX", box.minX);
            TraceJson.number(value, "minY", box.minY);
            TraceJson.number(value, "minZ", box.minZ);
            TraceJson.number(value, "maxX", box.maxX);
            TraceJson.number(value, "maxY", box.maxY);
            TraceJson.number(value, "maxZ", box.maxZ);
            out.add(value);
        }
        return out;
    }

    private static JsonObject offset(BlockState state) {
        BlockBehaviourAccess access = (BlockBehaviourAccess) state.getBlock();
        float horizontal = access.pathtracing$maxHorizontalOffset();
        float vertical = access.pathtracing$maxVerticalOffset();
        boolean xyz = state.getOffset(BlockPos.ZERO).y != 0;
        JsonObject out = new JsonObject();
        out.addProperty("method", "net.minecraft.world.level.block.state.BlockBehaviour.Properties.offsetType");
        out.addProperty("type", xyz ? "XYZ" : "XZ");
        TraceJson.number(out, "maxHorizontalOffset", horizontal);
        TraceJson.number(out, "maxVerticalOffset", vertical);
        out.addProperty("seed", "long s = (long)(x * 3129871) ^ (long)z * 116129781L; s = (s * s * 42317861L + s * 11L) >> 16;");
        out.addProperty("x", "clamp(((double)((float)(s & 15L) / 15.0F) - 0.5D) * 0.5D, -(double)maxHorizontalOffset, (double)maxHorizontalOffset)");
        out.addProperty("y", xyz ? "((double)((float)((s >> 4) & 15L) / 15.0F) - 1.0D) * (double)maxVerticalOffset" : "0.0D");
        out.addProperty("z", "clamp(((double)((float)((s >> 8) & 15L) / 15.0F) - 0.5D) * 0.5D, -(double)maxHorizontalOffset, (double)maxHorizontalOffset)");
        JsonArray samples = new JsonArray();
        for (BlockPos position : OFFSET_POSITIONS) {
            Vec3 value = state.getOffset(position);
            long seed = (long) (position.getX() * 3129871) ^ (long) position.getZ() * 116129781L;
            seed = (seed * seed * 42317861L + seed * 11L) >> 16;
            double x = Mth.clamp(((double) ((float) (seed & 15L) / 15.0F) - 0.5D) * 0.5D, -horizontal, horizontal);
            double y = xyz ? ((double) ((float) ((seed >> 4) & 15L) / 15.0F) - 1.0D) * vertical : 0;
            double z = Mth.clamp(((double) ((float) ((seed >> 8) & 15L) / 15.0F) - 0.5D) * 0.5D, -horizontal, horizontal);
            if (!same(value.x, x) || !same(value.y, y) || !same(value.z, z)) {
                throw new IllegalStateException("Unknown offset generator for " + state);
            }
            JsonObject sample = new JsonObject();
            JsonArray pos = new JsonArray();
            pos.add(position.getX());
            pos.add(position.getY());
            pos.add(position.getZ());
            sample.add("pos", pos);
            sample.add("offset", TraceJson.vector(value));
            samples.add(sample);
        }
        out.add("samples", samples);
        return out;
    }

    private static boolean same(double a, double b) {
        return Double.doubleToRawLongBits(a) == Double.doubleToRawLongBits(b);
    }

    private record StateGetter(BlockState state) implements BlockGetter {
        @Override
        public BlockEntity getBlockEntity(BlockPos pos) {
            return null;
        }

        @Override
        public BlockState getBlockState(BlockPos pos) {
            return pos.equals(BlockPos.ZERO) ? state : EmptyBlockGetter.INSTANCE.getBlockState(pos);
        }

        @Override
        public FluidState getFluidState(BlockPos pos) {
            return getBlockState(pos).getFluidState();
        }

        @Override
        public int getMinY() {
            return EmptyBlockGetter.INSTANCE.getMinY();
        }

        @Override
        public int getHeight() {
            return EmptyBlockGetter.INSTANCE.getHeight();
        }
    }
}
