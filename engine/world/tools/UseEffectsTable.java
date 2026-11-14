import java.nio.file.*;
import java.util.*;
import net.minecraft.core.component.DataComponents;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.world.item.component.UseEffects;

// Development-time extraction, parallel to ModelTable. No JVM needed by the reader.
public class UseEffectsTable {
    public static void main(String[] args) throws Exception {
        net.minecraft.SharedConstants.tryDetectVersion();
        net.minecraft.server.Bootstrap.bootStrap();
        var provider = net.minecraft.data.registries.VanillaRegistries.createLookup();
        var rows = new TreeMap<String, String>();
        for (var pending : BuiltInRegistries.DATA_COMPONENT_INITIALIZERS.build(provider)) {
            pending.apply();
            if (!pending.key().equals(BuiltInRegistries.ITEM.key())) continue;
            pending.forEach((ref, map) -> {
                UseEffects fx = map.get(DataComponents.USE_EFFECTS);
                // Every 26.2 item binds USE_EFFECTS explicitly (1,537 of 1,537); a future jar that does not must stop the export, not default (1.4b judge).
                if (fx == null) throw new IllegalStateException("no USE_EFFECTS binding: " + ref.key().identifier());
                rows.put(ref.key().identifier().toString(), fx.canSprint() + "\t" + String.format("%08x", Float.floatToRawIntBits(fx.speedMultiplier())));
            });
        }
        try (var out = Files.newBufferedWriter(Path.of(args[0]))) {
            for (var row : rows.entrySet()) { out.write(row.getKey() + "\t" + row.getValue() + "\n"); }
        }
    }
}
