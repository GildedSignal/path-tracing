package pathtracing.phase.mixin;

import net.minecraft.world.damagesource.DamageSource;
import net.minecraft.world.entity.LivingEntity;
import net.minecraft.world.entity.monster.Ravager;
import org.objectweb.asm.Opcodes;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import pathtracing.phase.LandingTrace;

@Mixin(Ravager.class)
abstract class LandingRavager {
    // Unlike the other vanilla writers of true, this path can mark after a no-op push.
    @Inject(method = "blockedByItem", at = @At(value = "FIELD", opcode = Opcodes.PUTFIELD,
        target = "Lnet/minecraft/world/entity/LivingEntity;hurtMarked:Z"))
    private void ptMark(LivingEntity defender, DamageSource source, float damage, CallbackInfo ci) {
        if (LandingTrace.ENABLED) LandingTrace.competingMark(defender, "Ravager.blockedByItem");
    }
}
