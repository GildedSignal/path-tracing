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
    static class ProbeInput extends net.minecraft.client.player.ClientInput {
        boolean forward;
        @Override public boolean hasForwardImpulse() { return forward; }
    }

    static class ProbePlayer extends net.minecraft.client.player.LocalPlayer {
        boolean sprint, water, under, slow, ground, food;
        net.minecraft.world.damagesource.DamageSources damage;
        @Override public net.minecraft.world.damagesource.DamageSources damageSources() { return damage; }
        double landingDistance;
        float landingMultiplier;
        float moveSpeed;
        @Override public float getSpeed() { return moveSpeed; }
        @Override public double getAttributeValue(net.minecraft.core.Holder<net.minecraft.world.entity.ai.attributes.Attribute> attribute) {
            return attribute == net.minecraft.world.entity.ai.attributes.Attributes.SAFE_FALL_DISTANCE ? 3.0 : 1.0;
        }
        net.minecraft.world.entity.player.Abilities abilities;
        ProbePlayer() { super(null, null, null, null, null, null, false, null); }
        @Override public boolean isSprinting() { return sprint; }
        @Override public boolean isMobilityRestricted() { return false; }
        @Override public boolean isPassenger() { return false; }
        @Override protected boolean hasEnoughFoodToDoExhaustiveManoeuvres() { return food; }
        @Override public boolean isUsingItem() { return false; }
        @Override public boolean isFallFlying() { return false; }
        @Override public boolean isUnderWater() { return under; }
        @Override public boolean isInWater() { return water; }
        @Override public boolean isMovingSlowly() { return slow; }
        @Override public boolean onGround() { return ground; }
        @Override public net.minecraft.world.entity.player.Abilities getAbilities() { return abilities; }
        @Override public void playSound(net.minecraft.sounds.SoundEvent sound, float volume, float pitch) {}
        @Override public boolean causeFallDamage(double distance, float multiplier, net.minecraft.world.damagesource.DamageSource source) {
            landingDistance = distance;
            landingMultiplier = multiplier;
            return false;
        }
    }

    // LocalPlayer.canStartSprinting; LocalPlayer.shouldStopRunSprinting; LocalPlayer.shouldStopSwimSprinting
    static void sprintOracle(Path path, sun.misc.Unsafe unsafe) throws Exception {
        ProbePlayer player = (ProbePlayer) unsafe.allocateInstance(ProbePlayer.class);
        player.abilities = new net.minecraft.world.entity.player.Abilities();
        ProbeInput input = new ProbeInput();
        player.input = input;
        var start = net.minecraft.client.player.LocalPlayer.class.getDeclaredMethod("canStartSprinting");
        var runStop = net.minecraft.client.player.LocalPlayer.class.getDeclaredMethod("shouldStopRunSprinting");
        var swimStop = net.minecraft.client.player.LocalPlayer.class.getDeclaredMethod("shouldStopSwimSprinting");
        start.setAccessible(true);
        runStop.setAccessible(true);
        swimStop.setAccessible(true);
        try (PrintWriter out = new PrintWriter(path.toFile())) {
            for (int bits = 0; bits < 4096; bits++) {
                player.sprint = (bits & 1) != 0;
                input.forward = (bits & 2) != 0;
                player.food = (bits & 4) != 0;
                player.water = (bits & 8) != 0;
                player.under = (bits & 16) != 0;
                boolean swimming = (bits & 32) != 0;
                player.ground = (bits & 64) != 0;
                boolean sneak = (bits & 128) != 0;
                player.horizontalCollision = (bits & 256) != 0;
                player.minorHorizontalCollision = (bits & 512) != 0;
                player.slow = (bits & 1024) != 0;
                boolean key = (bits & 2048) != 0;
                input.keyPresses = new net.minecraft.world.entity.player.Input(input.forward, false, false, false, false, sneak, key);
                boolean latched = player.sprint || ((boolean) start.invoke(player) && key);
                boolean stop = (boolean) (swimming ? swimStop : runStop).invoke(player);
                out.println(bits + " " + (latched && !stop));
            }
        }
    }

    // BedBlock.fallOn; HayBlock.fallOn; HoneyBlock.fallOn
    static void landingOracle(Path path, sun.misc.Unsafe unsafe) throws Exception {
        var level = (net.minecraft.client.multiplayer.ClientLevel) unsafe.allocateInstance(net.minecraft.client.multiplayer.ClientLevel.class);
        Field side = net.minecraft.world.level.Level.class.getDeclaredField("isClientSide");
        unsafe.putBoolean(level, unsafe.objectFieldOffset(side), true);
        Field sources = net.minecraft.world.level.Level.class.getDeclaredField("damageSources");
        unsafe.putObject(level, unsafe.objectFieldOffset(sources),
            unsafe.allocateInstance(net.minecraft.world.damagesource.DamageSources.class));
        ProbePlayer player = (ProbePlayer) unsafe.allocateInstance(ProbePlayer.class);
        player.damage = level.damageSources();
        Field entityType = net.minecraft.world.entity.Entity.class.getDeclaredField("type");
        unsafe.putObject(player, unsafe.objectFieldOffset(entityType), net.minecraft.world.entity.EntityTypes.PLAYER);
        var fallDamage = net.minecraft.world.entity.LivingEntity.class.getDeclaredMethod("calculateFallDamage", double.class, float.class);
        fallDamage.setAccessible(true);
        try (PrintWriter out = new PrintWriter(path.toFile())) {
            for (Class<?> type : new Class<?>[]{net.minecraft.world.level.block.BedBlock.class,
                    net.minecraft.world.level.block.HayBlock.class, net.minecraft.world.level.block.HoneyBlock.class}) {
                var block = (net.minecraft.world.level.block.Block) unsafe.allocateInstance(type);
                for (double distance : new double[]{3, 4, 6, 10, 23}) {
                    block.fallOn(level, null, net.minecraft.core.BlockPos.ZERO, player, distance);
                    out.printf(java.util.Locale.ROOT, "%s %.1f %016x %08x %d%n", type.getSimpleName(), distance,
                        Double.doubleToRawLongBits(player.landingDistance), Float.floatToRawIntBits(player.landingMultiplier),
                        (int) fallDamage.invoke(player, player.landingDistance, player.landingMultiplier));
                }
            }
        }
    }

    // LivingEntity.getFrictionInfluencedSpeed; LocalPlayer.modifyInput; Entity.getInputVector; AttributeInstance.getValue
    static void brakingOracle(Path path, sun.misc.Unsafe unsafe) throws Exception {
        ProbePlayer player = (ProbePlayer) unsafe.allocateInstance(ProbePlayer.class);
        player.ground = true;
        var frictionSpeed = net.minecraft.world.entity.LivingEntity.class.getDeclaredMethod("getFrictionInfluencedSpeed", float.class);
        var modifiedFriction = net.minecraft.world.entity.LivingEntity.class.getDeclaredMethod("computeModifiedFriction", float.class, float.class);
        var modifyInput = net.minecraft.client.player.LocalPlayer.class.getDeclaredMethod("modifyInput", net.minecraft.world.phys.Vec2.class);
        var inputVector = net.minecraft.world.entity.Entity.class.getDeclaredMethod("getInputVector", Vec3.class, float.class, float.class);
        frictionSpeed.setAccessible(true);
        modifiedFriction.setAccessible(true);
        modifyInput.setAccessible(true);
        inputVector.setAccessible(true);
        Field sprintModifier = net.minecraft.world.entity.LivingEntity.class.getDeclaredField("SPEED_MODIFIER_SPRINTING");
        sprintModifier.setAccessible(true);
        var attribute = new net.minecraft.world.entity.ai.attributes.AttributeInstance(
            net.minecraft.world.entity.ai.attributes.Attributes.MOVEMENT_SPEED, ignored -> {});
        attribute.setBaseValue((double) 0.1f);
        float walkSpeed = (float) attribute.getValue();
        attribute.addTransientModifier((net.minecraft.world.entity.ai.attributes.AttributeModifier) sprintModifier.get(null));
        float sprintSpeed = (float) attribute.getValue();
        try (PrintWriter out = new PrintWriter(path.toFile())) {
            for (float rawFriction : new float[]{0.6f, 0.98f}) {
                float friction = (float) modifiedFriction.invoke(null, rawFriction, 1.0f);
                float drag = friction * (float) modifiedFriction.invoke(null, 0.91f, 1.0f);
                for (boolean sprint : new boolean[]{false, true}) {
                    player.moveSpeed = sprint ? sprintSpeed : walkSpeed;
                    double steady = 0;
                    for (int tick = 0; tick < 2000; tick++) {
                        float acceleration = (float) frictionSpeed.invoke(player, friction);
                        var movement = (net.minecraft.world.phys.Vec2) modifyInput.invoke(player, new net.minecraft.world.phys.Vec2(0, 1));
                        Vec3 input = (Vec3) inputVector.invoke(null, new Vec3(movement.x, 0, movement.y), acceleration, 0.0f);
                        steady = (steady + input.z) * drag;
                    }
                    for (String mode : new String[]{"recurrence", "player"}) {
                        for (int k = 0; k <= 6; k++) {
                            double velocity = steady;
                            player.moveSpeed = sprint ? sprintSpeed : walkSpeed;
                            int stopped = 0;
                            for (int tick = 1; tick <= 400; tick++) {
                                if (velocity * velocity < 9.0e-6) { stopped = tick; break; }
                                if (mode.equals("recurrence")) player.moveSpeed = walkSpeed;
                                float acceleration = (float) frictionSpeed.invoke(player, friction);
                                var movement = (net.minecraft.world.phys.Vec2) modifyInput.invoke(player,
                                    new net.minecraft.world.phys.Vec2(0, tick <= k ? -1 : 0));
                                Vec3 input = (Vec3) inputVector.invoke(null, new Vec3(movement.x, 0, movement.y), acceleration, 0.0f);
                                velocity = (velocity + input.z) * drag;
                                player.moveSpeed = walkSpeed;
                            }
                            if (stopped == 0) throw new AssertionError("braking threshold missing");
                            out.printf("%s %08x %s %d %d%n", mode, Float.floatToRawIntBits(rawFriction), sprint, k, stopped);
                        }
                    }
                }
            }
        }
    }

    // Mth.equal; Vec3.normalize; BlockGetter.traverseBlocks; VoxelShape.clip
    static void rayOracle(Path path) throws Exception {
        Random rng = new Random(1505);
        try (PrintWriter out = new PrintWriter(path.toFile())) {
            for (int n = 0; n < 512; n++) {
                Vec3 from = new Vec3((rng.nextInt(17)-8)/4.0, (rng.nextInt(17)-8)/4.0, (rng.nextInt(17)-8)/4.0);
                Vec3 to = from.add((rng.nextInt(65)-32)/4.0, (rng.nextInt(65)-32)/4.0, (rng.nextInt(65)-32)/4.0);
                if (n % 8 == 0) to = from;
                if (n % 8 == 1) to = from.add(0, -8, 0);
                if (n % 8 == 2) to = from.add(8, 8, 8);
                for (double d : new double[]{from.x, from.y, from.z, to.x, to.y, to.z}) {
                    out.printf("%016x ", Double.doubleToRawLongBits(d));
                }
                var target = new net.minecraft.core.BlockPos(rng.nextInt(5)-2, rng.nextInt(5)-2, rng.nextInt(5)-2);
                float height = (rng.nextInt(8)+1)/8.0f;
                out.printf("%d %d %d %08x ", target.getX(), target.getY(), target.getZ(), Float.floatToRawIntBits(height));
                final Vec3 end = to;
                Boolean hit = net.minecraft.world.level.BlockGetter.traverseBlocks(from, to, null, (context, p) -> {
                    if (!p.equals(target)) return null;
                    return Shapes.box(0, 0, 0, 1, height, 1).clip(from, end, p) == null ? null : Boolean.TRUE;
                }, context -> Boolean.FALSE);
                out.print(hit + "|");
                net.minecraft.world.level.BlockGetter.traverseBlocks(from, to, null, (context, p) -> {
                    out.print(p.getX()+","+p.getY()+","+p.getZ()+";");
                    return null;
                }, context -> null);
                out.println();
            }
        }
    }

    // Mth.equal; Vec3.normalize
    static void epsilonOracle(Path path) throws Exception {
        try (PrintWriter out = new PrintWriter(path.toFile())) {
            double epsilon = (double) 1.0e-5f;
            for (double value : new double[]{Math.nextDown(epsilon), epsilon, Math.nextUp(epsilon), 1.0e-5}) {
                Vec3 normalized = new Vec3(value, 0, 0).normalize();
                out.printf("%016x %s %016x%n", Double.doubleToRawLongBits(value), Mth.equal(0.0, value),
                    Double.doubleToRawLongBits(normalized.x));
            }
        }
    }

    // AttributeInstance.calculateValue
    static void attributeOracle(Path path) throws Exception {
        var operations = net.minecraft.world.entity.ai.attributes.AttributeModifier.Operation.values();
        try (PrintWriter out = new PrintWriter(path.toFile())) {
            for (int count : new int[]{0, 1, 4, 24, 25, 49}) {
                for (boolean sprint : new boolean[]{false, true}) {
                    var attribute = new net.minecraft.world.entity.ai.attributes.AttributeInstance(
                        net.minecraft.world.entity.ai.attributes.Attributes.MOVEMENT_SPEED, ignored -> {});
                    double base = 1.5;
                    attribute.setBaseValue(base);
                    Random random = new Random(1505 + count);
                    StringBuilder modifiers = new StringBuilder();
                    for (int n = 0; n < count * 3; n++) {
                        int operation = n % 3;
                        String id = "test:modifier_" + n;
                        double amount = (random.nextDouble() - 0.5) * 0.04;
                        attribute.addTransientModifier(new net.minecraft.world.entity.ai.attributes.AttributeModifier(
                            net.minecraft.resources.Identifier.parse(id), amount, operations[operation]));
                        modifiers.append(id).append(' ').append(String.format("%016x", Double.doubleToRawLongBits(amount)))
                            .append(' ').append(operation).append(';');
                    }
                    if (sprint) {
                        attribute.addTransientModifier(new net.minecraft.world.entity.ai.attributes.AttributeModifier(
                            net.minecraft.resources.Identifier.parse("minecraft:sprinting"), (double)0.3f, operations[2]));
                    }
                    out.printf("%016x %s %016x|%s%n", Double.doubleToRawLongBits(base), sprint,
                        Double.doubleToRawLongBits(attribute.getValue()), modifiers);
                }
            }
        }
    }

    // Mth.SIN; Shapes.collide; Direction.axisStepOrder
    public static void main(String[] args) throws Exception {
        net.minecraft.SharedConstants.tryDetectVersion();
        net.minecraft.server.Bootstrap.bootStrap();
        Field unsafeField = sun.misc.Unsafe.class.getDeclaredField("theUnsafe");
        unsafeField.setAccessible(true);
        sun.misc.Unsafe unsafe = (sun.misc.Unsafe) unsafeField.get(null);
        Path evidence = Path.of(args[1]).getParent();
        sprintOracle(evidence.resolve("sprint.tsv"), unsafe);
        landingOracle(evidence.resolve("landings.tsv"), unsafe);
        brakingOracle(evidence.resolve("braking.tsv"), unsafe);
        rayOracle(evidence.resolve("rays.tsv"));
        epsilonOracle(evidence.resolve("epsilons.tsv"));
        attributeOracle(evidence.resolve("attributes.tsv"));
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

