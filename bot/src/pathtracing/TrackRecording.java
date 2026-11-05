package pathtracing;

import java.io.BufferedWriter;
import java.io.IOException;
import java.io.StringWriter;
import java.nio.ByteBuffer;
import java.nio.channels.FileChannel;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardOpenOption;
import java.util.HashMap;
import java.util.Map;

/** Discarded starts only open, close and delete a reservation; bytes stay in memory. */
final class TrackRecording {
    private static final Map<Path, Long> NEXT = new HashMap<>();
    final Path path;
    final Path temporary;
    private final FileChannel reservation;
    private final StringWriter buffer = new StringWriter();
    final BufferedWriter writer = new BufferedWriter(buffer);

    private TrackRecording(Path path, FileChannel reservation) {
        this.path = path;
        temporary = path.resolveSibling(path.getFileName() + ".tmp");
        this.reservation = reservation;
    }

    static TrackRecording open(Path directory, String prefix) throws IOException {
        Path key = directory.toAbsolutePath().normalize().resolve(prefix);
        if (!NEXT.containsKey(key)) {
            Files.createDirectories(directory);
            long next = 1;
            try (var files = Files.list(directory)) {
                for (Path file : files.toList()) {
                    String name = file.getFileName().toString();
                    if (!name.startsWith(prefix)) continue;
                    String tail = name.substring(prefix.length());
                    if (!tail.matches("[0-9]+\\.jsonl(\\.tmp)?")) continue;
                    next = Math.max(next, Math.addExact(Long.parseLong(tail.substring(0, tail.indexOf('.'))), 1));
                }
            }
            NEXT.put(key, next);
        }
        for (;;) {
            long index = NEXT.get(key);
            NEXT.put(key, Math.addExact(index, 1));
            Path path = directory.resolve(prefix + index + ".jsonl");
            try {
                return new TrackRecording(path, FileChannel.open(path.resolveSibling(path.getFileName() + ".tmp"),
                    StandardOpenOption.CREATE_NEW, StandardOpenOption.WRITE));
            } catch (java.nio.file.FileAlreadyExistsException collision) {
                // A concurrent reservation consumes this index too.
            }
        }
    }

    void commit() throws IOException {
        writer.close();
        try (reservation) {
            ByteBuffer bytes = StandardCharsets.UTF_8.encode(buffer.toString());
            while (bytes.hasRemaining()) reservation.write(bytes);
            reservation.force(true);
        }
        Files.move(temporary, path);
    }

    void discard() throws IOException {
        try { writer.close(); }
        finally {
            try { reservation.close(); }
            finally { Files.deleteIfExists(temporary); }
        }
    }
}
