package pathtracing.server;

import java.io.IOException;
import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.lang.foreign.ValueLayout;
import java.lang.invoke.VarHandle;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.channels.FileChannel;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardOpenOption;

/** Windows memory-mapped shared memory and SPSC ring bridge to Rust `mind.exe`. */
public final class ShmBridge implements AutoCloseable {
    public static final long MAGIC = 0x5054_4D49_4E44_3031L; // "PTMIND01"
    public static final long VERSION = 1L;

    public static final int S2E_OFFSET = 4096;
    public static final int S2E_CAP = 16 * 1024 * 1024;
    public static final int E2S_OFFSET = S2E_OFFSET + S2E_CAP;
    public static final int E2S_CAP = 4 * 1024 * 1024;
    public static final long TOTAL_SIZE = (long) E2S_OFFSET + (long) E2S_CAP;

    private static final long OFF_MAGIC = 0L;
    private static final long OFF_VERSION = 8L;
    private static final long OFF_SERVER_EPOCH = 16L;
    private static final long OFF_ENGINE_EPOCH = 24L;
    private static final long OFF_ENGINE_HEARTBEAT_MS = 32L;
    private static final long OFF_SERVER_TICK_SEQ = 40L;
    private static final long OFF_ENGINE_TICK_SEQ = 48L;

    private static final long OFF_S2E_HEAD = 64L;
    private static final long OFF_S2E_TAIL = 72L;
    private static final long OFF_S2E_CAP = 80L;
    private static final long OFF_S2E_OFFSET = 88L;

    private static final long OFF_E2S_HEAD = 128L;
    private static final long OFF_E2S_TAIL = 136L;
    private static final long OFF_E2S_CAP = 144L;
    private static final long OFF_E2S_OFFSET = 152L;

    public static final short MSG_WORLD_RESET = 1;
    public static final short MSG_WORLD_RULES = 2;
    public static final short MSG_SECTION_AIR = 3;
    public static final short MSG_SECTION_RLE = 4;
    public static final short MSG_BLOCK_DELTA = 5;
    public static final short MSG_CHUNK_UNLOAD = 6;

    public static final short MSG_BOT_SPAWN = 10;
    public static final short MSG_BOT_REMOVE = 11;
    public static final short MSG_BOT_SELF_SYNC = 12;
    public static final short MSG_BOT_GOAL = 13;
    public static final short MSG_BOT_STOP = 14;
    public static final short MSG_SWARM_SPAWN_BATCH = 15;
    public static final short MSG_SWARM_CLEAR = 16;
    public static final short MSG_SWARM_GOAL_BATCH = 17;
    public static final short MSG_SWARM_STOP_ALL = 18;
    public static final short MSG_OBSERVERS = 19;
    public static final short MSG_TICK_BEGIN = 20;
    public static final short MSG_SWARM_CONFIG = 21;
    public static final short MSG_RENDER_STREAM = 22;
    public static final short MSG_SWARM_POSTURE = 23;

    public static final int FORM_GRID = 0;
    public static final int FORM_POINT = 1;
    public static final int FORM_WEDGE = 2;
    public static final int FORM_QUAD = 3;
    public static final int FORM_CITADEL = 4;
    public static final int FORM_CORRIDOR = 5;

    public static final int POSTURE_STARE = 1 << 0;
    public static final int POSTURE_SALUTE = 1 << 1;

    public static final int MSG_BOT_PLAN_STATUS = 100;
    public static final int MSG_BOT_TICK_CMD = 101;
    public static final int MSG_BOT_TIER_CHANGE = 102;
    public static final int MSG_VISUAL_BATCH = 103;
    public static final int MSG_CHUNK_TICKET = 104;
    public static final int MSG_SWARM_PLAN_SUMMARY = 105;
    public static final int MSG_RENDER_FRAME = 106;
    public static final int MSG_TICK_END = 120;

    public static final int TIER_VIRTUAL = 0;
    public static final int TIER_VISUAL = 1;
    public static final int TIER_MATERIALIZED = 2;

    public static final int MODE_VIRTUAL = 0;
    public static final int MODE_AUTO = 1;
    public static final int MODE_MATERIALIZED = 2;

    public static final int PLAN_STATUS_READY = 0;
    public static final int PLAN_STATUS_ALREADY_THERE = 1;
    public static final int PLAN_STATUS_NEEDS_JOINTS = 2;
    public static final int PLAN_STATUS_REFUSED = 3;

    public static final int FLAG_ON_GROUND = 1 << 0;
    public static final int FLAG_HORIZ_COL = 1 << 1;
    public static final int FLAG_START_SPRINT = 1 << 2;
    public static final int FLAG_STOP_SPRINT = 1 << 3;
    public static final int FLAG_ARRIVED = 1 << 4;
    public static final int FLAG_SETTLED = 1 << 5;

    private static final ValueLayout.OfLong LONG_LE = ValueLayout.JAVA_LONG.withOrder(ByteOrder.LITTLE_ENDIAN);
    private static final VarHandle VH_LONG = LONG_LE.varHandle();

