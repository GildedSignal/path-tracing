package pathtracing.phase.mixin;

import net.minecraft.server.level.ServerPlayer;
import org.spongepowered.asm.mixin.Mixin;
import pathtracing.phase.ResourceTrace;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(ServerPlayer.class)
public abstract class RegenerationProbe {
    @Inject(method = "tickRegeneration", at = @At("HEAD"))
    private void ptBeforeRegeneration(CallbackInfo ci) {
        ResourceTrace.beforeRegeneration((ServerPlayer)(Object)this);
    }

    @Inject(method = "tickRegeneration", at = @At("RETURN"))
    private void ptAfterRegeneration(CallbackInfo ci) {
        ResourceTrace.afterRegeneration((ServerPlayer)(Object)this);
    }
}
