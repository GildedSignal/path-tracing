import net.minecraft.core.HolderLookup;
import net.minecraft.data.registries.VanillaRegistries;
import net.minecraft.tags.BlockTags;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.state.BlockState;

final class JarRules {
    static boolean blocksMotion(BlockState state) { return ModelTable.tag(state, BlockTags.BLOCKS_MOTION); }
    static boolean blocksFluidFlow(BlockState state) { return ModelTable.tag(state, BlockTags.BLOCKS_FLUID_FLOW); }
    static boolean hasFluidAdmissionTag() { return true; }
    static boolean washedAway(BlockState state) { return ModelTable.tag(state, BlockTags.WASHED_AWAY_BY_FLUIDS); }
    static boolean halvesFallDistance(Block block) {
        float reduction = block.getFallDistanceReduction();
        if (reduction == 0.0f) return false;
        if (reduction == 0.5f) return true;
        throw new ModelTable.Refused("fall distance reduction " + reduction);
    }
    static HolderLookup.Provider components() { return VanillaRegistries.createWorldLookup(); }
}
