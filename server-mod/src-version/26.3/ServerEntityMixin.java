package pathtracing.server.mixin;

import net.minecraft.server.level.ServerEntity;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.entity.UpdateInterval;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Mutable;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import pathtracing.server.SwarmBot;

@Mixin(ServerEntity.class)
public class ServerEntityMixin {
    @Shadow @Final @Mutable private UpdateInterval updateInterval;

    @Inject(method = "<init>", at = @At("TAIL"))
    private void ptEveryTickForSwarmBot(ServerLevel level, Entity entity, UpdateInterval interval,
            boolean trackDelta, ServerEntity.Synchronizer synchronizer, CallbackInfo ci) {
        if (entity instanceof SwarmBot) updateInterval = UpdateInterval.periodic(1);
    }
}
