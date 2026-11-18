package pathtracing.probe;

import com.mojang.authlib.GameProfile;
import com.mojang.brigadier.arguments.IntegerArgumentType;
import com.mojang.brigadier.arguments.StringArgumentType;
import io.netty.channel.ChannelFutureListener;
import java.io.IOException;
import java.lang.management.ManagementFactory;
import java.net.InetAddress;
import java.net.InetSocketAddress;
import java.net.SocketAddress;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Locale;
import java.util.UUID;
import net.fabricmc.api.DedicatedServerModInitializer;
import net.fabricmc.fabric.api.command.v2.CommandRegistrationCallback;
import net.fabricmc.fabric.api.event.lifecycle.v1.ServerTickEvents;
import net.minecraft.commands.CommandSourceStack;
import net.minecraft.commands.Commands;
import net.minecraft.core.SectionPos;
import net.minecraft.network.Connection;
import net.minecraft.network.PacketListener;
import net.minecraft.network.ProtocolInfo;
import net.minecraft.network.chat.Component;
import net.minecraft.network.protocol.Packet;
import net.minecraft.network.protocol.PacketFlow;
import net.minecraft.network.protocol.game.ClientboundPlayerPositionPacket;
import net.minecraft.network.protocol.game.ServerboundAcceptTeleportationPacket;
import net.minecraft.network.protocol.game.ServerboundClientTickEndPacket;
import net.minecraft.network.protocol.game.ServerboundMovePlayerPacket;
import net.minecraft.network.protocol.game.ServerboundPlayerInputPacket;
import net.minecraft.network.protocol.game.ServerboundPlayerLoadedPacket;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ClientInformation;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.server.network.CommonListenerCookie;
import net.minecraft.world.entity.ai.attributes.AttributeInstance;
import net.minecraft.world.entity.ai.attributes.Attributes;
import net.minecraft.world.entity.player.Input;
import net.minecraft.world.level.GameType;
import net.minecraft.world.phys.Vec3;

/** Disposable server cost probe for packet, lazy, and virtual bots on 26.2. */
public final class ServerProbe implements DedicatedServerModInitializer {
    enum Mode { PACKET, LAZY, VIRTUAL }

    static ServerProbe instance;
    static long tickStartNs;

    private static final Input FORWARD_INPUT = new Input(true, false, false, false, false, false, false);
    private static final Input COAST_INPUT = Input.EMPTY;
    private static final double GROUND_Y = -60.0;
    private static final int SPAWN_BATCH = 25;

    final ArrayList<ProbeBot> bots = new ArrayList<>();
    final long[] nanos = new long[600];
    final long[] walls = new long[600];
    final long[] heaps = new long[600];
    final Path output = Path.of(System.getProperty("probe.output", "probe-out"));

    MinecraftServer server;
    Mode mode = Mode.PACKET;
    String layout = "dense";
    String run = "idle";
    String status = "idle";
    int requested;
    int phase = -1;
    int sample;
    long measureStartMs;
    long totalTeleports;

    // Virtual mode primitive arrays (zero ServerPlayer objects)
    double[] vx = new double[0];
    double[] vy = new double[0];
    double[] vzPos = new double[0];
    double[] vVelZ = new double[0];
    float[] vYaw = new float[0];
    int[] vTickPhase = new int[0];

    @Override
    public void onInitializeServer() {
        instance = this;
        CommandRegistrationCallback.EVENT.register((dispatcher, registryAccess, environment) ->
            dispatcher.register(Commands.literal("probe")
                .requires(Commands.hasPermission(Commands.LEVEL_OWNERS))
                .then(Commands.literal("set")
                    .then(Commands.argument("mode", StringArgumentType.word())
                        .then(Commands.argument("n", IntegerArgumentType.integer(0, 10000))
                            .then(Commands.argument("layout", StringArgumentType.word())
                                .executes(ctx -> configure(
                                    ctx.getSource(),
                                    StringArgumentType.getString(ctx, "mode"),
                                    IntegerArgumentType.getInteger(ctx, "n"),
                                    StringArgumentType.getString(ctx, "layout")
                                ))))))
                .then(Commands.literal("measure")
                    .then(Commands.argument("run", StringArgumentType.word())
                        .executes(ctx -> startMeasure(
                            ctx.getSource(),
                            StringArgumentType.getString(ctx, "run")
                        ))))
                .then(Commands.literal("status")
                    .executes(ctx -> reply(
                        ctx.getSource(),
                        status + " " + activeCount() + "/" + requested + " " + phase
                    ))))
        );

        ServerTickEvents.START_SERVER_TICK.register(this::onServerTickStart);
    }

