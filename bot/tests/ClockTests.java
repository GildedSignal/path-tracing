package pathtracing;

import java.lang.reflect.Method;
import java.util.List;
import java.util.Map;
import java.util.jar.JarFile;
import org.objectweb.asm.ClassReader;
import org.objectweb.asm.ClassWriter;
import org.objectweb.asm.Opcodes;
import org.objectweb.asm.Type;
import org.objectweb.asm.commons.MethodRemapper;
import org.objectweb.asm.commons.SimpleRemapper;
import org.objectweb.asm.tree.*;

/** Executes the compiled handler against the actual vanilla method, without starting Minecraft. */
public final class ClockTests {
    public static final class Rate {
        public boolean normal;
        public float millis;
        public boolean runsNormally() { return normal; }
        public float millisecondsPerTick() { return millis; }
    }

    public static final class Level {
        public Rate rate = new Rate();
        public Rate tickRateManager() { return rate; }
    }

    public static void main(String[] args) throws Exception {
        String property = "pathtracing.followTickRate";
        String previous = System.getProperty(property);
        try (var mod = new JarFile(args[0]); var game = new JarFile(args[1])) {
            ClassNode recorder = read(mod, "pathtracing/Recorder");
            for (String entry : List.of("open", "recordTrack")) {
                require(recorder.methods.stream().filter(m -> m.name.equals(entry)).anyMatch(m ->
                    java.util.stream.Stream.of(m.instructions.toArray()).anyMatch(i -> i instanceof MethodInsnNode call
                        && call.name.equals("requireQualifiedClock"))), entry + " must guard recording creation");
            }
            MethodNode guard = recorder.methods.stream().filter(m -> m.name.equals("requireQualifiedClock")).findFirst().orElseThrow();
            for (String callName : List.of("exceedsCeiling", "reply")) {
                require(java.util.stream.Stream.of(guard.instructions.toArray()).anyMatch(i -> i instanceof MethodInsnNode call
                    && call.name.equals(callName)), "Guard must call " + callName);
            }
            require(java.util.stream.Stream.of(guard.instructions.toArray()).anyMatch(i -> i.getOpcode() == Opcodes.ATHROW), "Guard must refuse");
            require(!ClockQualification.exceedsCeiling(true, true, 2000.0F), "Ceiling is inclusive");
            require(ClockQualification.exceedsCeiling(true, true, Math.nextUp(2000.0F)), "Above ceiling must refuse");
            require(!ClockQualification.exceedsCeiling(false, true, 3000.0F), "Vanilla clock does not follow");
            require(!ClockQualification.exceedsCeiling(true, false, 3000.0F), "Non-normal clock does not follow");
            ClassNode mixin = read(mod, "pathtracing/mixin/MinecraftClockMixin");
            AnnotationNode target = mixin.invisibleAnnotations.stream().filter(a -> a.desc.endsWith("/Mixin;")).findFirst().orElseThrow();
            require(value(target, "value").equals(List.of(Type.getObjectType("net/minecraft/client/Minecraft"))), "Mixin target");
            MethodNode handler = mixin.methods.stream().filter(m -> m.name.equals("pathtracing$followTickRate")).findFirst().orElseThrow();
            AnnotationNode inject = handler.visibleAnnotations.stream().filter(a -> a.desc.endsWith("/Inject;")).findFirst().orElseThrow();
            require(value(inject, "method").equals(List.of("getTickTargetMillis(F)F")), "Injection selector");
            require(value(inject, "cancellable").equals(true), "Injection must be cancellable");
            AnnotationNode at = (AnnotationNode) ((List<?>) value(inject, "at")).getFirst();
            require(value(at, "value").equals("RETURN"), "Injection must preserve vanilla return paths");
            String config = new String(mod.getInputStream(mod.getJarEntry("pathtracing.mixins.json")).readAllBytes(), java.nio.charset.StandardCharsets.UTF_8);
            require(config.contains("\"MinecraftClockMixin\""), "Mixin registration");
            ClassNode minecraft = read(game, "net/minecraft/client/Minecraft");
            MethodNode vanilla = minecraft.methods.stream().filter(m -> m.name.equals("getTickTargetMillis") && m.desc.equals("(F)F")).findFirst().orElseThrow();
            Class<?> carrier = carrier(vanilla, handler);
            Object instance = carrier.getConstructor().newInstance();
            Method original = carrier.getMethod("vanilla", float.class);
            Method patched = carrier.getMethod("patched", float.class);
            int cases = 0;
            for (String setting : new String[] {null, "false", "garbage", "true"}) {
                if (setting == null) System.clearProperty(property);
                else System.setProperty(property, setting);
                for (boolean present : new boolean[] {false, true}) {
                    for (boolean normal : new boolean[] {false, true}) {
                        for (float millis : new float[] {5, 10, 50, 100}) {
                            for (float base : new float[] {0, -0.0F, 5, 50, 100, Float.POSITIVE_INFINITY, Float.NaN}) {
                                Level level = present ? new Level() : null;
                                if (present) { level.rate.normal = normal; level.rate.millis = millis; }
                                carrier.getField("level").set(instance, level);
                                float vanillaResult = (float) original.invoke(instance, base);
                                float actual = (float) patched.invoke(instance, base);
                                float expected = "true".equals(setting) && present && normal ? millis : vanillaResult;
                                require(Float.floatToRawIntBits(actual) == Float.floatToRawIntBits(expected),
                                    "Clock mismatch for " + setting + ", level " + present + ", normal " + normal + ", base " + base);
                                cases++;
                            }
                        }
                    }
                }
            }
            System.out.println("Clock tests passed: " + cases + " cases using compiled injection and vanilla 26.2 bytecode.");
        } finally {
            if (previous == null) System.clearProperty(property);
            else System.setProperty(property, previous);
        }
    }

