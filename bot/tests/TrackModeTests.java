package pathtracing;

import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import io.netty.buffer.Unpooled;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import java.util.function.Consumer;
import net.minecraft.SharedConstants;
import net.minecraft.core.HolderLookup;
import net.minecraft.core.RegistryAccess;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.data.registries.VanillaRegistries;
import net.minecraft.nbt.CompoundTag;
import net.minecraft.nbt.ListTag;
import net.minecraft.nbt.NbtOps;
import net.minecraft.network.RegistryFriendlyByteBuf;
import net.minecraft.network.protocol.game.ServerboundSetCreativeModeSlotPacket;
import net.minecraft.server.Bootstrap;
import net.minecraft.world.ItemStackWithSlot;
import net.minecraft.world.entity.EntityEquipment;
import net.minecraft.world.entity.EquipmentSlot;
import net.minecraft.world.flag.FeatureFlags;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.item.Items;

/** No world, client, connection or game process. Real Mojang codecs plus a controllable server port. */
final class TrackModeTests {
    private static int checks;
    private static TrackInventory original;
    private static TrackInventory kit;
    private static HolderLookup.Provider registries;

    static void run(Path root) throws Exception {
        SharedConstants.tryDetectVersion();
        Bootstrap.bootStrap();
        registries = VanillaRegistries.createLookup();
        BuiltInRegistries.DATA_COMPONENT_INITIALIZERS.build(registries).forEach(pending -> pending.apply());
        codecs(root.resolve("files"));
        allItems();
        entryCapture(root.resolve("entry"));
        transactions(root.resolve("transactions"));
        failures(root.resolve("failures"));
        serverOrdering(root.resolve("server-ordering"));
        System.out.println("Passed " + checks + " inventory and human track-mode checks.");
    }

    private static void allItems() {
        var network = RegistryAccess.fromRegistryOfRegistries(BuiltInRegistries.REGISTRY);
        List<String> failures = new ArrayList<>();
        int stacks = 0;
        for (var item : BuiltInRegistries.ITEM) {
            var one = new ItemStack(item);
            for (int count : new int[] {1, one.getMaxStackSize()}) {
                stacks++;
                var original = new ItemStack(item, count);
                var id = BuiltInRegistries.ITEM.getKey(item) + " x" + count;
                var buf = new RegistryFriendlyByteBuf(Unpooled.buffer(), network);
                String stage = "save codec";
                try {
                    var slots = new ArrayList<>(Collections.nCopies(TrackInventory.SIZE, ItemStack.EMPTY));
                    slots.set(0, original);
                    var inventory = TrackInventoryCodec.encode(slots, 0, registries);
                    stage = "session JSON";
                    var saved = new TrackStore.Session(inventory, "creative");
                    var restored = TrackStore.Session.read(JsonParser.parseString(TrackStore.canonical(saved.json())).getAsJsonObject());
                    var decoded = TrackInventoryCodec.decode(restored.inventory(), registries).getFirst();
                    if (!ItemStack.matches(original, decoded)) throw new IllegalStateException("Saved stack changed");
                    stage = "creative packet";
                    var codec = ServerboundSetCreativeModeSlotPacket.STREAM_CODEC;
                    codec.encode(buf, new ServerboundSetCreativeModeSlotPacket(TrackInventory.FIRST, decoded));
                    var received = codec.decode(buf);
                    if (received.slotNum() != TrackInventory.FIRST || !ItemStack.matches(original, received.itemStack()) || buf.isReadable()) {
                        throw new IllegalStateException("Decoded packet changed stack or left unread bytes");
                    }
                    stage = "server slot validation";
                    if (!received.itemStack().isItemEnabled(FeatureFlags.DEFAULT_FLAGS)
                        || (!received.itemStack().isEmpty() && received.itemStack().getCount() > received.itemStack().getMaxStackSize())) {
                        throw new IllegalStateException("Default server would reject stack");
                    }
                } catch (RuntimeException failure) {
                    failures.add(id + " (" + stage + "): " + failure);
                } finally {
                    buf.release();
                }
            }
        }
        System.out.println("Registry inventory round trips: " + BuiltInRegistries.ITEM.size() + " items, "
            + stacks + " stacks, " + failures.size() + " failures.");
        check(failures.isEmpty(), String.join("\n", failures));
    }

