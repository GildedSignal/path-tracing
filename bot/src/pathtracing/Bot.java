package pathtracing;

import java.util.List;
import java.util.regex.Matcher;
import java.util.regex.Pattern;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents;
import net.fabricmc.fabric.api.client.message.v1.ClientReceiveMessageEvents;
import net.fabricmc.fabric.api.client.networking.v1.ClientPlayConnectionEvents;
import net.minecraft.client.Minecraft;
import net.minecraft.core.BlockPos;
import net.minecraft.world.phys.Vec3;
import pathtracing.planner.Planner;
import pathtracing.planner.StraightLine;

/**
 * The bot's side. Takes goals from the owner's whispers (goto x y z, come, stop), asks the planner for a
 * path, then walks it by holding the same movement keys a person would.
 */
final class Bot {
    private static final Planner PLANNER = new StraightLine();
    private static final Pattern WHISPER = Pattern.compile("(\\S+) whispers to you: (.*)");
    private static final float TURN_PER_TICK = 30;
    private static final double REACHED = 0.35;
    private static final int STUCK_TICKS = 60;

    private static List<BlockPos> path = List.of();
    private static int next;
    private static double bestDistance;
    private static int sinceProgress;
    private static int survivalIn = -1;

    static void register() {
        // Wait a second: commands sent in the join callback itself are dropped (no command tree yet).
        ClientPlayConnectionEvents.JOIN.register((handler, sender, mc) -> survivalIn = 20);
        ClientReceiveMessageEvents.CHAT.register((message, signed, sender, type, time) -> {
            if (sender != null && signed != null && sender.name().equals(PathTracingMod.OWNER)) command(signed.signedContent());
        });
        ClientReceiveMessageEvents.GAME.register((message, overlay) -> {
            Matcher m = WHISPER.matcher(message.getString());
            if (m.matches() && m.group(1).equals(PathTracingMod.OWNER)) command(m.group(2));
        });
        ClientTickEvents.START_CLIENT_TICK.register(Bot::tick);
        Recorder.register();
    }

    private static void command(String text) {
        Minecraft mc = Minecraft.getInstance();
        if (mc.player == null || mc.level == null) return;
        String[] w = text.trim().split("\\s+");
        if (Recorder.replaying() && !w[0].equals("stop") && !w[0].equals("rec")) {
            reply(mc, "Replay is running. Use stop to cancel it.");
            return;
        }
        switch (w[0]) {
            case "replay", "rec", "dump" -> {
                try {
                    if (w[0].equals("replay") && (w.length == 2 || w.length == 3)) {
                        Recorder.replay(mc, w[1], w.length == 3 ? w[2] : w[1]);
                    } else if (w[0].equals("rec") && w.length == 3 && w[1].equals("start")) Recorder.record(mc, w[2]);
                    else if (w[0].equals("rec") && w.length == 2 && w[1].equals("stop")) Recorder.stop(mc);
                    else if (w[0].equals("dump") && w.length == 2 && w[1].equals("blocks")) {
                        reply(mc, BlockDumper.dump(mc));
                    } else reply(mc, "Usage: replay <name> [out], rec start <name>, rec stop, dump blocks");
                } catch (java.io.IOException | RuntimeException e) {
                    reply(mc, "Recorder/dumper: " + e.getMessage());
                }
            }
            case "goto" -> {
                try {
                    goTo(mc, new BlockPos(Integer.parseInt(w[1]), Integer.parseInt(w[2]), Integer.parseInt(w[3])));
                } catch (RuntimeException e) {
                    reply(mc, "Usage: goto <x> <y> <z>");
                }
            }
            case "come" -> mc.level.players().stream()
                .filter(p -> p.getName().getString().equals(PathTracingMod.OWNER))
                .findFirst()
                .ifPresentOrElse(p -> goTo(mc, p.blockPosition()), () -> reply(mc, "I can't see you."));
            case "stop" -> {
                if (Recorder.replaying()) Recorder.stop(mc);
                stop(mc, "Stopped.");
            }
            default -> { }
        }
    }

    private static void goTo(Minecraft mc, BlockPos goal) {
        BlockPos start = mc.player.blockPosition();
        long t0 = System.nanoTime();
        List<BlockPos> planned = PLANNER.plan(mc.level, start, goal);
        double ms = (System.nanoTime() - t0) / 1e6;
        if (planned == null || planned.isEmpty()) {
            stop(mc, String.format("No path to %s (%.1f ms).", goal.toShortString(), ms));
            return;
        }
        path = planned;
        next = 0;
        bestDistance = Double.MAX_VALUE;
        sinceProgress = 0;
        reply(mc, String.format("Going to %s: %d steps, planned in %.1f ms.", goal.toShortString(), planned.size(), ms));
    }

    private static void tick(Minecraft mc) {
        // Creative players double-tap jump into flight; the bot has to walk like everyone else.
        if (survivalIn >= 0 && survivalIn-- == 0 && mc.getConnection() != null) mc.getConnection().sendCommand("gamemode survival");
        if (Recorder.replaying()) return;
        if (path.isEmpty()) return;
        var player = mc.player;
        if (player == null) {
            path = List.of();
            return;
        }
        Vec3 pos = player.position();
        BlockPos target = path.get(next);
        double dx = target.getX() + 0.5 - pos.x;
        double dz = target.getZ() + 0.5 - pos.z;
        double distance = Math.sqrt(dx * dx + dz * dz);

        if (distance < REACHED && Math.abs(pos.y - target.getY()) < 1) {
            if (++next == path.size()) {
                stop(mc, "Arrived at " + target.toShortString() + ".");
                return;
            }
            bestDistance = Double.MAX_VALUE;
            sinceProgress = 0;
            return;
        }
        if (distance < bestDistance - 0.05) {
            bestDistance = distance;
            sinceProgress = 0;
        } else if (++sinceProgress > STUCK_TICKS) {
            stop(mc, "Stuck at " + player.blockPosition().toShortString() + " heading for " + target.toShortString() + ".");
            return;
        }

        // Minecraft yaw: 0 faces +Z, 90 faces -X.
        float want = (float) Math.toDegrees(Math.atan2(-dx, dz));
        float turn = wrapDegrees(want - player.getYRot());
        player.setYRot(player.getYRot() + Math.clamp(turn, -TURN_PER_TICK, TURN_PER_TICK));
        boolean jump = player.onGround() && (player.horizontalCollision || target.getY() > pos.y + 0.5);
        keys(mc, Math.abs(turn) < 60, jump);
    }

    private static void stop(Minecraft mc, String why) {
        path = List.of();
        keys(mc, false, false);
        reply(mc, why);
    }

    static void clearFollowPath() {
        path = List.of();
    }

    private static void keys(Minecraft mc, boolean forward, boolean jump) {
        mc.options.keyUp.setDown(forward);
        mc.options.keyJump.setDown(jump);
    }

    static void reply(Minecraft mc, String text) {
        if (mc.getConnection() != null) mc.getConnection().sendCommand("msg " + PathTracingMod.OWNER + " " + text);
    }

    private static float wrapDegrees(float degrees) {
        float d = degrees % 360;
        return d >= 180 ? d - 360 : d < -180 ? d + 360 : d;
    }
}
