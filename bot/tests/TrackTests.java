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
        recordings(root.resolve("recordings"));
        files(root.resolve("files"));
        rcon(root.resolve("rcon"));
        TrackModeTests.run(root.resolve("human-mode"));
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
        var b = new TrackStore.Cell(0, 5, 0);
        // A swimming/crawling-height player can have both endpoints outside a two-block fall.
        var above = new net.minecraft.world.phys.AABB(0.2, 6.1, 0.2, 0.8, 6.7, 0.8);
        var below = above.move(0, -2, 0);
        check(!TrackDetector.touchesB(null, above, b) && !TrackDetector.touchesB(null, below, b),
            "Neither endpoint overlaps the non-solid finish cell");
        check(TrackDetector.touchesB(above, below, b), "Swept stop detects a two-block-per-tick fall through non-solid B");
        var away = above.move(2, 0, 0);
        check(!TrackDetector.touchesB(away, away.move(0, -2, 0), b), "Swept stop rejects a fall beside B");
        detector.reset();
        event(detector, true, false, false, 60, 410, TrackDetector.Event.ARMED);
        event(detector, false, false, false, 61, 411, TrackDetector.Event.START);
        event(detector, false, TrackDetector.touchesB(above, below, b), false, 62, 412, TrackDetector.Event.STOP);
        check(detector.ticks() == 1, "Swept contact stops on the first crossing sample");
    }

    private static void usage() {
        var usage = new TrackUsage();
        for (int tick = 0; tick < 40; tick++) usage.use("minecraft:iron_pickaxe", 1, 1);
        check(usage.counts().get("minecraft:iron_pickaxe") == 1, "A 40-tick mining hold needs one pickaxe");
        usage.broken("minecraft:iron_pickaxe");
        for (int tick = 0; tick < 40; tick++) usage.use("minecraft:iron_pickaxe", 1, 1);
        check(usage.counts().get("minecraft:iron_pickaxe") == 2, "A broken pickaxe adds one replacement, not more swings");
        usage.broken("minecraft:iron_pickaxe");
        check(usage.counts().get("minecraft:iron_pickaxe") == 3, "Each break adds exactly one");
        for (int pearl = 0; pearl < 3; pearl++) usage.use("minecraft:ender_pearl", 16, 1);
        check(usage.counts().get("minecraft:ender_pearl") == 3, "Three thrown ender pearls need three units");
        usage.use("minecraft:stone", 64, 1);
        usage.use("minecraft:stone", 64, 1);
        usage.use("minecraft:stone", 64, 12);
        check(usage.counts().get("minecraft:stone") == 14, "Placement and stack drops count units");
        usage.use("minecraft:stone", 64, 0);
        check(usage.counts().get("minecraft:stone") == 14, "Unsuccessful drop adds nothing");
        usage.use("minecraft:water_bucket", 1, 40);
        check(usage.counts().get("minecraft:water_bucket") == 1, "Repeated reusable bucket use needs one bucket");
        usage.clear();
        check(usage.counts().isEmpty(), "Re-arm discards item uses");
        usage.use("minecraft:iron_pickaxe", 1, 1);
        check(usage.counts().get("minecraft:iron_pickaxe") == 1, "Re-arm also clears break history");
    }

    private static void recordings(Path root) throws Exception {
        Files.createDirectories(root);
        Files.writeString(root.resolve("track-course-human-7.jsonl"), "old\n");
        Files.writeString(root.resolve("track-course-human-9.jsonl.tmp"), "stale\n");
        var first = TrackRecording.open(root, "track-course-human-");
        check(first.path.getFileName().toString().equals("track-course-human-10.jsonl"), "First scan includes completed and temporary indices");
        first.writer.write("discard me\n");
        first.writer.flush();
        check(Files.size(first.temporary) == 0, "False-start data never reaches disk");
        first.discard();
        check(!Files.exists(first.path) && !Files.exists(first.temporary), "Re-arm/cancel/abandon discard removes reservation");
        // A rescan would notice this file and allocate 1001. Cached allocation must give 11.
        Files.writeString(root.resolve("track-course-human-1000.jsonl"), "unrelated later file\n");
        var second = TrackRecording.open(root, "track-course-human-");
        check(second.path.getFileName().toString().equals("track-course-human-11.jsonl"), "Next index is cached after the first scan");
        second.writer.write("kept\n");
        second.commit();
        check(Files.readString(second.path).equals("kept\n") && !Files.exists(second.temporary), "Finish publishes complete bytes and removes temporary name");
        var bot = TrackRecording.open(root, "track-course-bot-");
        check(!bot.path.equals(second.path), "Bot and human recording namespaces cannot collide");
        bot.discard();
    }

    private static JsonObject run(String player, long ticks, Map<String, Integer> items, String created) {
        JsonObject run = new JsonObject();
        run.addProperty("player", player);
        run.addProperty("gameMode", "survival");
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
        var creative = run("human", 2, Map.of(), created);
        creative.addProperty("gameMode", "creative");
        check(store.append(track.name(), creative) == 2, "Game modes have independent bests");
        check(store.append(track.name(), run("human", 25, Map.of(), created)) == 20, "Bot and creative times cannot become this player's survival best");
        check(store.append(track.name(), run("other-human", 15, Map.of(), created)) == 15, "Each human has a separate best");
        var bests = TrackStore.bests(store.history(track), "bot", false);
        check(bests.size() == 3, "Human bests group by both player and game mode");
        check(bests.stream().allMatch(r -> !r.get("player").getAsString().equals("bot")), "Human best line never contains bot runs");
        var bots = TrackStore.bests(store.history(track), "bot", true);
        check(bots.size() == 1 && bots.getFirst().get("T").getAsLong() == 10, "Bot best is separate");
        check(store.humanBest(track, "bot").get("gameMode").getAsString().equals("creative"), "Bot preparation uses the fastest human run's mode");
        check(store.append(track.name(), run("human", 80, Map.of(), "new-creation")) == 80, "Recreated tracks have their own best");
        List<Process> writers = new ArrayList<>();
        for (int i = 0; i < 2; i++) {
            writers.add(new ProcessBuilder(Path.of(System.getProperty("java.home"), "bin", "java.exe").toString(),
                "-cp", System.getProperty("java.class.path"), TrackTests.class.getName(), root.toString(), "writer").inheritIO().start());
        }
        for (Process writer : writers) check(writer.waitFor(20, TimeUnit.SECONDS) && writer.exitValue() == 0, "Concurrent writer completed");
        check(store.history("race").size() == 200, "Cross-process appends lose no lines");
        store.delete(track.name());
        check(store.list().isEmpty() && store.history(track.name()).size() == 8, "Delete retains audit history");
        Files.writeString(root.resolve("lab/tracks/runs/broken.jsonl"), "{broken\n");
        try { store.append("broken", run("human", 1, Map.of(), created)); throw new AssertionError("Corrupt history accepted"); }
        catch (RuntimeException expected) { checks++; }
        check(Files.readString(root.resolve("lab/tracks/runs/broken.jsonl")).equals("{broken\n"), "Corrupt history is preserved");
        var result = run("human", 42, Map.of("minecraft:ender_pearl", 3), created);
        var saved = store.save("broken", result);
        check(saved.failure() != null && saved.unsaved().getFileName().toString().matches("broken\\.unsaved-[0-9]+-[0-9]+\\.json"),
            "Corrupt history produces a timestamped unsaved result");
        check(com.google.gson.JsonParser.parseString(Files.readString(saved.unsaved())).equals(result), "Fallback retains every result field");
        Files.createDirectory(root.resolve("lab/tracks/runs/blocked.jsonl"));
        check(store.save("blocked", result).unsaved() != null, "An I/O append failure also saves the result");
        check(store.save("healthy", result).unsaved() == null, "Healthy append uses the normal run file");
        Path definitions = root.resolve("lab/tracks");
        Path stale = definitions.resolve("stale.json.tmp");
        Files.writeString(stale, "previous crash");
        var staleTrack = new TrackStore.Track("stale", track.a(), track.b(), track.dimension(), created, "human");
        store.create(staleTrack);
        check(store.read("stale").equals(staleTrack), "An old definition temp file cannot block a name");
        Path blocked = definitions.resolve("blocked.json");
        Files.createDirectory(blocked);
        Files.writeString(blocked.resolve("child"), "blocks move");
        try { TrackStore.publish(blocked, "new data"); throw new AssertionError("Failed publication accepted"); }
        catch (IOException expected) { checks++; }
        try (var paths = Files.list(definitions)) {
            check(paths.noneMatch(p -> p.getFileName().toString().startsWith("blocked.json.")), "Failed definition publication deletes its unique temp in finally");
        }
        try {
            store.create(new TrackStore.Track("same", track.a(), track.a(), track.dimension(), created, "human"));
            throw new AssertionError("Equal A and B accepted");
        } catch (IllegalArgumentException expected) { checks++; }
        check(!Files.exists(definitions.resolve("same.json")), "Equal endpoints are refused before publication");
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
