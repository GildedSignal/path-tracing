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
            require(calls(read(mod, "pathtracing/Tracks"), "pathtracing/RaceCommands", "tracks"), "Track registration uses the tested command tree");
            require(calls(read(mod, "pathtracing/Maps"), "pathtracing/RaceCommands", "maps"), "Map registration uses the tested command tree");
            require(calls(read(mod, "pathtracing/Tracks"), "pathtracing/TrackHumanRace", "tick"), "Client tick uses the tested human lifecycle");
            var removed = read(game, "net/minecraft/server/level/ServerPlayer").methods.stream()
                .filter(m -> m.name.equals("onEffectsRemoved")).findFirst().orElseThrow();
            require(callIndex(removed, "net/minecraft/network/protocol/game/ClientboundRemoveMobEffectPacket", "<init>") >= 0,
                "Mojang sends removal packets for completed player effects");
            var effectTick = read(game, "net/minecraft/world/effect/MobEffectInstance").methods.stream()
                .filter(m -> m.name.equals("tickServer")).findFirst().orElseThrow();
            int apply = callIndex(effectTick, "net/minecraft/world/effect/MobEffect", "applyEffectTick");
            require(apply >= 0 && apply < callIndex(effectTick, "net/minecraft/world/effect/MobEffectInstance", "tickDownDuration"),
                "Effect application precedes expiry, the refill barrier");
            for (String name : List.of("TrackInteractionMixin", "TrackPlayerMixin", "TrackAttackMixin", "TrackItemMixin", "TrackProjectileMixin", "TrackBreakMixin", "ClientPacketListenerMixin")) {
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
                            // With local capture, the callback follows the target's arguments and captured locals come
                            // after it; Mixin's CAPTURE_FAILHARD checks those locals against the bytecode at load.
                            Type[] arguments = Type.getArgumentTypes(method.desc);
                            boolean capturesLocals = has(annotation, "locals");
                            int callbackIndex = capturesLocals ? arguments.length : parameters.length - 1;
                            require(callbackIndex < parameters.length
                                && parameters[callbackIndex].getInternalName().endsWith("/" + callback), name + ": incorrect callback for " + selector);
                            if (callbackIndex > 0 || capturesLocals) {
                                require(Arrays.equals(Arrays.copyOf(parameters, callbackIndex), arguments),
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
        require(injections == 36, "Expected 36 checked injections, got " + injections);
        System.out.println("Verified " + injections + " compiled track injections against Minecraft 26.2.");
    }

    private static boolean calls(ClassNode type, String owner, String name) {
        return type.methods.stream().anyMatch(method -> callIndex(method, owner, name) >= 0);
    }

    private static int callIndex(MethodNode method, String owner, String name) {
        for (int i = 0; i < method.instructions.size(); i++) {
            if (method.instructions.get(i) instanceof MethodInsnNode call && call.owner.equals(owner) && call.name.equals(name)) return i;
        }
        return -1;
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

    private static boolean has(AnnotationNode annotation, String key) {
        if (annotation.values == null) return false;
        for (int index = 0; index < annotation.values.size(); index += 2) {
            if (annotation.values.get(index).equals(key)) return true;
        }
        return false;
    }

    private static void require(boolean condition, String message) {
        if (!condition) throw new AssertionError(message);
    }
}
