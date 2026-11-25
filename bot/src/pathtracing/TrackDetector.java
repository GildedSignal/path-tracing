package pathtracing;

/** One sample per client tick. Geometry and clocks are supplied by the caller. */
final class TrackDetector {
    private static final String[] FACES = {"west", "east", "down", "up", "north", "south"};

    static boolean touchesB(net.minecraft.world.phys.AABB previous, net.minecraft.world.phys.AABB current, TrackStore.Cell b, String face) {
        var box = previous == null ? current : previous.minmax(current);
        double[] lo = {box.minX, box.minY, box.minZ}, hi = {box.maxX, box.maxY, box.maxZ};
        double[] cell = {b.x(), b.y(), b.z()};
        for (int i = 0; i < 6; i++) {
            if (face != null && !face.equals(FACES[i])) continue;
            int axis = i / 2;
            double plane = cell[axis] + i % 2;
            if (lo[axis] > plane || hi[axis] < plane) continue;
            boolean overlap = true;
            for (int t = 0; t < 3; t++) if (t != axis && !(lo[t] < cell[t] + 1 && hi[t] > cell[t])) overlap = false;
            if (overlap) return true;
        }
        return false;
    }

    enum Event { NONE, ARMED, START, REARM, STOP, DISCARD }

    private boolean armed;
    private boolean running;
    private long startTick;
    private long startGameTime;
    private long ticks;
    private long gameTicks;
    private boolean rearmRequested;

    Event sample(boolean onA, boolean touchesB, boolean abandoned, long tick, long gameTime) {
        if (abandoned) {
            boolean active = armed || running;
            reset();
            return active ? Event.DISCARD : Event.NONE;
        }
        // Returning to A takes precedence, including overlapping endpoint cells.
        if (onA) {
            boolean wasRunning = running;
            boolean wasArmed = armed;
            running = false;
            armed = true;
            startTick = tick;
            startGameTime = gameTime;
            ticks = 0;
            gameTicks = 0;
            boolean refill = rearmRequested;
            rearmRequested = false;
            return wasRunning || refill ? Event.REARM : wasArmed ? Event.NONE : Event.ARMED;
        }
        Event event = Event.NONE;
        if (armed) {
            armed = false;
            running = true;
            event = Event.START;
        }
        if (running) {
            ticks = tick - startTick;
            gameTicks = gameTime - startGameTime;
            if (touchesB) {
                running = false;
                return Event.STOP;
            }
        }
        return event;
    }

    boolean active() { return armed || running; }
    void requestRearm() { rearmRequested = true; }
    boolean running() { return running; }
    long ticks() { return ticks; }
    long gameTicks() { return gameTicks; }

    void reset() {
        armed = false;
        running = false;
        ticks = 0;
        gameTicks = 0;
        rearmRequested = false;
    }
}
