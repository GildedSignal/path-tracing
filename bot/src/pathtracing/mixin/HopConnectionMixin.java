package pathtracing.mixin;

import io.netty.channel.ChannelHandlerContext;
import io.netty.channel.ChannelFutureListener;
import net.minecraft.network.Connection;
import net.minecraft.network.protocol.Packet;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import pathtracing.HopCapture;

@Mixin(Connection.class)
abstract class HopConnectionMixin {
    @Inject(method="doSendPacket",at=@At("HEAD"))
    private void ptSend(Packet<?> packet,ChannelFutureListener listener,boolean flush,CallbackInfo ci) {
        HopCapture.packet((Connection)(Object)this,packet,true);
    }
    @Inject(method="channelRead0(Lio/netty/channel/ChannelHandlerContext;Lnet/minecraft/network/protocol/Packet;)V",at=@At("HEAD"))
    private void ptReceive(ChannelHandlerContext context,Packet<?> packet,CallbackInfo ci) {
        HopCapture.packet((Connection)(Object)this,packet,false);
    }
}
