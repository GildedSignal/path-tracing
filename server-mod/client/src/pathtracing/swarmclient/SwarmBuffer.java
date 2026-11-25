package pathtracing.swarmclient;

import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.FloatBuffer;
import java.nio.charset.StandardCharsets;
import java.util.ArrayDeque;
import java.util.Arrays;
import java.util.HashSet;
import java.util.List;
import java.util.Locale;
import java.util.Set;
import java.util.UUID;
import net.minecraft.client.multiplayer.ClientLevel;
import net.minecraft.client.player.AbstractClientPlayer;
import net.minecraft.core.BlockPos;
import net.minecraft.util.Mth;
import net.minecraft.world.level.LightLayer;
import net.minecraft.world.phys.Vec3;
import org.joml.Matrix4f;
import org.joml.Vector4f;
import org.lwjgl.system.MemoryUtil;

/**
 * Thread-safe tick-locked buffer unifying virtual SwarmFramePayloads and live client RemotePlayers
 * into 5 frustum-culled front-to-back GPU instanced buckets (LOD0/LOD1 WIDE & SLIM + LOD2).
 */
public final class SwarmBuffer {
    public static final int INSTANCE_FLOATS = 12;
    public static final int INSTANCE_BYTES = INSTANCE_FLOATS * 4;
    public static final int BUCKET_LOD0_WIDE = 0;
    public static final int BUCKET_LOD0_SLIM = 1;
    public static final int BUCKET_LOD1_WIDE = 2;
    public static final int BUCKET_LOD1_SLIM = 3;
    public static final int BUCKET_LOD2 = 4;
    public static final int BUCKET_COUNT = 5;

    private static final int DELTA_FLAG = 0x8000_0000;
    private static final int MAX_INDEX = 65536;
    private static final int MAX_QUEUE = 4;
    private static final float TWO_PI = (float) (Math.PI * 2.0);
    private static final float PI = (float) Math.PI;
    private static final float LOD0_MAX_DIST_SQ = 96.0f * 96.0f;
    private static final float LOD1_MAX_DIST_SQ = 220.0f * 220.0f;
    private static final float BOT_BOUNDING_RADIUS = 2.2f;
    private static final int SOURCE_VIRTUAL_BIT = 0x4000_0000;
    private static final int SOURCE_INDEX_MASK = 0x3FFF_FFFF;

    private final ArrayDeque<byte[]> pendingFrames = new ArrayDeque<>(MAX_QUEUE);
    private final int[] prevSlotByIndex = new int[MAX_INDEX];
    private final int[] currSlotByIndex = new int[MAX_INDEX];
    private final float[] currPhaseByIndex = new float[MAX_INDEX];
    private final byte[] skinCache = new byte[MAX_INDEX];
    private final UUID[] uuidCache = new UUID[MAX_INDEX];
    private final Set<UUID> trackedBotUuids = new HashSet<>();
    private int frameSeq = 0;

    private int currCount = 0;
    private double currOriginX, currOriginY, currOriginZ;
    private float[] currPrevRx = new float[0];
    private float[] currPrevRy = new float[0];
    private float[] currPrevRz = new float[0];
    private float[] currRx = new float[0];
    private float[] currRy = new float[0];
    private float[] currRz = new float[0];
    private float[] currYaw = new float[0];
    private float[] currPrevYaw = new float[0];
    private float[] currUnwrappedYaw = new float[0];
    private float[] currPitch = new float[0];
    private float[] currSwingAmp = new float[0];
    private float[] currPrevPhase = new float[0];
    private float[] currCurrPhase = new float[0];
    private float[] currSkinLayer = new float[0];
    private int[] currFlags = new int[0];
    private int[] currIndices = new int[0];
    private int[] currSlotFrameSeq = new int[0];

    private float[] snapRx = new float[0];
    private float[] snapRy = new float[0];
    private float[] snapRz = new float[0];
    private float[] snapYaw = new float[0];

    private int[] bucket0Wide = new int[1024];
    private int[] bucket0Slim = new int[1024];
    private int[] bucket1Wide = new int[1024];
    private int[] bucket1Slim = new int[1024];
    private int[] bucket2 = new int[1024];

    private ByteBuffer gpuBytes = MemoryUtil.memAlloc(1024 * INSTANCE_BYTES).order(ByteOrder.LITTLE_ENDIAN);
    private FloatBuffer gpuBuf = gpuBytes.asFloatBuffer();
    private int gpuCapacityBots = 1024;

