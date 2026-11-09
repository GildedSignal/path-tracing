package pathtracing;

/** The verified ceiling applies when the clock actually follows the server. */
final class ClockQualification {
    static boolean exceedsCeiling(boolean following, boolean normal, float tickrate) {
        return following && normal && tickrate > 2000.0F;
    }
}
