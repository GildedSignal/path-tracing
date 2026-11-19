package pathtracing.server;

import com.mojang.authlib.GameProfile;
import com.mojang.brigadier.CommandDispatcher;
import com.mojang.brigadier.arguments.FloatArgumentType;
import com.mojang.brigadier.arguments.IntegerArgumentType;
import com.mojang.brigadier.arguments.StringArgumentType;
import java.nio.ByteBuffer;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.HashMap;
import java.util.HashSet;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.Set;
import java.util.UUID;
import net.fabricmc.fabric.api.networking.v1.ServerPlayNetworking;
import net.minecraft.commands.CommandSourceStack;
import net.minecraft.commands.Commands;
import net.minecraft.commands.arguments.coordinates.BlockPosArgument;
import net.minecraft.core.BlockPos;
import net.minecraft.network.chat.Component;
import net.minecraft.network.protocol.game.ServerboundAcceptTeleportationPacket;
import net.minecraft.network.protocol.game.ServerboundClientTickEndPacket;
import net.minecraft.network.protocol.game.ServerboundMovePlayerPacket;
import net.minecraft.network.protocol.game.ServerboundPlayerCommandPacket;
import net.minecraft.network.protocol.game.ServerboundPlayerInputPacket;
import net.minecraft.network.protocol.game.ServerboundPlayerLoadedPacket;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.server.network.CommonListenerCookie;
import net.minecraft.world.entity.ai.attributes.AttributeInstance;
import net.minecraft.world.entity.ai.attributes.Attributes;
import net.minecraft.world.entity.player.Input;
import net.minecraft.world.level.GameType;
import net.minecraft.world.phys.Vec3;
import pathtracing.server.mixin.FoodDataAccessor;
import pathtracing.server.mixin.LivingEntityAccessor;

/** Orchestrates Virtual, Visual, and Materialized SwarmBots alongside Mojang 26.2 ticks. */
public final class SwarmManager {
    enum Formation { GRID, POINT }
    enum StickMode { ALL, ONE }

    private record PendingGoal(int botId, int gx, int gy, int gz, long issueQpc, String requester) {}

    private record PendingSpawnBatch(
        int count,
        int startBotId,
        int startIndex,
        int mode,
        int cols,
        double baseX,
        double baseY,
        double baseZ,
        double spacingX,
        double spacingZ
    ) {}

    private record PendingBatchGoal(int gx, int gy, int gz, boolean grid, long issueQpc, String requester) {}

    private static final class SyntheticObserver {
        final int id;
        double x, y, z;
        float viewDist;

        SyntheticObserver(int id, double x, double y, double z, float viewDist) {
            this.id = id;
            this.x = x;
            this.y = y;
            this.z = z;
            this.viewDist = viewDist;
        }
    }

    private final List<SwarmBot> matBots = new ArrayList<>();
    private final Map<Integer, SwarmBot> matById = new HashMap<>();
    private final List<PendingGoal> pendingGoals = new ArrayList<>();
    private final Map<String, SyntheticObserver> syntheticObservers = new LinkedHashMap<>();
    private final VisualProjector visualProjector = new VisualProjector();
    private final ShmBridge.PacketWriter pw = new ShmBridge.PacketWriter();
    private final WorldBridge worldBridge;

    private final long[] tickNanos = new long[600];
    private int tickSampleHead = 0;
    private int tickSampleCount = 0;
    private final long[] flightNanos = new long[4096];
    private int flightSampleCount = 0;

    private final long[] mindNanos = new long[600];
    private int mindSampleHead = 0;
    private int mindSampleCount = 0;
    private final long[] mindFlightNanos = new long[4096];
    private int mindFlightSampleCount = 0;

    private boolean profilingFlight = false;
    private long currentTickStartNs = 0L;

    private MinecraftServer server;
    private ShmBridge bridge;
    private int nextBotId = 1;
    private int nextObserverId = 1;
    private long serverTickSeq = 0L;
    private long totalTeleports = 0L;
    private int reportedSections = 0;
    private int reportedDiskSections = 0;

    private int swarmMode = ShmBridge.MODE_MATERIALIZED;
    private int swarmTotalCount = 0;
    private int engineTotalBots = 0;
    private int engineActiveBots = 0;
    private int engineVirtCount = 0;
    private int engineVisCount = 0;
    private int engineMatCount = 0;
    private int engineArrivedCount = 0;
    private int engineSettledCount = 0;
    private int engineExactTCount = 0;
    private int engineMinActT = 0;
    private int engineMaxActT = 0;
    private int enginePromotions = 0;
    private int engineDemotions = 0;
    private int engineHookedPromotions = 0;

    private int summaryTotal = 0;
    private int summaryReady = 0;
    private int summaryAlready = 0;
    private int summaryJoints = 0;
    private int summaryRefused = 0;
    private long summaryMinSolveNs = 0L;
    private long summaryMedSolveNs = 0L;
    private long summaryMaxSolveNs = 0L;
    private int summaryMinPredT = 0;
    private int summaryMaxPredT = 0;
    private String summarySampleMsg = "";
    private boolean hasBatchSummary = false;

    private int cfgMaxVisual = 128;
    private int cfgMaxMaterialized = 8;
    private float cfgMaterializeRadius = 5.0f;
    private int cfgCanaryCount = 8;
    private int cfgCanaryPeriod = 100;
    private boolean configDirty = true;

    private static volatile boolean suppressJoinChat = false;
    private final Set<UUID> knownModdedHumans = new HashSet<>();
    private boolean renderStreamEnabled = false;
    private boolean lastSentModdedOnly = false;
    private long renderFramesSent = 0L;

    private boolean pendingClear = false;
    private boolean pendingStopAll = false;
    private PendingSpawnBatch pendingSpawnBatch;
    private PendingBatchGoal pendingBatchGoal;
    private int lastObserverCountSent = 0;

    private Formation formation = Formation.GRID;
    private StickMode stickMode = StickMode.ALL;
    private int formationAnchorX = 0;
    private int formationAnchorY = -60;
    private int formationAnchorZ = 0;
    private int formationCols = 1;
    private float formationSpacing = 1.5f;

    private boolean paradeActive = false;
    private int paradeOriginX = 0;
    private int paradeOriginY = -60;
    private int paradeOriginZ = 0;
    private int paradeDist = 60;
    private int paradeLeg = 0;
    private int paradeWaitTicks = 0;

    private UUID followPlayerUuid = null;
    private int followLastX = 0;
    private int followLastZ = 0;
    private boolean followHasLast = false;
    private int followCooldownTicks = 0;

    private String lastBatchRequester;
    private int expectedBatchReplies = 0;
    private int receivedBatchReplies = 0;
    private long batchMaxSolveNs = 0L;
    private int batchReadyCount = 0;
    private int batchRefusedCount = 0;
    private String batchSampleRefusal = "";

    public SwarmManager(WorldBridge worldBridge) {
        this.worldBridge = worldBridge;
    }

    public static boolean isSuppressingJoinChat() {
        return suppressJoinChat;
    }

    public static String botName(int botIndex) {
        return botIndex == 0 ? "PathBot" : String.format(Locale.ROOT, "Swarm%04d", botIndex);
    }

    public void attach(MinecraftServer srv, ShmBridge shm) {
        this.server = srv;
        this.bridge = shm;
    }

