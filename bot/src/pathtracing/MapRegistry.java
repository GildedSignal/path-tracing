package pathtracing;

import com.google.gson.JsonParser;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Map;
import java.util.TreeMap;

/** The maps.json shape shared by the client command and its offline import gate. */
final class MapRegistry {
    static Map<String, int[]> read(Path file) throws IOException {
        Map<String, int[]> maps = new TreeMap<>();
        if (!Files.exists(file)) return maps;
        var json = JsonParser.parseString(Files.readString(file));
        if (!json.isJsonArray()) throw new IOException("maps.json must contain an array of map objects.");
        for (var entry : json.getAsJsonArray()) {
            if (!entry.isJsonObject() || !entry.getAsJsonObject().has("name") || !entry.getAsJsonObject().has("spawn")) {
                throw new IOException("Each maps.json entry must be a map object with a name and spawn.");
            }
            var map = entry.getAsJsonObject();
            String name = map.get("name").getAsString();
            TrackStore.validateName(name);
            var spawn = map.getAsJsonArray("spawn");
            if (spawn.size() != 3) throw new IOException("Invalid spawn for map " + name + ".");
            int[] pos = new int[3];
            for (int i = 0; i < pos.length; i++) pos[i] = spawn.get(i).getAsBigDecimal().intValueExact();
            if (maps.putIfAbsent(name, pos) != null) throw new IOException("Duplicate map in maps.json: " + name);
        }
        return maps;
    }
}
