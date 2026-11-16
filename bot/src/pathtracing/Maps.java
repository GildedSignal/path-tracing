package pathtracing;

import com.google.gson.JsonParser;
import com.mojang.brigadier.arguments.StringArgumentType;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Map;
import java.util.TreeMap;
import net.fabricmc.fabric.api.client.command.v2.ClientCommandRegistrationCallback;
import net.fabricmc.fabric.api.client.command.v2.ClientCommands;
import net.minecraft.client.Minecraft;
import net.minecraft.network.chat.ClickEvent;
import net.minecraft.network.chat.Component;

final class Maps {
    static void register() {
        ClientCommandRegistrationCallback.EVENT.register((dispatcher, access) -> dispatcher.register(
            ClientCommands.literal("map").executes(c -> run(null)).then(
                ClientCommands.argument("name", StringArgumentType.word()).executes(c -> run(StringArgumentType.getString(c, "name"))))));
    }

    private static int run(String name) {
        var mc = Minecraft.getInstance();
        try {
            Path rig = Path.of(System.getProperty("pathtracing.rig", "."));
            Map<String, int[]> maps = new TreeMap<>();
            maps.put("overworld", MapSpawn.read(rig.resolve("server/world/level.dat")));
            Path file = rig.resolve("maps.json");
            if (Files.exists(file)) for (var entry : JsonParser.parseString(Files.readString(file)).getAsJsonArray()) {
                var map = entry.getAsJsonObject();
                String key = map.get("name").getAsString();
                TrackStore.validateName(key);
                var spawn = map.getAsJsonArray("spawn");
                maps.put(key, new int[] {spawn.get(0).getAsInt(), spawn.get(1).getAsInt(), spawn.get(2).getAsInt()});
            }
            if (name == null) {
                maps.keySet().forEach(key -> mc.player.sendSystemMessage(Component.literal(key).withStyle(
                    style -> style.withUnderlined(true).withClickEvent(new ClickEvent.RunCommand("/map " + key)))));
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
}
