package pathtracing;

import com.google.gson.GsonBuilder;
import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import java.io.DataInputStream;
import java.io.IOException;
import java.io.InputStream;
import java.io.RandomAccessFile;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.HashMap;
import java.util.Map;
import java.util.zip.GZIPInputStream;
import java.util.zip.InflaterInputStream;
import java.util.zip.ZipFile;
import net.minecraft.SharedConstants;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Direction;
import net.minecraft.nbt.CompoundTag;
import net.minecraft.nbt.NbtAccounter;
import net.minecraft.nbt.NbtIo;
import net.minecraft.nbt.NbtOps;
import net.minecraft.server.Bootstrap;
import net.minecraft.util.datafix.DataFixTypes;
import net.minecraft.util.datafix.DataFixers;
import net.minecraft.world.level.BlockGetter;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.entity.BlockEntity;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.material.FluidState;

/** Offline column audit. Every source and region handle is opened read-only. */
public final class MapSpawnAudit {
    private static CompoundTag sourceLevel(Path source) throws IOException {
        if (Files.isDirectory(source)) return NbtIo.readCompressed(source.resolve("level.dat"), NbtAccounter.unlimitedHeap());
        try (var zip = new ZipFile(source.toFile())) {
            var levels = zip.stream().filter(e -> !e.isDirectory() && (e.getName().equals("level.dat") || e.getName().endsWith("/level.dat"))).toList();
            if (levels.size() != 1) throw new IOException("Expected one level.dat in " + source);
            try (var input = zip.getInputStream(levels.getFirst())) {
                return NbtIo.readCompressed(input, NbtAccounter.unlimitedHeap());
            }
        }
    }

    static boolean standable(BlockGetter world, int x, int y, int z) {
        var feet = new BlockPos(x, y, z);
        var head = feet.above();
        var below = feet.below();
        return world.getBlockState(feet).getCollisionShape(world, feet).isEmpty()
            && world.getBlockState(head).getCollisionShape(world, head).isEmpty()
            && world.getFluidState(feet).isEmpty() && world.getFluidState(head).isEmpty()
            && Block.isFaceFull(world.getBlockState(below).getCollisionShape(world, below), Direction.UP);
    }

    static int[] checked(BlockGetter world, int[] spawn) {
        int x = spawn[0], y = spawn[1], z = spawn[2];
        if (y >= -63 && y <= 320 && standable(world, x, y, z)) return spawn.clone();
        // Nearest vertical distance; upwards wins an equal-distance tie.
        int radius = Math.max(Math.abs(320 - y), Math.abs(-63 - y));
        for (int d = 1; d <= radius; d++) {
            int up = y + d, down = y - d;
            if (up >= -63 && up <= 320 && standable(world, x, up, z)) return new int[] {x, up, z};
            if (down >= -63 && down <= 320 && standable(world, x, down, z)) return new int[] {x, down, z};
        }
        return null;
    }

