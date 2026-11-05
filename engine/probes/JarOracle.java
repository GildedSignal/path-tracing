import java.io.PrintWriter;
import java.lang.reflect.Field;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Random;
import net.minecraft.core.Direction;
import net.minecraft.util.Mth;
import net.minecraft.world.phys.AABB;
import net.minecraft.world.phys.Vec3;
import net.minecraft.world.phys.shapes.Shapes;
import net.minecraft.world.phys.shapes.VoxelShape;

public class JarOracle {
    // Mth.SIN; Shapes.collide; Direction.axisStepOrder
    public static void main(String[] args) throws Exception {
        Field field = Mth.class.getDeclaredField("SIN");
        field.setAccessible(true);
        float[] sine = (float[]) field.get(null);
        ByteBuffer buffer = ByteBuffer.allocate(sine.length * 4).order(ByteOrder.LITTLE_ENDIAN);
        for (float f : sine) buffer.putFloat(f);
        Files.write(Path.of(args[0]), buffer.array());
        Random rng = new Random(262);
        try (PrintWriter out = new PrintWriter(args[2])) {
            for (int n = 0; n < 512; n++) {
                Vec3 from = new Vec3((rng.nextInt(40)-20)/16.0,(rng.nextInt(40)-20)/16.0,(rng.nextInt(40)-20)/16.0);
                Vec3 delta = new Vec3((rng.nextInt(400)-200)/16.0,(rng.nextInt(400)-200)/16.0,(rng.nextInt(400)-200)/16.0);
                if (n % 4 == 0) delta = new Vec3(delta.x,0,0);
                if (n % 4 == 1) delta = Vec3.ZERO;
                Vec3 to = from.add(delta);
                double radius = 0.6f/2.0f;
                AABB bounds = new AABB(to.x-radius,to.y,to.z-radius,to.x+radius,to.y+1.8f,to.z+radius).deflate(1.0e-5f);
                for (double d : new double[]{from.x,from.y,from.z,to.x,to.y,to.z,bounds.minX,bounds.minY,bounds.minZ,bounds.maxX,bounds.maxY,bounds.maxZ}) {
                    out.print(String.format("%016x",Double.doubleToRawLongBits(d))+" ");
                }
                out.print("|");
                net.minecraft.world.level.BlockGetter.forEachBlockIntersectedBetween(from,to,bounds,(p,step)-> {
                    out.print(p.getX()+","+p.getY()+","+p.getZ()+","+step+";");
                    return true;
                });
                out.println();
            }
        }
        try (PrintWriter out = new PrintWriter(args[1])) {
            for (int n = 0; n < 1024; n++) {
                double[] a = new double[6];
                for (int j = 0; j < 3; j++) { a[j] = (rng.nextInt(40)-20)/16.0; a[j+3] = a[j]+(j==1?1.8f:0.6f); }
                AABB body = new AABB(a[0],a[1],a[2],a[3],a[4],a[5]);
                double[] v = new double[3];
                for (int j = 0; j < 3; j++) v[j] = (rng.nextInt(40)-20)/16.0;
                if (n % 4 == 0) v[2]=v[0];
                Vec3 wanted = new Vec3(v[0],v[1],v[2]);
                double[] box = new double[18];
                VoxelShape shape = Shapes.empty();
                for (int k=0;k<3;k++) {
                    for (int j=0;j<3;j++) { box[k*6+j]=(rng.nextInt(32)-16)/8.0; box[k*6+j+3]=box[k*6+j]+(rng.nextInt(8)+1)/8.0; }
                    shape=Shapes.or(shape,Shapes.box(box[k*6],box[k*6+1],box[k*6+2],box[k*6+3],box[k*6+4],box[k*6+5]));
                }
                if (n >= 512) {
                    double offset = 1.0 + (n % 16 - 8) * 2.5e-8;
                    a = new double[]{offset,0,0,offset+0.6f,1.8f,0.6f};
                    body = new AABB(a[0],a[1],a[2],a[3],a[4],a[5]);
                    shape = Shapes.block();
                    for (int k=0;k<3;k++) {
                        for (int j=0;j<3;j++) { box[k*6+j]=0; box[k*6+j+3]=1; }
                    }
                }
                Vec3 result=Vec3.ZERO;
                for (Direction.Axis axis : Direction.axisStepOrder(wanted)) {
                    double distance=wanted.get(axis);
                    if (distance != 0) result=result.with(axis,Shapes.collide(axis,body.move(result),java.util.List.of(shape),distance));
                }
                for (double d:a) out.print(String.format("%016x",Double.doubleToRawLongBits(d))+"\t");
                for (double d:v) out.print(String.format("%016x",Double.doubleToRawLongBits(d))+"\t");
                for (double d:box) out.print(String.format("%016x",Double.doubleToRawLongBits(d))+"\t");
                for (double d:new double[]{result.x,result.y,result.z}) out.print(String.format("%016x",Double.doubleToRawLongBits(d))+"\t");
                boolean overlap=Shapes.joinIsNotEmpty(shape,Shapes.create(body),net.minecraft.world.phys.shapes.BooleanOp.AND);
                out.print(String.format("%016x",Double.doubleToRawLongBits(overlap?1.0:0.0)));
                out.println();
            }
        }
    }
}

