package pathtracing;

import com.google.gson.JsonObject;
import java.io.DataInputStream;
import java.io.IOException;
import java.net.ServerSocket;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.time.Instant;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicInteger;

/** Runs with the JDK and Gson, without starting Minecraft. */
public final class TrackTests {
    private static int checks;

    public static void main(String[] args) throws Exception {
        Path root = Path.of(args[0]);
        if (args.length > 1 && args[1].equals("writer")) {
            TrackStore store = new TrackStore(root);
            for (int index = 0; index < 100; index++) store.append("race", run("human", index + 1, Map.of(), "creation"));
            return;
        }
        detector();
        usage();
        files(root.resolve("files"));
        rcon(root.resolve("rcon"));
        TrackMixinAudit.run(Path.of(args[1]), Path.of(args[2]));
        System.out.println("Passed " + checks + " headless track checks.");
    }

    private static void check(boolean condition, String message) {
        checks++;
        if (!condition) throw new AssertionError(message);
    }

    private static void event(TrackDetector detector, boolean a, boolean b, boolean abandon, long tick, long game, TrackDetector.Event expected) {
        check(detector.sample(a, b, abandon, tick, game) == expected, "Detector event at tick " + tick + ", expected " + expected);
    }

    private static void detector() {
        var detector = new TrackDetector();
        event(detector, false, true, false, 0, 100, TrackDetector.Event.NONE);
        check(!detector.active(), "Touching B without A must not start a run");
        event(detector, true, false, false, 1, 101, TrackDetector.Event.ARMED);
        for (int tick = 2; tick < 10; tick++) event(detector, true, false, false, tick, tick + 100, TrackDetector.Event.NONE);
        event(detector, false, false, false, 10, 110, TrackDetector.Event.START);
        check(detector.ticks() == 0 && detector.gameTicks() == 0, "Start tick is zero");
        event(detector, false, false, false, 11, 111, TrackDetector.Event.NONE);
        event(detector, false, true, false, 12, 112, TrackDetector.Event.STOP);
        check(detector.ticks() == 2 && detector.gameTicks() == 2 && !detector.running(), "Stop is start-to-stop difference");
        event(detector, false, true, false, 13, 113, TrackDetector.Event.NONE);
        detector.reset();
        event(detector, true, false, false, 20, 220, TrackDetector.Event.ARMED);
        event(detector, false, false, false, 21, 221, TrackDetector.Event.START);
        event(detector, true, false, false, 22, 222, TrackDetector.Event.REARM);
        check(!detector.running() && detector.active() && detector.ticks() == 0, "Jump in place discarded and re-armed");
        event(detector, false, false, false, 23, 224, TrackDetector.Event.START);
        event(detector, false, true, false, 30, 229, TrackDetector.Event.STOP);
        check(detector.ticks() == 7 && detector.gameTicks() == 5, "Client and game clocks remain separate after re-arm");
        detector.reset();
        event(detector, true, false, false, 40, 300, TrackDetector.Event.ARMED);
        event(detector, false, true, false, 41, 301, TrackDetector.Event.STOP);
        check(detector.ticks() == 0, "A to B on the departure sample is a zero-tick run");
        detector.reset();
        event(detector, true, true, false, 50, 400, TrackDetector.Event.ARMED);
        event(detector, true, true, false, 51, 401, TrackDetector.Event.NONE);
        event(detector, false, false, false, 52, 402, TrackDetector.Event.START);
        event(detector, true, true, false, 53, 403, TrackDetector.Event.REARM);
        event(detector, false, false, true, 54, 404, TrackDetector.Event.DISCARD);
        check(!detector.active(), "Abandoning an armed run clears it");
        event(detector, true, false, false, 55, 405, TrackDetector.Event.ARMED);
        event(detector, false, false, false, 56, 406, TrackDetector.Event.START);
        event(detector, false, true, true, 57, 407, TrackDetector.Event.DISCARD);
        detector.reset();
        event(detector, false, true, false, 58, 408, TrackDetector.Event.NONE);
        check(!detector.active(), "Explicit cancel requires arming again");
    }