    private int activeCount() {
        return mode == Mode.VIRTUAL ? vTickPhase.length : bots.size();
    }

    private static int reply(CommandSourceStack source, String text) {
        source.sendSuccess(() -> Component.literal(text), false);
        return 1;
    }

    private int configure(CommandSourceStack source, String modeArg, int n, String layoutArg) {
        Mode parsedMode = switch (modeArg) {
            case "packet" -> Mode.PACKET;
            case "lazy" -> Mode.LAZY;
            case "virtual" -> Mode.VIRTUAL;
            default -> throw new IllegalArgumentException("Invalid mode: " + modeArg);
        };
        if (!layoutArg.equals("dense") && !layoutArg.equals("dispersed")) {
            throw new IllegalArgumentException("Invalid layout: " + layoutArg);
        }
        if (parsedMode != Mode.VIRTUAL && n > 1000) {
            throw new IllegalArgumentException("N > 1000 only supported for virtual mode");
        }
        server = source.getServer();
        for (ProbeBot b : bots) {
            server.getPlayerList().remove(b);
        }
        bots.clear();
        clearVirtual();

        mode = parsedMode;
        layout = layoutArg;
        requested = n;
        phase = -1;
        sample = 0;
        totalTeleports = 0;

        if (mode == Mode.VIRTUAL) {
            allocateVirtual(n);
            status = "ready";
        } else if (n == 0) {
            status = "ready";
        } else {
            status = "spawning";
        }
        return reply(source, status);
    }

    private int startMeasure(CommandSourceStack source, String runArg) {
        if (!status.equals("ready") && !status.equals("done")) {
            return reply(source, status);
        }
        run = runArg;
        phase = 0;
        sample = 0;
        totalTeleports = 0;
        status = "warming";
        return reply(source, status);
    }

    private void onServerTickStart(MinecraftServer srv) {
        server = srv;
        if (status.equals("spawning")) {
            int spawnEnd = Math.min(requested, bots.size() + SPAWN_BATCH);
            while (bots.size() < spawnEnd) {
                try {
                    spawnBot(bots.size());
                } catch (Exception ex) {
                    status = "refused:" + ex;
                    saveFailure(status);
                    return;
                }
            }
            if (bots.size() >= requested) {
                status = "loading";
            }
        }
        if (status.equals("loading")) {
            if (allBotChunksLoaded()) {
                status = "ready";
            }
            return;
        }
        if (status.equals("ready") || status.equals("warming") || status.equals("measuring") || status.equals("done")) {
            stepBots();
        }
    }

    private boolean allBotChunksLoaded() {
        ServerLevel level = server.overworld();
        var cache = level.getChunkSource();
        for (int i = 0; i < bots.size(); i++) {
            ProbeBot b = bots.get(i);
            int cx = SectionPos.blockToSectionCoord(b.botX);
            int cz = SectionPos.blockToSectionCoord(b.botZ);
            for (int dx = -1; dx <= 1; dx++) {
                for (int dz = -1; dz <= 1; dz++) {
                    if (cache.getChunkNow(cx + dx, cz + dz) == null) {
                        return false;
                    }
                }
            }
        }
        return true;
    }

    private double spawnX(int i) {
        return layout.equals("dense") ? (i % 32) * 1.0 + 0.5 : (i % 8) * 512.0 + 0.5;
    }

    private double spawnZ(int i) {
        return layout.equals("dense") ? (i / 32) * 1.0 + 0.5 : (i / 8) * 512.0 + 0.5;
    }

    private void clearVirtual() {
        vx = new double[0];
        vy = new double[0];
        vzPos = new double[0];
        vVelZ = new double[0];
        vYaw = new float[0];
        vTickPhase = new int[0];
    }

    private void allocateVirtual(int n) {
        vx = new double[n];
        vy = new double[n];
        vzPos = new double[n];
        vVelZ = new double[n];
        vYaw = new float[n];
        vTickPhase = new int[n];
        for (int i = 0; i < n; i++) {
            vx[i] = spawnX(i);
            vy[i] = GROUND_Y;
            vzPos[i] = spawnZ(i);
            vVelZ[i] = 0.0;
            vYaw[i] = 0.0f;
            vTickPhase[i] = 0;
        }
    }