    private static void entryCapture(Path root) throws Exception {
        var store = store(root);
        store.writeKit("course", kit);
        var port = new Server(store);
        var mode = new TrackMode(store, "human", port);
        var stacks = new ArrayList<>(TrackInventoryCodec.decode(original, registries));
        stacks.set(10, new ItemStack(Items.SULFUR_SPIKE, 64));
        var withSpike = TrackInventoryCodec.encode(stacks, original.selected, registries);
        port.server = withSpike;
        port.reply = false;
        mode.enter("course");
        check(mode.state() == TrackMode.State.CAPTURE && store.readSession("human") == null
            && port.commands.isEmpty() && port.applications == 0, "Entry waits for server capture before mutating");
        port.answer();
        check(!mode.active() && store.readSession("human") == null && port.server.equals(withSpike),
            "A stale client cannot overwrite a server-only sulfur spike");
        check(port.messages.getLast().contains("Slot 15") && port.messages.getLast().contains("minecraft:sulfur_spike"),
            "Capture disagreement identifies the slot and sulfur spike");
        port.client = withSpike;
        port.expectedSaved = withSpike;
        mode.enter("course");
        mode.exit();
        port.answer();
        check(!mode.active() && store.readSession("human") == null, "Cancelling entry invalidates the capture reply");
        port.reply = true;
        mode.enter("course");
        ready(mode, 1);
        check(!port.server.equals(withSpike) && store.readSession("human").inventory().equals(withSpike),
            "An existing kit without the spike leaves the original safe in the session");
        mode.exit();
        mode.tick(10);
        port.reply = false;
        mode.tick(11);
        port.server = original;
        port.answer();
        check(mode.state() == TrackMode.State.FAILED && store.readSession("human").inventory().equals(withSpike)
            && port.client.equals(kit), "A spike missing from final readback keeps the session and does not confirm a local copy");
        check(port.messages.getLast().contains("Slot 15") && port.messages.getLast().contains("minecraft:sulfur_spike")
            && port.messages.getLast().contains("/track restore"), "Restore failure names the slot, item, and command");
        port.reply = true;
        check(mode.exit(), "A saved session can be restored again");
        mode.tick(12);
        mode.tick(13);
        check(!mode.active() && port.server.equals(withSpike) && port.client.equals(withSpike) && store.readSession("human") == null,
            "Retry restores the spike and deletes the verified session");

        var network = RegistryAccess.fromRegistryOfRegistries(BuiltInRegistries.REGISTRY);
        var plainStacks = new ArrayList<>(Collections.nCopies(TrackInventory.SIZE, ItemStack.EMPTY));
        plainStacks.set(10, new ItemStack(Items.SULFUR_SPIKE, 64));
        var plain = TrackInventoryCodec.encode(plainStacks, 0, registries);
        check(TrackInventoryCodec.creativePackets(plain, network, FeatureFlags.DEFAULT_FLAGS).size() == TrackInventory.SIZE,
            "Default feature flags and the full stack count accept the sulfur spike");
        var impossible = new TrackInventory(Collections.nCopies(TrackInventory.SIZE, "{id:'minecraft:stick',count:65}"), 0);
        try {
            TrackInventoryCodec.creativePackets(impossible, network, FeatureFlags.DEFAULT_FLAGS);
            throw new AssertionError("Oversized creative stack accepted");
        } catch (IllegalArgumentException expected) {
            check(expected.getMessage().contains("slot 5") && expected.getMessage().contains("minecraft:stick")
                && expected.getMessage().contains("maximum count"), "Preflight identifies an oversized stack and its slot");
        }
    }

    private static void check(boolean condition, String message) {
        checks++;
        if (!condition) throw new AssertionError(message);
    }

