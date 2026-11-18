package pathtracing.probe.mixin;

import java.util.function.BooleanSupplier;
import net.minecraft.server.MinecraftServer;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import pathtracing.probe.ServerProbe;

@Mixin(MinecraftServer.class)
public class TickProbeMixin {
    @Inject(method = "tickServer(Ljava/util/function/BooleanSupplier;)V", at = @At("HEAD"))
    private void ptTickStart(BooleanSupplier hasTime, CallbackInfo ci) {
        ServerProbe.tickStart();
    }

    @Inject(method = "tickServer(Ljava/util/function/BooleanSupplier;)V", at = @At("RETURN"))
    private void ptTickEnd(BooleanSupplier hasTime, CallbackInfo ci) {
        ServerProbe.tickEnd();
    }
}
