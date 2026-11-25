package pathtracing;

import java.util.regex.Matcher;
import java.util.regex.Pattern;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents;
import net.fabricmc.fabric.api.client.message.v1.ClientReceiveMessageEvents;
import net.fabricmc.fabric.api.client.networking.v1.ClientPlayConnectionEvents;
import net.minecraft.client.Minecraft;
import net.minecraft.core.BlockPos;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * The bot's side. Takes goals from the owner's whispers (goto x y z, come, stop), asks the planner for a
 * path, then walks it by holding the same movement keys a person would.
 */
final class Bot {
    private static final Logger LOGGER = LoggerFactory.getLogger("pathtracing");
    private static final Pattern WHISPER = Pattern.compile("(\\S+) whispers to you: (.*)");
    private static long commandIssue;
    private static int survivalIn = -1;

    static void register() {
        // Wait a second: commands sent in the join callback itself are dropped (no command tree yet).
        ClientPlayConnectionEvents.JOIN.register((handler, sender, mc) -> { survivalIn = 20; ResidentSource.join(mc); });
        ClientPlayConnectionEvents.DISCONNECT.register((handler, mc) -> { ResidentRunner.cancel(mc); HopCapture.flush(); });
        ClientReceiveMessageEvents.CHAT.register((message, signed, sender, type, time) -> {
            if (sender != null && signed != null && sender.name().equals(PathTracingMod.OWNER)) command(signed.signedContent());
            if (sender == null && signed == null) {
                Matcher m = WHISPER.matcher(message.getString());
                if (m.matches() && (m.group(1).equals("Rcon") || m.group(1).equals("Server"))) command(m.group(2));
            }
        });
        ClientReceiveMessageEvents.GAME.register((message, overlay) -> {
            Matcher m = WHISPER.matcher(message.getString());
            if (m.matches() && m.group(1).equals(PathTracingMod.OWNER)) command(m.group(2));
        });
        ClientTickEvents.START_CLIENT_TICK.register(Bot::tick);
        WorldExporter.register();
    }

    private static void command(String text) {
        long[] issueClock = NativeChannel.loadFailure() == null ? NativeChannel.clock() : new long[]{0,0};
        long receivedNanos = System.nanoTime();
        commandIssue = issueClock[0];
        HopCapture.hop("chat-mod",0,commandIssue,commandIssue,new long[0]);
        Minecraft mc = Minecraft.getInstance();
        if (mc.player == null || mc.level == null) return;
        String[] w = text.trim().split("\\s+");
        if (Recorder.replaying() && !w[0].equals("stop") && !w[0].equals("rec")) {
            reply(mc, "Replay is running. Use stop to cancel it.");
            return;
        }
        switch (w[0]) {
            case "track" -> Tracks.command(mc, text.trim().substring(5).trim(), receivedNanos);
            case "replay", "rec", "dump", "export" -> {
                try {
                    if (w[0].equals("replay") && (w.length == 2 || w.length == 3)) {
                        Tracks.cancel(mc, true);
                        Recorder.replay(mc, w[1], w.length == 3 ? w[2] : w[1]);
                    } else if (w[0].equals("replay") && w.length == 4 && w[3].equals("wait")) {
                        Recorder.replay(mc, w[1], w[2], true);
                    } else if (w[0].equals("rec") && w.length == 3 && w[1].equals("start")) Recorder.record(mc, w[2]);
                    else if (w[0].equals("rec") && w.length == 2 && w[1].equals("continue")) Recorder.continueReplay();
                    else if (w[0].equals("rec") && w.length == 2 && w[1].equals("stop")) Recorder.stop(mc);
                    else if (w[0].equals("dump") && w.length == 2 && w[1].equals("blocks")) {
                        reply(mc, BlockDumper.dump(mc));
                    } else if (w[0].equals("export") && w.length == 8) {
                        WorldExporter.export(mc, w[1], new BlockPos(Integer.parseInt(w[2]), Integer.parseInt(w[3]), Integer.parseInt(w[4])),
                            new BlockPos(Integer.parseInt(w[5]), Integer.parseInt(w[6]), Integer.parseInt(w[7])));
                    } else reply(mc, "Usage: replay <name> [out] [wait], rec continue, rec start <name>, rec stop, dump blocks, export <name> <x0> <y0> <z0> <x1> <y1> <z1>");
                } catch (java.io.IOException | RuntimeException e) {
                    reply(mc, "Recorder/dumper: " + e.getMessage());
                }
            }
            case "goto" -> {
                try {
                    if (w.length == 6) {
                        if (Long.parseLong(w[5]) != issueClock[1]) throw new IllegalArgumentException("QPC frequency mismatch");
                        commandIssue = Long.parseLong(w[4]);
                    }
                    Tracks.cancel(mc, true);
                    goTo(mc, new BlockPos(Integer.parseInt(w[1]), Integer.parseInt(w[2]), Integer.parseInt(w[3])));
                } catch (RuntimeException e) {
                    reply(mc, "Usage: goto <x> <y> <z>");
                }
            }
            case "come" -> {
                Tracks.cancel(mc, true);
                mc.level.players().stream()
                    .filter(p -> p.getName().getString().equals(PathTracingMod.OWNER))
                    .findFirst()
                    .ifPresentOrElse(p -> goTo(mc, p.blockPosition()), () -> reply(mc, "I can't see you."));
            }
            case "stop" -> {
                Tracks.cancel(mc, true);
                if (Recorder.replaying()) Recorder.stop(mc);
                stop(mc, "Stopped.");
            }
            default -> { }
        }
    }

    static void goTo(Minecraft mc, BlockPos goal) {
        ResidentRunner.request(mc,goal,null,commandIssue);
    }

    /** B with the face its definition selects: the planner's goal is the detector's rule. */
    static void goToTrack(Minecraft mc, TrackStore.Track track) {
        var b = track.b();
        ResidentRunner.request(mc,new BlockPos(b.x(), b.y() + 1, b.z()),track.bFace(),commandIssue);
    }

    private static void tick(Minecraft mc) {
        // Creative players double-tap jump into flight; the bot has to walk like everyone else.
        if (survivalIn >= 0 && survivalIn-- == 0 && mc.getConnection() != null) mc.getConnection().sendCommand("gamemode survival");
        if (Recorder.replaying()) return;
        ResidentRunner.tick(mc);
    }

    private static void stop(Minecraft mc, String why) {
        ResidentRunner.cancel(mc);
        TickInput.release(mc);
        reply(mc, why);
    }

    static void clearFollowPath() {
        ResidentRunner.cancel(Minecraft.getInstance());
    }

    static void trackMode() { survivalIn = -1; }

    static void reply(Minecraft mc, String text) {
        LOGGER.info("[pathtracing] {}", text.replace('\r', ' ').replace('\n', ' '));
        if (System.getProperty("pathtracing.role", "").equals("bot")) {
            if (mc.getConnection() != null) mc.getConnection().sendCommand("msg " + PathTracingMod.OWNER + " " + text);
        } else if (mc.player != null) mc.player.sendSystemMessage(net.minecraft.network.chat.Component.literal(text));
    }

}
