package pathtracing;

import java.io.IOException;
import java.util.function.Consumer;

/** Human-only transaction. The session is durable before the first server mutation. */
final class TrackMode {
    enum State { IDLE, CAPTURE, ALIVE, CREATIVE, INVENTORY, TARGET, REFILL, VERIFY, READY, FAILED }
    record Observation(TrackInventory inventory, String mode, boolean full) { }
    interface Port {
        TrackInventory inventory();
        String mode();
        boolean alive();
        void checkEntry();
        void checkRestorable(TrackInventory inventory);
        void command(String command);
        void apply(TrackInventory inventory);
        void confirm(TrackInventory inventory);
        void query(Consumer<Observation> response, Consumer<String> failure);
        void say(String text);
    }

    private final TrackStore store;
    private final Port port;
    private final String player;
    private TrackStore.Session session;
    private TrackInventory kit;
    private TrackInventory target;
    private String targetMode;
    private boolean restoring;
    private boolean recovered;
    private int generation;
    private long deadline;
    private long now;
    private long serverUpdates;
    private long refillAfter;
    private State state = State.IDLE;

    TrackMode(TrackStore store, String player, Port port) {
        this.store = store;
        this.player = player;
        this.port = port;
    }

    State state() { return state; }
    boolean active() { return state != State.IDLE; }
    boolean ready() { return state == State.READY; }
    boolean restoring() { return restoring; }
    void serverTick() { serverUpdates++; }
    String kitHash() { return TrackStore.sha256(kit.json()); }

    boolean validateStart() {
        if (ready() && port.mode().equals("survival") && kit.sameItems(port.inventory())) return true;
        fail("A run must leave A in survival with the kit.");
        return false;
    }

    void replaceKit(String name, boolean running) throws IOException {
        if (active() || running) throw new IllegalStateException("Cannot replace a kit during a run or track mode. Use /track exit first.");
        store.read(name);
        store.writeKit(name, port.inventory().withoutSticks());
        port.say("Track " + name + " kit replaced (sticks removed).");
    }

    void enter(String name) {
        if (active()) throw new IllegalStateException("Already in track mode.");
        state = State.CAPTURE;
        deadline = now + 200;
        int request = ++generation;
        try {
            port.checkEntry();
            port.query(observed -> capture(request, name, observed), reason -> queryFailed(request, reason));
        } catch (RuntimeException failure) { fail(failure.getMessage()); }
    }

    private void capture(int request, String name, Observation observed) {
        if (request != generation || state != State.CAPTURE) return;
        try {
            port.checkEntry();
            TrackInventory before = observed.inventory();
            String difference = port.inventory().difference(before);
            if (difference != null) {
                fail("Client and server inventory differ. " + difference);
                return;
            }
            if (!port.mode().equals(observed.mode())) {
                fail("Client and server game modes differ.");
                return;
            }
            port.checkRestorable(before);
            kit = store.readKit(name);
            if (kit == null) {
                kit = before.withoutSticks();
                store.writeKit(name, kit);
                port.say("Track " + name + " kit saved from your inventory (sticks removed).");
            } else port.checkRestorable(kit);
            TrackStore.Session saved = new TrackStore.Session(before, observed.mode());
            store.writeSession(player, saved);
            session = saved;
            prepare(false);
        } catch (IOException | RuntimeException failure) {
            fail(failure.getMessage());
        }
    }

    void rearm() {
        if (!ready()) throw new IllegalStateException("Track kit is not ready.");
        prepare(false);
    }

    void recover() throws IOException {
        session = store.readSession(player);
        if (session == null) return;
        recovered = true;
        port.say("Track session found; recovering your saved inventory and game mode.");
        prepare(true);
    }

    boolean exit() throws IOException {
        if (restoring && state != State.FAILED) return true;
        if (state == State.CAPTURE) {
            generation++;
            state = State.IDLE;
        }
        if (session == null) session = store.readSession(player);
        if (session != null) prepare(true);
        return session != null;
    }

