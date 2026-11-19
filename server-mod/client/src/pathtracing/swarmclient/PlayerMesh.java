package pathtracing.swarmclient;

/** Builds the 7-part articulated wide Minecraft player mesh (252 vertices, 12 floats/vertex). */
public final class PlayerMesh {
    public static final int FLOATS_PER_VERTEX = 12;
    public static final int PART_HEAD = 0;
    public static final int PART_TORSO = 1;
    public static final int PART_RIGHT_ARM = 2;
    public static final int PART_LEFT_ARM = 3;
    public static final int PART_RIGHT_LEG = 4;
    public static final int PART_LEFT_LEG = 5;

    private static final float SCALE = 0.9375f / 16.0f;

    public static float[] build() {
        float[] buf = new float[7 * 6 * 6 * FLOATS_PER_VERTEX];
        int[] cursor = { 0 };

        // Right & Left Legs (pivot at hip y = 12px)
        addBox(buf, cursor, -4f, 0f, -2f, 0f, 12f, 2f, -2f, 12f, 0f, 0, 16, 4, 12, 4, PART_RIGHT_LEG);
        addBox(buf, cursor, 0f, 0f, -2f, 4f, 12f, 2f, 2f, 12f, 0f, 16, 48, 4, 12, 4, PART_LEFT_LEG);

        // Torso (pivot at waist/shoulders y = 24px)
        addBox(buf, cursor, -4f, 12f, -2f, 4f, 24f, 2f, 0f, 24f, 0f, 16, 16, 8, 12, 4, PART_TORSO);

        // Right & Left Arms (pivot at shoulder y = 22px)
        addBox(buf, cursor, -8f, 12f, -2f, -4f, 24f, 2f, -6f, 22f, 0f, 40, 16, 4, 12, 4, PART_RIGHT_ARM);
        addBox(buf, cursor, 4f, 12f, -2f, 8f, 24f, 2f, 6f, 22f, 0f, 32, 48, 4, 12, 4, PART_LEFT_ARM);

        // Head + Hat overlay (pivot at neck y = 24px)
        addBox(buf, cursor, -4f, 24f, -4f, 4f, 32f, 4f, 0f, 24f, 0f, 0, 0, 8, 8, 8, PART_HEAD);
        addBox(buf, cursor, -4.25f, 23.75f, -4.25f, 4.25f, 32.25f, 4.25f, 0f, 24f, 0f, 32, 0, 8, 8, 8, PART_HEAD);

        if (cursor[0] != buf.length) {
            throw new IllegalStateException("Unexpected vertex float count: " + cursor[0]);
        }
        return buf;
    }

    private static void addBox(
        float[] buf,
        int[] cursor,
        float x0,
        float y0,
        float z0,
        float x1,
        float y1,
        float z1,
        float px,
        float py,
        float pz,
        int u,
        int v,
        int w,
        int h,
        int d,
        int partId
    ) {
        float sx0 = x0 * SCALE, sy0 = y0 * SCALE, sz0 = z0 * SCALE;
        float sx1 = x1 * SCALE, sy1 = y1 * SCALE, sz1 = z1 * SCALE;
        float spx = px * SCALE, spy = py * SCALE, spz = pz * SCALE;

        // Front (+Z): (u+d .. u+d+w, v+d .. v+d+h)
        quad(buf, cursor,
            sx0, sy0, sz1, sx1, sy0, sz1, sx1, sy1, sz1, sx0, sy1, sz1,
            0f, 0f, 1f, u + d, v + d + h, u + d + w, v + d, spx, spy, spz, partId);
        // Back (-Z): (u+2d+w .. u+2d+2w, v+d .. v+d+h)
        quad(buf, cursor,
            sx1, sy0, sz0, sx0, sy0, sz0, sx0, sy1, sz0, sx1, sy1, sz0,
            0f, 0f, -1f, u + 2 * d + w, v + d + h, u + 2 * d + 2 * w, v + d, spx, spy, spz, partId);
        // Right (-X): (u .. u+d, v+d .. v+d+h)
        quad(buf, cursor,
            sx0, sy0, sz0, sx0, sy0, sz1, sx0, sy1, sz1, sx0, sy1, sz0,
            -1f, 0f, 0f, u, v + d + h, u + d, v + d, spx, spy, spz, partId);
        // Left (+X): (u+d+w .. u+2d+w, v+d .. v+d+h)
        quad(buf, cursor,
            sx1, sy0, sz1, sx1, sy0, sz0, sx1, sy1, sz0, sx1, sy1, sz1,
            1f, 0f, 0f, u + d + w, v + d + h, u + 2 * d + w, v + d, spx, spy, spz, partId);
        // Top (+Y): (u+d .. u+d+w, v .. v+d)
        quad(buf, cursor,
            sx0, sy1, sz1, sx1, sy1, sz1, sx1, sy1, sz0, sx0, sy1, sz0,
            0f, 1f, 0f, u + d, v + d, u + d + w, v, spx, spy, spz, partId);
        // Bottom (-Y): (u+d+w .. u+d+2w, v .. v+d)
        quad(buf, cursor,
            sx0, sy0, sz0, sx1, sy0, sz0, sx1, sy0, sz1, sx0, sy0, sz1,
            0f, -1f, 0f, u + d + w, v, u + d + 2 * w, v + d, spx, spy, spz, partId);
    }

    private static void quad(
        float[] buf,
        int[] cursor,
        float ax, float ay, float az,
        float bx, float by, float bz,
        float cx, float cy, float cz,
        float dx, float dy, float dz,
        float nx, float ny, float nz,
        int uLeftPx,
        int vBotPx,
        int uRightPx,
        int vTopPx,
        float px, float py, float pz,
        int partId
    ) {
        float u0 = uLeftPx / 64.0f;
        float u1 = uRightPx / 64.0f;
        float vBot = vBotPx / 64.0f;
        float vTop = vTopPx / 64.0f;

        emitVertex(buf, cursor, ax, ay, az, px, py, pz, nx, ny, nz, u0, vBot, partId);
        emitVertex(buf, cursor, bx, by, bz, px, py, pz, nx, ny, nz, u1, vBot, partId);
        emitVertex(buf, cursor, cx, cy, cz, px, py, pz, nx, ny, nz, u1, vTop, partId);

        emitVertex(buf, cursor, ax, ay, az, px, py, pz, nx, ny, nz, u0, vBot, partId);
        emitVertex(buf, cursor, cx, cy, cz, px, py, pz, nx, ny, nz, u1, vTop, partId);
        emitVertex(buf, cursor, dx, dy, dz, px, py, pz, nx, ny, nz, u0, vTop, partId);
    }

    private static void emitVertex(
        float[] buf,
        int[] cursor,
        float x, float y, float z,
        float px, float py, float pz,
        float nx, float ny, float nz,
        float u, float v,
        int partId
    ) {
        int i = cursor[0];
        buf[i] = x;
        buf[i + 1] = y;
        buf[i + 2] = z;
        buf[i + 3] = px;
        buf[i + 4] = py;
        buf[i + 5] = pz;
        buf[i + 6] = nx;
        buf[i + 7] = ny;
        buf[i + 8] = nz;
        buf[i + 9] = u;
        buf[i + 10] = v;
        buf[i + 11] = partId;
        cursor[0] = i + FLOATS_PER_VERTEX;
    }
}