    public void registerCommands(CommandDispatcher<CommandSourceStack> dispatcher) {
        dispatcher.register(Commands.literal("swarm")
            .requires(Commands.hasPermission(Commands.LEVEL_OWNERS))
            .then(Commands.literal("spawn")
                .then(Commands.argument("count", IntegerArgumentType.integer(1, 10000))
                    .executes(ctx -> {
                        int count = IntegerArgumentType.getInteger(ctx, "count");
                        return cmdSpawn(ctx.getSource(), count, 1.5f, defaultModeName(count), null, -1);
                    })
                    .then(Commands.argument("spacing", FloatArgumentType.floatArg(0.5f, 64.0f))
                        .executes(ctx -> {
                            int count = IntegerArgumentType.getInteger(ctx, "count");
                            return cmdSpawn(
                                ctx.getSource(),
                                count,
                                FloatArgumentType.getFloat(ctx, "spacing"),
                                defaultModeName(count),
                                null,
                                -1
                            );
                        })
                        .then(Commands.argument("mode", StringArgumentType.word())
                            .executes(ctx -> cmdSpawn(
                                ctx.getSource(),
                                IntegerArgumentType.getInteger(ctx, "count"),
                                FloatArgumentType.getFloat(ctx, "spacing"),
                                StringArgumentType.getString(ctx, "mode"),
                                null,
                                -1
                            ))
                            .then(Commands.argument("pos", BlockPosArgument.blockPos())
                                .executes(ctx -> cmdSpawn(
                                    ctx.getSource(),
                                    IntegerArgumentType.getInteger(ctx, "count"),
                                    FloatArgumentType.getFloat(ctx, "spacing"),
                                    StringArgumentType.getString(ctx, "mode"),
                                    BlockPosArgument.getBlockPos(ctx, "pos"),
                                    -1
                                ))
                                .then(Commands.argument("cols", IntegerArgumentType.integer(1, 10000))
                                    .executes(ctx -> cmdSpawn(
                                        ctx.getSource(),
                                        IntegerArgumentType.getInteger(ctx, "count"),
                                        FloatArgumentType.getFloat(ctx, "spacing"),
                                        StringArgumentType.getString(ctx, "mode"),
                                        BlockPosArgument.getBlockPos(ctx, "pos"),
                                        IntegerArgumentType.getInteger(ctx, "cols")
                                    ))))))))
            .then(Commands.literal("goto")
                .then(Commands.argument("pos", BlockPosArgument.blockPos())
                    .executes(ctx -> cmdGoto(ctx.getSource(), BlockPosArgument.getBlockPos(ctx, "pos")))))
            .then(Commands.literal("parade")
                .executes(ctx -> cmdParade(ctx.getSource(), 60))
                .then(Commands.literal("off")
                    .executes(ctx -> cmdParadeOff(ctx.getSource())))
                .then(Commands.argument("distance", IntegerArgumentType.integer(10, 500))
                    .executes(ctx -> cmdParade(ctx.getSource(), IntegerArgumentType.getInteger(ctx, "distance")))))
            .then(Commands.literal("come")
                .executes(ctx -> cmdCome(ctx.getSource())))
            .then(Commands.literal("follow")
                .executes(ctx -> cmdFollow(ctx.getSource(), "on"))
                .then(Commands.argument("mode", StringArgumentType.word())
                    .executes(ctx -> cmdFollow(ctx.getSource(), StringArgumentType.getString(ctx, "mode")))))
            .then(Commands.literal("stop")
                .executes(ctx -> cmdStop(ctx.getSource())))
            .then(Commands.literal("remove")
                .executes(ctx -> cmdRemove(ctx.getSource())))
            .then(Commands.literal("formation")
                .then(Commands.argument("kind", StringArgumentType.word())
                    .executes(ctx -> cmdFormation(ctx.getSource(), StringArgumentType.getString(ctx, "kind")))))
            .then(Commands.literal("stick")
                .then(Commands.argument("mode", StringArgumentType.word())
                    .executes(ctx -> cmdStick(ctx.getSource(), StringArgumentType.getString(ctx, "mode")))))
            .then(Commands.literal("config")
                .then(Commands.argument("maxVisual", IntegerArgumentType.integer(0, 10000))
                    .then(Commands.argument("maxMaterialized", IntegerArgumentType.integer(0, 128))
                        .then(Commands.argument("materializeRadius", FloatArgumentType.floatArg(0.0f, 512.0f))
                            .then(Commands.argument("canaryCount", IntegerArgumentType.integer(0, 128))
                                .then(Commands.argument("canaryPeriod", IntegerArgumentType.integer(1, 100000))
                                    .executes(ctx -> cmdConfig(
                                        ctx.getSource(),
                                        IntegerArgumentType.getInteger(ctx, "maxVisual"),
                                        IntegerArgumentType.getInteger(ctx, "maxMaterialized"),
                                        FloatArgumentType.getFloat(ctx, "materializeRadius"),
                                        IntegerArgumentType.getInteger(ctx, "canaryCount"),
                                        IntegerArgumentType.getInteger(ctx, "canaryPeriod")
                                    ))))))))
            .then(Commands.literal("observer")
                .then(Commands.literal("add")
                    .then(Commands.argument("name", StringArgumentType.word())
                        .then(Commands.argument("pos", BlockPosArgument.blockPos())
                            .executes(ctx -> cmdObserverAdd(
                                ctx.getSource(),
                                StringArgumentType.getString(ctx, "name"),
                                BlockPosArgument.getBlockPos(ctx, "pos"),
                                64.0f
                            ))
                            .then(Commands.argument("viewDist", FloatArgumentType.floatArg(1.0f, 512.0f))
                                .executes(ctx -> cmdObserverAdd(
                                    ctx.getSource(),
                                    StringArgumentType.getString(ctx, "name"),
                                    BlockPosArgument.getBlockPos(ctx, "pos"),
                                    FloatArgumentType.getFloat(ctx, "viewDist")
                                ))))))
                .then(Commands.literal("move")
                    .then(Commands.argument("name", StringArgumentType.word())
                        .then(Commands.argument("pos", BlockPosArgument.blockPos())
                            .executes(ctx -> cmdObserverMove(
                                ctx.getSource(),
                                StringArgumentType.getString(ctx, "name"),
                                BlockPosArgument.getBlockPos(ctx, "pos")
                            )))))
                .then(Commands.literal("remove")
                    .then(Commands.argument("name", StringArgumentType.word())
                        .executes(ctx -> cmdObserverRemove(
                            ctx.getSource(),
                            StringArgumentType.getString(ctx, "name")
                        ))))
                .then(Commands.literal("clear")
                    .executes(ctx -> cmdObserverClear(ctx.getSource()))))
            .then(Commands.literal("status")
                .executes(ctx -> cmdStatus(ctx.getSource())))
            .then(Commands.literal("profile")
                .then(Commands.literal("reset")
                    .executes(ctx -> cmdProfileReset(ctx.getSource()))))
        );
    }

    private static String defaultModeName(int count) {
        return count <= 50 ? "materialized" : "auto";
    }

    private static int parseMode(String modeArg) {
        return switch (modeArg.toLowerCase(Locale.ROOT)) {
            case "virtual" -> ShmBridge.MODE_VIRTUAL;
            case "auto" -> ShmBridge.MODE_AUTO;
            case "materialized" -> ShmBridge.MODE_MATERIALIZED;
            default -> throw new IllegalArgumentException("Expected virtual, auto, or materialized");
        };
    }

    private int reply(CommandSourceStack source, String text) {
        source.sendSuccess(() -> Component.literal(text), false);
        return 1;
    }

    private void notifyPlayer(String name, String text) {
        if (server == null || name == null) {
            return;
        }
        ServerPlayer p = server.getPlayerList().getPlayerByName(name);
        if (p != null) {
            p.sendSystemMessage(Component.literal(text));
        }
    }

    private int cmdSpawn(
        CommandSourceStack source,
        int count,
        float spacing,
        String modeArg,
        BlockPos explicitPos,
        int explicitCols
    ) {
        this.server = source.getServer();
        int mode = parseMode(modeArg);
        if (mode == ShmBridge.MODE_MATERIALIZED && count > 64) {
            throw new IllegalArgumentException("Materialized mode limited to <= 64 bots; use auto or virtual");
        }

        clearAllState(true);

        Vec3 basePos = explicitPos != null
            ? new Vec3(explicitPos.getX() + 0.5, explicitPos.getY(), explicitPos.getZ() + 0.5)
            : (source.getPlayer() != null ? source.getPosition() : new Vec3(0.5, -60.0, 0.5));
        double baseX = Math.floor(basePos.x) + 0.5;
        double baseY = Math.floor(basePos.y);
        double baseZ = Math.floor(basePos.z) + 0.5;
        int cols = explicitCols > 0
            ? Math.min(count, explicitCols)
            : (count <= 64 ? Math.min(8, count) : (int) Math.ceil(Math.sqrt(count)));

        this.swarmMode = mode;
        this.swarmTotalCount = count;
        this.engineTotalBots = count;
        this.totalTeleports = 0L;
        this.pendingClear = false;
        this.formationCols = cols;
        this.formationSpacing = spacing;
        this.formationAnchorX = (int) Math.floor(baseX);
        this.formationAnchorY = (int) Math.floor(baseY);
        this.formationAnchorZ = (int) Math.floor(baseZ);
        this.paradeActive = false;
        this.followPlayerUuid = null;
        this.followHasLast = false;

        if (mode == ShmBridge.MODE_MATERIALIZED) {
            ServerLevel overworld = server.overworld();
            for (int i = 0; i < count; i++) {
                double relX = (i % cols) * (double) spacing;
                double relZ = (i / cols) * (double) spacing;
                int botId = nextBotId++;
                spawnMaterialized(overworld, botId, i, baseX + relX, baseY, baseZ + relZ, 0.0, 0.0, 0.0, 0.0f, 0.0f, true, false, relX, relZ, true);
            }
        } else {
            int startId = nextBotId;
            nextBotId += count;
            pendingSpawnBatch = new PendingSpawnBatch(
                count,
                startId,
                0,
                mode,
                cols,
                baseX,
                baseY,
                baseZ,
                spacing,
                spacing
            );
            engineVirtCount = count;
            engineVisCount = 0;
            engineMatCount = 0;
        }
        return reply(
            source,
            String.format(
                Locale.ROOT,
                "Spawned %d %s bot(s) at (%.1f, %.1f, %.1f) spacing %.2f cols %d",
                count,
                modeArg.toLowerCase(Locale.ROOT),
                baseX,
                baseY,
                baseZ,
                spacing,
                cols
            )
        );
    }

