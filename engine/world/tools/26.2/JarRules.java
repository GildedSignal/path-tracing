import net.minecraft.core.HolderLookup;
import net.minecraft.data.registries.VanillaRegistries;
import net.minecraft.world.level.block.BedBlock;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.state.BlockState;

final class JarRules {
    static boolean blocksMotion(BlockState state) { return state.blocksMotion(); }
    static boolean blocksFluidFlow(BlockState state) { return blocksMotion(state); }
    static boolean hasFluidAdmissionTag() { return false; }
    static boolean washedAway(BlockState state) { throw new AssertionError("no vanilla fluid admission tag"); }
    static boolean halvesFallDistance(Block block) { return block instanceof BedBlock; }
    static HolderLookup.Provider components() { return VanillaRegistries.createLookup(); }
}
