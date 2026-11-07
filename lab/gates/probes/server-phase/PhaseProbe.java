package pathtracing.phase;

import java.util.ArrayList;
import java.util.List;
import net.fabricmc.api.DedicatedServerModInitializer;
import net.fabricmc.fabric.api.command.v2.CommandRegistrationCallback;
import net.fabricmc.fabric.api.event.lifecycle.v1.ServerTickEvents;
import net.minecraft.commands.Commands;
import net.minecraft.commands.arguments.EntityArgument;
import com.mojang.brigadier.arguments.StringArgumentType;
import net.minecraft.network.chat.Component;
import net.minecraft.server.level.ServerPlayer;

public final class PhaseProbe implements DedicatedServerModInitializer {
    private static final int SAMPLES = 45;
    private final List<String> samples = new ArrayList<>(SAMPLES);
    private ServerPlayer observed;

    @Override
    public void onInitializeServer() {
        if (LandingTrace.ENABLED) CommandRegistrationCallback.EVENT.register((dispatcher, access, environment) ->
            dispatcher.register(Commands.literal("ptlanding")
                .requires(Commands.hasPermission(Commands.LEVEL_OWNERS))
                .then(Commands.literal("start").then(Commands.argument("player", EntityArgument.player())
                    .then(Commands.argument("run", StringArgumentType.word()).executes(context -> {
                        String run = StringArgumentType.getString(context, "run");
                        LandingTrace.start(EntityArgument.getPlayer(context, "player"), run);
                        context.getSource().sendSuccess(() -> Component.literal("Landing capture started " + run), false);
                        return 1;
                    }))))
                .then(Commands.literal("stop").then(Commands.argument("run", StringArgumentType.word()).executes(context -> {
                    String run = StringArgumentType.getString(context, "run");
                    LandingTrace.stop(run);
                    context.getSource().sendSuccess(() -> Component.literal("Landing capture sealed " + run), false);
                    return 1;
                })))
                .then(Commands.literal("cancel").executes(context -> {
                    LandingTrace.cancel();
                    return 1;
                }))));
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
                }))
                .then(Commands.literal("resources")
                    .requires(source -> ResourceTrace.ENABLED)
                    .then(Commands.literal("start").then(Commands.argument("player", EntityArgument.player())
                        .then(Commands.argument("run", StringArgumentType.word()).executes(context -> {
                            String run = StringArgumentType.getString(context, "run");
                            ResourceTrace.start(EntityArgument.getPlayer(context, "player"), run);
                            context.getSource().sendSuccess(() -> Component.literal("Resource capture started " + run), false);
                            return 1;
                        }))))
                    .then(Commands.literal("stop").then(Commands.argument("run", StringArgumentType.word()).executes(context -> {
                        String run = StringArgumentType.getString(context, "run");
                        ResourceTrace.stop(run);
                        context.getSource().sendSuccess(() -> Component.literal("Resource capture saved " + run), false);
                        return 1;
                    })))
                    .then(Commands.literal("cancel").executes(context -> {
                        ResourceTrace.cancel();
                        return 1;
                    })))));
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
