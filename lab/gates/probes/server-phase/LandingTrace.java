package pathtracing.phase;

import io.netty.buffer.ByteBuf;
import io.netty.buffer.ByteBufUtil;
import io.netty.channel.Channel;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardOpenOption;
import java.util.HexFormat;
import java.util.IdentityHashMap;
import net.minecraft.network.Connection;
import net.minecraft.network.VarInt;
import net.minecraft.network.chat.Component;
import net.minecraft.network.protocol.Packet;
import net.minecraft.network.protocol.game.ClientboundSetEntityMotionPacket;
import net.minecraft.network.protocol.game.ClientboundSystemChatPacket;
import net.minecraft.network.protocol.game.ServerboundMovePlayerPacket;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.damagesource.DamageSource;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.phys.Vec3;
import org.slf4j.LoggerFactory;
import pathtracing.phase.mixin.LandingChannel;
import pathtracing.phase.mixin.LandingRecipient;

public final class LandingTrace {
    public static final boolean ENABLED = "1".equals(System.getenv("PATHTRACING_LANDING_PROBE"))
        && Files.isRegularFile(Path.of("..", "landing-motion.disposable"));
    private static final HexFormat HEX = HexFormat.of();
    private static volatile LandingTrace active;

    private final ServerPlayer player;
    private final Connection connection;
    private final Channel channel;
    private final int entityId;
    private final String identity;
    private final String run;
    private final StringBuilder events = new StringBuilder();
    private final StringBuilder wire = new StringBuilder();
    // Only tagged instances sent through the observed player's actual listener qualify.
    private final IdentityHashMap<Packet<?>, Integer> emitted = new IdentityHashMap<>();
    private final IdentityHashMap<Packet<?>, Integer> dispatched = new IdentityHashMap<>();
    private final Packet<?> startFence;
    private final Packet<?> endFence;
    private boolean betweenFences;
    private boolean sealed;
    private int eventOrder;
    private int wireOrder;
    private int motionOrdinal;
    private long moveOrdinal;
    private ServerboundMovePlayerPacket accepted;
    private String floor;
    private DamageSource damage;
    private DamageSource markSource;
    private int pending = -1;

    private LandingTrace(ServerPlayer player, String run) {
        this.player = player;
        this.run = run;
        connection = ((LandingRecipient) player.connection).ptConnection();
        channel = ((LandingChannel) connection).ptChannel();
        entityId = player.getId();
        identity = player.getUUID() + "," + entityId;
        startFence = fence("start", run);
        endFence = fence("end", run);
    }

    private static ClientboundSystemChatPacket fence(String kind, String run) {
        return new ClientboundSystemChatPacket(Component.literal("pt-landing-v1 " + kind + " " + run), false);
    }

    static void start(ServerPlayer player, String run) {
        if (active != null || !run.matches("[A-Za-z0-9_-]+") || player.hurtMarked) {
            throw new IllegalStateException("Landing capture already active, invalid name, or pending hurtMarked");
        }
        var trace = new LandingTrace(player, run);
        trace.event("start," + run + "," + trace.identity + "," + player.level().tickRateManager().tickrate());
        active = trace;
        player.connection.send(trace.startFence);
    }

    static void stop(String run) {
        var trace = active;
        if (trace == null || trace.sealed || !trace.run.equals(run) || trace.player.isRemoved()) {
            throw new IllegalStateException("No matching landing capture for " + run);
        }
        trace.event("end," + run + "," + trace.identity + "," + trace.pending);
        trace.sealed = true;
        trace.player.connection.send(trace.endFence);
    }

    static void cancel() {
        var trace = active;
        if (trace == null) return;
        synchronized (trace) {
            if (active == trace) {
                trace.write(".server-motion.partial.csv", "landing-v1\n" + trace.events + "wire\n" + trace.wire);
                active = null;
            }
        }
    }

    private static LandingTrace observing(Object entity) {
        var trace = active;
        return trace != null && !trace.sealed && trace.player == entity ? trace : null;
    }

    public static void accepted(ServerPlayer player, ServerboundMovePlayerPacket packet, long ordinal) {
        var trace = observing(player);
        if (trace == null) return;
        trace.moveOrdinal = ordinal;
        trace.accepted = packet;
    }

    public static void endAccepted(ServerPlayer player) {
        var trace = observing(player);
        if (trace != null) trace.accepted = null;
    }

    public static void floor(Entity entity, String block) {
        var trace = observing(entity);
        if (trace != null) trace.floor = block;
    }

    public static void damage(Object entity, DamageSource source, float amount) {
        var trace = observing(entity);
        if (trace == null) return;
        trace.damage = source;
        var move = trace.accepted;
        trace.event("damage," + sourceKey(source) + "," + HEX.toHexDigits(Float.floatToRawIntBits(amount))
            + "," + trace.floor + "," + (move == null ? "missing" : trace.moveOrdinal + ","
                + move.isOnGround() + "," + move.horizontalCollision() + "," + move.hasPosition() + ","
                + bits(new Vec3(move.getX(0), move.getY(0), move.getZ(0)))));
    }

