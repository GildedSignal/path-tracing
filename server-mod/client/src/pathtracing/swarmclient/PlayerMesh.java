package pathtracing.swarmclient;

import com.mojang.blaze3d.vertex.PoseStack;
import java.util.Arrays;
import net.minecraft.client.model.geom.ModelPart;
import net.minecraft.client.model.geom.builders.CubeDeformation;
import net.minecraft.client.model.geom.builders.LayerDefinition;
import net.minecraft.client.model.player.PlayerModel;
import org.joml.Vector3f;
import org.joml.Vector3fc;

/**
 * Bakes WIDE and SLIM player base + 2nd-layer overlay meshes directly from Mojang's
 * `PlayerModel.createMesh`, plus a 2-box far LOD2 silhouette.
 */
public final class PlayerMesh {
    public static final int FLOATS_PER_VERTEX = 12;
    public static final int PART_HEAD = 0;
    public static final int PART_TORSO = 1;
    public static final int PART_RIGHT_ARM = 2;
    public static final int PART_LEFT_ARM = 3;
    public static final int PART_RIGHT_LEG = 4;
    public static final int PART_LEFT_LEG = 5;

    private static final float PLAYER_SCALE = 0.9375f;
    private static final float PIXEL_SCALE = PLAYER_SCALE / 16.0f;
    private static final int INITIAL_VERT_CAP = 1024;
    private static final int INITIAL_IDX_CAP = 1536;

    public record MeshData(
        float[] vertices,
        short[] indices,
        int vertexCount,
        int indexCount,
        int wideBaseFirst,
        int wideBaseCount,
        int wideOverlayFirst,
        int wideOverlayCount,
        int slimBaseFirst,
        int slimBaseCount,
        int slimOverlayFirst,
        int slimOverlayCount,
        int lod2First,
        int lod2Count
    ) {}

    private static final class Builder {
        float[] verts = new float[INITIAL_VERT_CAP * FLOATS_PER_VERTEX];
        short[] indices = new short[INITIAL_IDX_CAP];
        int vCount = 0;
        int iCount = 0;

        private final Vector3f tmpPos = new Vector3f();
        private final Vector3f tmpNorm = new Vector3f();
        private final Vector3f edge1 = new Vector3f();
        private final Vector3f edge2 = new Vector3f();
        private final Vector3f cross = new Vector3f();

        private void ensureVerts(int extra) {
            int needed = (vCount + extra) * FLOATS_PER_VERTEX;
            if (needed > verts.length) {
                verts = Arrays.copyOf(verts, Math.max(needed, verts.length * 2));
            }
        }

        private void ensureIndices(int extra) {
            int needed = iCount + extra;
            if (needed > indices.length) {
                indices = Arrays.copyOf(indices, Math.max(needed, indices.length * 2));
            }
        }

        void addCube(
            PoseStack.Pose pose,
            ModelPart.Cube cube,
            float[] pivot,
            int partId
        ) {
            if (cube == null || cube.polygons == null) {
                return;
            }
            for (ModelPart.Polygon poly : cube.polygons) {
                ModelPart.Vertex[] pVerts = poly.vertices();
                if (pVerts == null || pVerts.length != 4) {
                    continue;
                }
                Vector3fc rawN = poly.normal();
                pose.transformNormal(rawN, tmpNorm);
                float nx = tmpNorm.x;
                float ny = -tmpNorm.y;
                float nz = -tmpNorm.z;
                float lenSq = nx * nx + ny * ny + nz * nz;
                if (lenSq > 1e-8f) {
                    float invLen = (float) (1.0 / Math.sqrt(lenSq));
                    nx *= invLen;
                    ny *= invLen;
                    nz *= invLen;
                } else {
                    nx = 0f;
                    ny = 1f;
                    nz = 0f;
                }

                ensureVerts(4);
                int base = vCount;
                float[] xs = new float[4];
                float[] ys = new float[4];
                float[] zs = new float[4];
                for (int i = 0; i < 4; i++) {
                    ModelPart.Vertex mv = pVerts[i];
                    pose.pose().transformPosition(mv.worldX(), mv.worldY(), mv.worldZ(), tmpPos);
                    float lx = tmpPos.x * PLAYER_SCALE;
                    float ly = (1.501f - tmpPos.y) * PLAYER_SCALE;
                    float lz = -tmpPos.z * PLAYER_SCALE;
                    xs[i] = lx;
                    ys[i] = ly;
                    zs[i] = lz;
                    emitVertex(lx, ly, lz, pivot[0], pivot[1], pivot[2], nx, ny, nz, mv.u(), mv.v(), partId);
                }

                edge1.set(xs[1] - xs[0], ys[1] - ys[0], zs[1] - zs[0]);
                edge2.set(xs[2] - xs[0], ys[2] - ys[0], zs[2] - zs[0]);
                edge1.cross(edge2, cross);
                float dot = cross.x * nx + cross.y * ny + cross.z * nz;

                ensureIndices(6);
                if (dot >= 0.0f) {
                    indices[iCount++] = (short) base;
                    indices[iCount++] = (short) (base + 1);
                    indices[iCount++] = (short) (base + 2);
                    indices[iCount++] = (short) base;
                    indices[iCount++] = (short) (base + 2);
                    indices[iCount++] = (short) (base + 3);
                } else {
                    indices[iCount++] = (short) base;
                    indices[iCount++] = (short) (base + 3);
                    indices[iCount++] = (short) (base + 2);
                    indices[iCount++] = (short) base;
                    indices[iCount++] = (short) (base + 2);
                    indices[iCount++] = (short) (base + 1);
                }
            }
        }

