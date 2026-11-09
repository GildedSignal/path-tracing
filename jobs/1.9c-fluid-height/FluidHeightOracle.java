import java.io.IOException;
import java.io.PrintWriter;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Locale;
import net.minecraft.SharedConstants;
import net.minecraft.core.BlockPos;
import net.minecraft.server.Bootstrap;
import net.minecraft.world.level.BlockGetter;
import net.minecraft.world.level.block.entity.BlockEntity;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.material.FlowingFluid;
import net.minecraft.world.level.material.FluidState;
import net.minecraft.world.level.material.Fluids;

public class FluidHeightOracle {
    private record AboveCell(FluidState above) implements BlockGetter {
        @Override public FluidState getFluidState(BlockPos pos) {
            if (!pos.equals(BlockPos.ZERO.above())) throw new AssertionError("unexpected fluid read: " + pos);
            return above;
        }
        @Override public BlockEntity getBlockEntity(BlockPos pos) { throw new AssertionError(pos); }
        @Override public BlockState getBlockState(BlockPos pos) { throw new AssertionError(pos); }
        @Override public int getHeight() { throw new AssertionError(); }
        @Override public int getMinY() { throw new AssertionError(); }
    }

    private static void capture(PrintWriter out, FlowingFluid fluid, String kind, FlowingFluid other) {
        FluidState[] above = {
            Fluids.EMPTY.defaultFluidState(), fluid.getSource(false), fluid.getFlowing(1, true), other.getSource(false)
        };
        String[] labels = {"empty", "same-source", "same-flowing", "different"};
        for (int neighbor = 0; neighbor < above.length; neighbor++) {
            var world = new AboveCell(above[neighbor]);
            for (int amount = 1; amount <= 8; amount++) {
                for (boolean falling : new boolean[] {false, true}) {
                    var current = fluid.getFlowing(amount, falling);
                    int height = Float.floatToRawIntBits(current.getHeight(world, BlockPos.ZERO));
                    out.printf(Locale.ROOT, "%s,%d,%s,%s,%08x%n", kind, amount, falling, labels[neighbor], height);
                }
            }
            var source = fluid.getSource(false);
            int height = Float.floatToRawIntBits(source.getHeight(world, BlockPos.ZERO));
            out.printf(Locale.ROOT, "%s,%d,source,%s,%08x%n", kind, source.getAmount(), labels[neighbor], height);
        }
    }

    public static void main(String[] args) throws IOException {
        SharedConstants.tryDetectVersion();
        Bootstrap.bootStrap();
        try (var out = new PrintWriter(Files.newBufferedWriter(Path.of(args[0])))) {
            out.println("kind,amount,form,above,height");
            capture(out, Fluids.WATER, "water", Fluids.LAVA);
            capture(out, Fluids.LAVA, "lava", Fluids.WATER);
            if (out.checkError()) throw new IOException("Could not write oracle output.");
        }
    }
}
