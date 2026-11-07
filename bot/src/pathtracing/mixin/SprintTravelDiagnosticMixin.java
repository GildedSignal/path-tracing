package pathtracing.mixin;

import net.minecraft.world.entity.Entity;
import net.minecraft.world.entity.player.Player;
import net.minecraft.world.phys.Vec3;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import pathtracing.SprintDiagnostic;

@Mixin(Player.class)
abstract class SprintTravelDiagnosticMixin {
    @Inject(method = "travel", at = @At("HEAD"))
    private void ptTravel(Vec3 input, CallbackInfo ci) {
        SprintDiagnostic.observe("travel", (Entity)(Object)this);
    }
}
