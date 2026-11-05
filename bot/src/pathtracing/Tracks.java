package pathtracing;

import com.google.gson.JsonObject;
import java.io.IOException;
import java.nio.file.Path;
import java.time.Instant;
import java.util.ArrayList;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.TreeMap;
import java.util.concurrent.CompletableFuture;
import net.fabricmc.fabric.api.client.command.v2.ClientCommands;
import net.fabricmc.fabric.api.client.command.v2.ClientCommandRegistrationCallback;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents;
import net.fabricmc.fabric.api.client.networking.v1.ClientPlayConnectionEvents;
import com.mojang.brigadier.arguments.StringArgumentType;
import net.minecraft.client.Minecraft;
import net.minecraft.core.BlockPos;
import net.minecraft.core.particles.DustParticleOptions;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.network.chat.Component;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.phys.AABB;

/** Client-only markers and one shared timer for the human and the bot. */
public final class Tracks {
    private static final org.slf4j.Logger LOGGER = org.slf4j.LoggerFactory.getLogger("pathtracing");
    private static final TrackDetector DETECTOR = new TrackDetector();
    private static final TrackStore STORE = new TrackStore(Path.of(System.getProperty("pathtracing.rig", ".")));
    private static List<TrackStore.Track> tracks = List.of();
    private static TrackStore.Track selected;
    private static String creating;
    private static TrackStore.Cell first;
    private static String creationDimension;
    private static long clock;
    private static List<TrackStore.Track> suppressed = List.of();
    private static AABB previousBox;
    private static long sampleNanos;
    private static long startNanos;
    private static boolean tickFault;
    private static long botLimit = 6000;
    private static String botMode;
    private static String loadError;
    private static JsonObject run;
    private static final TrackUsage USED = new TrackUsage();
    private static Path recording;
    private static String recordingReason;
    private static CompletableFuture<Void> preparation;
    private static volatile int generation;
    private static boolean settling;
    private static int settled;
    private static long preparationTick;
    private static long commandTick;
    private static long commandNanos;
    private static Long latency;
    private static Double latencyMs;
    private static boolean botAttempt;
    private static Map<String, Integer> toolkit = Map.of();
    private static TrackMode human;
    private static TrackHumanClient humanClient;

    static void register() {
        ClientCommandRegistrationCallback.EVENT.register((dispatcher, access) -> {
            var root = ClientCommands.literal("track");
            for (String action : List.of("new", "delete", "best", "kit")) {
                root.then(ClientCommands.literal(action).then(ClientCommands.argument("name", StringArgumentType.word())
                    .executes(context -> command(Minecraft.getInstance(), action + " " + StringArgumentType.getString(context, "name")))));
            }
            for (String action : List.of("list", "cancel", "exit")) {
                root.then(ClientCommands.literal(action).executes(context -> command(Minecraft.getInstance(), action)));
            }
            dispatcher.register(root);
        });
        ClientTickEvents.START_CLIENT_TICK.register(Tracks::tick);
        ClientPlayConnectionEvents.DISCONNECT.register((handler, mc) -> {
            if (humanClient != null) humanClient.disconnect();
            human = null;
            humanClient = null;
            cancel(mc, false);
            creating = null;
            first = null;
            tracks = List.of();
            suppressed = List.of();
            tickFault = false;
        });
    }

    static int command(Minecraft mc, String text) {
        return command(mc, text, System.nanoTime());
    }

