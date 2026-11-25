package pathtracing.server;

import com.mojang.authlib.GameProfile;
import io.netty.channel.ChannelFutureListener;
import java.net.InetAddress;
import java.net.InetSocketAddress;
import java.net.SocketAddress;
import net.minecraft.network.Connection;
import net.minecraft.network.PacketListener;
import net.minecraft.network.ProtocolInfo;
import net.minecraft.network.chat.ChatType;
import net.minecraft.network.chat.OutgoingChatMessage;
import net.minecraft.network.protocol.Packet;
import net.minecraft.network.protocol.PacketFlow;
import net.minecraft.network.protocol.game.ClientboundPlayerPositionPacket;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ClientInformation;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.damagesource.DamageSource;
import net.minecraft.world.phys.Vec3;

/** Real-player bot in the server driven by `mind.exe` over shared memory. */
public final class SwarmBot extends ServerPlayer {
    public final int botId;
    public final SinkConnection sink;
    public final double relX;
    public final double relZ;

    public long lastSolveNs;
    public int lastPredictedTicks;
    public int activeTicks;
    public int actualT = -1;
    public boolean executing;
    public boolean settling;
    public boolean arrived;
    public boolean settled;
    public String lastStatus = "idle";
    public String lastMsg = "";
    public String lastRequester;
    public long lastTickedSeq = -1L;
    public boolean pendingDamageSync = false;
    public boolean deadNotified = false;
    public int deadTicks = 0;

    public SwarmBot(
        MinecraftServer server,
        ServerLevel level,
        GameProfile profile,
        int botId,
        SinkConnection sink,
        double relX,
        double relZ
    ) {
        super(server, level, profile, ClientInformation.createDefault());
        this.botId = botId;
        this.sink = sink;
        this.relX = relX;
        this.relZ = relZ;
    }

    @Override
    public boolean hurtServer(ServerLevel level, DamageSource source, float amount) {
        boolean hit = super.hurtServer(level, source, amount);
        if (hit) {
            this.pendingDamageSync = true;
        }
        return hit;
    }

    @Override
    public void die(DamageSource source) {
        super.die(source);
        this.pendingDamageSync = true;
        this.executing = false;
        this.settling = false;
        SwarmManager manager = ServerMindMod.manager();
        if (manager != null) {
            manager.onBotDied(this);
        }
    }

    @Override
    public void sendChatMessage(OutgoingChatMessage message, boolean filter, ChatType.Bound bound) {
        String senderName = bound.name().getString();
        String text = message.content().getString();
        SwarmManager manager = ServerMindMod.manager();
        if (manager != null) {
            manager.onWhisper(this, senderName, text);
        }
    }

    public static final class SinkConnection extends net.minecraft.network.Connection {
        public int pendingTpId = -1;
        public Vec3 pendingTpPos;

        public SinkConnection() {
            super(PacketFlow.SERVERBOUND);
        }

        private void capture(Packet<?> packet) {
            if (packet instanceof ClientboundPlayerPositionPacket tp) {
                pendingTpId = tp.id();
                pendingTpPos = tp.change().position();
            }
        }

        @Override
        public boolean isConnected() {
            return true;
        }

        @Override
        public boolean isMemoryConnection() {
            return true;
        }

        @Override
        public SocketAddress getRemoteAddress() {
            return new InetSocketAddress(InetAddress.getLoopbackAddress(), 0);
        }

        @Override
        public void send(Packet<?> p) {
            capture(p);
        }

        @Override
        public void send(Packet<?> p, ChannelFutureListener l) {
            capture(p);
        }

        @Override
        public void send(Packet<?> p, ChannelFutureListener l, boolean flush) {
            capture(p);
        }

        @Override
        public <T extends PacketListener> void setupInboundProtocol(ProtocolInfo<T> p, T l) {
        }

        @Override
        public void setupOutboundProtocol(ProtocolInfo<?> p) {
        }

        @Override
        public void flushChannel() {
        }
    }
}
