package pathtracing.phase.mixin;

import net.minecraft.world.entity.Entity;
import net.minecraft.world.phys.Vec3;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import pathtracing.phase.LandingTrace;

@Mixin(Entity.class)
abstract class LandingEntity {
    @Unique private boolean ptBeforeMark;

    @Inject(method = "markHurt", at = @At("HEAD"))
    private void ptBeforeMark(CallbackInfo ci) {
        if (LandingTrace.ENABLED) ptBeforeMark = ((Entity) (Object) this).hurtMarked;
    }

    @Inject(method = "markHurt", at = @At("RETURN"))
    private void ptAfterMark(CallbackInfo ci) {
        if (LandingTrace.ENABLED) LandingTrace.mark((Entity) (Object) this, ptBeforeMark);
    }

    @Inject(method = "setDeltaMovement(Lnet/minecraft/world/phys/Vec3;)V", at = @At("HEAD"))
    private void ptVelocity(Vec3 movement, CallbackInfo ci) {
        if (LandingTrace.ENABLED) LandingTrace.velocity(this, movement);
    }
}
