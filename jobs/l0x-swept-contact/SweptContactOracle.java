import java.util.ArrayList;
import java.util.List;
import java.util.Random;
import net.minecraft.world.phys.AABB;
import net.minecraft.world.phys.Vec3;

public class SweptContactOracle {
    private static void values(StringBuilder row, double... values) {
        for (double value : values) row.append(' ').append(Long.toHexString(Double.doubleToRawLongBits(value)));
    }

    private static void box(StringBuilder row, AABB box) {
        values(row, box.minX, box.minY, box.minZ, box.maxX, box.maxY, box.maxZ);
    }

    private static double ulps(double value, int count) {
        for (int i = 0; i < Math.abs(count); i++) value = count < 0 ? Math.nextDown(value) : Math.nextUp(value);
        return value;
    }

    private static void sample(String label, AABB player, Vec3 movement, List<AABB> hazards) {
        StringBuilder row = new StringBuilder(label);
        box(row, player);
        values(row, movement.x, movement.y, movement.z);
        for (AABB hazard : hazards) box(row, hazard);
        row.append(' ').append(player.collidedAlongVector(movement, hazards));
        System.out.println(row);
    }

    public static void main(String[] args) {
        Random random = new Random(262);
        for (int distance : new int[] {0, 10_000, 1_000_000, 29_999_984}) {
            for (int sign : new int[] {-1, 1}) {
                double origin = sign * (double) distance;
                for (int seed = 0; seed < 32; seed++) {
                    double x = origin + (seed == 0 ? 0.0 : random.nextDouble());
                    AABB player = new AABB(x - 0.30000001192092896, 0, -1,
                        x + 0.30000001192092896, 1.7999999523162842, -0.3999999761581421);
                    double inset = player.getCenter().x + player.getXsize() * 0.5 - 1e-7;
                    for (int n = -4; n <= 4; n++) {
                        for (double plane : new double[] {ulps(inset, n), player.maxX}) {
                            var wall = new AABB(plane, -0.25, 0, plane + 0.5, 1, 0.5);
                            sample("grazing/" + distance, player, new Vec3(0, 0, 2), List.of(wall));
                        }
                        double plane = ulps(inset + 0.25, n);
                        var hazards = List.of(new AABB(plane, 0.75, 0, plane + 0.5, 1.5, 0.25),
                            new AABB(plane + 0.125, -0.5, 0.125, plane + 0.75, 0.25, 0.75));
                        for (double xMotion : new double[] {0.125, 0.25, 0.75}) {
                            sample("compound/" + distance, player, new Vec3(xMotion, -0.125, 2), hazards);
                        }
                    }
                }
            }
        }
    }
}