    static int command(Minecraft mc, String text, long receivedNanos) {
        try {
            tickFault = false;
            String[] words = text.trim().split("\\s+");
            if (words.length == 1 && (words[0].equals("cancel") || words[0].equals("exit"))) {
                cancel(mc, true);
                creating = null;
                first = null;
                say(mc, "Track cancelled.");
            } else if (words.length == 1 && words[0].equals("list")) {
                reload();
                say(mc, tracks.isEmpty() ? "No tracks. Use /track new <name>." : "Tracks: "
                    + String.join(", ", tracks.stream().map(TrackStore.Track::name).toList()));
            } else if (words.length == 2) {
                TrackStore.validateName(words[1]);
                switch (words[0]) {
                    case "new" -> {
                        if (isBot()) throw new IllegalStateException("Create tracks in the player client.");
                        if (run != null || DETECTOR.running() || botAttempt || (human != null && human.active())) throw new IllegalStateException("A track run or track mode is in progress. Use /track exit first.");
                        if (STORE.list().stream().anyMatch(t -> t.name().equals(words[1]))) {
                            throw new IllegalArgumentException("Track already exists: " + words[1]);
                        }
                        cancel(mc, true);
                        creating = words[1];
                        first = null;
                        creationDimension = dimension(mc);
                        say(mc, "Track " + creating + ": right-click A, then B with the stick. Sneak + right-click cancels.");
                    }
                    case "delete" -> {
                        STORE.delete(words[1]);
                        if (selected != null && selected.name().equals(words[1])) cancel(mc, true);
                        reload();
                        say(mc, "Deleted track " + words[1] + ". Run history retained.");
                    }
                    case "best" -> {
                        var history = STORE.history(STORE.read(words[1]));
                        say(mc, words[1] + " human best: " + bestLine(history, false));
                        say(mc, words[1] + " bot best: " + bestLine(history, true));
                    }
                    case "kit" -> {
                        if (isBot()) throw new IllegalStateException("Set kits in the player client.");
                        human(mc).replaceKit(words[1], run != null || DETECTOR.running());
                    }
                    case "run" -> startBot(mc, words[1], receivedNanos);
                    default -> throw new IllegalArgumentException("Usage: track new/list/delete/best/kit/cancel/exit; bot track run <name>.");
                }
            } else throw new IllegalArgumentException("Usage: track new/list/delete/best/kit/cancel/exit; bot track run <name>.");
            return 1;
        } catch (IOException | RuntimeException e) {
            say(mc, "Track: " + e.getMessage());
            return 0;
        }
    }

    static boolean selecting() { return creating != null; }

    public static void serverTick() {
        if (!isBot() && human != null) human.serverTick();
    }

    static void click(Minecraft mc, BlockPos block, boolean cancel) {
        if (cancel) {
            command(mc, "cancel");
            return;
        }
        if (!dimension(mc).equals(creationDimension)) {
            say(mc, "Track endpoints must be in the same dimension. Use /track cancel.");
            return;
        }
        TrackStore.Cell cell = new TrackStore.Cell(block.getX(), block.getY(), block.getZ());
        if (first == null) {
            first = cell;
            say(mc, "Track " + creating + " A set at " + block.toShortString() + ". Right-click B.");
            return;
        }
        try {
            STORE.create(new TrackStore.Track(creating, first, cell, creationDimension, Instant.now().toString(), mc.player.getName().getString()));
            say(mc, "Track " + creating + " B set at " + block.toShortString() + ". Stand on A to arm.");
            creating = null;
            first = null;
            reload();
        } catch (IOException | RuntimeException e) {
            say(mc, "Track: " + e.getMessage());
        }
    }

    private static void reload() throws IOException {
        tracks = STORE.list();
        if (selected != null && !tracks.contains(selected)) {
            var mc = Minecraft.getInstance();
            cancel(mc, true);
            say(mc, "Track definition removed or changed. Run discarded.");
        }
    }

    private static void tick(Minecraft mc) {
        if (tickFault) return;
        try { tickBody(mc); }
        catch (Exception failure) {
            LOGGER.error("Track tick failed; cancelling the run", failure);
            tickFault = true;
            try { cancel(mc, true); }
            catch (Exception cleanup) { LOGGER.error("Track cancellation failed", cleanup); }
            try { say(mc, "Track tick failed; run cancelled. Use /track cancel to resume tracks."); }
            catch (Exception report) { LOGGER.error("Track error report failed", report); }
        }
    }

