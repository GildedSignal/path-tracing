package pathtracing.observer;
import net.minecraft.client.KeyMapping;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Pseudo;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Redirect;
import pathtracing.HopCapture;
/** Applied only beside the frozen baseline jar. Original control remains intact. */
@Pseudo
@Mixin(targets="pathtracing.Pointer",remap=false)
abstract class HopBaselinePointerMixin {
    @Redirect(method="tick",at=@At(value="INVOKE",target="Lnet/minecraft/client/KeyMapping;consumeClick()Z"),remap=false)
    private static boolean ptIssue(KeyMapping key) {
        boolean clicked=key.consumeClick();
        if(clicked) HopCapture.issuance();
        return clicked;
    }
}
