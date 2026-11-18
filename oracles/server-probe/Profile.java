import java.nio.file.*;
import java.util.*;
import jdk.jfr.consumer.*;

// JFR execution-sample shares, Server thread only, inside each measured window.
public class Profile {
    private static final List<String> BUCKETS = List.of(
        "waypoints",
        "entity_tracking",
        "entity_push",
        "movement_collision",
        "advancements_inventory",
        "chunks_spawning",
        "save_io",
        "rest"
    );

    static String category(RecordedStackTrace stack) {
        if (stack == null) return "rest";
        List<String> classes = new ArrayList<>();
        List<String> methods = new ArrayList<>();
        for (var f : stack.getFrames()) {
            classes.add(f.getMethod().getType().getName());
            methods.add(f.getMethod().getName());
        }
        String joined = String.join("\n", classes) + "\n" + String.join("\n", methods);
        if (joined.contains("ServerWaypointManager") || joined.contains("TrackedWaypoint")) {
            return "waypoints";
        }
        for (int i = 0; i < classes.size(); i++) {
            String c = classes.get(i);
            String m = methods.get(i);
            if (c.contains("ChunkMap$TrackedEntity") || c.endsWith("ServerEntity") ||
                c.endsWith("PlayerChunkSender") || c.endsWith("ChunkTrackingView") ||
                m.equals("updatePlayers") || m.equals("updatePlayer")) {
                return "entity_tracking";
            }
        }
        for (int i = 0; i < methods.size(); i++) {
            String m = methods.get(i);
            if (m.equals("pushEntities") || m.equals("getPushableEntities") || m.equals("doPush")) {
                return "entity_push";
            }
        }
        for (int i = 0; i < classes.size(); i++) {
            String c = classes.get(i);
            String m = methods.get(i);
            if (c.endsWith("ServerGamePacketListenerImpl") && m.equals("handleMovePlayer") ||
                c.endsWith("BlockCollisions") ||
                (c.endsWith("LivingEntity") || c.endsWith("Player") || c.endsWith("Entity") || c.endsWith("ServerPlayer")) &&
                (m.startsWith("travel") || m.equals("move") || m.equals("collide") || m.equals("aiStep"))) {
                return "movement_collision";
            }
        }
        for (String c : classes) {
            if (c.contains("Advancement") || c.contains("Inventory") || c.contains("ContainerMenu") ||
                c.contains("ServerRecipeBook") || c.contains("ServerStatsCounter")) {
                return "advancements_inventory";
            }
        }
        for (int i = 0; i < classes.size(); i++) {
            String c = classes.get(i);
            String m = methods.get(i);
            if (c.contains("PlayerList") && m.startsWith("save") || c.contains("NbtIo")) {
                return "save_io";
            }
        }
        for (int i = 0; i < classes.size(); i++) {
            String c = classes.get(i);
            String m = methods.get(i);
            if (c.startsWith("net.minecraft.world.level.chunk.") || c.startsWith("net.minecraft.world.level.levelgen.") ||
                c.endsWith("ServerChunkCache") || c.endsWith("ChunkMap") || c.endsWith("DistanceManager") ||
                c.endsWith("NaturalSpawner") || c.endsWith("ServerLevel") && m.equals("tickChunk")) {
                return "chunks_spawning";
            }
        }
        return "rest";
    }

    public static void main(String[] args) throws Exception {
        Path root = Path.of(args[0]);
        System.out.println("case,rate," + String.join(",", BUCKETS) + ",samples");
        try (var dirs = Files.list(root)) {
            for (Path dir : dirs.filter(Files::isDirectory).sorted().toList()) {
                Path jfr = dir.resolve("profile.jfr");
                if (!Files.exists(jfr) || Files.size(jfr) == 0) continue;
                Map<String, long[]> windows = new LinkedHashMap<>();
                for (String rate : List.of("20", "100")) {
                    Path w = dir.resolve(rate + ".window");
                    if (!Files.exists(w)) continue;
                    String[] a = Files.readString(w).trim().split(",");
                    windows.put(rate, new long[]{Long.parseLong(a[0]), Long.parseLong(a[1])});
                }
                Map<String, int[]> counts = new LinkedHashMap<>();
                for (String r : windows.keySet()) counts.put(r, new int[BUCKETS.size()]);
                try (var recording = new RecordingFile(jfr)) {
                    while (recording.hasMoreEvents()) {
                        var e = recording.readEvent();
                        if (!e.getEventType().getName().equals("jdk.ExecutionSample")) continue;
                        var thread = e.getThread("sampledThread");
                        if (thread == null || !"Server thread".equals(thread.getJavaName())) continue;
                        long time = e.getStartTime().toEpochMilli();
                        for (var w : windows.entrySet()) {
                            if (time >= w.getValue()[0] && time <= w.getValue()[1]) {
                                int idx = BUCKETS.indexOf(category(e.getStackTrace()));
                                counts.get(w.getKey())[idx]++;
                            }
                        }
                    }
                }
                for (var entry : counts.entrySet()) {
                    int[] a = entry.getValue();
                    int n = Arrays.stream(a).sum();
                    StringBuilder sb = new StringBuilder();
                    sb.append(dir.getFileName()).append(',').append(entry.getKey());
                    for (int v : a) {
                        sb.append(',').append(String.format(Locale.ROOT, "%.2f", n == 0 ? 0.0 : 100.0 * v / n));
                    }
                    sb.append(',').append(n);
                    System.out.println(sb);
                }
            }
        }
    }
}
