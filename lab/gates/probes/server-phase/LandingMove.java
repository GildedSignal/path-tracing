package pathtracing.phase.mixin;

import net.minecraft.network.protocol.game.ServerboundMovePlayerPacket;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.server.network.ServerGamePacketListenerImpl;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import pathtracing.phase.LandingTrace;

@Mixin(ServerGamePacketListenerImpl.class)
abstract class LandingMove {
    @Shadow public ServerPlayer player;
    @Unique private long ptMoveOrdinal;

    @Inject(method = "handleMovePlayer", at = @At(value = "INVOKE",
        target = "Lnet/minecraft/network/protocol/PacketUtils;ensureRunningOnSameThread(Lnet/minecraft/network/protocol/Packet;Lnet/minecraft/network/PacketListener;Lnet/minecraft/server/level/ServerLevel;)V",
        shift = At.Shift.AFTER))
    private void ptMove(ServerboundMovePlayerPacket packet, CallbackInfo ci) {
        if (LandingTrace.ENABLED) ptMoveOrdinal++;
    }

    // The first doCheckFallDamage follows a correction teleport. The second follows acceptance.
    @Inject(method = "handleMovePlayer", at = @At(value = "INVOKE", ordinal = 1,
        target = "Lnet/minecraft/server/level/ServerPlayer;doCheckFallDamage(DDDZ)V"))
    private void ptAccepted(ServerboundMovePlayerPacket packet, CallbackInfo ci) {
        if (LandingTrace.ENABLED) LandingTrace.accepted(player, packet, ptMoveOrdinal);
    }

    @Inject(method = "handleMovePlayer", at = @At(value = "INVOKE", ordinal = 1,
        target = "Lnet/minecraft/server/level/ServerPlayer;doCheckFallDamage(DDDZ)V", shift = At.Shift.AFTER))
    private void ptAfterAccepted(ServerboundMovePlayerPacket packet, CallbackInfo ci) {
        if (LandingTrace.ENABLED) LandingTrace.endAccepted(player);
    }
}