    private SwarmBot spawnMaterialized(
        ServerLevel level,
        int botId,
        int botIndex,
        double x,
        double y,
        double z,
        double vx,
        double vy,
        double vz,
        float yaw,
        float pitch,
        boolean onGround,
        boolean sprinting,
        double relX,
        double relZ,
        boolean syncToRust
    ) {
        String name = botName(botIndex);
        UUID uuid = UUID.nameUUIDFromBytes(("OfflinePlayer:" + name).getBytes(StandardCharsets.UTF_8));
        GameProfile profile = new GameProfile(uuid, name);
        SwarmBot.SinkConnection sink = new SwarmBot.SinkConnection();
        SwarmBot bot = new SwarmBot(server, level, profile, botId, sink, relX, relZ);

        try {
            suppressJoinChat = true;
            server.getPlayerList().placeNewPlayer(sink, bot, CommonListenerCookie.createInitial(profile, false));
        } finally {
            suppressJoinChat = false;
        }
        bot.setGameMode(GameType.SURVIVAL);

        AttributeInstance receiveRange = bot.getAttribute(Attributes.WAYPOINT_RECEIVE_RANGE);
        if (receiveRange != null) {
            receiveRange.setBaseValue(0.0);
        }

        bot.connection.teleport(x, y, z, yaw, pitch);
        if (sink.pendingTpId >= 0) {
            bot.connection.handleAcceptTeleportPacket(new ServerboundAcceptTeleportationPacket(sink.pendingTpId));
            sink.pendingTpId = -1;
            sink.pendingTpPos = null;
        }
        bot.connection.handleAcceptPlayerLoad(new ServerboundPlayerLoadedPacket());
        bot.snapTo(x, y, z, yaw, pitch);
        bot.setDeltaMovement(vx, vy, vz);
        bot.setOnGround(onGround);
        bot.setSprinting(sprinting);
        bot.connection.resetPosition();
        bot.level().getChunkSource().move(bot);

        matBots.add(bot);
        matById.put(botId, bot);

        if (syncToRust && bridge != null && bridge.isEngineAlive()) {
            sendBotSync(ShmBridge.MSG_BOT_SPAWN, bot, 0);
        }
        return bot;
    }

    private void demoteMaterialized(int botId) {
        SwarmBot bot = matById.remove(botId);
        if (bot == null) {
            return;
        }
        matBots.remove(bot);
        try {
            suppressJoinChat = true;
            server.getPlayerList().remove(bot);
        } finally {
            suppressJoinChat = false;
        }
    }

    private void clearAllState(boolean sendClearImmediately) {
        pendingGoals.clear();
        pendingBatchGoal = null;
        pendingSpawnBatch = null;
        pendingStopAll = false;
        paradeActive = false;
        followPlayerUuid = null;
        followHasLast = false;
        try {
            suppressJoinChat = true;
            for (int i = 0; i < matBots.size(); i++) {
                SwarmBot bot = matBots.get(i);
                if (sendClearImmediately && bridge != null && bridge.isEngineAlive()) {
                    pw.reset();
                    pw.u32(bot.botId);
                    pw.send(bridge, ShmBridge.MSG_BOT_REMOVE);
                }
                server.getPlayerList().remove(bot);
            }
        } finally {
            suppressJoinChat = false;
        }
        matBots.clear();
        matById.clear();
        visualProjector.clear(server);
        swarmTotalCount = 0;
        engineTotalBots = 0;
        engineActiveBots = 0;
        engineVirtCount = 0;
        engineVisCount = 0;
        engineMatCount = 0;
        engineArrivedCount = 0;
        engineSettledCount = 0;
        engineExactTCount = 0;
        engineMinActT = 0;
        engineMaxActT = 0;
        enginePromotions = 0;
        engineDemotions = 0;
        engineHookedPromotions = 0;
        hasBatchSummary = false;
        if (sendClearImmediately) {
            if (bridge != null && bridge.isEngineAlive()) {
                pw.reset();
                pw.send(bridge, ShmBridge.MSG_SWARM_CLEAR);
                pendingClear = false;
            } else {
                pendingClear = true;
            }
        }
    }

    private int cmdRemove(CommandSourceStack source) {
        this.server = source.getServer();
        int count = Math.max(swarmTotalCount, matBots.size());
        clearAllState(true);
        return reply(source, "Removed " + count + " bot(s).");
    }

    private int cmdFormation(CommandSourceStack source, String kind) {
        switch (kind.toLowerCase(Locale.ROOT)) {
            case "grid" -> formation = Formation.GRID;
            case "point" -> formation = Formation.POINT;
            default -> {
                source.sendFailure(Component.literal("Expected grid or point, got '" + kind + "'"));
                return 0;
            }
        }
        return reply(source, "Formation: " + formation.name().toLowerCase(Locale.ROOT));
    }

    private int cmdStick(CommandSourceStack source, String modeArg) {
        switch (modeArg.toLowerCase(Locale.ROOT)) {
            case "all" -> stickMode = StickMode.ALL;
            case "one" -> stickMode = StickMode.ONE;
            default -> {
                source.sendFailure(Component.literal("Expected all or one, got '" + modeArg + "'"));
                return 0;
            }
        }
        return reply(source, "Stick mode: " + stickMode.name().toLowerCase(Locale.ROOT));
    }

    private int cmdConfig(
        CommandSourceStack source,
        int maxVisual,
        int maxMaterialized,
        float materializeRadius,
        int canaryCount,
        int canaryPeriod
    ) {
        this.cfgMaxVisual = maxVisual;
        this.cfgMaxMaterialized = maxMaterialized;
        this.cfgMaterializeRadius = materializeRadius;
        this.cfgCanaryCount = canaryCount;
        this.cfgCanaryPeriod = canaryPeriod;
        this.configDirty = true;
        return reply(
            source,
            String.format(
                Locale.ROOT,
                "Config: maxVisual=%d maxMat=%d radius=%.1f canary=%d/%d",
                maxVisual, maxMaterialized, materializeRadius, canaryCount, canaryPeriod
            )
        );
    }

    private int cmdObserverAdd(CommandSourceStack source, String name, BlockPos pos, float viewDist) {
        SyntheticObserver obs = syntheticObservers.get(name);
        double x = pos.getX() + 0.5;
        double y = pos.getY();
        double z = pos.getZ() + 0.5;
        if (obs == null) {
            syntheticObservers.put(name, new SyntheticObserver(nextObserverId++, x, y, z, viewDist));
        } else {
            obs.x = x;
            obs.y = y;
            obs.z = z;
            obs.viewDist = viewDist;
        }
        return reply(source, String.format(Locale.ROOT, "Observer %s at (%.1f, %.1f, %.1f) r=%.1f", name, x, y, z, viewDist));
    }

    private int cmdObserverMove(CommandSourceStack source, String name, BlockPos pos) {
        SyntheticObserver obs = syntheticObservers.get(name);
        if (obs == null) {
            return cmdObserverAdd(source, name, pos, 64.0f);
        }
        obs.x = pos.getX() + 0.5;
        obs.y = pos.getY();
        obs.z = pos.getZ() + 0.5;
        return reply(source, String.format(Locale.ROOT, "Moved observer %s to (%.1f, %.1f, %.1f)", name, obs.x, obs.y, obs.z));
    }

    private int cmdObserverRemove(CommandSourceStack source, String name) {
        SyntheticObserver removed = syntheticObservers.remove(name);
        return reply(source, removed != null ? "Removed observer " + name : "No observer " + name);
    }

    private int cmdObserverClear(CommandSourceStack source) {
        int n = syntheticObservers.size();
        syntheticObservers.clear();
        return reply(source, "Cleared " + n + " observer(s).");
    }

