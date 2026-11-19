package pathtracing.swarmclient;

import com.mojang.blaze3d.GpuFormat;
import com.mojang.blaze3d.IndexType;
import com.mojang.blaze3d.PrimitiveTopology;
import com.mojang.blaze3d.buffers.GpuBuffer;
import com.mojang.blaze3d.buffers.GpuBufferSlice;
import com.mojang.blaze3d.buffers.Std140Builder;
import com.mojang.blaze3d.pipeline.BindGroupLayout;
import com.mojang.blaze3d.pipeline.ColorTargetState;
import com.mojang.blaze3d.pipeline.CompiledRenderPipeline;
import com.mojang.blaze3d.pipeline.DepthStencilState;
import com.mojang.blaze3d.pipeline.RenderPipeline;
import com.mojang.blaze3d.pipeline.RenderTarget;
import com.mojang.blaze3d.platform.NativeImage;
import com.mojang.blaze3d.shaders.ShaderSource;
import com.mojang.blaze3d.shaders.ShaderType;
import com.mojang.blaze3d.shaders.UniformType;
import com.mojang.blaze3d.systems.CommandEncoder;
import com.mojang.blaze3d.systems.GpuDevice;
import com.mojang.blaze3d.systems.RenderPass;
import com.mojang.blaze3d.systems.RenderSystem;
import com.mojang.blaze3d.textures.AddressMode;
import com.mojang.blaze3d.textures.FilterMode;
import com.mojang.blaze3d.textures.GpuSampler;
import com.mojang.blaze3d.textures.GpuTexture;
import com.mojang.blaze3d.textures.GpuTextureView;
import com.mojang.blaze3d.vertex.VertexFormat;
import java.io.InputStream;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.HashSet;
import java.util.List;
import java.util.Optional;
import java.util.OptionalDouble;
import java.util.Set;
import java.util.UUID;
import net.fabricmc.fabric.api.client.rendering.v1.level.LevelRenderContext;
import net.minecraft.client.Minecraft;
import net.minecraft.client.player.AbstractClientPlayer;
import net.minecraft.client.renderer.LevelTargetBundle;
import net.minecraft.client.renderer.fog.FogData;
import net.minecraft.client.renderer.state.level.CameraRenderState;
import net.minecraft.resources.Identifier;
import net.minecraft.server.packs.resources.Resource;
import org.joml.Matrix4f;
import org.joml.Vector3f;
import org.lwjgl.system.MemoryUtil;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import pathtracing.swarmclient.mixin.LevelRendererAccessor;

/**
 * Single adaptive Blaze3D GPU instanced renderer for all 10,000 bots (virtual and materialized),
 * baked directly from Mojang's WIDE/SLIM PlayerModel + 2nd overlay layer and Mojang lighting/lightmap.
 */
public final class SwarmRenderer {
    private static final Logger LOGGER = LoggerFactory.getLogger("pathtracing_swarm_client");
    private static final int UBO_SIZE = 112;
    private static final int LIGHTING_UBO_SIZE = 32;
    private static final int INITIAL_INSTANCE_CAPACITY_BOTS = 16384;

    // Exact order of Mojang DefaultPlayerSkin.DEFAULT_SKINS[0..17] (0..8 SLIM, 9..17 WIDE).
    private static final String[] DEFAULT_SKIN_PATHS = {
        "textures/entity/player/slim/alex.png",
        "textures/entity/player/slim/ari.png",
        "textures/entity/player/slim/efe.png",
        "textures/entity/player/slim/kai.png",
        "textures/entity/player/slim/makena.png",
        "textures/entity/player/slim/noor.png",
        "textures/entity/player/slim/steve.png",
        "textures/entity/player/slim/sunny.png",
        "textures/entity/player/slim/zuri.png",
        "textures/entity/player/wide/alex.png",
        "textures/entity/player/wide/ari.png",
        "textures/entity/player/wide/efe.png",
        "textures/entity/player/wide/kai.png",
        "textures/entity/player/wide/makena.png",
        "textures/entity/player/wide/noor.png",
        "textures/entity/player/wide/steve.png",
        "textures/entity/player/wide/sunny.png",
        "textures/entity/player/wide/zuri.png"
    };

