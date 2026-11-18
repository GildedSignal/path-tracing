package pathtracing.server;

import com.mojang.authlib.GameProfile;
import io.netty.buffer.Unpooled;
import java.nio.charset.StandardCharsets;
import java.util.EnumSet;
import java.util.HashMap;
import java.util.HashSet;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.Set;
import java.util.UUID;
import net.minecraft.core.Vec3i;
import net.minecraft.network.FriendlyByteBuf;
import net.minecraft.network.chat.Component;
import net.minecraft.network.protocol.Packet;
import net.minecraft.network.protocol.game.ClientGamePacketListener;
import net.minecraft.network.protocol.game.ClientboundAddEntityPacket;
import net.minecraft.network.protocol.game.ClientboundEntityPositionSyncPacket;
import net.minecraft.network.protocol.game.ClientboundPlayerInfoRemovePacket;
import net.minecraft.network.protocol.game.ClientboundPlayerInfoUpdatePacket;
import net.minecraft.network.protocol.game.ClientboundRemoveEntitiesPacket;
import net.minecraft.network.protocol.game.ClientboundRotateHeadPacket;
import net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket;
import net.minecraft.network.protocol.game.ClientboundTrackedWaypointPacket;
import net.minecraft.network.syncher.SynchedEntityData;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.util.Mth;
import net.minecraft.world.entity.EntityTypes;
import net.minecraft.world.entity.Pose;
import net.minecraft.world.entity.PositionMoveRotation;
import net.minecraft.world.level.GameType;
import net.minecraft.world.phys.Vec3;
import net.minecraft.world.waypoints.Waypoint;
import pathtracing.server.mixin.AvatarAccessor;
import pathtracing.server.mixin.EntityAccessor;
import pathtracing.server.mixin.PlayerInfoPacketAccessor;

/** Projects lightweight virtual bots to real human players using pure vanilla packets. */
public final class VisualProjector {
    private static final int VISUAL_ENTITY_BASE = 1_000_000_000;
    private static final EnumSet<ClientboundPlayerInfoUpdatePacket.Action> ADD_ACTIONS = EnumSet.of(
        ClientboundPlayerInfoUpdatePacket.Action.ADD_PLAYER,
        ClientboundPlayerInfoUpdatePacket.Action.UPDATE_GAME_MODE,
        ClientboundPlayerInfoUpdatePacket.Action.UPDATE_LISTED,
        ClientboundPlayerInfoUpdatePacket.Action.UPDATE_LATENCY,
        ClientboundPlayerInfoUpdatePacket.Action.UPDATE_DISPLAY_NAME,
        ClientboundPlayerInfoUpdatePacket.Action.UPDATE_HAT
    );

    private static final class VisualBot {
        final int botId;
        final int botIndex;
        final int entityId;
        final UUID uuid;
        final String name;
        int lastFlags = -1;
        long lastWaypointTick = -100L;

        VisualBot(int botId, int botIndex) {
            this.botId = botId;
            this.botIndex = botIndex;
            this.entityId = VISUAL_ENTITY_BASE + botId;
            this.name = botIndex == 0
                ? "PathBot"
                : String.format(Locale.ROOT, "Swarm%04d", botIndex);
            this.uuid = UUID.nameUUIDFromBytes(("OfflinePlayer:" + name).getBytes(StandardCharsets.UTF_8));
        }
    }

    private final Map<Integer, VisualBot> active = new HashMap<>();
    private final Map<UUID, Set<Integer>> visibleByHuman = new HashMap<>();
    private long packetsBuilt = 0L;

    public long packetsBuilt() {
        return packetsBuilt;
    }

    public void resetPacketCount() {
        packetsBuilt = 0L;
    }

    public int count() {
        return active.size();
    }