        void addFarBox(
            float x0, float y0, float z0,
            float x1, float y1, float z1,
            float px, float py, float pz,
            int u, int v, int w, int h, int d,
            boolean skipTop, boolean skipBottom
        ) {
            float sx0 = x0 * PIXEL_SCALE, sy0 = y0 * PIXEL_SCALE, sz0 = z0 * PIXEL_SCALE;
            float sx1 = x1 * PIXEL_SCALE, sy1 = y1 * PIXEL_SCALE, sz1 = z1 * PIXEL_SCALE;
            float spx = px * PIXEL_SCALE, spy = py * PIXEL_SCALE, spz = pz * PIXEL_SCALE;

            farQuad(sx0, sy0, sz1, sx1, sy0, sz1, sx1, sy1, sz1, sx0, sy1, sz1,
                0f, 0f, 1f, u + d, v + d + h, u + d + w, v + d, spx, spy, spz);
            farQuad(sx1, sy0, sz0, sx0, sy0, sz0, sx0, sy1, sz0, sx1, sy1, sz0,
                0f, 0f, -1f, u + 2 * d + w, v + d + h, u + 2 * d + 2 * w, v + d, spx, spy, spz);
            farQuad(sx0, sy0, sz0, sx0, sy0, sz1, sx0, sy1, sz1, sx0, sy1, sz0,
                -1f, 0f, 0f, u, v + d + h, u + d, v + d, spx, spy, spz);
            farQuad(sx1, sy0, sz1, sx1, sy0, sz0, sx1, sy1, sz0, sx1, sy1, sz1,
                1f, 0f, 0f, u + d + w, v + d + h, u + 2 * d + w, v + d, spx, spy, spz);
            if (!skipTop) {
                farQuad(sx0, sy1, sz1, sx1, sy1, sz1, sx1, sy1, sz0, sx0, sy1, sz0,
                    0f, 1f, 0f, u + d, v + d, u + d + w, v, spx, spy, spz);
            }
            if (!skipBottom) {
                farQuad(sx0, sy0, sz0, sx1, sy0, sz0, sx1, sy0, sz1, sx0, sy0, sz1,
                    0f, -1f, 0f, u + d + w, v, u + d + 2 * w, v + d, spx, spy, spz);
            }
        }

        private void farQuad(
            float ax, float ay, float az,
            float bx, float by, float bz,
            float cx, float cy, float cz,
            float dx, float dy, float dz,
            float nx, float ny, float nz,
            int uLeftPx, int vBotPx, int uRightPx, int vTopPx,
            float px, float py, float pz
        ) {
            float u0 = uLeftPx / 64.0f;
            float u1 = uRightPx / 64.0f;
            float vBot = vBotPx / 64.0f;
            float vTop = vTopPx / 64.0f;

            ensureVerts(4);
            int base = vCount;
            emitVertex(ax, ay, az, px, py, pz, nx, ny, nz, u0, vBot, PART_TORSO);
            emitVertex(bx, by, bz, px, py, pz, nx, ny, nz, u1, vBot, PART_TORSO);
            emitVertex(cx, cy, cz, px, py, pz, nx, ny, nz, u1, vTop, PART_TORSO);
            emitVertex(dx, dy, dz, px, py, pz, nx, ny, nz, u0, vTop, PART_TORSO);

            ensureIndices(6);
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

        MeshData finish(
            int wideBaseFirst, int wideBaseCount,
            int wideOverlayFirst, int wideOverlayCount,
            int slimBaseFirst, int slimBaseCount,
            int slimOverlayFirst, int slimOverlayCount,
            int lod2First, int lod2Count
        ) {
            return new MeshData(
                Arrays.copyOf(verts, vCount * FLOATS_PER_VERTEX),
                Arrays.copyOf(indices, iCount),
                vCount,
                iCount,
                wideBaseFirst, wideBaseCount,
                wideOverlayFirst, wideOverlayCount,
                slimBaseFirst, slimBaseCount,
                slimOverlayFirst, slimOverlayCount,
                lod2First, lod2Count
            );
        }
    }

