package pathtracing.planner;

import java.util.List;
import net.minecraft.core.BlockPos;
import net.minecraft.world.level.BlockGetter;

/** The algorithm slot. Everything else in the mod is plumbing around this one call. */
public interface Planner {
    /**
     * @param world the bot client's copy of the world; only loaded chunks hold real blocks
     * @param start the cell the bot's feet are in
     * @param goal  the cell the feet should end in
     * @return the cells the feet pass through, in order, goal last and start excluded; null if unreachable
     */
    List<BlockPos> plan(BlockGetter world, BlockPos start, BlockPos goal);
}