    private final Vector4f pLeft = new Vector4f();
    private final Vector4f pRight = new Vector4f();
    private final Vector4f pBottom = new Vector4f();
    private final Vector4f pTop = new Vector4f();
    private final Vector4f pNear = new Vector4f();
    private final BlockPos.MutableBlockPos lightPos = new BlockPos.MutableBlockPos();

    public SwarmBuffer() {
        Arrays.fill(prevSlotByIndex, -1);
        Arrays.fill(currSlotByIndex, -1);
        Arrays.fill(skinCache, (byte) -1);
    }

    public static boolean isSwarmBotProfile(UUID uuid, String name) {
        if (uuid == null || name == null) {
            return false;
        }
        boolean match = "PathBot".equals(name);
        if (!match && name.length() == 9 && name.startsWith("Swarm")) {
            match = true;
            for (int i = 5; i < 9; i++) {
                char ch = name.charAt(i);
                if (ch < '0' || ch > '9') {
                    match = false;
                    break;
                }
            }
        }
        if (!match) {
            return false;
        }
        UUID expected = UUID.nameUUIDFromBytes(("OfflinePlayer:" + name).getBytes(StandardCharsets.UTF_8));
        return uuid.equals(expected);
    }

    public synchronized void clear() {
        pendingFrames.clear();
        Arrays.fill(prevSlotByIndex, -1);
        Arrays.fill(currSlotByIndex, -1);
        Arrays.fill(currPhaseByIndex, 0.0f);
        currCount = 0;
        frameSeq = 0;
        trackedBotUuids.clear();
    }

    public synchronized boolean isTrackedBotUuid(UUID uuid) {
        return uuid != null && trackedBotUuids.contains(uuid);
    }

    public synchronized int totalBots() {
        return currCount;
    }

    public synchronized void enqueue(byte[] frame) {
        if (frame == null || frame.length < 32) {
            return;
        }
        while (pendingFrames.size() >= MAX_QUEUE) {
            pendingFrames.removeFirst();
        }
        pendingFrames.addLast(frame);
    }

    /**
     * Advances interpolation endpoints once per client tick cleanly aligned with Mojang's
     * Entity.tick() + DeltaTracker clock, preventing mid-frame jumps and idle wiggle.
     */
    public synchronized void onClientTick() {
        for (int i = 0; i < currCount; i++) {
            currPrevRx[i] = currRx[i];
            currPrevRy[i] = currRy[i];
            currPrevRz[i] = currRz[i];
            currPrevYaw[i] = currYaw[i];
            currUnwrappedYaw[i] = currYaw[i];
            currPrevPhase[i] = currCurrPhase[i];
        }
        if (pendingFrames.isEmpty()) {
            return;
        }
        int seq = ++frameSeq;
        while (!pendingFrames.isEmpty()) {
            applyFrame(pendingFrames.removeFirst(), seq);
        }
    }