    public void addOrUpdateVisual(
        MinecraftServer server,
        int botId,
        int botIndex,
        double x,
        double y,
        double z,
        double vx,
        double vy,
        double vz,
        float yaw,
        float pitch,
        int flags,
        long serverTick
    ) {
        VisualBot vb = active.get(botId);
        if (vb != null) {
            updateVisual(server, botId, x, y, z, vx, vy, vz, yaw, pitch, flags, serverTick);
            return;
        }
        vb = new VisualBot(botId, botIndex);
        vb.lastFlags = flags & 0xFF;
        vb.lastWaypointTick = serverTick;
        active.put(botId, vb);

        Packet<ClientGamePacketListener> infoPkt = buildPlayerInfo(vb);
        Packet<ClientGamePacketListener> addPkt = new ClientboundAddEntityPacket(
            vb.entityId,
            vb.uuid,
            x,
            y,
            z,
            pitch,
            yaw,
            EntityTypes.PLAYER,
            0,
            new Vec3(vx, vy, vz),
            yaw
        );
        Packet<ClientGamePacketListener> metaPkt = buildMetadata(vb.entityId, vb.lastFlags, true);
        Packet<ClientGamePacketListener> wpPkt = ClientboundTrackedWaypointPacket.addWaypointPosition(
            vb.uuid,
            new Waypoint.Icon(),
            new Vec3i(Mth.floor(x), Mth.floor(y), Mth.floor(z))
        );
        packetsBuilt += 4;

        sendToOverworldHumans(server, x, z, p -> {
            Set<Integer> seen = visibleByHuman.computeIfAbsent(p.getUUID(), k -> new HashSet<>());
            seen.add(botId);
            p.connection.send(infoPkt);
            p.connection.send(addPkt);
            p.connection.send(metaPkt);
            p.connection.send(wpPkt);
        });
    }

    public void updateVisual(
        MinecraftServer server,
        int botId,
        double x,
        double y,
        double z,
        double vx,
        double vy,
        double vz,
        float yaw,
        float pitch,
        int flags,
        long serverTick
    ) {
        VisualBot vb = active.get(botId);
        if (vb == null) {
            return;
        }
        boolean onGround = (flags & 1) != 0;
        Packet<ClientGamePacketListener> posPkt = new ClientboundEntityPositionSyncPacket(
            vb.entityId,
            new PositionMoveRotation(new Vec3(x, y, z), new Vec3(vx, vy, vz), yaw, pitch),
            onGround
        );
        Packet<ClientGamePacketListener> headPkt = buildRotateHead(vb.entityId, yaw);
        packetsBuilt += 2;

        int maskedFlags = flags & 0xFF;
        Packet<ClientGamePacketListener> metaPkt = null;
        if (maskedFlags != vb.lastFlags) {
            vb.lastFlags = maskedFlags;
            metaPkt = buildMetadata(vb.entityId, maskedFlags, false);
            packetsBuilt++;
        }

        Packet<ClientGamePacketListener> wpPkt = null;
        if (serverTick - vb.lastWaypointTick >= 10L) {
            vb.lastWaypointTick = serverTick;
            wpPkt = ClientboundTrackedWaypointPacket.updateWaypointPosition(
                vb.uuid,
                new Waypoint.Icon(),
                new Vec3i(Mth.floor(x), Mth.floor(y), Mth.floor(z))
            );
            packetsBuilt++;
        }

        Packet<ClientGamePacketListener> finalMeta = metaPkt;
        Packet<ClientGamePacketListener> finalWp = wpPkt;
        sendToOverworldHumans(server, x, z, p -> {
            Set<Integer> seen = visibleByHuman.get(p.getUUID());
            if (seen == null || !seen.contains(botId)) {
                return;
            }
            p.connection.send(posPkt);
            p.connection.send(headPkt);
            if (finalMeta != null) {
                p.connection.send(finalMeta);
            }
            if (finalWp != null) {
                p.connection.send(finalWp);
            }
        });
    }

    public void removeVisual(MinecraftServer server, int botId, boolean keepTabEntry) {
        VisualBot vb = active.remove(botId);
        if (vb == null) {
            return;
        }
        Packet<ClientGamePacketListener> rmEnt = new ClientboundRemoveEntitiesPacket(vb.entityId);
        Packet<ClientGamePacketListener> rmWp = ClientboundTrackedWaypointPacket.removeWaypoint(vb.uuid);
        Packet<ClientGamePacketListener> rmTab = keepTabEntry
            ? null
            : new ClientboundPlayerInfoRemovePacket(List.of(vb.uuid));
        packetsBuilt += keepTabEntry ? 2 : 3;

        if (server == null) {
            return;
        }
        for (ServerPlayer p : server.overworld().players()) {
            if (p instanceof SwarmBot) {
                continue;
            }
            Set<Integer> seen = visibleByHuman.get(p.getUUID());
            if (seen != null && seen.remove(botId)) {
                p.connection.send(rmEnt);
                p.connection.send(rmWp);
                if (rmTab != null) {
                    p.connection.send(rmTab);
                }
            }
        }
    }