    static final class Dimension implements BlockGetter {
        private final Path region;
        private final Map<Long, Map<Integer, BlockState[]>> chunks = new HashMap<>();
        final JsonObject inputs = new JsonObject();
        Dimension(Path dimension) { region = dimension.resolve("region"); }
        @Override public int getHeight() { return 384; }
        @Override public int getMinY() { return -64; }
        @Override public BlockEntity getBlockEntity(BlockPos p) { return null; }
        @Override public FluidState getFluidState(BlockPos p) { return getBlockState(p).getFluidState(); }
        @Override public BlockState getBlockState(BlockPos p) {
            if (p.getY() < -64 || p.getY() >= 320) return Blocks.VOID_AIR.defaultBlockState();
            int cx = Math.floorDiv(p.getX(), 16), cz = Math.floorDiv(p.getZ(), 16);
            long key = ((long) cx << 32) | (cz & 0xffffffffL);
            Map<Integer, BlockState[]> sections = chunks.computeIfAbsent(key, ignored -> {
                try { return load(cx, cz); } catch (IOException error) { throw new java.io.UncheckedIOException(error); }
            });
            var section = sections.get(Math.floorDiv(p.getY(), 16));
            if (section == null) return Blocks.AIR.defaultBlockState();
            return section[Math.floorMod(p.getY(), 16) * 256 + Math.floorMod(p.getZ(), 16) * 16 + Math.floorMod(p.getX(), 16)];
        }
        private Map<Integer, BlockState[]> load(int x, int z) throws IOException {
            Path file = region.resolve("r." + Math.floorDiv(x, 32) + "." + Math.floorDiv(z, 32) + ".mca");
            inputs.addProperty(file.toString(), sha256(file));
            CompoundTag chunk;
            try (var input = new RandomAccessFile(file.toFile(), "r")) {
                input.seek((Math.floorMod(x, 32) + 32L * Math.floorMod(z, 32)) * 4);
                int location = input.readInt();
                if (location == 0) throw new IOException("Missing chunk " + x + "," + z + " in " + file);
                input.seek((long) (location >>> 8) * 4096);
                int length = input.readInt();
                int compression = input.readUnsignedByte();
                byte[] bytes;
                if ((compression & 128) != 0) bytes = Files.readAllBytes(region.resolve("c." + x + "." + z + ".mcc"));
                else {
                    if (length < 1 || length + 4 > (location & 255) * 4096) throw new IOException("Invalid Anvil length in " + file);
                    bytes = new byte[length - 1]; input.readFully(bytes);
                }
                InputStream payload = new java.io.ByteArrayInputStream(bytes);
                payload = switch (compression & 127) {
                    case 1 -> new GZIPInputStream(payload);
                    case 2 -> new InflaterInputStream(payload);
                    case 3 -> payload;
                    default -> throw new IOException("Unsupported compression " + compression + " in " + file);
                };
                try (var data = new DataInputStream(payload)) { chunk = NbtIo.read(data, NbtAccounter.unlimitedHeap()); }
            }
            int version = chunk.getIntOr("DataVersion", -1);
            if (version < SharedConstants.getCurrentVersion().dataVersion().version()) {
                chunk = DataFixTypes.CHUNK.updateToCurrentVersion(DataFixers.getDataFixer(), chunk, version);
            }
            var sections = new HashMap<Integer, BlockState[]>();
            for (var tag : chunk.getListOrEmpty("sections")) {
                var section = (CompoundTag) tag;
                var states = section.getCompound("block_states");
                if (states.isEmpty()) continue;
                var paletteTag = states.get().getListOrEmpty("palette");
                BlockState[] palette = new BlockState[paletteTag.size()];
                for (int i = 0; i < palette.length; i++) palette[i] = BlockState.CODEC.parse(NbtOps.INSTANCE, paletteTag.get(i)).getOrThrow();
                var values = new BlockState[4096];
                long[] packed = states.get().getLongArray("data").orElse(new long[0]);
                int bits = Math.max(4, 32 - Integer.numberOfLeadingZeros(palette.length - 1));
                int perLong = 64 / bits;
                long mask = (1L << bits) - 1;
                for (int i = 0; i < values.length; i++) values[i] = palette[palette.length == 1 ? 0 : (int) ((packed[i / perLong] >>> ((i % perLong) * bits)) & mask)];
                sections.put(section.getByteOr("Y", (byte) 0) * 1, values);
            }
            if (sections.isEmpty()) throw new IOException("No block sections in chunk " + x + "," + z);
            return sections;
        }
    }

    private static String sha256(Path file) throws IOException {
        try {
            var digest = java.security.MessageDigest.getInstance("SHA-256");
            try (var input = Files.newInputStream(file)) {
                byte[] buffer = new byte[65536];
                for (int n; (n = input.read(buffer)) != -1;) digest.update(buffer, 0, n);
            }
            return java.util.HexFormat.of().formatHex(digest.digest());
        } catch (java.security.NoSuchAlgorithmException impossible) { throw new AssertionError(impossible); }
    }

    public static void main(String[] args) throws Exception {
        SharedConstants.tryDetectVersion(); Bootstrap.bootStrap();
        Path rig = Path.of(args[0]), output = Path.of(args[1]), recordFile = Path.of(args[2]);
        if (output.toAbsolutePath().normalize().startsWith(rig.toAbsolutePath().normalize())
            || recordFile.toAbsolutePath().normalize().startsWith(rig.toAbsolutePath().normalize())) throw new IOException("Outputs must be outside the rig");
        var maps = JsonParser.parseString(Files.readString(rig.resolve("maps.json"))).getAsJsonArray();
        MapRegistry.read(rig.resolve("maps.json"));
        JsonArray rows = new JsonArray();
        for (var entry : maps) {
            var map = entry.getAsJsonObject();
            String name = map.get("name").getAsString();
            var level = sourceLevel(Path.of(map.get("source").getAsString()));
            int[] spawn = MapSpawn.read(level);
            var dimension = new Dimension(rig.resolve("server/world/dimensions/pathtracing/" + name));
            int[] checked = checked(dimension, spawn);
            JsonObject row = new JsonObject();
            row.addProperty("name", name); row.add("old", map.get("spawn").deepCopy());
            row.add("sourceSpawn", new com.google.gson.Gson().toJsonTree(spawn));
            row.addProperty("sourceSpawnData", level.getCompoundOrEmpty("Data").getCompoundOrEmpty("spawn").toString());
            row.addProperty("source", map.get("source").getAsString());
            row.addProperty("registrySha256", sha256(rig.resolve("maps.json")));
            row.add("regionsSha256", dimension.inputs);
            row.addProperty("status", checked == null ? "NO_STANDABLE_CELL" : "CHECKED");
            if (checked != null) map.add("spawn", new com.google.gson.Gson().toJsonTree(checked));
            row.add("new", map.get("spawn").deepCopy()); rows.add(row);
            System.out.println(row);
        }
        var gson = new GsonBuilder().setPrettyPrinting().disableHtmlEscaping().create();
        Files.writeString(output, gson.toJson(maps) + "\n");
        Files.writeString(recordFile, gson.toJson(rows) + "\n");
    }
}
