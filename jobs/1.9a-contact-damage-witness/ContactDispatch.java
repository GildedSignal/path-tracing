package pathtracing.contact.mixin;

import java.util.List;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.entity.Entity;
import pathtracing.contact.ContactTrace;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(Entity.class)
abstract class ContactDispatch {
    private static final String EFFECTS = "applyEffectsFromBlocks(Ljava/util/List;)V";
    private static final String STEP_ON = "Lnet/minecraft/world/level/block/Block;stepOn("
        + "Lnet/minecraft/world/level/Level;Lnet/minecraft/core/BlockPos;"
        + "Lnet/minecraft/world/level/block/state/BlockState;Lnet/minecraft/world/entity/Entity;)V";

    @Inject(method = EFFECTS, at = @At(value = "INVOKE", target = STEP_ON), allow = 1)
    private void contactBeforeStep(List<?> movements, CallbackInfo ci) {
        if ((Object) this instanceof ServerPlayer player) ContactTrace.stepBegin(player);
    }

    @Inject(method = EFFECTS, at = @At(value = "INVOKE", target = STEP_ON, shift = At.Shift.AFTER), allow = 1)
    private void contactAfterStep(List<?> movements, CallbackInfo ci) {
        if ((Object) this instanceof ServerPlayer player) ContactTrace.stepEnd(player);
    }
}
