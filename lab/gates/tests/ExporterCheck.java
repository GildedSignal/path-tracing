package pathtracing;

import com.google.gson.Gson;
import com.google.gson.JsonObject;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.TreeMap;
import net.minecraft.SharedConstants;
import net.minecraft.core.BlockPos;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.core.Direction;
import net.minecraft.resources.Identifier;
import net.minecraft.server.Bootstrap;
import net.minecraft.world.entity.ai.attributes.AttributeInstance;
import net.minecraft.world.entity.ai.attributes.AttributeModifier;
import net.minecraft.world.entity.ai.attributes.Attributes;
import net.minecraft.world.level.EmptyBlockGetter;
import net.minecraft.world.level.block.BedBlock;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.PointedDripstoneBlock;
import net.minecraft.world.level.block.state.BlockBehaviour;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.phys.Vec3;
import net.minecraft.world.phys.AABB;
import net.minecraft.world.phys.shapes.Shapes;

/** Offline registry bootstrap only. No client, level or server is started. */
public final class ExporterCheck {
    private static boolean overrides(Class<?> type, String hook, Class<?> base) {
        if (type == base) return false;
        for (var method : type.getDeclaredMethods()) {
            if (method.getName().equals(hook)) return true;
        }
        return overrides(type.getSuperclass(), hook, base);
    }

    private static Block registryBlock(String id) {
        var key = Identifier.parse("minecraft:" + id);
        Block block = BuiltInRegistries.BLOCK.getValue(key);
        if (!BuiltInRegistries.BLOCK.getKey(block).equals(key)) throw new AssertionError("Missing block " + id);
        return block;
    }

    private static JsonObject export(BlockState state) throws Exception {
        var properties = WorldExporter.class.getDeclaredMethod("properties", JsonObject.class, BlockState.class);
        properties.setAccessible(true);
        var out = new JsonObject();
        out.addProperty("block", BuiltInRegistries.BLOCK.getKey(state.getBlock()).toString());
        var shape = state.getCollisionShape(EmptyBlockGetter.INSTANCE, BlockPos.ZERO);
        out.add("boxes", new Gson().toJsonTree(shape.toAabbs().stream().map(box ->
            new double[]{box.minX, box.minY, box.minZ, box.maxX, box.maxY, box.maxZ}).toList()));
        properties.invoke(null, out, state);
        return out;
    }

    private static void hookFields(JsonObject row, String inside, boolean unsupportedFallOn) {
        if (!row.get("inside").getAsString().equals(inside)
            || !row.has("unsupportedFallOn") || row.get("unsupportedFallOn").getAsBoolean() != unsupportedFallOn) {
            throw new AssertionError("Expected inside=" + inside + " unsupportedFallOn=" + unsupportedFallOn + " for " + row);
        }
    }

    private static void landing(JsonObject row, float multiplier, float bounce, boolean slime, boolean bed) {
        hookFields(row, "none", false);
        if (!row.get("fallOnMultiplierBits").getAsString().equals("%08x".formatted(Float.floatToRawIntBits(multiplier)))
            || !row.get("bounceBits").getAsString().equals("%08x".formatted(Float.floatToRawIntBits(bounce)))
            || row.get("slime").getAsBoolean() != slime || row.get("bed").getAsBoolean() != bed) {
            throw new AssertionError("Landing data changed for " + row);
        }
    }

    private static void contactRows(Path output) throws Exception {
        var rows = new ArrayList<String>();
        var slime = export(registryBlock("slime_block").defaultBlockState());
        landing(slime, 1.0F, 1.0F, true, false);
        rows.add(slime.toString());
        var hay = export(registryBlock("hay_block").defaultBlockState());
        landing(hay, 0.2F, 0.0F, false, false);
        rows.add(hay.toString());
        for (Block block : BuiltInRegistries.BLOCK) {
            if (!(block instanceof BedBlock)) continue;
            var bed = export(block.defaultBlockState());
            landing(bed, 1.0F, 0.75F, false, true);
            rows.add(bed.toString());
        }
        Block berry = registryBlock("sweet_berry_bush");
        if (!overrides(berry.getClass(), "entityInside", BlockBehaviour.class)) throw new AssertionError(berry);
        var bush = export(berry.defaultBlockState());
        hookFields(bush, "unmodeled", false);
        rows.add(bush.toString());
        Block pointed = registryBlock("pointed_dripstone");
        if (!overrides(pointed.getClass(), "fallOn", Block.class)) throw new AssertionError(pointed);
        var dripstone = export(pointed.defaultBlockState().setValue(PointedDripstoneBlock.TIP_DIRECTION, Direction.UP));
        hookFields(dripstone, "none", true);
        rows.add(dripstone.toString());
        Files.write(output, rows);
    }

