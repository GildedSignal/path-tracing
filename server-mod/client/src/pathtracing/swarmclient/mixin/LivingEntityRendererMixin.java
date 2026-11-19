package pathtracing.swarmclient.mixin;

import net.minecraft.client.renderer.entity.LivingEntityRenderer;
import net.minecraft.client.renderer.entity.state.AvatarRenderState;
import net.minecraft.client.renderer.entity.state.LivingEntityRenderState;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;
import pathtracing.swarmclient.SwarmClientMod;

/** Suppresses vanilla LivingEntityRenderer body submission for SwarmBots rendered by the single GPU instancer. */
@Mixin(LivingEntityRenderer.class)
public abstract class LivingEntityRendererMixin {
    @Inject(method = "isBodyVisible(Lnet/minecraft/client/renderer/entity/state/LivingEntityRenderState;)Z", at = @At("HEAD"), cancellable = true)
    private void ptSuppressSwarmBody(LivingEntityRenderState state, CallbackInfoReturnable<Boolean> cir) {
        if (state instanceof AvatarRenderState ars && SwarmClientMod.isManagedEntityId(ars.id)) {
            ars.isInvisibleToPlayer = true;
            cir.setReturnValue(false);
        }
    }
}
