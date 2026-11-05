package pathtracing;

import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import net.minecraft.core.HolderLookup;
import net.minecraft.nbt.CompoundTag;
import net.minecraft.nbt.NbtOps;
import net.minecraft.world.ItemStackWithSlot;
import net.minecraft.world.entity.EntityEquipment;
import net.minecraft.world.entity.EquipmentSlot;
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
