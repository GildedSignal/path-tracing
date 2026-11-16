package pathtracing;

import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import java.util.List;
import net.minecraft.nbt.CompoundTag;
import net.minecraft.nbt.NbtAccounter;
import net.minecraft.nbt.NbtIo;
import net.minecraft.nbt.StringTag;

/** Offline importer/view checks against copies of the committed world metadata. */
public final class MapTests {
    private static int checks;
    private static void check(boolean value, String message) {
        checks++;
        if (!value) throw new AssertionError(message);
    }

    public static void main(String[] args) throws Exception {
        Path rig = Path.of(args[0]);
        Path world = rig.resolve("server/world");
        Path root = rig.getParent();
        int[] spawn = MapSpawn.read(world.resolve("level.dat"));
        check(Arrays.equals(spawn, new int[] {0, -60, 0}), "Current vanilla spawn layout");
        var legacy = new CompoundTag();
        var data = new CompoundTag();
        data.putInt("SpawnX", 12); data.putInt("SpawnY", 80); data.putInt("SpawnZ", -7);
        legacy.put("Data", data);
        NbtIo.writeCompressed(legacy, root.resolve("legacy.dat"));
        check(Arrays.equals(MapSpawn.read(root.resolve("legacy.dat")), new int[] {12, 80, -7}), "Legacy vanilla spawn layout");
        for (String name : List.of("hard_track_2", "a-b")) {
            TrackStore.validateName(name);
            check(TrackWorldView.region(world, "pathtracing:" + name).equals(world.resolve("dimensions/pathtracing/" + name + "/region")), "Imported region: " + name);
        }
        check(TrackWorldView.region(world, "minecraft:overworld").equals(world.resolve("dimensions/minecraft/overworld/region")), "Existing overworld region");
        try { TrackWorldView.region(world, "minecraft:the_nether"); throw new AssertionError("Unsupported dimension accepted"); }
        catch (IllegalArgumentException expected) { check(expected.getMessage().contains("minecraft:the_nether"), "Refusal names the dimension"); }
        Path tracks = rig.resolve("lab/tracks");
        Files.createDirectories(tracks);
        Path track = tracks.resolve("hard_track_2.json");
        Files.writeString(track, "{\"A\":[5,-61,26],\"B\":[5,-61,55],\"dimension\":\"pathtracing:hard_track_2\",\"created\":\"test\",\"creator\":\"test\"}");
        var level = NbtIo.readCompressed(world.resolve("level.dat"), NbtAccounter.unlimitedHeap());
        level.getCompound("Data").orElseThrow().getCompound("DataPacks").orElseThrow().getList("Enabled").orElseThrow().add(StringTag.valueOf("file/pathtracing-maps"));
        NbtIo.writeCompressed(level, world.resolve("level.dat"));
        byte[] before = Files.readAllBytes(world.resolve("level.dat"));
        Path view = root.resolve("view");
        TrackWorldView.create(world, track, view);
        check(Arrays.equals(before, Files.readAllBytes(world.resolve("level.dat"))), "Dimension projection preserves source metadata bytes");
        var enabled = NbtIo.readCompressed(view.resolve("level.dat"), NbtAccounter.unlimitedHeap()).getCompoundOrEmpty("Data").getCompoundOrEmpty("DataPacks").getListOrEmpty("Enabled");
        check(enabled.size() == 1 && enabled.get(0).asString().orElseThrow().equals("vanilla"), "Dimension-only pack projected without changing effective vanilla laws");
        check(Files.isSameFile(view.resolve("dimensions/minecraft/overworld/region/r.0.0.mca"), world.resolve("dimensions/pathtracing/hard_track_2/region/r.0.0.mca")), "View reads the selected map's actual region bytes");
        Path unexpected = world.resolve("datapacks/pathtracing-maps/data/pathtracing/tags/block/extra.json");
        Files.createDirectories(unexpected.getParent());
        Files.writeString(unexpected, "{}");
        try { TrackWorldView.create(world, track, root.resolve("rejected-view")); throw new AssertionError("Law-changing pack accepted"); }
        catch (java.io.IOException expected) { check(expected.getMessage().contains("more than map dimensions"), "Other datapack resources cannot bypass the world reader gate"); }
        Files.delete(unexpected);
        System.out.println("Passed " + checks + " offline map checks.");
    }
}