    private void applyFrame(byte[] frame, int seq) {
        if (frame.length < 32) {
            return;
        }
        ByteBuffer bb = ByteBuffer.wrap(frame).order(ByteOrder.LITTLE_ENDIAN);
        bb.getInt(); // tick
        int countWord = bb.getInt();
        boolean isDelta = (countWord & DELTA_FLAG) != 0;
        int count = Math.max(0, Math.min(MAX_INDEX, countWord & 0x7FFF_FFFF));
        double ox = bb.getDouble();
        double oy = bb.getDouble();
        double oz = bb.getDouble();
        if (frame.length < 32 + count * 20) {
            return;
        }

        if (isDelta) {
            if (count == 0) {
                return;
            }
            float deltaOriginShiftX = (float) (ox - currOriginX);
            float deltaOriginShiftY = (float) (oy - currOriginY);
            float deltaOriginShiftZ = (float) (oz - currOriginZ);
            for (int i = 0; i < count; i++) {
                float rx = bb.getFloat() + deltaOriginShiftX;
                float ry = bb.getFloat() + deltaOriginShiftY;
                float rz = bb.getFloat() + deltaOriginShiftZ;
                int yawQ = Short.toUnsignedInt(bb.getShort());
                int pitchQ = bb.getShort();
                int botIndex = Short.toUnsignedInt(bb.getShort());
                int speedQ = Byte.toUnsignedInt(bb.get());
                int flags = Byte.toUnsignedInt(bb.get());

                int slot = botIndex < MAX_INDEX ? currSlotByIndex[botIndex] : -1;
                if (slot < 0 || slot >= currCount) {
                    continue;
                }
                currSlotFrameSeq[slot] = seq;
                updateSlot(slot, botIndex, rx, ry, rz, yawQ, pitchQ, speedQ, flags, false);
            }
            return;
        }

        // Snapshot prior tick state in case the array shrinks, grows, or reorders on a full frame.
        Arrays.fill(prevSlotByIndex, -1);
        double oldOriginX = currOriginX;
        double oldOriginY = currOriginY;
        double oldOriginZ = currOriginZ;
        int oldCount = currCount;
        if (snapRx.length < oldCount) {
            int cap = Math.max(oldCount, 1024);
            snapRx = new float[cap];
            snapRy = new float[cap];
            snapRz = new float[cap];
            snapYaw = new float[cap];
        }
        for (int i = 0; i < oldCount; i++) {
            snapRx[i] = currPrevRx[i];
            snapRy[i] = currPrevRy[i];
            snapRz[i] = currPrevRz[i];
            snapYaw[i] = currPrevYaw[i];
            int bi = currIndices[i];
            if (bi >= 0 && bi < MAX_INDEX) {
                prevSlotByIndex[bi] = i;
            }
        }

        ensureCurrCapacity(count);
        Arrays.fill(currSlotByIndex, -1);

        float originShiftX = oldCount > 0 ? (float) (oldOriginX - ox) : 0.0f;
        float originShiftY = oldCount > 0 ? (float) (oldOriginY - oy) : 0.0f;
        float originShiftZ = oldCount > 0 ? (float) (oldOriginZ - oz) : 0.0f;

        currOriginX = ox;
        currOriginY = oy;
        currOriginZ = oz;
        currCount = count;
        trackedBotUuids.clear();

        for (int i = 0; i < count; i++) {
            float rx = bb.getFloat();
            float ry = bb.getFloat();
            float rz = bb.getFloat();
            int yawQ = Short.toUnsignedInt(bb.getShort());
            int pitchQ = bb.getShort();
            int botIndex = Short.toUnsignedInt(bb.getShort());
            int speedQ = Byte.toUnsignedInt(bb.get());
            int flags = Byte.toUnsignedInt(bb.get());

            int prevSlot = botIndex < MAX_INDEX ? prevSlotByIndex[botIndex] : -1;
            if (prevSlot >= 0) {
                currPrevRx[i] = snapRx[prevSlot] + originShiftX;
                currPrevRy[i] = snapRy[prevSlot] + originShiftY;
                currPrevRz[i] = snapRz[prevSlot] + originShiftZ;
                currPrevYaw[i] = snapYaw[prevSlot];
                currPrevPhase[i] = botIndex < MAX_INDEX ? currPhaseByIndex[botIndex] : 0.0f;
            } else {
                float yRad = (yawQ / 65536.0f) * TWO_PI;
                currPrevRx[i] = rx;
                currPrevRy[i] = ry;
                currPrevRz[i] = rz;
                currPrevYaw[i] = yRad;
                currPrevPhase[i] = 0.0f;
            }
            if (botIndex < MAX_INDEX) {
                currSlotByIndex[botIndex] = i;
            }
            UUID botUuid = uuidFor(botIndex);
            if (botUuid != null) {
                trackedBotUuids.add(botUuid);
            }
            currIndices[i] = botIndex;
            currSlotFrameSeq[i] = seq;
            updateSlot(i, botIndex, rx, ry, rz, yawQ, pitchQ, speedQ, flags, prevSlot < 0);
        }
    }

