package pathtracing.server.mixin;

import java.util.List;
import net.minecraft.network.protocol.game.ClientboundPlayerInfoUpdatePacket;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Mutable;
import org.spongepowered.asm.mixin.gen.Accessor;

@Mixin(ClientboundPlayerInfoUpdatePacket.class)
public interface PlayerInfoPacketAccessor {
    @Mutable
    @Accessor("entries")
    void ptSetEntries(List<ClientboundPlayerInfoUpdatePacket.Entry> entries);
}
