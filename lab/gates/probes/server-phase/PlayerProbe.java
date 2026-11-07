package pathtracing.phase.mixin;

import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.damagesource.DamageSource;
import net.minecraft.world.entity.player.Player;
import org.spongepowered.asm.mixin.Mixin;
import pathtracing.phase.ResourceTrace;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(Player.class)
public abstract class PlayerProbe {
    // At this call, the damage argument's local slot holds the post-armor,
    // post-magic and post-absorption amount used by the health subtraction.
    @Inject(method = "actuallyHurt", at = @At(value = "INVOKE",
        target = "Lnet/minecraft/world/entity/player/Player;setHealth(F)V"))
    private void ptDamage(ServerLevel level, DamageSource source, float amount, CallbackInfo ci) {
        ResourceTrace.damage(this, amount);
    }
}