    private static void tickBody(Minecraft mc) throws IOException {
        clock++;
        if (mc.player == null || mc.level == null) return;
        if (!isBot()) {
            TrackMode mode = human(mc);
            if (selected != null && mode.active() && !mode.restoring() && mode.state() != TrackMode.State.FAILED && abandoned(mc)) {
                say(mc, "Track run discarded.");
                cancel(mc, true);
            }
            boolean wasRestoring = mode.restoring();
            mode.tick(clock);
            if (wasRestoring && !mode.active()) {
                // Recover first, independently of track files. Do not immediately re-enter
                // if the join/restore position is still on A.
                reload();
                suppressed = tracks.stream().filter(t -> t.dimension().equals(dimension(mc)) && onA(mc, t)).toList();
                return;
            }
            if (mode.active() && !mode.ready()) {
                if (!mode.restoring() && mode.state() != TrackMode.State.FAILED && selected != null && !onA(mc, selected)) {
                    mode.fail("Stay on A until the survival kit is ready.");
                }
                if (mode.state() == TrackMode.State.FAILED && selected != null) cancel(mc, true, false);
                return;
            }
        }
        if (clock % 20 == 0) {
            try {
                reload();
                loadError = null;
            } catch (IOException | RuntimeException e) {
                if (!java.util.Objects.equals(e.getMessage(), loadError)) say(mc, "Track files: " + e.getMessage());
                loadError = e.getMessage();
            }
        }
        if (!isBot() && clock % 5 == 0) markers(mc);
        if (preparation != null) return;
        if (settling) {
            settle(mc);
            return;
        }
        if (selected == null) {
            suppressed = suppressed.stream().filter(t -> t.dimension().equals(dimension(mc)) && onA(mc, t)).toList();
            if (creating != null || Recorder.replaying() || isBot()) return;
            selected = tracks.stream().filter(t -> !suppressed.contains(t) && t.dimension().equals(dimension(mc)) && onA(mc, t)).findFirst().orElse(null);
            if (selected == null) return;
            if (!isBot()) {
                try { human.enter(selected.name()); }
                catch (IOException | RuntimeException failure) {
                    human.fail("Cannot enter track mode: " + failure.getMessage());
                    cancel(mc, true, false);
                }
                return;
            }
        }
        boolean abandoned = !selected.dimension().equals(dimension(mc)) || !mc.player.isAlive()
            || mc.player.getY() < Math.min(selected.a().y(), selected.b().y()) - 64
            || (botAttempt && run != null && DETECTOR.running() && DETECTOR.ticks() + 1 >= botLimit);
        if (!isBot() && DETECTOR.active() && !DETECTOR.running() && !onA(mc, selected) && !human.validateStart()) {
            cancel(mc, true, false);
            return;
        }
        sampleNanos = System.nanoTime();
        var event = DETECTOR.sample(onA(mc, selected), touchesB(mc, selected), abandoned, clock, mc.level.getGameTime());
        previousBox = mc.player.getBoundingBox();
        switch (event) {
            case ARMED -> say(mc, selected.name() + " armed. Leave A to start.");
            case START -> begin(mc);
            case REARM -> {
                discardRecording(mc);
                run = null;
                USED.clear();
                if (!isBot()) human.rearm();
                mc.player.sendOverlayMessage(Component.literal(selected.name() + " re-armed."));
            }
            case STOP -> {
                if (run == null) begin(mc); // A and B can be crossed within the same sampled tick.
                finish(mc);
            }
            case DISCARD -> {
                say(mc, "Track run discarded.");
                cancel(mc, true);
            }
            default -> { }
        }
        if (selected != null && !DETECTOR.running() && run == null) USED.clear();
        if (DETECTOR.running() && mc.player.isFallFlying()) itemUsed(mc.player.getItemBySlot(net.minecraft.world.entity.EquipmentSlot.CHEST), 1);
        if (DETECTOR.running()) mc.player.sendOverlayMessage(Component.literal(selected.name() + " " + time(DETECTOR.ticks())));
    }

    private static boolean onA(Minecraft mc, TrackStore.Track track) {
        var a = track.a();
        return mc.player.onGround() && mc.player.mainSupportingBlockPos
            .filter(p -> p.getX() == a.x() && p.getY() == a.y() && p.getZ() == a.z()).isPresent();
    }

    private static boolean abandoned(Minecraft mc) {
        return !selected.dimension().equals(dimension(mc)) || !mc.player.isAlive()
            || mc.player.getY() < Math.min(selected.a().y(), selected.b().y()) - 64;
    }

    private static TrackMode human(Minecraft mc) throws java.io.UncheckedIOException {
        if (human == null) {
            humanClient = new TrackHumanClient(mc);
            human = new TrackMode(STORE, mc.player.getName().getString(), humanClient);
            try {
                human.recover();
            } catch (IOException failure) {
                human.fail("Cannot read the saved session: " + failure.getMessage());
                throw new java.io.UncheckedIOException(failure);
            } catch (RuntimeException failure) {
                human.fail("Cannot read the saved session: " + failure.getMessage());
                throw failure;
            }
        }
        return human;
    }

