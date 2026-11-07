package pathtracing.phase.mixin;

import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.damagesource.DamageSource;
import net.minecraft.world.entity.LivingEntity;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;
import pathtracing.phase.LandingTrace;

@Mixin(LivingEntity.class)
abstract class LandingHurt {
    @Inject(method = "hurtServer", at = @At(value = "INVOKE",
        target = "Lnet/minecraft/world/entity/LivingEntity;markHurt()V"))
    private void ptHurt(ServerLevel level, DamageSource source, float amount, CallbackInfoReturnable<Boolean> ci) {
        if (LandingTrace.ENABLED) LandingTrace.markSource(this, source);
    }

    @Inject(method = "hurtServer", at = @At(value = "INVOKE",
        target = "Lnet/minecraft/world/entity/LivingEntity;markHurt()V", shift = At.Shift.AFTER))
    private void ptAfterHurt(ServerLevel level, DamageSource source, float amount, CallbackInfoReturnable<Boolean> ci) {
        if (LandingTrace.ENABLED) LandingTrace.markSource(this, null);
    }
}
