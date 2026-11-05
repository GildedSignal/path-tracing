package pathtracing;

import java.util.Map;
import java.util.TreeMap;

/** Count a successful right-click and its later completion as one use. */
final class TrackUsage {
    private final Map<String, Integer> counts = new TreeMap<>();
    private Object pendingUse;

    void add(String item, int count) {
        if (count > 0) counts.merge(item, count, Math::addExact);
    }

    void rightClick(String item, Object ongoingUse) {
        add(item, 1);
        pendingUse = ongoingUse;
    }

    void complete(String item, Object stack) {
        if (pendingUse != stack) add(item, 1);
        pendingUse = null;
    }

    Map<String, Integer> counts() { return Map.copyOf(counts); }

    void clear() {
        counts.clear();
        pendingUse = null;
    }
}