    private int cmdProfileReset(CommandSourceStack source) {
        tickSampleHead = 0;
        tickSampleCount = 0;
        flightSampleCount = 0;
        mindSampleHead = 0;
        mindSampleCount = 0;
        mindFlightSampleCount = 0;
        visualProjector.resetPacketCount();
        worldBridge.resetTicketsIssued();
        renderFramesSent = 0L;
        return reply(source, "Tick profile reset.");
    }

    private BlockPos snapGoalToGround(ServerLevel level, BlockPos pos) {
        if (!level.isLoaded(pos)) {
            return pos;
        }
        if (!level.getBlockState(pos).getCollisionShape(level, pos).isEmpty()
            && level.getBlockState(pos.above()).getCollisionShape(level, pos.above()).isEmpty()
            && level.getBlockState(pos.above(2)).getCollisionShape(level, pos.above(2)).isEmpty()) {
            return pos.above();
        }
        if (level.getBlockState(pos.below()).getCollisionShape(level, pos.below()).isEmpty()) {
            int minY = Math.max(level.getMinY(), pos.getY() - 128);
            for (int y = pos.getY() - 1; y >= minY; y--) {
                BlockPos solid = new BlockPos(pos.getX(), y, pos.getZ());
                BlockPos air1 = solid.above();
                BlockPos air2 = solid.above(2);
                if (!level.getBlockState(solid).getCollisionShape(level, solid).isEmpty()
                    && level.getBlockState(air1).getCollisionShape(level, air1).isEmpty()
                    && level.getBlockState(air2).getCollisionShape(level, air2).isEmpty()) {
                    return air1;
                }
            }
        }
        return pos;
    }

    private BlockPos centeredFormationGoal(BlockPos groundPos) {
        int total = Math.max(swarmTotalCount, matBots.size());
        if (formation != Formation.GRID || total <= 1) {
            return groundPos;
        }
        int cols = Math.max(1, formationCols);
        int rows = (total + cols - 1) / cols;
        int gx = groundPos.getX() - Math.round((cols - 1) * formationSpacing * 0.5f);
        int gz = groundPos.getZ() - Math.round((rows - 1) * formationSpacing * 0.5f);
        return new BlockPos(gx, groundPos.getY(), gz);
    }

    private BlockPos paradeCorner(int leg) {
        return switch (leg & 3) {
            case 0 -> new BlockPos(paradeOriginX + paradeDist, paradeOriginY, paradeOriginZ);
            case 1 -> new BlockPos(paradeOriginX + paradeDist, paradeOriginY, paradeOriginZ + paradeDist);
            case 2 -> new BlockPos(paradeOriginX, paradeOriginY, paradeOriginZ + paradeDist);
            default -> new BlockPos(paradeOriginX, paradeOriginY, paradeOriginZ);
        };
    }

    private boolean anyBotActive() {
        if (swarmMode == ShmBridge.MODE_MATERIALIZED) {
            for (int i = 0; i < matBots.size(); i++) {
                SwarmBot b = matBots.get(i);
                if (b.executing || b.settling) {
                    return true;
                }
            }
            return false;
        }
        return engineActiveBots > 0;
    }

    private int cmdGoto(CommandSourceStack source, BlockPos goal) {
        int total = Math.max(swarmTotalCount, matBots.size());
        if (total == 0) {
            return reply(source, "No bots spawned.");
        }
        paradeActive = false;
        followPlayerUuid = null;
        followHasLast = false;
        BlockPos snapped = snapGoalToGround(source.getServer().overworld(), goal);
        String requester = source.getPlayer() != null ? source.getPlayer().getPlainTextName() : null;
        issueGoal(snapped.getX(), snapped.getY(), snapped.getZ(), 0L, requester, true);
        String snapNote = snapped.getY() != goal.getY() ? " (snapped from y=" + goal.getY() + ")" : "";
        return reply(source, "Goal (" + snapped.getX() + " " + snapped.getY() + " " + snapped.getZ() + ")" + snapNote + " queued for " + total + " bot(s).");
    }

    private int cmdParade(CommandSourceStack source, int distance) {
        int total = Math.max(swarmTotalCount, matBots.size());
        if (total == 0) {
            return reply(source, "No bots spawned.");
        }
        this.server = source.getServer();
        followPlayerUuid = null;
        followHasLast = false;
        BlockPos origin = snapGoalToGround(server.overworld(), new BlockPos(formationAnchorX, formationAnchorY, formationAnchorZ));
        paradeOriginX = origin.getX();
        paradeOriginY = origin.getY();
        paradeOriginZ = origin.getZ();
        paradeDist = distance;
        paradeLeg = 0;
        paradeWaitTicks = 0;
        paradeActive = true;
        BlockPos leg0 = paradeCorner(0);
        String requester = source.getPlayer() != null ? source.getPlayer().getPlainTextName() : null;
        issueGoal(leg0.getX(), leg0.getY(), leg0.getZ(), 0L, requester, true);
        return reply(
            source,
            String.format(
                Locale.ROOT,
                "Stormtrooper parade active for %d bot(s): %d-block square from (%d %d %d).",
                total, distance, paradeOriginX, paradeOriginY, paradeOriginZ
            )
        );
    }

    private int cmdParadeOff(CommandSourceStack source) {
        paradeActive = false;
        stopAll();
        return reply(source, "Parade stopped (" + Math.max(swarmTotalCount, matBots.size()) + " bot(s)).");
    }

    private int cmdCome(CommandSourceStack source) {
        int total = Math.max(swarmTotalCount, matBots.size());
        if (total == 0) {
            return reply(source, "No bots spawned.");
        }
        ServerPlayer player = source.getPlayer();
        if (player == null) {
            source.sendFailure(Component.literal("/swarm come requires a player caller."));
            return 0;
        }
        paradeActive = false;
        followPlayerUuid = null;
        followHasLast = false;
        BlockPos ground = snapGoalToGround(source.getServer().overworld(), player.blockPosition());
        BlockPos anchor = centeredFormationGoal(ground);
        issueGoal(anchor.getX(), anchor.getY(), anchor.getZ(), 0L, player.getPlainTextName(), true);
        return reply(source, "Marching " + total + " bot(s) around (" + ground.getX() + " " + ground.getY() + " " + ground.getZ() + ").");
    }

    private int cmdFollow(CommandSourceStack source, String modeArg) {
        int total = Math.max(swarmTotalCount, matBots.size());
        if (total == 0) {
            return reply(source, "No bots spawned.");
        }
        switch (modeArg.toLowerCase(Locale.ROOT)) {
            case "on" -> {
                ServerPlayer player = source.getPlayer();
                if (player == null) {
                    source.sendFailure(Component.literal("/swarm follow on requires a player caller."));
                    return 0;
                }
                paradeActive = false;
                followPlayerUuid = player.getUUID();
                followHasLast = false;
                followCooldownTicks = 0;
                BlockPos ground = snapGoalToGround(source.getServer().overworld(), player.blockPosition());
                BlockPos anchor = centeredFormationGoal(ground);
                followLastX = anchor.getX();
                followLastZ = anchor.getZ();
                followHasLast = true;
                followCooldownTicks = 30;
                issueGoal(anchor.getX(), anchor.getY(), anchor.getZ(), 0L, player.getPlainTextName(), true);
                return reply(source, "Swarm following " + player.getPlainTextName() + ".");
            }
            case "off" -> {
                followPlayerUuid = null;
                followHasLast = false;
                return reply(source, "Swarm follow disabled.");
            }
            default -> {
                source.sendFailure(Component.literal("Expected on or off, got '" + modeArg + "'"));
                return 0;
            }
        }
    }

    private int cmdStop(CommandSourceStack source) {
        stopAll();
        return reply(source, "Stopped " + Math.max(swarmTotalCount, matBots.size()) + " bot(s).");
    }

    public void onWhisper(SwarmBot target, String senderName, String text) {
        String[] w = text.trim().split("\\s+");
        if (w.length == 0 || w[0].isEmpty()) {
            return;
        }
        switch (w[0]) {
            case "goto" -> {
                if (w.length < 4) {
                    notifyPlayer(senderName, "Usage: goto <x> <y> <z>");
                    return;
                }
                try {
                    int gx = Integer.parseInt(w[1]);
                    int gy = Integer.parseInt(w[2]);
                    int gz = Integer.parseInt(w[3]);
                    long qpc = w.length >= 5 ? Long.parseLong(w[4]) : 0L;
                    boolean all = stickMode == StickMode.ALL && !matBots.isEmpty() && target.botId == matBots.getFirst().botId;
                    if (all) {
                        issueGoal(gx, gy, gz, qpc, senderName, true);
                    } else {
                        issueSingleGoal(target, gx, gy, gz, qpc, senderName);
                    }
                } catch (NumberFormatException ex) {
                    notifyPlayer(senderName, "Invalid coordinates.");
                }
            }
            case "stop" -> {
                stopAll();
                notifyPlayer(senderName, "Stopped " + Math.max(swarmTotalCount, matBots.size()) + " bot(s).");
            }
            default -> { }
        }
    }