    private void updateSlot(
        int slot, int botIndex,
        float rx, float ry, float rz,
        int yawQ, int pitchQ, int speedQ, int flags,
        boolean resetPrevPhase
    ) {
        float yawRad = (yawQ / 65536.0f) * TWO_PI;
        float pitchRad = (pitchQ / 32767.0f) * (PI * 0.5f);
        float speed = speedQ * (1.0f / 200.0f);
        boolean onGround = (flags & 1) != 0;

        float pYaw = currPrevYaw[slot];
        float diff = yawRad - pYaw;
        while (diff > PI) diff -= TWO_PI;
        while (diff < -PI) diff += TWO_PI;
        float unwrappedYaw = pYaw + diff;

        float pPhase = resetPrevPhase ? 0.0f : currPrevPhase[slot];
        float cPhase = pPhase + speed * 2.6f;
        while (pPhase >= TWO_PI && cPhase >= TWO_PI) {
            pPhase -= TWO_PI;
            cPhase -= TWO_PI;
        }
        if (botIndex < MAX_INDEX) {
            currPhaseByIndex[botIndex] = cPhase;
        }

        float swingAmp = (speed < 0.01f && onGround)
            ? 0.0f
            : (onGround ? Math.min(1.0f, speed * 3.8f) : Math.min(0.6f, speed * 2.2f));

        currRx[slot] = rx;
        currRy[slot] = ry;
        currRz[slot] = rz;
        currYaw[slot] = yawRad;
        currUnwrappedYaw[slot] = unwrappedYaw;
        currPitch[slot] = pitchRad;
        currSwingAmp[slot] = swingAmp;
        currPrevPhase[slot] = pPhase;
        currCurrPhase[slot] = cPhase;
        currSkinLayer[slot] = skinLayerFor(botIndex);
        currFlags[slot] = flags;
    }

    private UUID uuidFor(int botIndex) {
        if (botIndex < 0 || botIndex >= MAX_INDEX) {
            return null;
        }
        UUID cached = uuidCache[botIndex];
        if (cached != null) {
            return cached;
        }
        String name = botIndex == 0 ? "PathBot" : String.format(Locale.ROOT, "Swarm%04d", botIndex);
        UUID uuid = UUID.nameUUIDFromBytes(("OfflinePlayer:" + name).getBytes(StandardCharsets.UTF_8));
        uuidCache[botIndex] = uuid;
        return uuid;
    }

    private float skinLayerFor(int botIndex) {
        if (botIndex < 0 || botIndex >= MAX_INDEX) {
            return 15.0f; // Mojang DEFAULT_SKINS[15] = wide/steve
        }
        byte cached = skinCache[botIndex];
        if (cached >= 0) {
            return (float) cached;
        }
        UUID uuid = uuidFor(botIndex);
        int idx = Math.floorMod(uuid.hashCode(), 18);
        skinCache[botIndex] = (byte) idx;
        return (float) idx;
    }

    private void ensureCurrCapacity(int count) {
        if (currRx.length >= count) {
            return;
        }
        int cap = Math.max(count, 1024);
        currPrevRx = new float[cap];
        currPrevRy = new float[cap];
        currPrevRz = new float[cap];
        currRx = new float[cap];
        currRy = new float[cap];
        currRz = new float[cap];
        currYaw = new float[cap];
        currPrevYaw = new float[cap];
        currUnwrappedYaw = new float[cap];
        currPitch = new float[cap];
        currSwingAmp = new float[cap];
        currPrevPhase = new float[cap];
        currCurrPhase = new float[cap];
        currSkinLayer = new float[cap];
        currFlags = new int[cap];
        currIndices = new int[cap];
        currSlotFrameSeq = new int[cap];
        ensureBucketCapacity(cap);
    }

    private void ensureBucketCapacity(int need) {
        if (bucket0Wide.length >= need) {
            return;
        }
        int cap = Math.max(need, bucket0Wide.length * 2);
        bucket0Wide = new int[cap];
        bucket0Slim = new int[cap];
        bucket1Wide = new int[cap];
        bucket1Slim = new int[cap];
        bucket2 = new int[cap];
    }

    private void ensureGpuCapacity(int count) {
        if (gpuCapacityBots >= count) {
            return;
        }
        int newCap = Math.max(count, gpuCapacityBots * 2);
        MemoryUtil.memFree(gpuBytes);
        gpuBytes = MemoryUtil.memAlloc(newCap * INSTANCE_BYTES).order(ByteOrder.LITTLE_ENDIAN);
        gpuBuf = gpuBytes.asFloatBuffer();
        gpuCapacityBots = newCap;
    }

    private boolean isVisibleInFrustum(float rcx, float rcy, float rcz) {
        if (pNear.x * rcx + pNear.y * rcy + pNear.z * rcz + pNear.w < -BOT_BOUNDING_RADIUS) return false;
        if (pLeft.x * rcx + pLeft.y * rcy + pLeft.z * rcz + pLeft.w < -BOT_BOUNDING_RADIUS) return false;
        if (pRight.x * rcx + pRight.y * rcy + pRight.z * rcz + pRight.w < -BOT_BOUNDING_RADIUS) return false;
        if (pBottom.x * rcx + pBottom.y * rcy + pBottom.z * rcz + pBottom.w < -BOT_BOUNDING_RADIUS) return false;
        return !(pTop.x * rcx + pTop.y * rcy + pTop.z * rcz + pTop.w < -BOT_BOUNDING_RADIUS);
    }

