package pathtracing;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import java.io.IOException;
import java.nio.ByteBuffer;
import java.nio.channels.FileChannel;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardOpenOption;
import java.nio.file.StandardCopyOption;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.util.HexFormat;
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

    private Path kitFile(String name) {
        validateName(name);
        return directory.resolve(name + ".kit.json");
    }

    TrackInventory readKit(String name) throws IOException {
        Path path = kitFile(name);
        return Files.exists(path) ? TrackInventory.read(JsonParser.parseString(Files.readString(path)).getAsJsonObject()) : null;
    }

    void writeKit(String name, TrackInventory kit) throws IOException {
        try (var channel = lockChannel(); var lock = channel.lock()) {
            durableReplace(kitFile(name), canonical(kit.withoutSticks().json()) + "\n");
        }
    }

    record Session(TrackInventory inventory, String gameMode) {
        Session {
            if (!List.of("survival", "creative", "adventure", "spectator").contains(gameMode)) {
                throw new IllegalArgumentException("Invalid saved game mode.");
            }
        }

        JsonObject json() {
            JsonObject out = new JsonObject();
            out.add("inventory", inventory.json());
            out.addProperty("gameMode", gameMode);
            return out;
        }

        static Session read(JsonObject in) {
            return new Session(TrackInventory.read(in.getAsJsonObject("inventory")), in.get("gameMode").getAsString());
        }
    }

    private Path sessionFile(String player) {
        if (!player.matches("[A-Za-z0-9_]{1,16}")) throw new IllegalArgumentException("Invalid session player name.");
        return directory.resolve(".session-" + player + ".json");
    }

    Session readSession(String player) throws IOException {
        Path path = sessionFile(player);
        if (!Files.exists(path)) return null;
        return Session.read(JsonParser.parseString(Files.readString(path)).getAsJsonObject());
    }

    void writeSession(String player, Session session) throws IOException {
        try (var channel = lockChannel(); var lock = channel.lock()) {
            Path path = sessionFile(player);
            if (Files.exists(path)) throw new IOException("Saved track session exists. Use /track restore to recover it first.");
            durableReplace(path, canonical(session.json()) + "\n");
        }
    }

    void deleteSession(String player) throws IOException { Files.delete(sessionFile(player)); }

    private static void durableReplace(Path target, String contents) throws IOException {
        Path temporary = Files.createTempFile(target.getParent(), target.getFileName() + ".", ".tmp");
        try {
            try (var output = FileChannel.open(temporary, StandardOpenOption.WRITE)) {
                ByteBuffer bytes = StandardCharsets.UTF_8.encode(contents);
                while (bytes.hasRemaining()) output.write(bytes);
                output.force(true);
            }
            Files.move(temporary, target, StandardCopyOption.ATOMIC_MOVE, StandardCopyOption.REPLACE_EXISTING);
        } finally { Files.deleteIfExists(temporary); }
    }

    /** Sorted object keys, fixed array order, compact Gson primitives, UTF-8 and no trailing newline. */
    static String canonical(JsonElement value) {
        if (value.isJsonObject()) {
            Map<String, JsonElement> sorted = new TreeMap<>();
            value.getAsJsonObject().entrySet().forEach(e -> sorted.put(e.getKey(), e.getValue()));
            return "{" + String.join(",", sorted.entrySet().stream()
                .map(e -> new com.google.gson.JsonPrimitive(e.getKey()) + ":" + canonical(e.getValue())).toList()) + "}";
        }
        if (value.isJsonArray()) {
            List<String> values = new ArrayList<>();
            value.getAsJsonArray().forEach(e -> values.add(canonical(e)));
            return "[" + String.join(",", values) + "]";
        }
        return value.toString();
    }

    static String sha256(JsonElement value) {
        try { return HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(canonical(value).getBytes(StandardCharsets.UTF_8))); }
        catch (NoSuchAlgorithmException impossible) { throw new AssertionError(impossible); }
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