    private void resetBatchTelemetry(String requester, int expected) {
        flightSampleCount = 0;
        mindFlightSampleCount = 0;
        profilingFlight = true;
        lastBatchRequester = requester;
        expectedBatchReplies = expected;
        receivedBatchReplies = 0;
        batchMaxSolveNs = 0L;
        batchReadyCount = 0;
        batchRefusedCount = 0;
        batchSampleRefusal = "";
        hasBatchSummary = false;
        engineArrivedCount = 0;
        engineSettledCount = 0;
        engineExactTCount = 0;
        engineMinActT = 0;
        engineMaxActT = 0;
    }

    private void issueGoal(int gx, int gy, int gz, long issueQpc, String requester, boolean useFormation) {
        pendingGoals.clear();
        pendingBatchGoal = null;
        if (useFormation && formation == Formation.GRID) {
            this.formationAnchorX = gx;
            this.formationAnchorY = gy;
            this.formationAnchorZ = gz;
        }
        if (swarmMode == ShmBridge.MODE_MATERIALIZED) {
            resetBatchTelemetry(requester, matBots.size());
            for (int i = 0; i < matBots.size(); i++) {
                SwarmBot b = matBots.get(i);
                int tx = (useFormation && formation == Formation.GRID && matBots.size() > 1)
                    ? gx + (int) Math.floor(b.relX)
                    : gx;
                int tz = (useFormation && formation == Formation.GRID && matBots.size() > 1)
                    ? gz + (int) Math.floor(b.relZ)
                    : gz;
                resetBotPlanFields(b, requester);
                pendingGoals.add(new PendingGoal(b.botId, tx, gy, tz, issueQpc, requester));
            }
        } else {
            int total = Math.max(swarmTotalCount, engineTotalBots);
            resetBatchTelemetry(requester, total);
            for (int i = 0; i < matBots.size(); i++) {
                resetBotPlanFields(matBots.get(i), requester);
            }
            pendingBatchGoal = new PendingBatchGoal(
                gx,
                gy,
                gz,
                useFormation && formation == Formation.GRID,
                issueQpc,
                requester
            );
        }
    }

    private void issueSingleGoal(SwarmBot bot, int gx, int gy, int gz, long issueQpc, String requester) {
        pendingGoals.clear();
        pendingBatchGoal = null;
        resetBatchTelemetry(requester, 1);
        resetBotPlanFields(bot, requester);
        pendingGoals.add(new PendingGoal(bot.botId, gx, gy, gz, issueQpc, requester));
    }

    private static void resetBotPlanFields(SwarmBot b, String requester) {
        b.lastSolveNs = 0L;
        b.lastPredictedTicks = 0;
        b.activeTicks = 0;
        b.actualT = -1;
        b.executing = false;
        b.settling = false;
        b.arrived = false;
        b.settled = false;
        b.lastStatus = "pending";
        b.lastMsg = "";
        b.lastRequester = requester;
    }

    private void stopAll() {
        pendingGoals.clear();
        pendingBatchGoal = null;
        profilingFlight = false;
        paradeActive = false;
        followPlayerUuid = null;
        followHasLast = false;
        for (int i = 0; i < matBots.size(); i++) {
            SwarmBot b = matBots.get(i);
            b.executing = false;
            b.settling = false;
            b.lastStatus = "stopped";
        }
        if (swarmMode == ShmBridge.MODE_MATERIALIZED) {
            if (bridge != null && bridge.isEngineAlive()) {
                for (int i = 0; i < matBots.size(); i++) {
                    pw.reset();
                    pw.u32(matBots.get(i).botId);
                    pw.send(bridge, ShmBridge.MSG_BOT_STOP);
                }
            }
        } else {
            pendingStopAll = true;
        }
    }

    private int cmdStatus(CommandSourceStack source) {
        boolean alive = bridge != null && bridge.isEngineAlive();
        long engineEpoch = bridge != null ? bridge.engineEpoch() : 0L;
        long serverEpoch = bridge != null ? bridge.serverEpoch() : 0L;

        int totalBots = swarmMode == ShmBridge.MODE_MATERIALIZED
            ? matBots.size()
            : Math.max(swarmTotalCount, engineTotalBots);

        if (swarmMode == ShmBridge.MODE_MATERIALIZED) {
            return reply(source, formatMaterializedStatus(alive, engineEpoch, serverEpoch));
        }

        String solveSummary = hasBatchSummary
            ? String.format(
                Locale.ROOT,
                "%.1f/%.1f/%.1f",
                summaryMinSolveNs / 1000.0,
                summaryMedSolveNs / 1000.0,
                summaryMaxSolveNs / 1000.0
            )
            : "n/a";
        String predSummary = summaryMaxPredT > 0
            ? (summaryMinPredT == summaryMaxPredT
                ? String.valueOf(summaryMinPredT)
                : summaryMinPredT + ".." + summaryMaxPredT)
            : "n/a";
        String actSummary = engineMaxActT > 0
            ? (engineMinActT == engineMaxActT
                ? String.valueOf(engineMinActT)
                : engineMinActT + ".." + engineMaxActT)
            : "n/a";

        StringBuilder sb = new StringBuilder();
        sb.append("engine=").append(alive ? "connected" : "disconnected")
          .append(" engine_epoch=").append(engineEpoch)
          .append(" server_epoch=").append(serverEpoch)
          .append(" sections=").append(reportedSections)
          .append(" disk_sections=").append(reportedDiskSections)
          .append(" loaded_chunks=").append(worldBridge.loadedChunkCount())
          .append(" bots=").append(totalBots)
          .append(" executing=").append(engineActiveBots)
          .append(" settling=0")
          .append(" arrived=").append(engineArrivedCount).append('/').append(totalBots)
          .append(" settled=").append(engineSettledCount).append('/').append(totalBots)
          .append(" exact_T=").append(engineExactTCount).append('/').append(totalBots)
          .append(" teleports=").append(totalTeleports)
          .append(" solve_us=").append(solveSummary)
          .append(" predicted_T=").append(predSummary)
          .append(" actual_T=").append(actSummary)
          .append(" tick_ms=").append(formatSummary(flightSampleCount, flightNanos, tickSampleCount, tickNanos))
          .append(" mind_ms=").append(formatSummary(mindFlightSampleCount, mindFlightNanos, mindSampleCount, mindNanos))
          .append(" status=ready:").append(summaryReady)
          .append(",already:").append(summaryAlready)
          .append(",joints:").append(summaryJoints)
          .append(",refused:").append(summaryRefused)
          .append(" tiers=virt:").append(engineVirtCount)
          .append(",vis:").append(engineVisCount)
          .append(",mat:").append(engineMatCount)
          .append(" prom=").append(enginePromotions)
          .append(" dem=").append(engineDemotions)
          .append(" hook_prom=").append(engineHookedPromotions)
          .append(" vis_pkts=").append(visualProjector.packetsBuilt())
          .append(" render_frames=").append(renderFramesSent)
          .append(" tickets=").append(worldBridge.ticketsIssued());
        if (!matBots.isEmpty()) {
            SwarmBot b0 = matBots.getFirst();
            sb.append(String.format(
                Locale.ROOT,
                " bot0=[status:%s,msg:%s,pred:%d,act:%d,pos:(%.3f,%.3f,%.3f)]",
                b0.lastStatus,
                b0.lastMsg.replace(' ', '_'),
                b0.lastPredictedTicks,
                b0.actualT,
                b0.getX(),
                b0.getY(),
                b0.getZ()
            ));
        }
        return reply(source, sb.toString());
    }

