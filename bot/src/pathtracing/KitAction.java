package pathtracing;

import java.util.ArrayList;
import java.util.List;
import net.minecraft.client.Minecraft;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Direction;
import net.minecraft.world.InteractionHand;
import net.minecraft.world.phys.BlockHitResult;
import net.minecraft.world.phys.Vec3;

/** One world edit of a native KIT_PLAN: the click or hold a player makes, at a plan tick.
 *  (x, y, z) is the edited cell; Place and BucketEmpty click its neighbour against `face`. */
record KitAction(int tick, int slot, int opcode, int x, int y, int z, int face, float yaw, float pitch, int hold) {
    static final int INTERACT = 1, PLACE = 2, BREAK_HOLD = 3, BUCKET_EMPTY = 4, BUCKET_PICKUP = 5, WORDS = 10;

    static List<KitAction> read(long[] answer, int at, int count) {
        List<KitAction> out = new ArrayList<>(count);
        for (int i = 0; i < count; i++) {
            int o = at + i * WORDS;
            out.add(new KitAction((int) answer[o], (int) answer[o + 1], (int) answer[o + 2], (int) answer[o + 3], (int) answer[o + 4],
                (int) answer[o + 5], (int) answer[o + 6], Float.intBitsToFloat((int) answer[o + 7]), Float.intBitsToFloat((int) answer[o + 8]),
                (int) answer[o + 9]));
        }
        return out;
    }

    /** A block echo at (x, y, z) is the plan's own when an action already issued edits that cell
     *  (a door's other half moves with it); any other edit in the corridor stops the flight. */
    static boolean ours(List<KitAction> actions, int issuedUpTo, int x, int y, int z) {
        for (KitAction a : actions) {
            if (a.tick > issuedUpTo || a.x != x || a.z != z) continue;
            if (a.y == y || a.opcode == INTERACT && Math.abs(a.y - y) == 1) return true;
        }
        return false;
    }

    BlockPos clicked() {
        BlockPos cell = new BlockPos(x, y, z);
        return opcode == PLACE || opcode == BUCKET_EMPTY ? cell.relative(Direction.from3DDataValue(face).getOpposite()) : cell;
    }

    /** Performs tick `k` of this action at the start of the client tick, after the movement input.
     *  The look turns to the clicked face for the click and back, so the movement keeps its yaw;
     *  a break's ticks hold still and already look at the block. Returns false while a break's
     *  last tick passes with the block still standing (the client's 5-tick destroy delay or a
     *  float rounding), or while waiting for a broken block's drop to finish its 10t pickup delay
     *  before restoring it, so the runner holds that tick instead of moving on. */
    boolean perform(Minecraft mc, int k) {
        return perform(mc, k, java.util.Map.of());
    }

    boolean perform(Minecraft mc, int k, java.util.Map<BlockPos, net.minecraft.world.level.block.state.BlockState> before) {
        var player = mc.player;
        int useSlot = slot;
        Direction side = Direction.from3DDataValue(face);
        BlockPos target = clicked();
        if (opcode == BREAK_HOLD) {
            player.getInventory().setSelectedSlot(useSlot);
            if (k == 0) mc.gameMode.startDestroyBlock(target, side);
            mc.gameMode.continueDestroyBlock(target, side);
            player.swing(InteractionHand.MAIN_HAND);
            return k < hold - 1 || mc.level.getBlockState(target).canBeReplaced();
        }
        if (opcode == PLACE) {
            var prev = before.get(new BlockPos(x, y, z));
            net.minecraft.world.item.Item want = (prev != null && !prev.isAir()) ? prev.getBlock().asItem() : null;
            var held = player.getInventory().getItem(useSlot);
            boolean haveMatch = want != null ? held.is(want) : (!held.isEmpty() && held.getItem() instanceof net.minecraft.world.item.BlockItem);
            if (!haveMatch) {
                for (int i = 0; i < 9; i++) {
                    var st = player.getInventory().getItem(i);
                    boolean match = want != null ? st.is(want) : (!st.isEmpty() && st.getItem() instanceof net.minecraft.world.item.BlockItem);
                    if (match) {
                        useSlot = i;
                        haveMatch = true;
                        break;
                    }
                }
            }
            if (!haveMatch) {
                var drops = mc.level.getEntitiesOfClass(
                    net.minecraft.world.entity.item.ItemEntity.class,
                    player.getBoundingBox().inflate(5.0),
                    e -> e.isAlive() && !e.getItem().isEmpty()
                        && (want == null ? (e.getItem().getItem() instanceof net.minecraft.world.item.BlockItem) : e.getItem().is(want))
                );
                if (!drops.isEmpty()) {
                    var best = drops.get(0);
                    double bestD2 = best.distanceToSqr(player);
                    for (int i = 1; i < drops.size(); i++) {
                        double d2 = drops.get(i).distanceToSqr(player);
                        if (d2 < bestD2) { best = drops.get(i); bestD2 = d2; }
                    }
                    if (!player.getBoundingBox().inflate(1.0, 0.5, 1.0).intersects(best.getBoundingBox())) {
                        float yawTo = (float) Math.toDegrees(Math.atan2(best.getZ() - player.getZ(), best.getX() - player.getX())) - 90.0f;
                        player.setYRot(yawTo);
                        mc.options.keyUp.setDown(true);
                    }
                }
                return false;
            }
        }
        player.getInventory().setSelectedSlot(useSlot);
        float moveYaw = player.getYRot(), movePitch = player.getXRot();
        player.setYRot(yaw); player.setXRot(pitch);
        Vec3 hit = Vec3.atCenterOf(target).add(side.getStepX() * 0.5, side.getStepY() * 0.5, side.getStepZ() * 0.5);
        var result = mc.gameMode.useItemOn(player, InteractionHand.MAIN_HAND, new BlockHitResult(hit, side, target, false));
        if (!result.consumesAction()) result = mc.gameMode.useItem(player, InteractionHand.MAIN_HAND);
        if (result.consumesAction()) player.swing(InteractionHand.MAIN_HAND);
        player.setYRot(moveYaw); player.setXRot(movePitch);
        return true;
    }
}
