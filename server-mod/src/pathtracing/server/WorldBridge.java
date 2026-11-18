package pathtracing.server;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.LinkedHashSet;
import java.util.Map;
import java.util.Set;
import java.util.stream.Collectors;
import net.minecraft.core.BlockPos;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.Ticket;
import net.minecraft.server.level.TicketType;
import net.minecraft.world.level.ChunkPos;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.chunk.LevelChunk;
import net.minecraft.world.level.chunk.LevelChunkSection;
import net.minecraft.world.level.gamerules.GameRules;
import net.minecraft.world.level.storage.LevelResource;

/** Tracks loaded overworld chunks and streams world snapshots & live block deltas to Rust. */
public final class WorldBridge {
    private final Set<ChunkPos> loadedOverworldChunks = new LinkedHashSet<>();
    private final Map<BlockState, String> canonicalCache = new HashMap<>();
    private final ArrayList<BlockDelta> pendingDeltas = new ArrayList<>();
    private final ShmBridge.PacketWriter pw = new ShmBridge.PacketWriter();

    private long session = 1L;
    private long ticketsIssued = 0L;
    private int lastDifficulty = -1;
    private boolean lastNaturalRegen = true;
    private boolean syncedToEngine = false;

    private record BlockDelta(int x, int y, int z, String canonical) {}

    public int loadedChunkCount() {
        return loadedOverworldChunks.size();
    }

    public long ticketsIssued() {
        return ticketsIssued;
    }

    public void resetTicketsIssued() {
        ticketsIssued = 0L;
    }

    public void requestTicket(MinecraftServer server, int cx, int cz, int level) {
        if (server == null) {
            return;
        }
        int ticketLevel = Math.max(31, level - 1);
        server.overworld().getChunkSource().addTicket(
            new Ticket(TicketType.ENDER_PEARL, ticketLevel),
            new ChunkPos(cx, cz)
        );
        ticketsIssued++;
    }

    public void onChunkLoad(MinecraftServer server, ServerLevel level, LevelChunk chunk, ShmBridge bridge) {
        if (level != server.overworld()) {
            return;
        }
        ChunkPos pos = chunk.getPos();
        loadedOverworldChunks.add(pos);
        if (syncedToEngine && bridge != null && bridge.isEngineAlive()) {
            sendChunk(level, chunk, bridge);
        }
    }

    public void onChunkUnload(MinecraftServer server, ServerLevel level, LevelChunk chunk, ShmBridge bridge) {
        if (level != server.overworld()) {
            return;
        }
        ChunkPos pos = chunk.getPos();
        loadedOverworldChunks.remove(pos);
        if (syncedToEngine && bridge != null && bridge.isEngineAlive()) {
            pw.reset();
            pw.i32(pos.x());
            pw.i32(pos.z());
            pw.send(bridge, ShmBridge.MSG_CHUNK_UNLOAD);
        }
    }

    public void queueBlockDelta(MinecraftServer server, ServerLevel level, BlockPos pos, BlockState state) {
        if (level != server.overworld() || !syncedToEngine) {
            return;
        }
        ChunkPos cp = new ChunkPos(pos.getX() >> 4, pos.getZ() >> 4);
        if (!loadedOverworldChunks.contains(cp)) {
            return;
        }
        pendingDeltas.add(new BlockDelta(pos.getX(), pos.getY(), pos.getZ(), canonical(state)));
    }

    public void flushPendingDeltas(MinecraftServer server, ShmBridge bridge) {
        if (bridge == null || !bridge.isEngineAlive()) {
            pendingDeltas.clear();
            return;
        }
        checkRules(server.overworld(), bridge);
        for (int i = 0; i < pendingDeltas.size(); i++) {
            BlockDelta d = pendingDeltas.get(i);
            pw.reset();
            pw.i32(d.x);
            pw.i32(d.y);
            pw.i32(d.z);
            pw.str(d.canonical);
            pw.send(bridge, ShmBridge.MSG_BLOCK_DELTA);
        }
        pendingDeltas.clear();
    }

