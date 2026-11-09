package pathtracing;

import io.netty.buffer.Unpooled;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import net.minecraft.core.HolderLookup;
import net.minecraft.core.RegistryAccess;
import net.minecraft.nbt.CompoundTag;
import net.minecraft.nbt.NbtOps;
import net.minecraft.network.RegistryFriendlyByteBuf;
import net.minecraft.network.protocol.game.ServerboundSetCreativeModeSlotPacket;
import net.minecraft.world.ItemStackWithSlot;
import net.minecraft.world.entity.EntityEquipment;
import net.minecraft.world.entity.EquipmentSlot;
import net.minecraft.world.flag.FeatureFlagSet;
import net.minecraft.world.item.ItemStack;

/** The same lossless registry-aware item codec for files, packets and server verification. */
final class TrackInventoryCodec {
    static TrackInventory encode(List<ItemStack> slots, int selected, HolderLookup.Provider registries) {
        var ops = registries.createSerializationContext(NbtOps.INSTANCE);
        return new TrackInventory(slots.stream().map(s -> ItemStack.OPTIONAL_CODEC.encodeStart(ops, s).getOrThrow().toString()).toList(), selected);
    }

    static List<ItemStack> decode(TrackInventory inventory, HolderLookup.Provider registries) {
        var ops = registries.createSerializationContext(NbtOps.INSTANCE);
        List<ItemStack> slots = new ArrayList<>();
        for (int slot = TrackInventory.FIRST; slot < TrackInventory.FIRST + TrackInventory.SIZE; slot++) {
            slots.add(ItemStack.OPTIONAL_CODEC.parse(ops, inventory.slot(slot)).getOrThrow());
        }
        return slots;
    }

    static List<ServerboundSetCreativeModeSlotPacket> creativePackets(TrackInventory inventory, RegistryAccess registries, FeatureFlagSet features) {
        var ops = registries.createSerializationContext(NbtOps.INSTANCE);
        var packets = new ArrayList<ServerboundSetCreativeModeSlotPacket>(TrackInventory.SIZE);
        var codec = ServerboundSetCreativeModeSlotPacket.STREAM_CODEC;
        var buffer = new RegistryFriendlyByteBuf(Unpooled.buffer(), registries);
        try {
            for (int slot = TrackInventory.FIRST; slot < TrackInventory.FIRST + TrackInventory.SIZE; slot++) {
                var saved = inventory.slot(slot);
                try {
                    var stack = ItemStack.OPTIONAL_CODEC.parse(ops, saved).getOrThrow();
                    if (!stack.isItemEnabled(features)) throw new IllegalArgumentException("item is disabled by server feature flags");
                    if (!stack.isEmpty() && stack.getCount() > stack.getMaxStackSize()) {
                        throw new IllegalArgumentException("stack exceeds its maximum count " + stack.getMaxStackSize());
                    }
                    var packet = new ServerboundSetCreativeModeSlotPacket(slot, stack);
                    buffer.clear();
                    codec.encode(buffer, packet);
                    if (!ItemStack.matches(stack, codec.decode(buffer).itemStack()) || buffer.isReadable()) {
                        throw new IllegalArgumentException("creative packet changes item data");
                    }
                    packets.add(packet);
                } catch (RuntimeException failure) {
                    throw new IllegalArgumentException("Cannot restore slot " + slot + " (" + saved.getStringOr("id", "empty") + "): "
                        + failure.getMessage(), failure);
                }
            }
        } finally {
            buffer.release();
        }
        return packets;
    }

    static TrackInventory server(CompoundTag tag, HolderLookup.Provider registries) {
        var ops = registries.createSerializationContext(NbtOps.INSTANCE);
        List<ItemStack> slots = new ArrayList<>(Collections.nCopies(TrackInventory.SIZE, ItemStack.EMPTY));
        for (var entry : tag.getList("Inventory").orElseThrow(() -> new IllegalArgumentException("Server inventory missing."))) {
            var item = ItemStackWithSlot.CODEC.parse(ops, entry).getOrThrow();
            int index = item.slot();
            if (index < 0 || index >= 36) throw new IllegalArgumentException("Unexpected server inventory slot " + index);
            int menuSlot = index < 9 ? 36 + index : index;
            slots.set(menuSlot - TrackInventory.FIRST, item.stack());
        }
        var equipment = EntityEquipment.CODEC.parse(ops, tag.getCompoundOrEmpty("equipment")).getOrThrow();
        var armour = List.of(EquipmentSlot.HEAD, EquipmentSlot.CHEST, EquipmentSlot.LEGS, EquipmentSlot.FEET);
        for (int i = 0; i < armour.size(); i++) slots.set(i, equipment.get(armour.get(i)));
        slots.set(40, equipment.get(EquipmentSlot.OFFHAND));
        return encode(slots, tag.getInt("SelectedItemSlot").orElseThrow(), registries);
    }
}
