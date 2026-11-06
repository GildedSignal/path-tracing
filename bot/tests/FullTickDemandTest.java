package pathtracing;

import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Map;
import net.fabricmc.api.EnvType;
import net.fabricmc.loader.impl.launch.knot.Knot;
import net.minecraft.client.DeltaTracker;
import org.objectweb.asm.ClassReader;
import org.objectweb.asm.ClassWriter;
import org.objectweb.asm.Opcodes;
import org.objectweb.asm.Type;
import org.objectweb.asm.commons.MethodRemapper;
import org.objectweb.asm.commons.SimpleRemapper;
import org.objectweb.asm.tree.*;

/** Exercises the Fabric-transformed loop with a counting tick and no window or network. */
public final class FullTickDemandTest {
    private static final String MC = "net/minecraft/client/Minecraft";
    private static final String CARRIER = "pathtracing/ClockLoopCarrier";

    public static final class Rate {
        public boolean normal = true;
        public boolean frozen;
        public boolean runsNormally() { return normal; }
        public boolean isFrozen() { return frozen; }
    }

    public static final class Level {
        public final Rate rate = new Rate();
        public Rate tickRateManager() { return rate; }
    }

    public static void main(String[] args) throws Exception {
        var loader = new Knot(EnvType.CLIENT).init(new String[] {"--gameDir", args[0]});
        Class.forName(MC.replace('/', '.'), false, loader);
        var exported = new ClassNode();
        new ClassReader(Files.readAllBytes(Path.of(".mixin.out/class", MC + ".class"))).accept(exported, 0);
        var carrier = carrier(exported);
        var timer = new DeltaTracker.Timer(20, 0, base -> 0.5f);
        int demand = timer.advanceGameTime(12);
        require(demand == 24, "Vanilla timer must supply 24 ticks");
        boolean enabled = Boolean.parseBoolean(args[1]);
        verify(carrier, 0, false, new Level(), 0);
        verify(carrier, 4, false, new Level(), 4);
        verify(carrier, demand, false, new Level(), enabled ? demand : 10);
        verify(carrier, demand, true, new Level(), 10);
        var frozen = new Level();
        frozen.rate.frozen = true;
        frozen.rate.normal = false;
        verify(carrier, demand, false, frozen, 10);
        frozen.rate.normal = true; // The manager can run elements while stepping a frozen world.
        verify(carrier, demand, false, frozen, 10);
        verify(carrier, demand, false, null, 10);
        System.out.printf("Fabric-transformed loop, full demand %s: timer produced %d, executed %d. Pause, freeze, step and no level capped at 10.%n",
            enabled, demand, enabled ? demand : 10);
    }

    private static void verify(Class<?> carrier, int demand, boolean pause, Level level, int expected) throws Exception {
        Object instance = carrier.getConstructor().newInstance();
        carrier.getField("pause").setBoolean(instance, pause);
        carrier.getField("level").set(instance, level);
        int actual = (int) carrier.getMethod("runTick", int.class).invoke(instance, demand);
        require(actual == expected, "Demand " + demand + ": expected " + expected + ", executed " + actual);
    }

