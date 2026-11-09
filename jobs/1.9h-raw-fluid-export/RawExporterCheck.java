package pathtracing;

import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import java.nio.file.*;
import java.util.*;
import net.minecraft.SharedConstants;
import net.minecraft.server.Bootstrap;
import net.minecraft.core.BlockPos;
import net.minecraft.world.level.BlockGetter;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.entity.BlockEntity;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.block.state.properties.BlockStateProperties;
import net.minecraft.world.level.material.Fluids;

public final class RawExporterCheck {
    record Grid(Map<BlockPos, BlockState> cells) implements BlockGetter {
        public BlockState getBlockState(BlockPos p) { return cells.getOrDefault(p, Blocks.AIR.defaultBlockState()); }
        public net.minecraft.world.level.material.FluidState getFluidState(BlockPos p) { return getBlockState(p).getFluidState(); }
        public BlockEntity getBlockEntity(BlockPos p) { return null; }
        public int getHeight() { return 3; }
        public int getMinY() { return -1; }
    }
    static JsonArray coords(BlockPos p) { JsonArray a = new JsonArray(); a.add(p.getX()); a.add(p.getY()); a.add(p.getZ()); return a; }
    static void capture(Path folder, String name, boolean lava, boolean falling, String neighbor) throws Exception {
        var cells = new HashMap<BlockPos, BlockState>();
        var type = lava ? Fluids.LAVA : Fluids.WATER;
        cells.put(BlockPos.ZERO, type.getFlowing(4, falling).createLegacyBlock());
        cells.put(new BlockPos(1, 0, 0), type.getFlowing(1, false).createLegacyBlock());
        cells.put(new BlockPos(-1, 0, 0), type.getFlowing(8, false).createLegacyBlock());
        if (neighbor.equals("wall")) cells.put(new BlockPos(0,1,-1), Blocks.STONE.defaultBlockState());
        if (neighbor.equals("ice")) cells.put(new BlockPos(0,0,-1), Blocks.ICE.defaultBlockState());
        if (neighbor.equals("wet")) cells.put(new BlockPos(0,0,-1), Blocks.OAK_SLAB.defaultBlockState().setValue(BlockStateProperties.WATERLOGGED, true));
        var grid = new Grid(cells);
        var lines = new ArrayList<String>();
        lines.add("{\"kind\":\"world\",\"rawFluid\":true,\"min\":[-1,-1,-1],\"max\":[1,1,1]}");
        lines.add("{\"kind\":\"player\",\"movementSpeedBase\":0.10000000149011612,\"speedModifiers\":[],\"position\":{\"x\":0.5,\"y\":0,\"z\":0.5},\"velocity\":{\"x\":0,\"y\":0,\"z\":0},\"isSprinting\":false}");
        for (BlockPos mutable : BlockPos.betweenClosed(-1,-1,-1,1,1,1)) {
            BlockPos pos = mutable.immutable(); var state = grid.getBlockState(pos); var fluid = state.getFluidState();
            JsonObject row = new JsonObject(); row.addProperty("kind", "block"); row.add("at", coords(pos));
            row.add("boxes", new JsonArray()); row.addProperty("air", state.isAir());
            row.addProperty("fluid", fluid.getType().isSame(Fluids.WATER) ? "water" : fluid.getType().isSame(Fluids.LAVA) ? "lava" : "empty");
            row.addProperty("fluidHeight", fluid.getHeight(grid, pos));
            JsonObject flow = new JsonObject(); flow.addProperty("x", -1); flow.addProperty("y", 0); flow.addProperty("z", 0); row.add("flow", flow);
            WorldExporter.rawFluid(row, grid, pos, state);
            lines.add(row.toString());
        }
        lines.add("{\"kind\":\"worldEnd\",\"blocks\":27}");
        Files.write(folder.resolve(name + ".jsonl"), lines);
        var f = grid.getFluidState(BlockPos.ZERO); var v = f.getFlow(grid, BlockPos.ZERO);
        Files.writeString(folder.resolve(name + ".bits"), "%016x %016x %016x".formatted(Double.doubleToRawLongBits(v.x), Double.doubleToRawLongBits(v.y), Double.doubleToRawLongBits(v.z)));
    }
    public static void main(String[] args) throws Exception {
        SharedConstants.tryDetectVersion(); Bootstrap.bootStrap(); Path p = Path.of(args[0]);
        capture(p,"water",false,false,""); capture(p,"lava",true,false,"");
        capture(p,"wall",false,true,"wall"); capture(p,"ice",false,true,"ice"); capture(p,"wet",false,true,"wet");
    }
}