    private static ItemStack item(String snbt) {
        return ItemStack.CODEC.parse(registries.createSerializationContext(NbtOps.INSTANCE), TrackInventory.parse(snbt)).getOrThrow();
    }

    private static TrackStore store(Path root) throws IOException {
        TrackStore store = new TrackStore(root);
        store.create(new TrackStore.Track("course", new TrackStore.Cell(0, 60, 0), new TrackStore.Cell(10, 60, 0),
            "minecraft:overworld", "creation", "human"));
        return store;
    }

    private static void codecs(Path root) throws Exception {
        List<ItemStack> items = new ArrayList<>(Collections.nCopies(41, ItemStack.EMPTY));
        // Include all armour slots, main inventory, hotbar and offhand, with complete component data.
        String components = "components:{'minecraft:damage':17,'minecraft:custom_name':{text:'Named tool',italic:false},"
            + "'minecraft:enchantments':{'minecraft:unbreaking':3},'minecraft:custom_data':{byte:1b,short:2s,long:3L,"
            + "float:4.5f,double:6.25d,ints:[I;1,2],bytes:[B;1b,2b],longs:[L;1L,2L],nested:{v:'value'}}}";
        for (int i = 0; i < 4; i++) items.set(i, item("{id:'minecraft:diamond_" + List.of("helmet", "chestplate", "leggings", "boots").get(i) + "',count:1," + components + "}"));
        items.set(4, item("{id:'minecraft:diamond_sword',count:1," + components + "}"));
        items.set(31, item("{id:'minecraft:stick',count:64,components:{'minecraft:custom_name':{text:'Remove me'}}}"));
        items.set(32, item("{id:'minecraft:bow',count:1," + components + "}"));
        items.set(40, item("{id:'minecraft:shield',count:1," + components + "}"));
        original = TrackInventoryCodec.encode(items, 7, registries);
        kit = original.withoutSticks();
        check(original.slot(36).getStringOr("id", "").equals("minecraft:stick") && kit.slot(36).isEmpty(), "Named sticks are removed without changing the original");
        check(!kit.equals(original), "Negative control: a kit retaining a stick differs");
        check(kit.selected == 7 && kit.slot(5).equals(original.slot(5)) && kit.slot(45).equals(original.slot(45)), "Kit retains armour, offhand, data and selection");
        var allSticks = new TrackInventory(Collections.nCopies(41, original.slot(36).toString()), 0).withoutSticks();
        for (int slot = 5; slot <= 45; slot++) check(allSticks.slot(slot).isEmpty(), "Sticks removed from slot " + slot);
        var decoded = TrackInventoryCodec.decode(original, registries);
        check(decoded.get(4).isEnchanted() && decoded.get(4).getDamageValue() == 17, "Enchantment and damage are real decoded item components");
        check(decoded.get(4).getHoverName().getString().equals("Named tool"), "Custom item name decodes");
        check(TrackInventoryCodec.encode(decoded, 7, registries).equals(original), "Real item codec retains NBT types and every component");
        check(TrackInventory.read(JsonParser.parseString(original.json().toString()).getAsJsonObject()).equals(original), "Inventory JSON round trip");
        var store = store(root);
        store.writeKit("course", original);
        check(store.readKit("course").equals(kit), "Kit JSON round trip removes sticks");
        check(store.list().size() == 1, "Kit JSON is excluded from track listing");
        String onDisk = Files.readString(root.resolve("lab/tracks/course.kit.json"));
        check(onDisk.equals(TrackStore.canonical(kit.json()) + "\n"), "Kit file is canonical JSON");
        String independentHash = java.util.HexFormat.of().formatHex(java.security.MessageDigest.getInstance("SHA-256")
            .digest(onDisk.stripTrailing().getBytes(java.nio.charset.StandardCharsets.UTF_8)));
        check(TrackStore.sha256(kit.json()).equals(independentHash), "Kit SHA-256 covers canonical bytes without newline");
        JsonObject reverseKeys = new JsonObject();
        reverseKeys.add("slots", kit.json().get("slots"));
        reverseKeys.addProperty("selected", 7);
        reverseKeys.addProperty("version", 1);
        check(TrackStore.sha256(reverseKeys).equals(independentHash), "Hash is independent of JSON object insertion order");
        for (String mode : List.of("creative", "survival", "adventure", "spectator")) {
            var session = new TrackStore.Session(original, mode);
            store.writeSession("human", session);
            check(store.readSession("human").equals(session), "Complete session round trip for " + mode);
            String saved = Files.readString(root.resolve("lab/tracks/.session-human.json"));
            try { store.writeSession("human", new TrackStore.Session(kit, "survival")); throw new AssertionError("Session overwritten"); }
            catch (IOException expected) { checks++; }
            check(Files.readString(root.resolve("lab/tracks/.session-human.json")).equals(saved), "Existing session cannot be overwritten");
            store.deleteSession("human");
        }
        // Construct actual 26.2 player-save fields to exercise the server acknowledgement decoder.
        CompoundTag server = new CompoundTag();
        ListTag inventory = new ListTag();
        var ops = registries.createSerializationContext(NbtOps.INSTANCE);
        for (int index = 0; index < 36; index++) {
            int menu = index < 9 ? 36 + index : index;
            ItemStack stack = decoded.get(menu - 5);
            if (!stack.isEmpty()) inventory.add(ItemStackWithSlot.CODEC.encodeStart(ops, new ItemStackWithSlot(index, stack)).getOrThrow());
        }
        server.put("Inventory", inventory);
        server.putInt("SelectedItemSlot", 7);
        var equipment = new EntityEquipment();
        for (int i = 0; i < 4; i++) equipment.set(List.of(EquipmentSlot.HEAD, EquipmentSlot.CHEST, EquipmentSlot.LEGS, EquipmentSlot.FEET).get(i), decoded.get(i));
        equipment.set(EquipmentSlot.OFFHAND, decoded.get(40));
        server.put("equipment", EntityEquipment.CODEC.encodeStart(ops, equipment).getOrThrow());
        check(TrackInventoryCodec.server(server, registries).equals(original), "Server Inventory/equipment fields map to all 41 menu slots exactly");
        server.getCompoundOrEmpty("equipment").remove("offhand");
        check(!TrackInventoryCodec.server(server, registries).equals(original), "Negative control: missing server offhand is detected");
        store.writeKit("course", kit.withoutSticks());
        check(store.readKit("course").equals(kit), "Kit can be atomically replaced");
    }

