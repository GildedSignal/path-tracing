package pathtracing;

import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import java.io.BufferedReader;
import java.io.BufferedWriter;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents;
import net.fabricmc.fabric.api.client.networking.v1.ClientPlayConnectionEvents;
import net.minecraft.client.Minecraft;
import net.minecraft.network.protocol.game.ClientboundPlayerPositionPacket;
import net.minecraft.network.protocol.game.ClientboundSetEntityMotionPacket;

/** START applies controls, END records their result. Packets between ticks belong to the next END. */
public final class Recorder {
    private static BufferedWriter output;
    private static String name;
    private static int tick;
    private static TickInput applied;
    private static List<TickInput> replay;
    private static double[] start;
    private static int settled;
    private static boolean teleported;
    private static JsonArray anomalies = new JsonArray();

    static void register() {
        ClientTickEvents.START_CLIENT_TICK.register(Recorder::startTick);
        ClientTickEvents.END_CLIENT_TICK.register(Recorder::endTick);
        ClientPlayConnectionEvents.DISCONNECT.register((handler, mc) -> stop(mc));
    }

    static boolean replaying() {
        return replay != null;
    }

    static Path file(Minecraft mc, String directory, String fileName) {
        if (!fileName.matches("[A-Za-z0-9_-]+")) throw new IllegalArgumentException("Names use letters, digits, _ and -.");
        return mc.gameDirectory.toPath().resolve("pathtracing").resolve(directory).resolve(fileName + ".jsonl");
    }

    static void record(Minecraft mc, String runName) throws IOException {
        if (output != null || replaying()) throw new IllegalStateException("A recording or replay is already running.");
        open(mc, runName);
        Bot.reply(mc, "Recording " + name + ".");
    }

    private static void open(Minecraft mc, String runName) throws IOException {
        Path path = file(mc, "recordings", runName);
        Files.createDirectories(path.getParent());
        output = Files.newBufferedWriter(path, StandardCharsets.UTF_8);
        name = runName;
        tick = 0;
        applied = null;
        anomalies = new JsonArray();
    }

    static void replay(Minecraft mc, String runName) throws IOException {
        if (output != null || replaying()) throw new IllegalStateException("A recording or replay is already running.");
        List<TickInput> inputs = new ArrayList<>();
        double[] origin = new double[5];
        try (BufferedReader reader = Files.newBufferedReader(file(mc, "inputs", runName), StandardCharsets.UTF_8)) {
            String first = reader.readLine();
            if (first == null) throw new IllegalArgumentException("Missing start line.");
            JsonArray values = JsonParser.parseString(first).getAsJsonObject().getAsJsonArray("start");
            if (values == null || values.size() != 5) throw new IllegalArgumentException("Expected start: [x,y,z,yaw,pitch].");
            for (int i = 0; i < 5; i++) {
                if (!values.get(i).isJsonPrimitive() || !values.get(i).getAsJsonPrimitive().isNumber()) {
                    throw new IllegalArgumentException("Start values must be numbers.");
                }
                origin[i] = values.get(i).getAsDouble();
                if (!Double.isFinite(origin[i]) || (i >= 3 && !Float.isFinite((float) origin[i]))) {
                    throw new IllegalArgumentException("Start values must be finite.");
                }
            }
            String line;
            while ((line = reader.readLine()) != null) {
                JsonObject in = JsonParser.parseString(line).getAsJsonObject();
                if (!in.has("t") || !in.get("t").isJsonPrimitive() || !in.getAsJsonPrimitive("t").isNumber()
                    || in.get("t").getAsBigDecimal().compareTo(java.math.BigDecimal.valueOf(inputs.size())) != 0) {
                    throw new IllegalArgumentException("Input ticks must be consecutive, starting at 0.");
                }
                inputs.add(TickInput.read(in));
            }
        }
        if (inputs.isEmpty()) throw new IllegalArgumentException("Replay has no input ticks.");
        open(mc, runName);
        replay = inputs;
        start = origin;
        settled = 0;
        teleported = false;
        TickInput.release(mc);
        mc.getConnection().sendCommand("tp @s " + origin[0] + " " + origin[1] + " " + origin[2] + " " + origin[3] + " " + origin[4]);
        Bot.reply(mc, "Replay " + name + ": waiting for the teleport and 5 settled ticks.");
    }

