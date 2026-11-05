package pathtracing;

import java.io.DataInputStream;
import java.io.IOException;
import java.net.InetSocketAddress;
import java.net.Socket;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import java.util.Properties;
import java.util.function.BooleanSupplier;

/** Vanilla localhost RCON, run off the client thread. Credentials never leave this class. */
final class TrackRcon {
    static void execute(Path rig, List<String> commands) throws IOException {
        execute(rig, commands, () -> false);
    }

    static void execute(Path rig, List<String> commands, BooleanSupplier cancelled) throws IOException {
        Properties properties = new Properties();
        try (var reader = Files.newBufferedReader(rig.resolve("server/server.properties"), StandardCharsets.UTF_8)) {
            properties.load(reader);
        }
        String password = properties.getProperty("rcon.password");
        if (password == null || password.isEmpty()) throw new IOException("RCON password is not configured.");
        int port = Integer.parseInt(properties.getProperty("rcon.port", "25575"));
        try (Socket socket = new Socket()) {
            socket.connect(new InetSocketAddress("127.0.0.1", port), 3000);
            socket.setSoTimeout(3000);
            exchange(socket, 1, 3, password);
            int id = 2;
            for (String command : commands) {
                if (cancelled.getAsBoolean()) return;
                exchange(socket, id++, 2, command);
            }
        }
    }

    private static void exchange(Socket socket, int id, int type, String body) throws IOException {
        byte[] bytes = body.getBytes(StandardCharsets.UTF_8);
        ByteBuffer packet = ByteBuffer.allocate(bytes.length + 14).order(ByteOrder.LITTLE_ENDIAN);
        packet.putInt(bytes.length + 10).putInt(id).putInt(type).put(bytes).put((byte) 0).put((byte) 0);
        socket.getOutputStream().write(packet.array());
        DataInputStream input = new DataInputStream(socket.getInputStream());
        int length = Integer.reverseBytes(input.readInt());
        if (length < 10 || length > 1048576) throw new IOException("Invalid RCON response length.");
        byte[] response = new byte[length];
        input.readFully(response);
        if (response[length - 1] != 0 || response[length - 2] != 0) throw new IOException("Invalid RCON response terminator.");
        ByteBuffer decoded = ByteBuffer.wrap(response).order(ByteOrder.LITTLE_ENDIAN);
        if (decoded.getInt() != id) throw new IOException("RCON authentication or command failed.");
        int responseType = decoded.getInt();
        if (responseType != (type == 3 ? 2 : 0)) throw new IOException("Unexpected RCON response type.");
    }
}
