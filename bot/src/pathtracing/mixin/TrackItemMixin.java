package pathtracing.mixin;

import net.minecraft.client.Minecraft;
import net.minecraft.world.entity.LivingEntity;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.level.Level;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;
import pathtracing.Tracks;

@Mixin(ItemStack.class)
abstract class TrackItemMixin {
    @Inject(method = "finishUsingItem", at = @At("HEAD"))
    private void pathtracing$consume(Level level, LivingEntity user, CallbackInfoReturnable<ItemStack> ci) {
        if (user == Minecraft.getInstance().player) Tracks.completedUse((ItemStack) (Object) this);
    }

    @Inject(method = "releaseUsing", at = @At("HEAD"))
    private void pathtracing$release(Level level, LivingEntity user, int remaining, CallbackInfo ci) {
        if (user == Minecraft.getInstance().player) Tracks.completedUse((ItemStack) (Object) this);
    }
}