    private void prepare(boolean restore) {
        generation++;
        restoring = restore;
        target = restore ? session.inventory() : kit;
        targetMode = restore ? session.gameMode() : "survival";
        state = State.ALIVE;
        deadline = now + 200;
        if (!port.alive()) port.say("Track restore waits for respawn; saved session retained.");
    }

    void tick(long tick) {
        now = tick;
        if (state == State.IDLE || state == State.FAILED || state == State.READY) return;
        try {
            if (state == State.ALIVE) {
                if (!port.alive()) return;
                state = State.CREATIVE;
                deadline = now + 200;
                // Always order a server command before slots, including a stale creative client view.
                port.command("gamemode creative");
            }
            if (now >= deadline) { fail("Server did not confirm " + state + " within 200 client ticks (permission or packet refusal)."); return; }
            if (state == State.CREATIVE && port.mode().equals("creative")) {
                state = State.INVENTORY;
                port.apply(target);
                int request = generation;
                port.query(observed -> acceptInventory(request, observed), reason -> queryFailed(request, reason));
            } else if (state == State.TARGET && port.mode().equals(targetMode)) {
                if (!restoring) {
                    // Instant effects run on the server's player tick. A later time packet is
                    // a server-tick barrier; client saturation is not reliably synchronised.
                    refillAfter = serverUpdates;
                    state = State.REFILL;
                } else verify();
            } else if (state == State.REFILL && serverUpdates > refillAfter) {
                verify();
            }
        } catch (RuntimeException failure) { fail(failure.getMessage()); }
    }

    private void verify() {
        state = State.VERIFY;
        int request = generation;
        port.query(observed -> complete(request, observed), reason -> queryFailed(request, reason));
    }

    private void queryFailed(int request, String reason) {
        if (request == generation && (state == State.CAPTURE || state == State.INVENTORY || state == State.VERIFY)) fail(reason);
    }

    private void acceptInventory(int request, Observation observed) {
        if (request != generation || state != State.INVENTORY) return;
        try {
            String difference = target.difference(observed.inventory());
            if (difference != null) {
                fail("Server inventory differs after applying slots. " + difference);
                return;
            }
            if (!observed.mode().equals("creative")) {
                fail("Server did not confirm creative mode.");
                return;
            }
            if (!restoring) {
                port.confirm(target);
                port.command("effect give @s minecraft:instant_health 1 10 true");
                port.command("effect give @s minecraft:saturation 1 20 true");
            }
            state = State.TARGET;
            deadline = now + 200;
            if (!port.mode().equals(targetMode)) port.command("gamemode " + targetMode);
        } catch (RuntimeException failure) { fail(failure.getMessage()); }
    }

    private void complete(int request, Observation observed) {
        if (request != generation || state != State.VERIFY) return;
        try {
            String difference = target.difference(observed.inventory());
            if (difference != null) {
                fail("Server inventory differs after switching game mode. " + difference);
                return;
            }
            if (!targetMode.equals(observed.mode()) || (!restoring && !observed.full())) {
                fail("Server did not confirm the target game mode or the kit's refill.");
                return;
            }
            if (restoring) {
                // The creative write marks the remote slot, so the server need not echo it.
                port.confirm(observed.inventory());
                store.deleteSession(player);
                session = null;
                state = State.IDLE;
                restoring = false;
                port.say(recovered ? "Track crash recovery complete: saved inventory and game mode restored." : "Track mode exited: saved inventory and game mode restored.");
                recovered = false;
            } else state = State.READY;
        } catch (IOException | RuntimeException failure) { fail(failure.getMessage()); }
    }

    void fail(String reason) {
        generation++;
        if (state == State.CAPTURE) {
            state = State.IDLE;
            port.say("Track mode could not start: " + reason);
            return;
        }
        state = State.FAILED;
        port.say("Track mode stopped: " + reason + " Saved session retained. Use /track restore to retry.");
    }
}