    private String formatMaterializedStatus(boolean alive, long engineEpoch, long serverEpoch) {
        int execCount = 0;
        int settleCount = 0;
        int arrivedCount = 0;
        int settledCount = 0;
        int needsJointsCount = 0;
        int refusedCount = 0;
        int readyCount = 0;
        int alreadyCount = 0;

        long minSolveNs = Long.MAX_VALUE;
        long maxSolveNs = 0L;
        long[] solveArr = new long[matBots.size()];
        int solveCount = 0;

        int minPred = Integer.MAX_VALUE;
        int maxPred = 0;
        int minAct = Integer.MAX_VALUE;
        int maxAct = 0;
        int exactMatches = 0;

        for (int i = 0; i < matBots.size(); i++) {
            SwarmBot b = matBots.get(i);
            if (b.executing) execCount++;
            if (b.settling) settleCount++;
            if (b.arrived) arrivedCount++;
            if (b.settled) settledCount++;
            switch (b.lastStatus) {
                case "ready" -> readyCount++;
                case "already_there" -> alreadyCount++;
                case "needs_joints" -> needsJointsCount++;
                case "refused" -> refusedCount++;
                default -> { }
            }
            if (b.lastSolveNs > 0L) {
                solveArr[solveCount++] = b.lastSolveNs;
                minSolveNs = Math.min(minSolveNs, b.lastSolveNs);
                maxSolveNs = Math.max(maxSolveNs, b.lastSolveNs);
            }
            if (b.lastPredictedTicks > 0) {
                minPred = Math.min(minPred, b.lastPredictedTicks);
                maxPred = Math.max(maxPred, b.lastPredictedTicks);
            }
            if (b.actualT >= 0) {
                minAct = Math.min(minAct, b.actualT);
                maxAct = Math.max(maxAct, b.actualT);
                if (b.actualT == b.lastPredictedTicks) {
                    exactMatches++;
                }
            }
        }

        String solveSummary;
        if (solveCount == 0) {
            solveSummary = "n/a";
        } else {
            Arrays.sort(solveArr, 0, solveCount);
            solveSummary = String.format(
                Locale.ROOT,
                "%.1f/%.1f/%.1f",
                minSolveNs / 1000.0,
                solveArr[solveCount / 2] / 1000.0,
                maxSolveNs / 1000.0
            );
        }
        String predSummary = minPred <= maxPred ? (minPred == maxPred ? String.valueOf(minPred) : minPred + ".." + maxPred) : "n/a";
        String actSummary = minAct <= maxAct ? (minAct == maxAct ? String.valueOf(minAct) : minAct + ".." + maxAct) : "n/a";

        StringBuilder sb = new StringBuilder();
        sb.append("engine=").append(alive ? "connected" : "disconnected")
          .append(" engine_epoch=").append(engineEpoch)
          .append(" server_epoch=").append(serverEpoch)
          .append(" sections=").append(reportedSections)
          .append(" disk_sections=").append(reportedDiskSections)
          .append(" loaded_chunks=").append(worldBridge.loadedChunkCount())
          .append(" bots=").append(matBots.size())
          .append(" executing=").append(execCount)
          .append(" settling=").append(settleCount)
          .append(" arrived=").append(arrivedCount).append('/').append(matBots.size())
          .append(" settled=").append(settledCount).append('/').append(matBots.size())
          .append(" exact_T=").append(exactMatches).append('/').append(matBots.size())
          .append(" teleports=").append(totalTeleports)
          .append(" solve_us=").append(solveSummary)
          .append(" predicted_T=").append(predSummary)
          .append(" actual_T=").append(actSummary)
          .append(" tick_ms=").append(formatSummary(flightSampleCount, flightNanos, tickSampleCount, tickNanos))
          .append(" mind_ms=").append(formatSummary(mindFlightSampleCount, mindFlightNanos, mindSampleCount, mindNanos))
          .append(" status=ready:").append(readyCount)
          .append(",already:").append(alreadyCount)
          .append(",joints:").append(needsJointsCount)
          .append(",refused:").append(refusedCount);

        if (!matBots.isEmpty()) {
            SwarmBot b0 = matBots.getFirst();
            sb.append(String.format(
                Locale.ROOT,
                " bot0=[status:%s,msg:%s,pred:%d,act:%d,pos:(%.3f,%.3f,%.3f)]",
                b0.lastStatus,
                b0.lastMsg.replace(' ', '_'),
                b0.lastPredictedTicks,
                b0.actualT,
                b0.getX(),
                b0.getY(),
                b0.getZ()
            ));
        }
        return sb.toString();
    }

    private static String formatSummary(int fCount, long[] fArr, int sCount, long[] sArr) {
        int n = fCount > 0 ? fCount : sCount;
        if (n == 0) {
            return "n/a";
        }
        long[] src = fCount > 0 ? fArr : sArr;
        long[] copy = Arrays.copyOf(src, n);
        Arrays.sort(copy);
        return String.format(
            Locale.ROOT,
            "%.3f/%.3f/%.3f/%.3f/%.3f(n=%d)",
            copy[0] / 1e6,
            copy[n / 2] / 1e6,
            copy[Math.min(n - 1, (int) Math.ceil(0.90 * n) - 1)] / 1e6,
            copy[Math.min(n - 1, (int) Math.ceil(0.99 * n) - 1)] / 1e6,
            copy[n - 1] / 1e6,
            n
        );
    }

    public void onServerTickStart(MinecraftServer srv) {
        currentTickStartNs = System.nanoTime();
        this.server = srv;
        if (bridge == null) {
            return;
        }

        boolean alive = bridge.isEngineAlive();
        if (!alive) {
            worldBridge.markDisconnected();
            pendingGoals.clear();
            for (int i = 0; i < matBots.size(); i++) {
                SwarmBot b = matBots.get(i);
                b.executing = false;
                b.settling = false;
                b.connection.tick();
            }
            return;
        }

        boolean heavyWorkThisTick = false;
        if (bridge.checkEngineReconnect()) {
            worldBridge.syncAll(srv, bridge);
            configDirty = true;
            knownModdedHumans.clear();
            renderStreamEnabled = false;
            lastSentModdedOnly = false;
            for (int i = 0; i < matBots.size(); i++) {
                SwarmBot b = matBots.get(i);
                b.executing = false;
                b.settling = false;
                sendBotSync(ShmBridge.MSG_BOT_SPAWN, b, 0);
            }
            heavyWorkThisTick = true;
        }

        worldBridge.flushPendingDeltas(srv, bridge);

        if (pendingClear) {
            pendingClear = false;
            pw.reset();
            pw.send(bridge, ShmBridge.MSG_SWARM_CLEAR);
        }

        sendObserversIfNeeded(srv);

        if (configDirty) {
            configDirty = false;
            pw.reset();
            pw.u32(lastSentModdedOnly ? 0 : cfgMaxVisual);
            pw.u32(cfgMaxMaterialized);
            pw.f32(cfgMaterializeRadius);
            pw.u32(cfgCanaryCount);
            pw.u32(cfgCanaryPeriod);
            pw.send(bridge, ShmBridge.MSG_SWARM_CONFIG);
        }

        if (pendingSpawnBatch != null) {
            PendingSpawnBatch sb = pendingSpawnBatch;
            pendingSpawnBatch = null;
            pw.reset();
            pw.u32(sb.count);
            pw.u32(sb.startBotId);
            pw.u32(sb.startIndex);
            pw.u8(sb.mode);
            pw.u32(sb.cols);
            pw.f64(sb.baseX);
            pw.f64(sb.baseY);
            pw.f64(sb.baseZ);
            pw.f64(sb.spacingX);
            pw.f64(sb.spacingZ);
            pw.send(bridge, ShmBridge.MSG_SWARM_SPAWN_BATCH);
            heavyWorkThisTick = true;
        }

        for (int i = 0; i < matBots.size(); i++) {
            SwarmBot b = matBots.get(i);
            if (b.sink.pendingTpId >= 0) {
                int id = b.sink.pendingTpId;
                b.sink.pendingTpId = -1;
                b.sink.pendingTpPos = null;
                b.connection.handleAcceptTeleportPacket(new ServerboundAcceptTeleportationPacket(id));
                if (b.executing || b.settling) {
                    totalTeleports++;
                }
                b.executing = false;
                b.settling = false;
                sendBotSync(ShmBridge.MSG_BOT_SELF_SYNC, b, 1);
            } else if (b.hurtMarked) {
                b.executing = false;
                b.settling = false;
                sendBotSync(ShmBridge.MSG_BOT_SELF_SYNC, b, 2);
            }
        }

        if (pendingStopAll) {
            pendingStopAll = false;
            pw.reset();
            pw.send(bridge, ShmBridge.MSG_SWARM_STOP_ALL);
        }

        if (paradeActive
            && pendingBatchGoal == null
            && pendingSpawnBatch == null
            && pendingGoals.isEmpty()
            && Math.max(swarmTotalCount, matBots.size()) > 0
            && !anyBotActive()) {
            if (++paradeWaitTicks >= 15) {
                paradeLeg = (paradeLeg + 1) & 3;
                paradeWaitTicks = 0;
                BlockPos corner = paradeCorner(paradeLeg);
                issueGoal(corner.getX(), corner.getY(), corner.getZ(), 0L, null, true);
            }
        } else if (followPlayerUuid != null
            && pendingBatchGoal == null
            && pendingSpawnBatch == null
            && pendingGoals.isEmpty()
            && Math.max(swarmTotalCount, matBots.size()) > 0
            && !anyBotActive()) {
            if (followCooldownTicks > 0) {
                followCooldownTicks--;
            } else {
                ServerPlayer target = srv.getPlayerList().getPlayer(followPlayerUuid);
                if (target != null) {
                    BlockPos ground = snapGoalToGround(srv.overworld(), target.blockPosition());
                    BlockPos anchor = centeredFormationGoal(ground);
                    int dx = anchor.getX() - followLastX;
                    int dz = anchor.getZ() - followLastZ;
                    if (!followHasLast || (dx * dx + dz * dz) >= 16 * 16) {
                        followLastX = anchor.getX();
                        followLastZ = anchor.getZ();
                        followHasLast = true;
                        followCooldownTicks = 30;
                        issueGoal(anchor.getX(), anchor.getY(), anchor.getZ(), 0L, null, true);
                    }
                }
            }
        }

        if (pendingBatchGoal != null) {
            PendingBatchGoal bg = pendingBatchGoal;
            pendingBatchGoal = null;
            pw.reset();
            pw.i32(bg.gx);
            pw.i32(bg.gy);
            pw.i32(bg.gz);
            pw.u8(bg.grid ? 0 : 1);
            pw.u64(bg.issueQpc);
            pw.send(bridge, ShmBridge.MSG_SWARM_GOAL_BATCH);
            heavyWorkThisTick = true;
        }

        for (int i = 0; i < pendingGoals.size(); i++) {
            PendingGoal g = pendingGoals.get(i);
            pw.reset();
            pw.u32(g.botId);
            pw.i32(g.gx);
            pw.i32(g.gy);
            pw.i32(g.gz);
            pw.u64(g.issueQpc);
            pw.send(bridge, ShmBridge.MSG_BOT_GOAL);
            heavyWorkThisTick = true;
        }
        pendingGoals.clear();

        long tickSeq = ++serverTickSeq;
        pw.reset();
        pw.u64(tickSeq);
        pw.send(bridge, ShmBridge.MSG_TICK_BEGIN);
        bridge.setServerTickSeq(tickSeq);

        bridge.waitForTick(tickSeq, heavyWorkThisTick ? 500L : 50L);
        bridge.drainE2S((msgType, bb) -> handleEngineFrame(msgType, bb, tickSeq));

        for (int i = 0; i < matBots.size(); i++) {
            SwarmBot b = matBots.get(i);
            if (b.lastTickedSeq != tickSeq) {
                b.connection.tick();
            }
        }
    }