    public static void markSource(Object entity, DamageSource source) {
        var trace = observing(entity);
        if (trace != null) trace.markSource = source;
    }

    public static void mark(Entity entity, boolean before) {
        var trace = observing(entity);
        if (trace == null) return;
        var source = trace.markSource;
        boolean sameDamage = source != null && source == trace.damage;
        trace.pending = trace.event("mark," + (source == null ? "missing" : sourceKey(source)) + ","
            + before + "," + entity.hurtMarked + "," + sameDamage + "," + bits(entity.getDeltaMovement()));
        trace.damage = null;
    }

    public static void velocity(Object entity, Vec3 value) {
        var trace = observing(entity);
        if (trace != null && (trace.pending >= 0 || trace.floor != null)) {
            trace.event("write," + bits(value));
        }
    }

    public static void competingMark(Entity entity, String site) {
        var trace = observing(entity);
        if (trace != null) trace.event("competing-mark," + site);
    }

    public static void consume(Entity entity, Packet<?> packet) {
        var trace = observing(entity);
        if (trace == null || !(packet instanceof ClientboundSetEntityMotionPacket motion)) return;
        int order = trace.event("consume," + trace.pending + "," + entity.hurtMarked + ","
            + motion.id() + "," + bits(motion.movement()));
        trace.pending = -1;
        synchronized (trace) { trace.emitted.put(packet, order); }
    }

    public static void dispatch(Connection recipient, Packet<?> packet) {
        var trace = active;
        if (trace == null || trace.connection != recipient) return;
        synchronized (trace) {
            var emission = trace.emitted.remove(packet);
            if (emission == null) return;
            trace.event("dispatch," + emission + "," + trace.identity);
            trace.dispatched.put(packet, emission);
        }
    }

    public static void encoded(Channel recipient, Packet<?> packet, ByteBuf output) {
        var trace = active;
        if (trace == null || trace.channel != recipient) return;
        synchronized (trace) {
            if (packet == trace.startFence) {
                trace.betweenFences = true;
                trace.wire("start," + trace.run + "," + trace.identity);
            } else if (trace.betweenFences && packet == trace.endFence) {
                trace.wire("end," + trace.run + "," + trace.identity);
                trace.betweenFences = false;
                trace.save();
                active = null;
            } else if (trace.betweenFences && packet instanceof ClientboundSetEntityMotionPacket motion
                && motion.id() == trace.entityId) {
                var emission = trace.dispatched.remove(packet);
                int ordinal = ++trace.motionOrdinal;
                try {
                    var payload = output.duplicate();
                    VarInt.read(payload); // packet type ID, already written by the protocol codec
                    String bytes = ByteBufUtil.hexDump(payload);
                    var decoded = ClientboundSetEntityMotionPacket.STREAM_CODEC.decode(payload);
                    trace.wire("motion," + ordinal + "," + trace.identity + "," + decoded.id()
                        + "," + (emission == null ? -1 : emission) + "," + bytes + "," + bits(decoded.movement()));
                } catch (RuntimeException failure) {
                    trace.wire("decode-failure," + ordinal + "," + ByteBufUtil.hexDump(output));
                    LoggerFactory.getLogger("pathtracing").error("Cannot inspect motion bytes for " + trace.run, failure);
                }
            }
        }
    }

    private synchronized int event(String row) {
        int order = eventOrder++;
        events.append(order).append(',').append(player.level().getGameTime()).append(',')
            .append(player.tickCount).append(',').append(row).append('\n');
        return order;
    }

    private void wire(String row) { wire.append(wireOrder++).append(',').append(row).append('\n'); }

    private void save() {
        write(".server-motion.csv", "landing-v1\n" + events + "wire\n" + wire);
    }

    private void write(String suffix, String content) {
        try {
            var directory = Files.createDirectories(Path.of("pathtracing-phase"));
            var partial = directory.resolve(run + suffix + ".tmp");
            Files.writeString(partial, content, StandardOpenOption.CREATE_NEW);
            Files.move(partial, directory.resolve(run + suffix));
        } catch (IOException e) {
            LoggerFactory.getLogger("pathtracing").error("Cannot save server landing trace for " + run, e);
        }
    }

    private static String sourceKey(DamageSource source) {
        return source.typeHolder().unwrapKey().map(key -> key.identifier().toString()).orElse("missing");
    }

    private static String bits(Vec3 vector) {
        return HEX.toHexDigits(Double.doubleToRawLongBits(vector.x)) + ","
            + HEX.toHexDigits(Double.doubleToRawLongBits(vector.y)) + ","
            + HEX.toHexDigits(Double.doubleToRawLongBits(vector.z));
    }
}
