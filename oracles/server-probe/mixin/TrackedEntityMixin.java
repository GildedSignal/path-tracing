package pathtracing.probe.mixin;

import net.minecraft.server.level.ServerPlayer;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import pathtracing.probe.ServerProbe;

@Mixin(targets = "net.minecraft.server.level.ChunkMap$TrackedEntity")
public class TrackedEntityMixin {
    @Inject(
        method = "updatePlayer(Lnet/minecraft/server/level/ServerPlayer;)V",
        at = @At("HEAD"),
        cancellable = true
    )
    private void ptSkipBotObserver(ServerPlayer player, CallbackInfo ci) {
        if (player instanceof ServerProbe.ProbeBot) {
            ci.cancel();
        }
    }
}
