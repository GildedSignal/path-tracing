public final class MiningOracle {
    private record Target(int position, long identity) {
        @Override public String toString() { return position + ":" + identity; }
    }

    private record Sample(Target target, float speed, float hardness, boolean correct) {
        @Override public String toString() {
            return target + ":" + bits(speed) + ":" + bits(hardness) + ":" + (correct ? 1 : 0);
        }
    }

    private record Work(Target target, int start) {
        @Override public String toString() { return target + "@" + start; }
    }

    private static float perTick(float speed, float hardness, boolean correct) {
        if (hardness == -1.0f) return 0.0f;
        int divisor = correct ? 30 : 100;
        return speed / hardness / divisor;
    }

    private static float elapsed(float progress, int now, int start) {
        int delta = now - start;
        return progress * (float)(delta + 1);
    }

    private static boolean atOne(float fraction) { return fraction >= 1.0f; }
    private static boolean atStop(float fraction) { return fraction >= 0.7f; }

    private static float perTick(Sample sample) {
        return perTick(sample.speed, sample.hardness, sample.correct);
    }

    private static String bits(float value) { return Integer.toHexString(Float.floatToIntBits(value)); }
    private static String wire(Object value) { return value == null ? "-" : value.toString(); }

    private final String name;
    private int ticks;
    private Work active;
    private Work delayed;
    private Target lookup;

    private MiningOracle(String name, int ticks) {
        this.name = name;
        this.ticks = ticks;
        row("reset", ticks, null, true, "idle");
    }

    private void row(String action, int arg, Sample sample, boolean accepted, String output) {
        System.out.printf("%s|%s|%d|%s|%d|%d/%s/%s|%s|%s%n",
                name, action, arg, wire(sample), accepted ? 1 : 0,
                ticks, wire(active), wire(delayed), wire(lookup), output);
    }

    private static String evaluated(Target target, float progress, boolean destroy) {
        return (destroy ? "destroy:" : "progress:") + target + ":" + bits(progress);
    }

    private String start(Sample sample) {
        if (active != null) return "refuse:start-active";
        Target target = sample.target;
        if (delayed != null && delayed.target.position == target.position) return "refuse:start-delayed";
        float progress = perTick(sample);
        if (atOne(progress)) return evaluated(target, progress, true);
        active = new Work(target, ticks);
        return evaluated(target, progress, false);
    }

    private String stop(int position, Sample sample) {
        if (active == null) return "refuse:stop-inactive";
        Work original = active;
        if (position != original.target.position) return "refuse:stop-position";
        if (sample == null) return "idle";
        if (!sample.target.equals(original.target)) {
            active = null;
            return "refuse:changed:" + original.target;
        }
        float progress = elapsed(perTick(sample), ticks, original.start);
        if (atStop(progress)) {
            active = null;
            return evaluated(original.target, progress, true);
        }
        if (delayed == null) {
            active = null;
            delayed = original;
        }
        return evaluated(original.target, progress, false);
    }

    private String tick(Sample sample) {
        ticks++;
        if (delayed != null) {
            Target target = lookup = delayed.target;
            if (sample == null) {
                delayed = null;
                return "gone:" + target;
            }
            if (!sample.target.equals(target)) {
                delayed = null;
                return "refuse:changed:" + target;
            }
            float progress = elapsed(perTick(sample), ticks, delayed.start);
            if (atOne(progress)) {
                delayed = null;
                return evaluated(target, progress, true);
            }
            return evaluated(target, progress, false);
        }
        if (active == null) return "idle";
        Target target = lookup = active.target;
        if (sample == null) {
            active = null;
            return "gone:" + target;
        }
        if (!sample.target.equals(target)) {
            active = null;
            return "refuse:changed:" + target;
        }
        return evaluated(target, elapsed(perTick(sample), ticks, active.start), false);
    }

    private void start(Sample sample, boolean accepted) {
        lookup = null;
        row("start", sample.target.position, sample, accepted,
                accepted ? start(sample) : "refuse:rejected");
    }

    private void stop(int position, Sample sample, boolean accepted) {
        lookup = null;
        row("stop", position, sample, accepted, accepted ? stop(position, sample) : "refuse:rejected");
    }

    private void serverTick(Sample sample) {
        lookup = null;
        row("tick", 0, sample, true, tick(sample));
    }

    private void seek(int tick) {
        ticks = tick;
        lookup = null;
        row("seek", tick, null, true, "idle");
    }

    private static final Target A = new Target(4, 71);
    private static final Target B = new Target(-7, 99);
    private static final Target REPLACEMENT = new Target(4, 72);

    private static Sample speed(Target target, float speed) { return new Sample(target, speed, 1, true); }

    private static float adjacent(float center, int delta) {
        return Float.intBitsToFloat(Float.floatToRawIntBits(center) + delta);
    }

    private static void requireBoundary(float actual, float center, int delta) {
        if (Float.floatToRawIntBits(actual) != Float.floatToRawIntBits(adjacent(center, delta))) {
            throw new AssertionError("fixture missed boundary: " + bits(actual) + " vs " + bits(adjacent(center, delta)));
        }
    }

    private static void boundaries() {
        for (int delta = -1; delta <= 1; delta++) {
            Sample startSample = speed(A, adjacent(30f, delta));
            requireBoundary(perTick(startSample), 1.0f, delta);
            new MiningOracle("start-boundary-" + delta, 8).start(startSample, true);

            Sample stopSample = speed(A, adjacent(10.5f, delta));
            requireBoundary(elapsed(perTick(stopSample), 9, 8), 0.7f, delta);
            MiningOracle stop = new MiningOracle("stop-boundary-" + delta, 8);
            stop.start(speed(A, 1), true);
            stop.serverTick(speed(A, 30));
            stop.stop(A.position, stopSample, true);

            Sample delayedSample = speed(A, adjacent(7.5f, delta));
            requireBoundary(elapsed(perTick(delayedSample), 11, 8), 1.0f, delta);
            MiningOracle delayed = new MiningOracle("delayed-boundary-" + delta, 8);
            delayed.start(speed(A, 2), true);
            delayed.serverTick(speed(A, 1));
            delayed.stop(A.position, speed(A, 3), true);
            delayed.serverTick(speed(A, 6));
            delayed.serverTick(delayedSample);
        }
    }

    private static void feederCases() {
        Sample[] inputs = {
            new Sample(A, 75, 2.5f, true), new Sample(A, 75, 2.5f, false),
            new Sample(A, Float.NaN, -1, true), new Sample(A, 1, 0, true),
            new Sample(A, 0, 0, true), new Sample(A, -0f, 1, false),
            new Sample(A, Float.intBitsToFloat(63), 1, true),
            new Sample(A, 1, Float.NaN, true), new Sample(A, 1, -0f, true),
            new Sample(A, Float.MAX_VALUE, 0.25f, false),
            new Sample(A, Float.POSITIVE_INFINITY, Float.POSITIVE_INFINITY, false),
        };
        for (int i = 0; i < inputs.length; i++) new MiningOracle("feeder-" + i, 0).start(inputs[i], true);

        MiningOracle nan = new MiningOracle("nan-while-running", 40);
        nan.start(speed(A, 1), true);
        nan.stop(A.position, speed(A, Float.NaN), true);
        nan.serverTick(speed(A, Float.NaN));
        nan.serverTick(new Sample(A, 75, 1.5f, false));
    }

    private static void clocks() {
        MiningOracle wrap = new MiningOracle("clock-wrap", Integer.MAX_VALUE - 1);
        wrap.start(speed(A, 4), true);
        wrap.serverTick(speed(A, 5));
        wrap.stop(A.position, speed(A, 1), true);
        wrap.serverTick(speed(A, 6));
        wrap.serverTick(speed(A, 7.5f));

        MiningOracle elapsed = new MiningOracle("elapsed-wrap-and-rounding", 0);
        elapsed.start(speed(A, 1), true);
        elapsed.seek((1 << 24) - 2);
        for (int i = 0; i < 4; i++) elapsed.serverTick(speed(A, 30));
        elapsed.seek(Integer.MAX_VALUE - 1);
        elapsed.serverTick(speed(A, 30));
        elapsed.stop(A.position, speed(A, 30), true);
        elapsed.serverTick(speed(A, 30));
    }

    private static void slots() {
        MiningOracle ordered = new MiningOracle("older-delayed-first", 120);
        ordered.start(speed(A, 6), true);
        ordered.stop(A.position, speed(A, 6), true);
        ordered.start(speed(B, 2), true);
        ordered.serverTick(speed(A, 9));
        ordered.stop(B.position, speed(B, 1), true);
        ordered.serverTick(speed(A, 10));
        ordered.serverTick(speed(B, 30));
        ordered.stop(B.position, speed(B, 10), true);

        MiningOracle busy = new MiningOracle("stop-active-with-delayed", 5);
        busy.start(speed(A, 1), true);
        busy.stop(A.position, speed(A, 1), true);
        busy.start(speed(B, 1), true);
        busy.serverTick(speed(A, 1));
        busy.stop(B.position, speed(B, 10.5f), true);
        busy.serverTick(null);

        MiningOracle gone = new MiningOracle("delayed-air-consumes-tick", 15);
        gone.start(speed(A, 1), true);
        gone.stop(A.position, speed(A, 1), true);
        gone.start(speed(B, 2), true);
        gone.serverTick(null);
        gone.serverTick(speed(B, 3));
    }

    private static void adapter() {
        MiningOracle inputs = new MiningOracle("input-contract", 27);
        inputs.start(speed(A, 30), false);
        inputs.serverTick(null);
        inputs.stop(A.position, speed(A, 30), true);
        inputs.start(speed(A, 1), true);
        inputs.start(speed(A, 30), true);
        inputs.start(speed(B, 30), true);
        inputs.stop(B.position, speed(B, 30), true);
        inputs.stop(A.position, speed(A, 30), false);
        inputs.stop(A.position, null, true);
        inputs.serverTick(null);
        inputs.start(speed(A, 1), true);
        inputs.stop(A.position, speed(REPLACEMENT, 30), true);
        inputs.start(speed(A, 1), true);
        inputs.stop(A.position, speed(A, 1), true);
        inputs.start(speed(REPLACEMENT, 30), true);
        inputs.stop(A.position, speed(A, 30), true);
        inputs.serverTick(speed(REPLACEMENT, 30));

        MiningOracle active = new MiningOracle("active-replacement", 0);
        active.start(speed(A, 1), true);
        active.serverTick(speed(REPLACEMENT, 30));
    }

    public static void main(String[] args) {
        boundaries();
        feederCases();
        clocks();
        slots();
        adapter();
    }
}
