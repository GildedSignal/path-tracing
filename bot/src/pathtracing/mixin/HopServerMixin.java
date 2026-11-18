package pathtracing.mixin;

import java.util.function.BooleanSupplier;
import net.minecraft.server.MinecraftServer;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import pathtracing.HopCapture;

@Mixin(MinecraftServer.class)
abstract class HopServerMixin {
    @Inject(method="tickServer",at=@At("HEAD"))
    private void ptTick(BooleanSupplier hasTime,CallbackInfo ci) { HopCapture.serverTick(((MinecraftServer)(Object)this).getTickCount()); }
    @Inject(method="tickServer",at=@At("RETURN"))
    private void ptResources(BooleanSupplier hasTime,CallbackInfo ci) { HopCapture.resources((MinecraftServer)(Object)this); }
}
