package pathtracing.probe.mixin;

import java.util.ArrayList;
import java.util.List;
import net.minecraft.server.level.ChunkMap;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.Redirect;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import pathtracing.probe.ServerProbe;

@Mixin(ChunkMap.class)
public class ChunkMapMixin {
    @Unique
    private final List<ServerPlayer> ptRealPlayers = new ArrayList<>();
    @Unique
    private ServerLevel ptCachedLevel;
    @Unique
    private long ptCachedTick = Long.MIN_VALUE;
    @Unique
    private int ptCachedSize = -1;

    @Inject(
        method = "updateChunkTracking(Lnet/minecraft/server/level/ServerPlayer;)V",
        at = @At("HEAD"),
        cancellable = true
    )
    private void ptSkipBotChunks(ServerPlayer player, CallbackInfo ci) {
        if (player instanceof ServerProbe.ProbeBot) {
            ci.cancel();
        }
    }

    @Redirect(
        method = {
            "tick()V",
            "addEntity(Lnet/minecraft/world/entity/Entity;)V"
        },
        at = @At(
            value = "INVOKE",
            target = "Lnet/minecraft/server/level/ServerLevel;players()Ljava/util/List;"
        )
    )
    private List<ServerPlayer> ptFilterRealPlayers(ServerLevel level) {
        List<ServerPlayer> all = level.players();
        if (all.isEmpty()) {
            return all;
        }
        long tick = level.getGameTime();
        int size = all.size();
        if (level != ptCachedLevel || tick != ptCachedTick || size != ptCachedSize) {
            ptRealPlayers.clear();
            for (int i = 0; i < size; i++) {
                ServerPlayer p = all.get(i);
                if (!(p instanceof ServerProbe.ProbeBot)) {
                    ptRealPlayers.add(p);
                }
            }
            ptCachedLevel = level;
            ptCachedTick = tick;
            ptCachedSize = size;
        }
        return ptRealPlayers;
    }

    @Redirect(
        method = "tick()V",
        at = @At(
            value = "INVOKE",
            target = "Ljava/util/List;add(Ljava/lang/Object;)Z"
        )
    )
    private boolean ptSkipMovedBot(List<Object> movedPlayers, Object element) {
        if (element instanceof ServerProbe.ProbeBot) {
            return false;
        }
        return movedPlayers.add(element);
    }
}
