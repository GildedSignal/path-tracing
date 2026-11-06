package pathtracing.mixin;

import net.minecraft.world.entity.Entity;
import net.minecraft.world.phys.Vec3;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;
import pathtracing.Recorder;

@Mixin(Entity.class)
public abstract class EntityCollisionMixin {
    @Inject(method = "collide(Lnet/minecraft/world/phys/Vec3;)Lnet/minecraft/world/phys/Vec3;", at = @At("RETURN"))
    private void pathtracing$collision(Vec3 requested, CallbackInfoReturnable<Vec3> callback) {
        Recorder.collisionStep((Entity) (Object) this, requested, callback.getReturnValue());
    }
}
