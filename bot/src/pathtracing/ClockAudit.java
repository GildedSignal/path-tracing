package pathtracing;

import org.slf4j.LoggerFactory;

/** Lab clock options and main-thread accounting for the sustained replay. */
public final class ClockAudit {
    private static final boolean BOT = "bot".equals(System.getProperty("pathtracing.role"));
    private static final boolean FULL_TICK_DEMAND = BOT && Boolean.getBoolean("pathtracing.fullTickDemand")
        && Boolean.getBoolean("pathtracing.followTickRate");
    private static final boolean ENABLED = BOT
        && Boolean.getBoolean("pathtracing.clockAudit");
    private static int demand, completed;
    private static long scheduled, executed, dropped, frames;
    private static long startMillis, startNanos, elapsedNanos;
    private static boolean sampling, finished;
    private static String replay;
    private static int rows;
    private static boolean complete;

    private ClockAudit() {}

    public static boolean fullTickDemand() {
        return FULL_TICK_DEMAND;
    }

    public static void scheduled(int ticks) {
        if (ENABLED) demand = ticks;
    }

    public static void executed() {
        if (ENABLED) completed++;
    }

    static void begin(String name) {
        if (!ENABLED) return;
        replay = name;
        scheduled = executed = dropped = frames = 0;
        startMillis = System.currentTimeMillis();
        startNanos = System.nanoTime();
        sampling = true;
    }

    static void finish(int tickRows, boolean clean) {
        if (!sampling) return;
        elapsedNanos = System.nanoTime() - startNanos;
        rows = tickRows;
        complete = clean;
        finished = true;
    }

    public static void frameFinished() {
        if (!ENABLED) return;
        if (sampling) {
            // Include the whole first and last frames so demand and execution share boundaries.
            frames++;
            scheduled += demand;
            executed += completed;
            dropped += demand - completed;
            if (finished) {
                LoggerFactory.getLogger("pathtracing").info(
                    "Clock audit {} rows={} complete={} activeNanos={} frames={} scheduled={} executed={} dropped={} startMillis={}",
                    replay, rows, complete, elapsedNanos, frames, scheduled, executed, dropped, startMillis);
                sampling = finished = false;
            }
        }
        demand = completed = 0;
    }
}
