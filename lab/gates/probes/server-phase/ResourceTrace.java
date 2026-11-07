package pathtracing.phase;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardOpenOption;
import java.util.HexFormat;
import net.minecraft.server.level.ServerPlayer;
import pathtracing.phase.mixin.FoodFields;
import net.minecraft.world.level.GameType;

public final class ResourceTrace {
    static final boolean ENABLED = "1".equals(System.getenv("PATHTRACING_RESOURCE_PROBE"))
        && Files.isRegularFile(Path.of("..", "resource-pair.disposable"));
    static final boolean CORPUS = ENABLED && "1".equals(System.getenv("PATHTRACING_TERRAIN_CORPUS"))
        && Files.isRegularFile(Path.of("..", "terrain-corpus.disposable"));
    private static final HexFormat HEX = HexFormat.of();
    private static ResourceTrace active;
    private final ServerPlayer player;
    private final String run;
    private final StringBuilder rows;
    private int order;
    private float beforeSaturation;
    private int beforeFood;
    private boolean depleted;
    private boolean refilled;
    private boolean inFoodTick;

    private ResourceTrace(ServerPlayer player, String run) {
        this.player = player;
        this.run = run;
        rows = new StringBuilder(CORPUS
            ? "resource-v2," + run + "," + player.getUUID() + "," + player.getId() + "\n"
            : "resource-v1," + run + "\n");
    }

    static void start(ServerPlayer player, String run) {
        if (active != null || !run.matches("[A-Za-z0-9_-]+")) {
            throw new IllegalStateException("Resource probe already active or invalid replay name");
        }
        if (!player.isAlive() || player.isPassenger() || player.getMaxHealth() != 20.0f || !player.getActiveEffects().isEmpty()
            || player.gameMode() != GameType.SURVIVAL) {
            throw new IllegalStateException("Expected an unmounted survival player with default max health and no effects");
        }
        active = new ResourceTrace(player, run);
        active.snapshot("start");
    }

    static void stop(String run) {
        var trace = active;
        if (trace == null || !trace.run.equals(run) || trace.player.isRemoved()) {
            throw new IllegalStateException("No matching resource capture for " + run);
        }
        trace.snapshot("end");
        active = null;
        trace.write(".csv");
    }

    private void write(String suffix) {
        try {
            var directory = Files.createDirectories(Path.of("pathtracing-phase"));
            Files.writeString(directory.resolve(run + suffix), rows, StandardOpenOption.CREATE_NEW);
        } catch (IOException failure) {
            throw new IllegalStateException("Cannot write resource trace for " + run, failure);
        }
    }

    static void cancel() {
        var trace = active;
        active = null;
        if (CORPUS && trace != null) trace.write(".resources.partial.csv");
    }

    public static void charge(Object food, float amount) {
        var trace = active;
        if (trace != null && !trace.inFoodTick && trace.player.getFoodData() == food) {
            trace.row("charge," + bits(amount));
        }
    }

    public static void damage(Object player, float amount) {
        var trace = active;
        if (trace != null && !trace.inFoodTick && trace.player == player) {
            trace.row("damage," + bits(amount));
        }
    }

    public static void beforeRegeneration(ServerPlayer player) {
        var trace = active;
        if (trace == null || trace.player != player) return;
        trace.row("regen");
        trace.beforeSaturation = player.getFoodData().getSaturationLevel();
    }

    public static void afterRegeneration(ServerPlayer player) {
        var trace = active;
        if (CORPUS && trace != null && trace.player == player) {
            trace.snapshot("after-regen");
            return;
        }
        if (trace != null && trace.player == player && trace.depleted && !trace.refilled
            && player.getFoodData().getSaturationLevel() > trace.beforeSaturation) {
            trace.row("refill");
            trace.refilled = true;
        }
    }

    public static void beforeFood(ServerPlayer player) {
        var trace = active;
        if (trace == null || trace.player != player) return;
        var food = player.getFoodData();
        trace.row("food");
        trace.inFoodTick = true;
        trace.beforeSaturation = food.getSaturationLevel();
        trace.beforeFood = food.getFoodLevel();
    }

    public static void afterFood(ServerPlayer player) {
        var trace = active;
        if (trace == null || trace.player != player) return;
        trace.inFoodTick = false;
        if (CORPUS) {
            trace.snapshot("after-food");
            return;
        }
        if (trace.depleted) return;
        var food = player.getFoodData();
        if (food.getSaturationLevel() < trace.beforeSaturation || food.getFoodLevel() < trace.beforeFood) {
            trace.snapshot("depleted");
            trace.depleted = true;
        }
    }

    private void snapshot(String kind) {
        var food = player.getFoodData();
        var fields = (FoodFields) food;
        row(kind + "," + bits(player.getHealth()) + "," + food.getFoodLevel() + ","
            + bits(food.getSaturationLevel()) + "," + bits(fields.ptExhaustion()) + "," + fields.ptFoodTimer());
    }

    private void row(String data) {
        rows.append(order++).append(',').append(player.level().getGameTime()).append(',')
            .append(player.tickCount).append(',').append(data).append('\n');
    }

    private static String bits(float value) {
        return HEX.toHexDigits(Float.floatToRawIntBits(value));
    }
}
