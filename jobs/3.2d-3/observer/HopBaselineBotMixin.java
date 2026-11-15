package pathtracing.observer;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Pseudo;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import pathtracing.HopCapture;
@Pseudo
@Mixin(targets="pathtracing.Bot",remap=false)
abstract class HopBaselineBotMixin {
    @Inject(method="command",at=@At("HEAD"),remap=false)
    private static void ptCommand(String text,CallbackInfo ci) { HopCapture.mark("chat-mod"); }
}
