import net.fabricmc.api.EnvType;
import net.fabricmc.loader.impl.launch.knot.Knot;

final class TransformCheck {
    public static void main(String[] args) throws Exception {
        // Apply the required injections and link the transformed targets. No game main.
        var loader = new Knot(EnvType.SERVER).init(new String[0]);
        for (String target : new String[] {"net.minecraft.world.entity.Entity", "net.minecraft.server.level.ServerPlayer"}) {
            Class<?> type = Class.forName(target, false, loader);
            boolean injected = java.util.Arrays.stream(type.getDeclaredMethods())
                .anyMatch(method -> method.getName().contains("contactBefore"));
            if (!injected) throw new AssertionError("Missing contact injections in " + target);
            System.err.println("PASS Fabric transformed " + target);
        }
    }
}
