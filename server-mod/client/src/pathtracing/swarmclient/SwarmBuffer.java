package pathtracing.swarmclient;

import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.FloatBuffer;
import java.util.Arrays;
import org.lwjgl.system.MemoryUtil;

/** Thread-safe double buffer translating 20B/bot SwarmFramePayloads into 56B GPU instance data. */
public final class SwarmBuffer {
    public static final int INSTANCE_FLOATS = 14;
    public static final int INSTANCE_BYTES = INSTANCE_FLOATS * 4;
    private static final int MAX_INDEX = 65536;
    private static final float TWO_PI = (float) (Math.PI * 2.0);
    private static final float PI = (float) Math.PI;

    private final int[] prevSlotByIndex = new int[MAX_INDEX];
    private final float[] prevPhaseByIndex = new float[MAX_INDEX];
    private final float[] currPhaseByIndex = new float[MAX_INDEX];

    private int prevCount = 0;
    private double prevOriginX, prevOriginY, prevOriginZ;
    private float[] prevRx = new float[0];
    private float[] prevRy = new float[0];
    private float[] prevRz = new float[0];
    private float[] prevYaw = new float[0];

    private int currCount = 0;
    private double currOriginX, currOriginY, currOriginZ;
    private float[] currRx = new float[0];
    private float[] currRy = new float[0];
    private float[] currRz = new float[0];
    private float[] currYaw = new float[0];
    private int[] currIndices = new int[0];

    private ByteBuffer gpuBytes = MemoryUtil.memAlloc(1024 * INSTANCE_BYTES).order(ByteOrder.LITTLE_ENDIAN);
    private FloatBuffer gpuBuf = gpuBytes.asFloatBuffer();
    private int gpuCapacityBots = 1024;
    private long lastFrameNanos = 0L;
    private long version = 0L;
    private long uploadedVersion = -1L;

    public SwarmBuffer() {
        Arrays.fill(prevSlotByIndex, -1);
    }

    public synchronized void clear() {
        Arrays.fill(prevSlotByIndex, -1);
        Arrays.fill(prevPhaseByIndex, 0.0f);
        Arrays.fill(currPhaseByIndex, 0.0f);
        prevCount = 0;
        currCount = 0;
        lastFrameNanos = 0L;
        version++;
    }

    public synchronized void ingest(byte[] frame) {
        if (frame.length < 32) {
            return;
        }
        ByteBuffer bb = ByteBuffer.wrap(frame).order(ByteOrder.LITTLE_ENDIAN);
        bb.getInt(); // tick
        int count = Math.max(0, Math.min(MAX_INDEX, bb.getInt()));
        double ox = bb.getDouble();
        double oy = bb.getDouble();
        double oz = bb.getDouble();
        if (frame.length < 32 + count * 20) {
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

        if (currRx.length < count) {
            int cap = Math.max(count, 1024);
            currRx = new float[cap];
            currRy = new float[cap];
            currRz = new float[cap];
            currYaw = new float[cap];
            currIndices = new int[cap];
        }
        if (gpuCapacityBots < count) {
            int newCap = Math.max(count, gpuCapacityBots * 2);
            MemoryUtil.memFree(gpuBytes);
            gpuBytes = MemoryUtil.memAlloc(newCap * INSTANCE_BYTES).order(ByteOrder.LITTLE_ENDIAN);
            gpuBuf = gpuBytes.asFloatBuffer();
            gpuCapacityBots = newCap;
        }

        float originShiftX = prevCount > 0 ? (float) (prevOriginX - ox) : 0.0f;
        float originShiftY = prevCount > 0 ? (float) (prevOriginY - oy) : 0.0f;
        float originShiftZ = prevCount > 0 ? (float) (prevOriginZ - oz) : 0.0f;

        currOriginX = ox;
        currOriginY = oy;
        currOriginZ = oz;
        currCount = count;

        gpuBuf.clear();
        for (int i = 0; i < count; i++) {
            float rx = bb.getFloat();
            float ry = bb.getFloat();
            float rz = bb.getFloat();
            int yawQ = Short.toUnsignedInt(bb.getShort());
            int pitchQ = bb.getShort();
            int botIndex = Short.toUnsignedInt(bb.getShort());
            int speedQ = Byte.toUnsignedInt(bb.get());
            int flags = Byte.toUnsignedInt(bb.get());

            float yawRad = (yawQ / 65536.0f) * TWO_PI;
            float pitchRad = (pitchQ / 32767.0f) * (PI * 0.5f);
            float speed = speedQ * (1.0f / 200.0f);
            boolean onGround = (flags & 1) != 0;

            float pxPos = rx;
            float pyPos = ry;
            float pzPos = rz;
            float pYaw = yawRad;
            int prevSlot = botIndex < MAX_INDEX ? prevSlotByIndex[botIndex] : -1;
            if (prevSlot >= 0) {
                pxPos = prevRx[prevSlot] + originShiftX;
                pyPos = prevRy[prevSlot] + originShiftY;
                pzPos = prevRz[prevSlot] + originShiftZ;
                pYaw = prevYaw[prevSlot];
            }

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
            int skinLayer = botIndex % 9;

            currRx[i] = rx;
            currRy[i] = ry;
            currRz[i] = rz;
            currYaw[i] = yawRad;
            currIndices[i] = botIndex;

            int base = i * INSTANCE_FLOATS;
            gpuBuf.put(base, pxPos);
            gpuBuf.put(base + 1, pyPos);
            gpuBuf.put(base + 2, pzPos);
            gpuBuf.put(base + 3, rx);
            gpuBuf.put(base + 4, ry);
            gpuBuf.put(base + 5, rz);
            gpuBuf.put(base + 6, pYaw);
            gpuBuf.put(base + 7, unwrappedYaw);
            gpuBuf.put(base + 8, pitchRad);
            gpuBuf.put(base + 9, swingAmp);
            gpuBuf.put(base + 10, pPhase);
            gpuBuf.put(base + 11, cPhase);
            gpuBuf.put(base + 12, (float) skinLayer);
            gpuBuf.put(base + 13, (float) flags);
        }
        gpuBuf.position(0);
        gpuBuf.limit(count * INSTANCE_FLOATS);
        lastFrameNanos = System.nanoTime();
        version++;
    }

    public synchronized int prepareDraw(
        double camX,
        double camY,
        double camZ,
        float[] outRelOriginAndAlpha,
        UploadFn upload
    ) {
        if (currCount <= 0) {
            return 0;
        }
        outRelOriginAndAlpha[0] = (float) (currOriginX - camX);
        outRelOriginAndAlpha[1] = (float) (currOriginY - camY);
        outRelOriginAndAlpha[2] = (float) (currOriginZ - camZ);
        float alpha = (System.nanoTime() - lastFrameNanos) * (1.0f / 50_000_000.0f);
        outRelOriginAndAlpha[3] = alpha < 0.0f ? 0.0f : Math.min(1.0f, alpha);

        if (uploadedVersion != version) {
            gpuBytes.position(0);
            gpuBytes.limit(currCount * INSTANCE_BYTES);
            upload.upload(gpuBytes, currCount);
            uploadedVersion = version;
        }
        return currCount;
    }

    public interface UploadFn {
        void upload(ByteBuffer buf, int count);
    }
}
