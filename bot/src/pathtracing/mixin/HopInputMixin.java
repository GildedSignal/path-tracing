package pathtracing.mixin;

import net.minecraft.server.network.ServerGamePacketListenerImpl;
import net.minecraft.network.protocol.game.*;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import pathtracing.HopCapture;

/** RETURN is after the network-thread handoff and actual game-thread handler. */
@Mixin(ServerGamePacketListenerImpl.class)
abstract class HopInputMixin {
    @Inject(method="handleMovePlayer",at=@At("RETURN")) private void ptMove(ServerboundMovePlayerPacket p,CallbackInfo ci) { HopCapture.applied(p); }
    @Inject(method="handlePlayerInput",at=@At("RETURN")) private void ptKeys(ServerboundPlayerInputPacket p,CallbackInfo ci) { HopCapture.applied(p); }
    @Inject(method="handlePlayerCommand",at=@At("RETURN")) private void ptCommand(ServerboundPlayerCommandPacket p,CallbackInfo ci) { HopCapture.applied(p); }
    @Inject(method="handlePlayerAction",at=@At("RETURN")) private void ptAction(ServerboundPlayerActionPacket p,CallbackInfo ci) { HopCapture.applied(p); }
    @Inject(method="handleUseItem",at=@At("RETURN")) private void ptUse(ServerboundUseItemPacket p,CallbackInfo ci) { HopCapture.applied(p); }
    @Inject(method="handleUseItemOn",at=@At("RETURN")) private void ptUseOn(ServerboundUseItemOnPacket p,CallbackInfo ci) { HopCapture.applied(p); }
    @Inject(method="handleSetCarriedItem",at=@At("RETURN")) private void ptSlot(ServerboundSetCarriedItemPacket p,CallbackInfo ci) { HopCapture.applied(p); }
}