    private static boolean touchesB(Minecraft mc, TrackStore.Track track) {
        return TrackDetector.touchesB(previousBox, mc.player.getBoundingBox(), track.b());
    }

    private static void begin(Minecraft mc) {
        startNanos = sampleNanos;
        run = TickInput.settings(mc);
        run.addProperty("player", mc.player.getName().getString());
        run.addProperty("gameMode", mc.gameMode.getPlayerMode().getName());
        if (!isBot()) run.addProperty("kitSha256", human.kitHash());
        run.add("inventory", TrackStore.items(inventory(mc)));
        run.addProperty("date", Instant.now().toString());
        run.addProperty("trackCreated", selected.created());
        run.addProperty("dimension", selected.dimension());
        if (botAttempt) run.add("toolkit", TrackStore.items(toolkit));
        recordingReason = null;
        try {
            recording = Recorder.recordTrack(mc, STORE.rig.resolve("recordings"), selected.name());
        } catch (IOException | RuntimeException e) {
            recording = null;
            recordingReason = e.toString();
        }
    }

    private static void finish(Minecraft mc) {
        long ticks = DETECTOR.ticks();
        long gameTicks = DETECTOR.gameTicks();
        run.addProperty("T", ticks);
        run.addProperty("gameTimeDifference", gameTicks);
        run.addProperty("wallStartNanos", startNanos);
        run.addProperty("wallStopNanos", sampleNanos);
        run.addProperty("wallNanos", sampleNanos - startNanos);
        run.add("itemsUsed", TrackStore.items(USED.counts()));
        if (botAttempt) {
            if (latency != null) {
                run.addProperty("commandToFirstInputTicks", latency);
                run.addProperty("commandToFirstInputMs", latencyMs);
            } else run.addProperty("latencyNote", "No non-empty bot input before finish.");
        }
        if (recording != null) {
            if (Recorder.finishTrack(mc, recording)) run.addProperty("recording", "recordings/" + recording.getFileName());
            else recordingReason = "Recording aborted or could not be finalized; inspect the .tmp file.";
            recording = null;
        }
        if (recordingReason != null) run.addProperty("recordingNote", recordingReason);
        try {
            var saved = STORE.save(selected.name(), run);
            say(mc, selected.name() + " finished in " + time(ticks) + (ticks == gameTicks ? "" : "; level game time " + gameTicks + " ticks")
                + (saved.unsaved() == null ? ". Best " + run.get("player").getAsString() + " [" + run.get("gameMode").getAsString() + "] " + time(saved.best()) + "."
                    : ". Run append failed; result saved to " + saved.unsaved() + ".")
                + (recordingReason == null ? "" : " No recording: " + recordingReason));
        } catch (IOException | RuntimeException e) {
            say(mc, "Track result could not be saved: " + e.getMessage() + ". Result " + run);
        }
        cancel(mc, true);
    }

    private static void discardRecording(Minecraft mc) {
        try {
            if (recording != null) Recorder.discardTrack(mc, recording);
        } finally { recording = null; }
    }

    static void cancel(Minecraft mc, boolean stopBot) {
        cancel(mc, stopBot, true);
    }

    private static void cancel(Minecraft mc, boolean stopBot, boolean restoreHuman) {
        if (!isBot() && stopBot && restoreHuman && mc.player != null && mc.level != null) {
            try { human(mc).exit(); }
            catch (IOException | RuntimeException failure) { human.fail("Cannot restore saved session: " + failure.getMessage()); }
        }
        try {
            if (mc.player != null && mc.level != null) suppressed = tracks.stream()
                .filter(t -> t.dimension().equals(dimension(mc)) && onA(mc, t)).toList();
        } finally {
            previousBox = null;
            generation++;
            settling = false;
            boolean wasBot = botAttempt;
            selected = null;
            run = null;
            USED.clear();
            DETECTOR.reset();
            botAttempt = false;
            try {
                if (stopBot && wasBot) {
                    Bot.clearFollowPath();
                    TickInput.release(mc);
                }
            } finally { discardRecording(mc); }
        }
    }

