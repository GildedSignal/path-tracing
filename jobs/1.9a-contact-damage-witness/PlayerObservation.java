package pathtracing.contact.mixin;

import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.damagesource.DamageSource;
import pathtracing.contact.ContactTrace;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

@Mixin(ServerPlayer.class)
abstract class PlayerObservation {
    @Inject(method = "doTick()V", at = @At("HEAD"), allow = 1)
    private void contactBeginTick(CallbackInfo ci) {
        ContactTrace.begin((ServerPlayer) (Object) this);
    }

    @Inject(method = "doTick()V", at = @At("RETURN"), allow = 1)
    private void contactEndTick(CallbackInfo ci) {
        ContactTrace.end((ServerPlayer) (Object) this);
    }

    @Inject(method = "hurtServer", at = @At("HEAD"), allow = 1)
    private void contactBeforeDamage(ServerLevel level, DamageSource source, float amount,
                                     CallbackInfoReturnable<Boolean> cir) {
        ContactTrace.damageBegin((ServerPlayer) (Object) this, source, amount);
    }

    @Inject(method = "hurtServer", at = @At("RETURN"))
    private void contactAfterDamage(ServerLevel level, DamageSource source, float amount,
                                    CallbackInfoReturnable<Boolean> cir) {
        ContactTrace.damageEnd((ServerPlayer) (Object) this, source, cir.getReturnValueZ());
    }
}
