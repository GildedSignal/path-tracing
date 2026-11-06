package pathtracing.mixin;

import net.minecraft.client.Minecraft;
import net.minecraft.client.multiplayer.ClientLevel;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

@Mixin(Minecraft.class)
abstract class MinecraftClockMixin {
    @Shadow public ClientLevel level;

    @Inject(method = "getTickTargetMillis(F)F", at = @At("RETURN"), cancellable = true)
    private void pathtracing$followTickRate(float base, CallbackInfoReturnable<Float> result) {
        if (Boolean.getBoolean("pathtracing.followTickRate") && level != null
            && level.tickRateManager().runsNormally()) {
            result.setReturnValue(level.tickRateManager().millisecondsPerTick());
        }
    }
}
