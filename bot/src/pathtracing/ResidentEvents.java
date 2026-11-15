package pathtracing;
/** Packet hooks preserve observed bits and times; shared comparison is slice F. */
public final class ResidentEvents {
    public static void observe(String kind) { ResidentRunner.event(kind); }
    private ResidentEvents() { }
}
