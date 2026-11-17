package pathtracing;

import java.io.IOException;
import java.nio.file.Path;
import net.minecraft.nbt.NbtAccounter;
import net.minecraft.nbt.NbtIo;

/** Read both current and older vanilla spawn layouts with Mojang's NBT reader. */
final class MapSpawn {
    static int[] read(Path path) throws IOException {
        return read(NbtIo.readCompressed(path, NbtAccounter.unlimitedHeap()));
    }

    static int[] read(net.minecraft.nbt.CompoundTag level) throws IOException {
        var data = level.getCompound("Data").orElseThrow();
        var spawn = data.getCompound("spawn");
        if (spawn.isPresent()) {
            var pos = spawn.get().getIntArray("pos").orElseThrow();
            if (pos.length != 3) throw new IOException("Invalid spawn position");
            return pos;
        }
        return new int[] { data.getInt("SpawnX").orElseThrow(), data.getInt("SpawnY").orElseThrow(), data.getInt("SpawnZ").orElseThrow() };
    }

    public static void main(String[] args) throws IOException {
        int[] p = read(Path.of(args[0]));
        System.out.println(p[0] + " " + p[1] + " " + p[2]);
    }
}
