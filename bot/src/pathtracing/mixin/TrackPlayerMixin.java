package pathtracing.mixin;

import net.minecraft.client.player.LocalPlayer;
import net.minecraft.world.item.ItemStack;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;
import pathtracing.Tracks;

@Mixin(LocalPlayer.class)
abstract class TrackPlayerMixin {
    @Unique private ItemStack pathtracing$dropped;

    @Inject(method = "drop", at = @At("HEAD"))
    private void pathtracing$beforeDrop(boolean all, CallbackInfoReturnable<Boolean> ci) {
        pathtracing$dropped = ((LocalPlayer) (Object) this).getMainHandItem().copy();
    }

    @Inject(method = "drop", at = @At("RETURN"))
    private void pathtracing$drop(boolean all, CallbackInfoReturnable<Boolean> ci) {
        if (ci.getReturnValue()) Tracks.itemUsed(pathtracing$dropped, all ? pathtracing$dropped.getCount() : 1);
    }
}
