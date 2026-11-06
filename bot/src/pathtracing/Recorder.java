package pathtracing;

import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import java.io.BufferedReader;
import java.io.BufferedWriter;
import java.io.IOException;
import java.math.BigDecimal;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardOpenOption;
import java.util.ArrayList;
import java.util.List;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents;
import net.fabricmc.fabric.api.client.networking.v1.ClientPlayConnectionEvents;
import net.minecraft.client.Minecraft;
import net.minecraft.network.protocol.game.ClientboundExplodePacket;
import net.minecraft.network.protocol.game.ClientboundPlayerPositionPacket;
import net.minecraft.network.protocol.game.ClientboundPlayerRotationPacket;
import net.minecraft.network.protocol.game.ClientboundSetEntityMotionPacket;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.entity.ai.attributes.Attributes;
import net.minecraft.world.phys.Vec3;

/** START applies controls, END records their result. Packets between ticks belong to the next END. */
public final class Recorder {
    private static BufferedWriter output;
    private static JsonObject header;
    private static Path outputPath;
    private static Path temporaryPath;
    private static String name;
    private static int tick;
    private static TickInput applied;
    private static List<TickInput> replay;
    private static double[] start;
    private static int settled;
    private static int settlingTicks;
    private static Integer previousTickCount;
    private static boolean teleported;
    private static boolean awaitStart;
    private static JsonArray anomalies = new JsonArray();
    private static JsonArray collisionSteps = new JsonArray();
    private static TrackRecording trackRecording;
    private static JsonObject departureInitial;
    private static JsonObject departureRow;
    private static JsonObject pendingDeparture;
    private static boolean capturingDeparture;

    static void register() {
        ClientTickEvents.START_CLIENT_TICK.register(Recorder::startTick);
        ClientTickEvents.END_CLIENT_TICK.register(Recorder::endTick);
        ClientPlayConnectionEvents.DISCONNECT.register((handler, mc) -> abort(mc, "Disconnected at tick " + tick + "."));
    }

    static boolean replaying() {
        return replay != null;
    }

    static boolean active() {
        return output != null || replaying();
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
        open(mc, runName, file(mc, "recordings", runName));
    }

    static Path recordTrack(Minecraft mc, Path directory, String track) throws IOException {
        if (active()) throw new IllegalStateException("A recording or replay is already running.");
        JsonObject metadata = TickInput.holdSettings(mc);
        if (departureRow == null || departureInitial == null
            || departureRow.get("tickCount").getAsInt() != mc.player.tickCount) {
            throw new IllegalStateException("Departure tick unavailable; run timed without a recording.");
        }
        // The player namespace lets both clients cache their own index without rescanning.
        trackRecording = TrackRecording.open(directory, "track-" + track + "-" + mc.player.getName().getString() + "-");
        output = trackRecording.writer;
        outputPath = trackRecording.path;
        temporaryPath = trackRecording.temporary;
        name = outputPath.getFileName().toString().replace(".jsonl", "");
        header = metadata;
        header.addProperty("type", "recording");
        header.add("initialPlayer", departureInitial);
        pendingDeparture = departureRow.deepCopy();
        pendingDeparture.addProperty("t", 0);
        tick = 1;
        previousTickCount = departureRow.get("tickCount").getAsInt();
        capturingDeparture = false;
        applied = null;
        anomalies = new JsonArray();
        return outputPath;
    }

    static boolean finishTrack(Minecraft mc, Path path) {
        if (output == null || !path.equals(outputPath) || replaying()) return false;
        finish(mc, true);
        return Files.exists(path);
    }

    static void discardTrack(Minecraft mc, Path path) {
        if (output != null && path.equals(outputPath) && !replaying()) finish(mc, false);
    }

    private static void open(Minecraft mc, String runName, Path path) throws IOException {
        JsonObject metadata = TickInput.holdSettings(mc);
        metadata.addProperty("type", "recording");
        if (Files.exists(path)) throw new java.nio.file.FileAlreadyExistsException(path.toString(), null, "Output already exists");
        Path temporary = path.resolveSibling(path.getFileName() + ".tmp");
        if (Files.exists(temporary)) throw new java.nio.file.FileAlreadyExistsException(temporary.toString(), null,
            "Temporary output already exists. Delete it before reusing this name.");
        Files.createDirectories(path.getParent());
        BufferedWriter writer = Files.newBufferedWriter(temporary, StandardCharsets.UTF_8, StandardOpenOption.CREATE_NEW);
        header = metadata;
        output = writer;
        outputPath = path;
        temporaryPath = temporary;
        name = runName;
        tick = 0;
        previousTickCount = null;
        applied = null;
        anomalies = new JsonArray();
    }

