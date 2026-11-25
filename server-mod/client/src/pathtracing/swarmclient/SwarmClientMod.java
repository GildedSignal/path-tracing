package pathtracing.swarmclient;

import net.fabricmc.api.ClientModInitializer;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents;
import net.fabricmc.fabric.api.client.networking.v1.ClientPlayConnectionEvents;
import net.fabricmc.fabric.api.client.networking.v1.ClientPlayNetworking;
import net.fabricmc.fabric.api.client.rendering.v1.level.LevelRenderEvents;
import net.fabricmc.fabric.api.networking.v1.PayloadTypeRegistry;

/** Client entrypoint receiving SwarmFramePayloads and drawing all bots via one unified Mojang-baked GPU instancer. */
public final class SwarmClientMod implements ClientModInitializer {
    private static final SwarmBuffer BUFFER = new SwarmBuffer();
    private static final SwarmRenderer RENDERER = new SwarmRenderer(BUFFER);

    public static boolean isManagedEntityId(int entityId) {
        return RENDERER.isManagedEntityId(entityId);
    }

    @Override
    public void onInitializeClient() {
        PayloadTypeRegistry.clientboundPlay().registerLarge(
            SwarmFramePayload.TYPE,
            SwarmFramePayload.CODEC,
            SwarmFramePayload.MAX_BYTES
        );
        ClientPlayNetworking.registerGlobalReceiver(SwarmFramePayload.TYPE, (payload, context) ->
            BUFFER.enqueue(payload.data())
        );
        ClientTickEvents.END_CLIENT_TICK.register(client -> BUFFER.onClientTick());
        ClientPlayConnectionEvents.DISCONNECT.register((handler, client) -> {
            BUFFER.clear();
            RENDERER.clearManagedEntities();
        });
        LevelRenderEvents.AFTER_SOLID_FEATURES.register(RENDERER::render);
    }
}