    private static final String VERT_SHADER = """
        #version 330
        in vec3 LocalPos;
        in vec3 Pivot;
        in vec3 Normal;
        in vec2 UV0;
        in float PartId;

        in vec4 InstPosSkin;
        in vec4 InstPose;
        in vec4 InstExtra;

        layout(std140) uniform Projection {
            mat4 ProjMat;
        };

        layout(std140) uniform Lighting {
            vec3 Light0_Direction;
            vec3 Light1_Direction;
        };

        layout(std140) uniform SwarmUniforms {
            mat4 uViewMat;
            vec4 uOriginRelCam;
            vec4 uFogColor;
            vec4 uFogParams;
        };

        out vec2 vUv;
        out float vShade;
        out vec2 vLightUv;
        out float vSkinLayer;
        out float vHurtRed;
        out float vCamDist;

        const float PX_SCALE = 0.9375 / 16.0;

        vec3 rotXAroundPivot(vec3 p, vec3 pivot, float a) {
            vec3 d = p - pivot;
            float c = cos(a);
            float s = sin(a);
            return pivot + vec3(d.x, d.y * c - d.z * s, d.y * s + d.z * c);
        }

        vec3 rotNormalX(vec3 n, float a) {
            float c = cos(a);
            float s = sin(a);
            return vec3(n.x, n.y * c - n.z * s, n.y * s + n.z * c);
        }

        vec3 rotYAroundPivot(vec3 p, vec3 pivot, float a) {
            vec3 d = p - pivot;
            float c = cos(a);
            float s = sin(a);
            return pivot + vec3(d.x * c - d.z * s, d.y, d.x * s + d.z * c);
        }

        vec3 rotNormalY(vec3 n, float a) {
            float c = cos(a);
            float s = sin(a);
            return vec3(n.x * c - n.z * s, n.y, n.x * s + n.z * c);
        }

        void main() {
            int part = int(PartId + 0.5);
            vec3 localP = LocalPos;
            vec3 localN = Normal;

            float meta = InstPosSkin.w;
            float skinIdx = mod(meta, 32.0);
            float flagRem = floor(meta / 32.0);
            bool sneaking = mod(flagRem, 2.0) > 0.5;
            bool hurtRed = floor(flagRem / 2.0) > 0.5;

            float headPitch = InstPose.z;
            float legSwing = InstPose.w;
            float armSwing = legSwing * (1.0 / 1.4);
            float headRelYaw = InstExtra.x;
            float deathRoll = InstExtra.y;

            if (part == 0) {
                localP = rotXAroundPivot(localP, Pivot, headPitch);
                localN = rotNormalX(localN, headPitch);
                localP = rotYAroundPivot(localP, Pivot, headRelYaw);
                localN = rotNormalY(localN, headRelYaw);
                if (sneaking) {
                    localP.y -= 4.2 * PX_SCALE;
                }
            } else if (part == 1) {
                if (sneaking) {
                    localP = rotXAroundPivot(localP, Pivot, 0.5);
                    localN = rotNormalX(localN, 0.5);
                    localP.y -= 3.2 * PX_SCALE;
                }
            } else if (part == 2) {
                float a = -armSwing + (sneaking ? 0.4 : 0.0);
                localP = rotXAroundPivot(localP, Pivot, a);
                localN = rotNormalX(localN, a);
                if (sneaking) {
                    localP.y -= 3.2 * PX_SCALE;
                }
            } else if (part == 3) {
                float a = armSwing + (sneaking ? 0.4 : 0.0);
                localP = rotXAroundPivot(localP, Pivot, a);
                localN = rotNormalX(localN, a);
                if (sneaking) {
                    localP.y -= 3.2 * PX_SCALE;
                }
            } else if (part == 4) {
                localP = rotXAroundPivot(localP, Pivot, legSwing);
                localN = rotNormalX(localN, legSwing);
                if (sneaking) {
                    localP.z -= 4.0 * PX_SCALE;
                }
            } else if (part == 5) {
                localP = rotXAroundPivot(localP, Pivot, -legSwing);
                localN = rotNormalX(localN, -legSwing);
                if (sneaking) {
                    localP.z -= 4.0 * PX_SCALE;
                }
            }

            if (sneaking) {
                localP.y -= 0.125 * 0.9375;
            }

            if (deathRoll > 0.0001) {
                float cr = cos(deathRoll);
                float sr = sin(deathRoll);
                localP = vec3(localP.x * cr + localP.y * sr, -localP.x * sr + localP.y * cr, localP.z);
                localN = vec3(localN.x * cr + localN.y * sr, -localN.x * sr + localN.y * cr, localN.z);
            }

            float cy = InstPose.x;
            float sy = InstPose.y;
            vec3 rotated = vec3(
                localP.x * cy - localP.z * sy,
                localP.y,
                localP.x * sy + localP.z * cy
            );
            vec3 worldNorm = normalize(vec3(
                localN.x * cy - localN.z * sy,
                localN.y,
                localN.x * sy + localN.z * cy
            ));

            vec2 light = max(vec2(0.0), vec2(dot(Light0_Direction, worldNorm), dot(Light1_Direction, worldNorm)));
            vShade = min(1.0, (light.x + light.y) * 0.6 + 0.4);

            vec3 relCam = uOriginRelCam.xyz + InstPosSkin.xyz + rotated;
            gl_Position = ProjMat * (uViewMat * vec4(relCam, 1.0));
            vUv = UV0;
            vLightUv = InstExtra.zw;
            vSkinLayer = skinIdx;
            vHurtRed = hurtRed ? 1.0 : 0.0;
            vCamDist = length(relCam);
        }
        """;

