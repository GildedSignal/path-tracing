package pathtracing;

import net.minecraft.client.player.LocalPlayer;
import net.minecraft.network.protocol.game.ClientboundLevelChunkWithLightPacket;
import net.minecraft.world.InteractionHand;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.level.block.BedBlock;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.state.BlockState;

public final class MinecraftApi {
    public static final String SWING = "Lnet/minecraft/client/player/LocalPlayer;swing(Lnet/minecraft/world/InteractionHand;)V";

    public static int chunkX(ClientboundLevelChunkWithLightPacket packet) { return packet.getX(); }
    public static int chunkZ(ClientboundLevelChunkWithLightPacket packet) { return packet.getZ(); }
    public static int damageCooldown(Entity entity) { return entity.invulnerableTime; }
    public static boolean blocksFluidFlow(BlockState state) { return state.blocksMotion(); }
    static float fallDistanceReduction(Block block) { return block instanceof BedBlock ? 0.5f : 0.0f; }

    static void swing(LocalPlayer player, boolean attack) { player.swing(InteractionHand.MAIN_HAND); }
}
