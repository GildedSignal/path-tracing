package pathtracing.mixin;

import net.minecraft.world.entity.Entity;
import net.minecraft.world.entity.LivingEntity;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import pathtracing.SprintDiagnostic;

@Mixin(LivingEntity.class)
abstract class SprintDiagnosticMixin {
    @Inject(method = "setSprinting", at = @At("HEAD"))
    private void ptSprintBefore(boolean value, CallbackInfo ci) {
        if (SprintDiagnostic.ENABLED) {
            var caller = Thread.currentThread().getStackTrace()[3];
            SprintDiagnostic.observe("setSprinting(" + value + ") caller=" + caller, (Entity)(Object)this);
        }
    }
    @Inject(method = "setSprinting", at = @At("RETURN"))
    private void ptSprintAfter(boolean value, CallbackInfo ci) {
        SprintDiagnostic.observe("setSprinting-return", (Entity)(Object)this);
    }
}
