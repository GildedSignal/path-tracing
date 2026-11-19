package pathtracing.swarmclient;

import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.FloatBuffer;
import java.nio.charset.StandardCharsets;
import java.util.Arrays;
import java.util.HashSet;
import java.util.Locale;
import java.util.Set;
import java.util.UUID;
import org.joml.Matrix4f;
import org.joml.Vector4f;
import org.lwjgl.system.MemoryUtil;

/**
 * Thread-safe buffer ingesting full & delta SwarmFramePayloads, running frustum culling + 3-tier LOD
 * binning on CPU, and outputting precomputed-trig 32B/bot GPU instance data sorted front-to-back by LOD.
 */
public final class SwarmBuffer {
    public static final int INSTANCE_FLOATS = 8;
    public static final int INSTANCE_BYTES = INSTANCE_FLOATS * 4;
    private static final int DELTA_FLAG = 0x8000_0000;
    private static final int MAX_INDEX = 65536;
    private static final float TWO_PI = (float) (Math.PI * 2.0);
    private static final float PI = (float) Math.PI;
    private static final float LOD0_MAX_DIST_SQ = 96.0f * 96.0f;
    private static final float LOD1_MAX_DIST_SQ = 220.0f * 220.0f;
    private static final float BOT_BOUNDING_RADIUS = 2.2f;

    private final int[] prevSlotByIndex = new int[MAX_INDEX];
    private final int[] currSlotByIndex = new int[MAX_INDEX];
    private final float[] prevPhaseByIndex = new float[MAX_INDEX];
    private final float[] currPhaseByIndex = new float[MAX_INDEX];
    private final byte[] skinCache = new byte[MAX_INDEX];
    private final UUID[] uuidCache = new UUID[MAX_INDEX];
    private final Set<UUID> trackedBotUuids = new HashSet<>();
    private int frameSeq = 0;

    private int prevCount = 0;
    private double prevOriginX, prevOriginY, prevOriginZ;
    private float[] prevRx = new float[0];
    private float[] prevRy = new float[0];
    private float[] prevRz = new float[0];
    private float[] prevYaw = new float[0];

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

    private int[] bucket0 = new int[1024];
    private int[] bucket1 = new int[1024];
    private int[] bucket2 = new int[1024];

    private ByteBuffer gpuBytes = MemoryUtil.memAlloc(1024 * INSTANCE_BYTES).order(ByteOrder.LITTLE_ENDIAN);
    private FloatBuffer gpuBuf = gpuBytes.asFloatBuffer();
    private int gpuCapacityBots = 1024;
    private long lastFrameNanos = 0L;

    private final Vector4f pLeft = new Vector4f();
    private final Vector4f pRight = new Vector4f();
    private final Vector4f pBottom = new Vector4f();
    private final Vector4f pTop = new Vector4f();
    private final Vector4f pNear = new Vector4f();

    public SwarmBuffer() {
        Arrays.fill(prevSlotByIndex, -1);
        Arrays.fill(currSlotByIndex, -1);
        Arrays.fill(skinCache, (byte) -1);
    }

    public synchronized void clear() {
        Arrays.fill(prevSlotByIndex, -1);
        Arrays.fill(currSlotByIndex, -1);
        Arrays.fill(prevPhaseByIndex, 0.0f);
        Arrays.fill(currPhaseByIndex, 0.0f);
        prevCount = 0;
        currCount = 0;
        frameSeq = 0;
        lastFrameNanos = 0L;
        trackedBotUuids.clear();
    }

    public synchronized boolean isTrackedBotUuid(UUID uuid) {
        return uuid != null && trackedBotUuids.contains(uuid);
    }

    public synchronized int totalBots() {
        return currCount;
    }

