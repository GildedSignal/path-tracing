package pathtracing;

import com.google.gson.JsonParser;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Map;
import java.util.TreeMap;
import net.fabricmc.fabric.api.client.command.v2.ClientCommandRegistrationCallback;
import net.minecraft.client.Minecraft;
import net.minecraft.network.chat.ClickEvent;
import net.minecraft.network.chat.Component;

final class Maps {
    static void register() {
        ClientCommandRegistrationCallback.EVENT.register((dispatcher, access) -> RaceCommands.maps(dispatcher,
            Path.of(System.getProperty("pathtracing.rig", "."), "maps.json"), Maps::run));
    }

    /** Maps served as their own world (scripts/map.ps1), in every version's rig: name -> version. */
    static Map<String, String> worlds() {
        Map<String, String> out = new TreeMap<>();
        String scripts = System.getProperty("pathtracing.scripts");
        if (scripts == null) return out;
        try {
            Path root = Path.of(scripts).getParent();
            var versions = JsonParser.parseString(Files.readString(Path.of(scripts, "versions.json"))).getAsJsonObject();
            for (var v : versions.entrySet()) {
                var row = v.getValue().getAsJsonObject();
                Path rig = row.has("rig") ? Path.of(row.get("rig").getAsString()) : root;
                Path maps = rig.resolve("server/maps");
                if (!Files.isDirectory(maps)) continue;
                try (var dirs = Files.list(maps)) {
                    dirs.filter(d -> Files.exists(d.resolve("level.dat"))).forEach(d -> out.put(d.getFileName().toString(), v.getKey()));
                }
            }
        } catch (IOException | RuntimeException ignored) {
        }
        return out;
    }

    private static int run(String name) {
        var mc = Minecraft.getInstance();
        try {
            Path rig = Path.of(System.getProperty("pathtracing.rig", "."));
            Map<String, int[]> maps = new TreeMap<>(MapRegistry.read(rig.resolve("maps.json")));
            maps.put("overworld", MapSpawn.read(rig.resolve("server/world/level.dat")));
            Map<String, String> worlds = worlds();
            // On a map served as its own world, the overworld and the old dimension maps are in the
            // rig's own world: switch the server back to it first.
            var properties = new java.util.Properties();
            try (var reader = Files.newBufferedReader(rig.resolve("server/server.properties"))) { properties.load(reader); }
            boolean onMapWorld = !properties.getProperty("level-name", "world").equals("world");
            if (name != null && onMapWorld && maps.containsKey(name)) {
                new ProcessBuilder("powershell", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File",
                    Path.of(System.getProperty("pathtracing.scripts"), "map.ps1").toString(), "world", "-McVersion",
                    System.getProperty("pathtracing.mcVersion", "26.2"))
                    .redirectErrorStream(true).redirectOutput(rig.resolve("map-switch.log").toFile()).start();
                mc.player.sendSystemMessage(Component.literal("Switching the server back to your world: it restarts; join again in about 30 s"
                    + (name.equals("overworld") ? "." : ", then /map " + name + ".")));
                return 1;
            }
            if (name == null) {
                maps.keySet().forEach(key -> line(mc, key, key));
                worlds.forEach((key, version) -> line(mc, key, key + "  (own world, " + version + ")"));
            } else if (worlds.containsKey(name) && !maps.containsKey(name)) {
                String here = System.getProperty("pathtracing.mcVersion", "26.2");
                String there = worlds.get(name);
                new ProcessBuilder("powershell", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File",
                    Path.of(System.getProperty("pathtracing.scripts"), "map.ps1").toString(), name, "-Player", "-From", here)
                    .redirectErrorStream(true).redirectOutput(rig.resolve("map-switch.log").toFile()).start();
                mc.player.sendSystemMessage(Component.literal(there.equals(here)
                    ? "Switching the server to " + name + ": it restarts; join again in about 30 s."
                    : name + " is a " + there + " map: its " + there + " server and a " + there + " client are starting."));
            } else {
                int[] p = maps.get(name);
                if (p == null) throw new IllegalArgumentException("Unknown map: " + name);
                String dim = name.equals("overworld") ? "minecraft:overworld" : "pathtracing:" + name;
                String xyz = p[0] + " " + p[1] + " " + p[2];
                mc.player.connection.sendCommand("execute in " + dim + " run tp @s " + xyz);
                mc.player.connection.sendCommand("execute in " + dim + " run tp " + PathTracingMod.BOT + " " + xyz);
            }
            return 1;
        } catch (Exception failure) { mc.player.sendSystemMessage(Component.literal("Map: " + failure.getMessage())); return 0; }
    }

    private static void line(Minecraft mc, String key, String label) {
        mc.player.sendSystemMessage(Component.literal(label).withStyle(
            style -> style.withUnderlined(true).withClickEvent(new ClickEvent.RunCommand("/map " + key))));
    }
}
