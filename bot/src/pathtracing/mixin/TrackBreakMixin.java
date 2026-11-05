package pathtracing.mixin;

import net.minecraft.client.Minecraft;
import net.minecraft.world.entity.LivingEntity;
import net.minecraft.world.item.ItemStack;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import pathtracing.Tracks;

/** The server's equipment-break event reaches this client-side method. */
@Mixin(LivingEntity.class)
abstract class TrackBreakMixin {
    @Inject(method = "breakItem", at = @At("HEAD"))
    private void pathtracing$broken(ItemStack stack, CallbackInfo ci) {
        if ((Object) this == Minecraft.getInstance().player) Tracks.itemBroken(stack);
    }
}
