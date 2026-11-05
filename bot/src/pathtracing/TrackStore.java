package pathtracing;

import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import java.io.IOException;
import java.nio.ByteBuffer;
import java.nio.channels.FileChannel;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardOpenOption;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.TreeMap;

/** Shared rig files. Run readers and appenders use the same cross-process lock. */
final class TrackStore {
    record Cell(int x, int y, int z) {
        JsonArray json() {
            JsonArray out = new JsonArray();
            out.add(x);
            out.add(y);
            out.add(z);
            return out;
        }

        static Cell read(JsonArray in) {
            if (in == null || in.size() != 3) throw new IllegalArgumentException("Expected three block coordinates.");
            return new Cell(in.get(0).getAsBigDecimal().intValueExact(),
                in.get(1).getAsBigDecimal().intValueExact(), in.get(2).getAsBigDecimal().intValueExact());
        }
    }

    record Track(String name, Cell a, Cell b, String dimension, String created, String creator) { }

    final Path rig;
    private final Path directory;

    TrackStore(Path rig) {
        this.rig = rig;
        directory = rig.resolve("lab/tracks");
    }

    static void validateName(String name) {
        if (!name.matches("[a-z0-9-]+")) throw new IllegalArgumentException("Track names use [a-z0-9-]+.");
    }

    private Path file(String name) {
        validateName(name);
        return directory.resolve(name + ".json");
    }

    void create(Track track) throws IOException {
        if (track.a.equals(track.b)) throw new IllegalArgumentException("B must differ from A.");
        JsonObject out = new JsonObject();
        out.add("A", track.a.json());
        out.add("B", track.b.json());
        out.addProperty("dimension", track.dimension);
        out.addProperty("created", track.created);
        out.addProperty("creator", track.creator);
        Files.createDirectories(directory);
        // Publish a complete definition atomically. The name is reserved under the shared lock.
        try (var channel = lockChannel(); var lock = channel.lock()) {
            if (Files.exists(file(track.name))) throw new IOException("Track already exists: " + track.name);
            publish(file(track.name), out + "\n");
        }
    }

    static void publish(Path target, String contents) throws IOException {
        Path temporary = Files.createTempFile(target.getParent(), target.getFileName() + ".", ".tmp");
        try {
            Files.writeString(temporary, contents, StandardCharsets.UTF_8);
            Files.move(temporary, target);
        } finally {
            Files.deleteIfExists(temporary);
        }
    }

    Track read(String name) throws IOException {
        JsonObject in = JsonParser.parseString(Files.readString(file(name))).getAsJsonObject();
        return new Track(name, Cell.read(in.getAsJsonArray("A")), Cell.read(in.getAsJsonArray("B")),
            in.get("dimension").getAsString(), in.get("created").getAsString(), in.get("creator").getAsString());
    }

    List<Track> list() throws IOException {
        if (!Files.isDirectory(directory)) return List.of();
        List<Track> out = new ArrayList<>();
        try (var paths = Files.list(directory)) {
            for (Path path : paths.filter(p -> p.getFileName().toString().matches("[a-z0-9-]+\\.json")).sorted().toList()) {
                String name = path.getFileName().toString();
                out.add(read(name.substring(0, name.length() - 5)));
            }
        }
        return out;
    }

    void delete(String name) throws IOException {
        validateName(name);
        try (var channel = lockChannel(); var lock = channel.lock()) {
            Files.delete(file(name));
        }
    }

    private FileChannel lockChannel() throws IOException {
        Files.createDirectories(directory);
        return FileChannel.open(directory.resolve(".lock"), StandardOpenOption.CREATE, StandardOpenOption.WRITE);
    }

    private Path runs(String name) {
        validateName(name);
        return directory.resolve("runs").resolve(name + ".jsonl");
    }

    private List<JsonObject> readRuns(String name) throws IOException {
        if (!Files.exists(runs(name))) return List.of();
        List<JsonObject> out = new ArrayList<>();
        for (String line : Files.readAllLines(runs(name), StandardCharsets.UTF_8)) {
            out.add(JsonParser.parseString(line).getAsJsonObject());
        }
        return out;
    }

