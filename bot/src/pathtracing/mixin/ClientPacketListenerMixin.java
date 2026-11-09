package pathtracing.mixin;

import net.minecraft.client.multiplayer.ClientPacketListener;
import net.minecraft.network.protocol.game.ClientboundExplodePacket;
import net.minecraft.network.protocol.game.ClientboundBlockUpdatePacket;
import net.minecraft.network.protocol.game.ClientboundEntityPositionSyncPacket;
import net.minecraft.network.protocol.game.ClientboundForgetLevelChunkPacket;
import net.minecraft.network.protocol.game.ClientboundLevelChunkWithLightPacket;
import net.minecraft.network.protocol.game.ClientboundPlayerPositionPacket;
import net.minecraft.network.protocol.game.ClientboundPlayerRotationPacket;
import net.minecraft.network.protocol.game.ClientboundRespawnPacket;
import net.minecraft.network.protocol.game.ClientboundSectionBlocksUpdatePacket;
import net.minecraft.network.protocol.game.ClientboundSetEntityMotionPacket;
import net.minecraft.network.protocol.game.ClientboundSetTimePacket;
import net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket;
import net.minecraft.network.protocol.game.ClientboundUpdateAttributesPacket;
import net.minecraft.client.Minecraft;
import net.minecraft.network.protocol.game.ClientboundSystemChatPacket;
import net.minecraft.network.protocol.game.ClientboundTeleportEntityPacket;
import net.minecraft.world.entity.Entity;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.spongepowered.asm.mixin.injection.callback.LocalCapture;
import pathtracing.LandingClientTrace;
import pathtracing.Recorder;
import pathtracing.Tracks;
import pathtracing.SprintDiagnostic;

@Mixin(ClientPacketListener.class)
abstract class ClientPacketListenerMixin {
    @Inject(method = "handleUpdateAttributes", at = @At("RETURN"))
    private void ptSprintAttributes(ClientboundUpdateAttributesPacket packet, CallbackInfo ci) {
        var p = Minecraft.getInstance().player;
        if (SprintDiagnostic.ENABLED && p != null && packet.getEntityId() == p.getId()) SprintDiagnostic.observe("attributes-packet-return", p);
    }
    @Inject(method = "handleSetEntityData", at = @At("RETURN"))
    private void ptSprintEntityData(ClientboundSetEntityDataPacket packet, CallbackInfo ci) {
        var p = Minecraft.getInstance().player;
        if (SprintDiagnostic.ENABLED && p != null && packet.id() == p.getId()) SprintDiagnostic.observe("entity-data-packet-return", p);
    }
    @Inject(method = "handleSetTime", at = @At("RETURN"))
    private void pathtracing$serverTick(ClientboundSetTimePacket packet, CallbackInfo ci) {
        Tracks.serverTick();
    }

    // RETURN runs after vanilla's main-thread check; HEAD would see each queued packet twice.
    @Inject(method = "handleMovePlayer", at = @At("RETURN"))
    private void pathtracing$position(ClientboundPlayerPositionPacket packet, CallbackInfo ci) {
        LandingClientTrace.unexpected("ClientboundPlayerPositionPacket");
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
        LandingClientTrace.unexpected("ClientboundPlayerRotationPacket");
        Recorder.rotationPacket(packet);
    }

    @Inject(method = "handleExplosion", at = @At("RETURN"))
    private void pathtracing$explosion(ClientboundExplodePacket packet, CallbackInfo ci) {
        if (packet.playerKnockback().isPresent()) LandingClientTrace.unexpected("ClientboundExplodePacket");
        Recorder.explosionPacket(packet);
    }

    @Inject(method = "handleEntityPositionSync", at = @At("RETURN"))
    private void ptPositionSync(ClientboundEntityPositionSyncPacket packet, CallbackInfo ci) {
        if (LandingClientTrace.CORPUS) LandingClientTrace.ownEntityPacket(packet.id(), "ClientboundEntityPositionSyncPacket");
    }

    @Inject(method = "handleTeleportEntity", at = @At("RETURN"))
    private void ptTeleport(ClientboundTeleportEntityPacket packet, CallbackInfo ci) {
        if (LandingClientTrace.CORPUS) LandingClientTrace.ownEntityPacket(packet.id(), "ClientboundTeleportEntityPacket");
    }

    @Inject(method = "handleRespawn", at = @At("RETURN"))
    private void ptRespawn(ClientboundRespawnPacket packet, CallbackInfo ci) {
        LandingClientTrace.unexpected("ClientboundRespawnPacket");
    }

    @Inject(method = "handleBlockUpdate", at = @At("RETURN"))
    private void ptBlockUpdate(ClientboundBlockUpdatePacket packet, CallbackInfo ci) {
        if (LandingClientTrace.CORPUS) LandingClientTrace.blockUpdate(packet.getPos());
    }

    @Inject(method = "handleChunkBlocksUpdate", at = @At("RETURN"))
    private void ptSectionUpdate(ClientboundSectionBlocksUpdatePacket packet, CallbackInfo ci) {
        if (LandingClientTrace.CORPUS) packet.runUpdates((pos, state) -> LandingClientTrace.blockUpdate(pos));
    }

    @Inject(method = "handleLevelChunkWithLight", at = @At("RETURN"))
    private void ptChunkLoad(ClientboundLevelChunkWithLightPacket packet, CallbackInfo ci) {
        if (LandingClientTrace.CORPUS) LandingClientTrace.chunkUpdate(packet.getX(), packet.getZ());
    }

    @Inject(method = "handleForgetLevelChunk", at = @At("RETURN"))
    private void ptChunkUnload(ClientboundForgetLevelChunkPacket packet, CallbackInfo ci) {
        if (LandingClientTrace.CORPUS) LandingClientTrace.chunkUpdate(packet.pos().x(), packet.pos().z());
    }
}