    static void replay(Minecraft mc, String runName, String outName) throws IOException {
        replay(mc, runName, outName, false);
    }

    static void replay(Minecraft mc, String runName, String outName, boolean waitForStart) throws IOException {
        if (output != null || replaying()) throw new IllegalStateException("A recording or replay is already running.");
        TickInput.holdSettings(mc);
        Path path = file(mc, "recordings", outName);
        if (Files.exists(path)) throw new IllegalStateException("Output already exists: " + path);
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
        open(mc, outName);
        replay = inputs;
        start = origin;
        settled = 0;
        settlingTicks = 0;
        teleported = false;
        awaitStart = waitForStart;
        Bot.clearFollowPath();
        TickInput.release(mc);
        mc.getConnection().sendCommand("tp @s " + BigDecimal.valueOf(origin[0]).toPlainString()
            + " " + BigDecimal.valueOf(origin[1]).toPlainString() + " " + BigDecimal.valueOf(origin[2]).toPlainString()
            + " " + new BigDecimal(Float.toString((float) origin[3])).toPlainString()
            + " " + new BigDecimal(Float.toString((float) origin[4])).toPlainString());
        Bot.reply(mc, "Replay " + name + ": waiting for the teleport and 5 settled ticks.");
    }

    static void stop(Minecraft mc) {
        finish(mc, true);
    }

    static void continueReplay() {
        if (!replaying() || !awaitStart || start != null) throw new IllegalStateException("Replay has not settled.");
        awaitStart = false;
    }

    private static void abort(Minecraft mc, String reason) {
        if (!active()) return;
        Bot.reply(mc, reason + (trackRecording == null ? " Aborted; partial output remains in " + temporaryPath + "." : " Aborted; track recording discarded."));
        finish(mc, false);
    }

    private static void finish(Minecraft mc, boolean clean) {
        boolean wasReplay = replaying();
        boolean complete = clean && (!wasReplay || tick == replay.size());
        if (wasReplay) ClockAudit.finish(tick, complete);
        replay = null;
        start = null;
        awaitStart = false;
        applied = null;
        anomalies = new JsonArray();
        if (wasReplay) TickInput.release(mc);
        if (output == null) return;
        BufferedWriter writer = output;
        output = null;
        TrackRecording track = trackRecording;
        trackRecording = null;
        try {
            if (track != null && !clean) {
                header = null;
                pendingDeparture = null;
                track.discard();
                return;
            }
            try (writer) {
                writePreamble(writer);
                JsonObject end = new JsonObject();
                end.addProperty("type", "end");
                end.addProperty("ticks", tick);
                end.addProperty("complete", complete);
                writer.write(end.toString());
                writer.write('\n');
            }
            if (clean) {
                if (track == null) Files.move(temporaryPath, outputPath);
                else track.commit();
                Bot.reply(mc, "Saved " + name + ": " + tick + " ticks.");
            }
        } catch (IOException e) {
            Bot.reply(mc, "Recording finalization failed; temporary output retained: " + e.getMessage());
        }
    }

    private static void startTick(Minecraft mc) {
        applied = null;
        capturingDeparture = output == null && Tracks.captureDeparture() && mc.player != null && mc.level != null;
        if (capturingDeparture) {
            departureInitial = PlayerSnapshot.capture(mc);
            applied = TickInput.capture(mc);
            anomalies = new JsonArray();
            collisionSteps = new JsonArray();
            return;
        }
        if (output == null) {
            departureInitial = null;
            departureRow = null;
            return;
        }
        collisionSteps = new JsonArray();
        if (mc.player == null || mc.level == null) {
            abort(mc, "Player or world unavailable at tick " + tick + ".");
            return;
        }
        if (replaying()) {
            if (start != null || awaitStart) {
                TickInput.release(mc);
                return;
            }
        }
        if (header != null) {
            if (!header.has("initialPlayer")) header.add("initialPlayer", PlayerSnapshot.capture(mc));
            try {
                writePreamble(output);
                output.flush();
                header = null;
            } catch (IOException e) {
                abort(mc, "Initial player write failed: " + e.getMessage());
                return;
            }
        }
        if (replaying()) {
            if (tick == 0) ClockAudit.begin(name);
            replay.get(tick).apply(mc);
        }
        applied = TickInput.capture(mc);
    }

