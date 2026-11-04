package pathtracing.planner;

import java.util.List;
import net.minecraft.core.BlockPos;
import net.minecraft.world.level.BlockGetter;

/** Placeholder so the plumbing can be tested end to end: walk straight at the goal, ignore the world. */
public final class StraightLine implements Planner {
    @Override
    public List<BlockPos> plan(BlockGetter world, BlockPos start, BlockPos goal) {
        return List.of(goal);
    }
}
