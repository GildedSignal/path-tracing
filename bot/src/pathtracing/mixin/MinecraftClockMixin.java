package pathtracing.mixin;

import net.minecraft.client.Minecraft;
import net.minecraft.client.multiplayer.ClientLevel;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.Redirect;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;
import pathtracing.ClockAudit;

@Mixin(Minecraft.class)
abstract class MinecraftClockMixin {
    @Shadow public ClientLevel level;
    @Shadow private volatile boolean pause;

    @Inject(method = "getTickTargetMillis(F)F", at = @At("RETURN"), cancellable = true)
    private void pathtracing$followTickRate(float base, CallbackInfoReturnable<Float> result) {
        if (Boolean.getBoolean("pathtracing.followTickRate") && level != null
            && level.tickRateManager().runsNormally()) {
            result.setReturnValue(level.tickRateManager().millisecondsPerTick());
        }
    }

    // The timer has already consumed the elapsed time. Keep Mojang's loop and change its bound.
    @Redirect(method = "runTick(Z)V", at = @At(value = "INVOKE", target = "Ljava/lang/Math;min(II)I"),
        require = 1, allow = 1)
    private int pathtracing$ticksThisFrame(int vanillaLimit, int demand) {
        ClockAudit.scheduled(demand);
        if (ClockAudit.fullTickDemand() && !pause && level != null) {
            var rate = level.tickRateManager();
            if (!rate.isFrozen() && rate.runsNormally()) return demand;
        }
        return Math.min(vanillaLimit, demand);
    }

    @Inject(method = "runTick(Z)V", at = @At(value = "INVOKE",
        target = "Lnet/minecraft/client/Minecraft;tick()V", shift = At.Shift.AFTER), require = 1, allow = 1)
    private void pathtracing$countCompletedTick(boolean tick, CallbackInfo callback) {
        ClockAudit.executed();
    }

    @Inject(method = "runTick(Z)V", at = @At("RETURN"), require = 1, allow = 1)
    private void pathtracing$finishClockFrame(boolean tick, CallbackInfo callback) {
        ClockAudit.frameFinished();
    }
}
