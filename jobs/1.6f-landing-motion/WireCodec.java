import io.netty.buffer.ByteBufUtil;
import io.netty.buffer.Unpooled;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import java.util.HexFormat;
import net.minecraft.network.protocol.game.ClientboundSetEntityMotionPacket;
import net.minecraft.world.phys.Vec3;

/** Re-encode the witnessed dispatch with the staged 26.2 codec. */
class WireCodec {
    public static void main(String[] args) throws Exception {
        for (String file : args) {
            var rows = Files.readAllLines(Path.of(file));
            for (String row : rows.subList(rows.indexOf("wire") + 1, rows.size())) {
                var wire = row.split(",");
                if (!wire[1].equals("motion")) continue;
                int cause = Integer.parseInt(wire[6]);
                var consume = rows.get(cause + 1).split(",");
                if (!consume[3].equals("consume")) throw new IllegalStateException(row);
                var vector = new Vec3(number(consume[7]), number(consume[8]), number(consume[9]));
                var packet = new ClientboundSetEntityMotionPacket(Integer.parseInt(consume[6]), vector);
                var codec = ClientboundSetEntityMotionPacket.STREAM_CODEC;
                var bytes = Unpooled.buffer();
                try {
                    codec.encode(bytes, packet);
                    if (!ByteBufUtil.hexDump(bytes).equals(wire[7])) {
                        throw new IllegalStateException(file + ": dispatch did not encode to " + row);
                    }
                    var decoded = codec.decode(bytes);
                    if (bytes.isReadable() || decoded.id() != Integer.parseInt(wire[5])
                        || !Arrays.equals(raw(decoded.movement()), Arrays.copyOfRange(wire, 8, 11))) {
                        throw new IllegalStateException(file + ": decoded payload disagrees with " + row);
                    }
                } finally {
                    bytes.release();
                }
                System.out.println(file + ": staged codec reproduced own-motion ordinal " + wire[2]);
            }
        }
    }

    private static double number(String hex) {
        return Double.longBitsToDouble(HexFormat.fromHexDigitsToLong(hex));
    }

    private static String[] raw(Vec3 value) {
        var hex = HexFormat.of();
        return new String[] { hex.toHexDigits(Double.doubleToRawLongBits(value.x)),
            hex.toHexDigits(Double.doubleToRawLongBits(value.y)), hex.toHexDigits(Double.doubleToRawLongBits(value.z)) };
    }
}