    private static void usage() {
        var usage = new TrackUsage();
        Object food = new Object();
        usage.rightClick("minecraft:apple", food);
        usage.complete("minecraft:apple", food);
        check(usage.counts().get("minecraft:apple") == 1, "Consumption must not double count its right-click");
        usage.complete("minecraft:bread", new Object());
        check(usage.counts().get("minecraft:bread") == 1, "Consumption started before the timer still counts");
        usage.rightClick("minecraft:stone", null);
        usage.rightClick("minecraft:stone", null);
        usage.add("minecraft:stone", 12);
        check(usage.counts().get("minecraft:stone") == 14, "Placement, repeated uses and stack drops count");
        usage.add("minecraft:stone", 0);
        check(usage.counts().get("minecraft:stone") == 14, "Unsuccessful drop adds nothing");
        usage.clear();
        check(usage.counts().isEmpty(), "Re-arm discards item uses");
        usage.complete("minecraft:apple", food);
        check(usage.counts().get("minecraft:apple") == 1, "Re-arm discards the old consumption token");
    }

    private static JsonObject run(String player, long ticks, Map<String, Integer> items, String created) {
        JsonObject run = new JsonObject();
        run.addProperty("player", player);
        run.addProperty("T", ticks);
        run.addProperty("trackCreated", created);
        run.add("itemsUsed", TrackStore.items(items));
        return run;
    }

    private static void files(Path root) throws Exception {
        TrackStore store = new TrackStore(root);
        check(store.list().isEmpty(), "An empty rig has no tracks");
        String created = Instant.now().toString();
        var track = new TrackStore.Track("course-1", new TrackStore.Cell(-4, 60, 3), new TrackStore.Cell(8, 65, 10),
            "minecraft:overworld", created, "human");
        store.create(track);
        check(store.read(track.name()).equals(track), "Definition roundtrip");
        check(store.list().equals(List.of(track)), "Track listing");
        try { store.create(track); throw new AssertionError("Overwrite accepted"); }
        catch (IOException expected) { checks++; }
        for (String name : List.of("../escape", "UPPER", "bad_name", "x/y", "")) {
            try { store.read(name); throw new AssertionError("Invalid name accepted"); }
            catch (IllegalArgumentException expected) { checks++; }
        }
        check(store.append(track.name(), run("human", 30, Map.of("minecraft:stone", 2, "minecraft:water_bucket", 1), created)) == 30, "First best");
        check(store.append(track.name(), run("human", 20, Map.of("minecraft:stone", 5, "minecraft:apple", 3), created)) == 20, "Improved best");
        store.append(track.name(), run("bot", 10, Map.of("minecraft:diamond", 100), created));
        store.append(track.name(), run("human", 50, Map.of("minecraft:gold_ingot", 100), "older-creation"));
        check(store.toolkit(track, "human").equals(Map.of("minecraft:stone", 5, "minecraft:water_bucket", 1, "minecraft:apple", 3)),
            "Toolkit uses per-item maxima, excludes bot runs and old definitions");
        check(store.history(track.name()).size() == 4, "Append preserves every run");
        check(store.history(track).size() == 3, "Current definition excludes archived runs");
        check(store.append(track.name(), run("human", 80, Map.of(), "new-creation")) == 80, "Recreated tracks have their own best");
        List<Process> writers = new ArrayList<>();
        for (int i = 0; i < 2; i++) {
            writers.add(new ProcessBuilder(Path.of(System.getProperty("java.home"), "bin", "java.exe").toString(),
                "-cp", System.getProperty("java.class.path"), TrackTests.class.getName(), root.toString(), "writer").inheritIO().start());
        }
        for (Process writer : writers) check(writer.waitFor(20, TimeUnit.SECONDS) && writer.exitValue() == 0, "Concurrent writer completed");
        check(store.history("race").size() == 200, "Cross-process appends lose no lines");
        store.delete(track.name());
        check(store.list().isEmpty() && store.history(track.name()).size() == 5, "Delete retains audit history");
        Files.writeString(root.resolve("lab/tracks/runs/broken.jsonl"), "{broken\n");
        try { store.append("broken", run("human", 1, Map.of(), created)); throw new AssertionError("Corrupt history accepted"); }
        catch (RuntimeException expected) { checks++; }
        check(Files.readString(root.resolve("lab/tracks/runs/broken.jsonl")).equals("{broken\n"), "Corrupt history is preserved");
    }

