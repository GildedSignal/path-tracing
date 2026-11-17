package pathtracing;

import com.google.gson.GsonBuilder;
import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import com.mojang.brigadier.CommandDispatcher;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.TreeMap;
import java.util.function.Consumer;

/** Real file store + production selection, transaction, detector, UI and command trees; simulated packets. */
final class TrackRaceTests {
    private static int checks;
    private static TrackInventory original, kit;
    private static void check(boolean condition, String reason) { checks++; if (!condition) throw new AssertionError(reason); }
    private static TrackStore store(Path path) throws Exception {
        var store = new TrackStore(path);
        store.create(new TrackStore.Track("course", new TrackStore.Cell(0, 60, 0), new TrackStore.Cell(10, 60, 0), "minecraft:overworld", "creation", "human"));
        store.writeKit("course", kit);
        return store;
    }

    /** FIFO network, independently clocked server player effects, and delayed server responses. */
    private static final class Server implements TrackMode.Port {
        final TrackStore store;
        final int delay;
        long now;
        TrackInventory client = original, inventory = original;
        String clientMode = "adventure", serverMode = "adventure";
        boolean full, rejectNextSlots, rejectQueries;
        int queries;
        Consumer<Boolean> effect = ignored -> { };
        final TreeMap<Long, List<Runnable>> packets = new TreeMap<>();
        final List<String> chat = new ArrayList<>(), overlay = new ArrayList<>(), commands = new ArrayList<>();
        Server(TrackStore store, int delay) { this.store = store; this.delay = delay; }
        void after(int ticks, Runnable callback) { packets.computeIfAbsent(now + ticks, ignored -> new ArrayList<>()).add(callback); }
        void clock(long tick) {
            now = tick;
            while (!packets.isEmpty() && packets.firstKey() <= tick) for (var task : packets.pollFirstEntry().getValue()) task.run();
        }
        void durable() {
            try { check(store.readSession("human").inventory().equals(original), "Every mutation preserves the durable original"); }
            catch (Exception failure) { throw new AssertionError(failure); }
        }
        @Override public TrackInventory inventory() { return client; }
        @Override public String mode() { return clientMode; }
        @Override public boolean alive() { return true; }
        @Override public void checkEntry() { }
        @Override public void checkRestorable(TrackInventory value) { }
        @Override public void say(String value) { chat.add(value); }
        @Override public void command(String command) {
            durable(); commands.add(command);
            after(delay, () -> {
                if (command.startsWith("gamemode ")) {
                    serverMode = command.substring(9);
                    String acknowledged = serverMode;
                    after(delay, () -> clientMode = acknowledged);
                } else if (command.startsWith("effect ")) {
                    boolean health = command.contains("instant_health");
                    after(1, () -> { full = true; after(delay, () -> effect.accept(health)); });
                }
            });
        }
        @Override public void apply(TrackInventory value) {
            durable();
            check(clientMode.equals("creative"), "Client creative confirmation precedes slot packets");
            after(delay, () -> {
                check(serverMode.equals("creative"), "Server creative precedes slot writes");
                if (rejectNextSlots) rejectNextSlots = false; else inventory = value;
            });
        }
        @Override public void confirm(TrackInventory value) { client = value; }
        @Override public void query(Consumer<TrackMode.Observation> response, Consumer<String> failure) {
            queries++;
            after(delay, () -> {
                var observation = new TrackMode.Observation(inventory, serverMode, full);
                boolean refused = rejectQueries;
                after(delay, () -> { if (refused) failure.accept("query refused"); else response.accept(observation); });
            });
        }
    }

    private static TrackHumanRace race(Server server, TrackDetector detector) throws Exception {
        var mode = new TrackMode(server.store, "human", server);
        server.effect = mode::refillEffectRemoved;
        return new TrackHumanRace(mode, detector, server.overlay::add, server.chat::add);
    }