    private final FileChannel channel;
    private final MemorySegment segment;
    private long lastEngineEpoch;
    private byte[] rxBuf = new byte[65536];

    private ShmBridge(FileChannel channel, MemorySegment segment) {
        this.channel = channel;
        this.segment = segment;
        initServer();
    }

    public static ShmBridge open(Path path) throws IOException {
        Path parent = path.getParent();
        if (parent != null) {
            Files.createDirectories(parent);
        }
        FileChannel ch = FileChannel.open(
            path,
            StandardOpenOption.CREATE,
            StandardOpenOption.READ,
            StandardOpenOption.WRITE
        );
        if (ch.size() < TOTAL_SIZE) {
            ch.write(ByteBuffer.allocate(1), TOTAL_SIZE - 1);
        }
        MemorySegment seg = ch.map(FileChannel.MapMode.READ_WRITE, 0L, TOTAL_SIZE, Arena.global());
        return new ShmBridge(ch, seg);
    }

    private void initServer() {
        long magic = (long) VH_LONG.getAcquire(segment, OFF_MAGIC);
        long ver = (long) VH_LONG.getAcquire(segment, OFF_VERSION);
        long s2eCap = (long) VH_LONG.getAcquire(segment, OFF_S2E_CAP);
        long e2sCap = (long) VH_LONG.getAcquire(segment, OFF_E2S_CAP);
        if (magic != MAGIC || ver != VERSION || s2eCap != S2E_CAP || e2sCap != E2S_CAP) {
            segment.asSlice(0, S2E_OFFSET).fill((byte) 0);
            VH_LONG.setRelease(segment, OFF_VERSION, VERSION);
            VH_LONG.setRelease(segment, OFF_S2E_CAP, (long) S2E_CAP);
            VH_LONG.setRelease(segment, OFF_S2E_OFFSET, (long) S2E_OFFSET);
            VH_LONG.setRelease(segment, OFF_E2S_CAP, (long) E2S_CAP);
            VH_LONG.setRelease(segment, OFF_E2S_OFFSET, (long) E2S_OFFSET);
            VH_LONG.setRelease(segment, OFF_SERVER_EPOCH, 1L);
            VH_LONG.setRelease(segment, OFF_MAGIC, MAGIC);
            lastEngineEpoch = 0L;
        } else {
            long e2sHead = (long) VH_LONG.getAcquire(segment, OFF_E2S_HEAD);
            VH_LONG.setRelease(segment, OFF_E2S_TAIL, e2sHead);
            long nextServerEpoch = Math.max(1L, ((long) VH_LONG.getAcquire(segment, OFF_SERVER_EPOCH)) + 1L);
            VH_LONG.setRelease(segment, OFF_SERVER_EPOCH, nextServerEpoch);
            lastEngineEpoch = 0L;
        }
    }

    public long serverEpoch() {
        return (long) VH_LONG.getAcquire(segment, OFF_SERVER_EPOCH);
    }

    public long engineEpoch() {
        return (long) VH_LONG.getAcquire(segment, OFF_ENGINE_EPOCH);
    }

    public boolean isEngineAlive() {
        long magic = (long) VH_LONG.getAcquire(segment, OFF_MAGIC);
        if (magic != MAGIC) {
            return false;
        }
        long epoch = (long) VH_LONG.getAcquire(segment, OFF_ENGINE_EPOCH);
        if (epoch == 0L) {
            return false;
        }
        long hb = (long) VH_LONG.getAcquire(segment, OFF_ENGINE_HEARTBEAT_MS);
        return (System.currentTimeMillis() - hb) < 1000L;
    }

    public boolean checkEngineReconnect() {
        if (!isEngineAlive()) {
            return false;
        }
        long epoch = engineEpoch();
        if (epoch != lastEngineEpoch) {
            lastEngineEpoch = epoch;
            return true;
        }
        return false;
    }

    public void setServerTickSeq(long tickSeq) {
        VH_LONG.setRelease(segment, OFF_SERVER_TICK_SEQ, tickSeq);
    }

    public boolean waitForTick(long serverTick, long timeoutMs) {
        long deadline = System.nanoTime() + timeoutMs * 1_000_000L;
        while (true) {
            long engSeq = (long) VH_LONG.getAcquire(segment, OFF_ENGINE_TICK_SEQ);
            if (engSeq >= serverTick) {
                return true;
            }
            if (!isEngineAlive() || System.nanoTime() >= deadline) {
                return false;
            }
            Thread.onSpinWait();
        }
    }