    private static void startBot(Minecraft mc, String name, long receivedNanos) throws IOException {
        if (!isBot()) throw new IllegalStateException("Use /msg " + PathTracingMod.BOT + " track run <name>.");
        if (preparation != null) throw new IllegalStateException("Track preparation is still running.");
        if (Recorder.active() && recording == null) throw new IllegalStateException("A recording or replay is already running.");
        TrackStore.Track track = STORE.read(name);
        if (!track.dimension().equals(dimension(mc))) throw new IllegalStateException("Bot must be in the track's dimension.");
        Map<String, Integer> kit = STORE.toolkit(track, PathTracingMod.OWNER);
        JsonObject humanBest = STORE.humanBest(track, PathTracingMod.BOT);
        String mode = humanBest == null ? mc.gameMode.getPlayerMode().getName() : humanBest.get("gameMode").getAsString();
        if (!List.of("survival", "creative", "adventure", "spectator").contains(mode)) throw new IOException("Invalid baseline game mode.");
        checkCapacity(kit);
        String player = mc.player.getName().getString();
        if (!player.matches("[A-Za-z0-9_]{1,16}")) throw new IllegalArgumentException("Invalid bot player name.");
        cancel(mc, true);
        Bot.clearFollowPath();
        TickInput.release(mc);
        selected = track;
        botAttempt = true;
        botMode = mode;
        botLimit = humanBest == null ? 6000 : Math.multiplyExact(20, humanBest.get("T").getAsLong());
        Bot.trackMode();
        toolkit = Map.copyOf(kit);
        latency = null;
        latencyMs = null;
        commandTick = clock;
        commandNanos = receivedNanos;
        List<String> commands = new ArrayList<>();
        commands.add("gamemode " + mode + " " + player);
        commands.add("clear " + player);
        kit.forEach((item, count) -> commands.add("give " + player + " " + item + " " + count));
        var a = track.a();
        commands.add("tp " + player + " " + (a.x() + 0.5) + " " + (a.y() + 1.0) + " " + (a.z() + 0.5));
        int request = generation;
        preparation = CompletableFuture.runAsync(() -> {
            try { TrackRcon.execute(STORE.rig, commands, () -> request != generation); }
            catch (IOException e) { throw new java.io.UncheckedIOException(e); }
        });
        preparation.whenComplete((ignored, error) -> mc.execute(() -> {
            preparation = null;
            if (request != generation || mc.player == null || mc.level == null) return;
            if (error != null) {
                say(mc, "Track preparation failed: " + error.getCause().getMessage());
                cancel(mc, true);
                return;
            }
            settling = true;
            settled = 0;
            preparationTick = clock;
        }));
        say(mc, "Track " + name + ": preparing toolkit and teleport over RCON.");
    }

    private static void checkCapacity(Map<String, Integer> kit) throws IOException {
        long slots = 0;
        for (var entry : kit.entrySet()) {
            var id = net.minecraft.resources.Identifier.parse(entry.getKey());
            var item = BuiltInRegistries.ITEM.getOptional(id).orElseThrow(() -> new IOException("Unknown item " + entry.getKey()));
            int stackSize = new ItemStack(item).getMaxStackSize();
            slots += (entry.getValue() + (long) stackSize - 1) / stackSize;
        }
        if (slots > 36) throw new IOException("Track toolkit exceeds the bot's 36 inventory slots.");
    }

    private static void settle(Minecraft mc) {
        var a = selected.a();
        var velocity = mc.player.getDeltaMovement();
        boolean atStart = mc.player.getX() == a.x() + 0.5 && mc.player.getZ() == a.z() + 0.5;
        settled = atStart && onA(mc, selected) && velocity.x == 0 && velocity.z == 0 && inventory(mc).equals(toolkit)
            && mc.gameMode.getPlayerMode().getName().equals(botMode) ? settled + 1 : 0;
        if (settled == 5) {
            settling = false;
            DETECTOR.sample(true, false, false, clock, mc.level.getGameTime());
            previousBox = mc.player.getBoundingBox();
            say(mc, selected.name() + " armed. Toolkit " + TrackStore.items(toolkit) + ".");
            var b = selected.b();
            Bot.goTo(mc, new BlockPos(b.x(), b.y() + 1, b.z()));
        } else if (clock - preparationTick >= 200) {
            say(mc, "Track preparation failed: teleport, supporting block or exact toolkit did not settle within 200 ticks.");
            cancel(mc, true);
        }
    }

