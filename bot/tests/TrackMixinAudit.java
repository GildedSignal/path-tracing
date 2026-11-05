package pathtracing;

import java.io.IOException;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.jar.JarFile;
import org.objectweb.asm.ClassReader;
import org.objectweb.asm.Opcodes;
import org.objectweb.asm.Type;
import org.objectweb.asm.tree.AnnotationNode;
import org.objectweb.asm.tree.ClassNode;
import org.objectweb.asm.tree.MethodInsnNode;
import org.objectweb.asm.tree.MethodNode;

/** Check the compiled injection signatures and call sites against the staged law jar. */
final class TrackMixinAudit {
    static void run(Path modPath, Path gamePath) throws IOException {
        int injections = 0;
        try (var mod = new JarFile(modPath.toFile()); var game = new JarFile(gamePath.toFile())) {
            for (String name : List.of("TrackInteractionMixin", "TrackPlayerMixin", "TrackAttackMixin", "TrackItemMixin", "TrackProjectileMixin")) {
                ClassNode mixin = read(mod, "pathtracing/mixin/" + name);
                AnnotationNode declaration = annotations(mixin.visibleAnnotations, mixin.invisibleAnnotations).stream()
                    .filter(a -> a.desc.equals("Lorg/spongepowered/asm/mixin/Mixin;")).findFirst().orElseThrow();
                Type targetType = (Type) ((List<?>) value(declaration, "value")).getFirst();
                ClassNode target = read(game, targetType.getInternalName());
                for (MethodNode handler : mixin.methods) {
                    for (AnnotationNode annotation : annotations(handler.visibleAnnotations, handler.invisibleAnnotations)) {
                        if (!annotation.desc.equals("Lorg/spongepowered/asm/mixin/injection/Inject;")) continue;
                        for (Object selector : (List<?>) value(annotation, "method")) {
                            var candidates = target.methods.stream().filter(m -> m.name.equals(selector)).toList();
                            require(candidates.size() == 1, name + ": missing or ambiguous target " + selector);
                            MethodNode method = candidates.getFirst();
                            Type[] parameters = Type.getArgumentTypes(handler.desc);
                            boolean returnsVoid = Type.getReturnType(method.desc).equals(Type.VOID_TYPE);
                            String callback = returnsVoid ? "CallbackInfo" : "CallbackInfoReturnable";
                            require(parameters[parameters.length - 1].getInternalName().endsWith("/" + callback), name + ": incorrect callback for " + selector);
                            if (parameters.length > 1) {
                                require(Arrays.equals(Arrays.copyOf(parameters, parameters.length - 1), Type.getArgumentTypes(method.desc)),
                                    name + ": incorrect captured parameters for " + selector);
                            }
                            require((handler.access & Opcodes.ACC_STATIC) == (method.access & Opcodes.ACC_STATIC), name + ": static mismatch");
                            for (Object item : (List<?>) value(annotation, "at")) {
                                AnnotationNode at = (AnnotationNode) item;
                                if (!value(at, "value").equals("INVOKE")) continue;
                                String invoke = (String) value(at, "target");
                                boolean found = false;
                                for (var instruction : method.instructions) {
                                    if (instruction instanceof MethodInsnNode call
                                        && invoke.equals("L" + call.owner + ";" + call.name + call.desc)) found = true;
                                }
                                require(found, name + ": missing call site " + invoke);
                            }
                            injections++;
                        }
                    }
                }
            }
        }
        require(injections == 18, "Expected 18 checked track injections, got " + injections);
        System.out.println("Verified " + injections + " compiled track injections against Minecraft 26.2.");
    }

    private static ClassNode read(JarFile jar, String name) throws IOException {
        ClassNode node = new ClassNode();
        try (var input = jar.getInputStream(jar.getJarEntry(name + ".class"))) {
            new ClassReader(input).accept(node, 0);
        }
        return node;
    }

    private static List<AnnotationNode> annotations(List<AnnotationNode> visible, List<AnnotationNode> invisible) {
        List<AnnotationNode> out = new ArrayList<>();
        if (visible != null) out.addAll(visible);
        if (invisible != null) out.addAll(invisible);
        return out;
    }

    private static Object value(AnnotationNode annotation, String key) {
        for (int index = 0; index < annotation.values.size(); index += 2) {
            if (annotation.values.get(index).equals(key)) return annotation.values.get(index + 1);
        }
        throw new AssertionError("Missing annotation key " + key);
    }

    private static void require(boolean condition, String message) {
        if (!condition) throw new AssertionError(message);
    }
}