    public void syncAll(MinecraftServer server, ShmBridge bridge) {
        if (bridge == null || !bridge.isEngineAlive()) {
            return;
        }
        pendingDeltas.clear();
        ServerLevel level = server.overworld();
        lastDifficulty = level.getDifficulty().getId();
        lastNaturalRegen = level.getGameRules().get(GameRules.NATURAL_HEALTH_REGENERATION);
        session++;

        pw.reset();
        pw.u64(session);
        pw.u8(lastDifficulty);
        pw.u8(lastNaturalRegen ? 1 : 0);
        pw.str(server.getWorldPath(LevelResource.ROOT).toAbsolutePath().toString());
        if (!pw.send(bridge, ShmBridge.MSG_WORLD_RESET)) {
            return;
        }

        var cache = level.getChunkSource();
        var it = loadedOverworldChunks.iterator();
        while (it.hasNext()) {
            ChunkPos pos = it.next();
            LevelChunk chunk = cache.getChunkNow(pos.x(), pos.z());
            if (chunk == null) {
                it.remove();
                continue;
            }
            if (!sendChunk(level, chunk, bridge)) {
                return;
            }
        }
        syncedToEngine = true;
    }

    public void markDisconnected() {
        syncedToEngine = false;
        pendingDeltas.clear();
    }

    private void checkRules(ServerLevel level, ShmBridge bridge) {
        int diff = level.getDifficulty().getId();
        boolean regen = level.getGameRules().get(GameRules.NATURAL_HEALTH_REGENERATION);
        if (diff != lastDifficulty || regen != lastNaturalRegen) {
            lastDifficulty = diff;
            lastNaturalRegen = regen;
            pw.reset();
            pw.u8(diff);
            pw.u8(regen ? 1 : 0);
            pw.send(bridge, ShmBridge.MSG_WORLD_RULES);
        }
    }

    private boolean sendChunk(ServerLevel level, LevelChunk chunk, ShmBridge bridge) {
        ChunkPos pos = chunk.getPos();
        int minSecY = level.getMinSectionY();
        LevelChunkSection[] sections = chunk.getSections();
        for (int s = 0; s < sections.length; s++) {
            int cy = minSecY + s;
            LevelChunkSection sec = sections[s];
            if (sec.hasOnlyAir()) {
                pw.reset();
                pw.i32(pos.x());
                pw.i32(cy);
                pw.i32(pos.z());
                if (!pw.send(bridge, ShmBridge.MSG_SECTION_AIR)) {
                    return false;
                }
                continue;
            }

            Map<BlockState, Integer> localPalette = new HashMap<>();
            ArrayList<String> paletteNames = new ArrayList<>();
            int[] runLen = new int[4096];
            int[] runIdx = new int[4096];
            int runCount = 0;
            int lastIdx = -1;

            for (int y = 0; y < 16; y++) {
                for (int z = 0; z < 16; z++) {
                    for (int x = 0; x < 16; x++) {
                        BlockState bs = sec.getBlockState(x, y, z);
                        Integer idxObj = localPalette.get(bs);
                        int idx;
                        if (idxObj != null) {
                            idx = idxObj;
                        } else {
                            idx = paletteNames.size();
                            localPalette.put(bs, idx);
                            paletteNames.add(canonical(bs));
                        }
                        if (idx == lastIdx && runCount > 0) {
                            runLen[runCount - 1]++;
                        } else {
                            runLen[runCount] = 1;
                            runIdx[runCount] = idx;
                            runCount++;
                            lastIdx = idx;
                        }
                    }
                }
            }

            pw.reset();
            pw.i32(pos.x());
            pw.i32(cy);
            pw.i32(pos.z());
            pw.u16(paletteNames.size());
            pw.u16(runCount);
            for (int i = 0; i < paletteNames.size(); i++) {
                pw.str(paletteNames.get(i));
            }
            for (int i = 0; i < runCount; i++) {
                pw.u16(runLen[i]);
                pw.u16(runIdx[i]);
            }
            if (!pw.send(bridge, ShmBridge.MSG_SECTION_RLE)) {
                return false;
            }
        }
        return true;
    }

    private String canonical(BlockState state) {
        String cached = canonicalCache.get(state);
        if (cached != null) {
            return cached;
        }
        String props = state.getValues().map(Object::toString).sorted().collect(Collectors.joining(","));
        String out = BuiltInRegistries.BLOCK.getKey(state.getBlock()).toString()
            + (props.isEmpty() ? "" : "[" + props + "]");
        canonicalCache.put(state, out);
        return out;
    }
}