    public synchronized void ingest(byte[] frame) {
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
            int seq = ++frameSeq;
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
                updateSlot(slot, botIndex, rx, ry, rz, yawQ, pitchQ, speedQ, flags,
                    currRx[slot], currRy[slot], currRz[slot], currYaw[slot]);
            }
            lastFrameNanos = System.nanoTime();
            return;
        }

        // Snapshot old curr -> prev using bot_index slot map.
        Arrays.fill(prevSlotByIndex, -1);
        prevOriginX = currOriginX;
        prevOriginY = currOriginY;
        prevOriginZ = currOriginZ;
        prevCount = currCount;
        if (prevRx.length < prevCount) {
            int cap = Math.max(prevCount, 1024);
            prevRx = new float[cap];
            prevRy = new float[cap];
            prevRz = new float[cap];
            prevYaw = new float[cap];
        }
        for (int i = 0; i < prevCount; i++) {
            prevRx[i] = currRx[i];
            prevRy[i] = currRy[i];
            prevRz[i] = currRz[i];
            prevYaw[i] = currYaw[i];
            int bi = currIndices[i];
            if (bi >= 0 && bi < MAX_INDEX) {
                prevSlotByIndex[bi] = i;
            }
        }

        ensureCurrCapacity(count);
        Arrays.fill(currSlotByIndex, -1);
        int seq = ++frameSeq;

        float originShiftX = prevCount > 0 ? (float) (prevOriginX - ox) : 0.0f;
        float originShiftY = prevCount > 0 ? (float) (prevOriginY - oy) : 0.0f;
        float originShiftZ = prevCount > 0 ? (float) (prevOriginZ - oz) : 0.0f;

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

            float pxPos = rx;
            float pyPos = ry;
            float pzPos = rz;
            float pYaw = (yawQ / 65536.0f) * TWO_PI;
            int prevSlot = botIndex < MAX_INDEX ? prevSlotByIndex[botIndex] : -1;
            if (prevSlot >= 0) {
                pxPos = prevRx[prevSlot] + originShiftX;
                pyPos = prevRy[prevSlot] + originShiftY;
                pzPos = prevRz[prevSlot] + originShiftZ;
                pYaw = prevYaw[prevSlot];
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
            updateSlot(i, botIndex, rx, ry, rz, yawQ, pitchQ, speedQ, flags, pxPos, pyPos, pzPos, pYaw);
        }
        lastFrameNanos = System.nanoTime();
    }

    private void updateSlot(
        int slot, int botIndex,
        float rx, float ry, float rz,
        int yawQ, int pitchQ, int speedQ, int flags,
        float pxPos, float pyPos, float pzPos, float pYaw
    ) {
        float yawRad = (yawQ / 65536.0f) * TWO_PI;
        float pitchRad = (pitchQ / 32767.0f) * (PI * 0.5f);
        float speed = speedQ * (1.0f / 200.0f);
        boolean onGround = (flags & 1) != 0;

        float diff = yawRad - pYaw;
        while (diff > PI) diff -= TWO_PI;
        while (diff < -PI) diff += TWO_PI;
        float unwrappedYaw = pYaw + diff;

        float pPhase = botIndex < MAX_INDEX ? currPhaseByIndex[botIndex] : 0.0f;
        float cPhase = pPhase + speed * 2.6f;
        while (pPhase >= TWO_PI && cPhase >= TWO_PI) {
            pPhase -= TWO_PI;
            cPhase -= TWO_PI;
        }
        if (botIndex < MAX_INDEX) {
            prevPhaseByIndex[botIndex] = pPhase;
            currPhaseByIndex[botIndex] = cPhase;
        }

        float swingAmp = (speed < 0.01f && onGround)
            ? 0.0f
            : (onGround ? Math.min(1.0f, speed * 3.8f) : Math.min(0.6f, speed * 2.2f));

        currPrevRx[slot] = pxPos;
        currPrevRy[slot] = pyPos;
        currPrevRz[slot] = pzPos;
        currRx[slot] = rx;
        currRy[slot] = ry;
        currRz[slot] = rz;
        currYaw[slot] = yawRad;
        currPrevYaw[slot] = pYaw;
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
            return 6.0f;
        }
        byte cached = skinCache[botIndex];
        if (cached >= 0) {
            return (float) cached;
        }
        UUID uuid = uuidFor(botIndex);
        int idx = Math.floorMod(uuid.hashCode(), 18) % 9;
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
        bucket0 = new int[cap];
        bucket1 = new int[cap];
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

    /**
     * Culls against the camera frustum, bins into 3 distance LODs, and writes 32 B/bot sorted
     * front-to-back into a reusable native ByteBuffer.
     */
    public synchronized int buildVisibleInstances(
        double camX,
        double camY,
        double camZ,
        Matrix4f viewProj,
        Set<UUID> liveClientBotUuids,
        float[] outOriginRelCam,
        int[] outLodCounts,
        ByteBuffer[] outBufRef
    ) {
        outLodCounts[0] = 0;
        outLodCounts[1] = 0;
        outLodCounts[2] = 0;
        if (currCount <= 0) {
            return 0;
        }

        float originX = (float) (currOriginX - camX);
        float originY = (float) (currOriginY - camY);
        float originZ = (float) (currOriginZ - camZ);
        outOriginRelCam[0] = originX;
        outOriginRelCam[1] = originY;
        outOriginRelCam[2] = originZ;

        float rawAlpha = (System.nanoTime() - lastFrameNanos) * (1.0f / 50_000_000.0f);
        float alpha = rawAlpha < 0.0f ? 0.0f : Math.min(1.0f, rawAlpha);

        viewProj.frustumPlane(Matrix4f.PLANE_NX, pLeft);
        viewProj.frustumPlane(Matrix4f.PLANE_PX, pRight);
        viewProj.frustumPlane(Matrix4f.PLANE_NY, pBottom);
        viewProj.frustumPlane(Matrix4f.PLANE_PY, pTop);
        pNear.set(viewProj.m03(), viewProj.m13(), viewProj.m23(), viewProj.m33()).normalize3();

        int seq = frameSeq;
        int c0 = 0, c1 = 0, c2 = 0;
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

            if (pNear.x * rcx + pNear.y * rcy + pNear.z * rcz + pNear.w < -BOT_BOUNDING_RADIUS) continue;
            if (pLeft.x * rcx + pLeft.y * rcy + pLeft.z * rcz + pLeft.w < -BOT_BOUNDING_RADIUS) continue;
            if (pRight.x * rcx + pRight.y * rcy + pRight.z * rcz + pRight.w < -BOT_BOUNDING_RADIUS) continue;
            if (pBottom.x * rcx + pBottom.y * rcy + pBottom.z * rcz + pBottom.w < -BOT_BOUNDING_RADIUS) continue;
            if (pTop.x * rcx + pTop.y * rcy + pTop.z * rcz + pTop.w < -BOT_BOUNDING_RADIUS) continue;

            float distSq = rcx * rcx + rcy * rcy + rcz * rcz;
            if (distSq < LOD0_MAX_DIST_SQ) {
                bucket0[c0++] = i;
            } else if (distSq < LOD1_MAX_DIST_SQ) {
                bucket1[c1++] = i;
            } else {
                bucket2[c2++] = i;
            }
        }

        int totalDrawn = c0 + c1 + c2;
        if (totalDrawn == 0) {
            return 0;
        }
        ensureGpuCapacity(totalDrawn);
        gpuBuf.clear();

        int cursor = 0;
        cursor = writeBucket(bucket0, c0, alpha, seq, true, cursor);
        cursor = writeBucket(bucket1, c1, alpha, seq, false, cursor);
        cursor = writeBucket(bucket2, c2, alpha, seq, false, cursor);

        gpuBytes.position(0);
        gpuBytes.limit(totalDrawn * INSTANCE_BYTES);
        outLodCounts[0] = c0;
        outLodCounts[1] = c1;
        outLodCounts[2] = c2;
        outBufRef[0] = gpuBytes;
        return totalDrawn;
    }

    private int writeBucket(int[] bucket, int count, float alpha, int seq, boolean articulated, int cursor) {
        for (int k = 0; k < count; k++) {
            int i = bucket[k];
            float effAlpha = (currSlotFrameSeq[i] == seq) ? alpha : 1.0f;
            float bx = currPrevRx[i] + (currRx[i] - currPrevRx[i]) * effAlpha;
            float by = currPrevRy[i] + (currRy[i] - currPrevRy[i]) * effAlpha;
            float bz = currPrevRz[i] + (currRz[i] - currPrevRz[i]) * effAlpha;
            boolean sneaking = (currFlags[i] & 4) != 0;
            float byAdj = sneaking ? (by - 0.125f) : by;
            float skinMeta = currSkinLayer[i] + (sneaking ? 16.0f : 0.0f);

            float yaw = currPrevYaw[i] + (currUnwrappedYaw[i] - currPrevYaw[i]) * effAlpha;
            float cosYaw = (float) Math.cos(yaw);
            float sinYaw = (float) Math.sin(yaw);

            float headPitch = 0.0f;
            float legSwing = 0.0f;
            if (articulated) {
                headPitch = currPitch[i];
                float amp = currSwingAmp[i];
                if (amp > 0.001f) {
                    float phase = currPrevPhase[i] + (currCurrPhase[i] - currPrevPhase[i]) * effAlpha;
                    legSwing = (float) Math.sin(phase) * 0.72f * amp;
                }
            }

            int base = cursor * INSTANCE_FLOATS;
            gpuBuf.put(base, bx);
            gpuBuf.put(base + 1, byAdj);
            gpuBuf.put(base + 2, bz);
            gpuBuf.put(base + 3, skinMeta);
            gpuBuf.put(base + 4, cosYaw);
            gpuBuf.put(base + 5, sinYaw);
            gpuBuf.put(base + 6, headPitch);
            gpuBuf.put(base + 7, legSwing);
            cursor++;
        }
        return cursor;
    }
}