    static void firstInput(Minecraft mc, boolean nonEmpty) {
        if (!botAttempt || latency != null || !nonEmpty) return;
        latency = clock - commandTick;
        latencyMs = (System.nanoTime() - commandNanos) / 1e6;
        say(mc, "Track command to first input: " + latency + " ticks (" + String.format(Locale.ROOT, "%.3f", latencyMs) + " ms).");
    }

    public static void itemUsed(ItemStack stack, int count) {
        if (!DETECTOR.active() || stack == null || stack.isEmpty() || count <= 0) return;
        USED.use(BuiltInRegistries.ITEM.getKey(stack.getItem()).toString(), stack.getMaxStackSize(), count);
    }

    public static void attacked(ItemStack stack) {
        if (stack != null && stack.getMaxStackSize() == 1) itemUsed(stack, 1);
    }

    public static void itemBroken(ItemStack stack) {
        if (!DETECTOR.active() || stack.isEmpty() || stack.getMaxStackSize() != 1) return;
        USED.broken(BuiltInRegistries.ITEM.getKey(stack.getItem()).toString());
    }

    public static void rightClick(ItemStack before, ItemStack after, boolean success) {
        if (!success || before == null || before.isEmpty()) return;
        var player = Minecraft.getInstance().player;
        if (before.getMaxStackSize() == 1) itemUsed(before, 1);
        else if (player != null && !player.isUsingItem()) {
            int consumed = before.getCount() - (ItemStack.isSameItem(before, after) ? after.getCount() : 0);
            itemUsed(before, consumed > 0 ? consumed : player.hasInfiniteMaterials() ? 1 : 0);
        }
    }

    public static void completedUse(ItemStack stack) {
        itemUsed(stack, 1);
    }

    static boolean captureDeparture() { return DETECTOR.active() && !DETECTOR.running() && run == null && (isBot() || (human != null && human.ready())); }

    private static String bestLine(List<JsonObject> history, boolean bots) {
        var bests = TrackStore.bests(history, PathTracingMod.BOT, bots);
        return bests.isEmpty() ? "none." : String.join("; ", bests.stream().map(r -> r.get("player").getAsString()
            + " [" + r.get("gameMode").getAsString() + "] " + time(r.get("T").getAsLong())).toList());
    }

    private static Map<String, Integer> inventory(Minecraft mc) {
        Map<String, Integer> out = new TreeMap<>();
        var inventory = mc.player.getInventory();
        for (int slot = 0; slot < inventory.getContainerSize(); slot++) {
            ItemStack stack = inventory.getItem(slot);
            if (!stack.isEmpty()) out.merge(BuiltInRegistries.ITEM.getKey(stack.getItem()).toString(), stack.getCount(), Math::addExact);
        }
        return out;
    }

    private static void markers(Minecraft mc) {
        for (var track : tracks) {
            if (!track.dimension().equals(dimension(mc))) continue;
            marker(mc, track.a(), 0x40ff70);
            marker(mc, track.b(), 0xff5050);
        }
        if (first != null && dimension(mc).equals(creationDimension)) marker(mc, first, 0x40ff70);
    }

    private static void marker(Minecraft mc, TrackStore.Cell cell, int color) {
        int range = mc.options.getEffectiveRenderDistance();
        if (Math.abs((cell.x() >> 4) - (mc.player.blockPosition().getX() >> 4)) > range
            || Math.abs((cell.z() >> 4) - (mc.player.blockPosition().getZ() >> 4)) > range) return;
        mc.level.addParticle(new DustParticleOptions(color, 1.5f), true, true,
            cell.x() + 0.5, cell.y() + 1.15, cell.z() + 0.5, 0, 0.01, 0);
    }

    private static String dimension(Minecraft mc) { return mc.level.dimension().identifier().toString(); }
    private static boolean isBot() { return System.getProperty("pathtracing.role", "").equals("bot"); }
    private static String time(long ticks) { return ticks + " ticks (" + String.format(Locale.ROOT, "%.2f", ticks / 20.0) + " s)"; }
    private static void say(Minecraft mc, String text) { Bot.reply(mc, text); }
}