    private static void rcon(Path root) throws Exception {
        Files.createDirectories(root.resolve("server"));
        for (boolean reject : List.of(false, true)) {
            try (var server = new ServerSocket(0)) {
                Files.writeString(root.resolve("server/server.properties"), "rcon.port=" + server.getLocalPort() + "\nrcon.password=test-only\n");
                List<String> commands = new ArrayList<>();
                var handler = CompletableFuture.runAsync(() -> {
                    try (var socket = server.accept()) {
                        var input = new DataInputStream(socket.getInputStream());
                        for (int index = 1; index <= (reject ? 1 : 4); index++) {
                            int size = Integer.reverseBytes(input.readInt());
                            byte[] packet = new byte[size];
                            input.readFully(packet);
                            ByteBuffer data = ByteBuffer.wrap(packet).order(ByteOrder.LITTLE_ENDIAN);
                            int id = data.getInt();
                            int type = data.getInt();
                            String body = new String(packet, 8, size - 10, StandardCharsets.UTF_8);
                            if (index == 1) {
                                if (id != 1 || type != 3 || !body.equals("test-only")) throw new AssertionError("Bad RCON auth");
                            } else commands.add(body);
                            byte[] response = ByteBuffer.allocate(14).order(ByteOrder.LITTLE_ENDIAN).putInt(10)
                                .putInt(reject ? -1 : id).putInt(index == 1 ? 2 : 0).put((byte) 0).put((byte) 0).array();
                            for (byte value : response) socket.getOutputStream().write(value); // Deliberately fragmented.
                        }
                    } catch (IOException e) { throw new RuntimeException(e); }
                });
                List<String> expected = List.of("clear PathBot", "give PathBot minecraft:stone 5", "tp PathBot 0.5 61.0 0.5");
                if (reject) {
                    try { TrackRcon.execute(root, expected); throw new AssertionError("Rejected auth accepted"); }
                    catch (IOException error) { check(!error.getMessage().contains("test-only"), "Auth error contains no credential"); }
                } else TrackRcon.execute(root, expected);
                handler.get(10, TimeUnit.SECONDS);
                check(commands.equals(reject ? List.of() : expected), "RCON preserves command order and rejects unauthenticated commands");
            }
        }
        try (var server = new ServerSocket(0)) {
            Files.writeString(root.resolve("server/server.properties"), "rcon.port=" + server.getLocalPort() + "\nrcon.password=test-only\n");
            var handler = CompletableFuture.runAsync(() -> {
                try (var socket = server.accept()) {
                    var input = new DataInputStream(socket.getInputStream());
                    for (int id = 1; id <= 2; id++) {
                        int size = Integer.reverseBytes(input.readInt());
                        byte[] packet = new byte[size];
                        input.readFully(packet);
                        if (id == 2 && !new String(packet, 8, size - 10, StandardCharsets.UTF_8).equals("clear PathBot")) {
                            throw new AssertionError("Unexpected first command");
                        }
                        socket.getOutputStream().write(ByteBuffer.allocate(14).order(ByteOrder.LITTLE_ENDIAN)
                            .putInt(10).putInt(id).putInt(id == 1 ? 2 : 0).put((byte) 0).put((byte) 0).array());
                    }
                    if (input.read() != -1) throw new AssertionError("Commands sent after cancellation");
                } catch (IOException e) { throw new RuntimeException(e); }
            });
            AtomicInteger commands = new AtomicInteger();
            TrackRcon.execute(root, List.of("clear PathBot", "give PathBot minecraft:stone 5", "tp PathBot 0.5 61.0 0.5"),
                () -> commands.incrementAndGet() > 1);
            handler.get(10, TimeUnit.SECONDS);
            check(commands.get() == 2, "Cancellation stops the RCON batch before give and tp");
        }
    }
}
