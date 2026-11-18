package pathtracing.server.mixin;

import net.minecraft.server.level.ServerPlayer;
import net.minecraft.server.players.PlayerList;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import pathtracing.server.SwarmBot;

@Mixin(PlayerList.class)
public class PlayerListMixin {
    @Inject(
        method = "save(Lnet/minecraft/server/level/ServerPlayer;)V",
        at = @At("HEAD"),
        cancellable = true
    )
    private void ptSkipBotSave(ServerPlayer player, CallbackInfo ci) {
        if (player instanceof SwarmBot) {
            ci.cancel();
        }
    }
}
