package pathtracing.server;

import java.util.List;
import net.minecraft.network.protocol.Packet;
import net.minecraft.network.protocol.game.ClientGamePacketListener;
import net.minecraft.network.protocol.game.ClientboundBundlePacket;
import net.minecraft.network.protocol.game.ClientboundEntityPositionSyncPacket;
import net.minecraft.network.protocol.game.ClientboundSetEntityMotionPacket;
import net.minecraft.network.protocol.game.ServerboundAcceptTeleportationPacket;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.entity.PositionPath;
import net.minecraft.world.phys.Vec3;

final class MinecraftApi {
    static int damageCooldown(ServerPlayer player) { return player.damageCooldownTime; }
    static boolean velocityChanged(ServerPlayer player) { return player.syncVelocity; }

    static void acceptTeleport(ServerPlayer player, int id) {
        player.connection.handleAcceptTeleportPacket(new ServerboundAcceptTeleportationPacket(id,
            player.getX(), player.getY(), player.getZ(), player.getYRot(), player.getXRot()));
    }

    static Packet<ClientGamePacketListener> position(int id, Vec3 pos, Vec3 vel, float yaw, float pitch, boolean onGround) {
        return new ClientboundBundlePacket(List.of(
            new ClientboundEntityPositionSyncPacket(id, PositionPath.of(pos), yaw, pitch, onGround),
            new ClientboundSetEntityMotionPacket(id, vel)));
    }
}
