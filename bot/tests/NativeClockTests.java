package pathtracing;

public final class NativeClockTests {
    public static void main(String[] args) {
        if (NativeChannel.loadFailure() != null) throw new AssertionError(NativeChannel.loadFailure());
        long[] first = NativeChannel.clock();
        long[] second = NativeChannel.clock();
        if (first == null || second == null || first.length != 2 || second.length != 2 ||
            first[1] <= 0 || first[1] != second[1] || second[0] < first[0]) {
            throw new AssertionError("invalid QPC samples");
        }
        System.out.println("QPC samples monotonic; frequency=" + first[1]);
    }
}
