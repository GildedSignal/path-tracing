package pathtracing.mixin;

import net.minecraft.client.multiplayer.ClientPacketListener;
import net.minecraft.network.protocol.game.ClientboundExplodePacket;
import net.minecraft.network.protocol.game.ClientboundPlayerPositionPacket;
import net.minecraft.network.protocol.game.ClientboundPlayerRotationPacket;
import net.minecraft.network.protocol.game.ClientboundSetEntityMotionPacket;
import net.minecraft.network.protocol.game.ClientboundSetTimePacket;
import net.minecraft.network.protocol.game.ClientboundSystemChatPacket;
import net.minecraft.world.entity.Entity;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.spongepowered.asm.mixin.injection.callback.LocalCapture;
import pathtracing.LandingClientTrace;
import pathtracing.Recorder;
import pathtracing.Tracks;

@Mixin(ClientPacketListener.class)
abstract class ClientPacketListenerMixin {
    @Inject(method = "handleSetTime", at = @At("RETURN"))
    private void pathtracing$serverTick(ClientboundSetTimePacket packet, CallbackInfo ci) {
        Tracks.serverTick();
    }

    // RETURN runs after vanilla's main-thread check; HEAD would see each queued packet twice.
    @Inject(method = "handleMovePlayer", at = @At("RETURN"))
    private void pathtracing$position(ClientboundPlayerPositionPacket packet, CallbackInfo ci) {
        Recorder.positionPacket(packet);
    }

    @Inject(method = "handleSetEntityMotion", at = @At("RETURN"))
    private void pathtracing$motion(ClientboundSetEntityMotionPacket packet, CallbackInfo ci) {
        Recorder.motionPacket(packet);
    }

    @Inject(method = "handleSystemChat", at = @At("RETURN"))
    private void ptLandingFence(ClientboundSystemChatPacket packet, CallbackInfo ci) {
        if (LandingClientTrace.ENABLED) LandingClientTrace.fence(packet.content().getString());
    }

    @Inject(method = "handleSetEntityMotion", at = @At(value = "INVOKE",
        target = "Lnet/minecraft/world/entity/Entity;lerpMotion(Lnet/minecraft/world/phys/Vec3;)V"),
        locals = LocalCapture.CAPTURE_FAILHARD)
    private void ptBeforeMotion(ClientboundSetEntityMotionPacket packet, CallbackInfo ci, Entity target) {
        if (LandingClientTrace.ENABLED) LandingClientTrace.beforeMotion(packet, target);
    }

    @Inject(method = "handleSetEntityMotion", at = @At(value = "INVOKE",
        target = "Lnet/minecraft/world/entity/Entity;lerpMotion(Lnet/minecraft/world/phys/Vec3;)V",
        shift = At.Shift.AFTER), locals = LocalCapture.CAPTURE_FAILHARD)
    private void ptAfterMotion(ClientboundSetEntityMotionPacket packet, CallbackInfo ci, Entity target) {
        if (LandingClientTrace.ENABLED) LandingClientTrace.afterMotion(packet, target);
    }

    @Inject(method = "handleRotatePlayer", at = @At("RETURN"))
    private void pathtracing$rotation(ClientboundPlayerRotationPacket packet, CallbackInfo ci) {
        Recorder.rotationPacket(packet);
    }

    @Inject(method = "handleExplosion", at = @At("RETURN"))
    private void pathtracing$explosion(ClientboundExplodePacket packet, CallbackInfo ci) {
        Recorder.explosionPacket(packet);
    }
}
