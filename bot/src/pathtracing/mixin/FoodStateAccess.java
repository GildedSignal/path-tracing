package pathtracing.mixin;

import net.minecraft.world.food.FoodData;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.gen.Accessor;

@Mixin(FoodData.class)
public interface FoodStateAccess {
    @Accessor("exhaustionLevel") float pathtracing$exhaustion();
    @Accessor("tickTimer") int pathtracing$foodTickTimer();
}
