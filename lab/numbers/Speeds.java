// The speed limits of the light cone, from the 26.2 movement law.
// Constants were read from the 26.2 client jar with javap: LivingEntity.aiStep/travelInAir/
// getFrictionInfluencedSpeed/jumpFromGround, Player.getFlyingSpeed, Attributes, Blocks, FoodConstants,
// ThrowableProjectile, EnderpearlItem. Flat ground, forward input, yaw 0, perfect jump taps.
//   java lab/numbers/Speeds.java   (see docs/engine.md, "Facts verified so far")
public class Speeds {
  static final double GRAVITY = 0.08, JUMP = 0.41999998688697815;      // attribute defaults (0.42f widened)
  static final float WALK = 0.1f, SPRINT = 0.13f;                       // movement_speed 0.1, sprint x1.3 total
  static final float NORMAL = 0.6f, ICE = 0.98f, BLUE_ICE = 0.989f;     // Block.getFriction

  static double y, vy, z, vz; static boolean onGround;

  // One tick: aiStep thresholds, jumpFromGround, then travelInAir (accelerate, move, drag).
  static void tick(float friction, boolean sprint, boolean jump, double ceiling) {
    if (vz * vz < 9.0E-6) vz = 0;
    if (Math.abs(vy) < 0.003) vy = 0;
    if (jump && onGround) { vy = Math.max(JUMP, vy); if (sprint) vz += 0.2; }
    float f = onGround ? friction : 1.0f;
    float speed = sprint ? SPRINT : WALK;
    float accel = onGround ? (f > 0.6 ? speed * (0.21600002f / (f * f * f)) : speed) : (sprint ? 0.025999999f : 0.02f);
    vz += 0.98 * accel;
    z += vz;
    double ny = y + vy;
    onGround = false;
    if (ny <= 0) { ny = 0; vy = 0; onGround = true; }
    else if (ny > ceiling - 1.8) { ny = ceiling - 1.8; vy = 0; }
    y = ny;
    vz *= f * 0.91f;
    vy = (vy - GRAVITY) * 0.98f;
  }

  static double speed(float friction, boolean sprint, boolean jump, double ceiling) {
    y = 0; vy = 0; z = 0; vz = 0; onGround = true;
    for (int t = 0; t < 2000; t++) tick(friction, sprint, jump, ceiling);   // reach steady state
    double z0 = z;
    for (int t = 0; t < 2400; t++) tick(friction, sprint, jump, ceiling);   // 2400 = lcm of the jump cycles
    return (z - z0) / 2400 * 20;
  }

  public static void main(String[] a) {
    double open = 1e9, low = 2.0;   // no ceiling; a ceiling 2 blocks above the floor (headhitter)
    System.out.printf("Horizontal (m/s, steady state)%n");
    System.out.printf("  walk                          %6.3f%n", speed(NORMAL, false, false, open));
    System.out.printf("  sprint                        %6.3f%n", speed(NORMAL, true, false, open));
    System.out.printf("  sprint-jump                   %6.3f%n", speed(NORMAL, true, true, open));
    System.out.printf("  sprint-jump, 2-block ceiling  %6.3f%n", speed(NORMAL, true, true, low));
    System.out.printf("  sprint on ice                 %6.3f%n", speed(ICE, true, false, open));
    System.out.printf("  sprint-jump on ice            %6.3f%n", speed(ICE, true, true, open));
    System.out.printf("  same, 2-block ceiling         %6.3f%n", speed(ICE, true, true, low));
    System.out.printf("  same on blue ice              %6.3f%n", speed(BLUE_ICE, true, true, low));

    System.out.printf("%nVertical%n");
    System.out.printf("  terminal fall speed           %6.1f m/s%n", -GRAVITY * 0.98f / (1 - 0.98f) * 20);
    System.out.printf("  ladder climb                  %6.3f m/s%n", (0.2 - GRAVITY) * 0.98f * 20);
    for (int h : new int[] {4, 10, 23, 50, 100, 300}) {
      double yy = h, v = 0; int t = 0;
      while (yy > 0) { if (Math.abs(v) < 0.003) v = 0; yy += v; v = (v - GRAVITY) * 0.98f; t++; }
      System.out.printf("  fall %3d blocks: %3d ticks (%5.2f s), %3d damage (20 = death at full health)%n",
          h, t, t / 20.0, (int) Math.floor(h - 3.0));
    }

    // Ender pearl: 1.5 blocks/tick from eye height minus 0.1; each tick gravity 0.03, drag 0.99, then move.
    double best = 0; int bestT = 0; double bestPitch = 0;
    for (double pitch = 0; pitch <= 80; pitch += 0.1) {
      double r = Math.toRadians(pitch), py = 1.62 - 0.1, pz = 0, vy2 = 1.5 * Math.sin(r), vz2 = 1.5 * Math.cos(r);
      int t = 0;
      while (py > 0 && t < 2000) { vy2 -= 0.03; vy2 *= 0.99; vz2 *= 0.99; py += vy2; pz += vz2; t++; }
      if (pz > best) { best = pz; bestT = t; bestPitch = pitch; }
    }
    System.out.printf("%nEnder pearl: max range on flat %.1f blocks at %.1f deg up, %d ticks in flight (%.1f m/s),%n",
        best, bestPitch, bestT, best / bestT * 20);
    System.out.printf("  20-tick cooldown, 5 damage per throw = 5 x 6 exhaustion = 7.5 food points to heal,%n");
    System.out.printf("  or 400 ticks of natural regeneration (80 ticks per health point at food >= 18)%n");
  }
}
