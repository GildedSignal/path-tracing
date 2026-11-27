package pathtracing.mixin;

import net.minecraft.client.multiplayer.MultiPlayerGameMode;
import net.minecraft.client.player.LocalPlayer;
import net.minecraft.world.item.ItemStack;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import pathtracing.MinecraftApi;
import pathtracing.Tracks;

@Mixin(MultiPlayerGameMode.class)
abstract class TrackPlayerMixin {
    @Unique private ItemStack pathtracing$dropped;

    @Inject(method = "dropItem", at = @At("HEAD"))
    private void pathtracing$beforeDrop(LocalPlayer player, boolean all, CallbackInfo ci) {
        pathtracing$dropped = player.getMainHandItem().copy();
    }

    @Inject(method = "dropItem", at = @At(value = "INVOKE", target = MinecraftApi.SWING))
    private void pathtracing$drop(LocalPlayer player, boolean all, CallbackInfo ci) {
        Tracks.itemUsed(pathtracing$dropped, all ? pathtracing$dropped.getCount() : 1);
    }
}
