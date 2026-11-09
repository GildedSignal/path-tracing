import java.io.IOException;
import java.io.PrintWriter;
import java.lang.reflect.Method;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Locale;
import java.util.SplittableRandom;
import net.minecraft.SharedConstants;
import net.minecraft.server.Bootstrap;
import net.minecraft.world.entity.LivingEntity;
import net.minecraft.world.entity.decoration.ArmorStand;
import net.minecraft.world.phys.Vec3;
import sun.misc.Unsafe;

/** Runs Mojang's free-glide step before collision, with default gravity and no effects. */
public class ElytraOracle {
    private static final class FlightProbe extends ArmorStand {
        FlightProbe() { super(null, 0, 0, 0); }
        @Override protected double getEffectiveGravity() { return 0.08; }
    }

    private static Vec3 step(FlightProbe probe, Method step, Vec3 velocity, float yaw, float pitch)
            throws ReflectiveOperationException {
        probe.setYRot(yaw);
        probe.setXRot(pitch);
        return (Vec3) step.invoke(probe, velocity);
    }

    private static void capture(PrintWriter out, FlightProbe probe, Method step, Vec3 velocity, float yaw, float pitch)
            throws ReflectiveOperationException {
        Vec3 next = step(probe, step, velocity, yaw, pitch);
        out.printf(Locale.ROOT, "%08x,%08x,%016x,%016x,%016x,%016x,%016x,%016x%n",
            Float.floatToRawIntBits(yaw), Float.floatToRawIntBits(pitch),
            Double.doubleToRawLongBits(velocity.x), Double.doubleToRawLongBits(velocity.y),
            Double.doubleToRawLongBits(velocity.z), Double.doubleToRawLongBits(next.x),
            Double.doubleToRawLongBits(next.y), Double.doubleToRawLongBits(next.z));
    }

    public static void main(String[] args) throws ReflectiveOperationException, IOException {
        SharedConstants.tryDetectVersion();
        Bootstrap.bootStrap();
        var unsafeField = Unsafe.class.getDeclaredField("theUnsafe");
        unsafeField.setAccessible(true);
        // Bypass construction of an unused world. Rotation is initialized before each call.
        FlightProbe probe = (FlightProbe) ((Unsafe) unsafeField.get(null)).allocateInstance(FlightProbe.class);
        Method step = LivingEntity.class.getDeclaredMethod("updateFallFlyingMovement", Vec3.class);
        step.setAccessible(true);
        Vec3 dive = new Vec3(0.1, -0.8, 0.05);
        for (int i = 0; i < 32768; i++) {
            step(probe, step, dive, 90, i % 181 - 90);
        }
        try (var out = new PrintWriter(Files.newBufferedWriter(Path.of(args[0])))) {
            out.println("yaw,pitch,vx,vy,vz,next_vx,next_vy,next_vz");
            capture(out, probe, step, new Vec3(0.5, -0.2, 0.5), 0, 0);
            capture(out, probe, step, dive, 90, 90);
            capture(out, probe, step, new Vec3(0.7, -0.1, 0.3), 210, -30);
            capture(out, probe, step, new Vec3(0, -0.5, 0), 0, 0);
            for (float pitch : new float[] {-90, -0.0f, 0.0f, 90}) {
                capture(out, probe, step, dive, 90, Math.nextDown(pitch));
                capture(out, probe, step, dive, 90, Math.nextUp(pitch));
            }
            var random = new SplittableRandom(262);
            for (int i = 0; i < 8192; i++) {
                float pitch = (float) random.nextDouble(-90, 90);
                float yaw = (float) random.nextDouble(-180, 180);
                var velocity = new Vec3(random.nextDouble(-8, 8), random.nextDouble(-8, 8), random.nextDouble(-8, 8));
                capture(out, probe, step, velocity, yaw, pitch);
            }
            if (out.checkError()) throw new IOException("Could not write oracle output.");
        }
    }
}