    private static String buildFragShader(boolean cutout) {
        String alphaCheck = cutout ? "    if (texel.a < 0.1) discard;\n" : "";
        return """
            #version 330
            in vec2 vUv;
            in float vShade;
            in vec2 vLightUv;
            in float vSkinLayer;
            in float vHurtRed;
            in float vCamDist;

            layout(std140) uniform SwarmUniforms {
                mat4 uViewMat;
                vec4 uOriginRelCam;
                vec4 uFogColor;
                vec4 uFogParams;
            };

            uniform sampler2D Sampler0;
            uniform sampler2D Sampler2;

            out vec4 fragColor;

            void main() {
                vec2 atlasUv = vec2(
                    (floor(vSkinLayer + 0.5) + clamp(vUv.x, 0.0001, 0.9999)) / 18.0,
                    clamp(vUv.y, 0.0001, 0.9999)
                );
                vec4 texel = texture(Sampler0, atlasUv);
            """ + alphaCheck + """
                vec3 lm = texture(Sampler2, vLightUv).rgb;
                vec3 lit = texel.rgb * vShade * lm;
                if (vHurtRed > 0.5) {
                    lit = mix(vec3(1.0, 0.0, 0.0), lit, 0.70);
                }

                float fogStart = uFogParams.x;
                float fogEnd = uFogParams.y;
                if (fogEnd > fogStart && uFogColor.a > 0.001) {
                    float fog = clamp((vCamDist - fogStart) / (fogEnd - fogStart), 0.0, 1.0) * uFogColor.a;
                    lit = mix(lit, uFogColor.rgb, fog);
                }
                fragColor = vec4(lit, 1.0);
            }
            """;
    }

    private static final String FRAG_SHADER_OPAQUE = buildFragShader(false);
    private static final String FRAG_SHADER_OVERLAY = buildFragShader(true);

    private static final Identifier OPAQUE_SHADER_ID = Identifier.fromNamespaceAndPath("pathtracing", "core/swarm_bots_opaque");
    private static final Identifier OVERLAY_SHADER_ID = Identifier.fromNamespaceAndPath("pathtracing", "core/swarm_bots_overlay");

    private static final ShaderSource OPAQUE_SHADER_SOURCE = (id, type) -> switch (type) {
        case VERTEX -> VERT_SHADER;
        case FRAGMENT -> FRAG_SHADER_OPAQUE;
    };

    private static final ShaderSource OVERLAY_SHADER_SOURCE = (id, type) -> switch (type) {
        case VERTEX -> VERT_SHADER;
        case FRAGMENT -> FRAG_SHADER_OVERLAY;
    };

    private static final VertexFormat MESH_FORMAT = VertexFormat.builder(0)
        .addAttribute("LocalPos", GpuFormat.RGB32_FLOAT)
        .addAttribute("Pivot", GpuFormat.RGB32_FLOAT)
        .addAttribute("Normal", GpuFormat.RGB32_FLOAT)
        .addAttribute("UV0", GpuFormat.RG32_FLOAT)
        .addAttribute("PartId", GpuFormat.R32_FLOAT)
        .build();

