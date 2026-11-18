package pathtracing.server;

import java.io.IOException;
import java.nio.file.Path;
import net.fabricmc.api.DedicatedServerModInitializer;
import net.fabricmc.fabric.api.command.v2.CommandRegistrationCallback;
import net.fabricmc.fabric.api.event.lifecycle.v1.ServerChunkEvents;
import net.fabricmc.fabric.api.event.lifecycle.v1.ServerLifecycleEvents;
import net.fabricmc.fabric.api.event.lifecycle.v1.ServerTickEvents;
import net.minecraft.core.BlockPos;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.gamerules.GameRules;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/** Dedicated server mod connecting Mojang 26.2 directly to Rust `mind.exe` over shared memory. */
public final class ServerMindMod implements DedicatedServerModInitializer {
    private static final Logger LOGGER = LoggerFactory.getLogger("pathtracing_server");

    private static ServerMindMod instance;
    private final WorldBridge worldBridge = new WorldBridge();
    private final SwarmManager swarmManager = new SwarmManager(worldBridge);
    private MinecraftServer server;
    private ShmBridge bridge;

    public static SwarmManager manager() {
        return instance != null ? instance.swarmManager : null;
    }

    public static void onBlockDelta(ServerLevel level, BlockPos pos, BlockState state) {
        if (instance != null) {
            MinecraftServer srv = level.getServer();
            if (srv != null) {
                instance.worldBridge.queueBlockDelta(srv, level, pos, state);
            }
        }
    }

    @Override
    public void onInitializeServer() {
        instance = this;

        CommandRegistrationCallback.EVENT.register((dispatcher, registryAccess, environment) ->
            swarmManager.registerCommands(dispatcher)
        );

        ServerLifecycleEvents.SERVER_STARTING.register(srv -> this.server = srv);

        ServerLifecycleEvents.SERVER_STARTED.register(srv -> {
            this.server = srv;
            srv.overworld().getGameRules().set(GameRules.PLAYERS_SLEEPING_PERCENTAGE, 0, srv);
            Path shmPath = Path.of(System.getProperty("pathtracing.shm", "D:\\path-tracing-run\\server-mind.shm"));
            try {
                this.bridge = ShmBridge.open(shmPath);
                swarmManager.attach(srv, bridge);
                LOGGER.info("PathTracing ServerMind shm opened at {}", shmPath);
            } catch (IOException ex) {
                throw new RuntimeException("Failed to open ServerMind shm: " + shmPath, ex);
            }
        });

        ServerLifecycleEvents.SERVER_STOPPING.register(srv -> {
            if (bridge != null) {
                try {
                    bridge.close();
                } catch (IOException ignored) {
                }
                bridge = null;
            }
        });

        ServerChunkEvents.CHUNK_LOAD.register((level, chunk, generated) -> {
            MinecraftServer srv = level.getServer();
            if (srv != null) {
                worldBridge.onChunkLoad(srv, level, chunk, bridge);
            }
        });

        ServerChunkEvents.CHUNK_UNLOAD.register((level, chunk) -> {
            MinecraftServer srv = level.getServer();
            if (srv != null) {
                worldBridge.onChunkUnload(srv, level, chunk, bridge);
            }
        });

        ServerTickEvents.START_SERVER_TICK.register(swarmManager::onServerTickStart);
        ServerTickEvents.END_SERVER_TICK.register(srv -> swarmManager.onServerTickEnd());
    }
}
