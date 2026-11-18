package pathtracing.probe.mixin;

import net.minecraft.server.level.ServerPlayer;
import net.minecraft.server.waypoints.ServerWaypointManager;
import net.minecraft.world.waypoints.WaypointTransmitter;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import pathtracing.probe.ServerProbe;

@Mixin(ServerWaypointManager.class)
public class WaypointManagerMixin {
    @Inject(
        method = "addPlayer(Lnet/minecraft/server/level/ServerPlayer;)V",
        at = @At("HEAD"),
        cancellable = true
    )
    private void ptSkipBotAdd(ServerPlayer player, CallbackInfo ci) {
        if (player instanceof ServerProbe.ProbeBot) {
            if (player.isTransmittingWaypoint()) {
                ((ServerWaypointManager) (Object) this).trackWaypoint((WaypointTransmitter) player);
            }
            ci.cancel();
        }
    }

    @Inject(
        method = "updatePlayer(Lnet/minecraft/server/level/ServerPlayer;)V",
        at = @At("HEAD"),
        cancellable = true
    )
    private void ptSkipBotUpdate(ServerPlayer player, CallbackInfo ci) {
        if (player instanceof ServerProbe.ProbeBot) {
            ci.cancel();
        }
    }

    @Inject(
        method = "removePlayer(Lnet/minecraft/server/level/ServerPlayer;)V",
        at = @At("HEAD"),
        cancellable = true
    )
    private void ptSkipBotRemove(ServerPlayer player, CallbackInfo ci) {
        if (player instanceof ServerProbe.ProbeBot) {
            ((ServerWaypointManager) (Object) this).untrackWaypoint((WaypointTransmitter) player);
            ci.cancel();
        }
    }
}