    private void spawnBot(int i) {
        String name = String.format(Locale.ROOT, "Probe%04d", i);
        UUID uuid = UUID.nameUUIDFromBytes(("OfflinePlayer:" + name).getBytes(StandardCharsets.UTF_8));
        GameProfile profile = new GameProfile(uuid, name);
        SinkConnection sink = new SinkConnection();
        ProbeBot bot = new ProbeBot(server, server.overworld(), profile, sink, mode == Mode.LAZY);

        server.getPlayerList().placeNewPlayer(sink, bot, CommonListenerCookie.createInitial(profile, false));
        bot.setGameMode(GameType.SURVIVAL);

        AttributeInstance receiveRange = bot.getAttribute(Attributes.WAYPOINT_RECEIVE_RANGE);
        if (receiveRange != null) {
            receiveRange.setBaseValue(0.0);
        }

        double x = spawnX(i);
        double z = spawnZ(i);
        bot.botX = x;
        bot.botZ = z;
        bot.botVelZ = 0.0;
        bot.botYaw = 0.0f;
        bot.tickPhase = 0;
        bot.lastSecX = SectionPos.blockToSectionCoord(x);
        bot.lastSecZ = SectionPos.blockToSectionCoord(z);

        bot.connection.teleport(x, GROUND_Y, z, 0.0f, 0.0f);
        if (sink.pendingTpId >= 0) {
            bot.connection.handleAcceptTeleportPacket(new ServerboundAcceptTeleportationPacket(sink.pendingTpId));
            sink.pendingTpId = -1;
            sink.pendingTpPos = null;
        }
        bot.connection.handleAcceptPlayerLoad(new ServerboundPlayerLoadedPacket());
        bot.snapTo(x, GROUND_Y, z, 0.0f, 0.0f);
        bot.setDeltaMovement(Vec3.ZERO);
        bot.setOnGround(true);
        bot.connection.resetPosition();
        bot.level().getChunkSource().move(bot);
        bots.add(bot);
    }

    private void stepBots() {
        if (mode == Mode.VIRTUAL) {
            int n = vTickPhase.length;
            for (int i = 0; i < n; i++) {
                int t = vTickPhase[i];
                double vz = vVelZ[i];
                if (t < 40) {
                    vz += 0.098f;
                    vYaw[i] = 0.0f;
                } else if (t < 50) {
                    vYaw[i] = 0.0f;
                } else if (t < 90) {
                    vz -= 0.098f;
                    vYaw[i] = 180.0f;
                } else {
                    vYaw[i] = 180.0f;
                }
                vzPos[i] += vz;
                vz *= 0.546f;
                if (Math.abs(vz) < 0.003) {
                    vz = 0.0;
                }
                vVelZ[i] = vz;
                vTickPhase[i] = (t + 1 == 100) ? 0 : t + 1;
            }
            return;
        }

        boolean countTeleports = status.equals("measuring");
        if (mode == Mode.LAZY) {
            ServerLevel level = server.overworld();
            var chunkCache = level.getChunkSource();
            for (int i = 0; i < bots.size(); i++) {
                ProbeBot b = bots.get(i);
                b.stepTrajectory();
                b.absSnapTo(b.botX, GROUND_Y, b.botZ, b.botYaw, 0.0f);
                int sx = SectionPos.blockToSectionCoord(b.botX);
                int sz = SectionPos.blockToSectionCoord(b.botZ);
                if (sx != b.lastSecX || sz != b.lastSecZ) {
                    b.lastSecX = sx;
                    b.lastSecZ = sz;
                    chunkCache.move(b);
                }
            }
            return;
        }

        // PACKET mode: speak vanilla serverbound packets at the vanilla packet phase before level tick.
        for (int i = 0; i < bots.size(); i++) {
            ProbeBot b = bots.get(i);
            SinkConnection sink = b.sink;
            if (sink.pendingTpId >= 0) {
                int id = sink.pendingTpId;
                Vec3 pos = sink.pendingTpPos;
                sink.pendingTpId = -1;
                sink.pendingTpPos = null;
                b.connection.handleAcceptTeleportPacket(new ServerboundAcceptTeleportationPacket(id));
                if (pos != null) {
                    b.botX = pos.x;
                    b.botZ = pos.z;
                    b.botVelZ = 0.0;
                }
                if (countTeleports) {
                    totalTeleports++;
                }
            }
            boolean forward = b.stepTrajectory();
            b.connection.handlePlayerInput(new ServerboundPlayerInputPacket(forward ? FORWARD_INPUT : COAST_INPUT));
            b.connection.handleMovePlayer(
                new ServerboundMovePlayerPacket.PosRot(b.botX, GROUND_Y, b.botZ, b.botYaw, 0.0f, true, false)
            );
            b.connection.handleClientTickEnd(ServerboundClientTickEndPacket.INSTANCE);
            b.connection.tick();
        }
    }

    public static void tickStart() {
        tickStartNs = System.nanoTime();
    }

    public static void tickEnd() {
        long endNs = System.nanoTime();
        if (instance != null) {
            instance.finishTick(endNs, endNs - tickStartNs);
        }
    }

