package pathtracing;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.StandardOpenOption;
import java.util.HexFormat;
import net.minecraft.client.Minecraft;
import net.minecraft.core.BlockPos;
import net.minecraft.network.Connection;
import net.minecraft.network.protocol.game.ClientboundSetEntityMotionPacket;
import net.minecraft.network.protocol.game.ServerboundMovePlayerPacket;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.phys.Vec3;
import org.slf4j.LoggerFactory;

/** Independent main-thread observation between two ordered packets on the replay connection. */
public final class LandingClientTrace {
    public static final boolean ENABLED = "1".equals(System.getenv("PATHTRACING_LANDING_PROBE"))
        && "bot".equals(System.getProperty("pathtracing.role"));
    public static final boolean CORPUS = ENABLED && "1".equals(System.getenv("PATHTRACING_TERRAIN_CORPUS"))
        && Files.isRegularFile(java.nio.file.Path.of(System.getProperty("pathtracing.rig", "."), "terrain-corpus.disposable"));
    private static final HexFormat HEX = HexFormat.of();
    private static Connection connection;
    private static long moves;
    private static String run;
    private static StringBuilder rows;
    private static long motions;
    private static Vec3 before;
    private static Vec3 after;
    private static boolean inTick;
    private static int replayTick = -1;
    private static long started;

    public static void fence(String message) {
        if (!ENABLED || !message.startsWith("pt-landing-v1 ")) return;
        var parts = message.split(" ");
        if (parts.length != 3 || !parts[2].matches("[A-Za-z0-9_-]+")) return;
        var mc = Minecraft.getInstance();
        var player = mc.player;
        if (player == null || mc.getConnection() == null) return;
        var fence = parts[1] + "," + parts[2] + "," + player.getUUID() + "," + player.getId();
        if (parts[1].equals("start") && rows == null) {
            run = parts[2];
            rows = new StringBuilder();
            motions = 0;
            row(fence);
        } else if (parts[1].equals("end") && parts[2].equals(run) && rows != null) {
            row(fence);
            write(".client-motion.csv");
            rows = null;
            run = null;
        }
    }

    static void tickStart() { inTick = true; }
    static void tickEnd() { inTick = false; }

    static void replayStart(String name, int tick, int playerTick, float rate) {
        if (rows == null) return;
        replayTick = tick;
        if (tick == 0) {
            started = System.nanoTime();
            row("begin," + name + "," + playerTick + "," + rate
                + "," + Boolean.getBoolean("pathtracing.followTickRate")
                + "," + Boolean.getBoolean("pathtracing.fullTickDemand"));
        }
    }

    static void replayEnd(String name, int ticks, boolean complete) {
        if (rows != null) {
            row("finished," + name + "," + ticks + "," + complete + "," + (System.nanoTime() - started));
            write(".client-motion.partial.csv");
        }
        replayTick = -1;
    }

    public static void move(Connection sender, ServerboundMovePlayerPacket packet) {
        if (!ENABLED) return;
        var mc = Minecraft.getInstance();
        if (mc.getConnection() == null || sender != mc.getConnection().getConnection()) return;
        if (connection != sender) {
            connection = sender;
            moves = 0;
        }
        long ordinal = ++moves;
        if (rows == null || replayTick < 0 || !inTick) return;
        row("move," + ordinal + "," + replayTick + "," + mc.player.tickCount + ","
            + packet.isOnGround() + "," + packet.horizontalCollision() + "," + packet.hasPosition() + ","
            + bits(new Vec3(packet.getX(0), packet.getY(0), packet.getZ(0))));
    }

    public static void beforeMotion(ClientboundSetEntityMotionPacket packet, Entity target) {
        if (ours(packet) && target == Minecraft.getInstance().player) before = target.getDeltaMovement();
    }

    public static void afterMotion(ClientboundSetEntityMotionPacket packet, Entity target) {
        if (ours(packet) && target == Minecraft.getInstance().player) after = target.getDeltaMovement();
    }

    public static void motion(ClientboundSetEntityMotionPacket packet, int nextTick) {
        if (!ours(packet)) return;
        var player = Minecraft.getInstance().player;
        row("motion," + (++motions) + "," + player.getUUID() + "," + packet.id() + "," + nextTick
            + "," + player.tickCount + "," + inTick + "," + bits(packet.movement())
            + "," + bits(before) + "," + bits(after));
        before = null;
        after = null;
    }

    public static void unexpected(String kind) {
        if (CORPUS && rows != null) row("unsupported," + replayTick + "," + kind);
    }

    public static void ownEntityPacket(int id, String kind) {
        var player = Minecraft.getInstance().player;
        if (player != null && player.getId() == id) unexpected(kind);
    }

    public static void blockUpdate(BlockPos pos) {
        if (CORPUS && rows != null) row("block-update," + replayTick + "," + pos.getX() + "," + pos.getY() + "," + pos.getZ());
    }

    public static void chunkUpdate(int x, int z) {
        if (CORPUS && rows != null) row("chunk-update," + replayTick + "," + x + "," + z);
    }

    static void checkPlayer(Minecraft mc, int tick) {
        if (rows == null) return;
        var player = mc.player;
        if (!player.isAlive() || player.getFoodData().getFoodLevel() <= 6 || player.isPassenger()
            || player.isFallFlying() || player.getAbilities().flying || player.isUsingItem()
            || player.noPhysics || player.isSpectator() || player.getAbilities().instabuild
            || !player.getActiveEffects().isEmpty()) {
            row("unsupported," + tick + ",player movement state");
        }
    }

    private static boolean ours(ClientboundSetEntityMotionPacket packet) {
        var player = Minecraft.getInstance().player;
        return rows != null && player != null && packet.id() == player.getId();
    }

    private static void row(String data) { rows.append(data).append('\n'); }

    private static void write(String suffix) {
        var directory = Minecraft.getInstance().gameDirectory.toPath().resolve("pathtracing/recordings");
        try {
            var partial = directory.resolve(run + suffix + ".tmp");
            Files.writeString(partial, rows, StandardOpenOption.CREATE_NEW);
            Files.move(partial, directory.resolve(run + suffix));
        } catch (IOException e) {
            LoggerFactory.getLogger("pathtracing").error("Cannot save client landing trace for " + run, e);
        }
    }

    private static String bits(Vec3 value) {
        if (value == null) return "missing";
        return HEX.toHexDigits(Double.doubleToRawLongBits(value.x)) + ","
            + HEX.toHexDigits(Double.doubleToRawLongBits(value.y)) + ","
            + HEX.toHexDigits(Double.doubleToRawLongBits(value.z));
    }
}