    private static final class Server implements TrackMode.Port {
        final TrackStore store;
        TrackInventory client = original;
        TrackInventory server = original;
        TrackInventory expectedSaved = original;
        String clientMode = "creative";
        String serverMode = "creative";
        boolean alive = true;
        boolean full;
        boolean permitMode = true;
        boolean permitSlots = true;
        boolean permitRefill = true;
        boolean reply = true;
        boolean acknowledge = true;
        boolean deferRefill;
        boolean pendingRefill;
        int applications;
        int confirmations;
        int queries;
        Consumer<TrackMode.Observation> pending;
        Consumer<String> pendingFailure;
        final List<String> messages = new ArrayList<>();
        final List<String> commands = new ArrayList<>();
        Server(TrackStore store) { this.store = store; }
        @Override public TrackInventory inventory() { return client; }
        @Override public String mode() { return clientMode; }
        @Override public boolean alive() { return alive; }
        @Override public void checkEntry() { }
        @Override public void checkRestorable(TrackInventory inventory) { }
        @Override public void say(String message) { messages.add(message); }
        @Override public void command(String command) {
            savedBeforeMutation();
            commands.add(command);
            if (command.startsWith("gamemode ") && permitMode) {
                serverMode = command.substring(9);
                if (acknowledge) clientMode = serverMode;
            } else if (command.startsWith("effect ") && permitRefill) {
                if (deferRefill) pendingRefill = true;
                else full = true;
            }
        }
        void savedBeforeMutation() {
            try { check(store.readSession("human").inventory().equals(expectedSaved), "Original inventory is durable before mutation"); }
            catch (IOException failure) { throw new AssertionError(failure); }
        }
        @Override public void apply(TrackInventory value) {
            savedBeforeMutation();
            check(clientMode.equals("creative") && serverMode.equals("creative"), "Slot packets are sent only after creative acknowledgement");
            applications++;
            if (permitSlots) server = value;
        }
        @Override public void confirm(TrackInventory value) { confirmations++; client = value; }
        @Override public void query(Consumer<TrackMode.Observation> response, Consumer<String> failure) {
            queries++;
            pendingFailure = failure;
            if (reply) response.accept(new TrackMode.Observation(server, serverMode, full));
            else pending = response;
        }
        void answer() { var response = pending; pending = null; response.accept(new TrackMode.Observation(server, serverMode, full)); }
    }

