package pathtracing.phase.mixin;

import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.food.FoodData;
import org.spongepowered.asm.mixin.Mixin;
import pathtracing.phase.ResourceTrace;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(FoodData.class)
public abstract class FoodProbe {
    @Inject(method = "addExhaustion", at = @At("HEAD"))
    private void ptCharge(float amount, CallbackInfo ci) {
        ResourceTrace.charge(this, amount);
    }

    @Inject(method = "tick", at = @At("HEAD"))
    private void ptBeforeFood(ServerPlayer player, CallbackInfo ci) {
        ResourceTrace.beforeFood(player);
    }

    @Inject(method = "tick", at = @At("RETURN"))
    private void ptAfterFood(ServerPlayer player, CallbackInfo ci) {
        ResourceTrace.afterFood(player);
    }
}
