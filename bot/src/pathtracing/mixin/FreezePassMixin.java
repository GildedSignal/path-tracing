package pathtracing.mixin;
import net.minecraft.client.Minecraft;
import net.minecraft.world.entity.LivingEntity;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import pathtracing.Recorder;
@Mixin(LivingEntity.class)
abstract class FreezePassMixin {
    @Inject(method = "aiStep", at = @At(value = "INVOKE", target = "Lnet/minecraft/world/entity/LivingEntity;applyEffectsFromBlocks()V", shift = At.Shift.AFTER))
    private void pathtracing$freezePass(CallbackInfo ci) {
        if ((Object) this == Minecraft.getInstance().player) Recorder.freezePass();
    }
}
