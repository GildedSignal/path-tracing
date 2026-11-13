package pathtracing.mixin;

import net.minecraft.world.entity.Entity;
import net.minecraft.world.phys.Vec3;
import net.minecraft.network.syncher.EntityDataAccessor;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.gen.Accessor;

@Mixin(Entity.class)
public interface EntityStateAccess {
    @Accessor("DATA_TICKS_FROZEN") static EntityDataAccessor<Integer> pathtracing$ticksFrozen() { throw new AssertionError(); }
    @Accessor("stuckSpeedMultiplier") Vec3 pathtracing$stuckSpeedMultiplier();
    @Accessor("wasEyeInWater") boolean pathtracing$wasEyeInWater();
    @Accessor("onGroundNoBlocks") boolean pathtracing$onGroundNoBlocks();
}
