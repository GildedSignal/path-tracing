package pathtracing;

import net.minecraft.client.player.LocalPlayer;
import net.minecraft.network.protocol.game.ClientboundLevelChunkWithLightPacket;
import net.minecraft.tags.BlockTags;
import net.minecraft.world.InteractionHand;
import net.minecraft.world.entity.LivingEntity;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.state.BlockState;

public final class MinecraftApi {
    public static final String SWING = "Lnet/minecraft/client/player/LocalPlayer;swing(Lnet/minecraft/world/InteractionHand;Lnet/minecraft/world/item/component/SwingAnimation;Z)Z";

    public static int chunkX(ClientboundLevelChunkWithLightPacket packet) { return packet.x(); }
    public static int chunkZ(ClientboundLevelChunkWithLightPacket packet) { return packet.z(); }
    public static int damageCooldown(LivingEntity entity) { return entity.damageCooldownTime; }
    public static boolean blocksFluidFlow(BlockState state) { return state.is(BlockTags.BLOCKS_FLUID_FLOW); }
    static float fallDistanceReduction(Block block) { return block.getFallDistanceReduction(); }

    static void swing(LocalPlayer player, boolean attack) {
        var stack = player.getMainHandItem();
        player.swing(InteractionHand.MAIN_HAND, attack ? stack.getAttackAnimation() : stack.getInteractAnimation(), false);
    }
}