    private static final VertexFormat INSTANCE_FORMAT = VertexFormat.builder(1)
        .addAttribute("InstPosSkin", GpuFormat.RGBA32_FLOAT)
        .addAttribute("InstPose", GpuFormat.RGBA32_FLOAT)
        .addAttribute("InstExtra", GpuFormat.RGBA32_FLOAT)
        .build();

    private static final BindGroupLayout BIND_GROUP_LAYOUT = BindGroupLayout.builder()
        .withUniform("Projection", UniformType.UNIFORM_BUFFER)
        .withUniform("Lighting", UniformType.UNIFORM_BUFFER)
        .withUniform("SwarmUniforms", UniformType.UNIFORM_BUFFER)
        .withSampler("Sampler0")
        .withSampler("Sampler2")
        .build();

    private static final RenderPipeline PIPELINE_OPAQUE = RenderPipeline.builder()
        .withLocation(Identifier.fromNamespaceAndPath("pathtracing", "pipeline/swarm_bots_opaque"))
        .withVertexShader(OPAQUE_SHADER_ID)
        .withFragmentShader(OPAQUE_SHADER_ID)
        .withBindGroupLayout(BIND_GROUP_LAYOUT)
        .withVertexBinding(0, MESH_FORMAT)
        .withVertexBinding(1, INSTANCE_FORMAT)
        .withPrimitiveTopology(PrimitiveTopology.TRIANGLES)
        .withCull(true)
        .withColorTargetState(ColorTargetState.DEFAULT)
        .withDepthStencilState(DepthStencilState.DEFAULT)
        .build();

    private static final RenderPipeline PIPELINE_OVERLAY = RenderPipeline.builder()
        .withLocation(Identifier.fromNamespaceAndPath("pathtracing", "pipeline/swarm_bots_overlay"))
        .withVertexShader(OVERLAY_SHADER_ID)
        .withFragmentShader(OVERLAY_SHADER_ID)
        .withBindGroupLayout(BIND_GROUP_LAYOUT)
        .withVertexBinding(0, MESH_FORMAT)
        .withVertexBinding(1, INSTANCE_FORMAT)
        .withPrimitiveTopology(PrimitiveTopology.TRIANGLES)
        .withCull(false)
        .withColorTargetState(ColorTargetState.DEFAULT)
        .withDepthStencilState(DepthStencilState.DEFAULT)
        .build();

    private final SwarmBuffer buffer;
    private boolean initialized = false;
    private boolean initFailed = false;
    private PlayerMesh.MeshData meshData;
    private GpuBuffer meshGpuBuf;
    private GpuBuffer indexGpuBuf;
    private GpuBuffer instanceGpuBuf;
    private int instanceCapacityBytes = 0;
    private GpuBuffer uniformGpuBuf;
    private GpuBuffer fallbackLightGpuBuf;
    private GpuTexture skinTex;
    private GpuTextureView skinTexView;
    private GpuSampler skinSampler;
    private int skinsLoaded = 0;

    private final ByteBuffer uboBytes = MemoryUtil.memAlloc(UBO_SIZE).order(ByteOrder.LITTLE_ENDIAN);
    private final Matrix4f viewProj = new Matrix4f();
    private final float[] relOrigin = new float[3];
    private final int[] bucketCounts = new int[SwarmBuffer.BUCKET_COUNT];
    private final ByteBuffer[] instanceBufRef = new ByteBuffer[1];
    private final List<AbstractClientPlayer> liveBotPlayers = new ArrayList<>();
    private final Set<UUID> liveClientBotUuids = new HashSet<>();
    private final Set<Integer> managedEntityIds = new HashSet<>();
    private int lastLoggedTotal = -1;

    public SwarmRenderer(SwarmBuffer buffer) {
        this.buffer = buffer;
    }

    public synchronized void clearManagedEntities() {
        managedEntityIds.clear();
    }