    public void clear(MinecraftServer server) {
        if (active.isEmpty()) {
            visibleByHuman.clear();
            return;
        }
        int[] ids = new int[active.size()];
        UUID[] uuids = new UUID[active.size()];
        int k = 0;
        for (VisualBot vb : active.values()) {
            ids[k] = vb.entityId;
            uuids[k] = vb.uuid;
            k++;
        }
        Packet<ClientGamePacketListener> rmEnts = new ClientboundRemoveEntitiesPacket(ids);
        Packet<ClientGamePacketListener> rmTab = new ClientboundPlayerInfoRemovePacket(List.of(uuids));
        packetsBuilt += 2;

        if (server != null) {
            for (ServerPlayer p : server.overworld().players()) {
                if (p instanceof SwarmBot) {
                    continue;
                }
                p.connection.send(rmEnts);
                p.connection.send(rmTab);
                for (UUID u : uuids) {
                    p.connection.send(ClientboundTrackedWaypointPacket.removeWaypoint(u));
                }
            }
        }
        active.clear();
        visibleByHuman.clear();
    }

    private interface HumanConsumer {
        void send(ServerPlayer human);
    }

    private void sendToOverworldHumans(MinecraftServer server, double x, double z, HumanConsumer fn) {
        if (server == null) {
            return;
        }
        double maxDist = (server.getPlayerList().getViewDistance() + 1) * 16.0;
        double maxDistSq = maxDist * maxDist;
        for (ServerPlayer p : server.overworld().players()) {
            if (p instanceof SwarmBot) {
                continue;
            }
            double dx = p.getX() - x;
            double dz = p.getZ() - z;
            if (dx * dx + dz * dz <= maxDistSq) {
                fn.send(p);
            }
        }
    }

    private static Packet<ClientGamePacketListener> buildPlayerInfo(VisualBot vb) {
        GameProfile profile = new GameProfile(vb.uuid, vb.name);
        ClientboundPlayerInfoUpdatePacket.Entry entry = new ClientboundPlayerInfoUpdatePacket.Entry(
            vb.uuid,
            profile,
            true,
            0,
            GameType.SURVIVAL,
            Component.literal(vb.name),
            true,
            0,
            null
        );
        ClientboundPlayerInfoUpdatePacket pkt = new ClientboundPlayerInfoUpdatePacket(ADD_ACTIONS, List.of());
        ((PlayerInfoPacketAccessor) pkt).ptSetEntries(List.of(entry));
        return pkt;
    }

    private static Packet<ClientGamePacketListener> buildMetadata(int entityId, int flags, boolean includeSkin) {
        boolean sprinting = (flags & 2) != 0;
        boolean sneaking = (flags & 4) != 0;
        byte sharedFlags = (byte) ((sneaking ? 0x02 : 0) | (sprinting ? 0x08 : 0));
        Pose pose = sneaking ? Pose.CROUCHING : Pose.STANDING;
        if (includeSkin) {
            return new ClientboundSetEntityDataPacket(entityId, List.of(
                SynchedEntityData.DataValue.create(EntityAccessor.ptSharedFlagsId(), sharedFlags),
                SynchedEntityData.DataValue.create(EntityAccessor.ptPoseId(), pose),
                SynchedEntityData.DataValue.create(AvatarAccessor.ptSkinLayersId(), (byte) 0x7F)
            ));
        }
        return new ClientboundSetEntityDataPacket(entityId, List.of(
            SynchedEntityData.DataValue.create(EntityAccessor.ptSharedFlagsId(), sharedFlags),
            SynchedEntityData.DataValue.create(EntityAccessor.ptPoseId(), pose)
        ));
    }

    private static Packet<ClientGamePacketListener> buildRotateHead(int entityId, float yaw) {
        FriendlyByteBuf buf = new FriendlyByteBuf(Unpooled.buffer(8));
        try {
            buf.writeVarInt(entityId);
            buf.writeByte(Mth.packDegrees(yaw));
            return ClientboundRotateHeadPacket.STREAM_CODEC.decode(buf);
        } finally {
            buf.release();
        }
    }
}
