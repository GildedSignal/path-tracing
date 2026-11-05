package pathtracing.mixin;

import net.minecraft.world.level.block.state.BlockBehaviour;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.gen.Invoker;

@Mixin(BlockBehaviour.class)
public interface BlockBehaviourAccess {
    @Invoker("getMaxHorizontalOffset")
    float pathtracing$maxHorizontalOffset();

    @Invoker("getMaxVerticalOffset")
    float pathtracing$maxVerticalOffset();
}