    public synchronized boolean isManagedEntityId(int entityId) {
        if (managedEntityIds.contains(entityId)) {
            return true;
        }
        Minecraft mc = Minecraft.getInstance();
        if (mc.level != null && mc.level.getEntity(entityId) instanceof AbstractClientPlayer p && p != mc.player) {
            UUID uuid = p.getUUID();
            if (buffer.isTrackedBotUuid(uuid) || SwarmBuffer.isSwarmBotProfile(uuid, p.getGameProfile().name())) {
                managedEntityIds.add(entityId);
                return true;
            }
        }
        return false;
    }

    public void render(LevelRenderContext ctx) {
        if (initFailed) {
            return;
        }
        CameraRenderState camState = ctx.levelState() != null ? ctx.levelState().cameraRenderState : null;
        if (camState == null || !camState.initialized || camState.projectionMatrix == null || camState.viewRotationMatrix == null) {
            return;
        }
        if (!(ctx.levelRenderer() instanceof LevelRendererAccessor accessor)) {
            return;
        }
        LevelTargetBundle targets = accessor.ptGetTargets();
        RenderTarget mainRt = (targets != null && targets.main != null) ? targets.main.get() : null;
        if (mainRt == null || mainRt.getColorTextureView() == null || mainRt.getDepthTextureView() == null) {
            return;
        }

        Minecraft mc = Minecraft.getInstance();
        GpuDevice device = RenderSystem.getDevice();
        GpuBufferSlice projBuf = RenderSystem.getProjectionMatrixBuffer();
        GpuTextureView lightmapView = mc.gameRenderer != null ? mc.gameRenderer.levelLightmap() : null;
        if (projBuf == null || lightmapView == null) {
            return;
        }
        GpuSampler lightmapSampler = RenderSystem.getSamplerCache().getClampToEdge(FilterMode.LINEAR);
        CommandEncoder encoder = device.createCommandEncoder();

        if (!initialized) {
            initGpu(device, encoder);
            if (initFailed) {
                return;
            }
        }

        CompiledRenderPipeline compiledOpaque = device.precompilePipeline(PIPELINE_OPAQUE, OPAQUE_SHADER_SOURCE);
        CompiledRenderPipeline compiledOverlay = device.precompilePipeline(PIPELINE_OVERLAY, OVERLAY_SHADER_SOURCE);
        if (!compiledOpaque.isValid() || !compiledOverlay.isValid()) {
            LOGGER.error(
                "[SwarmClient] Pipeline invalid (opaque={}, overlay={}); disabling renderer",
                compiledOpaque.isValid(), compiledOverlay.isValid()
            );
            initFailed = true;
            return;
        }

        long t0 = System.nanoTime();
        liveBotPlayers.clear();
        liveClientBotUuids.clear();
        synchronized (this) {
            managedEntityIds.clear();
            if (mc.level != null) {
                var players = mc.level.players();
                for (int i = 0; i < players.size(); i++) {
                    AbstractClientPlayer p = players.get(i);
                    if (p != mc.player) {
                        UUID uuid = p.getUUID();
                        if (buffer.isTrackedBotUuid(uuid) || SwarmBuffer.isSwarmBotProfile(uuid, p.getGameProfile().name())) {
                            p.getInterpolation().setInterpolationLength(1);
                            liveBotPlayers.add(p);
                            liveClientBotUuids.add(uuid);
                            managedEntityIds.add(p.getId());
                        }
                    }
                }
            }
        }

        float partialTick = mc.getDeltaTracker().getGameTimeDeltaPartialTick(false);
        viewProj.set(camState.projectionMatrix).mul(camState.viewRotationMatrix);
        int drawn = buffer.buildVisibleInstances(
            mc.level,
            camState.pos.x,
            camState.pos.y,
            camState.pos.z,
            partialTick,
            viewProj,
            liveBotPlayers,
            liveClientBotUuids,
            relOrigin,
            bucketCounts,
            instanceBufRef
        );
        if (drawn <= 0) {
            return;
        }

        uploadInstanceBuffer(device, encoder, instanceBufRef[0], drawn);

        float fogR = 0f, fogG = 0f, fogB = 0f, fogA = 0f, fogStart = 0f, fogEnd = 0f;
        FogData fog = camState.fogData;
        if (fog != null && fog.color != null) {
            fogR = fog.color.x;
            fogG = fog.color.y;
            fogB = fog.color.z;
            fogA = fog.color.w;
            fogStart = Math.min(fog.environmentalStart, fog.renderDistanceStart);
            fogEnd = Math.min(fog.environmentalEnd, fog.renderDistanceEnd);
        }

        uboBytes.clear();
        ByteBuffer uboSliceData = Std140Builder.intoBuffer(uboBytes)
            .putMat4f(camState.viewRotationMatrix)
            .putVec4(relOrigin[0], relOrigin[1], relOrigin[2], 0f)
            .putVec4(fogR, fogG, fogB, fogA)
            .putVec4(fogStart, fogEnd, 0f, 0f)
            .get();
        encoder.writeToBuffer(uniformGpuBuf.slice(), uboSliceData);

        GpuBufferSlice lightBuf = RenderSystem.getShaderLights();
        if (lightBuf == null) {
            lightBuf = fallbackLightGpuBuf.slice();
        }

        int c0w = bucketCounts[SwarmBuffer.BUCKET_LOD0_WIDE];
        int c0s = bucketCounts[SwarmBuffer.BUCKET_LOD0_SLIM];
        int c1w = bucketCounts[SwarmBuffer.BUCKET_LOD1_WIDE];
        int c1s = bucketCounts[SwarmBuffer.BUCKET_LOD1_SLIM];
        int c2 = bucketCounts[SwarmBuffer.BUCKET_LOD2];

        long bytes0w = (long) c0w * SwarmBuffer.INSTANCE_BYTES;
        long bytes0s = (long) c0s * SwarmBuffer.INSTANCE_BYTES;
        long bytes1w = (long) c1w * SwarmBuffer.INSTANCE_BYTES;
        long bytes1s = (long) c1s * SwarmBuffer.INSTANCE_BYTES;
        long bytes2 = (long) c2 * SwarmBuffer.INSTANCE_BYTES;

        long off0w = 0L;
        long off0s = off0w + bytes0w;
        long off1w = off0s + bytes0s;
        long off1s = off1w + bytes1w;
        long off2 = off1s + bytes1s;

        try (RenderPass pass = encoder.createRenderPass(
            () -> "swarm_bots",
            mainRt.getColorTextureView(),
            Optional.empty(),
            mainRt.getDepthTextureView(),
            OptionalDouble.empty()
        )) {
            pass.setPipeline(PIPELINE_OPAQUE);
            pass.setUniform("Projection", projBuf);
            pass.setUniform("Lighting", lightBuf);
            pass.setUniform("SwarmUniforms", uniformGpuBuf);
            pass.bindTexture("Sampler0", skinTexView, skinSampler);
            pass.bindTexture("Sampler2", lightmapView, lightmapSampler);
            pass.setVertexBuffer(0, meshGpuBuf.slice());
            pass.setIndexBuffer(indexGpuBuf, IndexType.SHORT);

            if (c0w > 0) {
                pass.setVertexBuffer(1, instanceGpuBuf.slice(off0w, bytes0w));
                pass.drawIndexed(meshData.wideBaseCount(), c0w, meshData.wideBaseFirst(), 0, 0);
            }
            if (c0s > 0) {
                pass.setVertexBuffer(1, instanceGpuBuf.slice(off0s, bytes0s));
                pass.drawIndexed(meshData.slimBaseCount(), c0s, meshData.slimBaseFirst(), 0, 0);
            }
            if (c1w > 0) {
                pass.setVertexBuffer(1, instanceGpuBuf.slice(off1w, bytes1w));
                pass.drawIndexed(meshData.wideBaseCount(), c1w, meshData.wideBaseFirst(), 0, 0);
            }
            if (c1s > 0) {
                pass.setVertexBuffer(1, instanceGpuBuf.slice(off1s, bytes1s));
                pass.drawIndexed(meshData.slimBaseCount(), c1s, meshData.slimBaseFirst(), 0, 0);
            }
            if (c2 > 0) {
                pass.setVertexBuffer(1, instanceGpuBuf.slice(off2, bytes2));
                pass.drawIndexed(meshData.lod2Count(), c2, meshData.lod2First(), 0, 0);
            }

            if (c0w > 0 || c0s > 0) {
                pass.setPipeline(PIPELINE_OVERLAY);
                pass.setUniform("Projection", projBuf);
                pass.setUniform("Lighting", lightBuf);
                pass.setUniform("SwarmUniforms", uniformGpuBuf);
                pass.bindTexture("Sampler0", skinTexView, skinSampler);
                pass.bindTexture("Sampler2", lightmapView, lightmapSampler);
                pass.setVertexBuffer(0, meshGpuBuf.slice());
                pass.setIndexBuffer(indexGpuBuf, IndexType.SHORT);
                if (c0w > 0) {
                    pass.setVertexBuffer(1, instanceGpuBuf.slice(off0w, bytes0w));
                    pass.drawIndexed(meshData.wideOverlayCount(), c0w, meshData.wideOverlayFirst(), 0, 0);
                }
                if (c0s > 0) {
                    pass.setVertexBuffer(1, instanceGpuBuf.slice(off0s, bytes0s));
                    pass.drawIndexed(meshData.slimOverlayCount(), c0s, meshData.slimOverlayFirst(), 0, 0);
                }
            }
        }

        int totalBots = Math.max(buffer.totalBots(), liveBotPlayers.size());
        if (lastLoggedTotal != totalBots) {
            long cpuUs = (System.nanoTime() - t0) / 1000L;
            var info = device.getDeviceInfo();
            LOGGER.info(
                "[SwarmClient] Unified Mojang-baked GPU draw active: backend={}, device={}, total={}, live={}, drawn={}, lod={}/{}/{}, skins={}/18, cpuUs={}",
                info.backendName(), info.name(), totalBots, liveBotPlayers.size(), drawn,
                c0w + c0s, c1w + c1s, c2, skinsLoaded, cpuUs
            );
            lastLoggedTotal = totalBots;
        }
    }

