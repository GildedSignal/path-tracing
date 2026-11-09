package pathtracing.phase.mixin;

import net.minecraft.core.BlockPos;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.level.Level;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.state.BlockState;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import pathtracing.phase.LandingTrace;

@Mixin(Block.class)
abstract class LandingFloor {
    @Inject(method = "fallOn", at = @At("HEAD"))
    private void ptFloor(Level level, BlockState state, BlockPos pos, Entity entity, double distance, CallbackInfo ci) {
        if (LandingTrace.ENABLED) LandingTrace.floor(entity, BuiltInRegistries.BLOCK.getKey(state.getBlock()).toString());
    }

    @Inject(method = "fallOn", at = @At("RETURN"))
    private void ptAfterFloor(Level level, BlockState state, BlockPos pos, Entity entity, double distance, CallbackInfo ci) {
        if (LandingTrace.ENABLED) LandingTrace.floor(entity, null);
    }
}
