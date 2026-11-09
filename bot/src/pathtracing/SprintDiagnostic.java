package pathtracing;

import net.minecraft.client.Minecraft;
import net.minecraft.resources.Identifier;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.entity.ai.attributes.Attributes;
import org.slf4j.LoggerFactory;

/** Disposable, opt-in observation of the sprint flag and attribute around movement. */
public final class SprintDiagnostic {
    public static final boolean ENABLED = "1".equals(System.getenv("PATHTRACING_SPRINT_DIAGNOSTIC"))
        && "bot".equals(System.getProperty("pathtracing.role"));
    private static final Identifier SPRINT = Identifier.parse("minecraft:sprinting");

    public static void observe(String phase, Entity entity) {
        if (!ENABLED) return;
        var player = Minecraft.getInstance().player;
        if (player == null || entity != player) return;
        var attribute = player.getAttribute(Attributes.MOVEMENT_SPEED);
        LoggerFactory.getLogger("SprintDiagnostic").info("{} playerTick={} sprint={} modifier={} speedBits={}",
            phase, player.tickCount, player.isSprinting(), attribute.getModifier(SPRINT) != null,
            Integer.toHexString(Float.floatToRawIntBits(player.getSpeed())));
    }
}
