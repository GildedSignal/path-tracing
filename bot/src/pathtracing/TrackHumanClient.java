package pathtracing;

import java.util.ArrayList;
import java.util.function.Consumer;
import net.minecraft.client.Minecraft;
import net.minecraft.network.protocol.game.ServerboundSetCarriedItemPacket;
import net.minecraft.network.protocol.game.ServerboundSetCreativeModeSlotPacket;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.level.GameType;

/** Own-player client commands and vanilla packets only; never RCON. */
final class TrackHumanClient implements TrackMode.Port {
    private final Minecraft mc;
    private boolean connected = true;

    TrackHumanClient(Minecraft mc) { this.mc = mc; }
    void disconnect() { connected = false; }

    @Override public TrackInventory inventory() {
        var slots = new ArrayList<ItemStack>();
        for (int slot = TrackInventory.FIRST; slot < TrackInventory.FIRST + TrackInventory.SIZE; slot++) {
            slots.add(mc.player.inventoryMenu.getSlot(slot).getItem().copy());
        }
        return TrackInventoryCodec.encode(slots, mc.player.getInventory().getSelectedSlot(), mc.level.registryAccess());
    }

    @Override public String mode() { return mc.gameMode.getPlayerMode().getName(); }
    @Override public boolean alive() { return mc.player.isAlive(); }
    @Override public void command(String command) { mc.player.connection.sendCommand(command); }
    @Override public void say(String text) { Bot.reply(mc, text); }

    @Override public void apply(TrackInventory inventory) {
        if (!mode().equals("creative")) throw new IllegalStateException("Slot packets require creative mode.");
        mc.player.closeContainer();
        var slots = TrackInventoryCodec.decode(inventory, mc.level.registryAccess());
        for (int i = 0; i < slots.size(); i++) {
            mc.player.connection.send(new ServerboundSetCreativeModeSlotPacket(i + TrackInventory.FIRST, slots.get(i)));
        }
        mc.player.getInventory().setSelectedSlot(inventory.selected);
        mc.player.connection.send(new ServerboundSetCarriedItemPacket(inventory.selected));
    }

    @Override public void confirm(TrackInventory inventory) {
        var slots = TrackInventoryCodec.decode(inventory, mc.level.registryAccess());
        for (int i = 0; i < slots.size(); i++) mc.player.inventoryMenu.getSlot(i + TrackInventory.FIRST).set(slots.get(i));
        mc.player.getInventory().setSelectedSlot(inventory.selected);
    }

    @Override public void query(Consumer<TrackMode.Observation> response, Consumer<String> failure) {
        var connection = mc.player.connection;
        mc.player.connection.getDebugQueryHandler().queryEntityTag(mc.player.getId(), tag -> {
            if (!connected || mc.player == null || mc.level == null || mc.player.connection != connection) return;
            try {
                if (tag == null) throw new IllegalArgumentException("Server refused player data query.");
                String gameMode = GameType.byId(tag.getInt("playerGameType").orElseThrow()).getName();
                boolean full = tag.getFloat("Health").orElseThrow() == mc.player.getMaxHealth()
                    && tag.getInt("foodLevel").orElseThrow() == 20 && tag.getFloat("foodSaturationLevel").orElseThrow() == 20.0f;
                response.accept(new TrackMode.Observation(TrackInventoryCodec.server(tag, mc.level.registryAccess()), gameMode, full));
            } catch (RuntimeException error) { failure.accept("Server verification: " + error.getMessage()); }
        });
    }
}
