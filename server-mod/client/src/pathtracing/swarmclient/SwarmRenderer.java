package pathtracing.swarmclient;

import com.mojang.blaze3d.GpuFormat;
import com.mojang.blaze3d.PrimitiveTopology;
import com.mojang.blaze3d.buffers.GpuBuffer;
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
import java.nio.FloatBuffer;
import java.util.Arrays;
import java.util.Optional;
import java.util.OptionalDouble;
import net.fabricmc.fabric.api.client.rendering.v1.level.LevelRenderContext;
import net.minecraft.client.Minecraft;
import net.minecraft.client.renderer.LevelTargetBundle;
import net.minecraft.client.renderer.fog.FogData;
import net.minecraft.client.renderer.state.level.CameraRenderState;
import net.minecraft.resources.Identifier;
import net.minecraft.server.packs.resources.Resource;
import org.joml.Matrix4f;
import org.lwjgl.system.MemoryUtil;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import pathtracing.swarmclient.mixin.LevelRendererAccessor;

/** Backend-agnostic Blaze3D instanced renderer drawing up to 10,000 articulated bots in 1 draw call (Vulkan & OpenGL). */
public final class SwarmRenderer {
    private static final Logger LOGGER = LoggerFactory.getLogger("pathtracing_swarm_client");
    private static final int UBO_SIZE = 112;
    private static final int INITIAL_INSTANCE_CAPACITY_BOTS = 16384;
    private static final String[] WIDE_SKINS = {
        "textures/entity/player/wide/steve.png",
        "textures/entity/player/wide/alex.png",
        "textures/entity/player/wide/ari.png",
        "textures/entity/player/wide/efe.png",
        "textures/entity/player/wide/kai.png",
        "textures/entity/player/wide/makena.png",
        "textures/entity/player/wide/noor.png",
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

        in vec3 InstPrevPos;
        in vec3 InstCurrPos;
        in vec4 InstRotAmp;
        in vec4 InstPhaseMeta;

        layout(std140) uniform SwarmUniforms {
            mat4 uViewProj;
            vec4 uOriginRelCamAndAlpha;
            vec4 uFogColor;
            vec4 uFogParams;
        };

        out vec2 vUv;
        out vec3 vNormal;
        out float vSkinLayer;
        out float vCamDist;

        vec3 rotXAroundPivot(vec3 p, vec3 pivot, float angle) {
            vec3 d = p - pivot;
            float c = cos(angle);
            float s = sin(angle);
            return pivot + vec3(d.x, d.y * c + d.z * s, -d.y * s + d.z * c);
        }

        vec3 rotNormalX(vec3 n, float angle) {
            float c = cos(angle);
            float s = sin(angle);
            return vec3(n.x, n.y * c + n.z * s, -n.y * s + n.z * c);
        }

        void main() {
            int flags = int(InstPhaseMeta.w + 0.5);
            if ((flags & 8) != 0) {
                gl_Position = vec4(2.0, 2.0, 2.0, 1.0);
                vUv = vec2(0.0);
                vNormal = vec3(0.0, 1.0, 0.0);
                vSkinLayer = 0.0;
                vCamDist = 0.0;
                return;
            }

            float partialTick = uOriginRelCamAndAlpha.w;
            float yaw = mix(InstRotAmp.x, InstRotAmp.y, partialTick);
            float headPitch = InstRotAmp.z;
            float swingAmp = InstRotAmp.w;
            float phase = mix(InstPhaseMeta.x, InstPhaseMeta.y, partialTick);
            bool sneaking = (flags & 4) != 0;

            float legSwing = sin(phase) * 0.72 * swingAmp;
            float armSwing = legSwing * 0.85;

            int part = int(PartId + 0.5);
            vec3 localP = LocalPos;
            vec3 localN = Normal;

            if (part == 0) {
                localP = rotXAroundPivot(localP, Pivot, -headPitch);
                localN = rotNormalX(localN, -headPitch);
            } else if (part == 2) {
                float a = -armSwing + (sneaking ? 0.4 : 0.0);
                localP = rotXAroundPivot(localP, Pivot, a);
                localN = rotNormalX(localN, a);
            } else if (part == 3) {
                float a = armSwing + (sneaking ? 0.4 : 0.0);
                localP = rotXAroundPivot(localP, Pivot, a);
                localN = rotNormalX(localN, a);
            } else if (part == 4) {
                localP = rotXAroundPivot(localP, Pivot, legSwing);
                localN = rotNormalX(localN, legSwing);
            } else if (part == 5) {
                localP = rotXAroundPivot(localP, Pivot, -legSwing);
                localN = rotNormalX(localN, -legSwing);
            }

            if (sneaking) {
                localP.y -= 0.125;
            }

            float cy = cos(yaw);
            float sy = sin(yaw);
            vec3 rotated = vec3(
                localP.x * cy - localP.z * sy,
                localP.y,
                localP.x * sy + localP.z * cy
            );
            vec3 worldNorm = vec3(
                localN.x * cy - localN.z * sy,
                localN.y,
                localN.x * sy + localN.z * cy
            );

            vec3 botPos = mix(InstPrevPos, InstCurrPos, partialTick);
            vec3 relCam = uOriginRelCamAndAlpha.xyz + botPos + rotated;
            gl_Position = uViewProj * vec4(relCam, 1.0);
            vUv = UV0;
            vNormal = worldNorm;
            vSkinLayer = InstPhaseMeta.z;
            vCamDist = length(relCam);
        }
        """;

    private static final String FRAG_SHADER = """
        #version 330
        in vec2 vUv;
        in vec3 vNormal;
        in float vSkinLayer;
        in float vCamDist;

        layout(std140) uniform SwarmUniforms {
            mat4 uViewProj;
            vec4 uOriginRelCamAndAlpha;
            vec4 uFogColor;
            vec4 uFogParams;
        };

        uniform sampler2D Sampler0;

        out vec4 fragColor;

        void main() {
            vec2 atlasUv = vec2(
                (floor(vSkinLayer + 0.5) + clamp(vUv.x, 0.001, 0.999)) / 9.0,
                clamp(vUv.y, 0.001, 0.999)
            );
            vec4 texel = texture(Sampler0, atlasUv);
            if (texel.a < 0.1) {
                discard;
            }
            vec3 n = normalize(vNormal);
            vec3 sunDir = normalize(vec3(0.25, 1.0, 0.35));
            float shade = clamp(0.62 + max(dot(n, sunDir), 0.0) * 0.38 - max(-n.y, 0.0) * 0.10, 0.48, 1.0);
            vec3 lit = texel.rgb * shade;

            float fogStart = uFogParams.x;
            float fogEnd = uFogParams.y;
            if (fogEnd > fogStart && uFogColor.a > 0.001) {
                float fog = clamp((vCamDist - fogStart) / (fogEnd - fogStart), 0.0, 1.0) * uFogColor.a;
                lit = mix(lit, uFogColor.rgb, fog);
            }
            fragColor = vec4(lit, 1.0);
        }
        """;

    private static final ShaderSource SHADER_SOURCE = (id, type) -> switch (type) {
        case VERTEX -> VERT_SHADER;
        case FRAGMENT -> FRAG_SHADER;
    };

    private static final VertexFormat MESH_FORMAT = VertexFormat.builder(0)
        .addAttribute("LocalPos", GpuFormat.RGB32_FLOAT)
        .addAttribute("Pivot", GpuFormat.RGB32_FLOAT)
        .addAttribute("Normal", GpuFormat.RGB32_FLOAT)
        .addAttribute("UV0", GpuFormat.RG32_FLOAT)
        .addAttribute("PartId", GpuFormat.R32_FLOAT)
        .build();

    private static final VertexFormat INSTANCE_FORMAT = VertexFormat.builder(1)
        .addAttribute("InstPrevPos", GpuFormat.RGB32_FLOAT)
        .addAttribute("InstCurrPos", GpuFormat.RGB32_FLOAT)
        .addAttribute("InstRotAmp", GpuFormat.RGBA32_FLOAT)
        .addAttribute("InstPhaseMeta", GpuFormat.RGBA32_FLOAT)
        .build();

    private static final BindGroupLayout BIND_GROUP_LAYOUT = BindGroupLayout.builder()
        .withUniform("SwarmUniforms", UniformType.UNIFORM_BUFFER)
        .withSampler("Sampler0")
        .build();

    private static final RenderPipeline PIPELINE = RenderPipeline.builder()
        .withLocation(Identifier.fromNamespaceAndPath("pathtracing", "pipeline/swarm_bots"))
        .withVertexShader(Identifier.fromNamespaceAndPath("pathtracing", "core/swarm_bots"))
        .withFragmentShader(Identifier.fromNamespaceAndPath("pathtracing", "core/swarm_bots"))
        .withBindGroupLayout(BIND_GROUP_LAYOUT)
        .withVertexBinding(0, MESH_FORMAT)
        .withVertexBinding(1, INSTANCE_FORMAT)
        .withPrimitiveTopology(PrimitiveTopology.TRIANGLES)
        .withCull(true)
        .withColorTargetState(ColorTargetState.DEFAULT)
        .withDepthStencilState(DepthStencilState.DEFAULT)
        .build();

    private final SwarmBuffer buffer;
    private boolean initialized = false;
    private boolean initFailed = false;
    private GpuBuffer meshGpuBuf;
    private GpuBuffer instanceGpuBuf;
    private int instanceCapacityBytes = 0;
    private GpuBuffer uniformGpuBuf;
    private GpuTexture skinTex;
    private GpuTextureView skinTexView;
    private GpuSampler skinSampler;
    private int skinsLoaded = 0;
    private int meshVertexCount = 0;

    private final ByteBuffer uboBytes = MemoryUtil.memAlloc(UBO_SIZE).order(ByteOrder.LITTLE_ENDIAN);
    private final Matrix4f viewProj = new Matrix4f();
    private final float[] relOriginAndAlpha = new float[4];
    private int lastLoggedCount = -1;

    public SwarmRenderer(SwarmBuffer buffer) {
        this.buffer = buffer;
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

        GpuDevice device = RenderSystem.getDevice();
        CommandEncoder encoder = device.createCommandEncoder();

        if (!initialized) {
            initGpu(device, encoder);
            if (initFailed) {
                return;
            }
        }

        CompiledRenderPipeline compiled = device.precompilePipeline(PIPELINE, SHADER_SOURCE);
        if (!compiled.isValid()) {
            LOGGER.error("[SwarmClient] Pipeline {} became invalid; disabling renderer", PIPELINE.getLocation());
            initFailed = true;
            return;
        }

        long t0 = System.nanoTime();
        int count = buffer.prepareDraw(
            camState.pos.x,
            camState.pos.y,
            camState.pos.z,
            relOriginAndAlpha,
            (ByteBuffer data, int botCount) -> uploadInstanceBuffer(device, encoder, data, botCount)
        );
        if (count <= 0) {
            return;
        }

        viewProj.set(camState.projectionMatrix).mul(camState.viewRotationMatrix);
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
            .putMat4f(viewProj)
            .putVec4(relOriginAndAlpha[0], relOriginAndAlpha[1], relOriginAndAlpha[2], relOriginAndAlpha[3])
            .putVec4(fogR, fogG, fogB, fogA)
            .putVec4(fogStart, fogEnd, 0f, 0f)
            .get();
        encoder.writeToBuffer(uniformGpuBuf.slice(), uboSliceData);

        try (RenderPass pass = encoder.createRenderPass(
            () -> "swarm_bots",
            mainRt.getColorTextureView(),
            Optional.empty(),
            mainRt.getDepthTextureView(),
            OptionalDouble.empty()
        )) {
            pass.setPipeline(PIPELINE);
            pass.setUniform("SwarmUniforms", uniformGpuBuf);
            pass.bindTexture("Sampler0", skinTexView, skinSampler);
            pass.setVertexBuffer(0, meshGpuBuf.slice());
            pass.setVertexBuffer(1, instanceGpuBuf.slice(0L, (long) count * SwarmBuffer.INSTANCE_BYTES));
            pass.draw(meshVertexCount, count, 0, 0);
        }

        if (lastLoggedCount != count) {
            long cpuUs = (System.nanoTime() - t0) / 1000L;
            LOGGER.info(
                "[SwarmClient] Blaze3D instanced draw active: backend={}, count={}, skins={}, cpuUs={}",
                device.getDeviceInfo().backendName(), count, skinsLoaded, cpuUs
            );
            lastLoggedCount = count;
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
            CompiledRenderPipeline compiled = device.precompilePipeline(PIPELINE, SHADER_SOURCE);
            if (!compiled.isValid()) {
                throw new IllegalStateException("Blaze3D pipeline compilation returned invalid pipeline");
            }

            float[] mesh = PlayerMesh.build();
            meshVertexCount = mesh.length / PlayerMesh.FLOATS_PER_VERTEX;
            ByteBuffer meshBytes = MemoryUtil.memAlloc(mesh.length * 4).order(ByteOrder.LITTLE_ENDIAN);
            FloatBuffer meshFloats = meshBytes.asFloatBuffer();
            meshFloats.put(mesh);
            meshBytes.position(0);
            meshBytes.limit(mesh.length * 4);
            meshGpuBuf = device.createBuffer(() -> "swarm_mesh", GpuBuffer.USAGE_VERTEX, meshBytes);
            MemoryUtil.memFree(meshBytes);

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

            loadSkinAtlas(device, encoder);
            initialized = true;
            LOGGER.info(
                "[SwarmClient] Initialized Blaze3D swarm renderer (backend={}, verts={}, skins={}/9)",
                device.getDeviceInfo().backendName(), meshVertexCount, skinsLoaded
            );
        } catch (Throwable t) {
            initFailed = true;
            LOGGER.error("[SwarmClient] Failed to initialize Blaze3D swarm renderer", t);
        }
    }

    private void loadSkinAtlas(GpuDevice device, CommandEncoder encoder) {
        int layerCount = WIDE_SKINS.length;
        int atlasWidth = 64 * layerCount;
        var rm = Minecraft.getInstance().getResourceManager();
        skinsLoaded = 0;

        try (NativeImage atlas = new NativeImage(atlasWidth, 64, true)) {
            for (int layer = 0; layer < layerCount; layer++) {
                int[] pixels = null;
                try {
                    Identifier loc = Identifier.withDefaultNamespace(WIDE_SKINS[layer]);
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
                    LOGGER.warn("[SwarmClient] Could not read skin {}: {}", WIDE_SKINS[layer], ex.toString());
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
