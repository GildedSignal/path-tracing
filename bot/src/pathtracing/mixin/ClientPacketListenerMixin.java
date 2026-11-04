package pathtracing.mixin;

import net.minecraft.client.multiplayer.ClientPacketListener;
import net.minecraft.network.protocol.game.ClientboundPlayerPositionPacket;
import net.minecraft.network.protocol.game.ClientboundSetEntityMotionPacket;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import pathtracing.Recorder;

@Mixin(ClientPacketListener.class)
abstract class ClientPacketListenerMixin {
    // RETURN runs after vanilla's main-thread check; HEAD would see each queued packet twice.
    @Inject(method = "handleMovePlayer", at = @At("RETURN"))
    private void pathtracing$position(ClientboundPlayerPositionPacket packet, CallbackInfo ci) {
        Recorder.positionPacket(packet);
    }

    @Inject(method = "handleSetEntityMotion", at = @At("RETURN"))
    private void pathtracing$motion(ClientboundSetEntityMotionPacket packet, CallbackInfo ci) {
        Recorder.motionPacket(packet);
    }
}