    private void sampleLightUv(ClientLevel level, double wx, double wy, double wz, float[] outUv) {
        if (level != null) {
            int bx = Mth.floor(wx);
            int by = Mth.floor(wy + 1.0);
            int bz = Mth.floor(wz);
            if (level.hasChunk(bx >> 4, bz >> 4)) {
                lightPos.set(bx, by, bz);
                int blockLight = level.getBrightness(LightLayer.BLOCK, lightPos);
                int skyLight = level.getBrightness(LightLayer.SKY, lightPos);
                outUv[0] = (blockLight + 0.5f) / 16.0f;
                outUv[1] = (skyLight + 0.5f) / 16.0f;
                return;
            }
        }
        outUv[0] = 0.5f / 16.0f;
        outUv[1] = 15.5f / 16.0f;
    }

    /**
     * Culls all virtual bots and live client RemotePlayers against the camera frustum, bins into
     * 5 adaptive GPU instanced buckets (LOD0/LOD1 WIDE+SLIM, LOD2 silhouette), and outputs 48 B/bot.
     */
    public synchronized int buildVisibleInstances(
        ClientLevel level,
        double camX,
        double camY,
        double camZ,
        float partialTick,
        Matrix4f viewProj,
        List<AbstractClientPlayer> liveBotPlayers,
        Set<UUID> liveClientBotUuids,
        float[] outOriginRelCam,
        int[] outBucketCounts,
        ByteBuffer[] outBufRef
    ) {
        Arrays.fill(outBucketCounts, 0);
        int liveCount = liveBotPlayers != null ? liveBotPlayers.size() : 0;
        if (currCount <= 0 && liveCount == 0) {
            return 0;
        }
        ensureBucketCapacity(currCount + liveCount + 16);

        double baseOriginX = currCount > 0 ? currOriginX : Math.floor(liveBotPlayers.getFirst().getX());
        double baseOriginY = currCount > 0 ? currOriginY : Math.floor(liveBotPlayers.getFirst().getY());
        double baseOriginZ = currCount > 0 ? currOriginZ : Math.floor(liveBotPlayers.getFirst().getZ());

        float originX = (float) (baseOriginX - camX);
        float originY = (float) (baseOriginY - camY);
        float originZ = (float) (baseOriginZ - camZ);
        outOriginRelCam[0] = originX;
        outOriginRelCam[1] = originY;
        outOriginRelCam[2] = originZ;

        float alpha = Mth.clamp(partialTick, 0.0f, 1.0f);

        viewProj.frustumPlane(Matrix4f.PLANE_NX, pLeft);
        viewProj.frustumPlane(Matrix4f.PLANE_PX, pRight);
        viewProj.frustumPlane(Matrix4f.PLANE_NY, pBottom);
        viewProj.frustumPlane(Matrix4f.PLANE_PY, pTop);
        pNear.set(viewProj.m03(), viewProj.m13(), viewProj.m23(), viewProj.m33()).normalize3();

        int c0Wide = 0, c0Slim = 0, c1Wide = 0, c1Slim = 0, c2 = 0;

        for (int i = 0; i < liveCount; i++) {
            AbstractClientPlayer p = liveBotPlayers.get(i);
            Vec3 pos = p.getPosition(alpha);
            float bx = (float) (pos.x - baseOriginX);
            float by = (float) (pos.y - baseOriginY);
            float bz = (float) (pos.z - baseOriginZ);
            float rcx = originX + bx;
            float rcy = originY + by + 0.9f;
            float rcz = originZ + bz;
            if (!isVisibleInFrustum(rcx, rcy, rcz)) {
                continue;
            }
            boolean isWide = Math.floorMod(p.getUUID().hashCode(), 18) >= 9;
            float distSq = rcx * rcx + rcy * rcy + rcz * rcz;
            int tagged = i;
            if (distSq < LOD0_MAX_DIST_SQ) {
                if (isWide) bucket0Wide[c0Wide++] = tagged;
                else bucket0Slim[c0Slim++] = tagged;
            } else if (distSq < LOD1_MAX_DIST_SQ) {
                if (isWide) bucket1Wide[c1Wide++] = tagged;
                else bucket1Slim[c1Slim++] = tagged;
            } else {
                bucket2[c2++] = tagged;
            }
        }

        int seq = frameSeq;
        for (int i = 0; i < currCount; i++) {
            UUID botUuid = uuidFor(currIndices[i]);
            if (liveClientBotUuids != null && botUuid != null && liveClientBotUuids.contains(botUuid)) {
                continue;
            }
            float effAlpha = (currSlotFrameSeq[i] == seq) ? alpha : 1.0f;
            float bx = currPrevRx[i] + (currRx[i] - currPrevRx[i]) * effAlpha;
            float by = currPrevRy[i] + (currRy[i] - currPrevRy[i]) * effAlpha;
            float bz = currPrevRz[i] + (currRz[i] - currPrevRz[i]) * effAlpha;

            float rcx = originX + bx;
            float rcy = originY + by + 0.9f;
            float rcz = originZ + bz;
            if (!isVisibleInFrustum(rcx, rcy, rcz)) {
                continue;
            }

            boolean isWide = currSkinLayer[i] >= 8.5f;
            float distSq = rcx * rcx + rcy * rcy + rcz * rcz;
            int tagged = i | SOURCE_VIRTUAL_BIT;
            if (distSq < LOD0_MAX_DIST_SQ) {
                if (isWide) bucket0Wide[c0Wide++] = tagged;
                else bucket0Slim[c0Slim++] = tagged;
            } else if (distSq < LOD1_MAX_DIST_SQ) {
                if (isWide) bucket1Wide[c1Wide++] = tagged;
                else bucket1Slim[c1Slim++] = tagged;
            } else {
                bucket2[c2++] = tagged;
            }
        }

        int totalDrawn = c0Wide + c0Slim + c1Wide + c1Slim + c2;
        if (totalDrawn == 0) {
            return 0;
        }
        ensureGpuCapacity(totalDrawn);
        gpuBuf.clear();

        float[] lightUv = new float[2];
        int cursor = 0;
        cursor = writeBucket(bucket0Wide, c0Wide, true, level, baseOriginX, baseOriginY, baseOriginZ, alpha, seq, liveBotPlayers, lightUv, cursor);
        cursor = writeBucket(bucket0Slim, c0Slim, true, level, baseOriginX, baseOriginY, baseOriginZ, alpha, seq, liveBotPlayers, lightUv, cursor);
        cursor = writeBucket(bucket1Wide, c1Wide, false, level, baseOriginX, baseOriginY, baseOriginZ, alpha, seq, liveBotPlayers, lightUv, cursor);
        cursor = writeBucket(bucket1Slim, c1Slim, false, level, baseOriginX, baseOriginY, baseOriginZ, alpha, seq, liveBotPlayers, lightUv, cursor);
        cursor = writeBucket(bucket2, c2, false, level, baseOriginX, baseOriginY, baseOriginZ, alpha, seq, liveBotPlayers, lightUv, cursor);

        gpuBytes.position(0);
        gpuBytes.limit(totalDrawn * INSTANCE_BYTES);
        outBucketCounts[BUCKET_LOD0_WIDE] = c0Wide;
        outBucketCounts[BUCKET_LOD0_SLIM] = c0Slim;
        outBucketCounts[BUCKET_LOD1_WIDE] = c1Wide;
        outBucketCounts[BUCKET_LOD1_SLIM] = c1Slim;
        outBucketCounts[BUCKET_LOD2] = c2;
        outBufRef[0] = gpuBytes;
        return totalDrawn;
    }

