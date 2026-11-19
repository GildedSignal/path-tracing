package pathtracing.swarmclient;

/** Builds an indexed 3-LOD opaque wide Minecraft player mesh (208 vertices, 312 short indices). */
public final class PlayerMesh {
    public static final int FLOATS_PER_VERTEX = 12;
    public static final int PART_HEAD = 0;
    public static final int PART_TORSO = 1;
    public static final int PART_RIGHT_ARM = 2;
    public static final int PART_LEFT_ARM = 3;
    public static final int PART_RIGHT_LEG = 4;
    public static final int PART_LEFT_LEG = 5;

    public static final int LOD0_FIRST_INDEX = 0;
    public static final int LOD0_INDEX_COUNT = 216;
    public static final int LOD1_FIRST_INDEX = 216;
    public static final int LOD1_INDEX_COUNT = 60;
    public static final int LOD2_FIRST_INDEX = 276;
    public static final int LOD2_INDEX_COUNT = 36;

    public static final int VERTEX_COUNT = 208;
    public static final int INDEX_COUNT = 312;

    private static final float SCALE = 0.9375f / 16.0f;

    private static final int FACE_FRONT = 1 << 0;
    private static final int FACE_BACK = 1 << 1;
    private static final int FACE_RIGHT = 1 << 2;
    private static final int FACE_LEFT = 1 << 3;
    private static final int FACE_TOP = 1 << 4;
    private static final int FACE_BOTTOM = 1 << 5;
    private static final int ALL_FACES = 0x3F;

    public record MeshData(float[] vertices, short[] indices) {}

    private static final class Builder {
        final float[] verts = new float[VERTEX_COUNT * FLOATS_PER_VERTEX];
        final short[] indices = new short[INDEX_COUNT];
        int vCount = 0;
        int iCount = 0;

        void addBox(
            float x0, float y0, float z0,
            float x1, float y1, float z1,
            float px, float py, float pz,
            int u, int v, int w, int h, int d,
            int partId,
            int faceMask
        ) {
            float sx0 = x0 * SCALE, sy0 = y0 * SCALE, sz0 = z0 * SCALE;
            float sx1 = x1 * SCALE, sy1 = y1 * SCALE, sz1 = z1 * SCALE;
            float spx = px * SCALE, spy = py * SCALE, spz = pz * SCALE;

            if ((faceMask & FACE_FRONT) != 0) {
                quad(sx0, sy0, sz1, sx1, sy0, sz1, sx1, sy1, sz1, sx0, sy1, sz1,
                    0f, 0f, 1f, u + d, v + d + h, u + d + w, v + d, spx, spy, spz, partId);
            }
            if ((faceMask & FACE_BACK) != 0) {
                quad(sx1, sy0, sz0, sx0, sy0, sz0, sx0, sy1, sz0, sx1, sy1, sz0,
                    0f, 0f, -1f, u + 2 * d + w, v + d + h, u + 2 * d + 2 * w, v + d, spx, spy, spz, partId);
            }
            if ((faceMask & FACE_RIGHT) != 0) {
                quad(sx0, sy0, sz0, sx0, sy0, sz1, sx0, sy1, sz1, sx0, sy1, sz0,
                    -1f, 0f, 0f, u, v + d + h, u + d, v + d, spx, spy, spz, partId);
            }
            if ((faceMask & FACE_LEFT) != 0) {
                quad(sx1, sy0, sz1, sx1, sy0, sz0, sx1, sy1, sz0, sx1, sy1, sz1,
                    1f, 0f, 0f, u + d + w, v + d + h, u + 2 * d + w, v + d, spx, spy, spz, partId);
            }
            if ((faceMask & FACE_TOP) != 0) {
                quad(sx0, sy1, sz1, sx1, sy1, sz1, sx1, sy1, sz0, sx0, sy1, sz0,
                    0f, 1f, 0f, u + d, v + d, u + d + w, v, spx, spy, spz, partId);
            }
            if ((faceMask & FACE_BOTTOM) != 0) {
                quad(sx0, sy0, sz0, sx1, sy0, sz0, sx1, sy0, sz1, sx0, sy0, sz1,
                    0f, -1f, 0f, u + d + w, v, u + d + 2 * w, v + d, spx, spy, spz, partId);
            }
        }

        void addProxyBox() {
            float sx0 = -4f * SCALE, sy0 = 0f, sz0 = -4f * SCALE;
            float sx1 = 4f * SCALE, sy1 = 32f * SCALE, sz1 = 4f * SCALE;
            quad(sx0, sy0, sz1, sx1, sy0, sz1, sx1, sy1, sz1, sx0, sy1, sz1,
                0f, 0f, 1f, 20, 32, 28, 20, 0f, 0f, 0f, PART_TORSO);
            quad(sx1, sy0, sz0, sx0, sy0, sz0, sx0, sy1, sz0, sx1, sy1, sz0,
                0f, 0f, -1f, 32, 32, 40, 20, 0f, 0f, 0f, PART_TORSO);
            quad(sx0, sy0, sz0, sx0, sy0, sz1, sx0, sy1, sz1, sx0, sy1, sz0,
                -1f, 0f, 0f, 16, 32, 20, 20, 0f, 0f, 0f, PART_TORSO);
            quad(sx1, sy0, sz1, sx1, sy0, sz0, sx1, sy1, sz0, sx1, sy1, sz1,
                1f, 0f, 0f, 28, 32, 32, 20, 0f, 0f, 0f, PART_TORSO);
            quad(sx0, sy1, sz1, sx1, sy1, sz1, sx1, sy1, sz0, sx0, sy1, sz0,
                0f, 1f, 0f, 8, 8, 16, 0, 0f, 0f, 0f, PART_TORSO);
            quad(sx0, sy0, sz0, sx1, sy0, sz0, sx1, sy0, sz1, sx0, sy0, sz1,
                0f, -1f, 0f, 8, 16, 12, 20, 0f, 0f, 0f, PART_TORSO);
        }