    private void sendObserversIfNeeded(MinecraftServer srv) {
        int realCount = 0;
        boolean hasVanilla = false;
        Set<UUID> currentModded = new HashSet<>();
        List<ServerPlayer> players = srv.overworld().players();
        for (int i = 0; i < players.size(); i++) {
            ServerPlayer p = players.get(i);
            if (!(p instanceof SwarmBot)) {
                realCount++;
                if (ServerPlayNetworking.canSend(p, SwarmFramePayload.TYPE)) {
                    currentModded.add(p.getUUID());
                } else {
                    hasVanilla = true;
                }
            }
        }
        boolean hasModded = !currentModded.isEmpty();
        boolean newModdedJoined = false;
        for (UUID u : currentModded) {
            if (!knownModdedHumans.contains(u)) {
                newModdedJoined = true;
                break;
            }
        }
        if (hasModded != renderStreamEnabled || newModdedJoined) {
            renderStreamEnabled = hasModded;
            pw.reset();
            pw.u8(hasModded ? 1 : 0);
            pw.u8(newModdedJoined ? 1 : 0);
            pw.send(bridge, ShmBridge.MSG_RENDER_STREAM);
        }
        knownModdedHumans.clear();
        knownModdedHumans.addAll(currentModded);
        boolean moddedOnly = hasModded && !hasVanilla && syntheticObservers.isEmpty();
        if (moddedOnly != lastSentModdedOnly) {
            lastSentModdedOnly = moddedOnly;
            configDirty = true;
        }

        int totalObs = realCount + syntheticObservers.size();
        if (totalObs == 0 && lastObserverCountSent == 0) {
            return;
        }
        lastObserverCountSent = totalObs;
        float humanViewDist = Math.max(16.0f, (srv.getPlayerList().getViewDistance() + 1) * 16.0f);

        pw.reset();
        pw.u32(totalObs);
        int hid = 1_000_000;
        for (int i = 0; i < players.size(); i++) {
            ServerPlayer p = players.get(i);
            if (!(p instanceof SwarmBot)) {
                pw.u32(hid++);
                pw.f64(p.getX());
                pw.f64(p.getY());
                pw.f64(p.getZ());
                pw.f32(humanViewDist);
            }
        }
        for (SyntheticObserver obs : syntheticObservers.values()) {
            pw.u32(obs.id);
            pw.f64(obs.x);
            pw.f64(obs.y);
            pw.f64(obs.z);
            pw.f32(obs.viewDist);
        }
        pw.send(bridge, ShmBridge.MSG_OBSERVERS);
    }

    public void onServerTickEnd() {
        if (currentTickStartNs == 0L) {
            return;
        }
        long elapsed = System.nanoTime() - currentTickStartNs;
        tickNanos[tickSampleHead] = elapsed;
        tickSampleHead = (tickSampleHead + 1) % tickNanos.length;
        if (tickSampleCount < tickNanos.length) {
            tickSampleCount++;
        }
        if (profilingFlight) {
            boolean anyActive = engineActiveBots > 0;
            if (!anyActive) {
                for (int i = 0; i < matBots.size(); i++) {
                    SwarmBot b = matBots.get(i);
                    if (b.executing || b.settling) {
                        anyActive = true;
                        break;
                    }
                }
            }
            if (anyActive && flightSampleCount < flightNanos.length) {
                flightNanos[flightSampleCount++] = elapsed;
            } else if (!anyActive && flightSampleCount > 0) {
                profilingFlight = false;
            }
        }
    }