    /** Original Tracks.tickBody human branches, including its 20-tick file poll and recovery suppression. */
    private static final class BeforeRace {
        final BeforeTrackMode mode;
        final TrackDetector detector = new TrackDetector();
        boolean loaded, selected, suppressed;
        BeforeRace(Server server) throws Exception {
            mode = new BeforeTrackMode(server.store, "human", new BeforeTrackMode.Port() {
                public TrackInventory inventory() { return server.inventory(); }
                public String mode() { return server.mode(); }
                public boolean alive() { return true; }
                public void checkEntry() { }
                public void checkRestorable(TrackInventory value) { }
                public void command(String value) { server.command(value); }
                public void apply(TrackInventory value) { server.apply(value); }
                public void confirm(TrackInventory value) { server.confirm(value); }
                public void say(String value) { server.say(value); }
                public void query(Consumer<BeforeTrackMode.Observation> response, Consumer<String> failure) {
                    server.query(o -> response.accept(new BeforeTrackMode.Observation(o.inventory(), o.mode(), o.full())), failure);
                }
            });
            mode.recover();
        }
        boolean tick(long tick) {
            if (tick % 20 == 0) mode.serverTick();
            boolean restoring = mode.restoring();
            mode.tick(tick);
            if (restoring && !mode.active()) { loaded = true; suppressed = true; return false; }
            if (!mode.active() && selected) { selected = false; suppressed = true; return false; }
            if (mode.active() && !mode.ready()) return false;
            if (tick % 20 == 0) loaded = true;
            if (!selected) {
                if (!loaded || suppressed) return false;
                selected = true; mode.enter("course"); return false;
            }
            detector.sample(true, false, false, tick, tick);
            return detector.active() && mode.ready();
        }
    }

    static void run(Path root, TrackInventory saved, TrackInventory raceKit) throws Exception {
        original = saved; kit = raceKit;
        commandWiring(root.resolve("commands"));
        lifecycle(root.resolve("lifecycle"));
        JsonObject evidence = new JsonObject();
        evidence.addProperty("baselineCommit", System.getProperty("pathtracing.beforeCommit"));
        var failedStore = store(root.resolve("before-failed-kit"));
        var failedServer = new Server(failedStore, 1);
        failedServer.rejectNextSlots = true;
        var failedRace = new BeforeRace(failedServer);
        boolean failedArmed = false;
        for (long tick = 1; tick <= 240; tick++) { failedServer.clock(tick); failedArmed |= failedRace.tick(tick); }
        check(!failedArmed && failedRace.mode.state() == BeforeTrackMode.State.FAILED && failedStore.readSession("human") != null,
            "Frozen baseline reproduces the retained FAILED session that needs restore");
        evidence.addProperty("beforeFailedKit", "FAILED with saved session, never armed in 240 ticks");
        evidence.addProperty("clock", "client ticks from first sampled contact with A; serverbound and clientbound delay=1..4 ticks, FIFO; effects tick one server tick later; phases=1..20");
        JsonArray before = new JsonArray(), after = new JsonArray(), beforeRecovery = new JsonArray(), afterRecovery = new JsonArray();
        for (int i = 0; i < 20; i++) {
            for (boolean stale : new boolean[] {false, true}) {
                int delay = i % 4 + 1;
                long step = i + 1;
                var oldStore = store(root.resolve("before-" + stale + "-" + i));
                var newStore = store(root.resolve("after-" + stale + "-" + i));
                if (stale) {
                    oldStore.writeSession("human", new TrackStore.Session(original, "adventure"));
                    newStore.writeSession("human", new TrackStore.Session(original, "adventure"));
                }
                var oldServer = new Server(oldStore, delay); oldServer.now = step;
                var newServer = new Server(newStore, delay); newServer.now = step;
                var oldRace = new BeforeRace(oldServer);
                var newRace = race(newServer, new TrackDetector());
                var contacts = List.of(newStore.read("course"));
                Long oldTime = null, newTime = null;
                for (long tick = step; tick < step + 240; tick++) {
                    oldServer.clock(tick); newServer.clock(tick);
                    if (oldTime == null && oldRace.tick(tick)) oldTime = tick - step;
                    if (newTime == null) {
                        newRace.tick(contacts, true, false, false, tick, tick);
                        if (newServer.overlay.getLast().equals("Armed: go")) newTime = tick - step;
                    }
                }
                check(newTime != null, "Selection through recovery, kit confirmation, detector and Armed completes");
                if (stale) { beforeRecovery.add(oldTime); afterRecovery.add(newTime); }
                else { check(oldTime != null, "Baseline fresh entry arms"); before.add(oldTime); after.add(newTime); }
            }
        }
        evidence.add("beforeFresh", before); evidence.add("afterFresh", after);
        evidence.add("beforeStale", beforeRecovery); evidence.add("afterStale", afterRecovery);
        evidence.add("afterRepeatedReset", repeatedReset(root.resolve("reset")));
        evidence.addProperty("beforeStaleObservationWindow", 240);
        evidence.addProperty("refusals", "null is no arming within 240 ticks; intentional slot refusals and untimed departures tested in lifecycle; no unexplained refusals");
        evidence.addProperty("checks", checks);
        Path record = Path.of(System.getProperty("pathtracing.uxEvidence", root.resolve("arming.json").toString()));
        Files.createDirectories(record.getParent());
        Files.writeString(record, new GsonBuilder().setPrettyPrinting().create().toJson(evidence) + "\n");
        System.out.println("Track race evidence: " + evidence);
    }

