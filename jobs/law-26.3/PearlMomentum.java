import net.minecraft.SharedConstants;
import net.minecraft.world.entity.PositionMoveRotation;
import net.minecraft.world.entity.Relative;
import net.minecraft.world.phys.Vec3;
import org.objectweb.asm.*;

// Run on each Mojang jar with ASM and that jar's libraries, and collect stdout as TSV.
class PearlMomentum {
    public static void main(String[] args) throws Exception {
        SharedConstants.tryDetectVersion();
        var relativeDelta = new boolean[1];
        try (var bytes = PearlMomentum.class.getResourceAsStream(
                "/net/minecraft/world/entity/projectile/throwableitemprojectile/ThrownEnderpearl.class")) {
            new ClassReader(bytes).accept(new ClassVisitor(Opcodes.ASM9) {
                @Override public MethodVisitor visitMethod(int access, String name, String descriptor,
                        String signature, String[] exceptions) {
                    if (!name.equals("onHit")) return null;
                    return new MethodVisitor(Opcodes.ASM9) {
                        @Override public void visitFieldInsn(int opcode, String owner, String name, String descriptor) {
                            if (opcode == Opcodes.GETSTATIC && owner.equals("net/minecraft/world/entity/Relative")
                                    && name.equals("DELTA")) relativeDelta[0] = true;
                        }
                    };
                }
            }, ClassReader.SKIP_DEBUG | ClassReader.SKIP_FRAMES);
        }
        var input = new PositionMoveRotation(new Vec3(7, 65, 11), new Vec3(0.25, -0.125, 0.5), 23, -5);
        var landing = new PositionMoveRotation(new Vec3(19, 64, -3), Vec3.ZERO, 0, 0);
        var relatives = relativeDelta[0] ? Relative.union(Relative.ROTATION, Relative.DELTA) : Relative.ROTATION;
        var velocity = PositionMoveRotation.calculateAbsolute(input, landing, relatives).deltaMovement();
        System.out.printf("%d\t%016x\t%016x\t%016x%n", SharedConstants.getCurrentVersion().dataVersion().version(),
            Double.doubleToRawLongBits(velocity.x), Double.doubleToRawLongBits(velocity.y), Double.doubleToRawLongBits(velocity.z));
    }
}
