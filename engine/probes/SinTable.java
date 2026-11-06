import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.file.Files;
import java.nio.file.Path;

public class SinTable {
    // Mth.lambda$static$0
    public static void main(String[] args) throws Exception {
        ByteBuffer bytes = ByteBuffer.allocate(65536 * 4).order(ByteOrder.LITTLE_ENDIAN);
        for (int i = 0; i < 65536; i++) bytes.putFloat((float) Math.sin((double) i / 10430.378350470453));
        Files.write(Path.of(args[0]), bytes.array());
    }
}