    private void finishTick(long wallNs, long elapsedNs) {
        if (phase < 0) {
            return;
        }
        if (++phase <= 200) {
            return;
        }
        status = "measuring";
        if (sample == 0) {
            measureStartMs = System.currentTimeMillis();
        }
        nanos[sample] = elapsedNs;
        walls[sample] = wallNs;
        heaps[sample] = ManagementFactory.getMemoryMXBean().getHeapMemoryUsage().getUsed();
        if (++sample == 600) {
            phase = -1;
            writeSampleOutputs();
        }
    }

    private void writeSampleOutputs() {
        try {
            Files.createDirectories(output);
            try (var w = Files.newBufferedWriter(output.resolve(run + ".csv"), StandardCharsets.UTF_8)) {
                w.write("tick,ns,wall_ns,heap_bytes\n");
                for (int i = 0; i < 600; i++) {
                    w.write(i + "," + nanos[i] + "," + walls[i] + "," + heaps[i] + "\n");
                }
            }
            Files.writeString(
                output.resolve(run + ".window"),
                measureStartMs + "," + System.currentTimeMillis(),
                StandardCharsets.UTF_8
            );
            int loadedChunks = server.overworld().getChunkSource().getLoadedChunksCount();
            String stats = String.format(
                Locale.ROOT,
                "{\"mode\":\"%s\",\"layout\":\"%s\",\"botCount\":%d,\"loadedChunks\":%d,\"teleports\":%d}%n",
                mode.name().toLowerCase(Locale.ROOT),
                layout,
                activeCount(),
                loadedChunks,
                totalTeleports
            );
            Files.writeString(output.resolve(run + ".stats.json"), stats, StandardCharsets.UTF_8);
            status = "done";
        } catch (IOException ex) {
            status = "refused:" + ex;
            saveFailure(status);
        }
    }

    private void saveFailure(String message) {
        try {
            Files.createDirectories(output);
            Files.writeString(output.resolve("failure.txt"), message, StandardCharsets.UTF_8);
        } catch (IOException ignored) {
        }
    }

    public static final class ProbeBot extends ServerPlayer {
        final SinkConnection sink;
        final boolean lazy;
        double botX;
        double botZ;
        double botVelZ;
        float botYaw;
        int tickPhase;
        int lastSecX;
        int lastSecZ;

        ProbeBot(MinecraftServer s, ServerLevel l, GameProfile p, SinkConnection sink, boolean lazy) {
            super(s, l, p, ClientInformation.createDefault());
            this.sink = sink;
            this.lazy = lazy;
        }

        boolean stepTrajectory() {
            int t = tickPhase;
            boolean forward = false;
            if (t < 40) {
                botVelZ += 0.098f;
                botYaw = 0.0f;
                forward = true;
            } else if (t < 50) {
                botYaw = 0.0f;
            } else if (t < 90) {
                botVelZ -= 0.098f;
                botYaw = 180.0f;
                forward = true;
            } else {
                botYaw = 180.0f;
            }
            botZ += botVelZ;
            botVelZ *= 0.546f;
            if (Math.abs(botVelZ) < 0.003) {
                botVelZ = 0.0;
            }
            tickPhase = (t + 1 == 100) ? 0 : t + 1;
            return forward;
        }

        @Override
        public void tick() {
            if (!lazy) {
                super.tick();
            }
        }
    }

    static final class SinkConnection extends Connection {
        int pendingTpId = -1;
        Vec3 pendingTpPos;

        SinkConnection() {
            super(PacketFlow.SERVERBOUND);
        }

        private void capture(Packet<?> packet) {
            if (packet instanceof ClientboundPlayerPositionPacket tp) {
                pendingTpId = tp.id();
                pendingTpPos = tp.change().position();
            }
        }

        @Override
        public boolean isConnected() {
            return true;
        }

        @Override
        public boolean isMemoryConnection() {
            return true;
        }

        @Override
        public SocketAddress getRemoteAddress() {
            return new InetSocketAddress(InetAddress.getLoopbackAddress(), 0);
        }

        @Override
        public void send(Packet<?> p) {
            capture(p);
        }

        @Override
        public void send(Packet<?> p, ChannelFutureListener l) {
            capture(p);
        }

        @Override
        public void send(Packet<?> p, ChannelFutureListener l, boolean flush) {
            capture(p);
        }

        @Override
        public <T extends PacketListener> void setupInboundProtocol(ProtocolInfo<T> p, T l) {
        }

        @Override
        public void setupOutboundProtocol(ProtocolInfo<?> p) {
        }

        @Override
        public void flushChannel() {
        }
    }
}
