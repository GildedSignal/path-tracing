package pathtracing.observer;
import net.minecraft.client.Minecraft;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Pseudo;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import pathtracing.HopCapture;
@Pseudo
@Mixin(targets="pathtracing.Tracks",remap=false)
abstract class HopBaselineTrackMixin {
    @Inject(method="finish",at=@At("HEAD"),remap=false)
    private static void ptArrival(Minecraft mc,CallbackInfo ci) { HopCapture.mark("arrival"); }
}
