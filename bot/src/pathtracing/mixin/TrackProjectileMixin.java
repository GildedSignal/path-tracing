package pathtracing.mixin;

import net.minecraft.client.Minecraft;
import net.minecraft.world.entity.LivingEntity;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.item.ProjectileWeaponItem;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;
import pathtracing.Tracks;

@Mixin(ProjectileWeaponItem.class)
abstract class TrackProjectileMixin {
    @Inject(method = "useAmmo", at = @At("RETURN"))
    private static void pathtracing$ammunition(ItemStack weapon, ItemStack ammunition, LivingEntity user, boolean copy, CallbackInfoReturnable<ItemStack> ci) {
        if (user == Minecraft.getInstance().player) Tracks.itemUsed(ci.getReturnValue(), ci.getReturnValue().getCount());
    }
}
