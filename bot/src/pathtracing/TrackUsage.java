package pathtracing;

import java.util.Map;
import java.util.TreeMap;

/** Required toolkit: reusable items once plus replacements; stackables by units. */
final class TrackUsage {
    private final Map<String, Integer> counts = new TreeMap<>();
    private final Map<String, Integer> broken = new TreeMap<>();

    void use(String item, int maxStack, int units) {
        if (units <= 0) return;
        if (maxStack == 1) counts.putIfAbsent(item, Math.addExact(1, broken.getOrDefault(item, 0)));
        else counts.merge(item, units, Math::addExact);
    }

    void broken(String item) {
        broken.merge(item, 1, Math::addExact);
        counts.put(item, Math.addExact(1, broken.get(item)));
    }

    Map<String, Integer> counts() { return Map.copyOf(counts); }

    void clear() {
        counts.clear();
        broken.clear();
    }
}
