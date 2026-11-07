import java.lang.reflect.*;
import com.google.common.collect.ImmutableList;
import net.minecraft.SharedConstants;
import net.minecraft.server.Bootstrap;
import net.minecraft.network.syncher.SynchedEntityData;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.damagesource.DamageSource;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.level.storage.ValueInput;
import net.minecraft.world.level.storage.ValueOutput;
import net.minecraft.world.phys.Vec3;
import sun.misc.Unsafe;

// Frozen direct-runtime input corpus: one call per pair. No arithmetic oracle here.
public class Oracle {
  static class Probe extends Entity {
    boolean pushable, vehicle, same;
    Probe() { super(null, null); }
    public boolean isPushable() { return pushable; }
    public boolean isVehicle() { return vehicle; }
    public boolean isPassengerOfSameVehicle(Entity other) { return same; }
    protected void defineSynchedData(SynchedEntityData.Builder b) {}
    public boolean hurtServer(ServerLevel l, DamageSource s, float f) { return false; }
    protected void readAdditionalSaveData(ValueInput i) {}
    protected void addAdditionalSaveData(ValueOutput o) {}
  }
  static Unsafe u; static Field pos,delta,passengers;
  static long bits(double x) { return Double.doubleToRawLongBits(x); }
  static void run(int id,double dx,double dz,int mask) throws Exception {
    Probe a=(Probe)u.allocateInstance(Probe.class), b=(Probe)u.allocateInstance(Probe.class);
    pos.set(a,new Vec3(0,8,0)); pos.set(b,new Vec3(dx,-7,dz));
    delta.set(a,new Vec3(0.125,-0.0,-0.375)); delta.set(b,new Vec3(-0.75,2.5,0.25));
    passengers.set(a,ImmutableList.of()); passengers.set(b,ImmutableList.of());
    a.noPhysics=(mask&1)!=0; b.noPhysics=(mask&2)!=0;
    a.vehicle=(mask&4)!=0; b.vehicle=(mask&8)!=0;
    a.pushable=(mask&16)!=0; b.pushable=(mask&32)!=0; a.same=(mask&64)!=0;
    a.push((Entity)b);
    Vec3 x=a.getDeltaMovement(), y=b.getDeltaMovement();
    System.out.printf("%d %016x %016x %d %016x %016x %016x %016x %016x %016x%n",id,bits(dx),bits(dz),mask,bits(x.x),bits(x.y),bits(x.z),bits(y.x),bits(y.y),bits(y.z));
  }
  public static void main(String[] args) throws Exception {
    SharedConstants.tryDetectVersion(); Bootstrap.bootStrap();
    Field f=Unsafe.class.getDeclaredField("theUnsafe"); f.setAccessible(true); u=(Unsafe)f.get(null);
    pos=Entity.class.getDeclaredField("position"); pos.setAccessible(true);
    delta=Entity.class.getDeclaredField("deltaMovement"); delta.setAccessible(true);
    passengers=Entity.class.getDeclaredField("passengers"); passengers.setAccessible(true);
    double c=(double)0.01f; double[] d={Math.nextDown(c),c,Math.nextUp(c),Math.nextDown(1.0),1.0,Math.nextUp(1.0),4.0}; int id=0;
    for(double v:d) for(int sign:new int[]{-1,1}) for(int axis=0;axis<3;axis++) run(id++,axis==1?0:sign*v,axis==0?0:sign*v,48);
    for(int m=0;m<128;m++) run(id++,0.25,-0.5,m);
  }
}
