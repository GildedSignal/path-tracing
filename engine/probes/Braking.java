public class Braking {
    // LivingEntity.computeModifiedFriction.
    static float modifiedFriction(float friction) {
        return Math.max(0.0f, Math.min(1.0f, 1.0f - (1.0f - friction) * 1.0f));
    }

    // LivingEntity.getFrictionInfluencedSpeed.
    static float acceleration(float friction, boolean sprint) {
        float speed = sprint ? 0.13f : 0.1f;
        return friction > 0.6 ? speed * (0.21600002f / (friction * friction * friction)) : speed;
    }

    // LocalPlayer.modifyInput, Entity.getInputVector, LivingEntity.travelInAir.
    static double advance(double velocity, int key, float friction, boolean sprint) {
        float drag = friction * modifiedFriction(0.91f);
        return (velocity + (double)(0.98f * key) * (double)acceleration(friction, sprint)) * (double)drag;
    }

    // LivingEntity.aiStep.
    static boolean threshold(double velocity) {
        return velocity * velocity < 9.0E-6;
    }

    // LivingEntity.aiStep, LocalPlayer.shouldStopRunSprinting, LivingEntity.travelInAir.
    public static void main(String[] args) {
        System.out.println("terrain,gait,reverse_ticks,first_threshold_tick,rest_threshold_tick,first_negative_tick");
        for (float rawFriction : new float[] {0.6f, 0.98f}) {
            float friction = modifiedFriction(rawFriction);
            String terrain = rawFriction == 0.6f ? "stone" : "ice";
            for (boolean sprint : new boolean[] {false, true}) {
                double steady = 0;
                for (int tick = 0; tick < 2000; tick++) steady = advance(steady, 1, friction, sprint);
                int bestTicks = Integer.MAX_VALUE, bestK = -1;
                for (int k = 0; k <= 6; k++) {
                    double velocity = steady;
                    int firstZero = -1, restZero = -1, firstReverse = -1;
                    for (int tick = 1; tick <= 400; tick++) {
                        if (threshold(velocity)) {
                            if (firstZero < 0) firstZero = tick;
                            if (tick > k && restZero < 0) restZero = tick;
                            velocity = 0;
                        }
                        velocity = advance(velocity, tick <= k ? -1 : 0, friction, false);
                        if (velocity < 0 && firstReverse < 0) firstReverse = tick;
                    }
                    if (restZero < 0) throw new AssertionError("no rest within 400 ticks");
                    if (restZero < bestTicks) { bestTicks = restZero; bestK = k; }
                    System.out.printf("%s,%s,%d,%d,%d,%d%n", terrain, sprint ? "sprint" : "walk",
                        k, firstZero, restZero, firstReverse);
                }
                System.err.printf("fastest %s %s: reverse %d tick(s), then release; rest on tick %d%n",
                    terrain, sprint ? "sprint" : "walk", bestK, bestTicks);
            }
        }
    }
}
