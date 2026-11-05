package pathtracing.mixin;

import net.minecraft.world.entity.Entity;
import net.minecraft.world.phys.Vec3;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.gen.Accessor;

@Mixin(Entity.class)
public interface EntityStateAccess {
    @Accessor("stuckSpeedMultiplier") Vec3 pathtracing$stuckSpeedMultiplier();
    @Accessor("wasEyeInWater") boolean pathtracing$wasEyeInWater();
    @Accessor("onGroundNoBlocks") boolean pathtracing$onGroundNoBlocks();
}
