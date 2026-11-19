package pathtracing.swarmclient.mixin;

import net.minecraft.client.renderer.entity.player.AvatarRenderer;
import net.minecraft.client.renderer.entity.state.AvatarRenderState;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;
import pathtracing.swarmclient.SwarmClientMod;

/** Suppresses vanilla AvatarRenderer armor/cape feature layers for SwarmBots rendered by the single GPU instancer. */
@Mixin(AvatarRenderer.class)
public abstract class AvatarRendererMixin {
    @Inject(method = "shouldRenderLayers(Lnet/minecraft/client/renderer/entity/state/AvatarRenderState;)Z", at = @At("HEAD"), cancellable = true)
    private void ptSuppressSwarmLayers(AvatarRenderState state, CallbackInfoReturnable<Boolean> cir) {
        if (SwarmClientMod.isManagedEntityId(state.id)) {
            cir.setReturnValue(false);
        }
    }
}