    static void stop(Minecraft mc) {
        boolean wasReplay = replaying();
        replay = null;
        start = null;
        applied = null;
        anomalies = new JsonArray();
        if (wasReplay) TickInput.release(mc);
        if (output == null) return;
        BufferedWriter writer = output;
        output = null;
        try {
            writer.close();
            Bot.reply(mc, "Saved " + name + ": " + tick + " ticks.");
        } catch (IOException e) {
            Bot.reply(mc, "Recording close failed: " + e.getMessage());
        }
    }

    private static void startTick(Minecraft mc) {
        applied = null;
        if (output == null) return;
        if (mc.player == null || mc.level == null) {
            stop(mc);
            return;
        }
        if (replaying()) {
            if (start != null) {
                TickInput.release(mc);
                return;
            }
            replay.get(tick).apply(mc);
        }
        applied = TickInput.capture(mc);
    }

    private static void endTick(Minecraft mc) {
        if (output == null || mc.player == null) return;
        var player = mc.player;
        if (start != null) {
            boolean atStart = same(player.getX(), start[0]) && same(player.getY(), start[1]) && same(player.getZ(), start[2]);
            var velocity = player.getDeltaMovement();
            settled = teleported && atStart && player.onGround() && velocity.x == 0 && velocity.z == 0 ? settled + 1 : 0;
            if (settled == 5) {
                start = null;
                anomalies = new JsonArray();
            }
            return;
        }
        if (applied == null) return;
        JsonObject out = new JsonObject();
        out.addProperty("t", tick);
        out.add("inputs", applied.json());
        out.add("position", TraceJson.vector(player.position()));
        out.add("velocity", TraceJson.vector(player.getDeltaMovement()));
        TraceJson.number(out, "yRot", player.getYRot());
        TraceJson.number(out, "xRot", player.getXRot());
        out.addProperty("onGround", player.onGround());
        out.addProperty("horizontalCollision", player.horizontalCollision);
        out.addProperty("verticalCollision", player.verticalCollision);
        out.addProperty("isSprinting", player.isSprinting());
        out.addProperty("isShiftKeyDown", player.isShiftKeyDown());
        out.addProperty("pose", player.getPose().name());
        TraceJson.number(out, "fallDistance", player.fallDistance);
        out.addProperty("isInWater", player.isInWater());
        out.addProperty("onClimbable", player.onClimbable());
        TraceJson.number(out, "health", player.getHealth());
        out.addProperty("foodLevel", player.getFoodData().getFoodLevel());
        TraceJson.number(out, "saturationLevel", player.getFoodData().getSaturationLevel());
        if (!anomalies.isEmpty()) out.add("anomaly", anomalies);
        try {
            output.write(out.toString());
            output.write('\n');
            output.flush();
            tick++;
            anomalies = new JsonArray();
            if (replaying() && tick == replay.size()) stop(mc);
        } catch (IOException e) {
            Bot.reply(mc, "Recording write failed: " + e.getMessage());
            stop(mc);
        }
    }

    private static boolean same(double a, double b) {
        return Double.doubleToRawLongBits(a) == Double.doubleToRawLongBits(b);
    }

    public static void positionPacket(ClientboundPlayerPositionPacket packet) {
        if (output == null) return;
        if (start != null) {
            teleported = true;
            settled = 0;
            return;
        }
        JsonObject out = new JsonObject();
        out.addProperty("type", "ClientboundPlayerPositionPacket");
        out.addProperty("id", packet.id());
        out.add("position", TraceJson.vector(packet.change().position()));
        out.add("velocity", TraceJson.vector(packet.change().deltaMovement()));
        TraceJson.number(out, "yRot", packet.change().yRot());
        TraceJson.number(out, "xRot", packet.change().xRot());
        JsonArray relatives = new JsonArray();
        packet.relatives().stream().map(Enum::name).sorted().forEach(relatives::add);
        out.add("relatives", relatives);
        anomalies.add(out);
    }

    public static void motionPacket(ClientboundSetEntityMotionPacket packet) {
        var player = Minecraft.getInstance().player;
        if (output == null || start != null || player == null || packet.id() != player.getId()) return;
        JsonObject out = new JsonObject();
        out.addProperty("type", "ClientboundSetEntityMotionPacket");
        out.addProperty("id", packet.id());
        out.add("velocity", TraceJson.vector(packet.movement()));
        anomalies.add(out);
    }
}
