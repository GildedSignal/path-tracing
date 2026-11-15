package pathtracing;

import java.nio.file.Path;

/** The in-process channel; a load failure is retained for a clear refusal. */
final class NativeChannel {
    private static final String loadFailure;
    static {
        String failure = null;
        try {
            String library = System.getenv("PATHTRACING_NATIVE");
            if (library == null || library.isBlank()) {
                String root = System.getenv("PATHTRACING_ROOT");
                if (root == null || root.isBlank()) throw new IllegalStateException("PATHTRACING_ROOT is not set");
                library = Path.of(root, "engine", "target", "release", "pathtracing_native.dll").toString();
            }
            System.load(Path.of(library).toAbsolutePath().toString());
        } catch (LinkageError | RuntimeException e) {
            failure = e.toString();
        }
        loadFailure = failure;
    }

    static String loadFailure() { return loadFailure; }
    static native long[] clock();
    static native long[] roundTrip(long[] bits, long[] known, long[] provenance);
    static native long[] RequestGoal(long[] bits, long[] known, long[] provenance, long[] metadata, long[] goal, long issueStamp, long version);
    static native long[] PublishSourceDelta(long[] delta);
    static native long[] ObserveEvent(long[] event, long[] bits, long[] known, long[] provenance);
    static native long[] Cancel(long version);
    private NativeChannel() { }
}
