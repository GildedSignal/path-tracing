package pathtracing;

import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import net.minecraft.nbt.NbtAccounter;
import net.minecraft.nbt.NbtIo;

/** Project an overworld-type dimension onto the read-only snapshot reader's disk layout. */
final class TrackWorldView {
    static Path region(Path world, String dimension) {
        if (dimension.equals("minecraft:overworld")) return world.resolve("dimensions/minecraft/overworld/region");
        if (dimension.startsWith("pathtracing:")) {
            String name = dimension.substring("pathtracing:".length());
            try { TrackStore.validateName(name); }
            catch (IllegalArgumentException invalid) { throw new IllegalArgumentException("Unsupported track dimension: " + dimension, invalid); }
            return world.resolve("dimensions/pathtracing/" + name + "/region");
        }
        throw new IllegalArgumentException("Unsupported track dimension: " + dimension);
    }

    private static JsonObject definition() {
        return JsonParser.parseString("{\"type\":\"minecraft:overworld\",\"generator\":{\"type\":\"minecraft:flat\",\"settings\":{\"biome\":\"minecraft:the_void\",\"layers\":[],\"structure_overrides\":[]}}}").getAsJsonObject();
    }

    /** Only dimension declarations can be removed from the effective-law metadata. */
    private static void checkMapsPack(Path pack) throws IOException {
        boolean dimension = false;
        try (var paths = Files.walk(pack)) {
            for (Path file : paths.toList()) {
                if (Files.isSymbolicLink(file)) throw new IOException("Maps datapack contains a symbolic link: " + file);
                if (!Files.isRegularFile(file)) continue;
                String relative = pack.relativize(file).toString().replace('\\', '/');
                if (relative.equals("pack.mcmeta")) continue;
                if (!relative.matches("data/pathtracing/dimension/[a-z0-9_-]+\\.json")
                    || !JsonParser.parseString(Files.readString(file)).equals(definition())) {
                    throw new IOException("Maps datapack changes more than map dimensions: " + relative);
                }
                dimension = true;
            }
        }
        if (!dimension) throw new IOException("Maps datapack has no dimensions.");
    }

    static void create(Path world, Path trackFile, Path view) throws IOException {
        var track = JsonParser.parseString(Files.readString(trackFile)).getAsJsonObject();
        String dimension = track.get("dimension").getAsString();
        Path regions = region(world, dimension);
        Path pack = world.resolve("datapacks/pathtracing-maps");
        if (dimension.startsWith("pathtracing:")) {
            checkMapsPack(pack);
            if (!Files.isRegularFile(pack.resolve("data/pathtracing/dimension/" + dimension.substring(12) + ".json"))) {
                throw new IOException("Missing map definition: " + dimension);
            }
        }
        var level = NbtIo.readCompressed(world.resolve("level.dat"), NbtAccounter.unlimitedHeap());
        var enabled = level.getCompound("Data").orElseThrow().getCompound("DataPacks").orElseThrow().getList("Enabled").orElseThrow();
        for (int i = enabled.size() - 1; i >= 0; i--) {
            if (enabled.get(i).asString().orElseThrow().equals("file/pathtracing-maps")) {
                checkMapsPack(pack);
                enabled.remove(i);
            }
        }
        Files.createDirectory(view);
        var a = track.getAsJsonArray("A");
        var b = track.getAsJsonArray("B");
        String name = trackFile.getFileName().toString().replaceFirst("\\.json$", "");
        TrackStore.validateName(name);
        // The flight reader adds its existing horizontal pad and upper read halo.
        Files.writeString(view.resolve("bounds.txt"), name + " " + Math.min(a.get(0).getAsInt(), b.get(0).getAsInt())
            + " -64 " + Math.min(a.get(2).getAsInt(), b.get(2).getAsInt()) + " " + Math.max(a.get(0).getAsInt(), b.get(0).getAsInt())
            + " " + Math.max(a.get(1).getAsInt(), b.get(1).getAsInt()) + " " + Math.max(a.get(2).getAsInt(), b.get(2).getAsInt()));
        NbtIo.writeCompressed(level, view.resolve("level.dat"));
        Path data = view.resolve("data/minecraft");
        Files.createDirectories(data);
        for (String file : new String[] {"world_gen_settings.dat", "game_rules.dat"}) {
            Files.createLink(data.resolve(file), world.resolve("data/minecraft/" + file));
        }
        Path target = view.resolve("dimensions/minecraft/overworld/region");
        Files.createDirectories(target);
        try (var files = Files.list(regions)) {
            for (Path file : files.filter(p -> p.getFileName().toString().endsWith(".mca")).toList()) {
                Files.createLink(target.resolve(file.getFileName()), file);
            }
        }
    }

    public static void main(String[] args) throws IOException {
        create(Path.of(args[0]), Path.of(args[1]), Path.of(args[2]));
    }
}
