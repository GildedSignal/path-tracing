package pathtracing.swarmclient;

import net.fabricmc.api.ClientModInitializer;
import net.fabricmc.fabric.api.client.networking.v1.ClientPlayConnectionEvents;
import net.fabricmc.fabric.api.client.networking.v1.ClientPlayNetworking;
import net.fabricmc.fabric.api.client.rendering.v1.level.LevelRenderEvents;
import net.fabricmc.fabric.api.networking.v1.PayloadTypeRegistry;

/** Client entrypoint receiving SwarmFramePayloads and drawing all bots via GPU instancing. */
public final class SwarmClientMod implements ClientModInitializer {
    private static final SwarmBuffer BUFFER = new SwarmBuffer();
    private static final SwarmRenderer RENDERER = new SwarmRenderer(BUFFER);

    @Override
    public void onInitializeClient() {
        PayloadTypeRegistry.clientboundPlay().registerLarge(
            SwarmFramePayload.TYPE,
            SwarmFramePayload.CODEC,
            SwarmFramePayload.MAX_BYTES
        );
        ClientPlayNetworking.registerGlobalReceiver(SwarmFramePayload.TYPE, (payload, context) ->
            BUFFER.ingest(payload.data())
        );
        ClientPlayConnectionEvents.DISCONNECT.register((handler, client) -> BUFFER.clear());
        LevelRenderEvents.AFTER_SOLID_FEATURES.register(RENDERER::render);
    }
}