    public boolean pushS2E(short msgType, byte[] payload, int len) {
        int rec = (8 + len + 7) & ~7;
        if (rec > S2E_CAP) {
            return false;
        }
        long head = (long) VH_LONG.get(segment, OFF_S2E_HEAD);
        long tail = (long) VH_LONG.getAcquire(segment, OFF_S2E_TAIL);
        if ((head - tail) + rec > S2E_CAP) {
            long deadline = System.nanoTime() + 500_000_000L;
            while ((head - tail) + rec > S2E_CAP) {
                if (!isEngineAlive() || System.nanoTime() >= deadline) {
                    return false;
                }
                Thread.onSpinWait();
                tail = (long) VH_LONG.getAcquire(segment, OFF_S2E_TAIL);
            }
        }
        int off = (int) (head & (S2E_CAP - 1));
        long hdr = ((long) (msgType & 0xFFFF) << 32) | (len & 0xFFFF_FFFFL);
        segment.set(LONG_LE, (long) S2E_OFFSET + off, hdr);

        int payloadOff = (off + 8) & (S2E_CAP - 1);
        int first = Math.min(len, S2E_CAP - payloadOff);
        if (first > 0) {
            MemorySegment.copy(payload, 0, segment, ValueLayout.JAVA_BYTE, (long) S2E_OFFSET + payloadOff, first);
        }
        if (first < len) {
            MemorySegment.copy(payload, first, segment, ValueLayout.JAVA_BYTE, (long) S2E_OFFSET, len - first);
        }
        VH_LONG.setRelease(segment, OFF_S2E_HEAD, head + rec);
        return true;
    }

    public interface FrameConsumer {
        void accept(int msgType, ByteBuffer payload);
    }

    public void drainE2S(FrameConsumer consumer) {
        while (true) {
            long head = (long) VH_LONG.getAcquire(segment, OFF_E2S_HEAD);
            long tail = (long) VH_LONG.get(segment, OFF_E2S_TAIL);
            long used = head - tail;
            if (used < 8L) {
                return;
            }
            int off = (int) (tail & (E2S_CAP - 1));
            long hdr = segment.get(LONG_LE, (long) E2S_OFFSET + off);
            int len = (int) (hdr & 0xFFFF_FFFFL);
            int msgType = (int) ((hdr >>> 32) & 0xFFFFL);
            int rec = (8 + len + 7) & ~7;
            if (used < rec) {
                return;
            }
            if (rxBuf.length < len) {
                rxBuf = new byte[Math.max(len, rxBuf.length * 2)];
            }
            int payloadOff = (off + 8) & (E2S_CAP - 1);
            int first = Math.min(len, E2S_CAP - payloadOff);
            if (first > 0) {
                MemorySegment.copy(segment, ValueLayout.JAVA_BYTE, (long) E2S_OFFSET + payloadOff, rxBuf, 0, first);
            }
            if (first < len) {
                MemorySegment.copy(segment, ValueLayout.JAVA_BYTE, (long) E2S_OFFSET, rxBuf, first, len - first);
            }
            VH_LONG.setRelease(segment, OFF_E2S_TAIL, tail + rec);
            ByteBuffer bb = ByteBuffer.wrap(rxBuf, 0, len).order(ByteOrder.LITTLE_ENDIAN);
            consumer.accept(msgType, bb);
        }
    }

    public static String readString(ByteBuffer bb) {
        int len = Short.toUnsignedInt(bb.getShort());
        byte[] bytes = new byte[len];
        bb.get(bytes);
        return new String(bytes, StandardCharsets.UTF_8);
    }

    @Override
    public void close() throws IOException {
        channel.close();
    }

    /** Growable little-endian payload encoder for outgoing s2e frames. */
    public static final class PacketWriter {
        private byte[] buf = new byte[4096];
        private ByteBuffer bb = ByteBuffer.wrap(buf).order(ByteOrder.LITTLE_ENDIAN);

        public void reset() {
            bb.clear();
        }

        private void ensure(int extra) {
            if (bb.remaining() >= extra) {
                return;
            }
            int pos = bb.position();
            int nextCap = Math.max(buf.length * 2, pos + extra);
            byte[] next = new byte[nextCap];
            System.arraycopy(buf, 0, next, 0, pos);
            buf = next;
            bb = ByteBuffer.wrap(buf).order(ByteOrder.LITTLE_ENDIAN);
            bb.position(pos);
        }

        public void u8(int v) {
            ensure(1);
            bb.put((byte) v);
        }

        public void u16(int v) {
            ensure(2);
            bb.putShort((short) v);
        }

        public void i32(int v) {
            ensure(4);
            bb.putInt(v);
        }

        public void u32(int v) {
            ensure(4);
            bb.putInt(v);
        }

        public void u64(long v) {
            ensure(8);
            bb.putLong(v);
        }

        public void f32(float v) {
            ensure(4);
            bb.putFloat(v);
        }

        public void f64(double v) {
            ensure(8);
            bb.putDouble(v);
        }

        public void str(String s) {
            byte[] utf8 = s.getBytes(StandardCharsets.UTF_8);
            int len = Math.min(utf8.length, 65535);
            ensure(2 + len);
            bb.putShort((short) len);
            bb.put(utf8, 0, len);
        }

        public boolean send(ShmBridge bridge, short msgType) {
            return bridge.pushS2E(msgType, buf, bb.position());
        }
    }
}
