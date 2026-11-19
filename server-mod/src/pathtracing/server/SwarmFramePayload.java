package pathtracing.server;

import net.minecraft.network.RegistryFriendlyByteBuf;
import net.minecraft.network.codec.StreamCodec;
import net.minecraft.network.protocol.common.custom.CustomPacketPayload;
import net.minecraft.resources.Identifier;

/** Compact 20 bytes/bot + 32B header frame forwarded directly from Rust shm to modded clients. */
public record SwarmFramePayload(byte[] data) implements CustomPacketPayload {
    public static final int MAX_BYTES = 2 * 1024 * 1024;
    public static final Type<SwarmFramePayload> TYPE = new Type<>(
        Identifier.fromNamespaceAndPath("pathtracing", "swarm_frame")
    );
    public static final StreamCodec<RegistryFriendlyByteBuf, SwarmFramePayload> CODEC = StreamCodec.of(
        (buf, payload) -> buf.writeByteArray(payload.data),
        buf -> new SwarmFramePayload(buf.readByteArray(MAX_BYTES))
    );

    @Override
    public Type<? extends CustomPacketPayload> type() {
        return TYPE;
    }
}
