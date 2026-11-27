package pathtracing;

import com.mojang.brigadier.CommandDispatcher;
import com.mojang.brigadier.arguments.StringArgumentType;
import com.mojang.brigadier.builder.LiteralArgumentBuilder;
import com.mojang.brigadier.builder.RequiredArgumentBuilder;
import java.io.IOException;
import java.nio.file.Path;
import java.util.List;
import java.util.TreeSet;
import java.util.function.ToIntFunction;

/** Actual command trees, independent of the Minecraft client so the gate can dispatch them. */
final class RaceCommands {
    static <S> void tracks(CommandDispatcher<S> dispatcher, TrackStore store, ToIntFunction<String> execute) {
        var root = LiteralArgumentBuilder.<S>literal("track");
        root.then(LiteralArgumentBuilder.<S>literal("new").then(
            RequiredArgumentBuilder.<S, String>argument("name", StringArgumentType.word())
                .executes(c -> execute.applyAsInt("new " + StringArgumentType.getString(c, "name")))));
        for (String action : List.of("delete", "best", "kit", "run", "play")) {
            root.then(LiteralArgumentBuilder.<S>literal(action).then(
                RequiredArgumentBuilder.<S, String>argument("name", StringArgumentType.word())
                    .suggests((c, b) -> {
                        try { return NameSuggestions.suggest(b, store.list().stream().map(TrackStore.Track::name).toList()); }
                        catch (IOException failure) { return b.buildFuture(); }
                    }).executes(c -> execute.applyAsInt(action + " " + StringArgumentType.getString(c, "name")))));
        }
        for (String action : List.of("list", "cancel", "exit", "restore")) {
            root.then(LiteralArgumentBuilder.<S>literal(action).executes(c -> execute.applyAsInt(action)));
        }
        dispatcher.register(root);
    }

    static <S> void maps(CommandDispatcher<S> dispatcher, Path registry, ToIntFunction<String> execute) {
        dispatcher.register(LiteralArgumentBuilder.<S>literal("map").executes(c -> execute.applyAsInt(null)).then(
            RequiredArgumentBuilder.<S, String>argument("name", StringArgumentType.word())
                .suggests((c, b) -> {
                    var names = new TreeSet<String>();
                    names.add("overworld");
                    names.addAll(Maps.worlds().keySet());
                    try { names.addAll(MapRegistry.read(registry).keySet()); }
                    catch (IOException failure) { return NameSuggestions.suggest(b, names); }
                    return NameSuggestions.suggest(b, names);
                }).executes(c -> execute.applyAsInt(StringArgumentType.getString(c, "name")))));
    }
}