    private void handleEngineFrame(int msgType, ByteBuffer bb, long tickSeq) {
        switch (msgType) {
            case ShmBridge.MSG_BOT_PLAN_STATUS -> {
                int botId = bb.getInt();
                int status = bb.getInt();
                int predictedTicks = bb.getInt();
                long solveNs = bb.getLong();
                String msg = ShmBridge.readString(bb);
                SwarmBot bot = matById.get(botId);
                if (bot == null) {
                    return;
                }
                bot.lastSolveNs = solveNs;
                bot.lastPredictedTicks = predictedTicks;
                bot.lastMsg = msg;
                switch (status) {
                    case ShmBridge.PLAN_STATUS_READY -> {
                        bot.lastStatus = "ready";
                        bot.executing = true;
                        batchReadyCount++;
                    }
                    case ShmBridge.PLAN_STATUS_ALREADY_THERE -> {
                        bot.lastStatus = "already_there";
                        bot.arrived = true;
                        bot.settled = true;
                        bot.actualT = 0;
                        batchReadyCount++;
                    }
                    case ShmBridge.PLAN_STATUS_NEEDS_JOINTS -> {
                        bot.lastStatus = "needs_joints";
                        batchRefusedCount++;
                        if (batchSampleRefusal.isEmpty()) batchSampleRefusal = msg;
                    }
                    default -> {
                        bot.lastStatus = "refused";
                        batchRefusedCount++;
                        if (batchSampleRefusal.isEmpty()) batchSampleRefusal = msg;
                    }
                }
                batchMaxSolveNs = Math.max(batchMaxSolveNs, solveNs);
                if (++receivedBatchReplies == expectedBatchReplies && lastBatchRequester != null) {
                    String note = batchRefusedCount == 0
                        ? String.format(Locale.ROOT, "Swarm ready (%d bot(s), max solve %.1f us, T=%d)", batchReadyCount, batchMaxSolveNs / 1000.0, predictedTicks)
                        : String.format(Locale.ROOT, "Swarm refused %d/%d (%s)", batchRefusedCount, expectedBatchReplies, batchSampleRefusal);
                    notifyPlayer(lastBatchRequester, note);
                }
            }
            case ShmBridge.MSG_SWARM_PLAN_SUMMARY -> {
                summaryTotal = bb.getInt();
                summaryReady = bb.getInt();
                summaryAlready = bb.getInt();
                summaryJoints = bb.getInt();
                summaryRefused = bb.getInt();
                summaryMinSolveNs = bb.getLong();
                summaryMedSolveNs = bb.getLong();
                summaryMaxSolveNs = bb.getLong();
                bb.getLong(); // batch_wall_ns
                summaryMinPredT = bb.getInt();
                summaryMaxPredT = bb.getInt();
                summarySampleMsg = ShmBridge.readString(bb);
                hasBatchSummary = true;
                if (lastBatchRequester != null && swarmMode != ShmBridge.MODE_MATERIALIZED) {
                    String note = (summaryJoints + summaryRefused) == 0
                        ? String.format(Locale.ROOT, "Swarm ready (%d bot(s), med solve %.1f us, T=%d)", summaryReady + summaryAlready, summaryMedSolveNs / 1000.0, summaryMaxPredT)
                        : String.format(Locale.ROOT, "Swarm refused %d/%d (%s)", summaryJoints + summaryRefused, summaryTotal, summarySampleMsg);
                    notifyPlayer(lastBatchRequester, note);
                }
            }
            case ShmBridge.MSG_CHUNK_TICKET -> {
                int cx = bb.getInt();
                int cz = bb.getInt();
                int level = Byte.toUnsignedInt(bb.get());
                worldBridge.requestTicket(server, cx, cz, level);
            }
            case ShmBridge.MSG_BOT_TIER_CHANGE -> {
                int botId = bb.getInt();
                int botIndex = bb.getInt();
                int newTier = Byte.toUnsignedInt(bb.get());
                int flags = bb.getInt();
                double x = bb.getDouble();
                double y = bb.getDouble();
                double z = bb.getDouble();
                double vx = bb.getDouble();
                double vy = bb.getDouble();
                double vz = bb.getDouble();
                float yaw = bb.getFloat();
                float pitch = bb.getFloat();

                if (newTier != ShmBridge.TIER_MATERIALIZED) {
                    demoteMaterialized(botId);
                }
                if (newTier == ShmBridge.TIER_VISUAL) {
                    visualProjector.addOrUpdateVisual(server, botId, botIndex, x, y, z, vx, vy, vz, yaw, pitch, flags, tickSeq);
                } else {
                    visualProjector.removeVisual(server, botId, false);
                }
                if (newTier == ShmBridge.TIER_MATERIALIZED && !matById.containsKey(botId)) {
                    boolean onGround = (flags & 1) != 0;
                    boolean sprinting = (flags & 2) != 0;
                    spawnMaterialized(
                        server.overworld(),
                        botId,
                        botIndex,
                        x, y, z, vx, vy, vz,
                        yaw, pitch,
                        onGround, sprinting,
                        0.0, 0.0,
                        false
                    );
                }
            }
            case ShmBridge.MSG_VISUAL_BATCH -> {
                int count = bb.getInt();
                for (int i = 0; i < count; i++) {
                    int botId = bb.getInt();
                    double x = bb.getDouble();
                    double y = bb.getDouble();
                    double z = bb.getDouble();
                    float vx = bb.getFloat();
                    float vy = bb.getFloat();
                    float vz = bb.getFloat();
                    float yaw = bb.getFloat();
                    float pitch = bb.getFloat();
                    int flags = Byte.toUnsignedInt(bb.get());
                    visualProjector.updateVisual(server, botId, x, y, z, vx, vy, vz, yaw, pitch, flags, tickSeq);
                }
            }
            case ShmBridge.MSG_BOT_TICK_CMD -> {
                int botId = bb.getInt();
                int keys = bb.getInt();
                int flags = bb.getInt();
                double x = bb.getDouble();
                double y = bb.getDouble();
                double z = bb.getDouble();
                float yaw = bb.getFloat();
                float pitch = bb.getFloat();

                SwarmBot bot = matById.get(botId);
                if (bot == null) {
                    return;
                }
                bot.lastTickedSeq = tickSeq;

                if ((flags & ShmBridge.FLAG_START_SPRINT) != 0) {
                    bot.connection.handlePlayerCommand(
                        new ServerboundPlayerCommandPacket(bot, ServerboundPlayerCommandPacket.Action.START_SPRINTING)
                    );
                } else if ((flags & ShmBridge.FLAG_STOP_SPRINT) != 0) {
                    bot.connection.handlePlayerCommand(
                        new ServerboundPlayerCommandPacket(bot, ServerboundPlayerCommandPacket.Action.STOP_SPRINTING)
                    );
                }

                bot.connection.handlePlayerInput(new ServerboundPlayerInputPacket(new Input(
                    (keys & 1) != 0,
                    (keys & 2) != 0,
                    (keys & 4) != 0,
                    (keys & 8) != 0,
                    (keys & 16) != 0,
                    (keys & 32) != 0,
                    (keys & 64) != 0
                )));
                bot.connection.handleMovePlayer(new ServerboundMovePlayerPacket.PosRot(
                    x, y, z, yaw, pitch,
                    (flags & ShmBridge.FLAG_ON_GROUND) != 0,
                    (flags & ShmBridge.FLAG_HORIZ_COL) != 0
                ));
                bot.connection.handleClientTickEnd(ServerboundClientTickEndPacket.INSTANCE);
                bot.connection.tick();

                if (bot.executing) {
                    bot.activeTicks++;
                }
                if ((flags & ShmBridge.FLAG_ARRIVED) != 0) {
                    bot.arrived = true;
                    bot.actualT = bot.activeTicks;
                    bot.executing = false;
                    bot.settling = (flags & ShmBridge.FLAG_SETTLED) == 0;
                }
                if ((flags & ShmBridge.FLAG_SETTLED) != 0) {
                    bot.settling = false;
                    bot.settled = true;
                }
            }
            case ShmBridge.MSG_RENDER_FRAME -> {
                byte[] frame = new byte[bb.remaining()];
                bb.get(frame);
                if (server != null) {
                    SwarmFramePayload payload = new SwarmFramePayload(frame);
                    boolean sentAny = false;
                    for (ServerPlayer p : server.overworld().players()) {
                        if (!(p instanceof SwarmBot) && ServerPlayNetworking.canSend(p, SwarmFramePayload.TYPE)) {
                            ServerPlayNetworking.send(p, payload);
                            sentAny = true;
                        }
                    }
                    if (sentAny) {
                        renderFramesSent++;
                    }
                }
            }
            case ShmBridge.MSG_TICK_END -> {
                bb.getLong(); // server_tick
                engineActiveBots = bb.getInt();
                reportedSections = bb.getInt();
                reportedDiskSections = bb.getInt();
                engineTotalBots = bb.getInt();
                engineVirtCount = bb.getInt();
                engineVisCount = bb.getInt();
                engineMatCount = bb.getInt();
                engineArrivedCount = bb.getInt();
                engineSettledCount = bb.getInt();
                engineExactTCount = bb.getInt();
                engineMinActT = bb.getInt();
                engineMaxActT = bb.getInt();
                enginePromotions = bb.getInt();
                engineDemotions = bb.getInt();
                engineHookedPromotions = bb.getInt();
                long mindNs = bb.getLong();
                mindNanos[mindSampleHead] = mindNs;
                mindSampleHead = (mindSampleHead + 1) % mindNanos.length;
                if (mindSampleCount < mindNanos.length) {
                    mindSampleCount++;
                }
                if (profilingFlight && engineActiveBots > 0 && mindFlightSampleCount < mindFlightNanos.length) {
                    mindFlightNanos[mindFlightSampleCount++] = mindNs;
                }
            }
            default -> { }
        }
    }

    private void sendBotSync(short msgType, SwarmBot b, int reason) {
        if (bridge == null || !bridge.isEngineAlive()) {
            return;
        }
        int flags = (b.onGround() ? 1 : 0) | (b.isSprinting() ? 2 : 0);
        Vec3 vel = b.getDeltaMovement();
        var food = b.getFoodData();
        FoodDataAccessor foodAcc = (FoodDataAccessor) (Object) food;
        LivingEntityAccessor livingAcc = (LivingEntityAccessor) (Object) b;

        pw.reset();
        pw.u32(b.botId);
        pw.u32(flags);
        pw.f64(b.getX());
        pw.f64(b.getY());
        pw.f64(b.getZ());
        pw.f64(vel.x);
        pw.f64(vel.y);
        pw.f64(vel.z);
        pw.f32(b.getYRot());
        pw.f32(b.getXRot());
        pw.f32(b.getHealth());
        pw.f32(food.getSaturationLevel());
        pw.f32(foodAcc.ptGetExhaustionLevel());
        pw.i32(food.getFoodLevel());
        pw.u32(foodAcc.ptGetTickTimer());
        pw.i32(b.invulnerableTime);
        pw.f32(livingAcc.ptGetLastHurt());
        if (msgType == ShmBridge.MSG_BOT_SELF_SYNC) {
            pw.u32(reason);
        }
        pw.send(bridge, msgType);
    }
}