    private static void ready(TrackMode mode, long base) {
        for (long tick = base; tick < base + 10 && !mode.ready(); tick++) {
            mode.serverTick();
            mode.tick(tick);
        }
        check(mode.ready(), "Preparation reaches READY");
    }

    private static void exited(TrackMode mode, Server port, long base) throws IOException {
        for (long tick = base; tick < base + 10 && mode.active(); tick++) mode.tick(tick);
        check(!mode.active() && port.client.equals(original) && port.server.equals(original), "Exit restores complete original inventory including sticks");
        check(port.clientMode.equals("creative") && port.serverMode.equals("creative"), "Exit restores saved mode");
        check(port.store.readSession("human") == null, "Session deleted only after successful server verification");
    }

    private static void transactions(Path root) throws Exception {
        var store = store(root.resolve("lifecycle"));
        var port = new Server(store);
        var mode = new TrackMode(store, "human", port);
        mode.enter("course");
        check(store.readSession("human").inventory().equals(original) && port.applications == 0 && port.commands.isEmpty(), "Entry saves before any change");
        check(port.messages.stream().anyMatch(s -> s.contains("kit saved")), "First entry announces captured kit");
        ready(mode, 1);
        check(port.server.equals(kit) && port.serverMode.equals("survival") && port.full, "Entry prepares survival with kit and full resources");
        check(mode.kitHash().equals(TrackStore.sha256(store.readKit("course").json())), "Run uses stored kit hash");
        check(mode.validateStart(), "Survival kit permits departure");
        try { mode.replaceKit("course", true); throw new AssertionError("Run kit replacement allowed"); }
        catch (IllegalStateException expected) { checks++; }
        try { mode.replaceKit("course", false); throw new AssertionError("Track-mode kit replacement allowed"); }
        catch (IllegalStateException expected) { checks++; }
        check(store.readKit("course").equals(kit), "Refused replacement preserves kit");
        port.server = port.client = original;
        port.full = false;
        mode.rearm();
        ready(mode, 20);
        check(port.applications == 2 && port.server.equals(kit) && port.full, "Re-arm restores kit and full resources again");
        // Freeze the kit for the attempt even if another client replaces its file.
        store.writeKit("course", new TrackInventory(Collections.nCopies(41, "{}"), 0));
        check(mode.kitHash().equals(TrackStore.sha256(kit.json())), "Run hash remains that of the applied kit");
        var detector = new TrackDetector();
        check(detector.sample(true, false, false, 40, 40) == TrackDetector.Event.ARMED, "Ready kit arms normal detector");
        check(detector.sample(false, false, false, 41, 41) == TrackDetector.Event.START, "Departure starts normal detector");
        check(detector.sample(false, true, false, 48, 48) == TrackDetector.Event.STOP, "Finish uses normal detector");
        JsonObject run = new JsonObject();
        run.addProperty("player", "human");
        run.addProperty("gameMode", port.clientMode);
        run.addProperty("kitSha256", mode.kitHash());
        run.addProperty("T", detector.ticks());
        run.addProperty("trackCreated", "creation");
        store.save("course", run);
        mode.exit();
        check(store.history("course").getFirst().get("T").getAsLong() == 7 && port.server.equals(kit), "Finished run is recorded before restoring");
        exited(mode, port, 50);
        check(store.history("course").getFirst().get("gameMode").getAsString().equals("survival"), "Run records survival rather than restored mode");
        try { mode.replaceKit("course", true); throw new AssertionError("Run flag ignored"); }
        catch (IllegalStateException expected) { checks++; }
        mode.replaceKit("course", false);
        check(store.readKit("course").equals(kit), "Idle explicit kit command replaces it from current inventory minus sticks");

        for (String reason : List.of("abandon-death", "abandon-dimension", "abandon-fall", "cancel", "exit")) {
            mode.enter("course");
            ready(mode, 100);
            if (reason.equals("abandon-death")) port.alive = false;
            mode.exit();
            if (!port.alive) {
                int applications = port.applications;
                mode.tick(1000);
                check(mode.active() && mode.state() == TrackMode.State.ALIVE && port.applications == applications, "Death retains session and waits for respawn without packets");
                port.alive = true;
            }
            exited(mode, port, 1001);
            check(store.history("course").size() == 1, reason + " does not write a finished run");
        }
        // Crash at every transaction boundary; a fresh machine must restore the durable original.
        for (String phase : List.of("saved", "creative", "inventory", "survival", "ready", "restore-inventory")) {
            var crashStore = store(root.resolve("crash-" + phase));
            var crashPort = new Server(crashStore);
            crashPort.clientMode = crashPort.serverMode = "adventure";
            var crashed = new TrackMode(crashStore, "human", crashPort);
            crashed.enter("course");
            if (!phase.equals("saved")) {
                crashPort.reply = false;
                crashed.tick(1);
                if (!phase.equals("creative")) {
                    crashPort.answer();
                    if (!phase.equals("inventory")) {
                        crashPort.reply = true;
                        if (!phase.equals("survival")) ready(crashed, 2);
                        if (phase.equals("restore-inventory")) { crashed.exit(); crashPort.reply = false; crashed.tick(10); }
                    }
                }
            }
            // New connection and state; server may still hold an intermediate inventory/mode.
            crashPort.reply = true;
            var recovered = new TrackMode(crashStore, "human", crashPort);
            recovered.recover();
            for (long tick = 20; tick < 30 && recovered.active(); tick++) recovered.tick(tick);
            check(!recovered.active() && crashPort.server.equals(original) && crashPort.serverMode.equals("adventure"), "Crash recovery from " + phase + " restores original mode and inventory");
            check(crashStore.readSession("human") == null && crashPort.messages.stream().anyMatch(s -> s.contains("crash recovery complete")), "Recovery deletes session and announces success");
        }
    }

