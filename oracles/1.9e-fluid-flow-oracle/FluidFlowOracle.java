import java.io.IOException;
import java.io.PrintWriter;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.HashMap;
import java.util.Locale;
import java.util.Map;
import java.util.SplittableRandom;
import net.minecraft.SharedConstants;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Direction;
import net.minecraft.server.Bootstrap;
import net.minecraft.world.level.BlockGetter;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.entity.BlockEntity;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.block.state.properties.BlockStateProperties;
import net.minecraft.world.level.material.FluidState;
import net.minecraft.world.level.material.Fluids;

public final class FluidFlowOracle {
    private record Cell(BlockState block, FluidState fluid) {}
    private static final Direction[] HORIZONTAL = {Direction.NORTH, Direction.EAST, Direction.SOUTH, Direction.WEST};
    private static final String[] CELLS = {"air", "stone", "ice", "slab", "wet-slab", "w1", "w4", "w8", "l1", "l4", "l8"};

    private static Cell cell(String id) {
        BlockState block = switch (id) {
            case "air" -> Blocks.AIR.defaultBlockState();
            case "stone" -> Blocks.STONE.defaultBlockState();
            case "ice" -> Blocks.ICE.defaultBlockState();
            case "slab" -> Blocks.OAK_SLAB.defaultBlockState();
            case "wet-slab" -> Blocks.OAK_SLAB.defaultBlockState().setValue(BlockStateProperties.WATERLOGGED, true);
            default -> (id.startsWith("w") ? Blocks.WATER : Blocks.LAVA).defaultBlockState();
        };
        FluidState fluid = switch (id.charAt(0)) {
            case 'w', 'l' -> id.length() == 2
                ? (id.startsWith("w") ? Fluids.WATER : Fluids.LAVA).getFlowing(id.charAt(1) - '0', false)
                : block.getFluidState();
            default -> block.getFluidState();
        };
        // The corresponding liquid block level is immaterial to the current rule.
        return new Cell(block, fluid);
    }

    private record Fixture(Map<BlockPos, Cell> cells) implements BlockGetter {
        private Cell at(BlockPos position) {
            Cell value = cells.get(position);
            if (value == null) throw new AssertionError("unexpected read " + position);
            return value;
        }
        @Override public FluidState getFluidState(BlockPos pos) { return at(pos).fluid(); }
        @Override public BlockState getBlockState(BlockPos pos) { return at(pos).block(); }
        @Override public BlockEntity getBlockEntity(BlockPos pos) { throw new AssertionError(pos); }
        @Override public int getHeight() { throw new AssertionError(); }
        @Override public int getMinY() { throw new AssertionError(); }
    }

    private static void capture(PrintWriter out, String name, String kind, int amount, boolean falling,
            String[] cells) {
        var world = new HashMap<BlockPos, Cell>();
        for (int index = 0; index < HORIZONTAL.length; index++) {
            BlockPos neighbor = BlockPos.ZERO.relative(HORIZONTAL[index]);
            world.put(neighbor, cell(cells[index]));
            world.put(neighbor.below(), cell(cells[index + 4]));
            world.put(neighbor.above(), cell(cells[index + 8]));
        }
        var type = kind.equals("water") ? Fluids.WATER : Fluids.LAVA;
        var current = type.getFlowing(amount, falling);
        var velocity = current.getFlow(new Fixture(world), BlockPos.ZERO);
        out.printf(Locale.ROOT, "%s,%s,%d,%s,%s,%016x,%016x,%016x%n", name, kind, amount, falling,
            String.join(",", cells), Double.doubleToRawLongBits(velocity.x), Double.doubleToRawLongBits(velocity.y),
            Double.doubleToRawLongBits(velocity.z));
    }

    private static String[] cells(String... placements) {
        var cells = new String[12];
        java.util.Arrays.fill(cells, "air");
        for (int i = 0; i < placements.length; i++) cells[i] = placements[i];
        return cells;
    }

    public static void main(String[] args) throws IOException {
        SharedConstants.tryDetectVersion();
        Bootstrap.bootStrap();
        try (var out = new PrintWriter(Files.newBufferedWriter(Path.of(args[0])))) {
            out.println("case,kind,amount,falling,n,e,s,w,n_below,e_below,s_below,w_below,"
                + "n_above,e_above,s_above,w_above,x,y,z");
            capture(out, "level-equal", "water", 4, false, cells("w4", "w4", "w4", "w4"));
            capture(out, "level-gradient", "water", 4, false, cells("w4", "w1", "w4", "w8"));
            capture(out, "level-reversed", "water", 4, false, cells("w4", "w8", "w4", "w1"));
            capture(out, "open-downstep", "water", 4, false, cells("air", "air", "air", "air", "air", "w8"));
            capture(out, "blocked-downstep", "water", 4, false, cells("air", "stone", "air", "air", "air", "w8"));
            capture(out, "fall-stone", "water", 4, true, cells("stone"));
            capture(out, "fall-ice", "water", 4, true, cells("ice"));
            capture(out, "fall-wet-slab", "water", 4, true, cells("wet-slab"));
            capture(out, "fall-wall-above", "water", 4, true,
                cells("w1", "air", "air", "air", "air", "air", "air", "air", "air", "stone"));
            capture(out, "fall-open", "water", 4, true, cells("w1"));
            var random = new SplittableRandom(262);
            for (int i = 0; i < 1024; i++) {
                var neighborhood = new String[12];
                for (int j = 0; j < neighborhood.length; j++) {
                    neighborhood[j] = CELLS[random.nextInt(CELLS.length)];
                }
                capture(out, "mixed-" + i, random.nextBoolean() ? "water" : "lava",
                    random.nextInt(1, 9), random.nextBoolean(), neighborhood);
            }
            if (out.checkError()) throw new IOException("Could not write oracle output.");
        }
    }
}