    private static float[] bonePivot(ModelPart root, String boneName) {
        ModelPart bone = root.getChild(boneName);
        return new float[] {
            (bone.x / 16.0f) * PLAYER_SCALE,
            (1.501f - bone.y / 16.0f) * PLAYER_SCALE,
            -(bone.z / 16.0f) * PLAYER_SCALE
        };
    }

    private static void bakeLayer(Builder b, ModelPart root, boolean overlay) {
        float[][] pivots = {
            bonePivot(root, "head"),
            bonePivot(root, "body"),
            bonePivot(root, "right_arm"),
            bonePivot(root, "left_arm"),
            bonePivot(root, "right_leg"),
            bonePivot(root, "left_leg")
        };
        PoseStack poseStack = new PoseStack();
        root.visit(poseStack, (pose, path, cubeIdx, cube) -> {
            int slash = path.lastIndexOf('/');
            String leaf = slash >= 0 ? path.substring(slash + 1) : path;
            int partId = overlay
                ? switch (leaf) {
                    case "hat" -> PART_HEAD;
                    case "jacket" -> PART_TORSO;
                    case "right_sleeve" -> PART_RIGHT_ARM;
                    case "left_sleeve" -> PART_LEFT_ARM;
                    case "right_pants" -> PART_RIGHT_LEG;
                    case "left_pants" -> PART_LEFT_LEG;
                    default -> -1;
                }
                : switch (leaf) {
                    case "head" -> PART_HEAD;
                    case "body" -> PART_TORSO;
                    case "right_arm" -> PART_RIGHT_ARM;
                    case "left_arm" -> PART_LEFT_ARM;
                    case "right_leg" -> PART_RIGHT_LEG;
                    case "left_leg" -> PART_LEFT_LEG;
                    default -> -1;
                };
            if (partId >= 0) {
                b.addCube(pose, cube, pivots[partId], partId);
            }
        });
    }

    public static MeshData build() {
        ModelPart wideRoot = LayerDefinition.create(PlayerModel.createMesh(CubeDeformation.NONE, false), 64, 64).bakeRoot();
        ModelPart slimRoot = LayerDefinition.create(PlayerModel.createMesh(CubeDeformation.NONE, true), 64, 64).bakeRoot();

        Builder b = new Builder();

        int wideBaseFirst = b.iCount;
        bakeLayer(b, wideRoot, false);
        int wideBaseCount = b.iCount - wideBaseFirst;

        int wideOverlayFirst = b.iCount;
        bakeLayer(b, wideRoot, true);
        int wideOverlayCount = b.iCount - wideOverlayFirst;

        int slimBaseFirst = b.iCount;
        bakeLayer(b, slimRoot, false);
        int slimBaseCount = b.iCount - slimBaseFirst;

        int slimOverlayFirst = b.iCount;
        bakeLayer(b, slimRoot, true);
        int slimOverlayCount = b.iCount - slimOverlayFirst;

        int lod2First = b.iCount;
        b.addFarBox(-4f, 24f, -4f, 4f, 32f, 4f, 0f, 24f, 0f, 0, 0, 8, 8, 8, false, true);
        b.addFarBox(-6f, 0f, -2f, 6f, 24f, 2f, 0f, 24f, 0f, 16, 16, 8, 12, 4, true, false);
        int lod2Count = b.iCount - lod2First;

        if (b.vCount > Short.MAX_VALUE) {
            throw new IllegalStateException("Mesh vertex count exceeds short index range: " + b.vCount);
        }
        return b.finish(
            wideBaseFirst, wideBaseCount,
            wideOverlayFirst, wideOverlayCount,
            slimBaseFirst, slimBaseCount,
            slimOverlayFirst, slimOverlayCount,
            lod2First, lod2Count
        );
    }
}
