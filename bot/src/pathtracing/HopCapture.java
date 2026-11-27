package pathtracing;

import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardOpenOption;
import java.util.ArrayList;
import java.util.IdentityHashMap;
import java.util.Map;
import java.util.WeakHashMap;
import net.minecraft.network.Connection;
import net.minecraft.network.protocol.Packet;

/** Passive same-host QPC evidence. No modeled tick or latency subtraction. */
public final class HopCapture {
    public static final boolean ENABLED = Boolean.getBoolean("pathtracing.hopCapture");
    private static final ArrayList<String> rows = new ArrayList<>();
    private static final Map<Connection,long[]> sequences = new WeakHashMap<>();
    private static final Map<Packet<?>,String> received = new IdentityHashMap<>();
    private static long serverTick = -1;
    public static void issuance() {
        if (!ENABLED) return;
        long[] sample=NativeChannel.clock(); hop("issuance",0,sample[0],sample[0],new long[]{sample[1]});
    }
    public static void mark(String kind) {
        if (ENABLED) hop(kind,0,0,NativeChannel.clock()[0],new long[0]);
    }
    static { if (ENABLED) Runtime.getRuntime().addShutdownHook(new Thread(HopCapture::flush,"pathtracing-hop-flush")); }
    public static synchronized void hop(String kind,long version,long issue,long at,long[] data) {
        if (!ENABLED) return;
        rows.add(kind + "\t" + version + "\t" + issue + "\t" + at + "\t" + serverTick + "\t" + java.util.Arrays.toString(data));
    }
    public static synchronized void packet(Connection connection,Packet<?> packet,boolean sending) {
        if (!ENABLED) return;
        long[] sequence = sequences.computeIfAbsent(connection,ignored -> new long[2]);
        long n = ++sequence[sending?0:1];
        String type = packet.getClass().getName();
        // Sequence counts EVERY packet on this connection. Required real input
        // packets are identified by type, never by a separately received marker.
        var channel = ((pathtracing.mixin.HopConnectionAccess)connection).pathtracing$channel();
        String local = address(channel.localAddress()), remote = address(channel.remoteAddress());
        String pair = local.compareTo(remote)<0 ? local+"|"+remote : remote+"|"+local;
        String tag = pair + ":" + n + ":" + type;
        if (!sending && (packet instanceof net.minecraft.network.protocol.game.ServerboundMovePlayerPacket || type.contains("ServerboundPlayer") || type.contains("ServerboundUseItem") || type.contains("ServerboundSetCarriedItem"))) received.put(packet,tag);
        hop((sending?"packet-send:":"packet-receive:")+tag,0,0,NativeChannel.clock()[0],new long[0]);
    }
    public static synchronized void applied(Packet<?> packet) {
        if (!ENABLED) return;
        String tag = received.remove(packet);
        hop("packet-apply:" + (tag == null ? "MISSING_RECEIPT:"+packet.getClass().getSimpleName() : tag),0,0,NativeChannel.clock()[0],new long[0]);
    }
    public static synchronized void serverTick(long tick) {
        if (!ENABLED) return;
        serverTick = tick;
        long[] clock = NativeChannel.clock(); hop("server-tick-start",0,0,clock[0],new long[]{clock[1]});
    }
    public static void input(long sequence) {
        if (ENABLED) hop("client-input-applied",0,0,NativeChannel.clock()[0],new long[]{sequence});
    }
    public static void resources(net.minecraft.server.MinecraftServer server) {
        if (!ENABLED) return;
        for (var p : server.getPlayerList().getPlayers()) {
            var food = (pathtracing.mixin.FoodStateAccess)p.getFoodData();
            var living = (pathtracing.mixin.LivingStateAccess)p;
            hop("server-resources:"+p.getUUID()+":"+p.level().dimension().identifier(),0,0,NativeChannel.clock()[0],new long[]{
                p.getId(),server.getTickCount(),p.tickCount,p.level().getDifficulty().getId(),
                Integer.toUnsignedLong(Float.floatToRawIntBits(p.getHealth())),p.getFoodData().getFoodLevel(),
                Integer.toUnsignedLong(Float.floatToRawIntBits(p.getFoodData().getSaturationLevel())),
                Integer.toUnsignedLong(Float.floatToRawIntBits(food.pathtracing$exhaustion())),food.pathtracing$foodTickTimer(),
                MinecraftApi.damageCooldown(p),Integer.toUnsignedLong(Float.floatToRawIntBits(living.pathtracing$lastRawHit()))});
        }
    }
    public static synchronized void flush() {
        if (!ENABLED || rows.isEmpty()) return;
        String directory = System.getenv("PATHTRACING_CAPTURE");
        if (directory == null || directory.isBlank()) throw new IllegalStateException("PATHTRACING_CAPTURE required for hop capture");
        try {
            Path output = Path.of(directory); Files.createDirectories(output);
            String role = System.getProperty("pathtracing.role","server");
            Files.write(output.resolve("hops-"+role+"-"+ProcessHandle.current().pid()+".tsv"),rows,StandardCharsets.UTF_8,StandardOpenOption.CREATE,StandardOpenOption.APPEND);
            rows.clear();
        } catch (java.io.IOException e) { throw new java.io.UncheckedIOException(e); }
    }
    private static String address(java.net.SocketAddress value) {
        if (value instanceof java.net.InetSocketAddress inet) return inet.getAddress().getHostAddress()+":"+inet.getPort();
        return value.toString();
    }
    private HopCapture() { }
}
