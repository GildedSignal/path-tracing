package pathtracing;

/** One sample per client tick. Geometry and clocks are supplied by the caller. */
final class TrackDetector {
    enum Event { NONE, ARMED, START, REARM, STOP, DISCARD }

    private boolean armed;
    private boolean running;
    private long startTick;
    private long startGameTime;
    private long ticks;
    private long gameTicks;

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
            ticks = 0;
            gameTicks = 0;
            return wasRunning ? Event.REARM : wasArmed ? Event.NONE : Event.ARMED;
        }
        Event event = Event.NONE;
        if (armed) {
            armed = false;
            running = true;
            startTick = tick;
            startGameTime = gameTime;
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
    boolean running() { return running; }
    long ticks() { return ticks; }
    long gameTicks() { return gameTicks; }

    void reset() {
        armed = false;
        running = false;
        ticks = 0;
        gameTicks = 0;
    }
}