    private static void failures(Path root) throws Exception {
        for (String fault : List.of("permission", "slots", "query", "refill", "target-mode", "final-refusal", "restore-slots")) {
            var store = store(root.resolve(fault));
            var port = new Server(store);
            port.clientMode = port.serverMode = "adventure";
            var mode = new TrackMode(store, "human", port);
            mode.enter("course");
            switch (fault) {
                case "permission" -> port.permitMode = false;
                case "slots" -> port.permitSlots = false;
                case "query" -> port.reply = false;
                case "refill" -> port.permitRefill = false;
                case "target-mode" -> { port.reply = false; mode.tick(1); port.permitMode = false; port.answer(); }
                case "final-refusal" -> { port.reply = false; mode.tick(1); port.answer(); mode.tick(2); mode.serverTick(); mode.tick(3); port.server = original; port.answer(); }
                case "restore-slots" -> { ready(mode, 1); mode.exit(); port.permitSlots = false; }
            }
            for (long tick = 4; tick < 500 && mode.state() != TrackMode.State.FAILED; tick++) {
                mode.serverTick();
                mode.tick(tick);
            }
            check(mode.state() == TrackMode.State.FAILED && store.readSession("human").inventory().equals(original), fault + " stops and retains original session");
            check(port.messages.stream().anyMatch(s -> s.contains("Saved session retained")), fault + " reports failure in chat");
            if (fault.equals("permission")) check(port.applications == 0, "No creative packets without mode acknowledgement");
            if (fault.equals("slots")) check(port.confirmations == 0 && port.serverMode.equals("creative"), "Refused inventory never switches to survival or updates local inventory");
            int commands = port.commands.size();
            mode.tick(900);
            check(port.commands.size() == commands, "No automatic retries after " + fault);
            port.permitMode = port.permitSlots = port.permitRefill = port.reply = true;
            mode.exit();
            for (long tick = 1000; tick < 1010 && mode.active(); tick++) mode.tick(tick);
            check(!mode.active() && port.server.equals(original) && port.serverMode.equals("adventure"), "Explicit exit recovers after " + fault);
        }
        var store = store(root.resolve("acknowledgements"));
        var port = new Server(store);
        port.clientMode = port.serverMode = "adventure";
        port.acknowledge = false;
        var mode = new TrackMode(store, "human", port);
        mode.enter("course");
        port.reply = false;
        mode.tick(1);
        mode.tick(2);
        check(mode.state() == TrackMode.State.CREATIVE && port.applications == 0, "Server mode alone does not allow slots before client sees creative");
        port.clientMode = "creative";
        mode.tick(3);
        var obsoleteReply = port.pending;
        var obsoleteFailure = port.pendingFailure;
        mode.exit();
        obsoleteReply.accept(new TrackMode.Observation(port.server, port.serverMode, true));
        obsoleteFailure.accept("Obsolete malformed server reply");
        check(mode.state() == TrackMode.State.ALIVE && port.confirmations == 0, "Exit invalidates in-flight kit callback");
        port.reply = port.acknowledge = true;
        for (long tick = 4; tick < 14 && mode.active(); tick++) mode.tick(tick);
        check(!mode.active() && port.server.equals(original) && port.serverMode.equals("adventure"), "Exit during preparation restores original session");
        mode.enter("course");
        ready(mode, 20);
        port.clientMode = "creative";
        check(!mode.validateStart() && mode.state() == TrackMode.State.FAILED, "Departure outside survival is refused");
        mode.exit();
        for (long tick = 40; tick < 50 && mode.active(); tick++) mode.tick(tick);
        mode.enter("course");
        ready(mode, 60);
        port.client = original;
        check(!mode.validateStart() && mode.state() == TrackMode.State.FAILED, "Departure with changed kit is refused");
    }

