package pathtracing.mixin;

import io.netty.channel.ChannelFutureListener;
import net.minecraft.network.Connection;
import net.minecraft.network.protocol.Packet;
import net.minecraft.network.protocol.game.ServerboundAcceptTeleportationPacket;
import net.minecraft.network.protocol.game.ServerboundMovePlayerPacket;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import pathtracing.LandingClientTrace;
import pathtracing.Recorder;

@Mixin(Connection.class)
abstract class LandingMoveMixin {
    @Inject(method = "send(Lnet/minecraft/network/protocol/Packet;Lio/netty/channel/ChannelFutureListener;Z)V", at = @At("HEAD"))
    private void ptMove(Packet<?> packet, ChannelFutureListener listener, boolean flush, CallbackInfo ci) {
        if (LandingClientTrace.ENABLED && packet instanceof ServerboundMovePlayerPacket move) {
            LandingClientTrace.move((Connection) (Object) this, move);
        }
        // The client's acknowledgement of a correction, sent inside handleMovePlayer (offsets 56, 92).
        if (packet instanceof ServerboundAcceptTeleportationPacket accept) Recorder.acceptTeleport(accept);
        else if (packet instanceof ServerboundMovePlayerPacket move) Recorder.acknowledgeMove(move);
    }
}
