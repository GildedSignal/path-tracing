package pathtracing;

import com.mojang.brigadier.suggestion.Suggestions;
import com.mojang.brigadier.suggestion.SuggestionsBuilder;
import java.util.concurrent.CompletableFuture;

final class NameSuggestions {
    static CompletableFuture<Suggestions> suggest(SuggestionsBuilder builder, Iterable<String> names) {
        for (String name : names) if (name.startsWith(builder.getRemaining())) builder.suggest(name);
        return builder.buildFuture();
    }
}
