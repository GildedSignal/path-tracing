package pathtracing.mixin;

import net.minecraft.client.Minecraft;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.phys.HitResult;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;
import pathtracing.Tracks;

@Mixin(Minecraft.class)
abstract class TrackAttackMixin {
    @Unique private ItemStack pathtracing$attackStack;

    @Inject(method = "startAttack", at = @At("HEAD"))
    private void pathtracing$beforeAttack(CallbackInfoReturnable<Boolean> ci) {
        pathtracing$capture();
    }

    @Inject(method = "continueAttack", at = @At("HEAD"))
    private void pathtracing$beforeContinue(boolean held, CallbackInfo ci) {
        pathtracing$capture();
    }

    @Inject(method = "startAttack", at = @At(value = "INVOKE", target = "Lnet/minecraft/client/player/LocalPlayer;swing(Lnet/minecraft/world/InteractionHand;)V"))
    private void pathtracing$attack(CallbackInfoReturnable<Boolean> ci) {
        pathtracing$count();
    }

    @Inject(method = "continueAttack", at = @At(value = "INVOKE", target = "Lnet/minecraft/client/player/LocalPlayer;swing(Lnet/minecraft/world/InteractionHand;)V"))
    private void pathtracing$continue(boolean held, CallbackInfo ci) {
        pathtracing$count();
    }

    @Unique private void pathtracing$capture() {
        var player = Minecraft.getInstance().player;
        pathtracing$attackStack = player == null ? ItemStack.EMPTY : player.getMainHandItem().copy();
    }

    @Unique private void pathtracing$count() {
        var hit = Minecraft.getInstance().hitResult;
        if (hit != null && (hit.getType() == HitResult.Type.BLOCK || hit.getType() == HitResult.Type.ENTITY)) Tracks.attacked(pathtracing$attackStack);
    }
}