    private static Vec3 corner(boolean xFirst) {
        // Course coordinates keep both gaps in the same double binade.
        AABB body = new AABB(5779.3, -60, 4103.3, 5779.9, -58.2, 4103.9);
        var column = java.util.List.of(Shapes.create(new AABB(5780, -60, 4104, 5781, -58, 4105)));
        double x = 0.2;
        double z = 0.2;
        if (xFirst) {
            x = Shapes.collide(Direction.Axis.X, body, column, x);
            z = Shapes.collide(Direction.Axis.Z, body.move(x, 0, 0), column, z);
        } else {
            z = Shapes.collide(Direction.Axis.Z, body, column, z);
            x = Shapes.collide(Direction.Axis.X, body.move(0, 0, z), column, x);
        }
        return new Vec3(x, 0, z);
    }

    private static void replayPause() throws Exception {
        var replay = Recorder.class.getDeclaredField("replay");
        var start = Recorder.class.getDeclaredField("start");
        var waiting = Recorder.class.getDeclaredField("awaitStart");
        replay.setAccessible(true);
        start.setAccessible(true);
        waiting.setAccessible(true);
        try {
            replay.set(null, java.util.List.of());
            waiting.setBoolean(null, true);
            start.set(null, new double[5]);
            boolean caught = false;
            try { Recorder.continueReplay(); } catch (IllegalStateException expected) { caught = true; }
            if (!caught) throw new AssertionError("Unsettled replay continued");
            start.set(null, null);
            Recorder.continueReplay();
            if (waiting.getBoolean(null)) throw new AssertionError("Settled replay stayed paused");
            replay.set(null, null);
            caught = false;
            try { Recorder.continueReplay(); } catch (IllegalStateException expected) { caught = true; }
            if (!caught) throw new AssertionError("Inactive replay continued");
        } finally {
            replay.set(null, null);
            start.set(null, null);
            waiting.setBoolean(null, false);
        }
    }

    public static void main(String[] args) throws Exception {
        SharedConstants.tryDetectVersion();
        Bootstrap.bootStrap();
        replayPause();
        Vec3 xFirst = corner(true);
        Vec3 zFirst = corner(false);
        if (xFirst.x != 0.2 || xFirst.z == 0.2 || zFirst.x == 0.2 || zFirst.z != 0.2
            || !xFirst.equals(new Vec3(zFirst.z, 0, zFirst.x))) {
            throw new AssertionError("Outer corner failed to distinguish axis order");
        }
        contactRows(Path.of(args[1]));
        var classes = new TreeMap<String, String>();
        for (String id : Files.readAllLines(Path.of(args[0]))) {
            Block block = registryBlock(id);
            var row = export(block.defaultBlockState());
            var inside = row.get("inside").getAsString();
            boolean unsupportedFallOn = row.get("unsupportedFallOn").getAsBoolean();
            if (inside.equals("none") && overrides(block.getClass(), "entityInside", BlockBehaviour.class)) {
                throw new AssertionError("Inside hook silently dropped for " + block);
            }
            if (!unsupportedFallOn && overrides(block.getClass(), "fallOn", Block.class)
                && !row.get("slime").getAsBoolean() && !row.get("bed").getAsBoolean()
                && row.get("fallOnMultiplier").getAsFloat() == 1.0F && !inside.equals("powderSnow")) {
                throw new AssertionError("Landing hook silently dropped for " + block);
            }
            classes.put(block.getClass().getSimpleName(), "inside=" + inside + " unsupportedFallOn=" + unsupportedFallOn);
        }
        classes.forEach((type, hooks) -> System.out.println(type + " " + hooks));
        for (String id : new String[]{"bubble_column", "cactus"}) {
            hookFields(export(registryBlock(id).defaultBlockState()), "unmodeled", false);
        }
        var bits = TraceJson.vectorBits(new Vec3(1, -0.0, 2));
        if (bits.size() != 3 || !bits.get("yBits").getAsString().equals("8000000000000000")) {
            throw new AssertionError("Collision vectors must contain only raw bits");
        }
        var attribute = new AttributeInstance(Attributes.MOVEMENT_SPEED, ignored -> {});
        for (var operation : AttributeModifier.Operation.values()) {
            attribute.addTransientModifier(new AttributeModifier(Identifier.parse("test:op_" + operation.id()), -0.125, operation));
        }
        attribute.addTransientModifier(new AttributeModifier(Identifier.parse("minecraft:sprinting"), 0.3, AttributeModifier.Operation.ADD_MULTIPLIED_TOTAL));
        var modifiers = TraceJson.speedModifiers(attribute);
        if (modifiers.size() != 3) throw new AssertionError("Sprinting filter lost modifiers");
        for (var entry : modifiers) {
            var row = entry.getAsJsonArray();
            int operation = row.get(2).getAsInt();
            if (row.size() != 3 || !row.get(0).getAsString().equals("test:op_" + operation)
                || row.get(1).getAsDouble() != -0.125) throw new AssertionError("Modifier tuple changed");
        }
        System.out.println("PASS registry exporter hooks, landing data, collision bits, modifier tuples and outer-corner axis orders");
    }
}