    private static void endTick(Minecraft mc) {
        if ((output == null && !capturingDeparture) || mc.player == null) return;
        var player = mc.player;
        if (start != null) {
            settlingTicks++;
            boolean atStart = same(player.getX(), start[0]) && same(player.getY(), start[1]) && same(player.getZ(), start[2]);
            var velocity = player.getDeltaMovement();
            settled = teleported && atStart && player.onGround() && velocity.x == 0 && velocity.z == 0
                && player.getTicksFrozen() == 0 ? settled + 1 : 0;
            if (settled == 5) {
                start = null;
                anomalies = new JsonArray();
                if (awaitStart) Bot.reply(mc, "Settled " + name + "; waiting for rec continue.");
            } else if (settlingTicks >= 200) {
                abort(mc, "Replay start did not settle within 200 ticks of tp (tick " + settlingTicks + ").");
            }
            return;
        }
        if (applied == null) return;
        JsonObject out = new JsonObject();
        out.addProperty("t", tick);
        out.addProperty("tickCount", player.tickCount);
        out.addProperty("gameTime", mc.level.getGameTime());
        out.add("collisionSteps", collisionSteps);
        out.add("speedModifiers", TraceJson.speedModifiers(player.getAttribute(Attributes.MOVEMENT_SPEED)));
        out.addProperty("ticksFrozen", player.getTicksFrozen());
        boolean gap = previousTickCount != null && (long) player.tickCount - previousTickCount != 1;
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
        if (capturingDeparture) {
            departureRow = out;
            capturingDeparture = false;
            return;
        }
        try {
            output.write(out.toString());
            output.write('\n');
            output.flush();
            tick++;
            if (gap) {
                abort(mc, "tickCount did not advance by 1 at tick " + (tick - 1) + ": " + previousTickCount + " -> " + player.tickCount + ".");
                return;
            }
            previousTickCount = player.tickCount;
            anomalies = new JsonArray();
            if (replaying() && tick == replay.size()) stop(mc);
        } catch (IOException e) {
            abort(mc, "Recording write failed at tick " + tick + ": " + e.getMessage());
        }
    }

    private static void writePreamble(BufferedWriter writer) throws IOException {
        if (header != null) {
            writer.write(header.toString());
            writer.write('\n');
            header = null;
        }
        if (pendingDeparture != null) {
            writer.write(pendingDeparture.toString());
            writer.write('\n');
            pendingDeparture = null;
        }
    }

    private static boolean same(double a, double b) {
        return Double.doubleToRawLongBits(a) == Double.doubleToRawLongBits(b);
    }

    // Observe the actual collision request after vanilla's sneak/stuck processing.
    public static void collisionStep(Entity entity, Vec3 requested, Vec3 result) {
        if ((output == null && !capturingDeparture) || applied == null || entity != Minecraft.getInstance().player) return;
        JsonObject step = new JsonObject();
        step.add("requested", TraceJson.vectorBits(requested));
        step.add("result", TraceJson.vectorBits(result));
        collisionSteps.add(step);
    }

    public static void positionPacket(ClientboundPlayerPositionPacket packet) {
        if (output == null && !capturingDeparture) return;
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
        if ((output == null && !capturingDeparture) || start != null || player == null || packet.id() != player.getId()) return;
        JsonObject out = new JsonObject();
        out.addProperty("type", "ClientboundSetEntityMotionPacket");
        out.addProperty("id", packet.id());
        out.add("velocity", TraceJson.vector(packet.movement()));
        anomalies.add(out);
    }

    public static void rotationPacket(ClientboundPlayerRotationPacket packet) {
        if ((output == null && !capturingDeparture) || start != null || Minecraft.getInstance().player == null) return;
        JsonObject out = new JsonObject();
        out.addProperty("type", "ClientboundPlayerRotationPacket");
        TraceJson.number(out, "yRot", packet.yRot());
        TraceJson.number(out, "xRot", packet.xRot());
        out.addProperty("relativeY", packet.relativeY());
        out.addProperty("relativeX", packet.relativeX());
        anomalies.add(out);
    }

    public static void explosionPacket(ClientboundExplodePacket packet) {
        if ((output == null && !capturingDeparture) || start != null || Minecraft.getInstance().player == null || packet.playerKnockback().isEmpty()) return;
        JsonObject out = new JsonObject();
        out.addProperty("type", "ClientboundExplodePacket");
        out.add("center", TraceJson.vector(packet.center()));
        out.add("playerKnockback", TraceJson.vector(packet.playerKnockback().get()));
        anomalies.add(out);
    }
}