    private static Class<?> carrier(ClassNode source) {
        var loop = source.methods.stream().filter(m -> m.name.equals("runTick") && m.desc.equals("(Z)V"))
            .findFirst().orElseThrow();
        var clamp = call(loop, "pathtracing$ticksThisFrame");
        var tick = call(loop, "tick");
        var counted = call(loop, "pathtracing$countCompletedTick");
        var finished = call(loop, "pathtracing$finishClockFrame");
        require(clamp.desc.equals("(II)I"), "Transformed clamp descriptor");
        var exit = (JumpInsnNode) nextCode(clamp);
        require(exit.getOpcode() == Opcodes.IF_ICMPGE, "Redirect must feed the loop exit");
        require(tick == previousCode(previousCode(previousCode(previousCode(counted)))), "Audit must follow the real tick call");
        var increment = nextCode(counted);
        while (!(increment instanceof IincInsnNode)) increment = nextCode(increment);
        var back = (JumpInsnNode) nextCode(increment);
        require(back.getOpcode() == Opcodes.GOTO && exit.label != back.label, "Vanilla loop back edge");
        require(nextCode(finished).getOpcode() == Opcodes.RETURN, "Audit must close at frame return");

        var remapper = new SimpleRemapper(Map.of(
            MC, CARRIER,
            "net/minecraft/client/multiplayer/ClientLevel", Type.getInternalName(Level.class),
            "net/minecraft/world/TickRateManager", Type.getInternalName(Rate.class)));
        var writer = new ClassWriter(ClassWriter.COMPUTE_FRAMES | ClassWriter.COMPUTE_MAXS);
        writer.visit(Opcodes.V25, Opcodes.ACC_PUBLIC, CARRIER, null, "java/lang/Object", null);
        writer.visitField(Opcodes.ACC_PUBLIC, "level", Type.getDescriptor(Level.class), null, null).visitEnd();
        writer.visitField(Opcodes.ACC_PUBLIC, "pause", "Z", null, null).visitEnd();
        writer.visitField(Opcodes.ACC_PUBLIC, "ticks", "I", null, null).visitEnd();
        var init = writer.visitMethod(Opcodes.ACC_PUBLIC, "<init>", "()V", null, null);
        init.visitVarInsn(Opcodes.ALOAD, 0);
        init.visitMethodInsn(Opcodes.INVOKESPECIAL, "java/lang/Object", "<init>", "()V", false);
        init.visitInsn(Opcodes.RETURN);
        init.visitMaxs(0, 0);
        init.visitEnd();
        var count = writer.visitMethod(Opcodes.ACC_PUBLIC, "tick", "()V", null, null);
        count.visitVarInsn(Opcodes.ALOAD, 0);
        count.visitInsn(Opcodes.DUP);
        count.visitFieldInsn(Opcodes.GETFIELD, CARRIER, "ticks", "I");
        count.visitInsn(Opcodes.ICONST_1);
        count.visitInsn(Opcodes.IADD);
        count.visitFieldInsn(Opcodes.PUTFIELD, CARRIER, "ticks", "I");
        count.visitInsn(Opcodes.RETURN);
        count.visitMaxs(0, 0);
        count.visitEnd();

        // Copy the transformed loop test, actual tick call, increment and back edge.
        var body = new MethodNode(Opcodes.ACC_PUBLIC, "runTick", "(I)I", null, null);
        body.visitVarInsn(Opcodes.ILOAD, 1);
        body.visitVarInsn(Opcodes.ISTORE, 2);
        body.visitInsn(Opcodes.ICONST_0);
        body.visitVarInsn(Opcodes.ISTORE, 4);
        Map<LabelNode, LabelNode> labels = Map.of(back.label, new LabelNode(), exit.label, new LabelNode());
        body.instructions.add(labels.get(back.label));
        for (var insn = nextCode(back.label); insn != exit.getNext(); insn = insn.getNext()) {
            if (insn.getOpcode() >= 0) body.instructions.add(insn.clone(labels));
        }
        body.visitVarInsn(Opcodes.ALOAD, 0);
        body.instructions.add(tick.clone(labels));
        body.instructions.add(increment.clone(labels));
        body.instructions.add(back.clone(labels));
        body.instructions.add(labels.get(exit.label));
        body.visitVarInsn(Opcodes.ALOAD, 0);
        body.visitFieldInsn(Opcodes.GETFIELD, CARRIER, "ticks", "I");
        body.visitInsn(Opcodes.IRETURN);
        body.accept(new MethodRemapper(writer.visitMethod(body.access, body.name, body.desc, null, null), remapper));
        var handler = source.methods.stream().filter(m -> m.name.equals(clamp.name)).findFirst().orElseThrow();
        handler.accept(new MethodRemapper(writer.visitMethod(handler.access, handler.name, handler.desc, null, null), remapper));
        writer.visitEnd();
        byte[] bytes = writer.toByteArray();
        return new ClassLoader(FullTickDemandTest.class.getClassLoader()) {
            Class<?> define() { return defineClass(CARRIER.replace('/', '.'), bytes, 0, bytes.length); }
        }.define();
    }

    private static MethodInsnNode call(MethodNode method, String name) {
        MethodInsnNode result = null;
        for (var insn : method.instructions) {
            if (insn instanceof MethodInsnNode call && call.owner.equals(MC) && call.name.endsWith(name)) {
                require(result == null, "Multiple calls to " + name);
                result = call;
            }
        }
        if (result == null) throw new AssertionError("Missing call to " + name);
        return result;
    }

    private static AbstractInsnNode nextCode(AbstractInsnNode insn) {
        do { insn = insn.getNext(); } while (insn.getOpcode() < 0);
        return insn;
    }

    private static AbstractInsnNode previousCode(AbstractInsnNode insn) {
        do { insn = insn.getPrevious(); } while (insn.getOpcode() < 0);
        return insn;
    }

    private static void require(boolean condition, String message) {
        if (!condition) throw new AssertionError(message);
    }
}
