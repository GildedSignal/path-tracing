package pathtracing.server;

import net.minecraft.network.protocol.Packet;
import net.minecraft.network.protocol.game.ClientGamePacketListener;
import net.minecraft.network.protocol.game.ClientboundEntityPositionSyncPacket;
import net.minecraft.network.protocol.game.ServerboundAcceptTeleportationPacket;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.entity.PositionMoveRotation;
import net.minecraft.world.phys.Vec3;

final class MinecraftApi {
    static int damageCooldown(ServerPlayer player) { return player.invulnerableTime; }
    static boolean velocityChanged(ServerPlayer player) { return player.hurtMarked; }

    static void acceptTeleport(ServerPlayer player, int id) {
        player.connection.handleAcceptTeleportPacket(new ServerboundAcceptTeleportationPacket(id));
    }

    static Packet<ClientGamePacketListener> position(int id, Vec3 pos, Vec3 vel, float yaw, float pitch, boolean onGround) {
        return new ClientboundEntityPositionSyncPacket(id, new PositionMoveRotation(pos, vel, yaw, pitch), onGround);
    }
}
