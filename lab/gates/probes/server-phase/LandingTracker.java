package pathtracing.phase.mixin;

import net.minecraft.network.protocol.Packet;
import net.minecraft.server.level.ServerEntity;
import net.minecraft.world.entity.Entity;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.ModifyArg;
import pathtracing.phase.LandingTrace;

@Mixin(ServerEntity.class)
abstract class LandingTracker {
    @Shadow @Final private Entity entity;

    // This invocation is dominated by the hurtMarked test and its clearing write.
    @ModifyArg(method = "sendChanges", at = @At(value = "INVOKE",
        target = "Lnet/minecraft/server/level/ServerEntity$Synchronizer;sendToTrackingPlayersAndSelf(Lnet/minecraft/network/protocol/Packet;)V"))
    private Packet<?> ptConsume(Packet<?> packet) {
        if (LandingTrace.ENABLED) LandingTrace.consume(entity, packet);
        return packet;
    }
}