    private static void serverOrdering(Path root) throws Exception {
        var store = store(root);
        var port = new Server(store);
        port.deferRefill = true;
        var mode = new TrackMode(store, "human", port);
        mode.enter("course");
        mode.tick(1);
        check(port.pendingRefill && !port.full && mode.state() == TrackMode.State.TARGET,
            "Refill commands can be acknowledged before instant effects run on a server tick");
        mode.tick(2);
        mode.tick(3);
        check(mode.state() == TrackMode.State.REFILL && port.queries == 2,
            "Client ticks alone cannot verify the refill or arm the run");
        port.full = true;
        mode.serverTick();
        mode.tick(4);
        check(mode.ready() && port.queries == 3,
            "A later server time packet permits one authoritative refill verification");
        mode.exit();
        exited(mode, port, 5);

        // A failed verification stops the transaction; old callbacks cannot break recovery.
        mode.enter("course");
        port.reply = false;
        mode.tick(20);
        var error = port.pendingFailure;
        error.accept("Server refused player data query");
        check(mode.state() == TrackMode.State.FAILED && store.readSession("human") != null,
            "Malformed/refused verification retains the session and stops");
        mode.exit();
        error.accept("Late verification failure");
        check(mode.state() == TrackMode.State.ALIVE, "Late verification errors cannot invalidate explicit recovery");
        port.reply = true;
        exited(mode, port, 21);
        check(port.commands.stream().filter(c -> c.equals("gamemode creative")).count() == 4,
            "Every entry and exit explicitly requests creative, including an already-creative client");
    }
}
