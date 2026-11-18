package pathtracing.mixin;
import net.minecraft.server.dedicated.DedicatedServer;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;
import pathtracing.HopCapture;
/** Observe the legacy baseline's RCON hop on both sides of the game-thread handoff. */
@Mixin(DedicatedServer.class)
abstract class HopRconMixin {
    @Inject(method="runCommand",at=@At("HEAD")) private void ptReceive(String command,CallbackInfoReturnable<String> ci) { HopCapture.mark("legacy-rcon-receive"); }
    @Inject(method="lambda$runCommand$0",at=@At("RETURN")) private void ptApply(String command,CallbackInfo ci) { HopCapture.mark("legacy-rcon-applied"); }
}
