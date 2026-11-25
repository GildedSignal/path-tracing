package pathtracing.server.mixin;

import net.minecraft.network.chat.Component;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.server.players.PlayerList;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import pathtracing.server.SwarmBot;
import pathtracing.server.SwarmManager;

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

    @Inject(
        method = "broadcastSystemMessage(Lnet/minecraft/network/chat/Component;Z)V",
        at = @At("HEAD"),
        cancellable = true
    )
    private void ptSuppressBotJoinLeaveChat(Component message, boolean overlay, CallbackInfo ci) {
        if (SwarmManager.isSuppressingJoinChat()) {
            ci.cancel();
        }
    }
}
