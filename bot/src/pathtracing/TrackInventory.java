package pathtracing;

import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import java.util.ArrayList;
import java.util.List;
import net.minecraft.nbt.CompoundTag;
import net.minecraft.nbt.TagParser;

/** Inventory-menu slots 5..45, including empty slots. SNBT retains every NBT numeric type. */
final class TrackInventory {
    static final int FIRST = 5;
    static final int SIZE = 41;
    private final List<String> slots;
    final int selected;

    TrackInventory(List<String> slots, int selected) {
        if (slots.size() != SIZE || selected < 0 || selected > 8) throw new IllegalArgumentException("Invalid inventory snapshot.");
        this.slots = slots.stream().map(s -> parse(s).toString()).toList();
        this.selected = selected;
    }

    static CompoundTag parse(String value) {
        try { return TagParser.parseCompoundFully(value); }
        catch (com.mojang.brigadier.exceptions.CommandSyntaxException e) { throw new IllegalArgumentException("Invalid item SNBT.", e); }
    }

    CompoundTag slot(int menuSlot) { return parse(slots.get(menuSlot - FIRST)); }

    TrackInventory withoutSticks() {
        return new TrackInventory(slots.stream().map(s -> parse(s).getStringOr("id", "").equals("minecraft:stick") ? "{}" : s).toList(), selected);
    }

    JsonObject json() {
        JsonObject out = new JsonObject();
        out.addProperty("version", 1);
        out.addProperty("selected", selected);
        JsonArray items = new JsonArray();
        slots.forEach(items::add);
        out.add("slots", items);
        return out;
    }

    static TrackInventory read(JsonObject in) {
        if (in.get("version").getAsInt() != 1) throw new IllegalArgumentException("Unknown inventory snapshot version.");
        List<String> slots = new ArrayList<>();
        in.getAsJsonArray("slots").forEach(s -> slots.add(s.getAsString()));
        return new TrackInventory(slots, in.get("selected").getAsBigDecimal().intValueExact());
    }

    @Override public boolean equals(Object other) {
        return other instanceof TrackInventory that && selected == that.selected && sameItems(that);
    }

    boolean sameItems(TrackInventory that) {
        for (int slot = FIRST; slot < FIRST + SIZE; slot++) if (!slot(slot).equals(that.slot(slot))) return false;
        return true;
    }

    @Override public int hashCode() { return json().hashCode(); }
}
