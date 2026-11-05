package pathtracing;

import java.io.IOException;
import java.util.function.Consumer;

/** Human-only transaction. The session is durable before the first server mutation. */
final class TrackMode {
    enum State { IDLE, ALIVE, CREATIVE, INVENTORY, TARGET, VERIFY, READY, FAILED }
    record Observation(TrackInventory inventory, String mode, boolean full) { }
    interface Port {
        TrackInventory inventory();
        String mode();
        boolean alive();
        boolean full();
        void command(String command);
        void apply(TrackInventory inventory);
        void confirm(TrackInventory inventory);
        void query(Consumer<Observation> response);
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

    void enter(String name) throws IOException {
        if (active()) throw new IllegalStateException("Already in track mode.");
        TrackInventory before = port.inventory();
        kit = store.readKit(name);
        if (kit == null) {
            kit = before.withoutSticks();
            store.writeKit(name, kit);
            port.say("Track " + name + " kit saved from your inventory (sticks removed).");
        }
        TrackStore.Session saved = new TrackStore.Session(before, port.mode());
        store.writeSession(player, saved);
        session = saved;
        prepare(false);
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

    void exit() throws IOException {
        if (restoring && state != State.FAILED) return;
        if (session == null) session = store.readSession(player);
        if (session != null) prepare(true);
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
                if (!port.mode().equals("creative")) port.command("gamemode creative");
            }
            if (now >= deadline) { fail("Server did not confirm " + state + " within 200 client ticks (permission or packet refusal)."); return; }
            if (state == State.CREATIVE && port.mode().equals("creative")) {
                state = State.INVENTORY;
                port.apply(target);
                int request = generation;
                port.query(observed -> acceptInventory(request, observed));
            } else if (state == State.TARGET && port.mode().equals(targetMode) && (restoring || port.full())) {
                state = State.VERIFY;
                int request = generation;
                port.query(observed -> complete(request, observed));
            }
        } catch (RuntimeException failure) { fail(failure.getMessage()); }
    }

    private void acceptInventory(int request, Observation observed) {
        if (request != generation || state != State.INVENTORY) return;
        try {
            if (!observed.mode().equals("creative") || !target.equals(observed.inventory())) {
                fail("Server refused the exact inventory while in creative.");
                return;
            }
            port.confirm(target);
            if (!restoring) {
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
            if (!targetMode.equals(observed.mode()) || !target.equals(observed.inventory()) || (!restoring && !observed.full())) {
                fail("Server did not confirm the inventory, game mode and full health/food/saturation.");
                return;
            }
            if (restoring) {
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
        state = State.FAILED;
        port.say("Track mode stopped: " + reason + " Saved session retained; use /track exit or rejoin to recover.");
    }
}
