package pathtracing.mixin;

import net.minecraft.client.multiplayer.MultiPlayerGameMode;
import net.minecraft.client.player.LocalPlayer;
import net.minecraft.world.InteractionHand;
import net.minecraft.world.InteractionResult;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.entity.player.Player;
import net.minecraft.world.inventory.ContainerInput;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.phys.BlockHitResult;
import net.minecraft.world.phys.EntityHitResult;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;
import pathtracing.Tracks;

@Mixin(MultiPlayerGameMode.class)
abstract class TrackInteractionMixin {
    @Unique private ItemStack pathtracing$beforeUse;
    @Unique private ItemStack pathtracing$usingStack;
    @Unique private ItemStack pathtracing$beforeDrop;

    @Inject(method = "useItemOn", at = @At("HEAD"))
    private void pathtracing$beforeBlock(LocalPlayer player, InteractionHand hand, BlockHitResult hit, CallbackInfoReturnable<InteractionResult> ci) {
        pathtracing$usingStack = player.getItemInHand(hand);
        pathtracing$beforeUse = pathtracing$usingStack.copy();
    }

    @Inject(method = "useItem", at = @At("HEAD"))
    private void pathtracing$beforeAir(Player player, InteractionHand hand, CallbackInfoReturnable<InteractionResult> ci) {
        pathtracing$usingStack = player.getItemInHand(hand);
        pathtracing$beforeUse = pathtracing$usingStack.copy();
    }

    @Inject(method = "interact", at = @At("HEAD"))
    private void pathtracing$beforeEntity(Player player, Entity entity, EntityHitResult hit, InteractionHand hand, CallbackInfoReturnable<InteractionResult> ci) {
        pathtracing$usingStack = player.getItemInHand(hand);
        pathtracing$beforeUse = pathtracing$usingStack.copy();
    }

    @Inject(method = {"useItemOn", "useItem", "interact"}, at = @At("RETURN"))
    private void pathtracing$used(CallbackInfoReturnable<InteractionResult> ci) {
        var result = ci.getReturnValue();
        boolean usedItem = result.consumesAction()
            && (!(result instanceof InteractionResult.Success success) || success.wasItemInteraction());
        Tracks.rightClick(pathtracing$beforeUse, pathtracing$usingStack, usedItem);
    }

    @Inject(method = "handleContainerInput", at = @At("HEAD"))
    private void pathtracing$beforeContainerDrop(int menu, int slot, int button, ContainerInput input, Player player, CallbackInfo ci) {
        pathtracing$beforeDrop = ItemStack.EMPTY;
        if (input == ContainerInput.THROW && slot >= 0) {
            pathtracing$beforeDrop = player.containerMenu.getSlot(slot).getItem().copy();
        } else if (input == ContainerInput.PICKUP && slot == -999) {
            pathtracing$beforeDrop = player.containerMenu.getCarried().copy();
        }
    }

    @Inject(method = "handleContainerInput", at = @At("RETURN"))
    private void pathtracing$containerDrop(int menu, int slot, int button, ContainerInput input, Player player, CallbackInfo ci) {
        if (pathtracing$beforeDrop.isEmpty()) return;
        ItemStack after = slot >= 0 ? player.containerMenu.getSlot(slot).getItem() : player.containerMenu.getCarried();
        Tracks.itemUsed(pathtracing$beforeDrop, pathtracing$beforeDrop.getCount() - after.getCount());
    }

    @Inject(method = "handleCreativeModeItemDrop", at = @At("HEAD"))
    private void pathtracing$creativeDrop(ItemStack stack, CallbackInfo ci) {
        Tracks.itemUsed(stack, stack.getCount());
    }
}