    private int writeBucket(
        int[] bucket,
        int count,
        boolean articulated,
        ClientLevel level,
        double baseOriginX,
        double baseOriginY,
        double baseOriginZ,
        float alpha,
        int seq,
        List<AbstractClientPlayer> liveBotPlayers,
        float[] lightUv,
        int cursor
    ) {
        for (int k = 0; k < count; k++) {
            int entry = bucket[k];
            int idx = entry & SOURCE_INDEX_MASK;
            boolean isVirtual = (entry & SOURCE_VIRTUAL_BIT) != 0;

            float bx, by, bz;
            float skinIdx, cosBodyYaw, sinBodyYaw, headPitch, legSwing, headRelYaw, deathRoll;
            boolean sneaking, hurtRed;

            if (!isVirtual) {
                AbstractClientPlayer p = liveBotPlayers.get(idx);
                Vec3 pos = p.getPosition(alpha);
                bx = (float) (pos.x - baseOriginX);
                by = (float) (pos.y - baseOriginY);
                bz = (float) (pos.z - baseOriginZ);

                float bodyYawDeg = Mth.rotLerp(alpha, p.yBodyRotO, p.yBodyRot);
                float headYawDeg = Mth.rotLerp(alpha, p.yHeadRotO, p.yHeadRot);
                float bodyYawRad = (float) Math.toRadians(bodyYawDeg);
                cosBodyYaw = (float) Math.cos(bodyYawRad);
                sinBodyYaw = (float) Math.sin(bodyYawRad);
                headRelYaw = articulated ? (float) Math.toRadians(Mth.wrapDegrees(headYawDeg - bodyYawDeg)) : 0.0f;
                headPitch = articulated ? (float) Math.toRadians(Mth.lerp(alpha, p.xRotO, p.getXRot())) : 0.0f;

                if (articulated) {
                    float walkPos = p.walkAnimation.position(alpha);
                    float walkSpeed = Math.min(1.0f, p.walkAnimation.speed(alpha));
                    legSwing = Mth.cos(walkPos * 0.6662f) * 1.4f * walkSpeed;
                } else {
                    legSwing = 0.0f;
                }

                sneaking = p.isCrouching();
                hurtRed = p.hurtTime > 0 || p.deathTime > 0;
                if (p.deathTime > 0) {
                    float f = Mth.sqrt(Math.max(0.0f, ((p.deathTime + alpha - 1.0f) / 20.0f) * 1.6f));
                    deathRoll = Math.min(1.0f, f) * (PI * 0.5f);
                } else {
                    deathRoll = 0.0f;
                }
                skinIdx = Math.floorMod(p.getUUID().hashCode(), 18);
                sampleLightUv(level, pos.x, pos.y, pos.z, lightUv);
            } else {
                float effAlpha = (currSlotFrameSeq[idx] == seq) ? alpha : 1.0f;
                bx = currPrevRx[idx] + (currRx[idx] - currPrevRx[idx]) * effAlpha;
                by = currPrevRy[idx] + (currRy[idx] - currPrevRy[idx]) * effAlpha;
                bz = currPrevRz[idx] + (currRz[idx] - currPrevRz[idx]) * effAlpha;

                float yaw = currPrevYaw[idx] + (currUnwrappedYaw[idx] - currPrevYaw[idx]) * effAlpha;
                cosBodyYaw = (float) Math.cos(yaw);
                sinBodyYaw = (float) Math.sin(yaw);
                headRelYaw = 0.0f;
                headPitch = articulated ? currPitch[idx] : 0.0f;

                if (articulated) {
                    float amp = currSwingAmp[idx];
                    if (amp > 0.001f) {
                        float phase = currPrevPhase[idx] + (currCurrPhase[idx] - currPrevPhase[idx]) * effAlpha;
                        legSwing = (float) Math.sin(phase) * 1.15f * amp;
                    } else {
                        legSwing = 0.0f;
                    }
                } else {
                    legSwing = 0.0f;
                }

                sneaking = (currFlags[idx] & 4) != 0;
                hurtRed = false;
                deathRoll = 0.0f;
                skinIdx = currSkinLayer[idx];
                sampleLightUv(level, baseOriginX + bx, baseOriginY + by, baseOriginZ + bz, lightUv);
            }

            float packedMeta = skinIdx + (sneaking ? 32.0f : 0.0f) + (hurtRed ? 64.0f : 0.0f);
            int base = cursor * INSTANCE_FLOATS;
            gpuBuf.put(base, bx);
            gpuBuf.put(base + 1, by);
            gpuBuf.put(base + 2, bz);
            gpuBuf.put(base + 3, packedMeta);
            gpuBuf.put(base + 4, cosBodyYaw);
            gpuBuf.put(base + 5, sinBodyYaw);
            gpuBuf.put(base + 6, headPitch);
            gpuBuf.put(base + 7, legSwing);
            gpuBuf.put(base + 8, headRelYaw);
            gpuBuf.put(base + 9, deathRoll);
            gpuBuf.put(base + 10, lightUv[0]);
            gpuBuf.put(base + 11, lightUv[1]);
            cursor++;
        }
        return cursor;
    }
}