    List<JsonObject> history(String name) throws IOException {
        validateName(name);
        try (var channel = lockChannel(); var lock = channel.lock()) {
            return readRuns(name);
        }
    }

    List<JsonObject> history(Track track) throws IOException {
        return history(track.name).stream().filter(run -> run.get("trackCreated").getAsString().equals(track.created)).toList();
    }

    long append(String name, JsonObject run) throws IOException {
        validateName(name);
        try (var channel = lockChannel(); var lock = channel.lock()) {
            long best = run.get("T").getAsLong();
            for (JsonObject previous : readRuns(name)) {
                if (previous.get("trackCreated").equals(run.get("trackCreated"))
                    && previous.get("player").equals(run.get("player"))
                    && previous.get("gameMode").equals(run.get("gameMode"))) best = Math.min(best, previous.get("T").getAsLong());
            }
            Files.createDirectories(runs(name).getParent());
            try (var output = FileChannel.open(runs(name), StandardOpenOption.CREATE, StandardOpenOption.WRITE)) {
                long size = output.size();
                output.position(size);
                try {
                    ByteBuffer bytes = StandardCharsets.UTF_8.encode(run + "\n");
                    while (bytes.hasRemaining()) output.write(bytes);
                    output.force(true);
                } catch (IOException failure) {
                    output.truncate(size);
                    throw failure;
                }
            }
            return best;
        }
    }

    record Saved(long best, Path unsaved, Exception failure) { }

    Saved save(String name, JsonObject run) throws IOException {
        try { return new Saved(append(name, run), null, null); }
        catch (IOException | RuntimeException failure) {
            Files.createDirectories(runs(name).getParent());
            Path fallback = Files.createTempFile(runs(name).getParent(),
                name + ".unsaved-" + System.currentTimeMillis() + "-", ".json");
            try (var output = FileChannel.open(fallback, StandardOpenOption.WRITE)) {
                ByteBuffer bytes = StandardCharsets.UTF_8.encode(run + "\n");
                while (bytes.hasRemaining()) output.write(bytes);
                output.force(true);
            } catch (IOException | RuntimeException fallbackFailure) {
                fallbackFailure.addSuppressed(failure);
                throw fallbackFailure;
            }
            return new Saved(run.get("T").getAsLong(), fallback, failure);
        }
    }

    static List<JsonObject> bests(List<JsonObject> history, String bot, boolean bots) {
        Map<String, JsonObject> best = new TreeMap<>();
        for (var run : history) {
            String player = run.get("player").getAsString();
            if (player.equals(bot) != bots) continue;
            String key = player + ":" + run.get("gameMode").getAsString();
            if (!best.containsKey(key) || run.get("T").getAsLong() < best.get(key).get("T").getAsLong()) best.put(key, run);
        }
        return List.copyOf(best.values());
    }

    JsonObject humanBest(Track track, String bot) throws IOException {
        return history(track).stream().filter(r -> !r.get("player").getAsString().equals(bot))
            .min(java.util.Comparator.comparingLong(r -> r.get("T").getAsLong())).orElse(null);
    }

    Map<String, Integer> toolkit(Track track, String owner) throws IOException {
        Map<String, Integer> out = new TreeMap<>();
        for (JsonObject run : history(track)) {
            if (!run.get("player").getAsString().equals(owner)) continue;
            for (var item : run.getAsJsonObject("itemsUsed").entrySet()) {
                if (!item.getKey().matches("[a-z0-9_.-]+:[a-z0-9_./-]+")) throw new IOException("Invalid item id in run.");
                int count = item.getValue().getAsBigDecimal().intValueExact();
                if (count <= 0) throw new IOException("Invalid item count in run.");
                out.merge(item.getKey(), count, Math::max);
            }
        }
        return out;
    }

    static JsonObject items(Map<String, Integer> items) {
        JsonObject out = new JsonObject();
        items.forEach(out::addProperty);
        return out;
    }
}
