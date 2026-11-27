import java.nio.file.*;
import net.minecraft.SharedConstants;
import net.minecraft.nbt.*;
import net.minecraft.server.Bootstrap;
import net.minecraft.util.datafix.*;
import net.minecraft.world.level.*;
import net.minecraft.world.level.chunk.storage.*;
import net.minecraft.world.level.storage.PrimaryLevelData;

// Offline upgrade of the repo's vanilla Overworld fixture into a new directory.
class UpgradeFixture {
    static CompoundTag upgrade(CompoundTag tag, DataFixTypes type) {
        int version = SharedConstants.getCurrentVersion().dataVersion().version();
        int from = tag.getInt("DataVersion").orElseThrow();
        var upgraded = type.update(DataFixers.getDataFixer(), tag, from, version);
        upgraded.putInt("DataVersion", version);
        return upgraded;
    }

    public static void main(String[] args) throws Exception {
        Path source = Path.of(args[0]).toRealPath();
        Path destination = Path.of(args[1]).toAbsolutePath().normalize();
        Files.createDirectory(destination);
        SharedConstants.tryDetectVersion();
        Bootstrap.bootStrap();
        var level = NbtIo.readCompressed(source.resolve("level.dat"), NbtAccounter.unlimitedHeap());
        var data = upgrade(level.getCompound("Data").orElseThrow(), DataFixTypes.LEVEL);
        PrimaryLevelData.writeVersionTag(data);
        level.put("Data", data);
        NbtIo.writeCompressed(level, destination.resolve("level.dat"));

        String savedData = "data/minecraft/";
        Files.createDirectories(destination.resolve(savedData));
        for (String name : new String[]{"world_gen_settings", "game_rules"}) {
            Path path = Path.of(savedData, name + ".dat");
            var raw = NbtIo.readCompressed(source.resolve(path), NbtAccounter.unlimitedHeap());
            var type = name.equals("world_gen_settings") ? DataFixTypes.SAVED_DATA_WORLD_GEN_SETTINGS
                : DataFixTypes.SAVED_DATA_GAME_RULES;
            NbtIo.writeCompressed(upgrade(raw, type), destination.resolve(path));
        }

        String region = "dimensions/minecraft/overworld/region";
        var inFolder = source.resolve(region);
        var outFolder = Files.createDirectories(destination.resolve(region));
        var storage = new RegionStorageInfo("law-fixture", Level.OVERWORLD, "chunk");
        try (var files = Files.newDirectoryStream(inFolder, "r.*.*.mca")) {
            for (Path path : files) {
                var fields = path.getFileName().toString().split("\\.");
                int x0 = Integer.parseInt(fields[1]) * 32, z0 = Integer.parseInt(fields[2]) * 32;
                try (var inRegion = new RegionFile(storage, path, inFolder, false);
                     var outRegion = new RegionFile(storage, outFolder.resolve(path.getFileName()), outFolder, false)) {
                    for (int z = z0; z < z0 + 32; z++) for (int x = x0; x < x0 + 32; x++) {
                        var pos = new ChunkPos(x, z);
                        if (!inRegion.hasChunk(pos)) continue;
                        try (var in = inRegion.getChunkDataInputStream(pos);
                             var out = outRegion.getChunkDataOutputStream(pos)) {
                            var raw = NbtIo.read(in);
                            var context = new CompoundTag();
                            context.putString("dimension", "minecraft:overworld");
                            context.putString("generator", "minecraft:flat");
                            raw.put("__context", context);
                            var upgraded = upgrade(raw, DataFixTypes.CHUNK);
                            upgraded.remove("__context");
                            NbtIo.write(upgraded, out);
                        }
                    }
                }
            }
        }
        System.out.println("Upgraded vanilla fixture to " + destination);
    }
}
