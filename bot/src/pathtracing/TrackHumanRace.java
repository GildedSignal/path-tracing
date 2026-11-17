package pathtracing;

import java.io.IOException;
import java.util.List;
import java.util.function.Consumer;

/** The human race lifecycle, shared by the client and the headless gate. */
final class TrackHumanRace {
    private final TrackMode mode;
    private final TrackDetector detector;
    private final Consumer<String> overlay;
    private final Consumer<String> chat;
    private TrackStore.Track selected;
    private List<TrackStore.Track> suppressed = List.of();
    private boolean onA;
    private String refusal;
    private boolean initialized;
    private boolean entering;

    TrackHumanRace(TrackMode mode, TrackDetector detector, Consumer<String> overlay, Consumer<String> chat) throws IOException {
        this.mode = mode;
        this.detector = detector;
        this.overlay = overlay;
        this.chat = chat;
    }

    TrackStore.Track selected() { return selected; }

    TrackDetector.Event tick(List<TrackStore.Track> contacts, boolean enabled, boolean touchesB,
                             boolean abandoned, long tick, long gameTime) throws IOException {
        mode.tick(tick);
        if (!initialized) {
            initialized = true;
            if (!mode.active()) mode.recover();
        }
        suppressed = suppressed.stream().filter(contacts::contains).toList();
        if (selected == null && enabled) {
            selected = contacts.stream().filter(t -> !suppressed.contains(t)).findFirst().orElse(null);
            if (selected != null) { refusal = null; onA = true; }
        }
        boolean here = selected != null && contacts.contains(selected);
        if (onA && !here && (!mode.ready() || !detector.active() || !mode.validateStart())) {
            String reason = refusal == null ? mode.reason() : refusal;
            chat.accept("This run is not timed: " + reason);
            cancel(List.of(), true);
            return TrackDetector.Event.DISCARD;
        }
        onA = here;
        if (selected != null && abandoned) {
            cancel(contacts, true);
            return TrackDetector.Event.DISCARD;
        }
        if (mode.state() == TrackMode.State.FAILED && !mode.restoring() && refusal == null) {
            refusal = mode.reason();
            detector.reset();
            mode.exit(); // Roll back a refused kit once; a new A contact starts a new attempt.
        }
        if (selected != null && entering && !mode.active() && refusal == null) refusal = mode.reason();
        if (selected != null && !mode.active() && refusal == null) {
            entering = true;
            mode.enter(selected.name());
            if (!mode.active()) refusal = mode.reason();
        }
        var event = TrackDetector.Event.NONE;
        if (selected != null && mode.ready() && refusal == null) {
            event = detector.sample(here, touchesB, false, tick, gameTime);
            if (event == TrackDetector.Event.REARM) {
                detector.reset();
                mode.rearm();
            }
        }
        if (here) {
            String reason = refusal != null ? refusal : mode.state() == TrackMode.State.FAILED ? mode.reason() : null;
            overlay.accept(reason != null ? "Not armed: " + reason : mode.ready() && detector.active() ? "Armed: go" : "Arming\u2026");
        } else if (selected == null && !contacts.isEmpty()) {
            overlay.accept("Not armed: track mode exited; step off A and return");
        }
        return event;
    }

    void cancel(List<TrackStore.Track> contacts, boolean restore) throws IOException {
        suppressed = List.copyOf(contacts);
        selected = null;
        onA = false;
        refusal = null;
        entering = false;
        detector.reset();
        if (restore) mode.exit();
    }
}