    private void uploadInstanceBuffer(GpuDevice device, CommandEncoder encoder, ByteBuffer data, int botCount) {
        int neededBytes = botCount * SwarmBuffer.INSTANCE_BYTES;
        if (neededBytes > instanceCapacityBytes) {
            int newCap = Math.max(neededBytes, instanceCapacityBytes * 2);
            if (instanceGpuBuf != null) {
                instanceGpuBuf.close();
            }
            instanceGpuBuf = device.createBuffer(
                () -> "swarm_instances",
                GpuBuffer.USAGE_VERTEX | GpuBuffer.USAGE_COPY_DST,
                newCap
            );
            instanceCapacityBytes = newCap;
        }
        encoder.writeToBuffer(instanceGpuBuf.slice(0L, neededBytes), data);
    }

    private void initGpu(GpuDevice device, CommandEncoder encoder) {
        try {
            CompiledRenderPipeline compiledOpaque = device.precompilePipeline(PIPELINE_OPAQUE, OPAQUE_SHADER_SOURCE);
            CompiledRenderPipeline compiledOverlay = device.precompilePipeline(PIPELINE_OVERLAY, OVERLAY_SHADER_SOURCE);
            if (!compiledOpaque.isValid() || !compiledOverlay.isValid()) {
                throw new IllegalStateException("Blaze3D pipeline compilation invalid");
            }

            meshData = PlayerMesh.build();
            int vBytes = meshData.vertices().length * 4;
            ByteBuffer meshBytes = MemoryUtil.memAlloc(vBytes).order(ByteOrder.LITTLE_ENDIAN);
            meshBytes.asFloatBuffer().put(meshData.vertices());
            meshBytes.position(0).limit(vBytes);
            meshGpuBuf = device.createBuffer(() -> "swarm_mesh", GpuBuffer.USAGE_VERTEX, meshBytes);
            MemoryUtil.memFree(meshBytes);

            int iBytes = meshData.indices().length * 2;
            ByteBuffer idxBytes = MemoryUtil.memAlloc(iBytes).order(ByteOrder.LITTLE_ENDIAN);
            idxBytes.asShortBuffer().put(meshData.indices());
            idxBytes.position(0).limit(iBytes);
            indexGpuBuf = device.createBuffer(() -> "swarm_indices", GpuBuffer.USAGE_INDEX, idxBytes);
            MemoryUtil.memFree(idxBytes);

            instanceCapacityBytes = INITIAL_INSTANCE_CAPACITY_BOTS * SwarmBuffer.INSTANCE_BYTES;
            instanceGpuBuf = device.createBuffer(
                () -> "swarm_instances",
                GpuBuffer.USAGE_VERTEX | GpuBuffer.USAGE_COPY_DST,
                instanceCapacityBytes
            );

            uniformGpuBuf = device.createBuffer(
                () -> "swarm_uniforms",
                GpuBuffer.USAGE_UNIFORM | GpuBuffer.USAGE_COPY_DST,
                UBO_SIZE
            );

            ByteBuffer lightBytes = MemoryUtil.memAlloc(LIGHTING_UBO_SIZE).order(ByteOrder.LITTLE_ENDIAN);
            Vector3f d0 = new Vector3f(0.2f, 1.0f, -0.7f).normalize();
            Vector3f d1 = new Vector3f(-0.2f, 1.0f, 0.7f).normalize();
            ByteBuffer lightInit = Std140Builder.intoBuffer(lightBytes).putVec3(d0).putVec3(d1).get();
            fallbackLightGpuBuf = device.createBuffer(() -> "swarm_fallback_lights", GpuBuffer.USAGE_UNIFORM, lightInit);
            MemoryUtil.memFree(lightBytes);

            loadSkinAtlas(device, encoder);
            initialized = true;
            var info = device.getDeviceInfo();
            LOGGER.info(
                "[SwarmClient] Initialized Mojang-baked unified GPU renderer (backend={}, device={}, verts={}, indices={}, skins={}/18)",
                info.backendName(), info.name(), meshData.vertexCount(), meshData.indexCount(), skinsLoaded
            );
        } catch (Throwable t) {
            initFailed = true;
            LOGGER.error("[SwarmClient] Failed to initialize Mojang-baked GPU swarm renderer", t);
        }
    }

