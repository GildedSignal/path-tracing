package pathtracing;

import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents;
import net.minecraft.client.Minecraft;
import net.minecraft.core.BlockPos;
import net.minecraft.network.chat.Component;
import net.minecraft.world.item.Items;
import net.minecraft.world.phys.BlockHitResult;
import net.minecraft.world.phys.HitResult;

/**
 * Your side. Holding a stick: right-click a block to send the bot there, sneak + right-click to stop it.
 * The goal travels as an ordinary whisper (/msg PathBot goto x y z), so the server stays vanilla and you
 * can type the same commands by hand.
 */
final class Pointer {
    private static final double REACH = 256;

    static void register() {
        NativeChannel.loadFailure(); // Load before the first click.
        ClientTickEvents.START_CLIENT_TICK.register(Pointer::tick);
    }

    private static void tick(Minecraft mc) {
        var player = mc.player;
        if (player == null || mc.gui.screen() != null || !player.getMainHandItem().is(Items.STICK)) return;
        // Consume the click before vanilla sees it, so the stick never interacts with the block.
        while (mc.options.keyUse.consumeClick()) {
            long[] issuance = NativeChannel.loadFailure() == null ? NativeChannel.clock() : null;
            if (issuance == null) { player.sendSystemMessage(Component.literal("Native channel refused: " + NativeChannel.loadFailure())); continue; }
            HopCapture.hop("issuance",0,issuance[0],issuance[0],new long[]{issuance[1]});
            mc.options.keyUse.setDown(false);
            if (Tracks.stickInHumanMode(mc, player.isShiftKeyDown())) continue;
            if (player.isShiftKeyDown()) {
                if (Tracks.selecting()) {
                    Tracks.click(mc, BlockPos.ZERO, null, true);
                    continue;
                }
                send(mc, "stop");
                continue;
            }
            HitResult hit = player.pick(REACH, 1f, false);
            if (hit.getType() != HitResult.Type.BLOCK) {
                player.sendSystemMessage(Component.literal("Nothing within " + (int) REACH + " blocks."));
                continue;
            }
            BlockHitResult block = (BlockHitResult) hit;
            if (Tracks.selecting()) {
                Tracks.click(mc, block.getBlockPos(), block.getDirection(), false);
                continue;
            }
            if (Tracks.startAtClickedA(mc, block.getBlockPos())) continue;
            // The clicked block and face directly (the planner uses Goal { cell, face, settle }).
            BlockPos pos = block.getBlockPos();
            send(mc, "goto " + pos.getX() + " " + pos.getY() + " " + pos.getZ() + " " + block.getDirection().getName() + " " + issuance[0] + " " + issuance[1]);
        }
    }

    private static void send(Minecraft mc, String command) {
        mc.getConnection().sendCommand("msg " + PathTracingMod.BOT + " " + command);
    }
}
