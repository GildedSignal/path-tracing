package pathtracing.observer;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Pseudo;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import pathtracing.HopCapture;
@Pseudo
@Mixin(targets="pathtracing.TrackHumanClient",remap=false)
abstract class HopBaselineRestoreMixin {
    @Inject(method="say",at=@At("HEAD"),remap=false)
    private void ptRestored(String text,CallbackInfo ci) {
        if (text.equals("Track mode exited: saved inventory and game mode restored.") || text.equals("Track crash recovery complete: saved inventory and game mode restored.")) HopCapture.mark("restoration-closed");
    }
}