    private void loadSkinAtlas(GpuDevice device, CommandEncoder encoder) {
        int layerCount = DEFAULT_SKIN_PATHS.length;
        int atlasWidth = 64 * layerCount;
        var rm = Minecraft.getInstance().getResourceManager();
        skinsLoaded = 0;

        try (NativeImage atlas = new NativeImage(atlasWidth, 64, true)) {
            for (int layer = 0; layer < layerCount; layer++) {
                int[] pixels = null;
                try {
                    Identifier loc = Identifier.withDefaultNamespace(DEFAULT_SKIN_PATHS[layer]);
                    var resOpt = rm.getResource(loc);
                    if (resOpt.isPresent()) {
                        try (InputStream in = ((Resource) resOpt.get()).open(); NativeImage img = NativeImage.read(in)) {
                            if (img.getWidth() == 64 && img.getHeight() == 64) {
                                pixels = img.getPixelsABGR();
                                skinsLoaded++;
                            }
                        }
                    }
                } catch (Exception ex) {
                    LOGGER.warn("[SwarmClient] Could not read skin {}: {}", DEFAULT_SKIN_PATHS[layer], ex.toString());
                }
                if (pixels == null) {
                    pixels = fallbackSkin(layer);
                }
                int baseX = layer * 64;
                for (int y = 0; y < 64; y++) {
                    int rowOff = y * 64;
                    for (int x = 0; x < 64; x++) {
                        atlas.setPixelABGR(baseX + x, y, pixels[rowOff + x]);
                    }
                }
            }

            skinTex = device.createTexture(
                () -> "swarm_skin_atlas",
                GpuTexture.USAGE_TEXTURE_BINDING | GpuTexture.USAGE_COPY_DST,
                GpuFormat.RGBA8_UNORM,
                atlasWidth,
                64,
                1,
                1
            );
            encoder.writeToTexture(skinTex, atlas);
            skinTexView = device.createTextureView(skinTex);
            skinSampler = device.createSampler(
                AddressMode.CLAMP_TO_EDGE,
                AddressMode.CLAMP_TO_EDGE,
                FilterMode.NEAREST,
                FilterMode.NEAREST,
                1,
                OptionalDouble.empty()
            );
        }
    }

    private static int[] fallbackSkin(int seed) {
        int[] px = new int[64 * 64];
        int shirt = 0xFF000000 | (((seed * 73 + 90) & 0xFF) << 16) | (((seed * 137 + 80) & 0xFF) << 8) | ((seed * 53 + 120) & 0xFF);
        Arrays.fill(px, shirt);
        return px;
    }
}