    private static List<String> suggestions(CommandDispatcher<Object> dispatcher, String text) {
        return dispatcher.getCompletionSuggestions(dispatcher.parse(text, new Object())).join().getList().stream().map(x -> x.getText()).toList();
    }

    private static JsonArray repeatedReset(Path root) throws Exception {
        var store = store(root);
        var server = new Server(store, 1);
        var detector = new TrackDetector();
        var race = race(server, detector);
        var contacts = List.of(store.read("course"));
        long tick = 0;
        do { server.clock(++tick); race.tick(contacts, true, false, false, tick, tick); }
        while (!server.overlay.getLast().equals("Armed: go") && tick < 80);
        check(detector.active(), "Reset fixture arms");
        var times = new JsonArray();
        for (int i = 0; i < 20; i++) {
            server.full = false;
            detector.requestRearm(); // The same request used by the race stick.
            long step = ++tick;
            server.clock(tick); race.tick(contacts, true, false, false, tick, tick);
            check(server.overlay.getLast().equals("Arming\u2026"), "Reset immediately withdraws Armed while refilling");
            while (!server.overlay.getLast().equals("Armed: go") && tick < step + 80) {
                server.clock(++tick); race.tick(contacts, true, false, false, tick, tick);
            }
            check(detector.active() && server.full, "Every repeated reset is confirmed before Armed");
            times.add(tick - step);
        }
        race.cancel(List.of(), true);
        for (int i = 0; i < 40; i++) { server.clock(++tick); race.tick(List.of(), true, false, false, tick, tick); }
        check(store.readSession("human") == null && server.client.equals(original), "Repeated resets preserve and restore the original");
        return times;
    }
    private static void commandWiring(Path root) throws Exception {
        var store = store(root);
        for (String name : List.of("course_two", "other")) store.create(new TrackStore.Track(name, new TrackStore.Cell(0, 1, 0), new TrackStore.Cell(1, 1, 0), "minecraft:overworld", "creation", "human"));
        var dispatcher = new CommandDispatcher<Object>();
        var calls = new ArrayList<String>();
        RaceCommands.tracks(dispatcher, store, text -> { calls.add(text); return 1; });
        Files.writeString(root.resolve("maps.json"), "[{\"name\":\"course_map\",\"spawn\":[0,1,0]},{\"name\":\"other_map\",\"spawn\":[1,2,3]}]");
        RaceCommands.maps(dispatcher, root.resolve("maps.json"), text -> { calls.add("map " + text); return 1; });
        for (String action : List.of("delete", "best", "kit", "run", "play")) {
            check(suggestions(dispatcher, "track " + action + " co").equals(List.of("course", "course_two")), action + " is wired to stored names");
            check(suggestions(dispatcher, "track " + action + " z").isEmpty(), action + " rejects nonmatching prefixes");
            dispatcher.execute("track " + action + " course", new Object());
            check(calls.getLast().equals(action + " course"), action + " dispatches exact name");
        }
        check(suggestions(dispatcher, "track new co").isEmpty(), "New track names have no suggestions");
        check(suggestions(dispatcher, "map ").equals(List.of("course_map", "other_map", "overworld")), "Map tree includes registry and overworld");
        check(suggestions(dispatcher, "map o").equals(List.of("other_map", "overworld")), "Map tree filters prefix");
        dispatcher.execute("map course_map", new Object());
        check(calls.getLast().equals("map course_map"), "Map name dispatches");
    }

