import java.util.ArrayList;
import java.util.List;
import net.fabricmc.api.DedicatedServerModInitializer;
import net.fabricmc.fabric.api.command.v2.CommandRegistrationCallback;
import net.fabricmc.fabric.api.event.lifecycle.v1.ServerTickEvents;
import net.minecraft.commands.Commands;
import net.minecraft.commands.arguments.EntityArgument;
import net.minecraft.network.chat.Component;
import net.minecraft.server.level.ServerPlayer;

public final class PhaseProbe implements DedicatedServerModInitializer {
    private static final int SAMPLES = 45;
    private final List<String> samples = new ArrayList<>(SAMPLES);
    private ServerPlayer observed;

    @Override
    public void onInitializeServer() {
        CommandRegistrationCallback.EVENT.register((dispatcher, access, environment) ->
            dispatcher.register(Commands.literal("ptphase")
                .requires(Commands.hasPermission(Commands.LEVEL_OWNERS))
                .then(Commands.literal("start")
                    .then(Commands.argument("player", EntityArgument.player()).executes(context -> {
                        observed = EntityArgument.getPlayer(context, "player");
                        samples.clear();
                        context.getSource().sendSuccess(() -> Component.literal("Phase capture started"), false);
                        return 1;
                    })))
                .then(Commands.literal("read").executes(context -> {
                    context.getSource().sendSuccess(() -> Component.literal(String.join(";", samples)), false);
                    return samples.size();
                }))));
        ServerTickEvents.END_SERVER_TICK.register(server -> {
            if (observed == null) return;
            if (observed.isRemoved()) {
                observed = null;
                return;
            }
            var food = observed.getFoodData();
            samples.add(observed.level().getGameTime() + "," + observed.tickCount + ","
                + food.getFoodLevel() + "," + Integer.toHexString(Float.floatToRawIntBits(food.getSaturationLevel())));
            if (samples.size() == SAMPLES) observed = null;
        });
    }
}
