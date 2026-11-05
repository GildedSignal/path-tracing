package pathtracing;

import java.nio.file.Files;
import java.nio.file.Path;
import java.util.TreeMap;
import net.minecraft.SharedConstants;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.core.Direction;
import net.minecraft.resources.Identifier;
import net.minecraft.server.Bootstrap;
import net.minecraft.world.entity.ai.attributes.AttributeInstance;
import net.minecraft.world.entity.ai.attributes.AttributeModifier;
import net.minecraft.world.entity.ai.attributes.Attributes;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.state.BlockBehaviour;
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

    private static void check(Block block, boolean forceNone) {
        boolean hook = overrides(block.getClass(), "entityInside", BlockBehaviour.class)
            || overrides(block.getClass(), "fallOn", Block.class);
        String inside = forceNone ? "none" : WorldExporter.inside(block);
        if (hook && inside.equals("none")) throw new AssertionError("Hook silently dropped for " + block);
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
        var classes = new TreeMap<String, String>();
        for (String id : Files.readAllLines(Path.of(args[0]))) {
            var key = Identifier.parse("minecraft:" + id);
            Block block = BuiltInRegistries.BLOCK.getValue(key);
            if (!BuiltInRegistries.BLOCK.getKey(block).equals(key)) throw new AssertionError("Missing block " + id);
            check(block, false);
            classes.put(block.getClass().getSimpleName(), WorldExporter.inside(block));
        }
        classes.forEach((type, inside) -> System.out.println(type + " inside=" + inside + " unmodeled=" + inside.equals("unmodeled")));
        // Unmodeled overrides absent from the courses exercise both hook paths and inherited hooks.
        for (String id : new String[]{"pointed_dripstone", "bubble_column", "cactus"}) {
            Block block = BuiltInRegistries.BLOCK.getValue(Identifier.parse("minecraft:" + id));
            check(block, false);
            if (!WorldExporter.inside(block).equals("unmodeled")) throw new AssertionError(id);
            boolean caught = false;
            try { check(block, true); } catch (AssertionError expected) { caught = true; }
            if (!caught) throw new AssertionError("none mutant survived for " + id);
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
        System.out.println("PASS course exporter hooks, none mutants, collision bits, modifier tuples and outer-corner axis orders");
    }
}
