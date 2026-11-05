package pathtracing;

import net.fabricmc.api.ClientModInitializer;

/** One jar for both clients; the launcher's -Dpathtracing.role picks which half runs. */
public final class PathTracingMod implements ClientModInitializer {
    public static final String BOT = System.getProperty("pathtracing.bot", "PathBot");
    public static final String OWNER = System.getProperty("pathtracing.owner", "Brain_Forge");

    @Override
    public void onInitializeClient() {
        Tracks.register();
        switch (System.getProperty("pathtracing.role", "")) {
            case "player" -> Pointer.register();
            case "bot" -> Bot.register();
            default -> { }
        }
        Recorder.register();
    }
}
