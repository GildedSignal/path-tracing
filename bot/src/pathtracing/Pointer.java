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
        ClientTickEvents.START_CLIENT_TICK.register(Pointer::tick);
    }

    private static void tick(Minecraft mc) {
        var player = mc.player;
        if (player == null || mc.gui.screen() != null || !player.getMainHandItem().is(Items.STICK)) return;
        // Consume the click before vanilla sees it, so the stick never interacts with the block.
        while (mc.options.keyUse.consumeClick()) {
            if (player.isShiftKeyDown()) {
                send(mc, "stop");
                continue;
            }
            HitResult hit = player.pick(REACH, 1f, false);
            if (hit.getType() != HitResult.Type.BLOCK) {
                player.sendSystemMessage(Component.literal("Nothing within " + (int) REACH + " blocks."));
                continue;
            }
            BlockHitResult block = (BlockHitResult) hit;
            // The cell in front of the face you pointed at: on a floor, the air cell the bot should stand in.
            BlockPos goal = block.getBlockPos().relative(block.getDirection());
            send(mc, "goto " + goal.getX() + " " + goal.getY() + " " + goal.getZ());
        }
    }

    private static void send(Minecraft mc, String command) {
        mc.getConnection().sendCommand("msg " + PathTracingMod.BOT + " " + command);
    }
}