    private static void lifecycle(Path root) throws Exception {
        for (String origin : List.of("crash", "disconnect", "unfinished-exit", "clean")) {
            var store = store(root.resolve(origin));
            var server = new Server(store, 1);
            server.now = 1000;
            if (!origin.equals("clean")) {
                var interruptedMode = new TrackMode(store, "human", server);
                server.effect = interruptedMode::refillEffectRemoved;
                var interrupted = new TrackHumanRace(interruptedMode, new TrackDetector(), server.overlay::add, server.chat::add);
                var contact = List.of(store.read("course"));
                for (long t = server.now + 1, limitTick = t + 60; t < limitTick; t++) {
                    server.clock(t); interrupted.tick(contact, true, false, false, t, t);
                    if (origin.equals("disconnect") && interruptedMode.state() == TrackMode.State.INVENTORY) break;
                    if (!origin.equals("disconnect") && interruptedMode.ready()) break;
                }
                check(store.readSession("human") != null, origin + " interrupts a real durable transaction");
                if (origin.equals("unfinished-exit")) {
                    interrupted.cancel(List.of(), true);
                    check(interruptedMode.restoring(), "Exit is interrupted before restoration confirmation");
                }
                server.packets.clear(); // Old connection callbacks are discarded by TrackHumanClient.disconnect.
            }
            var detector = new TrackDetector();
            var race = race(server, detector);
            var contacts = List.of(store.read("course"));
            long limit = server.now + 80;
            long tick = server.now + 1;
            for (; tick < limit; tick++) {
                server.clock(tick); race.tick(contacts, true, false, false, tick, tick);
                check(server.overlay.getLast().equals("Arming\u2026") || server.overlay.getLast().equals("Armed: go"), "Continuous arming state on A");
                if (server.overlay.getLast().equals("Armed: go")) break;
            }
            check(tick < limit && detector.active(), origin + " recovers and arms without a command");
            check(store.readSession("human").inventory().equals(original), origin + " captures the restored original for the new run");
            race.tick(contacts, true, false, false, ++tick, tick);
            check(server.overlay.getLast().equals("Armed: go"), "Armed overlay repeats on A");
            check(race.tick(List.of(), true, false, false, ++tick, tick) == TrackDetector.Event.START, "An armed departure starts timing");
            check(race.tick(List.of(), true, true, false, ++tick, tick) == TrackDetector.Event.STOP, "An armed arrival counts");
        }
        for (String fault : List.of("slots", "capture", "early-departure", "changed-kit")) {
            var store = store(root.resolve(fault));
            var server = new Server(store, 1);
            server.rejectNextSlots = fault.equals("slots");
            server.rejectQueries = fault.equals("capture");
            var detector = new TrackDetector();
            var race = race(server, detector);
            var contacts = List.of(store.read("course"));
            long tick = 1;
            server.clock(tick); race.tick(contacts, true, false, false, tick, tick);
            check(server.overlay.getLast().equals("Arming\u2026"), "First contact says Arming");
            if (!fault.equals("early-departure")) {
                for (; tick < 60; tick++) {
                    server.clock(tick); race.tick(contacts, true, false, false, tick, tick);
                    if (server.overlay.getLast().startsWith("Not armed:") || server.overlay.getLast().equals("Armed: go")) break;
                }
                if (fault.equals("slots") || fault.equals("capture")) {
                    check(server.overlay.getLast().startsWith("Not armed:"), "Refused kit reports reason on A");
                    for (int i = 0; i < 40; i++) {
                        server.clock(++tick); race.tick(contacts, true, false, false, tick, tick);
                        check(server.overlay.getLast().startsWith("Not armed:"), "Refusal stays visible while rollback completes on A");
                    }
                    check(store.readSession("human") == null && server.client.equals(original), "A failed kit restores on A without waiting for departure");
                    int commands = server.commands.size();
                    int queries = server.queries;
                    race.tick(contacts, true, false, false, ++tick, tick);
                    check(server.commands.size() == commands && server.queries == queries, "A refused kit never enters an automatic retry loop");
                }
                else server.client = kit; // Remove the required inventory content from an otherwise armed client.
                if (fault.equals("changed-kit")) server.client = new TrackInventory(java.util.Collections.nCopies(41, "{}"), 0);
            }
            int messages = server.chat.size();
            server.rejectQueries = false;
            check(race.tick(List.of(), true, false, false, ++tick, tick) == TrackDetector.Event.DISCARD, "Unarmed departure is discarded");
            check(server.chat.subList(messages, server.chat.size()).stream().filter(s -> s.startsWith("This run is not timed: ")).count() == 1, "Departure immediately emits exactly one untimed chat line");
            for (int i = 0; i < 40; i++) { server.clock(++tick); race.tick(List.of(), true, true, false, tick, tick); }
            check(store.readSession("human") == null && server.client.equals(original), "Failed or abandoned kit rolls back automatically");
            for (int i = 0; i < 40; i++) {
                server.clock(++tick); race.tick(contacts, true, false, false, tick, tick);
                if (server.overlay.getLast().equals("Armed: go")) break;
            }
            check(server.overlay.getLast().equals("Armed: go"), "A later contact arms after " + fault + " without restore");
        }
    }
}
