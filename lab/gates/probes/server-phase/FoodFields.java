package pathtracing.phase.mixin;

import net.minecraft.world.food.FoodData;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.gen.Accessor;

@Mixin(FoodData.class)
public interface FoodFields {
    @Accessor("exhaustionLevel") float ptExhaustion();
    @Accessor("tickTimer") int ptFoodTimer();
}