    private static Class<?> carrier(MethodNode vanilla, MethodNode handler) {
        String name = "pathtracing/ClockCarrier";
        var remapper = new SimpleRemapper(Opcodes.ASM9, Map.of(
            "net/minecraft/client/Minecraft", name,
            "pathtracing/mixin/MinecraftClockMixin", name,
            "net/minecraft/client/multiplayer/ClientLevel", Type.getInternalName(Level.class),
            "net/minecraft/world/TickRateManager", Type.getInternalName(Rate.class)));
        var writer = new ClassWriter(ClassWriter.COMPUTE_FRAMES | ClassWriter.COMPUTE_MAXS);
        writer.visit(Opcodes.V25, Opcodes.ACC_PUBLIC, name, null, "java/lang/Object", null);
        writer.visitField(Opcodes.ACC_PUBLIC, "level", Type.getDescriptor(Level.class), null, null).visitEnd();
        var constructor = writer.visitMethod(Opcodes.ACC_PUBLIC, "<init>", "()V", null, null);
        constructor.visitCode();
        constructor.visitVarInsn(Opcodes.ALOAD, 0);
        constructor.visitMethodInsn(Opcodes.INVOKESPECIAL, "java/lang/Object", "<init>", "()V", false);
        constructor.visitInsn(Opcodes.RETURN);
        constructor.visitMaxs(0, 0);
        constructor.visitEnd();
        handler.accept(new MethodRemapper(writer.visitMethod(Opcodes.ACC_PRIVATE, handler.name, handler.desc, null, null), remapper));
        vanilla.accept(new MethodRemapper(writer.visitMethod(Opcodes.ACC_PUBLIC, "vanilla", "(F)F", null, null), remapper));
        MethodNode patched = new MethodNode(Opcodes.ACC_PUBLIC, "patched", "(F)F", null, null);
        vanilla.accept(patched);
        // Model Mixin's cancellable RETURN callback at every vanilla FRETURN. The callback
        // starts with the exact vanilla value; the actual compiled handler can replace it.
        String callback = "org/spongepowered/asm/mixin/injection/callback/CallbackInfoReturnable";
        for (var instruction : patched.instructions.toArray()) {
            if (instruction.getOpcode() != Opcodes.FRETURN) continue;
            InsnList code = new InsnList();
            code.add(new VarInsnNode(Opcodes.FSTORE, 3));
            code.add(new TypeInsnNode(Opcodes.NEW, callback));
            code.add(new InsnNode(Opcodes.DUP));
            code.add(new LdcInsnNode("getTickTargetMillis"));
            code.add(new InsnNode(Opcodes.ICONST_1));
            code.add(new VarInsnNode(Opcodes.FLOAD, 3));
            code.add(new MethodInsnNode(Opcodes.INVOKESPECIAL, callback, "<init>", "(Ljava/lang/String;ZF)V", false));
            code.add(new VarInsnNode(Opcodes.ASTORE, 4));
            code.add(new VarInsnNode(Opcodes.ALOAD, 0));
            code.add(new VarInsnNode(Opcodes.FLOAD, 1));
            code.add(new VarInsnNode(Opcodes.ALOAD, 4));
            code.add(new MethodInsnNode(Opcodes.INVOKESPECIAL, name, handler.name, handler.desc, false));
            code.add(new VarInsnNode(Opcodes.ALOAD, 4));
            code.add(new MethodInsnNode(Opcodes.INVOKEVIRTUAL, callback, "getReturnValueF", "()F", false));
            patched.instructions.insertBefore(instruction, code);
        }
        patched.accept(new MethodRemapper(writer.visitMethod(Opcodes.ACC_PUBLIC, "patched", "(F)F", null, null), remapper));
        writer.visitEnd();
        byte[] bytes = writer.toByteArray();
        return new ClassLoader(ClockTests.class.getClassLoader()) {
            Class<?> define() { return defineClass("pathtracing.ClockCarrier", bytes, 0, bytes.length); }
        }.define();
    }

    private static ClassNode read(JarFile jar, String name) throws Exception {
        ClassNode result = new ClassNode();
        try (var input = jar.getInputStream(jar.getJarEntry(name + ".class"))) { new ClassReader(input).accept(result, 0); }
        return result;
    }

    private static Object value(AnnotationNode annotation, String key) {
        for (int i = 0; i < annotation.values.size(); i += 2) {
            if (annotation.values.get(i).equals(key)) return annotation.values.get(i + 1);
        }
        throw new AssertionError("Missing annotation key " + key);
    }

    private static void require(boolean condition, String message) {
        if (!condition) throw new AssertionError(message);
    }
}