        private void quad(
            float ax, float ay, float az,
            float bx, float by, float bz,
            float cx, float cy, float cz,
            float dx, float dy, float dz,
            float nx, float ny, float nz,
            int uLeftPx, int vBotPx, int uRightPx, int vTopPx,
            float px, float py, float pz,
            int partId
        ) {
            float u0 = uLeftPx / 64.0f;
            float u1 = uRightPx / 64.0f;
            float vBot = vBotPx / 64.0f;
            float vTop = vTopPx / 64.0f;

            int base = vCount;
            emitVertex(ax, ay, az, px, py, pz, nx, ny, nz, u0, vBot, partId);
            emitVertex(bx, by, bz, px, py, pz, nx, ny, nz, u1, vBot, partId);
            emitVertex(cx, cy, cz, px, py, pz, nx, ny, nz, u1, vTop, partId);
            emitVertex(dx, dy, dz, px, py, pz, nx, ny, nz, u0, vTop, partId);

            indices[iCount++] = (short) base;
            indices[iCount++] = (short) (base + 1);
            indices[iCount++] = (short) (base + 2);
            indices[iCount++] = (short) base;
            indices[iCount++] = (short) (base + 2);
            indices[iCount++] = (short) (base + 3);
        }

        private void emitVertex(
            float x, float y, float z,
            float px, float py, float pz,
            float nx, float ny, float nz,
            float u, float v,
            int partId
        ) {
            int off = vCount * FLOATS_PER_VERTEX;
            verts[off] = x;
            verts[off + 1] = y;
            verts[off + 2] = z;
            verts[off + 3] = px;
            verts[off + 4] = py;
            verts[off + 5] = pz;
            verts[off + 6] = nx;
            verts[off + 7] = ny;
            verts[off + 8] = nz;
            verts[off + 9] = u;
            verts[off + 10] = v;
            verts[off + 11] = partId;
            vCount++;
        }
    }

    public static MeshData build() {
        Builder b = new Builder();

        // LOD 0: 6 opaque articulated boxes (indices 0..215, verts 0..143)
        b.addBox(-4f, 0f, -2f, 0f, 12f, 2f, -2f, 12f, 0f, 0, 16, 4, 12, 4, PART_RIGHT_LEG, ALL_FACES);
        b.addBox(0f, 0f, -2f, 4f, 12f, 2f, 2f, 12f, 0f, 16, 48, 4, 12, 4, PART_LEFT_LEG, ALL_FACES);
        b.addBox(-4f, 12f, -2f, 4f, 24f, 2f, 0f, 24f, 0f, 16, 16, 8, 12, 4, PART_TORSO, ALL_FACES);
        b.addBox(-8f, 12f, -2f, -4f, 24f, 2f, -6f, 22f, 0f, 40, 16, 4, 12, 4, PART_RIGHT_ARM, ALL_FACES);
        b.addBox(4f, 12f, -2f, 8f, 24f, 2f, 6f, 22f, 0f, 32, 48, 4, 12, 4, PART_LEFT_ARM, ALL_FACES);
        b.addBox(-4f, 24f, -4f, 4f, 32f, 4f, 0f, 24f, 0f, 0, 0, 8, 8, 8, PART_HEAD, ALL_FACES);
        if (b.iCount != LOD0_INDEX_COUNT) {
            throw new IllegalStateException("Unexpected LOD0 index count: " + b.iCount);
        }

        // LOD 1: 2 rigid boxes (10 faces = 40 verts, 60 indices: 216..275)
        b.addBox(-4f, 24f, -4f, 4f, 32f, 4f, 0f, 24f, 0f, 0, 0, 8, 8, 8, PART_TORSO, ALL_FACES & ~FACE_BOTTOM);
        b.addBox(-4f, 0f, -2f, 4f, 24f, 2f, 0f, 24f, 0f, 16, 16, 8, 12, 4, PART_TORSO, ALL_FACES & ~FACE_TOP);
        if (b.iCount != LOD0_INDEX_COUNT + LOD1_INDEX_COUNT) {
            throw new IllegalStateException("Unexpected LOD1 index count: " + b.iCount);
        }

        // LOD 2: 1 rigid 6-face proxy box (24 verts, 36 indices: 276..311)
        b.addProxyBox();
        if (b.vCount != VERTEX_COUNT || b.iCount != INDEX_COUNT) {
            throw new IllegalStateException("Unexpected mesh totals: verts=" + b.vCount + " indices=" + b.iCount);
        }
        return new MeshData(b.verts, b.indices);
    }
}
